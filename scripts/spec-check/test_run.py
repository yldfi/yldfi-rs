"""Unit tests for the weekly API spec check (python3 -m unittest)."""
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("run", Path(__file__).with_name("run.py"))
run = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run)

OAS3 = {
    "openapi": "3.0.0",
    "info": {"version": "1.2"},
    "paths": {
        "/api/v1/orders/{uid}": {
            "parameters": [{"name": "uid", "in": "path", "required": True}],
            "get": {
                "parameters": [{"$ref": "#/components/parameters/Chain"}],
                "responses": {"200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Order"}}}}},
            },
        },
        "/api/v1/quote": {
            "post": {
                "requestBody": {"content": {"application/json": {"schema": {
                    "type": "object", "required": ["sellToken"],
                    "properties": {"sellToken": {"type": "string"}, "kind": {"type": "string"}}}}}},
                "responses": {"200": {"content": {"application/json": {"schema": {
                    "type": "array", "items": {"$ref": "#/components/schemas/Order"}}}}}},
            },
        },
    },
    "components": {
        "parameters": {"Chain": {"name": "chain", "in": "query"}},
        "schemas": {"Order": {
            "type": "object", "required": ["uid"],
            "properties": {"uid": {"type": "string"}, "parent": {"$ref": "#/components/schemas/Order"},
                           "owner": {"allOf": [{"type": "object", "properties": {"address": {"type": "string"}}}]}}}},
    },
}

SWAGGER2 = {
    "swagger": "2.0",
    "info": {"version": "2.2"},
    "paths": {"/erc20/{address}/price": {"get": {
        "parameters": [{"name": "address", "in": "path", "required": True},
                       {"name": "body", "in": "body", "schema": {"$ref": "#/definitions/Req"}}],
        "responses": {"200": {"schema": {"$ref": "#/definitions/Price"}}}}}},
    "definitions": {"Price": {"properties": {"usdPrice": {"type": "number"}}},
                    "Req": {"properties": {"x": {"type": "string"}}}},
}


class SurfaceTest(unittest.TestCase):
    def test_openapi3_operation(self):
        s = run.openapi_surface(OAS3)
        self.assertEqual(s["version"], "1.2")
        op = s["operations"]["GET /api/v1/orders/{uid}"]
        self.assertEqual(op["params"], ["path:uid*", "query:chain"])
        self.assertIn("uid*", op["response"])
        self.assertIn("owner.address", op["response"])
        # A recursive ref stops at the first repeat instead of looping.
        self.assertIn("parent", op["response"])
        self.assertNotIn("parent.uid*", op["response"])

    def test_request_body_and_array_response(self):
        op = run.openapi_surface(OAS3)["operations"]["POST /api/v1/quote"]
        self.assertEqual(op["request"], ["kind", "sellToken*"])
        self.assertIn("[].uid*", op["response"])

    def test_swagger2(self):
        op = run.openapi_surface(SWAGGER2)["operations"]["GET /erc20/{address}/price"]
        self.assertEqual(op["params"], ["path:address*"])
        self.assertEqual(op["request"], ["x"])
        self.assertEqual(op["response"], ["usdPrice"])

    def test_discriminate_single_path_apis(self):
        doc = {"openapi": "3.1.0", "paths": {"/v2/api": {"get": {"parameters": [
            {"name": "module", "in": "query", "required": True, "schema": {"default": "account"}},
            {"name": "action", "in": "query", "required": True, "schema": {"default": "balance"}},
            {"name": "address", "in": "query", "required": True, "schema": {"type": "string"}}],
            "responses": {"200": {}}}}}}
        self.assertEqual(list(run.openapi_surface(doc, discriminate=True)["operations"]),
                         ["GET /v2/api?module=account&action=balance"])
        self.assertEqual(list(run.openapi_surface(doc)["operations"]), ["GET /v2/api"])

    def test_readme_page_spec(self):
        page = 'Intro\n\n# OpenAPI definition\n\n```json\n{"openapi": "3.0.0", "paths": {}}\n```\n'
        self.assertEqual(run.readme_page_spec(page), {"openapi": "3.0.0", "paths": {}})
        self.assertIsNone(run.readme_page_spec("# Buy a package\n\nNo spec here."))

    def test_graphql(self):
        schema = {
            "queryType": {"name": "Query"}, "mutationType": None,
            "types": [
                {"kind": "OBJECT", "name": "Query", "fields": [
                    {"name": "vaults", "isDeprecated": False,
                     "args": [{"name": "chainId", "type": {"kind": "SCALAR", "name": "Int"}}],
                     "type": {"kind": "LIST", "ofType": {"kind": "OBJECT", "name": "Vault"}}}]},
                {"kind": "OBJECT", "name": "Vault", "fields": [
                    {"name": "address", "isDeprecated": False, "args": [],
                     "type": {"kind": "NON_NULL", "ofType": {"kind": "SCALAR", "name": "String"}}}]},
                {"kind": "OBJECT", "name": "__Schema", "fields": []},
            ],
        }
        ops = run.graphql_surface(schema)["operations"]
        self.assertEqual(ops["query vaults"]["params"], ["chainId: Int"])
        self.assertEqual(ops["query vaults"]["response"], ["[Vault]"])
        self.assertEqual(ops["object Vault"]["response"], ["address: String!"])
        self.assertNotIn("object __Schema", ops)


def surface(**ops):
    base = {"deprecated": False, "params": [], "request": [], "response": []}
    return {"operations": {k: {**base, **v} for k, v in ops.items()}}


class DiffTest(unittest.TestCase):
    def kinds(self, old, new, used=()):
        return [(c["op"], c["kind"], c["what"].split(":")[0], c["used"]) for c in run.diff_surfaces(old, new, used)]

    def test_unchanged(self):
        s = surface(**{"GET /a": {"params": ["query:x"]}})
        self.assertEqual(run.diff_surfaces(s, s), [])

    def test_removed_and_added_operations(self):
        old, new = surface(**{"GET /a": {}}), surface(**{"GET /b": {}})
        self.assertEqual(self.kinds(old, new, {"GET /a"}),
                         [("GET /a", "breaking", "removed", True), ("GET /b", "addition", "added", False)])

    def test_field_changes(self):
        old = surface(**{"GET /a": {"params": ["query:x"], "response": ["id*", "name"]}})
        new = surface(**{"GET /a": {"params": ["query:x*", "query:y"], "response": ["id*", "price"]}})
        self.assertEqual(self.kinds(old, new), [
            ("GET /a", "breaking", "parameters now required", False),
            ("GET /a", "addition", "parameters added", False),
            ("GET /a", "breaking", "response fields removed", False),
            ("GET /a", "addition", "response fields added", False),
        ])

    def test_new_required_input_is_breaking(self):
        old = surface(**{"POST /q": {"request": ["a"]}})
        new = surface(**{"POST /q": {"request": ["a", "b*"], "response": ["c*"]}})
        self.assertEqual(self.kinds(old, new), [
            ("POST /q", "breaking", "new required request fields", False),
            ("POST /q", "addition", "response fields added", False),
        ])

    def test_deprecation_is_breaking(self):
        old, new = surface(**{"GET /a": {}}), surface(**{"GET /a": {"deprecated": True}})
        self.assertEqual(self.kinds(old, new), [("GET /a", "breaking", "now deprecated", False)])


class UsedPathTest(unittest.TestCase):
    def test_suffix_and_placeholder_match(self):
        crate = {tuple(run.path_segments(p)) for p in ("/orders/{}", "/getAllGauges", "/coins/{id}?x=1")}
        s = surface(**{"GET /api/v1/orders/{uid}": {}, "GET /v1/getAllGauges": {}, "GET /coins/{id}": {},
                       "GET /coins/list": {}, "GET /api/v1/trades": {}})
        self.assertEqual(run.used_operations(s, crate),
                         {"GET /api/v1/orders/{uid}", "GET /v1/getAllGauges", "GET /coins/{id}"})

    def test_placeholder_only_paths_never_match(self):
        self.assertFalse(run.segments_match(["{}"], ["{}"]))
        self.assertFalse(run.segments_match(["orders"], ["trades"]))

    def test_unmatched_spans_all_specs_of_a_crate(self):
        a = {"crate": "cowp", "surface": surface(**{"GET /api/v1/orders": {}})}
        b = {"crate": "cowp", "surface": surface(**{"GET /api/v1/auction": {}})}
        orig = run.crate_paths
        run.crate_paths = lambda crate: {("api", "v1", "orders"), ("api", "v1", "auction"), ("api", "v1", "gone")}
        try:
            self.assertEqual(run.crate_unmatched([a, b]), {"cowp": ["/api/v1/gone"]})
        finally:
            run.crate_paths = orig

    def test_unmatched(self):
        s = surface(**{"GET /a/b": {}})
        self.assertEqual(run.unmatched_paths(s, {("a", "b"), ("gone",)}), ["/gone"])


class ActionTest(unittest.TestCase):
    def test_action_matches_variants_only_exactly(self):
        self.assertTrue(run.action_matches("GET /v2/api?module=account&action=txlist", "module=account&action=txlist"))
        self.assertTrue(run.action_matches("GET /v2/api?module=account&action=txlist (advanced-filter-txlist)",
                                           "module=account&action=txlist"))
        self.assertFalse(run.action_matches("GET /v2/api?module=account&action=txlistinternal",
                                            "module=account&action=txlist"))

    def test_library_calls_file(self):
        import tempfile
        with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
            f.write("# comment\nnocrate | module=a&action=b | x\nlibonly | module=gastracker&action=gasoracle | gas_oracle\n")
        self.assertEqual(run.crate_actions("libonly", f.name), {"module=gastracker&action=gasoracle"})
        Path(f.name).unlink()

    def test_ethcli_marks_library_and_direct_actions(self):
        actions = run.crate_actions("ethcli")
        self.assertIn("module=proxy&action=eth_getTransactionByHash", actions)  # direct URL
        self.assertIn("module=account&action=txlist", actions)  # via foundry-block-explorers


class UrlTest(unittest.TestCase):
    def test_placeholders(self):
        import os
        os.environ.pop("SPEC_TEST_KEY", None)
        self.assertEqual(run.expand_url("https://x/${SPEC_TEST_KEY}/q")[1], ["SPEC_TEST_KEY"])
        os.environ["SPEC_TEST_KEY"] = "secret-value"
        self.assertEqual(run.expand_url("https://x/${SPEC_TEST_KEY}/q"), ("https://x/secret-value/q", []))
        self.assertEqual(run.redact("GET https://x/secret-value/q", "https://x/${SPEC_TEST_KEY}/q"), "GET https://x/***/q")
        del os.environ["SPEC_TEST_KEY"]

    def test_http_errors(self):
        self.assertEqual(run.classify_http_error(404)[0], "FAIL")
        self.assertEqual(run.classify_http_error(503)[0], "WARN")
        self.assertEqual(run.classify_http_error(429)[0], "WARN")


if __name__ == "__main__":
    unittest.main()
