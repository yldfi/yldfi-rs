//! HTTP client for the Solodit API

use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use std::time::Duration;

use crate::error::{Error, Result};
use crate::types::{ApiResponse, Finding, SearchFilter, SearchResults};

/// Base URL for Solodit API
pub const BASE_URL: &str = "https://solodit.cyfrin.io/api/v1/solodit";

/// Default timeout for requests
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default user agent
const USER_AGENT: &str = "sldt/0.1 (Rust; +https://github.com/yldfi/yldfi-rs)";

/// Solodit API client
///
/// This is an unofficial client for the Solodit vulnerability database.
/// Requires an API key from <https://solodit.cyfrin.io>
///
/// # Example
///
/// ```no_run
/// # async fn example() -> sldt::Result<()> {
/// let client = sldt::Client::new("sk_your_api_key_here")?;
/// let results = client.search("reentrancy").await?;
/// # Ok(())
/// # }
/// ```
/// Solodit API client
///
/// The API key is stored securely using `SecretString` to prevent
/// accidental exposure in logs or debug output.
#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base_url: String,
    api_key: SecretString,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("api_key", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Create a new client with an API key
    ///
    /// Get your API key from <https://solodit.cyfrin.io> (Profile > API Keys)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The API key is empty or whitespace-only
    /// - The HTTP client fails to initialize (rare, typically TLS issues)
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let api_key_str = api_key.into();

        // Validate API key is not empty
        if api_key_str.trim().is_empty() {
            return Err(Error::client(
                "API key cannot be empty. Get your key from https://solodit.cyfrin.io",
            ));
        }

        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| Error::client(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
            api_key: SecretString::new(api_key_str.into()),
        })
    }

    /// Create a client with custom timeout
    ///
    /// # Errors
    /// Returns an error if the API key is empty or HTTP client fails to initialize
    pub fn with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<Self> {
        let api_key_str = api_key.into();
        if api_key_str.trim().is_empty() {
            return Err(Error::client(
                "API key cannot be empty. Get your key from https://solodit.cyfrin.io",
            ));
        }

        let http = HttpClient::builder()
            .timeout(timeout)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| Error::client(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: BASE_URL.to_string(),
            api_key: SecretString::new(api_key_str.into()),
        })
    }

    /// Create a client with custom base URL (for testing)
    ///
    /// # Errors
    /// Returns an error if the API key is empty or HTTP client fails to initialize
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        let api_key_str = api_key.into();
        if api_key_str.trim().is_empty() {
            return Err(Error::client(
                "API key cannot be empty. Get your key from https://solodit.cyfrin.io",
            ));
        }

        let http = HttpClient::builder()
            .timeout(DEFAULT_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| Error::client(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url: base_url.into(),
            api_key: SecretString::new(api_key_str.into()),
        })
    }

    /// Build URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let endpoint = endpoint.trim_start_matches('/');
        format!("{base}/{endpoint}")
    }

    /// Build the request body from a `SearchFilter`
    fn build_request_body(&self, filter: &SearchFilter) -> serde_json::Value {
        let mut filters = json!({});

        // Keywords
        if let Some(ref keywords) = filter.keywords {
            filters["keywords"] = json!(keywords);
        }

        // Impact levels
        if !filter.impacts.is_empty() {
            let impacts: Vec<&str> = filter.impacts.iter().map(super::types::Impact::as_str).collect();
            filters["impact"] = json!(impacts);
        }

        // Firms
        if !filter.firms.is_empty() {
            filters["firms"] = json!(filter.firms);
        }

        // Tags
        if !filter.tags.is_empty() {
            filters["tags"] = json!(filter.tags);
        }

        // Protocol
        if let Some(ref protocol) = filter.protocol {
            filters["protocol"] = json!(protocol);
        }

        // Protocol categories
        if !filter.protocol_categories.is_empty() {
            filters["protocolCategory"] = json!(filter.protocol_categories);
        }

        // Forked protocols
        if !filter.forked.is_empty() {
            filters["forked"] = json!(filter.forked);
        }

        // Languages
        if !filter.languages.is_empty() {
            filters["languages"] = json!(filter.languages);
        }

        // User/finder
        if let Some(ref user) = filter.user {
            filters["user"] = json!(user);
        }

        // Finder count range
        if let Some(min) = filter.min_finders {
            filters["minFinders"] = json!(min.to_string());
        }
        if let Some(max) = filter.max_finders {
            filters["maxFinders"] = json!(max.to_string());
        }

        // Reported period
        if let Some(ref period) = filter.reported {
            filters["reported"] = json!({ "value": period.as_str() });
        }
        if let Some(ref date) = filter.reported_after {
            filters["reportedAfter"] = json!(date);
        }

        // Quality/Rarity scores
        if let Some(score) = filter.quality_score {
            filters["qualityScore"] = json!(score);
        }
        if let Some(score) = filter.rarity_score {
            filters["rarityScore"] = json!(score);
        }

        // Sort
        filters["sortField"] = json!(filter.sort_field.as_str());
        filters["sortDirection"] = json!(filter.sort_direction.as_str());

        json!({
            "page": filter.page,
            "pageSize": filter.page_size,
            "filters": filters
        })
    }

    /// Search for vulnerability findings
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> sldt::Result<()> {
    /// let client = sldt::Client::new("sk_your_api_key")?;
    /// let results = client.search("reentrancy").await?;
    ///
    /// for finding in results.findings {
    ///     println!("[{}] {}", finding.impact_level(), finding.title.unwrap_or_default());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, keywords: &str) -> Result<SearchResults> {
        self.search_with_filter(SearchFilter::new(keywords)).await
    }

    /// Search with custom filter options
    ///
    /// # Example
    ///
    /// ```no_run
    /// use sldt::{Client, SearchFilter, Impact};
    ///
    /// # async fn example() -> sldt::Result<()> {
    /// let client = Client::new("sk_your_api_key")?;
    ///
    /// let filter = SearchFilter::new("flash loan")
    ///     .impact(Impact::High)
    ///     .impact(Impact::Medium)
    ///     .page_size(50)
    ///     .sort_by_quality();
    ///
    /// let results = client.search_with_filter(filter).await?;
    /// println!("Found {} results", results.total);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_with_filter(&self, filter: SearchFilter) -> Result<SearchResults> {
        let url = self.build_url("/findings");
        let body = self.build_request_body(&filter);

        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Cyfrin-API-Key", self.api_key.expose_secret())
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();

            return match status {
                401 => Err(Error::unauthorized()),
                429 => Err(Error::rate_limited()),
                _ => Err(Error::api(status, body)),
            };
        }

        let api_response: ApiResponse = response.json().await?;
        Ok(SearchResults::from_response(api_response))
    }

    /// Get a specific finding by its slug
    ///
    /// Note: The official API doesn't have a dedicated endpoint for fetching by slug.
    /// This method searches for the exact slug and returns the first match.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> sldt::Result<()> {
    /// let client = sldt::Client::new("sk_your_api_key")?;
    /// let finding = client.get_by_slug("some-finding-slug").await?;
    /// println!("Title: {}", finding.title.unwrap_or_default());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_by_slug(&self, slug: &str) -> Result<Finding> {
        // Search with the slug as keyword and look for exact match
        let filter = SearchFilter::new(slug).page_size(100);
        let results = self.search_with_filter(filter).await?;

        results
            .findings
            .into_iter()
            .find(|f| f.slug.as_deref() == Some(slug))
            .ok_or_else(|| Error::not_found(slug))
    }

    /// Search for findings with pagination support
    ///
    /// Returns a paginator for iterating through all results
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> sldt::Result<()> {
    /// let client = sldt::Client::new("sk_your_api_key")?;
    /// let mut paginator = client.paginate(sldt::SearchFilter::new("oracle"));
    ///
    /// while let Some(findings) = paginator.next_page().await? {
    ///     for finding in findings {
    ///         println!("{}", finding.title.unwrap_or_default());
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[must_use] 
    pub fn paginate(&self, filter: SearchFilter) -> FindingPaginator {
        FindingPaginator {
            client: self.clone(),
            filter,
            current_page: 1,
            total_pages: None,
            done: false,
        }
    }

    /// Get current rate limit status by making a minimal request
    pub async fn check_rate_limit(&self) -> Result<crate::types::RateLimit> {
        let filter = SearchFilter::empty().page_size(1);
        let results = self.search_with_filter(filter).await?;
        Ok(results.rate_limit)
    }
}

/// Paginator for iterating through search results
pub struct FindingPaginator {
    client: Client,
    filter: SearchFilter,
    current_page: u32,
    total_pages: Option<u32>,
    done: bool,
}

impl FindingPaginator {
    /// Fetch the next page of results
    pub async fn next_page(&mut self) -> Result<Option<Vec<Finding>>> {
        if self.done {
            return Ok(None);
        }

        // Check if we've fetched all pages
        if let Some(total) = self.total_pages {
            if self.current_page > total {
                self.done = true;
                return Ok(None);
            }
        }

        let filter = self.filter.with_page(self.current_page);

        let results = self.client.search_with_filter(filter).await?;

        // Update total pages from response
        self.total_pages = Some(results.total_pages);

        if results.findings.is_empty() {
            self.done = true;
            return Ok(None);
        }

        self.current_page += 1;

        // Mark done if this was the last page
        if !results.has_more() {
            self.done = true;
        }

        Ok(Some(results.findings))
    }

    /// Get the total number of results (available after first page fetch)
    #[must_use] 
    pub fn total_pages(&self) -> Option<u32> {
        self.total_pages
    }

    /// Check if pagination is complete
    #[must_use] 
    pub fn is_done(&self) -> bool {
        self.done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_api_key_rejected() {
        assert!(Client::new("").is_err());
        assert!(Client::new("   ").is_err());
        assert!(Client::new("\t\n").is_err());
    }

    #[test]
    fn test_valid_api_key_accepted() {
        assert!(Client::new("test_key").is_ok());
        assert!(Client::new("sk_123456").is_ok());
    }

    #[test]
    fn test_build_url() {
        let client = Client::new("test_key").unwrap();
        let url = client.build_url("/findings");
        assert_eq!(url, "https://solodit.cyfrin.io/api/v1/solodit/findings");
    }

    #[test]
    fn test_build_url_no_leading_slash() {
        let client = Client::new("test_key").unwrap();
        let url = client.build_url("findings");
        assert_eq!(url, "https://solodit.cyfrin.io/api/v1/solodit/findings");
    }

    #[test]
    fn test_build_url_with_trailing_slash() {
        let client = Client::with_base_url("test_key", "https://example.com/api/").unwrap();
        let url = client.build_url("test");
        assert_eq!(url, "https://example.com/api/test");
    }

    #[test]
    fn test_build_request_body_simple() {
        let client = Client::new("test_key").unwrap();
        let filter = SearchFilter::new("reentrancy");
        let body = client.build_request_body(&filter);

        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 50);
        assert_eq!(body["filters"]["keywords"], "reentrancy");
    }

    #[test]
    fn test_build_request_body_with_impacts() {
        use crate::types::Impact;

        let client = Client::new("test_key").unwrap();
        let filter = SearchFilter::new("test")
            .impact(Impact::High)
            .impact(Impact::Medium);
        let body = client.build_request_body(&filter);

        let impacts = body["filters"]["impact"].as_array().unwrap();
        assert_eq!(impacts.len(), 2);
        assert!(impacts.contains(&json!("HIGH")));
        assert!(impacts.contains(&json!("MEDIUM")));
    }

    #[test]
    fn test_build_request_body_with_firms() {
        let client = Client::new("test_key").unwrap();
        let filter = SearchFilter::new("test")
            .firm("Cyfrin")
            .firm("Sherlock");
        let body = client.build_request_body(&filter);

        let firms = body["filters"]["firms"].as_array().unwrap();
        assert_eq!(firms.len(), 2);
    }
}
