//! Command executor for ethcli
//!
//! Runs ethcli commands as subprocesses and captures output.
//! Includes rate limiting, timeouts, input validation, and metrics.

use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Maximum concurrent subprocess executions to prevent DoS
const MAX_CONCURRENT_SUBPROCESSES: usize = 10;

/// Default timeout for subprocess execution (30 seconds)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Fast timeout for simple operations like cast conversions (10 seconds)
const FAST_TIMEOUT: Duration = Duration::from_secs(10);

/// Commands that use fast timeout (pure computation, no network)
const FAST_COMMANDS: &[&str] = &[
    "cast", "ens", "config", "address", "blacklist", "endpoints",
];

/// Maximum argument length to prevent memory exhaustion
const MAX_ARG_LENGTH: usize = 10_000;

/// Maximum number of arguments
const MAX_ARGS: usize = 100;

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

/// Sanitize error messages to prevent information leakage
fn sanitize_error(stderr: &str, stdout: &str) -> String {
    let mut message = String::new();

    // Extract useful error info without exposing sensitive details
    for line in stderr.lines().chain(stdout.lines()) {
        let line = line.trim();

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

        // Limit message length
        if message.len() + line.len() > 500 {
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

/// Verify the ethcli binary is valid and executable
fn verify_binary(path: &std::path::Path) -> Result<(), ExecutionError> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = std::fs::metadata(path).map_err(|e| {
        ExecutionError::BinaryNotFound(format!("{}: {}", path.display(), e))
    })?;

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
fn find_ethcli_binary() -> Result<std::path::PathBuf, ExecutionError> {
    // First check environment variable override
    if let Ok(path) = std::env::var("ETHCLI_PATH") {
        let path = std::path::PathBuf::from(&path);
        verify_binary(&path)?;
        return Ok(path);
    }

    // Check same directory as this binary
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let ethcli = dir.join("ethcli");
            if ethcli.exists() {
                verify_binary(&ethcli)?;
                return Ok(ethcli);
            }
        }
    }

    // Fall back to PATH lookup (can't verify without which/where)
    Ok(std::path::PathBuf::from("ethcli"))
}

/// Get timeout duration based on command type
fn get_timeout(command: &str) -> Duration {
    if FAST_COMMANDS.iter().any(|c| command.starts_with(c)) {
        FAST_TIMEOUT
    } else {
        DEFAULT_TIMEOUT
    }
}

/// Execute an ethcli command and return the output
///
/// This function:
/// - Validates all arguments for safety
/// - Limits concurrent subprocess execution
/// - Applies a timeout to prevent hanging
/// - Sanitizes error messages
pub async fn execute(args: &[&str]) -> Result<String, String> {
    execute_validated(args).await.map_err(|e| e.to_string())
}

/// Execute with full error type information
pub async fn execute_validated(args: &[&str]) -> Result<String, ExecutionError> {
    let start = Instant::now();
    let command = args.first().copied().unwrap_or("unknown");

    // Track total commands
    METRICS.commands_total.fetch_add(1, Ordering::Relaxed);

    // Validate arguments
    validate_args(args).map_err(ExecutionError::Validation)?;

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
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let cmd_timeout = get_timeout(command);
    debug!(command = %command, timeout_secs = %cmd_timeout.as_secs(), "Executing ethcli command");

    // Execute with tiered timeout
    let output = match timeout(cmd_timeout, cmd.output()).await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => {
            METRICS.commands_failed.fetch_add(1, Ordering::Relaxed);
            return Err(ExecutionError::SpawnFailed(e.to_string()));
        }
        Err(_) => {
            METRICS.timeouts.fetch_add(1, Ordering::Relaxed);
            warn!(command = %command, "Command timed out");
            return Err(ExecutionError::Timeout);
        }
    };

    let duration_ms = start.elapsed().as_millis();

    if output.status.success() {
        METRICS.commands_success.fetch_add(1, Ordering::Relaxed);
        debug!(command = %command, duration_ms = %duration_ms, "Command succeeded");
        String::from_utf8(output.stdout).map_err(|e| ExecutionError::InvalidUtf8(e.to_string()))
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

/// Build command args from optional parameters
pub struct ArgsBuilder {
    args: Vec<String>,
}

impl ArgsBuilder {
    pub fn new(command: &str) -> Self {
        Self {
            args: vec![command.to_string()],
        }
    }

    pub fn subcommand(mut self, sub: &str) -> Self {
        self.args.push(sub.to_string());
        self
    }

    pub fn arg(mut self, value: &str) -> Self {
        self.args.push(value.to_string());
        self
    }

    pub fn opt(mut self, flag: &str, value: Option<&str>) -> Self {
        if let Some(v) = value {
            self.args.push(flag.to_string());
            self.args.push(v.to_string());
        }
        self
    }

    pub fn opt_flag(mut self, flag: &str, enabled: bool) -> Self {
        if enabled {
            self.args.push(flag.to_string());
        }
        self
    }

    pub fn chain(self, chain: Option<&str>) -> Self {
        self.opt("--chain", chain)
    }

    pub fn format_json(mut self) -> Self {
        self.args.push("--format".to_string());
        self.args.push("json".to_string());
        self
    }

    pub async fn execute(self) -> Result<String, String> {
        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        execute(&args).await
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
        assert!(validate_eth_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA960455").is_err()); // 43 chars
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
        assert_eq!(builder.args, vec!["test", "--format", "json"]);
    }

    // -------------------------------------------------------------------------
    // get_timeout tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_get_timeout_fast_commands() {
        assert_eq!(get_timeout("cast"), FAST_TIMEOUT);
        assert_eq!(get_timeout("ens"), FAST_TIMEOUT);
        assert_eq!(get_timeout("config"), FAST_TIMEOUT);
    }

    #[test]
    fn test_get_timeout_slow_commands() {
        assert_eq!(get_timeout("rpc"), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout("tx"), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout("account"), DEFAULT_TIMEOUT);
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
