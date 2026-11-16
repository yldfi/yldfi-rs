#!/usr/bin/env python3
"""Extract example responses from an OpenAPI spec into one JSON file.

A client crate's tests deserialize each response into the crate's response
type, so a field whose type no longer matches the provider's spec fails
`cargo test` instead of failing at runtime.

The output maps each operation (by operationId, or "METHOD /path" when it
has none) to its path, its method when not GET, its server when the spec
gives the operation its own, and up to three responses
for its first 2xx JSON body:

  example   the spec's own example
  sample    built from the schema: every property present, nullable ones
            included, with a value of the declared type; for oneOf/anyOf
            the first alternative
  sample2   the same, taking the last oneOf/anyOf alternative (only when it
            differs from sample)

Examples often leave optional fields null; the samples make sure every
declared type is exercised. Sorted by operationId so refreshes diff cleanly.

Usage: examples.py SPEC_URL OUT_FILE
  e.g. examples.py https://raw.githubusercontent.com/coingecko/coingecko-api-oas/main/pro-api.json \\
           crates/cgko/tests/spec_examples.json
"""
import json
import sys

from run import fetch_openapi


MAX_SAMPLE_DEPTH = 12


def resolve(doc, schema):
    while isinstance(schema, dict) and "$ref" in schema:
        node = doc
        for part in schema["$ref"].lstrip("#/").split("/"):
            node = node[part]
        schema = node
    return schema


SCALARS = {"integer", "number", "string", "boolean"}


def is_scalar(doc, schema):
    schema = resolve(doc, schema)
    return isinstance(schema, dict) and schema.get("type") in SCALARS


# Marks a tuple whose shape the spec does not give; never written out
UNKNOWN_TUPLE = object()


def known(value):
    return [] if value is UNKNOWN_TUPLE else value


def sample(doc, schema, pick, example=None, depth=0):
    """A value matching `schema`. `pick` chooses among oneOf/anyOf branches.

    `example` is the matching part of the spec's example. Arrays whose items
    may be any of several scalar types are positional tuples such as
    [timestamp, value], which the schema cannot describe, so they take the
    example's value instead.
    """
    schema = resolve(doc, schema)
    if not isinstance(schema, dict) or depth > MAX_SAMPLE_DEPTH:
        return None
    if example is None:
        # A schema-level example still tells tuple lengths and map keys
        example = schema.get("example")
    for key in ("oneOf", "anyOf"):
        if schema.get(key):
            return sample(doc, pick(schema[key]), pick, example, depth + 1)
    if schema.get("allOf"):
        merged = {}
        for part in schema["allOf"]:
            value = sample(doc, part, pick, example, depth + 1)
            if isinstance(value, dict):
                merged.update(value)
        return merged
    if "enum" in schema and schema["enum"]:
        return schema["enum"][0]
    kind = schema.get("type")
    if isinstance(kind, list):
        kind = next((k for k in kind if k != "null"), None)
    if kind == "object" or "properties" in schema:
        if not schema.get("properties") and not isinstance(schema.get("additionalProperties"), dict):
            # No fields described: only an example says what is in it
            return example if isinstance(example, dict) else {}
        sub = example if isinstance(example, dict) else {}
        out = {
            name: known(sample(doc, prop, pick, sub.get(name), depth + 1))
            for name, prop in schema.get("properties", {}).items()
        }
        extra = schema.get("additionalProperties")
        if isinstance(extra, dict) and not out:
            key, value = next(iter(sub.items()), ("key", None))
            out[key] = known(sample(doc, extra, pick, value, depth + 1))
        return out
    if kind == "array":
        items = resolve(doc, schema.get("items", {}))
        alts = (items.get("oneOf") or items.get("anyOf") or []) if isinstance(items, dict) else []
        untyped = not isinstance(items, dict) or not any(
            k in items for k in ("type", "properties", "oneOf", "anyOf", "allOf", "$ref", "enum")
        )
        if (alts or untyped) and isinstance(example, list):
            # Positional tuples such as [timestamp, value] or [timestamp,
            # {breakdown}], or items with no schema: the example is all
            # there is to go on
            return example
        if alts:
            # A tuple with no example: its length and types are unknown
            return UNKNOWN_TUPLE
        first = example[0] if isinstance(example, list) and example else None
        item = sample(doc, items, pick, first, depth + 1)
        if item is None or item is UNKNOWN_TUPLE:
            # An empty list still checks the types around it
            return []
        if is_scalar(doc, items) and isinstance(example, list) and example:
            # fixed-length tuples such as [timestamp, price]: keep the length
            return [item] * len(example)
        return [item]
    # "number" is often used for integers (e.g. timestamps), so use one that
    # fits both
    return {"integer": 1, "number": 1, "boolean": True, "string": "1"}.get(kind)


METHODS = ("get", "post", "put", "patch", "delete")


def schema_example(doc, schema, depth=0):
    """The example a schema itself carries, through $ref and single-branch
    oneOf/anyOf/allOf wrappers."""
    schema = resolve(doc, schema)
    if not isinstance(schema, dict) or depth > MAX_SAMPLE_DEPTH:
        return None
    if "example" in schema:
        return schema["example"]
    for key in ("oneOf", "anyOf", "allOf"):
        alts = schema.get(key)
        if isinstance(alts, list) and len(alts) == 1:
            return schema_example(doc, alts[0], depth + 1)
    return None


def describes_fields(doc, schema):
    """False for a schema that is only `type: object` (or empty): a sample
    built from it is `{}` and checks nothing."""
    schema = resolve(doc, schema)
    if not isinstance(schema, dict):
        return False
    shape = ("properties", "additionalProperties", "items", "allOf", "oneOf", "anyOf", "enum")
    return any(k in schema for k in shape) or schema.get("type") not in (None, "object")


def success_response(doc, op):
    """(schema, example) of an operation's first 2xx JSON response, for
    OpenAPI 3 and Swagger 2."""
    responses = op.get("responses") or {}
    codes = sorted(c for c in responses if str(c).startswith("2"))
    if not codes:
        return None, None
    resp = resolve(doc, responses[codes[0]])
    if not isinstance(resp, dict):
        return None, None
    if "schema" in resp or "examples" in resp and "content" not in resp:  # Swagger 2
        examples = resp.get("examples") or {}
        example = next((v for k, v in examples.items() if "json" in k), None)
        return resp.get("schema"), example
    content = resp.get("content") or {}
    media = next((v for k, v in content.items() if "json" in k), None)
    if media is None:
        media = content.get("*/*")
    if not isinstance(media, dict):
        return None, None
    example = media.get("example")
    if example is None and media.get("examples"):
        example = resolve(doc, next(iter(media["examples"].values()))).get("value")
    return media.get("schema"), example


def extract(doc):
    """Every operation with a 2xx JSON response schema or example, keyed by
    operationId (or "METHOD /path" when it has none)."""
    out = {}
    for path, item in doc.get("paths", {}).items():
        item = resolve(doc, item)
        for method in METHODS:
            op = item.get(method) if isinstance(item, dict) else None
            if not isinstance(op, dict):
                continue
            schema, example = success_response(doc, op)
            if example is None and schema is not None:
                example = schema_example(doc, schema)
            if schema is None and example is None:
                continue
            entry = {"path": path}
            if method != "get":
                entry["method"] = method.upper()
            servers = op.get("servers") or item.get("servers")
            if servers:
                # Paths of one spec served from different hosts
                entry["server"] = servers[0].get("url")
            if example is not None:
                entry["example"] = example
            if schema and describes_fields(doc, schema):
                first = known(sample(doc, schema, lambda alts: alts[0], example))
                last = known(sample(doc, schema, lambda alts: alts[-1], example))
                entry["sample"] = first
                if last != first:
                    entry["sample2"] = last
            out[op.get("operationId") or f"{method.upper()} {path}"] = entry
    return dict(sorted(out.items()))


def main(argv):
    if len(argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    examples = extract(fetch_openapi(argv[1]))
    with open(argv[2], "w", encoding="utf-8") as f:
        json.dump(examples, f, indent=1, ensure_ascii=False)
        f.write("\n")
    print(f"wrote {len(examples)} examples to {argv[2]}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
