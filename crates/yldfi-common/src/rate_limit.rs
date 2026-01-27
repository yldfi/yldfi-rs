//! Client-side rate limiting utilities (SEC-008 fix)
//!
//! Provides configurable rate limiting to prevent accidental API abuse.
//! This protects both the client (from quota exhaustion) and the server
//! (from excessive requests).
//!
//! # Concurrency Note (LOW-001)
//!
//! Under high concurrency at window boundaries, there is a small race window
//! where slightly more requests than `max_requests` may be allowed. This is
//! because the window reset and request counting are not fully atomic to avoid
//! lock contention. For CLI tools and typical API usage, this is acceptable.
//! If strict enforcement is required, consider using a mutex-based approach.
//!
//! # Example
//!
//! ```no_run
//! use yldfi_common::rate_limit::RateLimiter;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Allow 10 requests per second
//!     let limiter = RateLimiter::new(10, Duration::from_secs(1));
//!
//!     for _ in 0..20 {
//!         limiter.acquire().await;
//!         // Make API request here
//!         println!("Request sent");
//!     }
//! }
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio::time::sleep;

/// A token bucket rate limiter for client-side request throttling.
///
/// Uses a semaphore-based approach with automatic token replenishment.
/// Thread-safe and suitable for use across multiple async tasks.
#[derive(Clone)]
pub struct RateLimiter {
    inner: Arc<RateLimiterInner>,
}

struct RateLimiterInner {
    /// Semaphore controlling concurrent request slots
    semaphore: Semaphore,
    /// Maximum requests per window
    max_requests: u32,
    /// Time window for rate limiting
    window: Duration,
    /// Fixed start time for computing elapsed time (CRIT-001 fix)
    start_time: Instant,
    /// Timestamp of last reset (as nanos since `start_time`)
    last_reset: AtomicU64,
    /// Number of requests in current window
    request_count: AtomicU64,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    ///
    /// * `max_requests` - Maximum number of requests allowed per window
    /// * `window` - Time window duration (e.g., 1 second)
    ///
    /// # Example
    ///
    /// ```
    /// use yldfi_common::rate_limit::RateLimiter;
    /// use std::time::Duration;
    ///
    /// // 5 requests per second
    /// let limiter = RateLimiter::new(5, Duration::from_secs(1));
    ///
    /// // 100 requests per minute
    /// let limiter = RateLimiter::new(100, Duration::from_secs(60));
    /// ```
    #[must_use]
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            inner: Arc::new(RateLimiterInner {
                semaphore: Semaphore::new(max_requests as usize),
                max_requests,
                window,
                start_time: Instant::now(), // CRIT-001 fix: store fixed start time
                last_reset: AtomicU64::new(0),
                request_count: AtomicU64::new(0),
            }),
        }
    }

    /// Create a rate limiter that allows unlimited requests (no throttling).
    ///
    /// Useful for testing or when rate limiting should be disabled.
    #[must_use]
    pub fn unlimited() -> Self {
        Self::new(u32::MAX, Duration::from_secs(1))
    }

    /// Acquire a permit to make a request.
    ///
    /// This method will block (async) if the rate limit has been reached,
    /// waiting until a new window starts or a slot becomes available.
    pub async fn acquire(&self) {
        let inner = &self.inner;
        let window_nanos = inner.window.as_nanos() as u64;

        // CRIT-002 fix: Loop to re-check window after sleeping
        loop {
            // CRIT-001 fix: Compute elapsed from stored start_time, not from now
            let now_nanos = inner.start_time.elapsed().as_nanos() as u64;

            // Check if we need to reset the window
            let last = inner.last_reset.load(Ordering::Relaxed);

            if now_nanos.saturating_sub(last) >= window_nanos {
                // Try to reset the window
                if inner
                    .last_reset
                    .compare_exchange(last, now_nanos, Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok()
                {
                    inner.request_count.store(0, Ordering::Relaxed);
                    // Release all permits back to max
                    let current = inner.semaphore.available_permits();
                    let to_add = inner.max_requests as usize - current;
                    if to_add > 0 {
                        inner.semaphore.add_permits(to_add);
                    }
                }
            }

            // Try to acquire a slot
            let count = inner.request_count.fetch_add(1, Ordering::Relaxed);
            if count < u64::from(inner.max_requests) {
                // We got a slot, try to acquire semaphore permit
                if let Ok(permit) = inner.semaphore.try_acquire() {
                    // Forget the permit so it gets returned when the window resets
                    permit.forget();
                    return;
                }
            }

            // Over the limit or no semaphore permit - undo the count increment and wait
            inner.request_count.fetch_sub(1, Ordering::Relaxed);

            // Calculate time remaining in current window
            let current_elapsed = inner.start_time.elapsed().as_nanos() as u64;
            let current_last = inner.last_reset.load(Ordering::Relaxed);
            let elapsed_in_window = current_elapsed.saturating_sub(current_last);
            let remaining = window_nanos.saturating_sub(elapsed_in_window);

            // Sleep for remaining window time (or a small backoff if window just reset)
            let sleep_time = if remaining > 0 {
                Duration::from_nanos(remaining)
            } else {
                Duration::from_millis(10) // Small backoff
            };
            sleep(sleep_time).await;
            // Loop back to re-check and try again
        }
    }

    /// Try to acquire a permit without blocking.
    ///
    /// Returns `true` if a permit was acquired, `false` if rate limited.
    ///
    /// # Note
    ///
    /// Uses a compare-and-swap loop to avoid TOCTOU race between checking
    /// the count and incrementing it.
    #[must_use]
    pub fn try_acquire(&self) -> bool {
        let inner = &self.inner;

        // Check and reset window before checking count
        let now_nanos = inner.start_time.elapsed().as_nanos() as u64;
        let window_nanos = inner.window.as_nanos() as u64;
        let last = inner.last_reset.load(Ordering::Relaxed);

        if now_nanos.saturating_sub(last) >= window_nanos {
            // Try to reset the window
            if inner
                .last_reset
                .compare_exchange(last, now_nanos, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                inner.request_count.store(0, Ordering::Relaxed);
                // Release all permits back to max
                let current = inner.semaphore.available_permits();
                let to_add = inner.max_requests as usize - current;
                if to_add > 0 {
                    inner.semaphore.add_permits(to_add);
                }
            }
        }

        // RL-002 fix: Use CAS loop to atomically check and increment count
        // This prevents TOCTOU race between load and fetch_add
        loop {
            let count = inner.request_count.load(Ordering::Relaxed);

            if count >= u64::from(inner.max_requests) {
                return false;
            }

            // Try to atomically increment only if count hasn't changed
            if inner
                .request_count
                .compare_exchange(count, count + 1, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                // Successfully incremented, try to acquire semaphore permit
                return inner.semaphore.try_acquire().map(tokio::sync::SemaphorePermit::forget).is_ok();
            }
            // CAS failed - another thread modified count, retry
        }
    }

    /// Get the current number of available permits.
    #[must_use]
    pub fn available(&self) -> usize {
        self.inner.semaphore.available_permits()
    }

    /// Get the maximum requests per window.
    #[must_use]
    pub fn max_requests(&self) -> u32 {
        self.inner.max_requests
    }

    /// Get the rate limit window duration.
    #[must_use]
    pub fn window(&self) -> Duration {
        self.inner.window
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_requests", &self.inner.max_requests)
            .field("window", &self.inner.window)
            .field("available", &self.available())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        assert_eq!(limiter.max_requests(), 10);
        assert_eq!(limiter.window(), Duration::from_secs(1));
    }

    #[test]
    fn test_unlimited_limiter() {
        let limiter = RateLimiter::unlimited();
        assert_eq!(limiter.max_requests(), u32::MAX);
    }

    #[test]
    fn test_try_acquire() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.try_acquire());
        assert!(limiter.try_acquire());
        // Third request should be rate limited
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_acquire_basic() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1));
        for _ in 0..5 {
            limiter.acquire().await;
        }
        // All 5 permits used
        assert_eq!(limiter.available(), 0);
    }
}
