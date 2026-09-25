//! HTTP client for the Solodit API
//!
//! Implements the documented Cyfrin Solodit Findings API: a single
//! `POST /findings` endpoint authenticated via the `X-Cyfrin-API-Key` header.

use reqwest::header::HeaderMap;
use reqwest::Client as HttpClient;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{json, Value};
use std::time::Duration;

use crate::error::{Error, Result};
use crate::types::{ApiResponse, Finding, Impact, RateLimit, SearchFilter, SearchResults};

/// Base URL for Solodit API
pub const BASE_URL: &str = "https://solodit.cyfrin.io/api/v1/solodit";

/// Default timeout for requests
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default user agent
const USER_AGENT: &str = "sldt/0.1 (Rust; +https://github.com/yldfi/yldfi-rs)";

/// Maximum characters of a raw (non-JSON) error body included in errors
const MAX_ERROR_BODY: usize = 500;

/// Solodit API client
///
/// This is an unofficial client for the Solodit vulnerability database.
/// Requires an API key from <https://solodit.cyfrin.io>.
///
/// The API key is stored using `SecretString` to prevent accidental exposure
/// in logs or debug output, and is redacted from any error message.
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
    fn build(api_key: String, base_url: String, timeout: Duration) -> Result<Self> {
        if api_key.trim().is_empty() {
            return Err(Error::client(
                "API key cannot be empty. Get your key from https://solodit.cyfrin.io (Profile > API Keys)",
            ));
        }

        let http = HttpClient::builder()
            .timeout(timeout)
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| Error::client(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            http,
            base_url,
            api_key: SecretString::new(api_key.trim().to_string().into()),
        })
    }

    /// Create a new client with an API key
    ///
    /// Get your API key from <https://solodit.cyfrin.io> (Profile > API Keys)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The API key is empty or whitespace-only
    /// - The HTTP client fails to initialize (rare, typically TLS issues)
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::build(api_key.into(), BASE_URL.to_string(), DEFAULT_TIMEOUT)
    }

    /// Create a client with custom timeout
    ///
    /// # Errors
    /// Returns an error if the API key is empty or HTTP client fails to initialize
    pub fn with_timeout(api_key: impl Into<String>, timeout: Duration) -> Result<Self> {
        Self::build(api_key.into(), BASE_URL.to_string(), timeout)
    }

    /// Create a client with custom base URL (for testing)
    ///
    /// # Errors
    /// Returns an error if the API key is empty or HTTP client fails to initialize
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Result<Self> {
        Self::build(api_key.into(), base_url.into(), DEFAULT_TIMEOUT)
    }

    /// Build URL for an endpoint
    fn build_url(&self, endpoint: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let endpoint = endpoint.trim_start_matches('/');
        format!("{base}/{endpoint}")
    }

    /// Build the `POST /findings` request body from a `SearchFilter`.
    ///
    /// Values are normalized to the documented constraints: `page >= 1`,
    /// `1 <= pageSize <= 100`, scores `0..=5`, `minFinders`/`maxFinders` as
    /// strings, and `reported.value = "after"` whenever `reportedAfter` is set.
    fn build_request_body(filter: &SearchFilter) -> Value {
        let mut filters = serde_json::Map::new();

        if let Some(keywords) = filter.keywords.as_deref().map(str::trim) {
            if !keywords.is_empty() {
                filters.insert("keywords".into(), json!(keywords));
            }
        }

        // Only the documented impact values; `Unknown` is never sent.
        let mut impacts: Vec<&str> = Vec::new();
        for impact in &filter.impacts {
            if *impact != Impact::Unknown && !impacts.contains(&impact.as_str()) {
                impacts.push(impact.as_str());
            }
        }
        if !impacts.is_empty() {
            filters.insert("impact".into(), json!(impacts));
        }

        if !filter.firms.is_empty() {
            filters.insert("firms".into(), json!(filter.firms));
        }
        if !filter.tags.is_empty() {
            filters.insert("tags".into(), json!(filter.tags));
        }
        if let Some(protocol) = &filter.protocol {
            filters.insert("protocol".into(), json!(protocol));
        }
        if !filter.protocol_categories.is_empty() {
            filters.insert("protocolCategory".into(), json!(filter.protocol_categories));
        }
        if !filter.forked.is_empty() {
            filters.insert("forked".into(), json!(filter.forked));
        }
        if !filter.languages.is_empty() {
            filters.insert("languages".into(), json!(filter.languages));
        }
        if let Some(user) = &filter.user {
            filters.insert("user".into(), json!(user));
        }

        // The spec types these as strings.
        if let Some(min) = filter.min_finders {
            filters.insert("minFinders".into(), json!(min.to_string()));
        }
        if let Some(max) = filter.max_finders {
            filters.insert("maxFinders".into(), json!(max.to_string()));
        }

        // `reportedAfter` is only honored when `reported.value == "after"`.
        let reported = match (&filter.reported, &filter.reported_after) {
            (_, Some(_)) => Some(crate::types::ReportedPeriod::After),
            (Some(p), None) => Some(*p),
            (None, None) => None,
        };
        if let Some(period) = reported {
            filters.insert("reported".into(), json!({ "value": period.as_str() }));
        }
        if let Some(date) = &filter.reported_after {
            filters.insert("reportedAfter".into(), json!(date));
        }

        if let Some(score) = filter.quality_score {
            filters.insert("qualityScore".into(), json!(score.min(5)));
        }
        if let Some(score) = filter.rarity_score {
            filters.insert("rarityScore".into(), json!(score.min(5)));
        }

        filters.insert("sortField".into(), json!(filter.sort_field.as_str()));
        filters.insert(
            "sortDirection".into(),
            json!(filter.sort_direction.as_str()),
        );

        json!({
            "page": filter.page.max(1),
            "pageSize": filter.page_size.clamp(1, 100),
            "filters": Value::Object(filters),
        })
    }

    /// Replace any occurrence of the API key in `text` (defense in depth).
    fn redact(&self, text: &str) -> String {
        let key = self.api_key.expose_secret();
        if key.is_empty() {
            text.to_string()
        } else {
            text.replace(key, "[REDACTED]")
        }
    }

    /// Extract the server's error message from a response body.
    ///
    /// Uses the `message` field of `{"message": "..."}` when present,
    /// otherwise a truncated copy of the raw body.
    fn error_message(&self, body: &str, fallback: &str) -> String {
        let msg = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|v| {
                v.get("message")
                    .or_else(|| v.get("error"))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_else(|| {
                let trimmed = body.trim();
                if trimmed.is_empty() {
                    fallback.to_string()
                } else {
                    trimmed.chars().take(MAX_ERROR_BODY).collect()
                }
            });
        self.redact(&msg)
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
    ///
    /// # Errors
    /// See [`Client::search_with_filter`].
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
    ///
    /// # Errors
    /// - [`Error::Api`] for 400 and other non-success statuses (with the server message)
    /// - [`Error::Unauthorized`] for 401 ("Missing API key" / "Invalid API key")
    /// - [`Error::RateLimited`] for 429 (with `X-RateLimit-*` header values)
    /// - [`Error::Http`] / [`Error::Json`] for transport or parse failures
    pub async fn search_with_filter(&self, filter: SearchFilter) -> Result<SearchResults> {
        let url = self.build_url("/findings");
        let body = Self::build_request_body(&filter);

        let response = self
            .http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Cyfrin-API-Key", self.api_key.expose_secret())
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let header_rate_limit = parse_rate_limit_headers(response.headers());
        let text = response.text().await?;

        if !status.is_success() {
            let code = status.as_u16();
            let reason = status.canonical_reason().unwrap_or("request failed");
            let message = self.error_message(&text, reason);
            return Err(match code {
                401 => Error::unauthorized(message),
                429 => {
                    let (limit, remaining, reset) = header_rate_limit
                        .map_or((None, None, None), |h| (h.limit, h.remaining, h.reset));
                    Error::rate_limited(message, limit, remaining, reset)
                }
                _ => Error::api(code, message),
            });
        }

        let value: Value = serde_json::from_str(&text)?;
        if !value.is_object() {
            return Err(Error::invalid_response(
                "expected a JSON object from POST /findings",
            ));
        }
        let body_had_rate_limit = value.get("rateLimit").is_some_and(Value::is_object);
        let api_response: ApiResponse = serde_json::from_value(value)?;

        Ok(SearchResults::from_response(api_response)
            .with_header_rate_limit(body_had_rate_limit, header_rate_limit.map(|h| h.complete())))
    }

    /// Look up a single finding by slug, numeric ID, or Solodit URL.
    ///
    /// The documented API has **no get-by-id endpoint** (only `POST /findings`),
    /// so this is a best-effort lookup: it runs keyword searches derived from
    /// the input (the raw value, then the slug with dashes replaced by spaces)
    /// and returns the first finding whose `slug` or `id` matches exactly.
    /// Each attempt costs one request against the rate limit (at most two).
    /// A finding whose title/content does not match its slug text may not be
    /// found this way.
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
    ///
    /// # Errors
    /// Returns [`Error::NotFound`] if no exact match is found, or any error
    /// from [`Client::search_with_filter`].
    pub async fn get_by_slug(&self, slug_or_id: &str) -> Result<Finding> {
        let needle = normalize_lookup(slug_or_id);
        if needle.is_empty() {
            return Err(Error::not_found(slug_or_id));
        }

        let mut queries = vec![needle.clone()];
        let spaced = needle.replace(['-', '_'], " ");
        if spaced != needle {
            queries.push(spaced);
        }

        for query in queries {
            let filter = SearchFilter::new(query).page_size(100);
            let results = self.search_with_filter(filter).await?;
            if let Some(found) = results.findings.into_iter().find(|f| {
                f.slug.as_deref() == Some(needle.as_str())
                    || f.id.as_deref() == Some(needle.as_str())
            }) {
                return Ok(found);
            }
        }

        Err(Error::not_found(needle))
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
    /// (`pageSize: 1`, which itself consumes one request).
    ///
    /// # Errors
    /// Any error from [`Client::search_with_filter`].
    pub async fn check_rate_limit(&self) -> Result<RateLimit> {
        let filter = SearchFilter::empty().page_size(1);
        let results = self.search_with_filter(filter).await?;
        Ok(results.rate_limit)
    }
}

/// Partially-present `X-RateLimit-*` headers.
#[derive(Debug, Clone, Copy)]
struct HeaderRateLimit {
    limit: Option<u32>,
    remaining: Option<u32>,
    reset: Option<u64>,
}

impl HeaderRateLimit {
    fn complete(self) -> RateLimit {
        RateLimit {
            limit: self.limit.unwrap_or_default(),
            remaining: self.remaining.unwrap_or_default(),
            reset: self.reset.unwrap_or_default(),
        }
    }
}

/// Parse `X-RateLimit-Limit`, `X-RateLimit-Remaining` and `X-RateLimit-Reset`.
/// Returns `None` if none of them are present.
fn parse_rate_limit_headers(headers: &HeaderMap) -> Option<HeaderRateLimit> {
    fn get<T: std::str::FromStr>(headers: &HeaderMap, name: &str) -> Option<T> {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.trim().parse().ok())
    }
    let parsed = HeaderRateLimit {
        limit: get(headers, "x-ratelimit-limit"),
        remaining: get(headers, "x-ratelimit-remaining"),
        reset: get(headers, "x-ratelimit-reset"),
    };
    (parsed.limit.is_some() || parsed.remaining.is_some() || parsed.reset.is_some())
        .then_some(parsed)
}

/// Normalize a lookup input: accept a full Solodit URL
/// (`https://solodit.cyfrin.io/issues/<slug>`) as well as a bare slug/ID.
fn normalize_lookup(input: &str) -> String {
    let s = input.trim();
    let s = s
        .split(['?', '#'])
        .next()
        .unwrap_or(s)
        .trim_end_matches('/');
    match s.rfind("/issues/") {
        Some(idx) => s[idx + "/issues/".len()..].to_string(),
        None => s.to_string(),
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
        let filter = SearchFilter::new("reentrancy");
        let body = Client::build_request_body(&filter);

        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 50);
        assert_eq!(body["filters"]["keywords"], "reentrancy");
    }

    #[test]
    fn test_build_request_body_with_impacts() {
        use crate::types::Impact;

        let filter = SearchFilter::new("test")
            .impact(Impact::High)
            .impact(Impact::Medium);
        let body = Client::build_request_body(&filter);

        let impacts = body["filters"]["impact"].as_array().unwrap();
        assert_eq!(impacts.len(), 2);
        assert!(impacts.contains(&json!("HIGH")));
        assert!(impacts.contains(&json!("MEDIUM")));
    }

    #[test]
    fn test_build_request_body_with_firms() {
        let filter = SearchFilter::new("test").firm("Cyfrin").firm("Sherlock");
        let body = Client::build_request_body(&filter);

        let firms = body["filters"]["firms"].as_array().unwrap();
        assert_eq!(firms.len(), 2);
        assert_eq!(firms[0], json!({"value": "Cyfrin"}));
    }

    #[test]
    fn test_build_request_body_normalizes_to_spec() {
        use crate::types::Impact;

        let filter = SearchFilter {
            page: 0,
            page_size: 500,
            impacts: vec![Impact::High, Impact::Unknown, Impact::High],
            min_finders: Some(1),
            max_finders: Some(3),
            reported_after: Some("2024-01-01".into()),
            quality_score: Some(9),
            ..SearchFilter::default()
        };
        let body = Client::build_request_body(&filter);
        assert_eq!(body["page"], 1);
        assert_eq!(body["pageSize"], 100);
        assert_eq!(body["filters"]["impact"], json!(["HIGH"]));
        assert_eq!(body["filters"]["minFinders"], json!("1"));
        assert_eq!(body["filters"]["maxFinders"], json!("3"));
        assert_eq!(body["filters"]["reported"], json!({"value": "after"}));
        assert_eq!(body["filters"]["reportedAfter"], json!("2024-01-01"));
        assert_eq!(body["filters"]["qualityScore"], json!(5));
        assert!(body["filters"].get("keywords").is_none());
        assert_eq!(body["filters"]["sortField"], json!("Recency"));
        assert_eq!(body["filters"]["sortDirection"], json!("Desc"));
    }

    #[test]
    fn test_error_message_parsing_and_redaction() {
        let client = Client::new("sk_secret").unwrap();
        assert_eq!(
            client.error_message(r#"{"message":"Invalid API key"}"#, "x"),
            "Invalid API key"
        );
        assert_eq!(client.error_message("", "Unauthorized"), "Unauthorized");
        assert_eq!(
            client.error_message("bad key sk_secret", "x"),
            "bad key [REDACTED]"
        );
    }

    #[test]
    fn test_normalize_lookup() {
        assert_eq!(normalize_lookup(" abc-def "), "abc-def");
        assert_eq!(
            normalize_lookup("https://solodit.cyfrin.io/issues/h-01-foo-bar?x=1"),
            "h-01-foo-bar"
        );
        assert_eq!(normalize_lookup("12345"), "12345");
    }

    #[test]
    fn test_parse_rate_limit_headers() {
        let mut h = HeaderMap::new();
        assert!(parse_rate_limit_headers(&h).is_none());
        h.insert("X-RateLimit-Limit", "20".parse().unwrap());
        h.insert("X-RateLimit-Remaining", "0".parse().unwrap());
        h.insert("X-RateLimit-Reset", "1700000000".parse().unwrap());
        let rl = parse_rate_limit_headers(&h).unwrap().complete();
        assert_eq!(
            rl,
            RateLimit {
                limit: 20,
                remaining: 0,
                reset: 1_700_000_000
            }
        );
    }
}
