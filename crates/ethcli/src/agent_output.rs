//! Output conventions for agents and scripts, applied to every command.
//!
//! - `--fields a,b.c` trims JSON / NDJSON output to the listed fields.
//! - With JSON output, a failure is reported on stderr as one JSON object
//!   `{"error": {"code", "message", "hint"}}` with a stable `code`.
//!
//! Commands print with `println!` in many places, so `--fields` is not applied
//! inside the commands. `main` runs the same binary again without `--fields`
//! (and with `--json`), captures its stdout and projects it here.

use serde_json::{Map, Value};

/// Environment variable set on the inner run so it never re-executes itself.
pub const FIELDS_CHILD_ENV: &str = "ETHCLI_FIELDS_INNER";

/// A field path such as `balance.wei`, split into its segments.
pub type FieldPath = Vec<String>;

/// Parse a `--fields` value: comma-separated dot paths.
///
/// # Errors
///
/// Returns an error if no field is given or a path has an empty segment.
pub fn parse_fields(spec: &str) -> anyhow::Result<Vec<FieldPath>> {
    let mut paths = Vec::new();
    for raw in spec.split(',') {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let path: FieldPath = raw.split('.').map(|s| s.trim().to_string()).collect();
        if path.iter().any(String::is_empty) {
            anyhow::bail!("Invalid --fields entry '{raw}': empty path segment");
        }
        paths.push(path);
    }
    if paths.is_empty() {
        anyhow::bail!("--fields needs at least one field name, e.g. --fields hash,value");
    }
    Ok(paths)
}

/// Keep only the listed paths of `value`.
///
/// Arrays are projected element by element (at any depth), objects keep only
/// the listed keys, and missing keys are left out rather than added as null.
pub fn project(value: &Value, paths: &[FieldPath]) -> Value {
    let refs: Vec<&[String]> = paths.iter().map(Vec::as_slice).collect();
    project_refs(value, &refs)
}

fn project_refs(value: &Value, paths: &[&[String]]) -> Value {
    match value {
        Value::Array(items) => Value::Array(items.iter().map(|v| project_refs(v, paths)).collect()),
        Value::Object(map) => {
            let mut out = Map::new();
            for path in paths {
                let Some((key, _)) = path.split_first() else {
                    continue;
                };
                if out.contains_key(key) {
                    continue;
                }
                let Some(child) = map.get(key) else {
                    continue;
                };
                // Every path through this key; a path ending here keeps it whole
                let rests: Vec<&[String]> = paths
                    .iter()
                    .filter(|p| p.first() == Some(key))
                    .map(|p| &p[1..])
                    .collect();
                let projected = if rests.iter().any(|r| r.is_empty()) {
                    child.clone()
                } else {
                    project_refs(child, &rests)
                };
                out.insert(key.clone(), projected);
            }
            Value::Object(out)
        }
        // A path that goes past a scalar keeps the scalar
        other => other.clone(),
    }
}

/// Project a command's captured stdout.
///
/// One JSON document is projected and printed in the same style it came in
/// (pretty or compact); otherwise every non-empty line must be JSON (NDJSON)
/// and is projected line by line.
///
/// # Errors
///
/// Returns an error if the output is not JSON.
pub fn project_output(text: &str, paths: &[FieldPath]) -> anyhow::Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        let projected = project(&value, paths);
        let mut out = if trimmed.contains('\n') {
            serde_json::to_string_pretty(&projected)?
        } else {
            serde_json::to_string(&projected)?
        };
        out.push('\n');
        return Ok(out);
    }
    let mut out = String::new();
    for line in trimmed.lines().filter(|l| !l.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).map_err(|_| {
            anyhow::Error::new(UsageError(
                "--fields needs JSON output, but this command printed text \
                 (it may have no JSON output, or an explicit -o/--format chose another format)"
                    .to_string(),
            ))
        })?;
        out.push_str(&serde_json::to_string(&project(&value, paths))?);
        out.push('\n');
    }
    Ok(out)
}

/// Take `--fields <spec>` / `--fields=<spec>` out of the command line.
///
/// Returns the remaining arguments and the spec, if one was given. Arguments
/// after a bare `--` are left alone.
pub fn take_fields_arg(args: Vec<String>) -> (Vec<String>, Option<String>) {
    let mut out = Vec::with_capacity(args.len());
    let mut spec = None;
    let mut iter = args.into_iter();
    let mut passthrough = false;
    while let Some(arg) = iter.next() {
        if passthrough {
            out.push(arg);
            continue;
        }
        if arg == "--" {
            passthrough = true;
            out.push(arg);
        } else if arg == "--fields" {
            spec = Some(iter.next().unwrap_or_default());
        } else if let Some(value) = arg.strip_prefix("--fields=") {
            spec = Some(value.to_string());
        } else {
            out.push(arg);
        }
    }
    (out, spec)
}

/// Add `--json` (before any bare `--`), so `--fields` implies JSON output.
pub fn with_json_flag(mut args: Vec<String>) -> Vec<String> {
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    if !args[..end].iter().skip(1).any(|a| a == "--json") {
        args.insert(end, "--json".to_string());
    }
    args
}

/// Did the caller ask for JSON output (`--json`, `-o json`, `--format ndjson`,
/// `--fields`, ...)? Decides whether errors are printed as JSON.
pub fn json_output_requested(args: &[String]) -> bool {
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let args = &args[..end];
    let is_json = |v: &str| matches!(v, "json" | "ndjson" | "jsonl");
    args.iter().enumerate().skip(1).any(|(i, a)| {
        if a == "--json" || a == "--fields" || a.starts_with("--fields=") {
            return true;
        }
        for flag in ["--format", "--output"] {
            if let Some(v) = a.strip_prefix(flag).and_then(|r| r.strip_prefix('=')) {
                return is_json(v);
            }
        }
        if let Some(v) = a.strip_prefix("-o").filter(|v| !v.is_empty()) {
            return is_json(v.trim_start_matches('='));
        }
        matches!(a.as_str(), "--format" | "--output" | "-o")
            && args.get(i + 1).is_some_and(|v| is_json(v))
    })
}

/// A command-line usage error (bad flag, bad value). Classified as
/// `invalid_argument`.
#[derive(Debug)]
pub struct UsageError(pub String);

impl std::fmt::Display for UsageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for UsageError {}

/// Stable error codes for JSON error output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidArgument,
    NotVerified,
    MissingApiKey,
    Unauthorized,
    RateLimited,
    UnsupportedChain,
    NotFound,
    RpcError,
    Internal,
}

impl ErrorCode {
    /// The code as printed in JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArgument => "invalid_argument",
            Self::NotVerified => "not_verified",
            Self::MissingApiKey => "missing_api_key",
            Self::Unauthorized => "unauthorized",
            Self::RateLimited => "rate_limited",
            Self::UnsupportedChain => "unsupported_chain",
            Self::NotFound => "not_found",
            Self::RpcError => "rpc_error",
            Self::Internal => "internal",
        }
    }

    /// A suggested next step, where there is an obvious one.
    pub fn hint(self) -> Option<&'static str> {
        Some(match self {
            Self::InvalidArgument => "check the arguments with `ethcli <command> --help`",
            Self::NotVerified => {
                "no verified ABI on the explorer; try `ethcli contract source <address>` \
                 (falls back to Sourcify and Blockscout) or `ethcli contract inspect <address>`"
            }
            Self::MissingApiKey => {
                "configure the provider's key (`ethcli config --help`); `ethcli doctor` lists what is missing"
            }
            Self::Unauthorized => {
                "the configured API key was rejected; check it with `ethcli config validate`"
            }
            Self::RateLimited => {
                "wait and retry, use a keyed or different provider, or add RPC endpoints (`ethcli endpoints --help`)"
            }
            Self::UnsupportedChain => {
                "list known chains with `ethcli chainlist list`, or pass a numeric chain ID"
            }
            Self::RpcError => "check RPC endpoints with `ethcli endpoints health`",
            Self::NotFound | Self::Internal => return None,
        })
    }
}

/// Map an error to a stable code, from its type where known and otherwise
/// from the messages in its chain.
pub fn classify(err: &anyhow::Error) -> ErrorCode {
    // downcast_ref also sees a UsageError attached with `.context(...)`
    if err.downcast_ref::<UsageError>().is_some()
        || err
            .chain()
            .any(|e| e.is::<UsageError>() || e.is::<clap::Error>())
    {
        return ErrorCode::InvalidArgument;
    }
    classify_message(&format!("{err:#}"))
}

/// Map an error message to a stable code. The more specific codes are
/// checked before the broad ones (`not_found`, `rpc_error`).
pub fn classify_message(msg: &str) -> ErrorCode {
    let m = msg.to_lowercase();
    let any = |needles: &[&str]| needles.iter().any(|n| m.contains(n));
    if any(&["not verified", "source code not verified", "unverified"]) {
        ErrorCode::NotVerified
    } else if any(&[
        "api key required",
        "no api key",
        "missing api key",
        "api key not set",
        "api key not configured",
        "api key is required",
        "no alchemy api key",
        "requires an api key",
        "requires api key",
        "not configured. run",
    ]) {
        ErrorCode::MissingApiKey
    } else if any(&[
        "unauthorized",
        "invalid api key",
        "bad api key",
        "401",
        "403 forbidden",
    ]) {
        ErrorCode::Unauthorized
    } else if any(&[
        "rate limit",
        "rate-limit",
        "429",
        "too many requests",
        "max rate",
    ]) {
        ErrorCode::RateLimited
    } else if any(&[
        "unsupported chain",
        "unknown chain",
        "invalid chain",
        "chain not supported",
    ]) {
        ErrorCode::UnsupportedChain
    } else if any(&["not found", "no such", "does not exist"]) {
        ErrorCode::NotFound
    } else if any(&[
        "invalid ",
        "failed to parse",
        "must be",
        "expected ",
        "cannot be empty",
        "must not be empty",
        "unknown argument",
    ]) {
        ErrorCode::InvalidArgument
    } else if any(&[
        "rpc",
        "timed out",
        "timeout",
        "connection",
        "error sending request",
        "all endpoints",
        "502",
        "503",
        "504",
        "network",
    ]) {
        ErrorCode::RpcError
    } else {
        ErrorCode::Internal
    }
}

/// The JSON error object for `err`, on one line. The message has URLs and
/// secrets redacted.
pub fn error_json(err: &anyhow::Error) -> String {
    let code = classify(err);
    let message = format!("{err:#}");
    let message = crate::error::sanitize_error_message(message.trim()).into_owned();
    let mut body = Map::new();
    body.insert("code".into(), Value::from(code.as_str()));
    body.insert("message".into(), Value::from(message));
    if let Some(hint) = code.hint() {
        body.insert("hint".into(), Value::from(hint));
    }
    let mut root = Map::new();
    root.insert("error".into(), Value::Object(body));
    Value::Object(root).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(s: &str) -> Vec<FieldPath> {
        parse_fields(s).unwrap()
    }

    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn projects_nested_paths_and_keeps_shape() {
        let v = json!({
            "address": "0x1",
            "balance": {"wei": "42", "symbol": "ETH"},
            "code": {"bytes": 10, "hash": "0xab"},
            "chain": "Ethereum"
        });
        assert_eq!(
            project(&v, &fields("address,balance.wei,code")),
            json!({"address": "0x1", "balance": {"wei": "42"}, "code": {"bytes": 10, "hash": "0xab"}})
        );
    }

    #[test]
    fn projects_each_array_element_at_any_depth() {
        let v = json!([
            {"hash": "0x1", "value": "1", "gas": "2"},
            {"hash": "0x2", "value": "3", "gas": "4"}
        ]);
        assert_eq!(
            project(&v, &fields("hash,value")),
            json!([{"hash": "0x1", "value": "1"}, {"hash": "0x2", "value": "3"}])
        );
        let wrapped = json!({"total": 2, "items": [{"a": 1, "b": 2}, {"a": 3, "b": 4}]});
        assert_eq!(
            project(&wrapped, &fields("items.a")),
            json!({"items": [{"a": 1}, {"a": 3}]})
        );
    }

    #[test]
    fn missing_fields_are_left_out() {
        let v = json!({"a": 1});
        assert_eq!(project(&v, &fields("a,b,c.d")), json!({"a": 1}));
        // Going past a scalar keeps the scalar
        assert_eq!(project(&v, &fields("a.x")), json!({"a": 1}));
    }

    #[test]
    fn rejects_empty_field_specs() {
        assert!(parse_fields("").is_err());
        assert!(parse_fields(" , ").is_err());
        assert!(parse_fields("a..b").is_err());
        assert_eq!(
            fields(" a , b.c "),
            vec![strings(&["a"]), strings(&["b", "c"])]
        );
    }

    #[test]
    fn output_keeps_style_and_handles_ndjson() {
        let pretty = "{\n  \"a\": 1,\n  \"b\": 2\n}\n";
        assert_eq!(
            project_output(pretty, &fields("a")).unwrap(),
            "{\n  \"a\": 1\n}\n"
        );
        assert_eq!(
            project_output("{\"a\":1,\"b\":2}\n", &fields("b")).unwrap(),
            "{\"b\":2}\n"
        );
        let ndjson = "{\"a\":1,\"b\":2}\n{\"a\":3,\"b\":4}\n";
        assert_eq!(
            project_output(ndjson, &fields("a")).unwrap(),
            "{\"a\":1}\n{\"a\":3}\n"
        );
        let err = project_output("Balance: 1 ETH\n", &fields("a")).unwrap_err();
        assert_eq!(classify(&err), ErrorCode::InvalidArgument);
        assert_eq!(project_output("", &fields("a")).unwrap(), "");
    }

    #[test]
    fn takes_fields_out_of_the_command_line() {
        let (rest, spec) = take_fields_arg(strings(&["ethcli", "rpc", "block", "--fields", "a,b"]));
        assert_eq!(rest, strings(&["ethcli", "rpc", "block"]));
        assert_eq!(spec.as_deref(), Some("a,b"));
        let (rest, spec) =
            take_fields_arg(strings(&["ethcli", "--fields=a", "x", "--", "--fields"]));
        assert_eq!(rest, strings(&["ethcli", "x", "--", "--fields"]));
        assert_eq!(spec.as_deref(), Some("a"));
        assert_eq!(
            with_json_flag(strings(&["ethcli", "x", "--", "y"])),
            strings(&["ethcli", "x", "--json", "--", "y"])
        );
        assert_eq!(
            with_json_flag(strings(&["ethcli", "x", "--json"])),
            strings(&["ethcli", "x", "--json"])
        );
    }

    #[test]
    fn detects_json_output_requests() {
        for args in [
            vec!["ethcli", "rpc", "block", "1", "--json"],
            vec!["ethcli", "rpc", "block", "1", "-o", "json"],
            vec!["ethcli", "rpc", "block", "1", "-ojson"],
            vec!["ethcli", "tx", "0x1", "--output=json"],
            vec!["ethcli", "logs", "--format", "ndjson"],
            vec!["ethcli", "rpc", "block", "1", "--fields", "a"],
        ] {
            assert!(json_output_requested(&strings(&args)), "{args:?}");
        }
        for args in [
            vec!["ethcli", "rpc", "block", "1"],
            vec!["ethcli", "rpc", "block", "1", "-o", "table"],
            vec!["ethcli", "cast", "keccak", "--", "--json"],
        ] {
            assert!(!json_output_requested(&strings(&args)), "{args:?}");
        }
    }

    #[test]
    fn maps_messages_to_stable_codes() {
        let cases = [
            (
                "Failed to fetch ABI: ABI error: Contract not verified on Etherscan: 0x01",
                ErrorCode::NotVerified,
            ),
            (
                "API key required (provide key or use --stdin)",
                ErrorCode::MissingApiKey,
            ),
            (
                "No Alchemy API key configured. Run `ethcli config set-alchemy` first.",
                ErrorCode::MissingApiKey,
            ),
            ("Unauthorized: invalid API key", ErrorCode::Unauthorized),
            ("Rate limited by Alchemy (429)", ErrorCode::RateLimited),
            ("Invalid chain: foo", ErrorCode::UnsupportedChain),
            ("Unsupported chain ID: 999", ErrorCode::UnsupportedChain),
            ("Transaction not found", ErrorCode::NotFound),
            (
                "Invalid address: odd number of digits",
                ErrorCode::InvalidArgument,
            ),
            ("error sending request for url", ErrorCode::RpcError),
            ("All RPC endpoints failed", ErrorCode::RpcError),
            ("something odd happened", ErrorCode::Internal),
        ];
        for (msg, code) in cases {
            assert_eq!(classify_message(msg), code, "{msg}");
        }
    }

    #[test]
    fn error_json_shape_and_redaction() {
        let err = anyhow::anyhow!("Contract not verified on Etherscan: 0x01")
            .context("Failed to fetch ABI");
        let v: Value = serde_json::from_str(&error_json(&err)).unwrap();
        assert_eq!(v["error"]["code"], "not_verified");
        assert_eq!(
            v["error"]["message"],
            "Failed to fetch ABI: Contract not verified on Etherscan: 0x01"
        );
        assert!(v["error"]["hint"]
            .as_str()
            .unwrap()
            .contains("contract source"));

        let err = anyhow::anyhow!("weird").context(UsageError("bad flag".into()));
        let v: Value = serde_json::from_str(&error_json(&err)).unwrap();
        assert_eq!(v["error"]["code"], "invalid_argument");

        let err = anyhow::anyhow!("boom at https://rpc.example/v2/abcdefghijklmnopqrstuvwxyz123");
        let line = error_json(&err);
        assert!(!line.contains("abcdefghijklmnopqrstuvwxyz123"), "{line}");
        assert!(!line.contains('\n'));
        let v: Value = serde_json::from_str(&line).unwrap();
        assert!(v["error"].get("hint").is_some());
        let err = anyhow::anyhow!("odd");
        let v: Value = serde_json::from_str(&error_json(&err)).unwrap();
        assert!(v["error"].get("hint").is_none());
    }
}
