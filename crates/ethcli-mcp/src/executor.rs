//! Command executor for ethcli
//!
//! Runs ethcli commands as subprocesses and captures output.
//! Includes rate limiting, timeouts, input validation, and metrics.

use std::process::{Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt};
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

/// Commands that use fast timeout (pure computation, no network)
const FAST_COMMANDS: &[&str] = &["cast", "ens", "config", "address", "blacklist", "endpoints"];

/// Opt-in env var for MCP tools that mutate local/remote state
const WRITE_TOOLS_ENV: &str = "ETHCLI_MCP_ENABLE_WRITE_TOOLS";

/// Flags that can carry credentials in subprocess arguments
const SECRET_BEARING_FLAGS: &[&str] = &[
    "--show-secrets",
    "--tenderly-key",
    "--alchemy-key",
    "--etherscan-api-key",
    "--rpc-header",
    "--fork-header",
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

fn is_mutating_dune_command(args: &[&str]) -> bool {
    matches!(
        (args.get(1).copied(), args.get(2).copied()),
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
                if arg.starts_with("--") {
                    index += 2;
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

fn is_mutating_command(args: &[&str]) -> bool {
    match (args.first().copied(), args.get(1).copied()) {
        (Some("config"), Some(sub)) => matches!(
            sub,
            "init"
                | "set-etherscan-key"
                | "set-tenderly"
                | "set-alchemy"
                | "set-moralis"
                | "set-chainlink"
                | "set-dune"
                | "set-dune-sim"
                | "set-solodit"
                | "add-debug-rpc"
                | "remove-debug-rpc"
        ),
        (Some("address"), Some("add" | "remove" | "import")) => true,
        (Some("blacklist"), Some("add" | "remove")) => true,
        (Some("endpoints"), Some("add" | "remove" | "enable" | "disable" | "optimize")) => true,
        (Some("chainlist"), Some("add")) => true,
        (Some("sig"), Some("cache-clear")) => true,
        (Some("simulate"), Some("share" | "unshare")) => true,
        (Some("dune"), _) => is_mutating_dune_command(args),
        (Some("tenderly"), _) => is_mutating_tenderly_command(args),
        (Some("cow-swap"), Some("create-order" | "cancel-order")) => true,
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

fn validate_command_policy(args: &[&str]) -> Result<(), ExecutionError> {
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

fn contains_secret_key_name(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("api key")
        || lower.contains("access_key")
        || lower.contains("access key")
        || lower.contains("x-access-key")
        || lower.contains("user_secret")
        || lower.contains("client_secret")
        || lower.contains("authorization")
        || lower.contains("auth_token")
        || lower.contains("bearer")
        || lower.contains("token")
}

fn should_redact_path_segment(host: &str, previous: Option<&str>, segment: &str) -> bool {
    if segment.is_empty() {
        return false;
    }

    let lower = segment.to_ascii_lowercase();
    if contains_secret_key_name(&lower) {
        return true;
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
                let value = if contains_secret_key_name(&key) {
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
    use std::os::unix::fs::PermissionsExt;

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
            let ethcli = dir.join("ethcli");
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
    if matches!(args, ["endpoints", "health", ..]) {
        return DEFAULT_TIMEOUT;
    }

    let command = args.first().copied().unwrap_or("unknown");
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

fn read_pipe<R>(mut pipe: R) -> JoinHandle<std::io::Result<Vec<u8>>>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut output = Vec::new();
        pipe.read_to_end(&mut output).await?;
        Ok(output)
    })
}

async fn collect_output(
    mut task: JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
    deadline: TokioInstant,
) -> Result<Vec<u8>, ExecutionError> {
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

fn abort_output_task(task: Option<JoinHandle<std::io::Result<Vec<u8>>>>) {
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

async fn run_command_with_timeout(
    mut cmd: Command,
    cmd_timeout: Duration,
) -> Result<Output, ExecutionError> {
    #[cfg(unix)]
    cmd.process_group(0);

    cmd.kill_on_drop(true);
    let deadline = TokioInstant::now() + cmd_timeout;

    let mut child = cmd
        .spawn()
        .map_err(|e| ExecutionError::SpawnFailed(e.to_string()))?;
    let child_pid = child.id();
    let stdout_task = child.stdout.take().map(read_pipe);
    let stderr_task = child.stderr.take().map(read_pipe);

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
        Vec::new()
    };

    let stderr = if let Some(task) = stderr_task {
        match collect_output(task, "stderr", deadline).await {
            Ok(output) => output,
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

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

/// Execute with full error type information
pub async fn execute_validated(args: &[&str]) -> Result<String, ExecutionError> {
    let start = Instant::now();
    let command = args.first().copied().unwrap_or("unknown");

    // Track total commands
    METRICS.commands_total.fetch_add(1, Ordering::Relaxed);

    // Validate arguments
    validate_args(args).map_err(ExecutionError::Validation)?;
    validate_command_policy(args)?;

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

    let cmd_timeout = get_timeout(args);
    debug!(command = %command, timeout_secs = %cmd_timeout.as_secs(), "Executing ethcli command");

    // Execute with tiered timeout
    let output = match run_command_with_timeout(cmd, cmd_timeout).await {
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
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| ExecutionError::InvalidUtf8(e.to_string()))?;
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

    /// Set -n/--network flag (for Alchemy commands where --network is defined
    /// on intermediate subcommands, not leaf subcommands). Must be called
    /// BEFORE the leaf .subcommand() call.
    pub fn network(self, network: Option<&str>) -> Self {
        self.opt("-n", network)
    }

    pub fn format_json(mut self) -> Self {
        // Use -o (short flag) because ethcli commands inconsistently use
        // --format or --output for the long flag, but all use -o for short
        self.args.push("-o".to_string());
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
        assert_eq!(get_timeout(&["ens"]), FAST_TIMEOUT);
        assert_eq!(get_timeout(&["config"]), FAST_TIMEOUT);
    }

    #[test]
    fn test_get_timeout_slow_commands() {
        assert_eq!(get_timeout(&["rpc"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["tx"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["account"]), DEFAULT_TIMEOUT);
        assert_eq!(get_timeout(&["endpoints", "health"]), DEFAULT_TIMEOUT);
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
    fn test_mutating_command_policy_allows_read_tools() {
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
        ] {
            assert!(matches!(
                validate_command_policy(args),
                Err(ExecutionError::PolicyDenied(_))
            ));
        }
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

        let result = run_command_with_timeout(cmd, Duration::from_millis(50)).await;
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

        let result = run_command_with_timeout(cmd, Duration::from_millis(50)).await;
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
