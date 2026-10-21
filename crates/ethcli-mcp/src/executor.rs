//! Command executor for ethcli
//!
//! Runs ethcli commands as subprocesses and captures output.
//! Includes rate limiting, timeouts, input validation, and metrics.

use crate::tools::{truncate_with_note, MAX_RESPONSE_BYTES};
use std::process::{ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;
use tokio::time::{timeout, timeout_at, Instant as TokioInstant};
use tracing::{debug, info, warn};
use url::Url;

/// Maximum concurrent subprocess executions to prevent DoS
const MAX_CONCURRENT_SUBPROCESSES: usize = 10;

/// Default timeout for subprocess execution (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Fast timeout for simple operations like cast conversions (10 seconds)
const FAST_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout for Dune executions. ethcli itself waits up to 300s for results;
/// this leaves it room to time out cleanly first.
const DUNE_EXECUTE_TIMEOUT: Duration = Duration::from_secs(330);

/// Commands that use fast timeout (pure computation or local files, no network).
/// Matched exactly against the top-level command.
const FAST_COMMANDS: &[&str] = &["cast", "config", "address", "blacklist", "endpoints"];

/// Subcommands of FAST_COMMANDS that go to the network and need the default timeout
const NETWORK_SUBCOMMANDS: &[(&str, &str)] = &[
    ("blacklist", "scan"),
    ("blacklist", "scan-portfolio"),
    ("endpoints", "health"),
    ("endpoints", "test"),
    ("endpoints", "optimize"),
];

/// Maximum stderr bytes kept from a subprocess (errors are summarised to 500 chars)
const MAX_STDERR_BYTES: usize = 64 * 1024;

/// Opt-in env var for MCP tools that mutate local/remote state
const WRITE_TOOLS_ENV: &str = "ETHCLI_MCP_ENABLE_WRITE_TOOLS";

/// Flags that can carry credentials in subprocess arguments, including their
/// clap aliases (`crates/ethcli/tests/mcp_tool_paths.rs` checks the aliases
/// against the ethcli clap tree)
const SECRET_BEARING_FLAGS: &[&str] = &[
    "--show-secrets",
    "--tenderly-key",
    "--alchemy-key",
    "--etherscan-api-key",
    "--etherscan-key",
    "--rpc-header",
    "--rpc-headers",
    "--fork-header",
    "--fork-headers",
];

/// Maximum argument length to prevent memory exhaustion
const MAX_ARG_LENGTH: usize = 10_000;

/// Maximum number of arguments
const MAX_ARGS: usize = 100;

const URL_PREFIXES: &[&str] = &["https://", "http://", "wss://", "ws://"];

/// Semaphore to limit concurrent subprocess spawns
static SUBPROCESS_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

fn get_semaphore() -> &'static Semaphore {
    SUBPROCESS_SEMAPHORE.get_or_init(|| Semaphore::new(MAX_CONCURRENT_SUBPROCESSES))
}

// =============================================================================
// METRICS
// =============================================================================

/// Metrics for observability
pub struct Metrics {
    /// Total commands executed
    pub commands_total: AtomicU64,
    /// Successful commands
    pub commands_success: AtomicU64,
    /// Failed commands
    pub commands_failed: AtomicU64,
    /// Rate limited requests
    pub rate_limited: AtomicU64,
    /// Timed out commands
    pub timeouts: AtomicU64,
}

impl Metrics {
    const fn new() -> Self {
        Self {
            commands_total: AtomicU64::new(0),
            commands_success: AtomicU64::new(0),
            commands_failed: AtomicU64::new(0),
            rate_limited: AtomicU64::new(0),
            timeouts: AtomicU64::new(0),
        }
    }

    /// Get a snapshot of current metrics
    #[allow(dead_code)] // Public API for external monitoring
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            commands_total: self.commands_total.load(Ordering::Relaxed),
            commands_success: self.commands_success.load(Ordering::Relaxed),
            commands_failed: self.commands_failed.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of metrics at a point in time
#[derive(Debug, Clone)]
#[allow(dead_code)] // Public API for external monitoring
pub struct MetricsSnapshot {
    pub commands_total: u64,
    pub commands_success: u64,
    pub commands_failed: u64,
    pub rate_limited: u64,
    pub timeouts: u64,
}

/// Global metrics instance
static METRICS: Metrics = Metrics::new();

/// Get the global metrics instance
#[allow(dead_code)] // Public API for external monitoring
pub fn metrics() -> &'static Metrics {
    &METRICS
}

/// Validation errors for input sanitization
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Argument exceeds maximum length
    ArgumentTooLong { index: usize, len: usize },
    /// Too many arguments
    TooManyArguments { count: usize },
    /// Invalid Ethereum address format
    InvalidAddress(String),
    /// Invalid characters in argument
    InvalidCharacters { index: usize, reason: &'static str },
    /// Empty required argument
    EmptyArgument { index: usize },
    /// A value would be parsed by clap as a flag
    OptionLikeValue(String),
    /// A flag was added after the `--` end-of-options marker
    FlagAfterEndOfOptions(String),
    /// Stdin payload exceeds the size limit
    StdinTooLong { len: usize },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ArgumentTooLong { index, len } => {
                write!(
                    f,
                    "Argument {} too long ({} chars, max {})",
                    index, len, MAX_ARG_LENGTH
                )
            }
            Self::TooManyArguments { count } => {
                write!(f, "Too many arguments ({}, max {})", count, MAX_ARGS)
            }
            Self::InvalidAddress(addr) => {
                write!(f, "Invalid Ethereum address format: {}", addr)
            }
            Self::InvalidCharacters { index, reason } => {
                write!(f, "Invalid characters in argument {}: {}", index, reason)
            }
            Self::EmptyArgument { index } => {
                write!(f, "Empty argument at position {}", index)
            }
            Self::OptionLikeValue(value) => {
                write!(
                    f,
                    "value {:?} must not start with '-' (it would be read as a flag)",
                    value
                )
            }
            Self::FlagAfterEndOfOptions(flag) => {
                write!(f, "flag {} added after the end of options", flag)
            }
            Self::StdinTooLong { len } => {
                write!(
                    f,
                    "stdin input too long ({} bytes, max {})",
                    len, MAX_ARG_LENGTH
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Execution errors
#[derive(Debug)]
pub enum ExecutionError {
    /// Input validation failed
    Validation(ValidationError),
    /// Rate limited - too many concurrent requests
    RateLimited,
    /// Command timed out
    Timeout,
    /// Failed to spawn process
    SpawnFailed(String),
    /// Command blocked by MCP security policy
    PolicyDenied(String),
    /// Command failed with exit code
    CommandFailed { exit_code: i32, message: String },
    /// Invalid UTF-8 in output
    InvalidUtf8(String),
    /// ethcli binary not found
    BinaryNotFound(String),
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::RateLimited => write!(f, "Too many concurrent requests, please retry"),
            Self::Timeout => write!(f, "Command timed out"),
            Self::SpawnFailed(e) => write!(f, "Failed to execute command: {}", e),
            Self::PolicyDenied(message) => write!(f, "{}", message),
            Self::CommandFailed { exit_code, message } => {
                write!(f, "Command failed (exit {}): {}", exit_code, message)
            }
            Self::InvalidUtf8(e) => write!(f, "Invalid output encoding: {}", e),
            Self::BinaryNotFound(path) => write!(f, "ethcli binary not found at: {}", path),
        }
    }
}

impl std::error::Error for ExecutionError {}

// Conversion for backward compatibility with String errors
impl From<ExecutionError> for String {
    fn from(e: ExecutionError) -> String {
        e.to_string()
    }
}

/// Validate an Ethereum address format
#[allow(dead_code)] // Public API for tool validation
pub fn validate_eth_address(addr: &str) -> Result<(), ValidationError> {
    // Allow ENS names (contain dots and letters)
    if addr.contains('.') && addr.chars().any(|c| c.is_alphabetic()) {
        return Ok(());
    }

    // Must start with 0x
    if !addr.starts_with("0x") && !addr.starts_with("0X") {
        return Err(ValidationError::InvalidAddress(addr.to_string()));
    }

    // Must be 42 characters (0x + 40 hex)
    if addr.len() != 42 {
        return Err(ValidationError::InvalidAddress(addr.to_string()));
    }

    // Must be valid hex
    if !addr[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidAddress(addr.to_string()));
    }

    Ok(())
}

/// Validate a transaction hash format
#[allow(dead_code)] // Public API for tool validation
pub fn validate_tx_hash(hash: &str) -> Result<(), ValidationError> {
    if !hash.starts_with("0x") && !hash.starts_with("0X") {
        return Err(ValidationError::InvalidAddress(format!(
            "Invalid tx hash: {}",
            hash
        )));
    }

    if hash.len() != 66 {
        return Err(ValidationError::InvalidAddress(format!(
            "Invalid tx hash length: {}",
            hash
        )));
    }

    if !hash[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidAddress(format!(
            "Invalid tx hash chars: {}",
            hash
        )));
    }

    Ok(())
}

/// Validate command arguments for safety
fn validate_args(args: &[&str]) -> Result<(), ValidationError> {
    if args.len() > MAX_ARGS {
        return Err(ValidationError::TooManyArguments { count: args.len() });
    }

    for (i, arg) in args.iter().enumerate() {
        if arg.len() > MAX_ARG_LENGTH {
            return Err(ValidationError::ArgumentTooLong {
                index: i,
                len: arg.len(),
            });
        }

        // Check for null bytes which could cause issues
        if arg.contains('\0') {
            return Err(ValidationError::InvalidCharacters {
                index: i,
                reason: "contains null byte",
            });
        }
    }

    Ok(())
}

fn env_flag_enabled(name: &str) -> bool {
    std::env::var(name)
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn write_tools_enabled() -> bool {
    env_flag_enabled(WRITE_TOOLS_ENV)
}

fn has_file_output_flag(args: &[&str]) -> bool {
    args.iter()
        .any(|arg| matches!(*arg, "-o" | "--output") || arg.starts_with("--output="))
}

fn is_file_output_command(args: &[&str]) -> bool {
    match (args.first().copied(), args.get(1).copied()) {
        (Some("address"), Some("export")) => has_file_output_flag(&args[2..]),
        (Some("contract"), Some("abi" | "source")) => has_file_output_flag(&args[2..]),
        _ => false,
    }
}

fn is_mutating_dune_command(path: &[&str]) -> bool {
    matches!(
        (path.get(1).copied(), path.get(2).copied()),
        (
            Some("queries"),
            Some("create" | "update" | "archive" | "unarchive" | "make-private" | "make-public")
        ) | (
            Some("tables"),
            Some("create" | "upload-csv" | "insert" | "clear" | "delete")
        ) | (Some("matviews"), Some("upsert" | "refresh" | "delete"))
            | (Some("pipelines"), Some("execute"))
    )
}

fn is_mutating_tenderly_command(args: &[&str]) -> bool {
    match (args.get(1).copied(), args.get(2).copied()) {
        (Some("vnets"), Some("create" | "delete" | "update" | "fork" | "send")) => true,
        (Some("vnets"), Some("admin")) => {
            let Some(mut index) = args
                .iter()
                .position(|arg| *arg == "admin")
                .map(|index| index + 1)
            else {
                return true;
            };

            let mut action = None;
            while index < args.len() {
                let arg = args[index];
                if arg.starts_with('-') {
                    index += if arg.contains('=') { 1 } else { 2 };
                    continue;
                }

                action = Some(arg);
                break;
            }

            !matches!(
                action,
                Some("get-latest" | "simulate-tx" | "simulate-bundle" | "create-access-list")
            )
        }
        (Some("actions"), Some("stop-many" | "resume-many")) => true,
        (Some("alerts"), Some("update" | "add-destination" | "remove-destination")) => true,
        (Some("contracts"), Some("update" | "remove-tag" | "bulk-tag")) => true,
        _ => false,
    }
}

/// The command path of `args`: the leading words, skipping `--flag value`
/// pairs that builders insert before the leaf subcommand (e.g.
/// `alchemy nft -n eth-mainnet report-spam`).
fn command_path<'a>(args: &[&'a str], depth: usize) -> Vec<&'a str> {
    let mut path = Vec::with_capacity(depth);
    let mut index = 0;
    while index < args.len() && path.len() < depth {
        let arg = args[index];
        if arg == "--" {
            break;
        }
        if arg.starts_with('-') {
            index += if arg.contains('=') { 1 } else { 2 };
            continue;
        }
        path.push(arg);
        index += 1;
    }
    path
}

fn has_flag(args: &[&str], flag: &str) -> bool {
    args.iter()
        .take_while(|arg| **arg != "--")
        .any(|arg| *arg == flag || arg.starts_with(&format!("{flag}=")))
}

pub(crate) fn is_mutating_command(args: &[&str]) -> bool {
    let path = command_path(args, 3);
    match (path.first().copied(), path.get(1).copied()) {
        (Some("config"), Some(sub)) => matches!(
            sub,
            "init"
                | "set-etherscan-key"
                | "set-tenderly"
                | "set-alchemy"
                | "set-alchemy-notify-token"
                | "set-alchemy-access-key"
                | "set-moralis"
                | "set-chainlink"
                | "set-dune"
                | "set-solodit"
                | "set-pyth"
                | "add-debug-rpc"
                | "remove-debug-rpc"
        ),
        (Some("address"), Some("add" | "remove" | "import")) => true,
        (Some("blacklist"), Some("add" | "remove")) => true,
        (Some("blacklist"), Some("scan" | "scan-portfolio")) => has_flag(args, "--auto-blacklist"),
        (Some("endpoints"), Some("add" | "remove" | "enable" | "disable" | "optimize")) => true,
        (Some("chainlist"), Some("add")) => true,
        (Some("sig"), Some("cache-clear")) => true,
        (Some("simulate"), Some("share" | "unshare")) => true,
        (Some("simulate"), Some(_)) => has_flag(args, "--save"),
        (Some("dune"), _) => is_mutating_dune_command(&path),
        (Some("tenderly"), _) => is_mutating_tenderly_command(args),
        (Some("cow-swap"), Some("create-order" | "cancel-order" | "cancel-orders")) => true,
        (Some("alchemy"), Some("nft")) => {
            matches!(
                path.get(2).copied(),
                Some("report-spam" | "refresh-metadata")
            )
        }
        (Some("moralis"), Some("nft")) => matches!(
            path.get(2).copied(),
            Some("sync-collection" | "resync-metadata" | "resync-traits")
        ),
        _ => false,
    }
}

fn describe_command(args: &[&str]) -> String {
    args.iter().take(3).copied().collect::<Vec<_>>().join(" ")
}

fn secret_bearing_flag<'a>(args: &[&'a str]) -> Option<&'a str> {
    args.iter().copied().find(|arg| {
        SECRET_BEARING_FLAGS.contains(arg)
            || SECRET_BEARING_FLAGS
                .iter()
                .any(|flag| arg.starts_with(&format!("{flag}=")))
    })
}

fn validate_command_policy(args: &[&str], has_stdin: bool) -> Result<(), ExecutionError> {
    // `--stdin` makes ethcli read the payload from stdin; only builders that
    // supply one (config_set_*) may use it.
    if !has_stdin && has_flag(args, "--stdin") {
        return Err(ExecutionError::PolicyDenied(
            "--stdin is only available to tools that provide their own stdin input".to_string(),
        ));
    }

    if let Some(flag) = secret_bearing_flag(args) {
        return Err(ExecutionError::PolicyDenied(format!(
            "{flag} is disabled over ethcli-mcp because it can expose credentials through subprocess arguments"
        )));
    }

    if is_file_output_command(args) {
        return Err(ExecutionError::PolicyDenied(
            "File output paths are disabled over ethcli-mcp; omit output to receive data in the tool response".to_string(),
        ));
    }

    if is_mutating_command(args) && !write_tools_enabled() {
        return Err(ExecutionError::PolicyDenied(format!(
            "{} is disabled in ethcli-mcp read-only mode; set {WRITE_TOOLS_ENV}=1 to enable mutating tools",
            describe_command(args)
        )));
    }

    Ok(())
}

/// Sanitize error messages to prevent information leakage
fn sanitize_error(stderr: &str, stdout: &str) -> String {
    let mut message = String::new();

    // Extract useful error info without exposing sensitive details
    for line in stderr.lines().chain(stdout.lines()) {
        // Transport errors quote request URLs, which can carry API keys
        let redacted = redact_sensitive_urls(line.trim());
        // Only assignments (`access_key = …`): "Invalid API key" stays readable
        let line = if redacted.contains('=') {
            redact_secret_line(&redacted).unwrap_or(redacted)
        } else {
            redacted
        };
        let line = line.as_str();

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Skip lines that might contain sensitive info
        if line.contains("API_KEY")
            || line.contains("api_key")
            || line.contains("Bearer ")
            || line.contains("Authorization:")
            || line.starts_with("at /")
            || line.contains("/Users/")
            || line.contains("/home/")
            || line.contains("C:\\Users\\")
            || line.contains("C:/Users/")
            || line.contains("\\AppData\\")
            || line.contains("PRIVATE")
            || line.contains("SECRET")
            || line.contains("PASSWORD")
            || line.contains("password")
            || line.contains("TOKEN=")
            || line.contains("token=")
        {
            continue;
        }

        // Limit message length, cutting a long line rather than dropping it
        if message.len() + line.len() > 500 {
            let room = 500usize.saturating_sub(message.len() + 2);
            if room > 0 {
                let mut cut = room.min(line.len());
                while !line.is_char_boundary(cut) {
                    cut -= 1;
                }
                if !message.is_empty() {
                    message.push_str("; ");
                }
                message.push_str(&line[..cut]);
            }
            message.push_str("...");
            break;
        }

        if !message.is_empty() {
            message.push_str("; ");
        }
        message.push_str(line);
    }

    if message.is_empty() {
        "Command failed".to_string()
    } else {
        message
    }
}

fn contains_secret_key_name(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("api key")
        || lower.contains("access_key")
        || lower.contains("access key")
        || lower.contains("x-access-key")
        || lower.contains("access-key")
        || lower.contains("notify_token")
        || lower.contains("x-alchemy-token")
        || lower.contains("user_secret")
        || lower.contains("client_secret")
        || lower.contains("authorization")
        || lower.contains("auth_token")
        || lower.contains("bearer")
        // `auth_token`, `apitoken`, `x-alchemy-token`: but not `tokenIn` or `tokens`
        || (lower.ends_with("token") && lower.len() > "token".len())
}

fn is_hex_value(segment: &str) -> bool {
    segment
        .strip_prefix("0x")
        .is_some_and(|hex| !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit()))
}

fn should_redact_path_segment(host: &str, previous: Option<&str>, segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    let lower = segment.to_ascii_lowercase();
    if contains_secret_key_name(&lower) {
        return true;
    }

    // Addresses and hashes (`/token/0x…`) are data, not keys
    if is_hex_value(segment) {
        return false;
    }

    let previous = previous.map(str::to_ascii_lowercase);
    if matches!(previous.as_deref(), Some("v2" | "v3" | "key" | "token")) && segment.len() >= 12 {
        return true;
    }

    let known_keyed_rpc_host = [
        "alchemy.com",
        "infura.io",
        "quicknode",
        "quiknode",
        "blastapi",
        "drpc",
        "ankr",
        "chainstack",
        "tenderly.co",
    ]
    .iter()
    .any(|provider| host.contains(provider));

    known_keyed_rpc_host
        && segment.len() >= 16
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn redact_url_candidate(candidate: &str) -> String {
    let Ok(mut url) = Url::parse(candidate) else {
        return candidate.to_string();
    };

    if !url.username().is_empty() {
        let _ = url.set_username("redacted");
    }
    if url.password().is_some() {
        let _ = url.set_password(Some("redacted"));
    }

    if url.query().is_some() {
        let pairs: Vec<(String, String)> = url
            .query_pairs()
            .map(|(key, value)| {
                let key = key.into_owned();
                let value = if contains_secret_key_name(&key) || key.eq_ignore_ascii_case("token") {
                    "redacted".to_string()
                } else {
                    value.into_owned()
                };
                (key, value)
            })
            .collect();

        url.set_query(None);
        {
            let mut query = url.query_pairs_mut();
            for (key, value) in pairs {
                query.append_pair(&key, &value);
            }
        }
    }

    let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
    if let Some(segments) = url.path_segments() {
        let segments: Vec<String> = segments
            .scan(None::<String>, |previous, segment| {
                let redacted = if should_redact_path_segment(&host, previous.as_deref(), segment) {
                    "redacted".to_string()
                } else {
                    segment.to_string()
                };
                *previous = Some(segment.to_string());
                Some(redacted)
            })
            .collect();

        if let Ok(mut path) = url.path_segments_mut() {
            path.clear();
            path.extend(segments.iter().map(String::as_str));
        }
    }

    url.to_string()
}

fn find_next_url_start(text: &str) -> Option<usize> {
    URL_PREFIXES
        .iter()
        .filter_map(|prefix| text.find(prefix))
        .min()
}

fn is_url_delimiter(byte: u8) -> bool {
    matches!(
        byte,
        b' ' | b'\n' | b'\r' | b'\t' | b'"' | b'\'' | b'<' | b'>' | b')' | b']' | b'}'
    )
}

fn redact_sensitive_urls(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = find_next_url_start(remaining) {
        output.push_str(&remaining[..start]);
        let url_and_after = &remaining[start..];
        let end = url_and_after
            .bytes()
            .position(is_url_delimiter)
            .unwrap_or(url_and_after.len());
        let candidate = &url_and_after[..end];
        output.push_str(&redact_url_candidate(candidate));
        remaining = &url_and_after[end..];
    }

    output.push_str(remaining);
    output
}

fn redact_secret_line(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    if URL_PREFIXES.iter().any(|prefix| lower.contains(prefix)) {
        return None;
    }

    let looks_secret = lower.contains("api_key")
        || lower.contains("access_key")
        || lower.contains("api key")
        || lower.contains("access key")
        || lower.contains("user_secret")
        || lower.contains("client_secret")
        || lower.contains("x-access-key")
        || lower.contains("notify_token")
        || lower.contains("auth_token")
        || lower.contains("x-alchemy-token")
        || lower.contains("authorization:");

    if !looks_secret {
        return None;
    }

    if let Some(index) = line.find('=') {
        return Some(format!("{} <redacted>", line[..=index].trim_end()));
    }
    if let Some(index) = line.find(':') {
        return Some(format!("{}: <redacted>", line[..index].trim_end()));
    }

    Some("<redacted>".to_string())
}

fn sanitize_success_output(stdout: &str) -> String {
    let redacted = redact_sensitive_urls(stdout);
    redacted
        .lines()
        .map(|line| redact_secret_line(line).unwrap_or_else(|| line.to_string()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Verify the ethcli binary is valid and executable
fn verify_binary(path: &std::path::Path) -> Result<(), ExecutionError> {
    let metadata = std::fs::metadata(path)
        .map_err(|e| ExecutionError::BinaryNotFound(format!("{}: {}", path.display(), e)))?;

    // Check it's a file
    if !metadata.is_file() {
        return Err(ExecutionError::BinaryNotFound(format!(
            "{} is not a file",
            path.display()
        )));
    }

    // Check it's executable (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = metadata.permissions().mode();
        if mode & 0o111 == 0 {
            return Err(ExecutionError::BinaryNotFound(format!(
                "{} is not executable",
                path.display()
            )));
        }
    }

    Ok(())
}

/// Find the ethcli binary path
pub(crate) fn find_ethcli_binary() -> Result<std::path::PathBuf, ExecutionError> {
    // First check environment variable override
    if let Ok(path) = std::env::var("ETHCLI_PATH") {
        let path = std::path::PathBuf::from(&path);
        if !path.is_absolute() {
            return Err(ExecutionError::BinaryNotFound(
                "ETHCLI_PATH must be an absolute path".to_string(),
            ));
        }
        verify_binary(&path)?;
        return Ok(path);
    }

    // Check same directory as this binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let ethcli = dir.join(format!("ethcli{}", std::env::consts::EXE_SUFFIX));
            if ethcli.exists() {
                verify_binary(&ethcli)?;
                return Ok(ethcli);
            }
        }
    }

    Err(ExecutionError::BinaryNotFound(
        "ethcli not found next to ethcli-mcp; set ETHCLI_PATH to an absolute ethcli binary path"
            .to_string(),
    ))
}

/// Get timeout duration based on command type
fn get_timeout(args: &[&str]) -> Duration {
    let path = command_path(args, 3);
    let command = path.first().copied().unwrap_or("unknown");
    let sub = path.get(1).copied().unwrap_or("");

    if command == "dune" && matches!(path.get(1..3), Some(["sql" | "query", "execute"])) {
        return DUNE_EXECUTE_TIMEOUT;
    }

    if FAST_COMMANDS.contains(&command) && !NETWORK_SUBCOMMANDS.contains(&(command, sub)) {
        FAST_TIMEOUT
    } else {
        DEFAULT_TIMEOUT
    }
}

/// Bytes read from a subprocess pipe, capped in memory
struct PipeOutput {
    data: Vec<u8>,
    /// Total bytes the process wrote, including any beyond the cap
    total: usize,
}

type PipeTask = JoinHandle<std::io::Result<PipeOutput>>;

/// Read a pipe to EOF, keeping at most `max_bytes`. The rest is drained and
/// discarded so the child never blocks on a full pipe.
fn read_pipe<R>(mut pipe: R, max_bytes: usize) -> PipeTask
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut data = Vec::new();
        let mut total = 0;
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let n = pipe.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            total += n;
            let room = max_bytes.saturating_sub(data.len());
            data.extend_from_slice(&chunk[..n.min(room)]);
        }
        Ok(PipeOutput { data, total })
    })
}

async fn collect_output(
    mut task: PipeTask,
    stream: &str,
    deadline: TokioInstant,
) -> Result<PipeOutput, ExecutionError> {
    match timeout_at(deadline, &mut task).await {
        Ok(joined) => joined
            .map_err(|e| ExecutionError::SpawnFailed(format!("Failed to join {stream}: {e}")))?
            .map_err(|e| ExecutionError::SpawnFailed(format!("Failed to read {stream}: {e}"))),
        Err(_) => {
            task.abort();
            Err(ExecutionError::Timeout)
        }
    }
}

fn abort_output_task(task: Option<PipeTask>) {
    if let Some(task) = task {
        task.abort();
    }
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    // The child is spawned into a new process group whose PGID equals its PID.
    // Killing -PGID also terminates grandchildren such as anvil/cast.
    unsafe {
        libc::kill(-(pid as libc::pid_t), libc::SIGKILL);
    }
}

#[cfg(not(unix))]
fn kill_process_group(_pid: u32) {}

/// Output of a finished subprocess
struct CommandOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    /// Bytes written to stdout, including any beyond the read cap
    stdout_total: usize,
    stderr: Vec<u8>,
}

/// Set up the child's stdio. stdin is closed unless the tool supplies a
/// payload: ethcli-mcp's own stdin is the MCP transport, so a child that
/// inherited it would consume the client's JSON-RPC messages.
fn configure_stdio(cmd: &mut Command, has_stdin: bool) {
    cmd.stdin(if has_stdin {
        Stdio::piped()
    } else {
        Stdio::null()
    })
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
}

async fn run_command_with_timeout(
    mut cmd: Command,
    cmd_timeout: Duration,
    stdin_payload: Option<&str>,
) -> Result<CommandOutput, ExecutionError> {
    #[cfg(unix)]
    cmd.process_group(0);

    cmd.kill_on_drop(true);
    let deadline = TokioInstant::now() + cmd_timeout;

    let mut child = cmd
        .spawn()
        .map_err(|e| ExecutionError::SpawnFailed(e.to_string()))?;
    let child_pid = child.id();
    let stdout_task = child
        .stdout
        .take()
        .map(|pipe| read_pipe(pipe, MAX_RESPONSE_BYTES + 1));
    let stderr_task = child
        .stderr
        .take()
        .map(|pipe| read_pipe(pipe, MAX_STDERR_BYTES));

    if let (Some(payload), Some(mut stdin)) = (stdin_payload, child.stdin.take()) {
        let write = async {
            stdin.write_all(payload.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.shutdown().await
        };
        // A write error means the child exited early; its status reports why.
        if let Err(e) = timeout_at(deadline, write).await.unwrap_or(Ok(())) {
            debug!(error = %e, "Failed to write subprocess stdin");
        }
        // Dropping `stdin` here closes the pipe so the child sees EOF.
    }

    let status = match timeout_at(deadline, child.wait()).await {
        Ok(Ok(status)) => status,
        Ok(Err(e)) => {
            abort_output_task(stdout_task);
            abort_output_task(stderr_task);
            return Err(ExecutionError::SpawnFailed(e.to_string()));
        }
        Err(_) => {
            if let Some(pid) = child_pid {
                kill_process_group(pid);
            }
            let _ = child.kill().await;
            let _ = timeout(Duration::from_secs(2), child.wait()).await;
            abort_output_task(stdout_task);
            abort_output_task(stderr_task);
            return Err(ExecutionError::Timeout);
        }
    };

    let stdout = if let Some(task) = stdout_task {
        match collect_output(task, "stdout", deadline).await {
            Ok(output) => output,
            Err(e @ ExecutionError::Timeout) => {
                if let Some(pid) = child_pid {
                    kill_process_group(pid);
                }
                abort_output_task(stderr_task);
                return Err(e);
            }
            Err(e) => {
                abort_output_task(stderr_task);
                return Err(e);
            }
        }
    } else {
        PipeOutput {
            data: Vec::new(),
            total: 0,
        }
    };

    let stderr = if let Some(task) = stderr_task {
        match collect_output(task, "stderr", deadline).await {
            Ok(output) => output.data,
            Err(e @ ExecutionError::Timeout) => {
                if let Some(pid) = child_pid {
                    kill_process_group(pid);
                }
                return Err(e);
            }
            Err(e) => return Err(e),
        }
    } else {
        Vec::new()
    };

    Ok(CommandOutput {
        status,
        stdout: stdout.data,
        stdout_total: stdout.total,
        stderr,
    })
}

/// Decode stdout. Output that hit the read cap is cut back to a char boundary
/// and truncated with a note giving the real size.
fn decode_stdout(mut stdout: Vec<u8>, total: usize) -> Result<String, ExecutionError> {
    if total <= MAX_RESPONSE_BYTES {
        return String::from_utf8(stdout).map_err(|e| ExecutionError::InvalidUtf8(e.to_string()));
    }
    let valid = match std::str::from_utf8(&stdout) {
        Ok(_) => stdout.len(),
        // A multi-byte char split by the cap: drop the partial char
        Err(e) if e.error_len().is_none() => e.valid_up_to(),
        Err(e) => return Err(ExecutionError::InvalidUtf8(e.to_string())),
    };
    stdout.truncate(valid);
    let text = String::from_utf8(stdout).map_err(|e| ExecutionError::InvalidUtf8(e.to_string()))?;
    // Leave room for the note so the tool response cap does not cut it off.
    Ok(truncate_with_note(text, MAX_RESPONSE_BYTES - 1024, total))
}

/// Execute an ethcli command and return the output
///
/// This function:
/// - Validates all arguments for safety
/// - Limits concurrent subprocess execution
/// - Applies a timeout to prevent hanging
/// - Writes `stdin_payload` to the child's stdin (stdin is closed otherwise)
/// - Sanitizes error messages
pub async fn execute_validated(
    args: &[&str],
    stdin_payload: Option<&str>,
) -> Result<String, ExecutionError> {
    let start = Instant::now();
    let command = args.first().copied().unwrap_or("unknown");

    // Track total commands
    METRICS.commands_total.fetch_add(1, Ordering::Relaxed);

    // Validate arguments
    validate_args(args).map_err(ExecutionError::Validation)?;
    if let Some(payload) = stdin_payload {
        if payload.len() > MAX_ARG_LENGTH {
            return Err(ExecutionError::Validation(ValidationError::StdinTooLong {
                len: payload.len(),
            }));
        }
    }
    validate_command_policy(args, stdin_payload.is_some())?;

    // Acquire semaphore permit (rate limiting)
    let _permit = match get_semaphore().try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            METRICS.rate_limited.fetch_add(1, Ordering::Relaxed);
            warn!(command = %command, "Rate limited");
            return Err(ExecutionError::RateLimited);
        }
    };

    // Find ethcli binary
    let ethcli_path = find_ethcli_binary()?;

    let mut cmd = Command::new(&ethcli_path);
    cmd.args(args);
    configure_stdio(&mut cmd, stdin_payload.is_some());

    let cmd_timeout = get_timeout(args);
    debug!(command = %command, timeout_secs = %cmd_timeout.as_secs(), "Executing ethcli command");

    // Execute with tiered timeout
    let output = match run_command_with_timeout(cmd, cmd_timeout, stdin_payload).await {
        Ok(output) => output,
        Err(ExecutionError::Timeout) => {
            METRICS.timeouts.fetch_add(1, Ordering::Relaxed);
            warn!(command = %command, "Command timed out");
            return Err(ExecutionError::Timeout);
        }
        Err(e) => {
            METRICS.commands_failed.fetch_add(1, Ordering::Relaxed);
            return Err(e);
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if output.status.success() {
        METRICS.commands_success.fetch_add(1, Ordering::Relaxed);
        debug!(command = %command, duration_ms = %duration_ms, "Command succeeded");
        let stdout = decode_stdout(output.stdout, output.stdout_total)?;
        Ok(sanitize_success_output(&stdout))
    } else {
        METRICS.commands_failed.fetch_add(1, Ordering::Relaxed);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let exit_code = output.status.code().unwrap_or(-1);
        info!(command = %command, exit_code = %exit_code, duration_ms = %duration_ms, "Command failed");
        Err(ExecutionError::CommandFailed {
            exit_code,
            message: sanitize_error(&stderr, &stdout),
        })
    }
}

/// Build command args from optional parameters.
///
/// Tool inputs never become flags: a positional or subcommand value starting
/// with `-` is rejected, and an option value starting with `-` is passed as
/// `--flag=value`. Positionals that may legitimately start with `-` (negative
/// numbers, SQL comments) go after [`ArgsBuilder::end_of_options`].
/// Validation errors are reported by [`ArgsBuilder::execute`].
pub struct ArgsBuilder {
    args: Vec<String>,
    stdin: Option<String>,
    end_of_options: bool,
    error: Option<ValidationError>,
}

impl ArgsBuilder {
    pub fn new(command: &str) -> Self {
        Self {
            args: vec![command.to_string()],
            stdin: None,
            end_of_options: false,
            error: None,
        }
    }

    fn fail(&mut self, error: ValidationError) {
        self.error.get_or_insert(error);
    }

    /// Push a positional value, rejecting anything clap would read as a flag
    fn push_value(&mut self, value: &str) {
        if !self.end_of_options && value.starts_with('-') {
            self.fail(ValidationError::OptionLikeValue(value.to_string()));
        }
        self.args.push(value.to_string());
    }

    fn push_flag(&mut self, flag: &str) -> bool {
        if self.end_of_options {
            self.fail(ValidationError::FlagAfterEndOfOptions(flag.to_string()));
            return false;
        }
        true
    }

    pub fn subcommand(mut self, sub: &str) -> Self {
        if sub.starts_with('-') {
            self.fail(ValidationError::OptionLikeValue(sub.to_string()));
        }
        self.args.push(sub.to_string());
        self
    }

    pub fn arg(mut self, value: &str) -> Self {
        self.push_value(value);
        self
    }

    pub fn opt(mut self, flag: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            if !self.push_flag(flag) {
                return self;
            }
            if v.starts_with('-') {
                // `--flag=-x` keeps clap from reading the value as a flag
                self.args.push(format!("{flag}={v}"));
            } else {
                self.args.push(flag.to_string());
                self.args.push(v.to_string());
            }
        }
        self
    }

    pub fn opt_flag(mut self, flag: &str, enabled: bool) -> Self {
        if enabled && self.push_flag(flag) {
            self.args.push(flag.to_string());
        }
        self
    }

    /// Insert `--`: everything after it is positional, so values may start
    /// with `-`. No flags can be added afterwards.
    pub fn end_of_options(mut self) -> Self {
        if !self.end_of_options {
            self.args.push("--".to_string());
            self.end_of_options = true;
        }
        self
    }

    /// Write `value` to the child's stdin instead of passing it in argv,
    /// where other local users could read it from the process list.
    pub fn stdin(mut self, value: &str) -> Self {
        self.stdin = Some(value.to_string());
        self
    }

    pub fn chain(self, chain: Option<&str>) -> Self {
        self.opt("--chain", chain)
    }

    /// Set -n/--network flag (for Alchemy commands where --network is defined
    /// on intermediate subcommands, not leaf subcommands). Must be called
    /// BEFORE the leaf .subcommand() call.
    pub fn network(self, network: Option<&str>) -> Self {
        self.opt("-n", network)
    }

    pub fn format_json(self) -> Self {
        // Use -o (short flag) because ethcli commands inconsistently use
        // --format or --output for the long flag, but all use -o for short
        self.opt("-o", Some("json"))
    }

    /// The argv built so far (for tests)
    #[cfg(test)]
    pub fn args(&self) -> &[String] {
        &self.args
    }

    pub async fn execute(self) -> Result<String, String> {
        if let Some(error) = self.error {
            return Err(ExecutionError::Validation(error).to_string());
        }
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        execute_validated(&args, self.stdin.as_deref())
            .await
            .map_err(|e| e.to_string())
    }
}

// =============================================================================
// UNIT TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // validate_eth_address tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_eth_address_valid() {
        assert!(validate_eth_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_ok());
        assert!(validate_eth_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").is_ok());
        assert!(validate_eth_address("0x0000000000000000000000000000000000000000").is_ok());
    }

    #[test]
    fn test_validate_eth_address_ens() {
        assert!(validate_eth_address("vitalik.eth").is_ok());
        assert!(validate_eth_address("foo.bar.eth").is_ok());
    }

    #[test]
    fn test_validate_eth_address_invalid_prefix() {
        assert!(validate_eth_address("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_err());
        assert!(validate_eth_address("1xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").is_err());
    }

    #[test]
    fn test_validate_eth_address_invalid_length() {
        assert!(validate_eth_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA9604").is_err()); // 41 chars
        assert!(validate_eth_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA960455").is_err());
        // 43 chars
    }

    #[test]
    fn test_validate_eth_address_invalid_hex() {
        assert!(validate_eth_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA9604G").is_err());
        assert!(validate_eth_address("0xZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ").is_err());
    }

    // -------------------------------------------------------------------------
    // validate_tx_hash tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_tx_hash_valid() {
        assert!(validate_tx_hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
        .is_ok());
    }

    #[test]
    fn test_validate_tx_hash_invalid_prefix() {
        assert!(validate_tx_hash(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )
        .is_err());
    }

    #[test]
    fn test_validate_tx_hash_invalid_length() {
        assert!(validate_tx_hash("0x1234567890abcdef").is_err());
        assert!(validate_tx_hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef00"
        )
        .is_err());
    }

    // -------------------------------------------------------------------------
    // validate_args tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_args_valid() {
        assert!(validate_args(&["arg1", "arg2", "arg3"]).is_ok());
        assert!(validate_args(&[]).is_ok());
    }

    #[test]
    fn test_validate_args_too_many() {
        let args: Vec<&str> = (0..101).map(|_| "arg").collect();
        assert!(matches!(
            validate_args(&args),
            Err(ValidationError::TooManyArguments { .. })
        ));
    }

    #[test]
    fn test_validate_args_too_long() {
        let long_arg = "x".repeat(10_001);
        assert!(matches!(
            validate_args(&[&long_arg]),
            Err(ValidationError::ArgumentTooLong { .. })
        ));
    }

    #[test]
    fn test_validate_args_null_byte() {
        assert!(matches!(
            validate_args(&["arg\0with\0nulls"]),
            Err(ValidationError::InvalidCharacters { .. })
        ));
    }

    // -------------------------------------------------------------------------
    // sanitize_error tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_sanitize_error_filters_api_key() {
        let result = sanitize_error("Error: API_KEY=secret123", "");
        assert!(!result.contains("API_KEY"));
        assert!(!result.contains("secret123"));
    }

    #[test]
    fn test_sanitize_error_filters_paths() {
        let result = sanitize_error("Error at /Users/john/secret/file.rs", "");
        assert!(!result.contains("/Users/"));

        let result = sanitize_error("Error at /home/user/.config", "");
        assert!(!result.contains("/home/"));

        let result = sanitize_error("Error at C:\\Users\\john\\secrets", "");
        assert!(!result.contains("C:\\Users\\"));
    }

    #[test]
    fn test_sanitize_error_filters_tokens() {
        let result = sanitize_error("Authorization: Bearer xyz123", "");
        assert!(!result.contains("Bearer"));

        let result = sanitize_error("TOKEN=abc123", "");
        assert!(!result.contains("TOKEN="));
    }

    #[test]
    fn test_sanitize_error_preserves_safe_messages() {
        let result = sanitize_error("Connection timeout", "");
        assert_eq!(result, "Connection timeout");

        let result = sanitize_error("Invalid argument: foo", "");
        assert_eq!(result, "Invalid argument: foo");
    }

    #[test]
    fn test_sanitize_error_redacts_keyed_urls() {
        let result = sanitize_error(
            "error sending request for url (https://eth-mainnet.g.alchemy.com/v2/abcdefghijklmnop1234): timed out",
            "",
        );
        assert!(!result.contains("abcdefghijklmnop1234"), "{result}");
        assert!(result.contains("timed out"));

        let result = sanitize_error(
            "request failed: https://api.example.com/x?apikey=SECRET123",
            "",
        );
        assert!(!result.contains("SECRET123"), "{result}");
    }

    #[test]
    fn test_sanitize_error_keeps_plain_key_messages() {
        assert_eq!(
            sanitize_error("Error: Invalid API key", ""),
            "Error: Invalid API key"
        );
    }

    #[test]
    fn test_sanitize_error_cuts_long_first_line() {
        let line = format!("Error: {}", "é".repeat(400));
        let result = sanitize_error(&line, "");
        assert!(result.starts_with("Error: é"), "{result}");
        assert!(result.ends_with("..."));
        assert!(result.len() <= 503);
    }

    #[test]
    fn test_token_paths_and_params_are_not_redacted() {
        let url = "https://etherscan.io/token/0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48?a=1";
        assert_eq!(redact_url_candidate(url), url);
        let quote = "https://api.example.com/quote?tokenIn=0xabc&tokenOut=0xdef&amount=1";
        assert_eq!(redact_url_candidate(quote), quote);

        // Real token parameters are still redacted
        let keyed =
            redact_url_candidate("https://api.example.com/x?token=s3cret&auth_token=s3cret");
        assert!(!keyed.contains("s3cret"), "{keyed}");
        assert!(contains_secret_key_name("x-alchemy-token"));
        assert!(!contains_secret_key_name("tokenIn"));
        assert!(!contains_secret_key_name("tokens"));
    }

    #[test]
    fn test_sanitize_error_truncates_long_messages() {
        let long_line = "x".repeat(600);
        let result = sanitize_error(&long_line, "");
        assert!(result.len() <= 503); // 500 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_sanitize_error_empty_returns_default() {
        let result = sanitize_error("", "");
        assert_eq!(result, "Command failed");
    }

    #[test]
    fn test_sanitize_success_output_redacts_keyed_rpc_urls() {
        let output = sanitize_success_output(
            "Endpoint https://eth-mainnet.g.alchemy.com/v2/supersecretapikey1234567890?api_key=abc123 is healthy",
        );

        assert!(!output.contains("supersecretapikey1234567890"));
        assert!(!output.contains("abc123"));
        assert!(output.contains("/v2/redacted"));
        assert!(output.contains("api_key=redacted"));
    }

    #[test]
    fn test_sanitize_success_output_preserves_public_urls() {
        let output = sanitize_success_output("RPC https://ethereum.publicnode.com is healthy");

        assert!(output.contains("https://ethereum.publicnode.com"));
    }

    #[test]
    fn test_sanitize_success_output_redacts_secret_lines() {
        let output = sanitize_success_output(
            "Etherscan API key: configured\naccess_key = \"secret\"\nAuthorization: Bearer abc",
        );

        assert!(output.contains("Etherscan API key: <redacted>"));
        assert!(output.contains("access_key = <redacted>"));
        assert!(output.contains("Authorization: <redacted>"));
        assert!(!output.contains("secret"));
        assert!(!output.contains("Bearer abc"));
    }

    // -------------------------------------------------------------------------
    // ArgsBuilder tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_args_builder_basic() {
        let builder = ArgsBuilder::new("test").arg("foo").arg("bar");
        assert_eq!(builder.args, vec!["test", "foo", "bar"]);
    }

    #[test]
    fn test_args_builder_subcommand() {
        let builder = ArgsBuilder::new("test").subcommand("sub").arg("foo");
        assert_eq!(builder.args, vec!["test", "sub", "foo"]);
    }

    #[test]
    fn test_args_builder_opt_some() {
        let builder = ArgsBuilder::new("test").opt("--flag", Some("value"));
        assert_eq!(builder.args, vec!["test", "--flag", "value"]);
    }

    #[test]
    fn test_args_builder_opt_none() {
        let builder = ArgsBuilder::new("test").opt("--flag", None);
        assert_eq!(builder.args, vec!["test"]);
    }

    #[test]
    fn test_args_builder_opt_flag() {
        let builder = ArgsBuilder::new("test").opt_flag("--verbose", true);
        assert_eq!(builder.args, vec!["test", "--verbose"]);

        let builder = ArgsBuilder::new("test").opt_flag("--verbose", false);
        assert_eq!(builder.args, vec!["test"]);
    }

    #[test]
    fn test_args_builder_chain() {
        let builder = ArgsBuilder::new("test").chain(Some("polygon"));
        assert_eq!(builder.args, vec!["test", "--chain", "polygon"]);
    }

    #[test]
    fn test_args_builder_format_json() {
        let builder = ArgsBuilder::new("test").format_json();
        assert_eq!(builder.args, vec!["test", "-o", "json"]);
    }

    // -------------------------------------------------------------------------
    // get_timeout tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_timeout_fast_commands() {
        assert_eq!(get_timeout(&["cast"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["config"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["config", "validate"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["address", "list"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["blacklist", "check", "0xabc"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["endpoints", "list"]), FAST_TIMEOUT);
    }

    #[test]
    fn test_get_timeout_slow_commands() {
        assert_eq!(get_timeout(&["rpc"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["tx"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["account"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["endpoints", "health"]), DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_get_timeout_matches_commands_exactly() {
        // `enso` used to match the `ens` prefix and get the 10s timeout
        assert_eq!(get_timeout(&["enso", "route"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["ens", "lookup", "0xabc"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["castle"]), DEFAULT_TIMEOUT);
        assert_eq!(
            get_timeout(&["blacklist", "scan-portfolio", "0xabc"]),
            DEFAULT_TIMEOUT
        );
        assert_eq!(
            get_timeout(&["blacklist", "scan", "0xabc"]),
            DEFAULT_TIMEOUT
        );
        assert_eq!(get_timeout(&["endpoints", "test", "x"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["endpoints", "optimize"]), DEFAULT_TIMEOUT);
    }

    #[test]
    fn test_get_timeout_dune_executions_outlast_ethcli() {
        let sql = [
            "dune",
            "sql",
            "execute",
            "--timeout",
            "300",
            "--",
            "SELECT 1",
        ];
        assert_eq!(get_timeout(&sql), DUNE_EXECUTE_TIMEOUT);
        assert_eq!(
            get_timeout(&["dune", "query", "execute", "--timeout", "300", "1"]),
            DUNE_EXECUTE_TIMEOUT
        );
        assert!(DUNE_EXECUTE_TIMEOUT > Duration::from_secs(300));
        assert_eq!(
            get_timeout(&["dune", "queries", "get", "1"]),
            DEFAULT_TIMEOUT
        );
    }

    // -------------------------------------------------------------------------
    // MCP command policy tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_file_output_policy_blocks_mcp_file_writes() {
        assert!(is_file_output_command(&[
            "contract",
            "abi",
            "0x0000000000000000000000000000000000000000",
            "-o",
            "/tmp/abi.json"
        ]));
        assert!(is_file_output_command(&[
            "address",
            "export",
            "--output=/tmp/address.json"
        ]));
        assert!(!is_file_output_command(&[
            "rpc", "receipt", "0x123", "-o", "json"
        ]));
    }

    #[test]
    fn test_mutating_command_policy_blocks_write_tools() {
        assert!(is_mutating_command(&[
            "config",
            "set-etherscan-key",
            "secret"
        ]));
        assert!(is_mutating_command(&[
            "address",
            "import",
            "addresses.json"
        ]));
        assert!(is_mutating_command(&["blacklist", "remove", "0xabc"]));
        assert!(is_mutating_command(&[
            "endpoints",
            "add",
            "https://rpc.example"
        ]));
        assert!(is_mutating_command(&["chainlist", "add", "base"]));
        assert!(is_mutating_command(&["sig", "cache-clear"]));
        assert!(is_mutating_command(&[
            "cow-swap",
            "cancel-order",
            "uid",
            "sig"
        ]));
        assert!(is_mutating_command(&["dune", "queries", "update", "1"]));
        assert!(is_mutating_command(&[
            "tenderly",
            "vnets",
            "admin",
            "--vnet",
            "abc",
            "set-balance",
            "0xabc",
            "1"
        ]));
    }

    #[test]
    fn test_mutating_command_policy_blocks_remote_side_effects() {
        assert!(is_mutating_command(&[
            "cow-swap",
            "cancel-orders",
            "--uid",
            "a,b",
            "sig",
            "-o",
            "json"
        ]));
        assert!(is_mutating_command(&[
            "alchemy",
            "nft",
            "-n",
            "eth-mainnet",
            "report-spam",
            "0xabc"
        ]));
        assert!(is_mutating_command(&[
            "alchemy",
            "nft",
            "refresh-metadata",
            "0xabc",
            "1"
        ]));
        for leaf in ["sync-collection", "resync-metadata", "resync-traits"] {
            assert!(
                is_mutating_command(&["moralis", "nft", "--chain", "eth", leaf, "0xabc"]),
                "{leaf}"
            );
        }
        assert!(is_mutating_command(&[
            "simulate", "call", "0xabc", "--sig", "f()", "--save"
        ]));
        assert!(is_mutating_command(&[
            "blacklist",
            "scan",
            "0xabc",
            "--auto-blacklist"
        ]));

        assert!(!is_mutating_command(&[
            "alchemy",
            "nft",
            "-n",
            "eth-mainnet",
            "is-spam",
            "0xabc"
        ]));
        assert!(!is_mutating_command(&[
            "moralis", "nft", "--chain", "eth", "metadata", "0xabc", "1"
        ]));
        assert!(!is_mutating_command(&[
            "simulate", "call", "0xabc", "--sig", "f()"
        ]));
        // A `--save` positional after `--` is data, not the flag
        assert!(!is_mutating_command(&[
            "simulate",
            "call",
            "0xabc",
            "--sig",
            "f(string)",
            "--",
            "--save"
        ]));
        assert!(!is_mutating_command(&["blacklist", "scan", "0xabc"]));
    }

    #[test]
    fn test_tenderly_admin_walker_skips_equals_flags() {
        assert!(!is_mutating_command(&[
            "tenderly",
            "vnets",
            "admin",
            "--vnet=-abc",
            "get-latest"
        ]));
        assert!(is_mutating_command(&[
            "tenderly",
            "vnets",
            "admin",
            "--vnet=-abc",
            "set-balance",
            "0xabc",
            "1"
        ]));
    }

    #[test]
    fn test_redacts_alchemy_notify_token_and_access_key_lines() {
        assert_eq!(
            redact_secret_line("notify_token = \"abc123\"").as_deref(),
            Some("notify_token = <redacted>")
        );
        assert_eq!(
            redact_secret_line("access_key = \"abc123\"").as_deref(),
            Some("access_key = <redacted>")
        );
        assert_eq!(
            redact_secret_line("X-Alchemy-Token: abc123").as_deref(),
            Some("X-Alchemy-Token: <redacted>")
        );
        assert!(contains_secret_key_name("access-key"));
        assert!(contains_secret_key_name("notify_token"));
    }

    #[test]
    fn test_mutating_command_policy_allows_read_tools() {
        assert!(is_mutating_command(&["config", "set-alchemy-notify-token"]));
        assert!(is_mutating_command(&["config", "set-alchemy-access-key"]));
        assert!(!is_mutating_command(&["config", "validate"]));
        assert!(!is_mutating_command(&["address", "list"]));
        assert!(!is_mutating_command(&["blacklist", "check", "0xabc"]));
        assert!(!is_mutating_command(&["endpoints", "health"]));
        assert!(!is_mutating_command(&["chainlist", "rpcs", "base"]));
        assert!(!is_mutating_command(&["dune", "queries", "get", "1"]));
        assert!(!is_mutating_command(&["tenderly", "vnets", "get", "abc"]));
        assert!(!is_mutating_command(&[
            "tenderly",
            "vnets",
            "admin",
            "--vnet",
            "abc",
            "get-latest"
        ]));
    }

    #[test]
    fn test_policy_blocks_secret_bearing_args() {
        for args in [
            &["simulate", "call", "0xabc", "--show-secrets"][..],
            &["simulate", "call", "0xabc", "--tenderly-key", "secret"][..],
            &["simulate", "call", "0xabc", "--alchemy-key=secret"][..],
            &[
                "simulate",
                "call",
                "0xabc",
                "--rpc-header",
                "x-api-key: secret",
            ][..],
            &[
                "contract",
                "source",
                "0xabc",
                "--etherscan-api-key",
                "secret",
            ][..],
            // clap aliases and the global key flag
            &["simulate", "call", "0xabc", "--rpc-headers", "x-api-key: s"][..],
            &["simulate", "call", "0xabc", "--fork-headers=x-api-key: s"][..],
            &["contract", "abi", "0xabc", "--etherscan-key", "secret"][..],
            &["logs", "-c", "0xabc", "--etherscan-key=secret"][..],
        ] {
            assert!(matches!(
                validate_command_policy(args, false),
                Err(ExecutionError::PolicyDenied(_))
            ));
        }
    }

    #[test]
    fn test_policy_blocks_stdin_flag_without_payload() {
        // `tx --stdin` would read the MCP client's JSON-RPC stream
        let args = ["tx", "--stdin", "-o", "json"];
        assert!(matches!(
            validate_command_policy(&args, false),
            Err(ExecutionError::PolicyDenied(_))
        ));
        assert!(!matches!(
            validate_command_policy(&["config", "set-dune", "--stdin"], true),
            Err(ExecutionError::PolicyDenied(message)) if message.starts_with("--stdin")
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_child_stdin_is_closed_by_default() {
        // If the child inherited stdin, `cat` would block on the test runner's
        // stdin (the MCP transport in production) until the timeout.
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("cat; echo done");
        configure_stdio(&mut cmd, false);

        let output = run_command_with_timeout(cmd, Duration::from_secs(5), None)
            .await
            .expect("child should see EOF on stdin");
        assert_eq!(String::from_utf8_lossy(&output.stdout), "done\n");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_stdin_payload_is_written_and_closed() {
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("cat");
        configure_stdio(&mut cmd, true);

        let output = run_command_with_timeout(cmd, Duration::from_secs(5), Some("secret-key"))
            .await
            .expect("cat should exit after stdin closes");
        assert_eq!(String::from_utf8_lossy(&output.stdout), "secret-key\n");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_large_stdout_is_capped_and_drained() {
        // 3 MB: well past the cap and the pipe buffer, so the child would
        // block (and time out) if the excess were not drained.
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg("head -c 3000000 /dev/zero | tr '\\0' a");
        configure_stdio(&mut cmd, false);

        let output = run_command_with_timeout(cmd, Duration::from_secs(10), None)
            .await
            .expect("large output should complete, not time out");
        assert_eq!(output.stdout_total, 3_000_000);
        assert_eq!(output.stdout.len(), MAX_RESPONSE_BYTES + 1);

        let text = decode_stdout(output.stdout, output.stdout_total).unwrap();
        assert!(text.len() <= MAX_RESPONSE_BYTES);
        assert!(
            text.contains("of 3000000 bytes"),
            "note should give the real size"
        );
    }

    #[test]
    fn test_decode_stdout_drops_char_split_by_cap() {
        let mut stdout = "a".repeat(MAX_RESPONSE_BYTES).into_bytes();
        stdout.push(0xC3); // first byte of a 2-byte char cut off by the cap
        let text = decode_stdout(stdout, MAX_RESPONSE_BYTES + 10).unwrap();
        assert!(text.contains("output truncated"));

        assert!(decode_stdout(b"ok".to_vec(), 2).is_ok());
        assert!(matches!(
            decode_stdout(vec![0xff], 1),
            Err(ExecutionError::InvalidUtf8(_))
        ));
    }

    // -------------------------------------------------------------------------
    // ArgsBuilder value validation
    // -------------------------------------------------------------------------

    #[test]
    fn test_args_builder_rejects_flag_like_positionals() {
        for value in ["--stdin", "--all", "--rpc-url=http://169.254.169.254", "-5"] {
            let builder = ArgsBuilder::new("tx").arg(value);
            assert!(
                matches!(builder.error, Some(ValidationError::OptionLikeValue(_))),
                "{value} should be rejected"
            );
        }
        let builder = ArgsBuilder::new("kong").subcommand("--help");
        assert!(builder.error.is_some());
    }

    #[test]
    fn test_args_builder_opt_dash_value_uses_equals() {
        let builder = ArgsBuilder::new("logs")
            .opt("-e", Some("-x"))
            .opt("--from", Some("--all"));
        assert!(builder.error.is_none());
        assert_eq!(builder.args, vec!["logs", "-e=-x", "--from=--all"]);
    }

    #[test]
    fn test_args_builder_end_of_options() {
        let builder = ArgsBuilder::new("cast")
            .subcommand("abi-encode")
            .arg("f(int256)")
            .end_of_options()
            .arg("-5")
            .arg("--all");
        assert!(builder.error.is_none());
        assert_eq!(
            builder.args,
            vec!["cast", "abi-encode", "f(int256)", "--", "-5", "--all"]
        );

        let builder = ArgsBuilder::new("cast").end_of_options().format_json();
        assert!(matches!(
            builder.error,
            Some(ValidationError::FlagAfterEndOfOptions(_))
        ));
    }

    #[tokio::test]
    async fn test_args_builder_execute_reports_rejected_value() {
        let err = ArgsBuilder::new("tenderly")
            .subcommand("vnets")
            .subcommand("delete")
            .arg("--all")
            .execute()
            .await
            .unwrap_err();
        assert!(err.contains("must not start with '-'"), "{err}");
    }

    #[tokio::test]
    async fn test_execute_rejects_oversized_stdin() {
        let payload = "x".repeat(MAX_ARG_LENGTH + 1);
        let err = execute_validated(&["config", "set-dune", "--stdin"], Some(&payload))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            ExecutionError::Validation(ValidationError::StdinTooLong { .. })
        ));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_timeout_kills_process_group_descendants() {
        let marker = format!(
            "/tmp/ethcli-mcp-timeout-marker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let _ = std::fs::remove_file(&marker);

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(format!("(sleep 1; touch {marker}) & wait"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let result = run_command_with_timeout(cmd, Duration::from_millis(50), None).await;
        assert!(matches!(result, Err(ExecutionError::Timeout)));

        tokio::time::sleep(Duration::from_millis(1_200)).await;
        assert!(
            !std::path::Path::new(&marker).exists(),
            "timeout should kill child processes in the spawned process group"
        );

        let _ = std::fs::remove_file(&marker);
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_timeout_kills_descendant_after_parent_exits() {
        let marker = format!(
            "/tmp/ethcli-mcp-timeout-detached-marker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let _ = std::fs::remove_file(&marker);

        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(format!("(sleep 1; touch {marker}) &"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let result = run_command_with_timeout(cmd, Duration::from_millis(50), None).await;
        assert!(matches!(result, Err(ExecutionError::Timeout)));

        tokio::time::sleep(Duration::from_millis(1_200)).await;
        assert!(
            !std::path::Path::new(&marker).exists(),
            "timeout should kill descendants even if the parent process already exited"
        );

        let _ = std::fs::remove_file(&marker);
    }

    // -------------------------------------------------------------------------
    // Metrics tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_metrics_snapshot() {
        let metrics = Metrics::new();
        metrics.commands_total.fetch_add(5, Ordering::Relaxed);
        metrics.commands_success.fetch_add(3, Ordering::Relaxed);
        metrics.commands_failed.fetch_add(2, Ordering::Relaxed);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.commands_total, 5);
        assert_eq!(snapshot.commands_success, 3);
        assert_eq!(snapshot.commands_failed, 2);
    }
}
