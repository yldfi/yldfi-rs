"""Unit tests for the weekly API check classifier (python3 -m unittest)."""
import importlib.util
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location("run", Path(__file__).with_name("run.py"))
run = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run)


def status(out, err="", code=1, keyless=True, resolves=True):
    return run.classify(code, out, err, keyless, resolve=lambda host: resolves)[0]


class ClassifyTest(unittest.TestCase):
    def test_success(self):
        self.assertEqual(status("26060135\n", code=0), "PASS")

    def test_aggregate_errors_heading_is_not_a_failure(self):
        out = "Aggregated Price for ETH\n  Median: $2689.60\nErrors:\n  alchemy: ALCHEMY_API_KEY not configured\n"
        self.assertEqual(status(out, code=0), "PASS")

    def test_error_line_with_exit_zero_is_not_a_pass(self):
        self.assertNotEqual(status("Error: Use fetch_quotes_all instead\n", code=0), "PASS")

    def test_empty_output_is_not_a_pass(self):
        self.assertNotEqual(status("", code=0), "PASS")

    # Real failures seen during the 2026-09 audit must FAIL.
    def test_gone_endpoint(self):
        self.assertEqual(status("", "Error: Failed to get block number: HTTP error 410 with body: gone"), "FAIL")

    def test_dns_failure(self):
        err = "Error: error sending request for url (https://core.gashawk.io/…): dns error: failed to lookup address"
        self.assertEqual(status("", err), "FAIL")

    def test_connection_error_to_unresolvable_host_fails(self):
        # ethcli omits the DNS cause: "error sending request for url (...)".
        err = "Error: Failed to get block number: error sending request for url (https://core.gashawk.io)"
        self.assertEqual(status("", err, resolves=False), "FAIL")
        self.assertEqual(status("", err, resolves=True), "WARN")

    def test_parse_failure(self):
        err = "Error: error decoding response body: missing field `pools` at line 1 column 2"
        self.assertEqual(status("", err), "FAIL")

    def test_not_found(self):
        self.assertEqual(status("", "Error: API error (404): Token not found"), "FAIL")

    def test_bad_request_means_request_drift(self):
        self.assertEqual(status("", "Error: API error (400): src must be an Ethereum address"), "FAIL")

    def test_keyless_api_starting_to_require_auth_fails(self):
        # e.g. Pyth Hermes after the Pyth Core upgrade, polygon-rpc.com.
        self.assertEqual(status("", "Error: HTTP error 401 with body: Unauthorized", keyless=True), "FAIL")

    def test_auth_error_on_keyed_api_warns(self):
        err = "Error: API error (401): Moralis Free usage is paused"
        self.assertEqual(status("", err, keyless=False), "WARN")

    def test_missing_key_skips(self):
        self.assertEqual(status("", "Error: ENSO_API_KEY not configured"), "SKIP")

    def test_transient_errors_warn(self):
        self.assertEqual(status("", "Error: HTTP error 429 with body: Too Many Requests"), "WARN")
        self.assertEqual(status("", "Error: API error (503): Service Unavailable"), "WARN")
        self.assertEqual(status("", "Error: timed out after 90s", code=124), "WARN")
        self.assertEqual(status("", "Error: API error (451): Service unavailable from a restricted location"), "WARN")

    def test_unknown_error_fails(self):
        self.assertEqual(status("", "Error: something unexpected happened"), "FAIL")

    def test_redaction(self):
        text = run.redact("Error: request to https://user:pw@rpc.example.com/v2/abcdef123456?key=s3cret failed")
        self.assertNotIn("abcdef123456", text)
        self.assertNotIn("s3cret", text)
        self.assertNotIn("pw@", text)
        self.assertIn("rpc.example.com", text)


class ParseChecksTest(unittest.TestCase):
    def test_checks_file_parses(self):
        checks = run.parse_checks(Path(__file__).with_name("checks.txt"))
        self.assertGreater(len(checks), 20)
        zerox = next(c for c in checks if c["provider"] == "0x")
        self.assertEqual(zerox["env"], "ZEROX_API_KEY|0X_API_KEY")
        self.assertTrue(zerox["args"].startswith("0x price "))


if __name__ == "__main__":
    unittest.main()
