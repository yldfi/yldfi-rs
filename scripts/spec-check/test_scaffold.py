import contextlib
import io
import json
import tempfile
import unittest

import scaffold


class ScaffoldTest(unittest.TestCase):
    def test_names(self):
        self.assertEqual(scaffold.pascal("rwas-id-market_chart"), "RwasIdMarketChart")
        self.assertEqual(scaffold.snake("coinId"), "coin_id")
        self.assertEqual(scaffold.snake("rwas-list"), "rwas_list")

    def test_drafts_method_options_and_types(self):
        doc = {
            "paths": {
                "/things/{thing_id}": {
                    "get": {
                        "operationId": "things-id",
                        "summary": "Thing by ID",
                        "parameters": [
                            {"name": "thing_id", "in": "path", "required": True},
                            {"name": "vs_currency", "in": "query", "required": True, "schema": {"type": "string"}},
                            {"name": "limit", "in": "query", "schema": {"type": "integer"}},
                        ],
                        "responses": {"200": {"content": {"application/json": {"schema": {
                            "type": "object",
                            "properties": {"type": {"type": "string"}, "price": {"type": "number"}},
                        }}}}},
                    }
                }
            }
        }
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(doc, f)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            scaffold.main(["scaffold.py", f.name, "things-id"])
        text = out.getvalue()
        self.assertIn("pub struct ThingsIdOptions", text)
        self.assertIn("(required)", text)
        self.assertIn("pub limit: Option<u64>,", text)
        self.assertIn('#[serde(rename = "type")]', text)
        self.assertIn("pub type_: Option<String>,", text)
        self.assertIn("pub price: Option<f64>,", text)
        self.assertIn("pub async fn things_id(&self, thing_id: &str, options: &ThingsIdOptions) -> Result<ThingsIdResponse>", text)
        self.assertIn('format!("/things/{thing_id}{}", options.to_query_string())', text)


if __name__ == "__main__":
    unittest.main()
