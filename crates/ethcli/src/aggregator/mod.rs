//! Aggregation layer for combining data from multiple API sources
//!
//! This module provides:
//! - Core aggregation types (`SourceResult`, `AggregatedResult`)
//! - Data normalization types (`NormalizedPrice`, `NormalizedBalance`, `NormalizedNft`)
//! - Price aggregation from multiple sources
//! - Chain name normalization across services

pub mod chain_map;
pub mod nft;
pub mod normalize;
pub mod portfolio;
pub mod price;
pub mod swap;
pub mod yields;

pub use chain_map::*;
pub use nft::*;
pub use normalize::*;
pub use portfolio::*;
pub use price::*;
pub use swap::*;
pub use yields::*;

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Result from a single API source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceResult<T> {
    /// Source identifier (e.g., "gecko", "llama", "alchemy")
    pub source: String,
    /// Normalized data (None if error occurred)
    pub data: Option<T>,
    /// Raw response for debugging
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<serde_json::Value>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Response time in milliseconds
    pub latency_ms: u64,
    /// Unix timestamp of response
    pub timestamp: u64,
}

impl<T> SourceResult<T> {
    /// Create a successful result
    pub fn success(source: impl Into<String>, data: T, latency_ms: u64) -> Self {
        Self {
            source: source.into(),
            data: Some(data),
            raw: None,
            error: None,
            latency_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a successful result with raw data
    pub fn success_with_raw(
        source: impl Into<String>,
        data: T,
        raw: serde_json::Value,
        latency_ms: u64,
    ) -> Self {
        Self {
            source: source.into(),
            data: Some(data),
            raw: Some(raw),
            error: None,
            latency_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Create a failed result
    pub fn error(source: impl Into<String>, error: impl Into<String>, latency_ms: u64) -> Self {
        Self {
            source: source.into(),
            data: None,
            raw: None,
            error: Some(error.into()),
            latency_ms,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Check if this result is successful
    pub fn is_success(&self) -> bool {
        self.data.is_some() && self.error.is_none()
    }
}

/// Aggregated result from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResult<T, A> {
    /// Combined/calculated aggregated value
    pub aggregated: A,
    /// Per-source breakdown
    pub sources: Vec<SourceResult<T>>,
    /// Number of sources queried
    pub sources_queried: usize,
    /// Number of sources that succeeded
    pub sources_succeeded: usize,
    /// Total wall clock time (parallel execution)
    pub total_latency_ms: u64,
}

impl<T, A> AggregatedResult<T, A> {
    /// Create a new aggregated result
    pub fn new(aggregated: A, sources: Vec<SourceResult<T>>, total_latency_ms: u64) -> Self {
        let sources_queried = sources.len();
        let sources_succeeded = sources.iter().filter(|s| s.is_success()).count();
        Self {
            aggregated,
            sources,
            sources_queried,
            sources_succeeded,
            total_latency_ms,
        }
    }

    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.sources_queried == 0 {
            0.0
        } else {
            (self.sources_succeeded as f64 / self.sources_queried as f64) * 100.0
        }
    }

    /// Check if all sources succeeded
    pub fn all_succeeded(&self) -> bool {
        self.sources_queried == self.sources_succeeded
    }

    /// Check if at least one source succeeded
    pub fn any_succeeded(&self) -> bool {
        self.sources_succeeded > 0
    }
}

/// Helper to measure async operation latency
pub struct LatencyMeasure {
    start: Instant,
}

impl LatencyMeasure {
    /// Start measuring
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.start.elapsed().as_millis() as u64
    }
}

/// Price source enum for CLI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceSource {
    All,
    Gecko,
    Llama,
    Alchemy,
    Moralis,
    Curve,
    /// CCXT exchange data (Binance, Bitget, OKX, Hyperliquid)
    Ccxt,
    /// Chainlink Data Streams
    Chainlink,
    /// Pyth Network Hermes API
    Pyth,
}

impl std::str::FromStr for PriceSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(PriceSource::All),
            "gecko" | "coingecko" => Ok(PriceSource::Gecko),
            "llama" | "defillama" => Ok(PriceSource::Llama),
            "alchemy" | "alcmy" => Ok(PriceSource::Alchemy),
            "moralis" | "mrls" => Ok(PriceSource::Moralis),
            "curve" | "crv" => Ok(PriceSource::Curve),
            "ccxt" | "cex" | "exchange" => Ok(PriceSource::Ccxt),
            "chainlink" | "cl" => Ok(PriceSource::Chainlink),
            "pyth" => Ok(PriceSource::Pyth),
            _ => Err(format!("Unknown price source: {}", s)),
        }
    }
}

impl std::fmt::Display for PriceSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PriceSource::All => write!(f, "all"),
            PriceSource::Gecko => write!(f, "gecko"),
            PriceSource::Llama => write!(f, "llama"),
            PriceSource::Alchemy => write!(f, "alchemy"),
            PriceSource::Moralis => write!(f, "moralis"),
            PriceSource::Curve => write!(f, "curve"),
            PriceSource::Ccxt => write!(f, "ccxt"),
            PriceSource::Chainlink => write!(f, "chainlink"),
            PriceSource::Pyth => write!(f, "pyth"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_result_success() {
        let result: SourceResult<i32> = SourceResult::success("test", 42, 100);
        assert!(result.is_success());
        assert_eq!(result.data, Some(42));
        assert_eq!(result.source, "test");
        assert_eq!(result.latency_ms, 100);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_source_result_error() {
        let result: SourceResult<i32> = SourceResult::error("test", "failed", 50);
        assert!(!result.is_success());
        assert!(result.data.is_none());
        assert_eq!(result.error, Some("failed".to_string()));
    }

    #[test]
    fn test_source_result_with_raw() {
        let raw = serde_json::json!({"value": 42});
        let result = SourceResult::success_with_raw("test", 42, raw.clone(), 100);
        assert!(result.is_success());
        assert_eq!(result.raw, Some(raw));
    }

    #[test]
    fn test_aggregated_result_new() {
        let sources = vec![
            SourceResult::success("a", 1, 10),
            SourceResult::success("b", 2, 20),
            SourceResult::error("c", "failed", 30),
        ];
        let result = AggregatedResult::new(100, sources, 50);

        assert_eq!(result.aggregated, 100);
        assert_eq!(result.sources_queried, 3);
        assert_eq!(result.sources_succeeded, 2);
        assert_eq!(result.total_latency_ms, 50);
    }

    #[test]
    fn test_aggregated_result_success_rate() {
        let sources = vec![
            SourceResult::success("a", 1, 10),
            SourceResult::error("b", "failed", 20),
        ];
        let result = AggregatedResult::new(0, sources, 30);

        assert!((result.success_rate() - 50.0).abs() < 0.01);
        assert!(!result.all_succeeded());
        assert!(result.any_succeeded());
    }

    #[test]
    fn test_aggregated_result_all_succeeded() {
        let sources = vec![
            SourceResult::success("a", 1, 10),
            SourceResult::success("b", 2, 20),
        ];
        let result = AggregatedResult::new(0, sources, 30);

        assert!(result.all_succeeded());
        assert!((result.success_rate() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_result_none_succeeded() {
        let sources: Vec<SourceResult<i32>> = vec![
            SourceResult::error("a", "err1", 10),
            SourceResult::error("b", "err2", 20),
        ];
        let result = AggregatedResult::new(0, sources, 30);

        assert!(!result.any_succeeded());
        assert!((result.success_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregated_result_empty() {
        let sources: Vec<SourceResult<i32>> = vec![];
        let result = AggregatedResult::new(0, sources, 0);

        assert_eq!(result.sources_queried, 0);
        assert_eq!(result.sources_succeeded, 0);
        assert!((result.success_rate() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_latency_measure() {
        let measure = LatencyMeasure::start();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = measure.elapsed_ms();
        assert!(elapsed >= 10);
    }

    #[test]
    fn test_price_source_from_str() {
        assert_eq!("gecko".parse::<PriceSource>().unwrap(), PriceSource::Gecko);
        assert_eq!(
            "coingecko".parse::<PriceSource>().unwrap(),
            PriceSource::Gecko
        );
        assert_eq!("llama".parse::<PriceSource>().unwrap(), PriceSource::Llama);
        assert_eq!(
            "defillama".parse::<PriceSource>().unwrap(),
            PriceSource::Llama
        );
        assert_eq!(
            "alchemy".parse::<PriceSource>().unwrap(),
            PriceSource::Alchemy
        );
        assert_eq!("ccxt".parse::<PriceSource>().unwrap(), PriceSource::Ccxt);
        assert_eq!(
            "chainlink".parse::<PriceSource>().unwrap(),
            PriceSource::Chainlink
        );
        assert_eq!("pyth".parse::<PriceSource>().unwrap(), PriceSource::Pyth);
        assert!("invalid".parse::<PriceSource>().is_err());
    }

    #[test]
    fn test_price_source_display() {
        assert_eq!(PriceSource::Gecko.to_string(), "gecko");
        assert_eq!(PriceSource::Llama.to_string(), "llama");
        assert_eq!(PriceSource::All.to_string(), "all");
    }
}
