use super::types::{DryRunFormat, TenderlyArgs};
use crate::config::{Chain, ConfigFile};
use crate::utils::address::resolve_label;
use std::process::Command;

/// Escape a string for use in single-quoted shell context.
/// SEC-SHELL-001: Prevents shell injection when users copy-paste dry-run output.
/// Single quotes in the input are escaped as '\'' (end quote, escaped quote, start quote).
fn escape_shell_single_quote(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Escape a string for use in JavaScript single-quoted strings.
fn escape_js_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Escape a string for use in PowerShell single-quoted strings.
/// In PowerShell, single quotes are escaped by doubling them.
fn escape_powershell_single_quote(s: &str) -> String {
    s.replace('\'', "''")
}

/// Escape a string for use in Python single-quoted strings.
fn escape_python_single_quote(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'")
}

/// Format a request as the specified output format
pub fn format_request(
    url: &str,
    method: &str,
    headers: &[(&str, &str)],
    body: &serde_json::Value,
    format: DryRunFormat,
    show_secrets: bool,
) -> String {
    let should_mask = |key: &str| -> bool {
        !show_secrets
            && (key.to_lowercase().contains("key") || key.to_lowercase().contains("authorization"))
    };

    match format {
        DryRunFormat::Json => serde_json::to_string_pretty(body).unwrap_or_default(),
        DryRunFormat::Url => url.to_string(),
        DryRunFormat::Curl => {
            let mut cmd = format!("curl -X {} '{}'", method, escape_shell_single_quote(url));
            for (key, value) in headers {
                let display_value = if should_mask(key) {
                    format!("${}", key.to_uppercase().replace("-", "_"))
                } else {
                    escape_shell_single_quote(value)
                };
                cmd.push_str(" \\\n  -H '");
                cmd.push_str(&escape_shell_single_quote(key));
                cmd.push_str(": ");
                cmd.push_str(&display_value);
                cmd.push('\'');
            }
            let body_str = serde_json::to_string(body).unwrap_or_default();
            cmd.push_str(" \\\n  -d '");
            cmd.push_str(&escape_shell_single_quote(&body_str));
            cmd.push('\'');
            cmd
        }
        DryRunFormat::Fetch => {
            let mut h_obj = String::from("{");
            for (i, (key, value)) in headers.iter().enumerate() {
                let val = if should_mask(key) {
                    format!("process.env.{}", key.to_uppercase().replace("-", "_"))
                } else {
                    format!("'{}'", escape_js_single_quote(value))
                };
                if i > 0 {
                    h_obj.push(',');
                }
                h_obj.push_str("\n    '");
                h_obj.push_str(&escape_js_single_quote(key));
                h_obj.push_str("': ");
                h_obj.push_str(&val);
            }
            h_obj.push_str("\n  }");

            let body_str = serde_json::to_string_pretty(body).unwrap_or_default();
            let mut s = format!(
                "const response = await fetch('{}', {{\n",
                escape_js_single_quote(url)
            );
            s.push_str(&format!(
                "  method: '{}',\n",
                escape_js_single_quote(method)
            ));
            s.push_str(&format!("  headers: {},\n", h_obj));
            s.push_str(&format!("  body: JSON.stringify({})\n", body_str));
            s.push_str("});\n");
            s.push_str("const data = await response.json();\n");
            s.push_str("console.log(data);");
            s
        }
        DryRunFormat::Powershell => {
            let mut h_hash = String::from("@{ ");
            for (key, value) in headers {
                let val = if should_mask(key) {
                    format!("$env:{}", key.to_uppercase().replace("-", "_"))
                } else {
                    format!("'{}'", escape_powershell_single_quote(value))
                };
                h_hash.push_str(&format!(
                    "\n    '{}' = {}",
                    escape_powershell_single_quote(key),
                    val
                ));
            }
            h_hash.push_str("\n}");

            let body_str = serde_json::to_string(body).unwrap_or_default();
            let mut s = format!("$headers = {}\n\n", h_hash);
            // PowerShell here-strings (@'...'@) don't need escaping for the content
            s.push_str("$body = @'\n");
            s.push_str(&body_str);
            s.push_str("\n'@\n\n");
            s.push_str(&format!(
                "Invoke-RestMethod -Uri '{}' -Method {} -Headers $headers -Body $body -ContentType 'application/json'",
                escape_powershell_single_quote(url),
                method
            ));
            s
        }
        DryRunFormat::Python => {
            let mut h_dict = String::from("{ ");
            for (i, (key, value)) in headers.iter().enumerate() {
                let val = if should_mask(key) {
                    format!("os.environ['{}']", key.to_uppercase().replace("-", "_"))
                } else {
                    format!("'{}'", escape_python_single_quote(value))
                };
                if i > 0 {
                    h_dict.push_str(", ");
                }
                h_dict.push_str(&format!(
                    "\n    '{}': {}",
                    escape_python_single_quote(key),
                    val
                ));
            }
            h_dict.push_str("\n}");

            let body_str = serde_json::to_string_pretty(body).unwrap_or_default();
            let mut s = String::from("import requests\nimport os\n\n");
            s.push_str(&format!("headers = {}\n\n", h_dict));
            s.push_str(&format!("data = {}\n\n", body_str));
            s.push_str(&format!(
                "response = requests.{}('{}', headers=headers, json=data)\n",
                method.to_lowercase(),
                escape_python_single_quote(url)
            ));
            s.push_str("print(response.json())");
            s
        }
        DryRunFormat::Httpie => {
            let mut cmd = format!("http {} '{}'", method, escape_shell_single_quote(url));
            for (key, value) in headers {
                let val = if should_mask(key) {
                    format!("${}", key.to_uppercase().replace("-", "_"))
                } else {
                    escape_shell_single_quote(value)
                };
                cmd.push_str(" \\\n  '");
                cmd.push_str(&escape_shell_single_quote(key));
                cmd.push(':');
                cmd.push_str(&val);
                cmd.push('\'');
            }
            let body_str = serde_json::to_string(body).unwrap_or_default();
            cmd.push_str(" \\\n  --raw '");
            cmd.push_str(&escape_shell_single_quote(&body_str));
            cmd.push('\'');
            cmd
        }
        DryRunFormat::Wget => {
            let mut cmd = format!(
                "wget -q -O - --method={} '{}'",
                method,
                escape_shell_single_quote(url)
            );
            for (key, value) in headers {
                let val = if should_mask(key) {
                    format!("${}", key.to_uppercase().replace("-", "_"))
                } else {
                    escape_shell_single_quote(value)
                };
                cmd.push_str(" \\\n  --header='");
                cmd.push_str(&escape_shell_single_quote(key));
                cmd.push_str(": ");
                cmd.push_str(&val);
                cmd.push('\'');
            }
            let body_str = serde_json::to_string(body).unwrap_or_default();
            cmd.push_str(" \\\n  --body-data='");
            cmd.push_str(&escape_shell_single_quote(&body_str));
            cmd.push('\'');
            cmd
        }
        DryRunFormat::Go => {
            let body_str = serde_json::to_string_pretty(body).unwrap_or_default();
            let mut h_lines = String::new();
            for (key, value) in headers {
                let val = if should_mask(key) {
                    format!("os.Getenv(\"{}\")", key.to_uppercase().replace("-", "_"))
                } else {
                    // Escape for Go double-quoted strings
                    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
                };
                h_lines.push_str(&format!(
                    "    req.Header.Set(\"{}\", {})\n",
                    key.replace('\\', "\\\\").replace('"', "\\\""),
                    val
                ));
            }
            let mut s = String::from("package main\n\nimport (\n    \"bytes\"\n    \"encoding/json\"\n    \"fmt\"\n    \"net/http\"\n    \"os\"\n)\n\nfunc main() {\n");
            // Go raw strings (backticks) don't allow backticks inside, so escape if present
            let safe_body = if body_str.contains('`') {
                format!(
                    "\"{}\"",
                    body_str
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                )
            } else {
                format!("`{}`", body_str)
            };
            s.push_str(&format!("    data := {}\n\n", safe_body));
            s.push_str(&format!(
                "    req, _ := http.NewRequest(\"{}\", \"{}\", bytes.NewBuffer([]byte(data)))\n",
                method,
                url.replace('\\', "\\\\").replace('"', "\\\"")
            ));
            s.push_str(&h_lines);
            s.push_str("    req.Header.Set(\"Content-Type\", \"application/json\")\n\n");
            s.push_str("    client := &http.Client{}\n");
            s.push_str("    resp, _ := client.Do(req)\n");
            s.push_str("    defer resp.Body.Close()\n\n");
            s.push_str("    var result map[string]interface{}\n");
            s.push_str("    json.NewDecoder(resp.Body).Decode(&result)\n");
            s.push_str("    fmt.Println(result)\n}");
            s
        }
        DryRunFormat::Rust => {
            let body_str = serde_json::to_string_pretty(body).unwrap_or_default();
            let mut h_lines = String::new();
            for (key, value) in headers {
                let val = if should_mask(key) {
                    format!(
                        "&std::env::var(\"{}\").unwrap()",
                        key.to_uppercase().replace("-", "_")
                    )
                } else {
                    format!("\"{}\"", value)
                };
                h_lines.push_str(&format!("        .header(\"{}\", {})\n", key, val));
            }
            let mut s = String::from("use reqwest::blocking::Client;\nuse serde_json::Value;\n\nfn main() -> Result<(), Box<dyn std::error::Error>> {\n");
            s.push_str(&format!(
                "    let body: Value = serde_json::from_str(r#\"{}\"#)?;\n\n",
                body_str
            ));
            s.push_str("    let client = Client::new();\n");
            s.push_str(&format!(
                "    let response = client\n        .{}(\"{}\")\n",
                method.to_lowercase(),
                url
            ));
            s.push_str(&h_lines);
            s.push_str("        .json(&body)\n");
            s.push_str("        .send()?\n");
            s.push_str("        .json::<Value>()?;\n");
            s.push_str("    println!(\"{{:#?}}\", response);\n");
            s.push_str("    Ok(())\n}");
            s
        }
        DryRunFormat::Axios => {
            let mut h_obj = String::from("{");
            for (i, (key, value)) in headers.iter().enumerate() {
                let val = if should_mask(key) {
                    format!("process.env.{}", key.to_uppercase().replace("-", "_"))
                } else {
                    format!("'{}'", value)
                };
                if i > 0 {
                    h_obj.push(',');
                }
                h_obj.push_str("\n    '");
                h_obj.push_str(key);
                h_obj.push_str("': ");
                h_obj.push_str(&val);
            }
            h_obj.push_str("\n  }");

            let body_str = serde_json::to_string_pretty(body).unwrap_or_default();
            let mut s = String::from("const axios = require('axios');\n\n");
            s.push_str(&format!(
                "axios.{}('{}', {},\n",
                method.to_lowercase(),
                url,
                body_str
            ));
            s.push_str(&format!("  {{ headers: {} }}\n", h_obj));
            s.push_str(")\n.then(response => console.log(response.data))\n");
            s.push_str(".catch(error => console.error(error));");
            s
        }
    }
}

pub fn build_calldata(
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
) -> anyhow::Result<String> {
    use crate::utils::is_safe_cli_value;

    if let Some(d) = data {
        // Validate data doesn't contain dangerous characters
        if !is_safe_cli_value(d) {
            return Err(anyhow::anyhow!(
                "Invalid data: contains potentially dangerous characters"
            ));
        }
        Ok(d.clone())
    } else if let Some(s) = sig {
        // Validate signature doesn't contain dangerous characters
        if !is_safe_cli_value(s) {
            return Err(anyhow::anyhow!(
                "Invalid signature: contains potentially dangerous characters"
            ));
        }
        // Validate each argument
        for (i, arg) in args.iter().enumerate() {
            if !is_safe_cli_value(arg) {
                return Err(anyhow::anyhow!(
                    "Invalid argument {}: contains potentially dangerous characters",
                    i
                ));
            }
        }

        let mut cmd = Command::new("cast");
        cmd.arg("calldata").arg(s);
        for arg in args {
            cmd.arg(resolve_label(arg));
        }
        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stderr_trimmed = stderr.trim();

            // Provide helpful hints for common array encoding issues
            let hint = if s.contains("[]") {
                "\n\nHint: For array parameters, pass the array in brackets WITHOUT quotes around elements:\n  \
                 Example: '[0x1234...,0x5678...]' for address[]\n  \
                 Example: '[1,2,3]' for uint256[]\n  \
                 Note: No spaces or quotes inside the brackets!"
            } else {
                ""
            };

            if stderr_trimmed.is_empty() {
                return Err(anyhow::anyhow!("Failed to encode calldata{}", hint));
            } else {
                return Err(anyhow::anyhow!(
                    "Failed to encode calldata: {}{}",
                    stderr_trimmed,
                    hint
                ));
            }
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Must provide --sig or --data"))
    }
}

pub fn value_to_hex(value: &str) -> anyhow::Result<String> {
    if value == "0" {
        Ok("0x0".to_string())
    } else {
        Ok(format!("0x{:x}", value.parse::<u128>()?))
    }
}

pub fn block_to_param(block: &str) -> anyhow::Result<String> {
    if block == "latest" || block == "pending" || block == "earliest" {
        Ok(block.to_string())
    } else {
        // Parse block number (supports decimal or hex input)
        let num = crate::utils::parse_block_number(block)?;
        Ok(format!("0x{:x}", num))
    }
}

/// Configured endpoint URLs for `chain` matching `pred`, best first:
/// higher priority first, random order within a priority tier (so one
/// flaky/rate-limited node is not always tried first).
fn ranked_urls(
    config: &ConfigFile,
    chain: Chain,
    pred: impl Fn(&crate::config::EndpointConfig) -> bool,
) -> Vec<String> {
    use rand::seq::SliceRandom;
    let mut eps: Vec<_> = config
        .endpoints
        .iter()
        .filter(|e| e.enabled && e.chain == chain && pred(e))
        .collect();
    eps.shuffle(&mut rand::thread_rng());
    eps.sort_by_key(|e| std::cmp::Reverse(e.priority));
    eps.into_iter().map(|e| e.url.clone()).collect()
}

fn push_unique(out: &mut Vec<String>, urls: impl IntoIterator<Item = String>) {
    for u in urls {
        if !out.contains(&u) {
            out.push(u);
        }
    }
}

/// Debug-capable RPC URLs in failover order: `--rpc-url` if given, else
/// `has_debug` endpoints (by priority), then `has_trace` endpoints, then
/// `debug_rpc_urls` from the config file.
pub fn get_debug_rpc_urls(rpc_url: &Option<String>, chain: Chain) -> Vec<String> {
    if let Some(url) = rpc_url {
        return vec![url.clone()];
    }
    let Some(config) = ConfigFile::load_default().ok().flatten() else {
        return Vec::new();
    };
    debug_urls_from_config(&config, chain)
}

fn debug_urls_from_config(config: &ConfigFile, chain: Chain) -> Vec<String> {
    let mut out = Vec::new();
    push_unique(&mut out, ranked_urls(config, chain, |e| e.has_debug));
    push_unique(&mut out, ranked_urls(config, chain, |e| e.has_trace));
    push_unique(&mut out, config.debug_rpc_urls.iter().cloned());
    out
}

/// Best debug-capable RPC URL (see [`get_debug_rpc_urls`])
pub fn get_debug_rpc_url(rpc_url: &Option<String>, chain: Chain) -> Option<String> {
    get_debug_rpc_urls(rpc_url, chain).into_iter().next()
}

/// Trace-capable RPC URLs in failover order (`has_trace` first, then `has_debug`)
pub fn get_trace_rpc_urls(rpc_url: &Option<String>, chain: Chain) -> Vec<String> {
    if let Some(url) = rpc_url {
        return vec![url.clone()];
    }
    let Some(config) = ConfigFile::load_default().ok().flatten() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    push_unique(&mut out, ranked_urls(&config, chain, |e| e.has_trace));
    push_unique(&mut out, ranked_urls(&config, chain, |e| e.has_debug));
    out
}

/// Best trace-capable RPC URL (see [`get_trace_rpc_urls`])
pub fn get_trace_rpc_url(rpc_url: &Option<String>, chain: Chain) -> Option<String> {
    get_trace_rpc_urls(rpc_url, chain).into_iter().next()
}

/// POST a JSON-RPC request, failing over across `urls` on transport errors,
/// non-2xx responses and rate-limit/capacity JSON-RPC errors.
///
/// Returns the `result` value (or the whole response if it has no `result`).
pub async fn post_jsonrpc_with_failover(
    urls: &[String],
    request: &serde_json::Value,
    quiet: bool,
) -> anyhow::Result<serde_json::Value> {
    use crate::rpc::is_failover_error;
    use crate::utils::url::redact_url;

    let method = request
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("request");
    let client = crate::utils::get_shared_http_client()
        .cloned()
        .unwrap_or_else(|_| reqwest::Client::new());
    let mut last_err: Option<anyhow::Error> = None;

    for (i, url) in urls.iter().enumerate() {
        let has_next = i + 1 < urls.len();
        let host = redact_url(url);
        if !quiet {
            eprintln!("Calling {method} on {host}...");
        }

        let attempt: anyhow::Result<serde_json::Value> = async {
            let response = client
                .post(url)
                .header("Content-Type", "application/json")
                .json(request)
                .send()
                .await
                .map_err(|e| anyhow::anyhow!("error sending request: {}", e.without_url()))?;
            let status = response.status();
            let body: serde_json::Value = response.json().await.map_err(|e| {
                anyhow::anyhow!(
                    "HTTP {status}: invalid JSON-RPC response: {}",
                    e.without_url()
                )
            })?;
            if let Some(error) = body.get("error") {
                return Err(anyhow::anyhow!("RPC error: {}", error));
            }
            if !status.is_success() {
                return Err(anyhow::anyhow!("HTTP {status}"));
            }
            Ok(body.get("result").cloned().unwrap_or(body))
        }
        .await;

        match attempt {
            Ok(v) => return Ok(v),
            Err(e) => {
                let msg = e.to_string();
                let retryable =
                    is_failover_error(&msg) || msg.starts_with("HTTP ") || msg.contains("-32601"); // method not found on this node
                if has_next && retryable {
                    if !quiet {
                        eprintln!("  {host} failed ({msg}); trying next endpoint");
                    }
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No RPC endpoints available")))
}

pub fn get_tenderly_credentials(args: &TenderlyArgs) -> anyhow::Result<(String, String, String)> {
    args.get_credentials()
}

pub fn create_tenderly_client(args: &TenderlyArgs) -> anyhow::Result<tndrly::Client> {
    args.create_client()
}

pub fn build_state_overrides(
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
) -> anyhow::Result<std::collections::HashMap<String, serde_json::Value>> {
    use std::collections::HashMap;
    let mut state_objects: HashMap<String, serde_json::Value> = HashMap::new();

    for o in balance_overrides {
        let parts: Vec<&str> = o.splitn(2, '=').collect();
        if parts.len() == 2 {
            let entry = state_objects
                .entry(parts[0].to_lowercase())
                .or_insert_with(|| serde_json::json!({}));
            entry["balance"] = serde_json::json!(parts[1]);
        }
    }

    for o in storage_overrides {
        let parts: Vec<&str> = o.splitn(2, '=').collect();
        if parts.len() == 2 {
            let addr_slot: Vec<&str> = parts[0].splitn(2, ':').collect();
            if addr_slot.len() == 2 {
                let entry = state_objects
                    .entry(addr_slot[0].to_lowercase())
                    .or_insert_with(|| serde_json::json!({}));
                if entry.get("storage").is_none() {
                    entry["storage"] = serde_json::json!({});
                }
                entry["storage"][addr_slot[1]] = serde_json::json!(parts[1]);
            }
        }
    }

    for o in code_overrides {
        let parts: Vec<&str> = o.splitn(2, '=').collect();
        if parts.len() == 2 {
            let entry = state_objects
                .entry(parts[0].to_lowercase())
                .or_insert_with(|| serde_json::json!({}));
            entry["code"] = serde_json::json!(parts[1]);
        }
    }

    Ok(state_objects)
}

#[cfg(test)]
mod failover_tests {
    use super::*;
    use crate::config::EndpointConfig;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn debug_urls_respect_priority_and_capability() {
        let mut config = ConfigFile::default();
        for (url, prio, debug, trace) in [
            ("https://low-debug.io", 1u8, true, false),
            ("https://high-debug.io", 9, true, false),
            ("https://trace-only.io", 10, false, true),
            ("https://plain.io", 10, false, false),
        ] {
            let mut e = EndpointConfig::new(url);
            e.chain = Chain::Ethereum;
            e.priority = prio;
            e.has_debug = debug;
            e.has_trace = trace;
            config.endpoints.push(e);
        }
        config.debug_rpc_urls = vec!["https://extra.io".to_string()];
        assert_eq!(
            debug_urls_from_config(&config, Chain::Ethereum),
            vec![
                "https://high-debug.io",
                "https://low-debug.io",
                "https://trace-only.io",
                "https://extra.io"
            ]
        );
    }

    #[tokio::test]
    async fn jsonrpc_fails_over_on_rate_limit() {
        let limited = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1,
                "error": {"code": -32005, "message": "rate limited"}
            })))
            .mount(&limited)
            .await;
        let healthy = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "result": {"ok": true}
            })))
            .mount(&healthy)
            .await;

        let req = serde_json::json!({"jsonrpc": "2.0", "method": "debug_traceCall", "id": 1});
        let out = post_jsonrpc_with_failover(&[limited.uri(), healthy.uri()], &req, true)
            .await
            .unwrap();
        assert_eq!(out, serde_json::json!({"ok": true}));
    }

    #[tokio::test]
    async fn jsonrpc_non_retryable_error_is_returned() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0", "id": 1,
                "error": {"code": -32000, "message": "execution reverted"}
            })))
            .expect(1)
            .mount(&server)
            .await;
        let req = serde_json::json!({"jsonrpc": "2.0", "method": "debug_traceCall", "id": 1});
        let err = post_jsonrpc_with_failover(&[server.uri(), server.uri()], &req, true)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("reverted"));
    }
}
