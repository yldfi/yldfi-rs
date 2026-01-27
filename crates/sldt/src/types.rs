//! Type definitions for Solodit API responses

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a field that can be either a string or an empty object (returns None for empty object)
fn deserialize_string_or_empty_object<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::String(s) if !s.is_empty() => Ok(Some(s)),
        Value::String(_) => Ok(None), // empty string
        Value::Null => Ok(None),
        Value::Object(map) if map.is_empty() => Ok(None), // empty object {}
        Value::Object(_) => Err(D::Error::custom(
            "expected string or empty object, got non-empty object",
        )),
        _ => Err(D::Error::custom("expected string or empty object")),
    }
}

/// Impact/severity level of a finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Impact {
    High,
    Medium,
    Low,
    Gas,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for Impact {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Impact::High => write!(f, "HIGH"),
            Impact::Medium => write!(f, "MEDIUM"),
            Impact::Low => write!(f, "LOW"),
            Impact::Gas => write!(f, "GAS"),
            Impact::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl Impact {
    /// Convert to API string format
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Impact::High => "HIGH",
            Impact::Medium => "MEDIUM",
            Impact::Low => "LOW",
            Impact::Gas => "GAS",
            Impact::Unknown => "UNKNOWN",
        }
    }
}

/// Sort direction for search results
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Desc,
    Asc,
}

impl SortDirection {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SortDirection::Desc => "Desc",
            SortDirection::Asc => "Asc",
        }
    }
}

/// Sort field for search results
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    Recency,
    Quality,
    Rarity,
}

impl SortField {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            SortField::Recency => "Recency",
            SortField::Quality => "Quality",
            SortField::Rarity => "Rarity",
        }
    }
}

/// Time period for filtering by report date
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum ReportedPeriod {
    /// Last 30 days
    Days30,
    /// Last 60 days
    Days60,
    /// Last 90 days
    Days90,
    /// After a specific date (use with reportedAfter)
    After,
    /// All time (no date filter)
    #[default]
    AllTime,
}

impl ReportedPeriod {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            ReportedPeriod::Days30 => "30",
            ReportedPeriod::Days60 => "60",
            ReportedPeriod::Days90 => "90",
            ReportedPeriod::After => "after",
            ReportedPeriod::AllTime => "alltime",
        }
    }
}

/// A labeled filter value (used for firms, tags, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterValue {
    /// The filter value
    pub value: String,
    /// Optional display label
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl FilterValue {
    /// Create a new filter value
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
        }
    }

    /// Create a filter value with a label
    pub fn with_label(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: Some(label.into()),
        }
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Audit firm information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFirm {
    /// Firm name
    #[serde(default)]
    pub name: Option<String>,
    /// URL to firm's square logo
    #[serde(default)]
    pub logo_square: Option<String>,
}

/// Protocol category score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCategoryScore {
    /// The category information
    #[serde(default)]
    pub protocols_protocolcategory: Option<ProtocolCategory>,
    /// Score for this category
    #[serde(default)]
    pub score: Option<f64>,
}

/// Protocol category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCategory {
    /// Category title
    #[serde(default)]
    pub title: Option<String>,
}

/// Protocol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    /// Protocol name
    #[serde(default)]
    pub name: Option<String>,
    /// Category scores for the protocol
    #[serde(default)]
    pub protocols_protocolcategoryscore: Vec<ProtocolCategoryScore>,
}

/// Warden/auditor who found the issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warden {
    /// Auditor handle/username
    pub handle: String,
}

/// Issue finder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFinder {
    /// The warden who found the issue
    #[serde(default)]
    pub wardens_warden: Option<Warden>,
}

/// Tag score associated with a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTagScore {
    /// Tag information
    #[serde(default)]
    pub tags_tag: Option<IssueTag>,
}

/// Tag associated with a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTag {
    /// Tag title
    #[serde(default)]
    pub title: Option<String>,
}

/// A vulnerability report/finding from Solodit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier
    #[serde(default)]
    pub id: Option<String>,

    /// URL-friendly slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Finding title
    #[serde(default)]
    pub title: Option<String>,

    /// Full content/description (markdown)
    #[serde(default)]
    pub content: Option<String>,

    /// Summary of the finding
    #[serde(default)]
    pub summary: Option<String>,

    /// Content kind (e.g., "MARKDOWN")
    #[serde(default)]
    pub kind: Option<String>,

    /// Impact/severity level
    #[serde(default)]
    pub impact: Option<String>,

    /// Quality score (0-5)
    #[serde(default)]
    pub quality_score: Option<f64>,

    /// General/rarity score (0-5)
    #[serde(default)]
    pub general_score: Option<f64>,

    /// Report date
    #[serde(default, deserialize_with = "deserialize_string_or_empty_object")]
    pub report_date: Option<String>,

    /// Audit firm ID
    #[serde(default)]
    pub auditfirm_id: Option<String>,

    /// Firm name (flattened)
    #[serde(default)]
    pub firm_name: Option<String>,

    /// Firm logo URL (flattened)
    #[serde(default)]
    pub firm_logo_square: Option<String>,

    /// Audit firm that conducted the review
    #[serde(default)]
    pub auditfirms_auditfirm: Option<AuditFirm>,

    /// Protocol ID
    #[serde(default)]
    pub protocol_id: Option<String>,

    /// Protocol name (flattened)
    #[serde(default)]
    pub protocol_name: Option<String>,

    /// Protocol that was audited
    #[serde(default)]
    pub protocols_protocol: Option<Protocol>,

    /// Contest ID (for competitive audits)
    #[serde(default)]
    pub contest_id: Option<String>,

    /// Contest link
    #[serde(default)]
    pub contest_link: Option<String>,

    /// Contest prize text
    #[serde(default)]
    pub contest_prize_txt: Option<String>,

    /// Sponsor name
    #[serde(default)]
    pub sponsor_name: Option<String>,

    /// Sponsor link
    #[serde(default)]
    pub sponsor_link: Option<String>,

    /// Number of finders
    #[serde(default)]
    pub finders_count: Option<i32>,

    /// People who found this issue
    #[serde(default)]
    pub issues_issue_finders: Vec<IssueFinder>,

    /// Tags associated with the finding
    #[serde(default)]
    pub issues_issuetagscore: Vec<IssueTagScore>,

    /// Source link (original report)
    #[serde(default)]
    pub source_link: Option<String>,

    /// GitHub link
    #[serde(default)]
    pub github_link: Option<String>,

    /// PDF link
    #[serde(default)]
    pub pdf_link: Option<String>,

    /// PDF page start
    #[serde(default)]
    pub pdf_page_from: Option<i32>,

    /// Whether bookmarked (always false for API)
    #[serde(default)]
    pub bookmarked: bool,

    /// Whether read (always false for API)
    #[serde(default)]
    pub read: bool,
}

impl Finding {
    /// Get the impact level as an enum
    #[must_use]
    pub fn impact_level(&self) -> Impact {
        match self.impact.as_deref() {
            Some("HIGH" | "high" | "High") => Impact::High,
            Some("MEDIUM" | "medium" | "Medium") => Impact::Medium,
            Some("LOW" | "low" | "Low") => Impact::Low,
            Some("GAS" | "gas" | "Gas") => Impact::Gas,
            _ => Impact::Unknown,
        }
    }

    /// Get the audit firm name
    #[must_use]
    pub fn firm(&self) -> Option<&str> {
        self.firm_name.as_deref().or_else(|| {
            self.auditfirms_auditfirm
                .as_ref()
                .and_then(|f| f.name.as_deref())
        })
    }

    /// Get the protocol name
    #[must_use]
    pub fn protocol(&self) -> Option<&str> {
        self.protocol_name.as_deref().or_else(|| {
            self.protocols_protocol
                .as_ref()
                .and_then(|p| p.name.as_deref())
        })
    }

    /// Get finder handles
    #[must_use]
    pub fn finder_handles(&self) -> Vec<&str> {
        self.issues_issue_finders
            .iter()
            .filter_map(|f| f.wardens_warden.as_ref().map(|w| w.handle.as_str()))
            .collect()
    }

    /// Get tags
    #[must_use]
    pub fn tags(&self) -> Vec<&str> {
        self.issues_issuetagscore
            .iter()
            .filter_map(|t| t.tags_tag.as_ref().and_then(|tag| tag.title.as_deref()))
            .collect()
    }

    /// Get the Solodit URL for this finding
    #[must_use]
    pub fn solodit_url(&self) -> Option<String> {
        self.slug
            .as_ref()
            .map(|s| format!("https://solodit.cyfrin.io/issues/{s}"))
    }
}

/// Search filter options for the API
#[derive(Debug, Clone, Default)]
pub struct SearchFilter {
    /// Keywords to search for in title and content
    pub keywords: Option<String>,

    /// Filter by impact levels
    pub impacts: Vec<Impact>,

    /// Filter by audit firms
    pub firms: Vec<FilterValue>,

    /// Filter by tags
    pub tags: Vec<FilterValue>,

    /// Filter by protocol name (partial match)
    pub protocol: Option<String>,

    /// Filter by protocol categories
    pub protocol_categories: Vec<FilterValue>,

    /// Filter by forked protocols
    pub forked: Vec<FilterValue>,

    /// Filter by programming languages
    pub languages: Vec<FilterValue>,

    /// Filter by finder/auditor handle (partial match)
    pub user: Option<String>,

    /// Minimum number of finders
    pub min_finders: Option<u32>,

    /// Maximum number of finders
    pub max_finders: Option<u32>,

    /// Filter by report date period
    pub reported: Option<ReportedPeriod>,

    /// Filter by reports after this date (ISO format, when reported = After)
    pub reported_after: Option<String>,

    /// Minimum quality score (0-5)
    pub quality_score: Option<u32>,

    /// Minimum rarity score (0-5)
    pub rarity_score: Option<u32>,

    /// Page number (1-indexed, default 1)
    pub page: u32,

    /// Results per page (default 50, max 100)
    pub page_size: u32,

    /// Sort field
    pub sort_field: SortField,

    /// Sort direction
    pub sort_direction: SortDirection,
}

impl SearchFilter {
    /// Create a new search filter with keywords
    pub fn new(keywords: impl Into<String>) -> Self {
        Self {
            keywords: Some(keywords.into()),
            page: 1,
            page_size: 50,
            quality_score: Some(1),
            rarity_score: Some(1),
            ..Default::default()
        }
    }

    /// Create an empty filter (returns all findings)
    #[must_use]
    pub fn empty() -> Self {
        Self {
            page: 1,
            page_size: 50,
            quality_score: Some(1),
            rarity_score: Some(1),
            ..Default::default()
        }
    }

    /// Set keywords to search for
    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    /// Set page number (1-indexed)
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = page.max(1);
        self
    }

    /// Set page size (max 100)
    #[must_use]
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = size.clamp(1, 100);
        self
    }

    /// Filter by a single impact level
    #[must_use]
    pub fn impact(mut self, impact: Impact) -> Self {
        self.impacts.push(impact);
        self
    }

    /// Filter by multiple impact levels
    pub fn impacts(mut self, impacts: impl IntoIterator<Item = Impact>) -> Self {
        self.impacts.extend(impacts);
        self
    }

    /// Filter by audit firm
    pub fn firm(mut self, firm: impl Into<FilterValue>) -> Self {
        self.firms.push(firm.into());
        self
    }

    /// Filter by multiple audit firms
    pub fn firms(mut self, firms: impl IntoIterator<Item = FilterValue>) -> Self {
        self.firms.extend(firms);
        self
    }

    /// Filter by tag
    pub fn tag(mut self, tag: impl Into<FilterValue>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Filter by multiple tags
    pub fn tags(mut self, tags: impl IntoIterator<Item = FilterValue>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Filter by protocol name (partial match)
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Filter by protocol category
    pub fn protocol_category(mut self, category: impl Into<FilterValue>) -> Self {
        self.protocol_categories.push(category.into());
        self
    }

    /// Filter by programming language
    pub fn language(mut self, lang: impl Into<FilterValue>) -> Self {
        self.languages.push(lang.into());
        self
    }

    /// Filter by finder/auditor handle
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Filter by number of finders
    #[must_use]
    pub fn finders_range(mut self, min: Option<u32>, max: Option<u32>) -> Self {
        self.min_finders = min;
        self.max_finders = max;
        self
    }

    /// Filter by report date period
    #[must_use]
    pub fn reported(mut self, period: ReportedPeriod) -> Self {
        self.reported = Some(period);
        self
    }

    /// Filter by reports after a specific date
    pub fn reported_after(mut self, date: impl Into<String>) -> Self {
        self.reported = Some(ReportedPeriod::After);
        self.reported_after = Some(date.into());
        self
    }

    /// Set minimum quality score (0-5)
    #[must_use]
    pub fn min_quality(mut self, score: u32) -> Self {
        self.quality_score = Some(score.min(5));
        self
    }

    /// Set minimum rarity score (0-5)
    #[must_use]
    pub fn min_rarity(mut self, score: u32) -> Self {
        self.rarity_score = Some(score.min(5));
        self
    }

    /// Sort by recency (newest first by default)
    #[must_use]
    pub fn sort_by_recency(mut self) -> Self {
        self.sort_field = SortField::Recency;
        self
    }

    /// Sort by quality score
    #[must_use]
    pub fn sort_by_quality(mut self) -> Self {
        self.sort_field = SortField::Quality;
        self
    }

    /// Sort by rarity score
    #[must_use]
    pub fn sort_by_rarity(mut self) -> Self {
        self.sort_field = SortField::Rarity;
        self
    }

    /// Sort ascending
    #[must_use]
    pub fn ascending(mut self) -> Self {
        self.sort_direction = SortDirection::Asc;
        self
    }

    /// Sort descending
    #[must_use]
    pub fn descending(mut self) -> Self {
        self.sort_direction = SortDirection::Desc;
        self
    }

    /// Create a copy of this filter with a different page number
    ///
    /// Used internally by the paginator to avoid manual field-by-field cloning.
    #[must_use]
    pub fn with_page(&self, page: u32) -> Self {
        Self {
            keywords: self.keywords.clone(),
            impacts: self.impacts.clone(),
            firms: self.firms.clone(),
            tags: self.tags.clone(),
            protocol: self.protocol.clone(),
            protocol_categories: self.protocol_categories.clone(),
            forked: self.forked.clone(),
            languages: self.languages.clone(),
            user: self.user.clone(),
            min_finders: self.min_finders,
            max_finders: self.max_finders,
            reported: self.reported,
            reported_after: self.reported_after.clone(),
            quality_score: self.quality_score,
            rarity_score: self.rarity_score,
            page,
            page_size: self.page_size,
            sort_field: self.sort_field,
            sort_direction: self.sort_direction,
        }
    }
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Total findings matching the filter
    #[serde(rename = "totalResults")]
    pub total_results: u64,

    /// Current page number
    #[serde(rename = "currentPage")]
    pub current_page: u32,

    /// Results per page
    #[serde(rename = "pageSize")]
    pub page_size: u32,

    /// Total pages available
    #[serde(rename = "totalPages")]
    pub total_pages: u32,

    /// Query execution time in seconds
    #[serde(default)]
    pub elapsed: Option<f64>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per window
    pub limit: u32,

    /// Remaining requests in current window
    pub remaining: u32,

    /// Unix timestamp when the window resets
    pub reset: u64,
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    /// List of findings
    pub findings: Vec<Finding>,

    /// Response metadata
    pub metadata: ResponseMetadata,

    /// Rate limit information
    #[serde(rename = "rateLimit")]
    pub rate_limit: RateLimit,
}

/// Search results (convenient wrapper around `ApiResponse`)
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// List of findings
    pub findings: Vec<Finding>,

    /// Total count of matching findings
    pub total: u64,

    /// Current page (1-indexed)
    pub page: u32,

    /// Page size
    pub page_size: u32,

    /// Total pages
    pub total_pages: u32,

    /// Rate limit info
    pub rate_limit: RateLimit,
}

impl SearchResults {
    /// Create from API response
    #[must_use]
    pub fn from_response(response: ApiResponse) -> Self {
        Self {
            findings: response.findings,
            total: response.metadata.total_results,
            page: response.metadata.current_page,
            page_size: response.metadata.page_size,
            total_pages: response.metadata.total_pages,
            rate_limit: response.rate_limit,
        }
    }

    /// Check if there are more pages
    #[must_use]
    pub fn has_more(&self) -> bool {
        self.page < self.total_pages
    }

    /// Get remaining rate limit
    #[must_use]
    pub fn rate_limit_remaining(&self) -> u32 {
        self.rate_limit.remaining
    }
}
