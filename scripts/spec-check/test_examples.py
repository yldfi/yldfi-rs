import unittest

from examples import extract, sample


def first(alts):
    return alts[0]


def last(alts):
    return alts[-1]


class SampleTest(unittest.TestCase):
    def test_every_property_gets_a_typed_value(self):
        schema = {
            "type": "object",
            "properties": {
                "n": {"type": "number", "nullable": True},
                "i": {"type": "integer"},
                "s": {"type": "string"},
                "b": {"type": "boolean"},
                "e": {"type": "string", "enum": ["coin", "rwa"]},
            },
        }
        self.assertEqual(
            sample({}, schema, first),
            {"n": 1, "i": 1, "s": "1", "b": True, "e": "coin"},
        )

    def test_all_of_a_scalar_is_that_scalar(self):
        doc = {"components": {"schemas": {"Amount": {"type": "string"}}}}
        schema = {"type": "object", "properties": {
            "amount": {"description": "atoms", "allOf": [{"$ref": "#/components/schemas/Amount"}]},
            "both": {"allOf": [{"type": "object", "properties": {"a": {"type": "integer"}}},
                               {"type": "object", "properties": {"b": {"type": "boolean"}}}]},
        }}
        self.assertEqual(sample(doc, schema, first), {"amount": "1", "both": {"a": 1, "b": True}})

    def test_refs_and_one_of(self):
        doc = {"components": {"schemas": {"A": {"oneOf": [{"type": "integer"}, {"type": "string"}]}}}}
        ref = {"$ref": "#/components/schemas/A"}
        self.assertEqual(sample(doc, ref, first), 1)
        self.assertEqual(sample(doc, ref, last), "1")

    def test_mixed_tuples_come_from_the_example(self):
        schema = {"type": "array", "items": {"type": "array", "items": {"oneOf": [{"type": "integer"}, {"type": "string"}]}}}
        example = [[1700000000000, 44, "fear"]]
        self.assertEqual(sample({}, schema, first, example), example)

    def test_scalar_tuples_keep_the_example_length(self):
        schema = {"type": "array", "items": {"type": "array", "items": {"type": "number"}}}
        self.assertEqual(sample({}, schema, first, [[1.5, 2.5], [3.5, 4.5]]), [[1, 1]])

    def test_schema_level_examples_give_tuple_shape(self):
        schema = {"type": "array", "items": {
            "type": "array", "items": {"oneOf": [{"type": "number"}, {"type": "object"}]},
            "example": [1640995200, 2500000000],
        }}
        self.assertEqual(sample({}, schema, last), [[1640995200, 2500000000]])

    def test_untyped_items_use_the_example(self):
        schema = {"type": "array", "items": {"type": "array", "example": [1640995200, 750000000]}}
        self.assertEqual(sample({}, schema, first), [[1640995200, 750000000]])

    def test_tuples_without_examples_leave_their_list_empty(self):
        tuple_ = {"type": "array", "items": {"oneOf": [{"type": "number"}, {"type": "object"}]}}
        schema = {"type": "object", "properties": {
            "chart": {"type": "array", "items": tuple_},
            "point": tuple_,
        }}
        self.assertEqual(sample({}, schema, first), {"chart": [], "point": []})

    def test_maps_use_an_example_key(self):
        schema = {"type": "object", "additionalProperties": {"type": "number"}}
        self.assertEqual(sample({}, schema, first, {"usd": 2.5}), {"usd": 1})

    def test_extract_keeps_schema_only_operations(self):
        def op(op_id, example=None):
            media = {"schema": {"type": "object", "properties": {"x": {"type": "string"}}}}
            if example is not None:
                media["example"] = example
            return {"get": {"operationId": op_id, "responses": {"200": {"content": {"application/json": media}}}}}

        doc = {"paths": {"/b": op("b", {"x": "y"}), "/a": op("a")}}
        self.assertEqual(
            extract(doc),
            {
                "a": {"path": "/a", "sample": {"x": "1"}},
                "b": {"path": "/b", "example": {"x": "y"}, "sample": {"x": "1"}},
            },
        )

    def test_extract_other_methods_and_operations_without_ids(self):
        schema = {"type": "object", "properties": {"ok": {"type": "boolean"}}}
        doc = {"paths": {"/orders": {
            "post": {"responses": {"201": {"content": {"application/json; charset=utf-8": {"schema": schema}}}}},
            "delete": {"responses": {"204": {"description": "no body"}}},
        }}}
        self.assertEqual(
            extract(doc),
            {"POST /orders": {"path": "/orders", "method": "POST", "sample": {"ok": True}}},
        )

    def test_extract_records_per_path_servers(self):
        media = {"application/json": {"schema": {"type": "integer"}}}
        doc = {"paths": {"/chart/{pool}": {
            "servers": [{"url": "https://yields.example"}],
            "get": {"operationId": "chart", "responses": {"200": {"content": media}}},
        }}}
        self.assertEqual(
            extract(doc),
            {"chart": {"path": "/chart/{pool}", "server": "https://yields.example", "sample": 1}},
        )

    def test_extract_writes_no_sample_for_shapeless_schemas(self):
        def op(schema):
            return {"get": {"operationId": "x", "responses": {"200": {"content": {"application/json": {"schema": schema}}}}}}

        # Still listed, so the coverage test sees it, but with nothing to check
        self.assertEqual(extract({"paths": {"/a": op({"type": "object"})}}), {"x": {"path": "/a"}})
        self.assertEqual(
            extract({"paths": {"/a": op({"type": "string"})}}),
            {"x": {"path": "/a", "sample": "1"}},
        )

    def test_extract_uses_schema_level_examples(self):
        doc = {
            "components": {"schemas": {"R": {"type": "object", "example": {"id": "a", "n": 2}}}},
            "paths": {"/r": {"get": {"operationId": "r", "responses": {"200": {"content": {"application/json": {
                "schema": {"oneOf": [{"$ref": "#/components/schemas/R"}]},
            }}}}}}},
        }
        self.assertEqual(
            extract(doc),
            {"r": {"path": "/r", "example": {"id": "a", "n": 2}, "sample": {"id": "a", "n": 2}}},
        )

    def test_extract_skips_dangling_refs(self):
        ok = {"200": {"content": {"application/json": {"schema": {"type": "integer"}}}}}
        bad = {"200": {"$ref": "#/components/responses/Missing"}}
        doc = {"paths": {"/a": {"get": {"operationId": "a", "responses": ok}},
                         "/b": {"get": {"operationId": "b", "responses": bad}}}}
        self.assertEqual(extract(doc), {"a": {"path": "/a", "sample": 1}})

    def test_extract_swagger2(self):
        doc = {
            "definitions": {"Thing": {"type": "object", "properties": {"id": {"type": "integer"}}}},
            "paths": {"/things": {"get": {"operationId": "things", "responses": {"200": {
                "schema": {"type": "array", "items": {"$ref": "#/definitions/Thing"}},
                "examples": {"application/json": [{"id": 7}]},
            }}}}},
        }
        self.assertEqual(
            extract(doc),
            {"things": {"path": "/things", "example": [{"id": 7}], "sample": [{"id": 1}]}},
        )


if __name__ == "__main__":
    unittest.main()
