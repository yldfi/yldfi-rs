#!/usr/bin/env python3
"""Print a draft Rust client method, options struct and response types for
one operation of an OpenAPI spec.

The output is a starting point to review, rename and trim before adding it
to a crate, not code to commit as is: names follow the spec's schema titles
and property names, and every field is an Option.

Usage: scaffold.py SPEC_URL_OR_FILE OPERATION_ID
  e.g. scaffold.py https://raw.githubusercontent.com/coingecko/coingecko-api-oas/main/pro-api.json rwas-list
"""
import json
import re
import sys

from examples import resolve
from run import fetch_openapi

# Strict and reserved Rust keywords (2021 edition and later)
RUST_KEYWORDS = set("""
    as async await break const continue crate dyn else enum extern false fn for gen if impl in
    let loop match mod move mut pub ref return self Self static struct super trait true type
    unsafe use where while abstract become box do final macro override priv try typeof unsized
    virtual yield
""".split())


def pascal(name):
    return "".join(p[:1].upper() + p[1:] for p in re.split(r"[^A-Za-z0-9]+", name) if p)


def ident(name):
    """A Rust identifier for `name`: snake_case, `_` appended to keywords and
    prefixed to names starting with a digit."""
    out = snake(name) or "_"
    if out in RUST_KEYWORDS:
        out += "_"
    if out[0].isdigit():
        out = "_" + out
    return out


def kind_of(schema):
    """The schema's type; OpenAPI 3.1 lists like ["string", "null"] give the
    first non-null entry."""
    kind = schema.get("type") if isinstance(schema, dict) else None
    if isinstance(kind, list):
        kind = next((k for k in kind if k != "null"), None)
    return kind


def snake(name):
    name = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", name)
    return re.sub(r"[^A-Za-z0-9]+", "_", name).strip("_").lower()


class Scaffold:
    def __init__(self, doc):
        self.doc = doc
        self.structs = []
        self.seen = set()

    def rust_type(self, schema, hint):
        raw = schema
        schema = resolve(self.doc, schema)
        if not isinstance(schema, dict):
            return "serde_json::Value"
        alts = schema.get("oneOf") or schema.get("anyOf")
        if alts:
            return "serde_json::Value"
        kind = kind_of(schema)
        if kind == "array":
            return f"Vec<{self.rust_type(schema.get('items', {}), hint)}>"
        if kind == "object" or "properties" in schema:
            if not schema.get("properties"):
                extra = schema.get("additionalProperties")
                inner = self.rust_type(extra, hint) if isinstance(extra, dict) else "serde_json::Value"
                return f"std::collections::HashMap<String, {inner}>"
            name = pascal(schema.get("title") or (raw.get("$ref", "").rsplit("/", 1)[-1] if isinstance(raw, dict) else "") or hint)
            self.struct(name, schema)
            return name
        return {"integer": "i64", "number": "f64", "string": "String", "boolean": "bool"}.get(kind, "serde_json::Value")

    def struct(self, name, schema):
        if name in self.seen:
            return
        self.seen.add(name)
        lines = ["#[derive(Debug, Clone, Deserialize, Serialize)]", f"pub struct {name} {{"]
        for prop, sub in schema.get("properties", {}).items():
            desc = resolve(self.doc, sub).get("description", "") if isinstance(resolve(self.doc, sub), dict) else ""
            if desc:
                lines.append(f"    /// {desc.splitlines()[0]}")
            field = ident(prop)
            if field != prop:
                lines.append(f'    #[serde(rename = "{prop}")]')
            lines.append(f"    pub {field}: Option<{self.rust_type(sub, name + pascal(prop))}>,")
        lines.append("}")
        self.structs.append("\n".join(lines))


def find(doc, op_id):
    for path, ops in doc.get("paths", {}).items():
        op = ops.get("get")
        if op and op.get("operationId") == op_id:
            return path, op
    raise SystemExit(f"no GET operation {op_id!r}")


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    src, op_id = argv[1], argv[2]
    if src.startswith("http"):
        doc = fetch_openapi(src)
    else:
        with open(src, encoding="utf-8") as f:
            doc = json.load(f)
    path, op = find(doc, op_id)
    s = Scaffold(doc)
    params = [resolve(doc, p) for p in op.get("parameters", [])]
    path_params = [p for p in params if p.get("in") == "path"]
    query = [p for p in params if p.get("in") == "query"]

    media = op.get("responses", {}).get("200", {}).get("content", {}).get("application/json", {})
    ret = s.rust_type(media.get("schema", {}), pascal(op_id) + "Response")
    method = snake(op_id)
    opts = pascal(op_id) + "Options"

    out = [f"// Draft for GET {path} ({op_id}). Review, rename and trim before use.", ""]
    if query:
        out += ["#[derive(Debug, Clone, Default)]", f"pub struct {opts} {{"]
        for p in query:
            desc = (p.get("description") or "").splitlines()
            note = " (required)" if p.get("required") else ""
            if desc or note:
                out.append(f"    /// {(desc[0].strip() if desc else '')}{note}".replace("///  ", "/// "))
            ty = {"integer": "u64", "number": "f64", "boolean": "bool"}.get(kind_of(p.get("schema", {})), "String")
            out.append(f"    pub {ident(p['name'])}: Option<{ty}>,")
        out += ["}", "", f"impl {opts} {{", "    #[must_use]", "    pub fn to_query_string(&self) -> String {",
                "        let mut params = Vec::new();"]
        for p in query:
            f = ident(p["name"])
            ty = kind_of(p.get("schema", {}))
            val = f"url::form_urlencoded::byte_serialize(v.as_bytes()).collect::<String>()" if ty not in ("integer", "number", "boolean") else "v"
            out.append(f'        if let Some(ref v) = self.{f} {{ params.push(format!("{p["name"]}={{}}", {val})); }}')
        out += ['        if params.is_empty() { String::new() } else { format!("?{}", params.join("&")) }', "    }", "}", ""]
    out += s.structs[::-1]
    args = "".join(f", {ident(p['name'])}: &str" for p in path_params)
    if query:
        args += f", options: &{opts}"
    rust_path = re.sub(r"\{([^}]+)\}", lambda m: "{" + ident(m.group(1)) + "}", path)
    summary = op.get("summary", op_id)
    out += ["", f"    /// {summary}", f"    pub async fn {method}(&self{args}) -> Result<{ret}> {{"]
    suffix = "{}" if query else ""
    fmt_args = ", options.to_query_string()" if query else ""
    out += [f'        let path = format!("{rust_path}{suffix}"{fmt_args});', "        self.client.get(&path).await", "    }"]
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
