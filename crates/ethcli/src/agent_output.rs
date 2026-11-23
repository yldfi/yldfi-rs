//! Output conventions for agents and scripts, applied to every command.
//!
//! - `--fields a,b.c` trims JSON / NDJSON output to the listed fields.
//! - With JSON output, a failure is reported on stderr as one JSON object
//!   `{"error": {"code", "message", "hint"}}` with a stable `code`.
//!
//! Commands print with `println!` in many places, so `--fields` is not applied
//! inside the commands. `main` runs the same binary again without `--fields`
//! (and with `--json`), captures its stdout and projects it here.

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
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

/// A JSON document whose scalars are kept as their original text, so numbers
/// of any size pass through `--fields` unchanged (a `serde_json::Value` would
/// turn integers above u64 into floats). Object keys keep their order.
#[derive(Debug)]
enum Node {
    Object(Vec<(String, Node)>),
    Array(Vec<Node>),
    Scalar(Box<RawValue>),
}

/// Object entries in document order, values left unparsed.
struct Entries(Vec<(String, Box<RawValue>)>);

impl<'de> Deserialize<'de> for Entries {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Entries;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a JSON object")
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Entries, A::Error> {
                let mut out = Vec::new();
                while let Some(entry) = map.next_entry::<String, Box<RawValue>>()? {
                    out.push(entry);
                }
                Ok(Entries(out))
            }
        }
        d.deserialize_map(V)
    }
}

impl Node {
    fn parse(raw: &RawValue) -> serde_json::Result<Self> {
        match raw.get().trim_start().as_bytes().first() {
            Some(b'{') => {
                let Entries(entries) = serde_json::from_str(raw.get())?;
                entries
                    .into_iter()
                    .map(|(k, v)| Ok((k, Node::parse(&v)?)))
                    .collect::<serde_json::Result<_>>()
                    .map(Node::Object)
            }
            Some(b'[') => {
                let items: Vec<Box<RawValue>> = serde_json::from_str(raw.get())?;
                items
                    .iter()
                    .map(|v| Node::parse(v))
                    .collect::<serde_json::Result<_>>()
                    .map(Node::Array)
            }
            _ => Ok(Node::Scalar(raw.to_owned())),
        }
    }

    /// Keep only `paths`: arrays element by element (at any depth), objects
    /// only the listed keys; missing keys are left out, and a path that goes
    /// past a scalar keeps the scalar.
    fn project(self, paths: &[&[String]]) -> Self {
        match self {
            Node::Array(items) => {
                Node::Array(items.into_iter().map(|n| n.project(paths)).collect())
            }
            Node::Object(entries) => Node::Object(
                entries
                    .into_iter()
                    .filter_map(|(key, child)| {
                        let rests: Vec<&[String]> = paths
                            .iter()
                            .filter(|p| p.first() == Some(&key))
                            .map(|p| &p[1..])
                            .collect();
                        if rests.is_empty() {
                            None
                        } else if rests.iter().any(|r| r.is_empty()) {
                            // A path ending here keeps the whole value
                            Some((key, child))
                        } else {
                            Some((key, child.project(&rests)))
                        }
                    })
                    .collect(),
            ),
            scalar => scalar,
        }
    }
}

impl Serialize for Node {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeMap, SerializeSeq};
        match self {
            Node::Object(entries) => {
                let mut map = s.serialize_map(Some(entries.len()))?;
                for (k, v) in entries {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            Node::Array(items) => {
                let mut seq = s.serialize_seq(Some(items.len()))?;
                for item in items {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Node::Scalar(raw) => raw.serialize(s),
        }
    }
}

/// Project one JSON document given as text; `None` if it is not JSON.
fn project_document(text: &str, paths: &[FieldPath], pretty: bool) -> Option<String> {
    let raw: Box<RawValue> = serde_json::from_str(text).ok()?;
    let refs: Vec<&[String]> = paths.iter().map(Vec::as_slice).collect();
    let node = Node::parse(&raw).ok()?;
    // A bare number, string, bool or null (text such as `255`) has no fields
    if matches!(node, Node::Scalar(_)) {
        return None;
    }
    let node = node.project(&refs);
    if pretty {
        serde_json::to_string_pretty(&node).ok()
    } else {
        serde_json::to_string(&node).ok()
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
    if let Some(mut out) = project_document(trimmed, paths, trimmed.contains('\n')) {
        out.push('\n');
        return Ok(out);
    }
    let mut out = String::new();
    for line in trimmed.lines().filter(|l| !l.trim().is_empty()) {
        let projected = project_document(line, paths, false).ok_or_else(|| {
            anyhow::Error::new(UsageError(
                "--fields needs JSON output, but this command printed text \
                 (it may have no JSON output, or an explicit -o/--format chose another format)"
                    .to_string(),
            ))
        })?;
        out.push_str(&projected);
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

/// Does the command line ask for help or the version (`-h`, `--help`, `-V`,
/// `--version`, `--help-json`)? Those print as usual even with `--fields`.
pub fn asks_for_help_or_version(args: &[String]) -> bool {
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    args[..end].iter().skip(1).any(|a| {
        matches!(
            a.as_str(),
            "-h" | "--help" | "-V" | "--version" | "--help-json"
        )
    })
}

/// An output format given explicitly that is not JSON (`-o table`,
/// `--format=csv`), if any.
pub fn explicit_non_json_format(args: &[String]) -> Option<String> {
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let args = &args[..end];
    let is_json = |v: &str| matches!(v, "json" | "ndjson" | "jsonl");
    for (i, a) in args.iter().enumerate().skip(1) {
        let value = if matches!(a.as_str(), "--format" | "--output" | "-o") {
            args.get(i + 1).cloned()
        } else if let Some(v) = a
            .strip_prefix("--format=")
            .or_else(|| a.strip_prefix("--output="))
        {
            Some(v.to_string())
        } else {
            a.strip_prefix("-o")
                .filter(|v| !v.is_empty())
                .map(|v| v.trim_start_matches('=').to_string())
        };
        if let Some(v) = value.filter(|v| !is_json(v)) {
            return Some(v);
        }
    }
    None
}

/// The whole message of a clap error, for the JSON error object: every line
/// (argument names, possible values, usage), and for a missing subcommand
/// the help with the list of subcommands.
pub fn clap_error_message(e: &clap::Error) -> String {
    let text = e.render().to_string();
    let text = text.trim();
    let text = text.strip_prefix("error: ").unwrap_or(text);
    if e.kind() == clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand {
        format!("a subcommand is required\n\n{text}")
    } else {
        text.to_string()
    }
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

/// Environment variable that forces the error format: `json` or `text`.
pub const ERROR_FORMAT_ENV: &str = "ETHCLI_ERROR_FORMAT";

/// Should a failure be printed as a JSON object?
///
/// `ETHCLI_ERROR_FORMAT=json|text` decides when set. Otherwise errors are JSON
/// when the command line asks for JSON output, or when stdout is not a
/// terminal (a script or agent is reading it). A terminal without JSON flags
/// keeps text errors.
pub fn json_errors(args: &[String], env: Option<&str>, stdout_is_terminal: bool) -> bool {
    match env.map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("json") => true,
        Some("text") => false,
        _ => json_output_requested(args) || !stdout_is_terminal,
    }
}

/// [`json_errors`] for this process.
pub fn json_errors_for_process(args: &[String]) -> bool {
    use std::io::IsTerminal;
    let env = std::env::var(ERROR_FORMAT_ENV).ok();
    json_errors(args, env.as_deref(), std::io::stdout().is_terminal())
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

/// Map an error to a stable code. See [`classify_with_hint`].
pub fn classify(err: &anyhow::Error) -> ErrorCode {
    classify_with_hint(err).0
}

/// Map an error to a stable code and an optional hint specific to the error.
///
/// The first error in the chain with a known type decides (ethcli's own error
/// types, usage errors, HTTP and I/O errors); only when none has one is the
/// text of the whole chain matched.
pub fn classify_with_hint(err: &anyhow::Error) -> (ErrorCode, Option<&'static str>) {
    // downcast_ref also sees a UsageError attached with `.context(...)`
    if err.downcast_ref::<UsageError>().is_some() {
        return (ErrorCode::InvalidArgument, None);
    }
    err.chain()
        .find_map(typed_code)
        .unwrap_or_else(|| (classify_message(&format!("{err:#}")), None))
}

/// The code of an error whose type is known, if it is one.
fn typed_code(e: &(dyn std::error::Error + 'static)) -> Option<(ErrorCode, Option<&'static str>)> {
    use crate::error::{AbiError, ConfigError, Error, OutputError, RpcError};
    if e.is::<UsageError>() || e.is::<clap::Error>() {
        return Some((ErrorCode::InvalidArgument, None));
    }
    if let Some(e) = e.downcast_ref::<Error>() {
        return Some(e.code_and_hint());
    }
    if let Some(e) = e.downcast_ref::<RpcError>() {
        return Some(e.code_and_hint());
    }
    if let Some(e) = e.downcast_ref::<AbiError>() {
        return Some((e.code(), None));
    }
    if let Some(e) = e.downcast_ref::<ConfigError>() {
        return Some((e.code(), None));
    }
    if let Some(e) = e.downcast_ref::<OutputError>() {
        return Some((e.code(), None));
    }
    if let Some(e) = e.downcast_ref::<crate::chainlink::ChainlinkError>() {
        return Some((e.code(), None));
    }
    if let Some(e) = e.downcast_ref::<reqwest::Error>() {
        return Some((http_code(e), None));
    }
    if let Some(e) = e.downcast_ref::<std::io::Error>() {
        return Some((io_code(e), None));
    }
    None
}

/// Text matching for an error of a known type whose variant carries an
/// upstream message (e.g. an explorer's reply): the message decides when it
/// is recognised, `default` otherwise.
fn refine(msg: &str, default: ErrorCode) -> ErrorCode {
    match classify_message(msg) {
        ErrorCode::Internal => default,
        code => code,
    }
}

fn http_code(e: &reqwest::Error) -> ErrorCode {
    match e.status().map(|s| s.as_u16()) {
        Some(429) => ErrorCode::RateLimited,
        Some(401 | 403) => ErrorCode::Unauthorized,
        Some(404) => ErrorCode::NotFound,
        Some(400 | 422) => ErrorCode::InvalidArgument,
        _ => ErrorCode::RpcError,
    }
}

fn io_code(e: &std::io::Error) -> ErrorCode {
    match e.kind() {
        std::io::ErrorKind::NotFound => ErrorCode::NotFound,
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => {
            ErrorCode::InvalidArgument
        }
        std::io::ErrorKind::TimedOut
        | std::io::ErrorKind::ConnectionRefused
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::ConnectionAborted => ErrorCode::RpcError,
        _ => ErrorCode::Internal,
    }
}

impl crate::error::Error {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        self.code_and_hint().0
    }

    fn code_and_hint(&self) -> (ErrorCode, Option<&'static str>) {
        use crate::error::Error;
        match self {
            Error::Rpc(e) => e.code_and_hint(),
            Error::Abi(e) => (e.code(), None),
            Error::Config(e) => (e.code(), None),
            Error::Output(e) => (e.code(), None),
            Error::Checkpoint(_) => (ErrorCode::Internal, None),
            Error::Io(e) => (io_code(e), None),
            Error::Json(_) => (ErrorCode::Internal, None),
            Error::Other(msg) => (classify_message(msg), None),
        }
    }
}

impl crate::error::RpcError {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        self.code_and_hint().0
    }

    fn code_and_hint(&self) -> (ErrorCode, Option<&'static str>) {
        use crate::error::RpcError;
        match self {
            RpcError::NoHealthyEndpoints => (
                ErrorCode::RpcError,
                Some("add an RPC endpoint with `ethcli endpoints add <url>`"),
            ),
            RpcError::RateLimited(_) => (ErrorCode::RateLimited, None),
            RpcError::BlockRangeTooLarge { .. } | RpcError::ResponseTooLarge(_) => (
                ErrorCode::InvalidArgument,
                Some("narrow the block range (smaller --from-block/--to-block)"),
            ),
            RpcError::ProxyNotSupported(_) => (ErrorCode::InvalidArgument, None),
            RpcError::Http(e) => (http_code(e), None),
            RpcError::Provider(msg) | RpcError::InvalidResponse(msg) => {
                (refine(msg, ErrorCode::RpcError), None)
            }
            RpcError::AllEndpointsFailed | RpcError::Timeout(_) | RpcError::ConnectionFailed(_) => {
                (ErrorCode::RpcError, None)
            }
        }
    }
}

impl crate::error::AbiError {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        use crate::error::AbiError;
        match self {
            AbiError::ContractNotVerified(_) => ErrorCode::NotVerified,
            AbiError::EtherscanUnsupportedChain(_) => ErrorCode::UnsupportedChain,
            // The explorer's reply: rate limits, plan restrictions, bad keys
            AbiError::EtherscanFetch(msg) => refine(msg, ErrorCode::RpcError),
            AbiError::ParseError(msg) => refine(msg, ErrorCode::InvalidArgument),
            AbiError::InvalidEventSignature(_) | AbiError::DecodeError(_) => {
                ErrorCode::InvalidArgument
            }
            AbiError::EventNotFound(_) | AbiError::FileNotFound(_) => ErrorCode::NotFound,
            AbiError::HttpClientInit(_) => ErrorCode::Internal,
        }
    }
}

impl crate::error::ConfigError {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        use crate::error::ConfigError;
        match self {
            ConfigError::InvalidChain(_) => ErrorCode::UnsupportedChain,
            ConfigError::InvalidFile(_)
            | ConfigError::InvalidAddress(_)
            | ConfigError::InvalidBlockNumber(_)
            | ConfigError::MissingField(_)
            | ConfigError::ParseError(_) => ErrorCode::InvalidArgument,
        }
    }
}

impl crate::error::OutputError {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        use crate::error::OutputError;
        match self {
            OutputError::UnsupportedFormat(_) => ErrorCode::InvalidArgument,
            OutputError::FileCreate(_) => ErrorCode::NotFound,
            OutputError::JsonWrite(_) | OutputError::CsvWrite(_) | OutputError::Sqlite(_) => {
                ErrorCode::Internal
            }
        }
    }
}

impl crate::chainlink::ChainlinkError {
    /// Stable code for JSON error output.
    pub fn code(&self) -> ErrorCode {
        use crate::chainlink::ChainlinkError;
        match self {
            ChainlinkError::NoFeed | ChainlinkError::NoFeedAtBlock(_) => ErrorCode::NotFound,
            ChainlinkError::InvalidAddress(_) | ChainlinkError::UnknownSymbol(_) => {
                ErrorCode::InvalidArgument
            }
            ChainlinkError::UnsupportedChain(_) => ErrorCode::UnsupportedChain,
            ChainlinkError::RpcError(_)
            | ChainlinkError::ContractError(_)
            | ChainlinkError::ArchiveNodeRequired => ErrorCode::RpcError,
            ChainlinkError::StalePrice
            | ChainlinkError::UninitializedFeed
            | ChainlinkError::InvalidPrice => ErrorCode::NotFound,
        }
    }
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
        "forbidden",
    ]) || matches!(http_status(&m), Some(401 | 403))
    {
        ErrorCode::Unauthorized
    } else if any(&["rate limit", "rate-limit", "too many requests", "max rate"])
        || http_status(&m) == Some(429)
    {
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
        " required",
        "usage:",
    ]) {
        ErrorCode::InvalidArgument
    } else if any(&[
        "rpc error",
        "rpc request",
        "rpc endpoint",
        "rpc endpoints",
        "json-rpc",
        "timed out",
        "timeout",
        "connection refused",
        "connection reset",
        "connection closed",
        "failed to connect",
        "error sending request",
        "all endpoints",
        "no endpoints",
        "network error",
        "dns error",
        "bad gateway",
        "service unavailable",
        "gateway timeout",
    ]) || matches!(http_status(&m), Some(500..=599))
    {
        ErrorCode::RpcError
    } else {
        ErrorCode::Internal
    }
}

/// An HTTP status written as a status, not any three digits in the text
/// (addresses, hashes and names contain them): `status 429`, `status code:
/// 401`, `HTTP 503`, `HTTP/1.1 502`, `error code 429`, `(429)`, `429 Too Many
/// Requests`. `msg` is lowercase.
fn http_status(msg: &str) -> Option<u16> {
    use regex::Regex;
    use std::sync::LazyLock;
    static PREFIXED: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b(?:status(?: code)?|http(?:/[\d.]+)?|error code|response code)[\s:=]*\(?([1-5]\d\d)\b")
            .expect("valid status regex")
    });
    static PHRASE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"\b([45]\d\d)\s+(?:unauthorized|forbidden|not found|too many requests|bad gateway|service unavailable|gateway timeout|internal server error|bad request)\b",
        )
        .expect("valid status phrase regex")
    });
    static PAREN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"\(([45]\d\d)\)").expect("valid status paren regex"));
    [&*PREFIXED, &*PHRASE, &*PAREN]
        .iter()
        .find_map(|re| re.captures(msg))
        .and_then(|c| c[1].parse().ok())
}

/// The JSON error object for `err`, on one line. The message has URLs and
/// secrets redacted.
pub fn error_json(err: &anyhow::Error) -> String {
    let (code, hint) = classify_with_hint(err);
    let message = format!("{err:#}");
    let message = crate::error::sanitize_error_message(message.trim()).into_owned();
    let mut body = Map::new();
    body.insert("code".into(), Value::from(code.as_str()));
    body.insert("message".into(), Value::from(message));
    if let Some(hint) = hint.or_else(|| code.hint()) {
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

    /// Project a Value through the text path and read the result back
    fn project(v: &Value, paths: &[FieldPath]) -> Value {
        let out = project_output(&v.to_string(), paths).unwrap();
        serde_json::from_str(&out).unwrap()
    }

    #[test]
    fn numbers_of_any_size_pass_through_unchanged() {
        let text = r#"{"big":115792089237316195423570985008687907853269984665640564039457584007913129639935,"f":1.10,"neg":-12345678901234567890123,"drop":1}"#;
        assert_eq!(
            project_output(text, &fields("big,f,neg")).unwrap(),
            r#"{"big":115792089237316195423570985008687907853269984665640564039457584007913129639935,"f":1.10,"neg":-12345678901234567890123}"#
                .to_string()
                + "\n"
        );
        // Kept whole, nested, and pretty
        let pretty = "{\n  \"x\": {\n    \"n\": 18446744073709551616\n  },\n  \"y\": 1\n}";
        assert_eq!(
            project_output(pretty, &fields("x")).unwrap(),
            "{\n  \"x\": {\n    \"n\": 18446744073709551616\n  }\n}\n"
        );
    }

    #[test]
    fn digits_in_names_addresses_and_hex_are_not_status_codes() {
        for msg in [
            "No resolver set for fable-nonexistent-401-zz.eth",
            "No TVL data for 0x0000000000000000000000000000000000429001 on chain 1",
            "Call reverted: 0x08c379a0000000000000000000000000000000000000000000000000646401780000",
            "Name 0x403a5029... has no record",
            "block 5021 has no receipts",
        ] {
            let code = classify_message(msg);
            assert!(
                !matches!(
                    code,
                    ErrorCode::Unauthorized | ErrorCode::RateLimited | ErrorCode::RpcError
                ),
                "{msg} -> {code:?}"
            );
        }
        assert_eq!(
            classify_message("Calldata or --sig required. Use: rpc call <addr> --sig ..."),
            ErrorCode::InvalidArgument
        );
        for (msg, code) in [
            ("request failed with status 429", ErrorCode::RateLimited),
            ("HTTP 401 from upstream", ErrorCode::Unauthorized),
            ("HTTP status client error (403 Forbidden) for url", ErrorCode::Unauthorized),
            ("server said 429 Too Many Requests", ErrorCode::RateLimited),
            (
                "server returned an error response: error code 429: Monthly capacity limit exceeded",
                ErrorCode::RateLimited,
            ),
            ("status code: 503", ErrorCode::RpcError),
            ("upstream answered (502)", ErrorCode::RpcError),
        ] {
            assert_eq!(classify_message(msg), code, "{msg}");
        }
    }

    #[test]
    fn bare_scalars_are_not_projectable() {
        assert!(project_output("255\n", &fields("a")).is_err());
        assert!(project_output("\"text\"\n", &fields("a")).is_err());
        assert!(project_output("1\n2\n", &fields("a")).is_err());
    }

    #[test]
    fn help_version_and_explicit_formats() {
        assert!(asks_for_help_or_version(&strings(&[
            "ethcli", "rpc", "block", "--help"
        ])));
        assert!(asks_for_help_or_version(&strings(&["ethcli", "-V"])));
        assert!(!asks_for_help_or_version(&strings(&[
            "ethcli", "x", "--", "--help"
        ])));
        assert_eq!(
            explicit_non_json_format(&strings(&["ethcli", "rpc", "block", "1", "-o", "table"])),
            Some("table".into())
        );
        assert_eq!(
            explicit_non_json_format(&strings(&["ethcli", "x", "--format=csv"])),
            Some("csv".into())
        );
        assert_eq!(
            explicit_non_json_format(&strings(&["ethcli", "x", "-o", "ndjson"])),
            None
        );
    }

    #[test]
    fn clap_errors_keep_the_whole_message() {
        use clap::Parser;
        let err = |args: &[&str]| match crate::cli::Cli::try_parse_from(args) {
            Ok(_) => panic!("{args:?} parsed"),
            Err(e) => e,
        };
        // Missing subcommand: the list of subcommands is in the message
        let msg = clap_error_message(&err(&["ethcli", "rpc"]));
        assert!(msg.starts_with("a subcommand is required"), "{msg}");
        assert!(msg.contains("balance") && msg.contains("block"), "{msg}");
        // Missing argument: its name is kept
        let msg = clap_error_message(&err(&["ethcli", "rpc", "balance"]));
        assert!(msg.contains("<ADDRESS>"), "{msg}");
        // Invalid value: the possible values are kept
        let msg = clap_error_message(&err(&["ethcli", "account", "txs", "0x1", "-o", "xml"]));
        assert!(msg.contains("xml") && msg.contains("json"), "{msg}");
    }

    #[test]
    fn keeps_document_key_order() {
        assert_eq!(
            project_output(r#"{"z":1,"a":2,"m":3}"#, &fields("a,z")).unwrap(),
            "{\"z\":1,\"a\":2}\n"
        );
    }

    #[test]
    fn known_typed_errors_never_map_to_internal() {
        use crate::chainlink::ChainlinkError;
        use crate::error::{AbiError, ConfigError, Error, OutputError, RpcError};
        let errors: Vec<anyhow::Error> = vec![
            RpcError::AllEndpointsFailed.into(),
            RpcError::NoHealthyEndpoints.into(),
            RpcError::Timeout(1).into(),
            RpcError::RateLimited("x".into()).into(),
            RpcError::BlockRangeTooLarge {
                max: 1,
                requested: 2,
            }
            .into(),
            RpcError::ResponseTooLarge(1).into(),
            RpcError::InvalidResponse("garbled".into()).into(),
            RpcError::ConnectionFailed("x".into()).into(),
            RpcError::Provider("weird".into()).into(),
            RpcError::ProxyNotSupported("x".into()).into(),
            AbiError::EtherscanFetch("NOTOK".into()).into(),
            AbiError::EtherscanUnsupportedChain("x".into()).into(),
            AbiError::ContractNotVerified("0x1".into()).into(),
            AbiError::InvalidEventSignature("x".into()).into(),
            AbiError::ParseError("x".into()).into(),
            AbiError::EventNotFound("x".into()).into(),
            AbiError::DecodeError("x".into()).into(),
            AbiError::FileNotFound("x".into()).into(),
            ConfigError::InvalidFile("x".into()).into(),
            ConfigError::InvalidAddress("x".into()).into(),
            ConfigError::InvalidBlockNumber("x".into()).into(),
            ConfigError::InvalidChain("x".into()).into(),
            ConfigError::MissingField("x".into()).into(),
            OutputError::UnsupportedFormat("x".into()).into(),
            Error::from(AbiError::ContractNotVerified("0x1".into())).into(),
            Error::from(RpcError::Timeout(1)).into(),
            ChainlinkError::NoFeed.into(),
            ChainlinkError::UnknownSymbol("x".into()).into(),
            ChainlinkError::UnsupportedChain("x".into()).into(),
            ChainlinkError::RpcError("x".into()).into(),
            ChainlinkError::StalePrice.into(),
        ];
        for err in errors {
            assert_ne!(classify(&err), ErrorCode::Internal, "{err:#}");
        }
    }

    #[test]
    fn typed_codes_win_over_text_and_survive_context() {
        use crate::error::{AbiError, Error, RpcError};
        // The context text says "not found"; the typed error decides
        let err = anyhow::Error::from(Error::from(AbiError::ContractNotVerified("0x1".into())))
            .context("token not found in cache, fetching ABI");
        assert_eq!(classify(&err), ErrorCode::NotVerified);
        // A typed error with an upstream message is refined by the message
        let err = anyhow::Error::from(AbiError::EtherscanFetch("Max rate limit reached".into()));
        assert_eq!(classify(&err), ErrorCode::RateLimited);
        // A typed error can carry its own hint
        let v: Value =
            serde_json::from_str(&error_json(&RpcError::NoHealthyEndpoints.into())).unwrap();
        assert_eq!(v["error"]["code"], "rpc_error");
        assert!(v["error"]["hint"]
            .as_str()
            .unwrap()
            .contains("endpoints add"));
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
    fn error_format_follows_env_flags_and_terminal() {
        let plain = strings(&["ethcli", "rpc", "block", "1"]);
        let json = strings(&["ethcli", "rpc", "block", "1", "--json"]);
        // Terminal, no JSON flag: text, as before
        assert!(!json_errors(&plain, None, true));
        // Piped stdout: JSON
        assert!(json_errors(&plain, None, false));
        assert!(json_errors(&json, None, true));
        // The environment variable decides when set
        assert!(json_errors(&plain, Some("json"), true));
        assert!(!json_errors(&json, Some("TEXT"), false));
        assert!(json_errors(&plain, Some("other"), false));
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
        // "rpc" in a host name is not an RPC error
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(v["error"]["code"], "internal");
        let err = anyhow::anyhow!("odd");
        let v: Value = serde_json::from_str(&error_json(&err)).unwrap();
        assert!(v["error"].get("hint").is_none());
    }
}
