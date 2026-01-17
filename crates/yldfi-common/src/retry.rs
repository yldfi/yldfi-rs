//! Retry utilities with exponential backoff
//!
//! Provides configurable retry logic for API clients with:
//! - Exponential backoff with configurable multiplier
//! - Maximum delay cap
//! - Jitter to prevent thundering herd
//! - Support for server-specified retry-after durations

use std::future::Future;
use std::time::Duration;

/// Configuration for retry behavior
#[derive(Clone, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries, just one attempt)
    pub max_retries: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g., 2.0 doubles delay each retry)
    pub backoff_multiplier: f64,
    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with specified max retries
    #[must_use]
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// No retries - just one attempt
    #[must_use]
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            ..Default::default()
        }
    }

    /// Quick retries for interactive use
    #[must_use]
    pub fn quick() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Aggressive retries for batch operations
    #[must_use]
    pub fn batch() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Set maximum retries
    #[must_use]
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set initial delay
    #[must_use]
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set maximum delay
    #[must_use]
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set backoff multiplier
    #[must_use]
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Enable or disable jitter
    #[must_use]
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate delay for a given attempt number (0-indexed)
    fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let base_delay =
            self.initial_delay.as_millis() as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped_delay = base_delay.min(self.max_delay.as_millis() as f64);

        let final_delay = if self.jitter {
            // Add up to 25% jitter
            let jitter_factor = 1.0 + (random_f64() * 0.25);
            capped_delay * jitter_factor
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay as u64)
    }
}

/// Random f64 between 0.0 and 1.0 using fastrand PRNG
fn random_f64() -> f64 {
    fastrand::f64()
}

/// Determines if an error should be retried
pub trait RetryableError {
    /// Returns true if the error is transient and the operation should be retried
    fn is_retryable(&self) -> bool;

    /// Returns the retry-after duration if specified by the error
    fn retry_after(&self) -> Option<Duration> {
        None
    }
}

/// Error wrapper that includes retry information
#[derive(Debug)]
pub struct RetryError<E> {
    /// The last error that occurred
    pub error: E,
    /// Number of attempts made
    pub attempts: u32,
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (after {} attempt{})",
            self.error,
            self.attempts,
            if self.attempts == 1 { "" } else { "s" }
        )
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl<E> RetryError<E> {
    /// Get the inner error
    pub fn into_inner(self) -> E {
        self.error
    }
}

/// Execute an async operation with retries
///
/// Uses exponential backoff with optional jitter. If the error provides a
/// retry-after duration, that will be used instead of the calculated delay.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async closure that returns Result<T, E>
///
/// # Returns
/// * `Ok(T)` - If the operation succeeds
/// * `Err(RetryError<E>)` - If all retries are exhausted or error is not retryable
///
/// # Example
///
/// ```no_run
/// use yldfi_common::{with_retry, RetryConfig, RetryableError};
///
/// #[derive(Debug)]
/// struct ApiError { retryable: bool }
///
/// impl RetryableError for ApiError {
///     fn is_retryable(&self) -> bool { self.retryable }
/// }
///
/// async fn call_api() -> Result<String, ApiError> {
///     Ok("success".to_string())
/// }
///
/// async fn example() {
///     let config = RetryConfig::default();
///     let result = with_retry(&config, call_api).await;
/// }
/// ```
pub async fn with_retry<T, E, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, RetryError<E>>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: RetryableError,
{
    let mut attempts = 0;
    let max_attempts = config.max_retries + 1;

    loop {
        attempts += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempts >= max_attempts || !e.is_retryable() {
                    return Err(RetryError { error: e, attempts });
                }

                // Use retry-after from error if available, otherwise calculate
                let delay = e
                    .retry_after()
                    .unwrap_or_else(|| config.delay_for_attempt(attempts - 1));
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Simple retry wrapper for operations that return Result with any error type
///
/// This version always retries on any error up to max_retries times.
/// Use this when you don't need fine-grained control over which errors are retryable.
///
/// # Example
///
/// ```no_run
/// use yldfi_common::with_simple_retry;
///
/// async fn flaky_operation() -> Result<String, std::io::Error> {
///     Ok("success".to_string())
/// }
///
/// async fn example() {
///     let result = with_simple_retry(3, flaky_operation).await;
/// }
/// ```
pub async fn with_simple_retry<T, E, F, Fut>(max_retries: u32, mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    let config = RetryConfig::new(max_retries);
    let mut attempts = 0;
    let max_attempts = config.max_retries + 1;

    loop {
        attempts += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempts >= max_attempts {
                    return Err(e);
                }

                let delay = config.delay_for_attempt(attempts - 1);
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        assert_eq!(config.delay_for_attempt(3), Duration::from_millis(800));
    }

    #[test]
    fn test_delay_cap() {
        let config = RetryConfig {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            jitter: false,
        };

        // After a few iterations, should cap at max_delay
        assert_eq!(config.delay_for_attempt(5), Duration::from_secs(5));
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_presets() {
        let quick = RetryConfig::quick();
        assert_eq!(quick.max_retries, 2);
        assert_eq!(quick.initial_delay, Duration::from_millis(50));

        let batch = RetryConfig::batch();
        assert_eq!(batch.max_retries, 5);
        assert_eq!(batch.initial_delay, Duration::from_millis(200));

        let none = RetryConfig::none();
        assert_eq!(none.max_retries, 0);
    }
}
