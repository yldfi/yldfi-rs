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

    def test_maps_use_an_example_key(self):
        schema = {"type": "object", "additionalProperties": {"type": "number"}}
        self.assertEqual(sample({}, schema, first, {"usd": 2.5}), {"usd": 1})

    def test_extract_skips_operations_without_examples(self):
        def op(op_id, example=None):
            media = {"schema": {"type": "object", "properties": {"x": {"type": "string"}}}}
            if example is not None:
                media["example"] = example
            return {"get": {"operationId": op_id, "responses": {"200": {"content": {"application/json": media}}}}}

        doc = {"paths": {"/b": op("b", {"x": "y"}), "/a": op("a")}}
        self.assertEqual(
            extract(doc),
            {"b": {"path": "/b", "example": {"x": "y"}, "sample": {"x": "1"}}},
        )


if __name__ == "__main__":
    unittest.main()
