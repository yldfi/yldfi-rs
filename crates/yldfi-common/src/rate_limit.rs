//! Client-side rate limiting utilities (SEC-008 fix)
//!
//! Provides configurable rate limiting to prevent accidental API abuse.
//! This protects both the client (from quota exhaustion) and the server
//! (from excessive requests).
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
    /// Timestamp of last reset (as nanos since some epoch)
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
        let now = Instant::now();
        let now_nanos = now.elapsed().as_nanos() as u64;

        // Check if we need to reset the window
        let last = inner.last_reset.load(Ordering::Relaxed);
        let window_nanos = inner.window.as_nanos() as u64;

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

        // Check if we're over the limit
        let count = inner.request_count.fetch_add(1, Ordering::Relaxed);
        if count >= inner.max_requests as u64 {
            // Wait for the window to reset
            let elapsed_in_window = now_nanos.saturating_sub(last);
            let remaining = window_nanos.saturating_sub(elapsed_in_window);
            if remaining > 0 {
                sleep(Duration::from_nanos(remaining)).await;
            }
        }

        // Acquire semaphore permit (will wait if none available)
        let permit = inner.semaphore.acquire().await;
        // Forget the permit so it gets returned when the window resets
        if let Ok(p) = permit {
            p.forget();
        }
    }

    /// Try to acquire a permit without blocking.
    ///
    /// Returns `true` if a permit was acquired, `false` if rate limited.
    #[must_use]
    pub fn try_acquire(&self) -> bool {
        let inner = &self.inner;
        let count = inner.request_count.load(Ordering::Relaxed);

        if count >= inner.max_requests as u64 {
            return false;
        }

        inner.request_count.fetch_add(1, Ordering::Relaxed);
        inner.semaphore.try_acquire().map(|p| p.forget()).is_ok()
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
