//! Proxy rotation support

use crate::error::{ConfigError, Result};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Proxy rotator for distributing requests across multiple proxies
pub struct ProxyRotator {
    /// List of proxy URLs
    proxies: Vec<String>,
    /// Current index for round-robin
    current: AtomicUsize,
    /// Rotation mode
    mode: RotationMode,
}

/// Proxy rotation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationMode {
    /// Round-robin through proxies
    #[default]
    RoundRobin,
    /// Random selection
    Random,
    /// Sticky - use same proxy until failure
    Sticky,
}

impl ProxyRotator {
    /// Create a new proxy rotator from a list of URLs
    pub fn new(proxies: Vec<String>, mode: RotationMode) -> Self {
        Self {
            proxies,
            current: AtomicUsize::new(0),
            mode,
        }
    }

    /// Create from a proxy file (one URL per line)
    pub fn from_file(path: &Path, mode: RotationMode) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| ConfigError::InvalidFile(format!("{}: {}", path.display(), e)))?;

        let proxies: Vec<String> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect();

        if proxies.is_empty() {
            return Err(ConfigError::InvalidFile("Proxy file is empty".to_string()).into());
        }

        Ok(Self::new(proxies, mode))
    }

    /// Create from a single proxy URL
    pub fn single(url: String) -> Self {
        Self::new(vec![url], RotationMode::Sticky)
    }

    /// Get the next proxy URL
    pub fn next(&self) -> Option<&str> {
        if self.proxies.is_empty() {
            return None;
        }

        let index = match self.mode {
            RotationMode::RoundRobin => {
                self.current.fetch_add(1, Ordering::Relaxed) % self.proxies.len()
            }
            RotationMode::Random => {
                let mut rng = rand::thread_rng();
                rand::Rng::gen_range(&mut rng, 0..self.proxies.len())
            }
            RotationMode::Sticky => self.current.load(Ordering::Relaxed) % self.proxies.len(),
        };

        self.proxies.get(index).map(|s| s.as_str())
    }

    /// Get current proxy without advancing
    pub fn current(&self) -> Option<&str> {
        if self.proxies.is_empty() {
            return None;
        }

        let index = self.current.load(Ordering::Relaxed) % self.proxies.len();
        self.proxies.get(index).map(|s| s.as_str())
    }

    /// Mark current proxy as failed and move to next
    pub fn mark_failed(&self) {
        if self.mode == RotationMode::Sticky {
            self.current.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get number of proxies
    pub fn len(&self) -> usize {
        self.proxies.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.proxies.is_empty()
    }

    /// Get all proxy URLs
    pub fn all(&self) -> &[String] {
        &self.proxies
    }
}

/// Validate a proxy URL format
pub fn validate_proxy_url(url: &str) -> Result<()> {
    let url = url.trim();

    // Check for supported schemes
    if !url.starts_with("http://")
        && !url.starts_with("https://")
        && !url.starts_with("socks5://")
        && !url.starts_with("socks5h://")
    {
        return Err(ConfigError::InvalidFile(format!(
            "Invalid proxy URL scheme: {}. Supported: http, https, socks5, socks5h",
            url
        ))
        .into());
    }

    // Basic URL validation
    if url.parse::<reqwest::Url>().is_err() {
        return Err(ConfigError::InvalidFile(format!("Invalid proxy URL: {}", url)).into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_robin() {
        let rotator = ProxyRotator::new(
            vec![
                "http://proxy1.com".to_string(),
                "http://proxy2.com".to_string(),
                "http://proxy3.com".to_string(),
            ],
            RotationMode::RoundRobin,
        );

        assert_eq!(rotator.next(), Some("http://proxy1.com"));
        assert_eq!(rotator.next(), Some("http://proxy2.com"));
        assert_eq!(rotator.next(), Some("http://proxy3.com"));
        assert_eq!(rotator.next(), Some("http://proxy1.com")); // Wraps around
    }

    #[test]
    fn test_sticky() {
        let rotator = ProxyRotator::new(
            vec![
                "http://proxy1.com".to_string(),
                "http://proxy2.com".to_string(),
            ],
            RotationMode::Sticky,
        );

        assert_eq!(rotator.next(), Some("http://proxy1.com"));
        assert_eq!(rotator.next(), Some("http://proxy1.com"));

        rotator.mark_failed();
        assert_eq!(rotator.next(), Some("http://proxy2.com"));
        assert_eq!(rotator.next(), Some("http://proxy2.com"));
    }

    #[test]
    fn test_single_proxy() {
        let rotator = ProxyRotator::single("socks5://localhost:9050".to_string());

        assert_eq!(rotator.len(), 1);
        assert_eq!(rotator.next(), Some("socks5://localhost:9050"));
    }

    #[test]
    fn test_validate_proxy_url() {
        assert!(validate_proxy_url("http://proxy.example.com:8080").is_ok());
        assert!(validate_proxy_url("https://user:pass@proxy.example.com").is_ok());
        assert!(validate_proxy_url("socks5://localhost:9050").is_ok());
        assert!(validate_proxy_url("socks5h://localhost:9050").is_ok());

        assert!(validate_proxy_url("ftp://proxy.example.com").is_err());
        assert!(validate_proxy_url("not-a-url").is_err());
    }
}
