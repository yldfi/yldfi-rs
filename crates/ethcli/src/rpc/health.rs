//! Health tracking for RPC endpoints

use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Health metrics for a single endpoint
#[derive(Debug, Clone)]
pub struct EndpointHealth {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Rate limit hits
    pub rate_limit_hits: u64,
    /// Timeout count
    pub timeout_count: u64,
    /// Rolling average latency (ms)
    pub avg_latency_ms: f64,
    /// Last successful request time
    pub last_success: Option<Instant>,
    /// Last failure time
    pub last_failure: Option<Instant>,
    /// Current circuit breaker state
    pub circuit_open: bool,
    /// Circuit breaker open until
    pub circuit_open_until: Option<Instant>,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Learned max block range (0 = use configured)
    pub learned_max_block_range: Option<u64>,
    /// Learned max logs (0 = use configured)
    pub learned_max_logs: Option<usize>,
}

impl Default for EndpointHealth {
    fn default() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            rate_limit_hits: 0,
            timeout_count: 0,
            avg_latency_ms: 0.0,
            last_success: None,
            last_failure: None,
            circuit_open: false,
            circuit_open_until: None,
            consecutive_failures: 0,
            learned_max_block_range: None,
            learned_max_logs: None,
        }
    }
}

impl EndpointHealth {
    /// Calculate error rate (0.0 - 1.0)
    pub fn error_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.failed_requests as f64 / self.total_requests as f64
    }

    /// Check if circuit breaker should allow request
    pub fn is_available(&self) -> bool {
        if !self.circuit_open {
            return true;
        }

        // Check if circuit breaker timeout has passed
        if let Some(until) = self.circuit_open_until {
            if Instant::now() >= until {
                return true; // Allow one probe request
            }
        }

        false
    }

    /// Calculate a health score (higher = better)
    pub fn health_score(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0; // New endpoint, assume healthy
        }

        let success_rate = 1.0 - self.error_rate();
        let latency_factor = 1.0 / (1.0 + self.avg_latency_ms / 1000.0);

        // Penalize for rate limits
        let rate_limit_penalty = if self.total_requests > 0 {
            1.0 - (self.rate_limit_hits as f64 / self.total_requests as f64).min(0.5)
        } else {
            1.0
        };

        // Combined score
        success_rate * 50.0 + latency_factor * 30.0 + rate_limit_penalty * 20.0
    }
}

/// Tracks health across all endpoints
pub struct HealthTracker {
    /// Health data per endpoint URL
    health: RwLock<HashMap<String, EndpointHealth>>,
    /// Circuit breaker threshold (consecutive failures)
    circuit_breaker_threshold: u32,
    /// Circuit breaker timeout
    circuit_breaker_timeout: Duration,
}

impl HealthTracker {
    /// Create a new health tracker
    pub fn new() -> Self {
        Self {
            health: RwLock::new(HashMap::new()),
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(60),
        }
    }

    /// Create with custom circuit breaker settings
    pub fn with_circuit_breaker(threshold: u32, timeout_secs: u64) -> Self {
        Self {
            health: RwLock::new(HashMap::new()),
            circuit_breaker_threshold: threshold,
            circuit_breaker_timeout: Duration::from_secs(timeout_secs),
        }
    }

    /// Record a successful request
    pub fn record_success(&self, url: &str, latency: Duration) {
        let mut health = self.health.write();
        let entry = health.entry(url.to_string()).or_default();

        entry.total_requests += 1;
        entry.successful_requests += 1;
        entry.consecutive_failures = 0;
        entry.last_success = Some(Instant::now());

        // Update rolling average latency (exponential moving average)
        let latency_ms = latency.as_millis() as f64;
        if entry.avg_latency_ms == 0.0 {
            entry.avg_latency_ms = latency_ms;
        } else {
            entry.avg_latency_ms = entry.avg_latency_ms * 0.8 + latency_ms * 0.2;
        }

        // Reset circuit breaker on success
        if entry.circuit_open {
            entry.circuit_open = false;
            entry.circuit_open_until = None;
        }
    }

    /// Record a failed request
    pub fn record_failure(&self, url: &str, is_rate_limit: bool, is_timeout: bool) {
        let mut health = self.health.write();
        let entry = health.entry(url.to_string()).or_default();

        entry.total_requests += 1;
        entry.failed_requests += 1;
        entry.consecutive_failures += 1;
        entry.last_failure = Some(Instant::now());

        if is_rate_limit {
            entry.rate_limit_hits += 1;
        }

        if is_timeout {
            entry.timeout_count += 1;
        }

        // Open circuit breaker if threshold exceeded
        if entry.consecutive_failures >= self.circuit_breaker_threshold {
            entry.circuit_open = true;
            entry.circuit_open_until = Some(Instant::now() + self.circuit_breaker_timeout);
        }
    }

    /// Record learned block range limit
    pub fn record_block_range_limit(&self, url: &str, limit: u64) {
        let mut health = self.health.write();
        let entry = health.entry(url.to_string()).or_default();

        // Only update if lower than current learned limit
        match entry.learned_max_block_range {
            None => entry.learned_max_block_range = Some(limit),
            Some(current) if limit < current => entry.learned_max_block_range = Some(limit),
            _ => {}
        }
    }

    /// Record learned max logs limit
    pub fn record_logs_limit(&self, url: &str, limit: usize) {
        let mut health = self.health.write();
        let entry = health.entry(url.to_string()).or_default();

        // Only update if lower than current learned limit
        match entry.learned_max_logs {
            None => entry.learned_max_logs = Some(limit),
            Some(current) if limit < current => entry.learned_max_logs = Some(limit),
            _ => {}
        }
    }

    /// Check if endpoint is available (not circuit-broken)
    pub fn is_available(&self, url: &str) -> bool {
        let health = self.health.read();
        health.get(url).map(|h| h.is_available()).unwrap_or(true)
    }

    /// Atomically check if endpoint is available for a probe request.
    /// If the circuit breaker timeout has expired, this extends it to prevent
    /// multiple concurrent probe requests (fixes race condition).
    pub fn try_probe(&self, url: &str) -> bool {
        let mut health = self.health.write();
        let entry = health.entry(url.to_string()).or_default();

        if !entry.circuit_open {
            return true;
        }

        // Check if circuit breaker timeout has passed
        if let Some(until) = entry.circuit_open_until {
            if Instant::now() >= until {
                // Extend timeout to prevent concurrent probes
                entry.circuit_open_until = Some(Instant::now() + self.circuit_breaker_timeout);
                return true; // Allow this one probe request
            }
        }

        false
    }

    /// Get health for an endpoint
    pub fn get_health(&self, url: &str) -> Option<EndpointHealth> {
        self.health.read().get(url).cloned()
    }

    /// Get all health data
    pub fn get_all_health(&self) -> HashMap<String, EndpointHealth> {
        self.health.read().clone()
    }

    /// Get effective max block range (learned or configured)
    pub fn effective_max_block_range(&self, url: &str, configured: u64) -> u64 {
        let health = self.health.read();
        if let Some(h) = health.get(url) {
            if let Some(learned) = h.learned_max_block_range {
                return learned.min(configured);
            }
        }
        configured
    }

    /// Get effective max logs (learned or configured)
    pub fn effective_max_logs(&self, url: &str, configured: usize) -> usize {
        let health = self.health.read();
        if let Some(h) = health.get(url) {
            if let Some(learned) = h.learned_max_logs {
                return learned.min(configured);
            }
        }
        configured
    }

    /// Get endpoints sorted by health score
    pub fn rank_endpoints(&self, urls: &[String]) -> Vec<(String, f64)> {
        let health = self.health.read();
        let mut ranked: Vec<_> = urls
            .iter()
            .filter_map(|url| {
                let h = health.get(url).cloned().unwrap_or_default();
                if h.is_available() {
                    Some((url.clone(), h.health_score()))
                } else {
                    None
                }
            })
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_tracking() {
        let tracker = HealthTracker::new();

        // Record some successes
        tracker.record_success("https://test.com", Duration::from_millis(100));
        tracker.record_success("https://test.com", Duration::from_millis(150));

        let health = tracker.get_health("https://test.com").unwrap();
        assert_eq!(health.total_requests, 2);
        assert_eq!(health.successful_requests, 2);
        assert_eq!(health.failed_requests, 0);
        assert!(health.avg_latency_ms > 100.0 && health.avg_latency_ms < 150.0);
    }

    #[test]
    fn test_circuit_breaker() {
        let tracker = HealthTracker::with_circuit_breaker(3, 60);

        // Record failures to trigger circuit breaker
        tracker.record_failure("https://test.com", false, false);
        tracker.record_failure("https://test.com", false, false);
        assert!(tracker.is_available("https://test.com"));

        tracker.record_failure("https://test.com", false, false);
        assert!(!tracker.is_available("https://test.com"));
    }

    #[test]
    fn test_error_rate() {
        let mut health = EndpointHealth::default();
        health.total_requests = 10;
        health.failed_requests = 3;

        assert!((health.error_rate() - 0.3).abs() < 0.001);
    }
}
