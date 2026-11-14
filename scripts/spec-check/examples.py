#!/usr/bin/env python3
"""Extract example responses from an OpenAPI spec into one JSON file.

A client crate's tests deserialize each response into the crate's response
type, so a field whose type no longer matches the provider's spec fails
`cargo test` instead of failing at runtime.

The output maps each GET operation's operationId to its path and up to
three responses for its 200 JSON body:

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
        sub = example if isinstance(example, dict) else {}
        out = {
            name: sample(doc, prop, pick, sub.get(name), depth + 1)
            for name, prop in schema.get("properties", {}).items()
        }
        extra = schema.get("additionalProperties")
        if isinstance(extra, dict) and not out:
            key, value = next(iter(sub.items()), ("key", None))
            out[key] = sample(doc, extra, pick, value, depth + 1)
        return out
    if kind == "array":
        items = resolve(doc, schema.get("items", {}))
        alts = (items.get("oneOf") or items.get("anyOf") or []) if isinstance(items, dict) else []
        if alts and all(is_scalar(doc, a) for a in alts):
            return example if isinstance(example, list) else []
        first = example[0] if isinstance(example, list) and example else None
        item = sample(doc, items, pick, first, depth + 1)
        if item is None:
            return []
        if is_scalar(doc, items) and isinstance(example, list) and example:
            # fixed-length tuples such as [timestamp, price]: keep the length
            return [item] * len(example)
        return [item]
    # "number" is often used for integers (e.g. timestamps), so use one that
    # fits both
    return {"integer": 1, "number": 1, "boolean": True, "string": "1"}.get(kind)


def extract(doc):
    out = {}
    for path, ops in doc.get("paths", {}).items():
        op = ops.get("get")
        if not op or "operationId" not in op:
            continue
        content = op.get("responses", {}).get("200", {}).get("content", {})
        media = content.get("application/json", {})
        example = media.get("example")
        if example is None and media.get("examples"):
            example = next(iter(media["examples"].values())).get("value")
        if example is None:
            continue
        entry = {"path": path, "example": example}
        schema = media.get("schema")
        if schema:
            first = sample(doc, schema, lambda alts: alts[0], example)
            last = sample(doc, schema, lambda alts: alts[-1], example)
            entry["sample"] = first
            if last != first:
                entry["sample2"] = last
        out[op["operationId"]] = entry
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
