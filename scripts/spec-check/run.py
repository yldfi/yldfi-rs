#!/usr/bin/env python3
"""Weekly upstream API spec check for the yldfi-rs client crates.

Downloads each provider's published OpenAPI/Swagger spec (or GraphQL schema)
from specs.txt, reduces it to its API surface (operations, parameters,
request and response fields) and compares that with the committed snapshot
in snapshots/<provider>.json:

  CHANGED    the surface differs from the snapshot. Changes are split into
             breaking (operation, parameter or field removed, parameter newly
             required, operation deprecated) and additions (new operations,
             parameters or fields), and operations a crate calls are marked.
  UNCHANGED  the surface matches the snapshot
  FAIL       the spec URL is gone (404/410, DNS failure) or no longer parses
  WARN       transient fetch problem: rate limit, 5xx, timeout
  SKIP       a URL placeholder's environment variable is not set
  NEW        no snapshot yet (written with --update)

After reviewing a change (and updating the crate if needed), accept it with
--update, which rewrites the snapshots. Exit status is 1 if anything is
CHANGED or FAIL.

Usage: run.py [--specs FILE] [--only REGEX] [--update] [--json OUT] [--summary OUT]
"""
import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

try:
    import yaml
except ImportError:  # JSON specs still work without PyYAML
    yaml = None

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
TIMEOUT_S = 60
RETRY_DELAY_S = 10
MAX_DEPTH = 3
METHODS = ("get", "put", "post", "delete", "patch", "head", "options")
USER_AGENT = "yldfi-rs-spec-check (+https://github.com/yldfi/yldfi-rs)"

GRAPHQL_INTROSPECTION = """
query { __schema {
  queryType { name } mutationType { name }
  types { kind name
    fields(includeDeprecated: true) { name isDeprecated args { name type { ...T } } type { ...T } }
    inputFields { name type { ...T } }
    enumValues(includeDeprecated: true) { name isDeprecated }
  }
} }
fragment T on __Type { kind name ofType { kind name ofType { kind name ofType { kind name } } } }
"""


def parse_specs(path):
    specs = []
    for n, raw in enumerate(Path(path).read_text().splitlines(), 1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = [p.strip() for p in line.split("|")]
        if len(parts) != 4 or parts[2] not in ("openapi", "mintlify", "readme", "graphql"):
            sys.exit(f"{path}:{n}: expected 'provider | crate | openapi|mintlify|readme|graphql | url'")
        provider, crate, kind, url = parts
        specs.append({"line": n, "provider": provider, "crate": crate, "kind": kind, "url": url})
    return specs


def expand_url(url):
    """Substitute ${VAR} from the environment; returns (url, missing vars)."""
    missing = [v for v in re.findall(r"\$\{(\w+)\}", url) if not os.environ.get(v)]
    return re.sub(r"\$\{(\w+)\}", lambda m: os.environ.get(m.group(1), ""), url), missing


def redact(text, url_template=""):
    for var in re.findall(r"\$\{(\w+)\}", url_template):
        if os.environ.get(var):
            text = text.replace(os.environ[var], "***")
    return text


# --- Fetching


class FetchError(Exception):
    def __init__(self, status, detail):
        super().__init__(detail)
        self.status = status
        self.detail = detail


def classify_http_error(code):
    if code in (404, 410):
        return "FAIL", f"spec URL gone (HTTP {code})"
    if code in (401, 403):
        return "FAIL", f"spec now requires auth (HTTP {code})"
    if code == 429 or code >= 500:
        return "WARN", f"HTTP {code}"
    return "FAIL", f"HTTP {code}"


def http_get(url, data=None):
    headers = {"User-Agent": USER_AGENT, "Accept": "application/json, application/yaml, text/yaml, */*"}
    if data is not None:
        headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=TIMEOUT_S) as resp:
            return resp.read().decode("utf-8")
    except urllib.error.HTTPError as e:
        raise FetchError(*classify_http_error(e.code)) from None
    except urllib.error.URLError as e:
        reason = str(e.reason)
        if re.search(r"name or service not known|nodename nor servname|no address|getaddrinfo", reason, re.I):
            raise FetchError("FAIL", f"DNS failure: host gone ({reason})") from None
        raise FetchError("WARN", f"connection error ({reason})") from None
    except TimeoutError:
        raise FetchError("WARN", f"timed out after {TIMEOUT_S}s") from None


def parse_document(text):
    try:
        return json.loads(text)
    except ValueError:
        pass
    if yaml is None:
        raise FetchError("FAIL", "not JSON, and PyYAML is not installed to try YAML")
    try:
        doc = yaml.safe_load(text)
    except yaml.YAMLError as e:
        raise FetchError("FAIL", f"spec no longer parses: {str(e).splitlines()[0]}") from None
    if not isinstance(doc, dict):
        raise FetchError("FAIL", "spec no longer parses: not a JSON/YAML object (HTML page?)")
    return doc


def fetch(spec, url):
    if spec["kind"] == "graphql":
        body = json.dumps({"query": GRAPHQL_INTROSPECTION}).encode()
        doc = parse_document(http_get(url, data=body))
        if not isinstance(doc.get("data"), dict) or "__schema" not in doc["data"]:
            raise FetchError("FAIL", "introspection returned no schema: " + json.dumps(doc.get("errors"))[:150])
        return graphql_surface(doc["data"]["__schema"])
    if spec["kind"] in ("mintlify", "readme"):
        return docs_index_surface(url, spec["kind"])
    return openapi_surface(fetch_openapi(url))


def fetch_openapi(url):
    doc = parse_document(http_get(url))
    if "openapi" not in doc and "swagger" not in doc:
        raise FetchError("FAIL", "spec no longer parses: no 'openapi' or 'swagger' key")
    return doc


def readme_page_spec(text):
    """A ReadMe .md page embeds its spec in a json block under "# OpenAPI definition"."""
    m = re.search(r"#\s*OpenAPI definition\s*```(?:json)?\s*\n(.*?)\n```", text, re.S)
    if not m:
        return None
    doc = parse_document(m.group(1))
    return doc if "openapi" in doc or "swagger" in doc else None


def docs_index_surface(base, kind):
    """Docs sites with one spec per endpoint page, listed in llms.txt, merged:
    mintlify (e.g. Etherscan) serves each page's spec at /openapi/<page>.json;
    readme (e.g. GoPlus) embeds it in the page's .md. Pages without a spec
    (guides) are skipped."""
    base = base.rstrip("/")
    index = http_get(f"{base}/llms.txt")
    section = "api-reference/endpoint" if kind == "mintlify" else "reference"
    pages = sorted(set(re.findall(r"(?:" + re.escape(base) + r")?/(" + section + r"/[A-Za-z0-9_/-]+)\.md", index)))
    if not pages:
        raise FetchError("FAIL", f"llms.txt lists no {section} pages")
    per_page, version = [], ""
    for page in pages:
        if kind == "mintlify":
            doc = fetch_openapi(f"{base}/openapi/{page}.json")
        else:
            doc = readme_page_spec(http_get(f"{base}/{page}.md"))
            if doc is None:
                continue
        surface = openapi_surface(doc, discriminate=True)
        per_page.append((page.rsplit("/", 1)[-1], surface["operations"]))
        version = version or surface["version"]
    if not per_page:
        raise FetchError("FAIL", "no endpoint page has a spec any more")
    # Several pages can document variants of one operation (txlist and
    # advanced-filter-txlist); keep each variant, named after its page.
    seen = {}
    for _, page_ops in per_page:
        for key in page_ops:
            seen[key] = seen.get(key, 0) + 1
    ops = {}
    for name, page_ops in per_page:
        for key, op in page_ops.items():
            ops[f"{key} ({name})" if seen[key] > 1 else key] = op
    return {"version": version, "operations": dict(sorted(ops.items()))}


# --- OpenAPI surface


def resolve(doc, node, seen):
    """Follow a local $ref; returns (node, ref name or None)."""
    ref = node.get("$ref") if isinstance(node, dict) else None
    if not isinstance(ref, str) or not ref.startswith("#/"):
        return node, None
    if ref in seen:
        return {}, ref
    target = doc
    for part in ref[2:].split("/"):
        part = part.replace("~1", "/").replace("~0", "~")
        target = target.get(part, {}) if isinstance(target, dict) else {}
    return target, ref


def schema_fields(doc, schema, prefix="", depth=0, seen=frozenset()):
    """Flatten a schema into sorted 'a.b[].c' field paths; '*' marks required."""
    schema, ref = resolve(doc, schema, seen)
    if ref:
        seen = seen | {ref}
    if not isinstance(schema, dict) or depth > MAX_DEPTH:
        return set()
    fields = set()
    for key in ("allOf", "oneOf", "anyOf"):
        for sub in schema.get(key) or []:
            fields |= schema_fields(doc, sub, prefix, depth, seen)
    if schema.get("type") == "array" or "items" in schema:
        return fields | schema_fields(doc, schema.get("items") or {}, prefix + "[]", depth, seen)
    required = set(schema.get("required") or []) if isinstance(schema.get("required"), list) else set()
    for name, sub in (schema.get("properties") or {}).items():
        path = f"{prefix}.{name}" if prefix else name
        fields.add(path + ("*" if name in required else ""))
        fields |= schema_fields(doc, sub, path, depth + 1, seen)
    return fields


def media_schema(content):
    if not isinstance(content, dict) or not content:
        return None
    for mt in ("application/json", "*/*"):
        if mt in content:
            return (content[mt] or {}).get("schema")
    return (next(iter(content.values())) or {}).get("schema")


def openapi_surface(doc, discriminate=False):
    """With discriminate, required query parameters that have a default (module,
    action) become part of the operation key, for APIs that put every endpoint
    on one path."""
    ops = {}
    for path, item in (doc.get("paths") or {}).items():
        item, _ = resolve(doc, item, frozenset())
        if not isinstance(item, dict):
            continue
        for method in METHODS:
            op = item.get(method)
            if not isinstance(op, dict):
                continue
            params, request, fixed = set(), set(), []
            for p in (item.get("parameters") or []) + (op.get("parameters") or []):
                p, _ = resolve(doc, p, frozenset())
                if not isinstance(p, dict) or "name" not in p:
                    continue
                if p.get("in") == "body":  # Swagger 2
                    request |= schema_fields(doc, p.get("schema") or {})
                    continue
                params.add(f"{p.get('in')}:{p['name']}" + ("*" if p.get("required") else ""))
                default = (p.get("schema") or {}).get("default")
                if discriminate and p.get("in") == "query" and p.get("required") and isinstance(default, str):
                    fixed.append(f"{p['name']}={default}")
            body, _ = resolve(doc, op.get("requestBody") or {}, frozenset())
            if isinstance(body, dict):
                request |= schema_fields(doc, media_schema(body.get("content")) or {})
            response = set()
            responses = op.get("responses") or {}
            ok = sorted(c for c in responses if str(c).startswith("2")) or sorted(c for c in responses if str(c) == "default")
            if ok:
                resp, _ = resolve(doc, responses[ok[0]] or {}, frozenset())
                if isinstance(resp, dict):
                    schema = resp.get("schema") if "schema" in resp else media_schema(resp.get("content"))
                    response = schema_fields(doc, schema or {})
            ops[f"{method.upper()} {path}" + ("?" + "&".join(fixed) if fixed else "")] = {
                "deprecated": bool(op.get("deprecated")),
                "params": sorted(params),
                "request": sorted(request),
                "response": sorted(response),
            }
    version = str((doc.get("info") or {}).get("version", ""))
    return {"version": version, "operations": dict(sorted(ops.items()))}


# --- GraphQL surface


def gql_type(t):
    if not t:
        return "?"
    if t["kind"] == "NON_NULL":
        return gql_type(t.get("ofType")) + "!"
    if t["kind"] == "LIST":
        return f"[{gql_type(t.get('ofType'))}]"
    return t.get("name") or "?"


def graphql_surface(schema):
    roots = {(schema.get("queryType") or {}).get("name"): "query", (schema.get("mutationType") or {}).get("name"): "mutation"}
    ops = {}
    for t in schema.get("types") or []:
        name = t.get("name") or ""
        if name.startswith("__") or t.get("kind") == "SCALAR":
            continue
        if name in roots and roots[name]:
            for f in t.get("fields") or []:
                ops[f"{roots[name]} {f['name']}"] = {
                    "deprecated": bool(f.get("isDeprecated")),
                    "params": sorted(f"{a['name']}: {gql_type(a['type'])}" for a in f.get("args") or []),
                    "request": [],
                    "response": [gql_type(f["type"])],
                }
            continue
        fields = t.get("fields") or t.get("inputFields") or []
        members = [f"{f['name']}: {gql_type(f['type'])}" for f in fields]
        members += [v["name"] for v in t.get("enumValues") or []]
        ops[f"{t['kind'].lower()} {name}"] = {"deprecated": False, "params": [], "request": [], "response": sorted(members)}
    return {"version": "", "operations": dict(sorted(ops.items()))}


# --- Which operations the crates call


def path_segments(path):
    path = path.split("?", 1)[0].strip("/")
    # "/derivatives{}" is a query string appended with format!, not part of the path.
    segs = [re.sub(r"(?<=[^/{}])(\{[^}]*\})+$", "", s) for s in path.split("/") if s]
    return [("{}" if re.fullmatch(r"(\{[^}]*\})+", s) else s) for s in segs]


def crate_paths(crate):
    """Path-like string literals in a crate's sources ("/api/v1/orders/{uid}")."""
    src = REPO / "crates" / crate / "src"
    found = set()
    for f in src.rglob("*.rs") if src.is_dir() else []:
        for lit in re.findall(r'"(/[A-Za-z0-9_{}./:-]*[A-Za-z][A-Za-z0-9_{}./:-]*)(?:\?[^"]*)?"', f.read_text()):
            segs = path_segments(lit)
            if any(s != "{}" for s in segs):
                found.add(tuple(segs))
    return found


ACTION_RE = re.compile(r"module=([a-z]+)&action=([A-Za-z_]+)")


def crate_actions(crate, library_calls=HERE / "library-calls.txt"):
    """Etherscan-style "module=…&action=…" pairs a crate calls: from its own
    URL literals, plus calls made through a library (library-calls.txt)."""
    src = REPO / "crates" / crate / "src"
    found = set()
    for f in src.rglob("*.rs") if src.is_dir() else []:
        found.update(f"module={m}&action={a}" for m, a in ACTION_RE.findall(f.read_text()))
    if Path(library_calls).exists():
        for raw in Path(library_calls).read_text().splitlines():
            parts = [x.strip() for x in raw.split("|")]
            if not raw.strip().startswith("#") and len(parts) == 3 and parts[0] == crate:
                found.add(parts[1])
    return found


def action_matches(key, action):
    query = key.partition("?")[2].split(" ", 1)[0]
    return query == action


def segments_match(a, b, strict=False):
    """Suffix match, so a crate path without the spec's base path still matches.

    A placeholder matches any segment, or with strict only another placeholder.
    """
    n = min(len(a), len(b))
    pairs = list(zip(a[-n:], b[-n:])) if n else []
    if not pairs or not all(x == y or (not strict and "{}" in (x, y)) for x, y in pairs):
        return False
    return any(x == y != "{}" for x, y in pairs)


def used_operations(surface, paths):
    """Spec operations whose path a crate calls (any method: crates rarely spell it next to the path)."""
    ops = [(k, path_segments(k.partition(" ")[2])) for k in surface["operations"] if k.partition(" ")[0].lower() in METHODS]
    used = set()
    for p in paths:
        # Prefer placeholder-for-placeholder matches, so "/coins/{id}" doesn't also claim "/coins/list".
        hits = [k for k, segs in ops if segments_match(segs, p, strict=True)] or [k for k, segs in ops if segments_match(segs, p)]
        used.update(hits)
    return used


def unmatched_paths(surface, paths):
    spec_paths = [path_segments(k.partition(" ")[2]) for k in surface["operations"]]
    return sorted("/" + "/".join(p) for p in paths if not any(segments_match(s, p) for s in spec_paths))


# --- Diffing


def diff_surfaces(old, new, used=()):
    """Returns a list of {op, kind: breaking|addition, used, what}."""
    changes = []

    def add(op, kind, what):
        changes.append({"op": op, "kind": kind, "used": op in used, "what": what})

    old_ops, new_ops = old.get("operations", {}), new.get("operations", {})
    for op in sorted(old_ops.keys() - new_ops.keys()):
        add(op, "breaking", "removed")
    for op in sorted(new_ops.keys() - old_ops.keys()):
        add(op, "addition", "added")
    for op in sorted(old_ops.keys() & new_ops.keys()):
        a, b = old_ops[op], new_ops[op]
        if b.get("deprecated") and not a.get("deprecated"):
            add(op, "breaking", "now deprecated")
        for section, label in (("params", "parameter"), ("request", "request field"), ("response", "response field")):
            before = {x.rstrip("*"): x.endswith("*") for x in a.get(section, [])}
            after = {x.rstrip("*"): x.endswith("*") for x in b.get(section, [])}
            removed = sorted(before.keys() - after.keys())
            added = sorted(after.keys() - before.keys())
            now_required = sorted(k for k in before.keys() & after.keys() if after[k] and not before[k])
            if removed:
                add(op, "breaking", f"{label}s removed: " + ", ".join(f"`{x}`" for x in removed))
            if now_required and section != "response":
                add(op, "breaking", f"{label}s now required: " + ", ".join(f"`{x}`" for x in now_required))
            if added:
                # A new required input breaks existing callers; a new output field doesn't.
                req = [x for x in added if after[x] and section != "response"]
                if req:
                    add(op, "breaking", f"new required {label}s: " + ", ".join(f"`{x}`" for x in req))
                rest = [x for x in added if x not in req]
                if rest:
                    add(op, "addition", f"{label}s added: " + ", ".join(f"`{x}`" for x in rest))
    return changes


def snapshot_path(provider):
    return HERE / "snapshots" / (re.sub(r"[^A-Za-z0-9_.-]", "-", provider) + ".json")


def check(spec, update=False):
    url, missing = expand_url(spec["url"])
    result = {"provider": spec["provider"], "crate": spec["crate"], "changes": [], "version": "", "surface": None,
              "unmatched_actions": []}
    if missing:
        return {**result, "status": "SKIP", "detail": "not configured: " + ", ".join(missing)}
    surface, status, detail = None, "FAIL", ""
    for attempt in (1, 2):
        try:
            surface = fetch(spec, url)
            break
        except FetchError as e:
            status, detail = e.status, redact(e.detail, spec["url"])
            if attempt == 1:
                time.sleep(RETRY_DELAY_S)
    if surface is None:
        return {**result, "status": status, "detail": detail}
    surface = {"provider": spec["provider"], "url": spec["url"], **surface}
    result["version"] = surface["version"]
    # Etherscan-style operations share one path, so they're matched by action instead.
    pathful = spec["kind"] in ("openapi", "readme")
    result["surface"] = surface if pathful else None
    paths = crate_paths(spec["crate"]) if pathful else set()
    actions = crate_actions(spec["crate"]) if spec["kind"] == "mintlify" else set()
    result["unmatched_actions"] = sorted(a for a in actions if not any(action_matches(k, a) for k in surface["operations"]))
    snap = snapshot_path(spec["provider"])
    old = json.loads(snap.read_text()) if snap.exists() else None
    n_ops = len(surface["operations"])
    if old is None:
        status, detail = "NEW", f"{n_ops} operations, no snapshot yet"
    else:
        used = used_operations(surface, paths) | used_operations(old, paths)
        used |= {k for s in (old, surface) for k in s["operations"] if any(action_matches(k, a) for a in actions)}
        result["changes"] = diff_surfaces(old, surface, used)
        if result["changes"] or old.get("url") != surface["url"]:
            breaking = sum(c["kind"] == "breaking" for c in result["changes"])
            used = sum(c["used"] for c in result["changes"])
            version = f", version {old.get('version') or '?'} → {surface['version'] or '?'}" if old.get("version") != surface["version"] else ""
            status, detail = "CHANGED", f"{breaking} breaking, {len(result['changes']) - breaking} additions, {used} on endpoints the crate calls{version}"
            if old.get("url") != surface["url"]:
                detail += "; spec URL changed"
        else:
            status, detail = "UNCHANGED", f"{n_ops} operations"
    if update and status in ("NEW", "CHANGED"):
        snap.parent.mkdir(exist_ok=True)
        snap.write_text(json.dumps(surface, indent=1, sort_keys=False) + "\n")
        detail += " (snapshot updated)"
    return {**result, "status": status, "detail": detail}


def crate_unmatched(results):
    """Crate paths missing from every fetched spec of that crate (a crate may span several specs)."""
    by_crate = {}
    for r in results:
        if r.get("surface"):
            by_crate.setdefault(r["crate"], []).append(r["surface"])
    out = {}
    for crate, surfaces in sorted(by_crate.items()):
        merged = {"operations": {k: v for s in surfaces for k, v in s["operations"].items()}}
        paths = unmatched_paths(merged, crate_paths(crate))
        if paths:
            out[crate] = paths
    return out


def markdown(results, max_changes=40):
    icon = {"CHANGED": "🔶", "FAIL": "❌", "WARN": "⚠️", "SKIP": "⏭️", "NEW": "🆕", "UNCHANGED": "✅"}
    order = {"FAIL": 0, "CHANGED": 1, "WARN": 2, "NEW": 3, "SKIP": 4, "UNCHANGED": 5}
    counts = {s: sum(r["status"] == s for r in results) for s in order}
    lines = ["## Weekly API spec check", "", " · ".join(f"{icon[k]} {k} {v}" for k, v in counts.items() if v), "",
             "| | Provider | Crate | Detail |", "|---|---|---|---|"]
    for r in sorted(results, key=lambda r: (order[r["status"]], r["provider"])):
        lines.append(f"| {icon[r['status']]} | {r['provider']} | `{r['crate']}` | {r['detail'].replace('|', '/')} |")
    for r in sorted(results, key=lambda r: r["provider"]):
        if not r["changes"]:
            continue
        lines += ["", f"### {r['provider']} (`{r['crate']}`)", ""]
        ranked = sorted(r["changes"], key=lambda c: (not c["used"], c["kind"] != "breaking", c["op"]))
        for c in ranked[:max_changes]:
            tag = ("**breaking**" if c["kind"] == "breaking" else "addition") + (" · **used by crate**" if c["used"] else "")
            lines.append(f"- {tag}: `{c['op']}` {c['what']}")
        if len(ranked) > max_changes:
            lines.append(f"- … and {len(ranked) - max_changes} more (see the run's JSON artifact)")
    unmatched = crate_unmatched(results)
    for r in results:
        if r.get("unmatched_actions"):
            unmatched.setdefault(r["crate"], []).extend(r["unmatched_actions"])
    if unmatched:
        lines += ["", "<details><summary>Paths the crates call that the current spec doesn't list "
                  "(heuristic: undocumented, renamed or removed endpoints)</summary>", ""]
        for crate, paths in unmatched.items():
            lines.append(f"- `{crate}`: " + ", ".join(f"`{p}`" for p in paths))
        lines += ["", "</details>"]
    return "\n".join(lines) + "\n"


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--specs", default=str(HERE / "specs.txt"))
    ap.add_argument("--only", help="regex: check only providers matching this")
    ap.add_argument("--update", action="store_true", help="write new/changed surfaces to snapshots/")
    ap.add_argument("--json")
    ap.add_argument("--summary", help="append a markdown summary here (e.g. $GITHUB_STEP_SUMMARY)")
    args = ap.parse_args()

    results = []
    for spec in parse_specs(args.specs):
        if args.only and not re.search(args.only, spec["provider"]):
            continue
        r = check(spec, update=args.update)
        results.append(r)
        print(f"{r['status']:<9} {r['provider']:<18} {r['detail'][:150]}", flush=True)

    counts = {s: sum(r["status"] == s for r in results) for s in ("CHANGED", "FAIL", "WARN", "NEW", "SKIP", "UNCHANGED")}
    print("\n" + "  ".join(f"{k}={v}" for k, v in counts.items()))
    if args.json:
        slim = [{k: v for k, v in r.items() if k != "surface"} for r in results]
        Path(args.json).write_text(json.dumps({"counts": counts, "results": slim, "unmatched": crate_unmatched(results)}, indent=2))
    if args.summary:
        with open(args.summary, "a") as f:
            f.write(markdown(results))
    if args.update:
        return 1 if counts["FAIL"] else 0
    return 1 if counts["FAIL"] or counts["CHANGED"] else 0


if __name__ == "__main__":
    sys.exit(main())
