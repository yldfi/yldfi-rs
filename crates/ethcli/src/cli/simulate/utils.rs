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
            return Err(anyhow::anyhow!("Failed to encode calldata"));
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
        Ok(format!("0x{:x}", block.parse::<u64>()?))
    }
}

pub fn get_debug_rpc_url(rpc_url: &Option<String>, chain: Chain) -> Option<String> {
    if rpc_url.is_some() {
        return rpc_url.clone();
    }
    let config = ConfigFile::load_default().ok().flatten()?;
    if let Some(ep) = config
        .endpoints
        .iter()
        .find(|e| e.has_debug && e.enabled && e.chain == chain)
    {
        return Some(ep.url.clone());
    }
    if let Some(ep) = config
        .endpoints
        .iter()
        .find(|e| e.has_trace && e.enabled && e.chain == chain)
    {
        return Some(ep.url.clone());
    }
    config.debug_rpc_urls.first().cloned()
}

pub fn get_trace_rpc_url(rpc_url: &Option<String>, chain: Chain) -> Option<String> {
    if rpc_url.is_some() {
        return rpc_url.clone();
    }
    let config = ConfigFile::load_default().ok().flatten()?;
    if let Some(ep) = config
        .endpoints
        .iter()
        .find(|e| e.has_trace && e.enabled && e.chain == chain)
    {
        return Some(ep.url.clone());
    }
    config
        .endpoints
        .iter()
        .find(|e| e.has_debug && e.enabled && e.chain == chain)
        .map(|e| e.url.clone())
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
