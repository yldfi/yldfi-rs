//! Type definitions for Solodit API responses

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Tolerant deserialization helpers
//
// The Solodit API is loosely typed in practice: IDs are BigInts that may be
// serialized as strings or numbers, `report_date` is sometimes `{}` (see
// solodit/solodit_content#153), and nested relations may be `null`. These
// helpers never fail on an unexpected *value*; they fall back to `None` /
// default instead so that one odd field cannot fail a whole page.
// ---------------------------------------------------------------------------

/// Convert a JSON value to a string, accepting numbers/bools and treating
/// `null`, `""` and objects/arrays (e.g. `{}`) as absent.
fn value_to_string(value: Value) -> Option<String> {
    match value {
        Value::String(s) if !s.is_empty() => Some(s),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// Convert a JSON value (number or numeric string) to `f64`.
fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
    .filter(|f| f.is_finite())
}

/// Convert a JSON value (number or numeric string) to `i64`.
fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64().or_else(|| {
            n.as_f64()
                .filter(|f| f.is_finite() && f.fract() == 0.0)
                .map(|f| f as i64)
        }),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// Convert a JSON value (number or numeric string) to `u64`.
fn value_to_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse::<u64>().ok(),
        _ => None,
    }
    .or_else(|| value_to_i64(value).and_then(|i| u64::try_from(i).ok()))
}

/// `Option<String>` accepting string, number (e.g. BigInt IDs), `null` or `{}`.
fn de_opt_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_string(Value::deserialize(deserializer)?))
}

/// `String` accepting string/number, defaulting to empty on anything else.
fn de_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_string(Value::deserialize(deserializer)?).unwrap_or_default())
}

/// `Option<f64>` accepting number or numeric string.
fn de_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_f64(&Value::deserialize(deserializer)?))
}

/// `Option<i32>` accepting number or numeric string.
fn de_opt_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_i64(&Value::deserialize(deserializer)?).and_then(|i| i32::try_from(i).ok()))
}

/// `u32` accepting number or numeric string, defaulting to 0.
fn de_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_u64(&Value::deserialize(deserializer)?)
        .and_then(|u| u32::try_from(u).ok())
        .unwrap_or_default())
}

/// `u64` accepting number or numeric string, defaulting to 0.
fn de_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(value_to_u64(&Value::deserialize(deserializer)?).unwrap_or_default())
}

/// `bool` defaulting to `false` on anything that is not a JSON boolean.
fn de_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(matches!(
        Value::deserialize(deserializer)?,
        Value::Bool(true)
    ))
}

/// `Option<T>` that becomes `None` if the value does not match `T`.
fn de_opt_lenient<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(serde_json::from_value(Value::deserialize(deserializer)?).ok())
}

/// `T` that falls back to `T::default()` if the value does not match `T`.
fn de_lenient<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned + Default,
{
    Ok(serde_json::from_value(Value::deserialize(deserializer)?).unwrap_or_default())
}

/// `Vec<T>` that accepts `null`/non-arrays (empty) and skips elements that
/// fail to deserialize instead of failing the whole vector.
fn de_vec_lenient<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::Array(items) => items
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
        _ => Vec::new(),
    })
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
    #[serde(default, deserialize_with = "de_opt_string")]
    pub name: Option<String>,
    /// URL to firm's square logo
    #[serde(default, deserialize_with = "de_opt_string")]
    pub logo_square: Option<String>,
}

/// Protocol category score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCategoryScore {
    /// The category information
    #[serde(default, deserialize_with = "de_opt_lenient")]
    pub protocols_protocolcategory: Option<ProtocolCategory>,
    /// Score for this category
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub score: Option<f64>,
}

/// Protocol category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCategory {
    /// Category title
    #[serde(default, deserialize_with = "de_opt_string")]
    pub title: Option<String>,
}

/// Protocol information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    /// Protocol name
    #[serde(default, deserialize_with = "de_opt_string")]
    pub name: Option<String>,
    /// Category scores for the protocol.
    ///
    /// Known upstream quirk: currently always `[]` (solodit/solodit_content#153).
    #[serde(default, deserialize_with = "de_vec_lenient")]
    pub protocols_protocolcategoryscore: Vec<ProtocolCategoryScore>,
}

/// Warden/auditor who found the issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warden {
    /// Auditor handle/username
    #[serde(default, deserialize_with = "de_string")]
    pub handle: String,
}

/// Issue finder information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueFinder {
    /// The warden who found the issue
    #[serde(default, deserialize_with = "de_opt_lenient")]
    pub wardens_warden: Option<Warden>,
}

/// Tag score associated with a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTagScore {
    /// Tag information
    #[serde(default, deserialize_with = "de_opt_lenient")]
    pub tags_tag: Option<IssueTag>,
}

/// Tag associated with a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTag {
    /// Tag title
    #[serde(default, deserialize_with = "de_opt_string")]
    pub title: Option<String>,
}

/// A vulnerability report/finding from Solodit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Unique identifier.
    ///
    /// The API serializes BigInt IDs; both JSON strings and numbers are accepted
    /// and normalized to a string.
    #[serde(default, deserialize_with = "de_opt_string")]
    pub id: Option<String>,

    /// URL-friendly slug
    #[serde(default, deserialize_with = "de_opt_string")]
    pub slug: Option<String>,

    /// Finding title
    #[serde(default, deserialize_with = "de_opt_string")]
    pub title: Option<String>,

    /// Full content/description (markdown)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub content: Option<String>,

    /// Summary of the finding
    #[serde(default, deserialize_with = "de_opt_string")]
    pub summary: Option<String>,

    /// Content kind (e.g., "MARKDOWN")
    #[serde(default, deserialize_with = "de_opt_string")]
    pub kind: Option<String>,

    /// Impact/severity level
    #[serde(default, deserialize_with = "de_opt_string")]
    pub impact: Option<String>,

    /// Quality score (0-5)
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub quality_score: Option<f64>,

    /// General/rarity score (0-5)
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub general_score: Option<f64>,

    /// Report date (ISO string).
    ///
    /// Known upstream quirk: sometimes returned as `{}`, which is mapped to `None`.
    #[serde(default, deserialize_with = "de_opt_string")]
    pub report_date: Option<String>,

    /// Audit firm ID
    #[serde(default, deserialize_with = "de_opt_string")]
    pub auditfirm_id: Option<String>,

    /// Firm name (flattened)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub firm_name: Option<String>,

    /// Firm logo URL (flattened)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub firm_logo_square: Option<String>,

    /// Audit firm that conducted the review
    #[serde(default, deserialize_with = "de_opt_lenient")]
    pub auditfirms_auditfirm: Option<AuditFirm>,

    /// Protocol ID
    #[serde(default, deserialize_with = "de_opt_string")]
    pub protocol_id: Option<String>,

    /// Protocol name (flattened)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub protocol_name: Option<String>,

    /// Protocol that was audited
    #[serde(default, deserialize_with = "de_opt_lenient")]
    pub protocols_protocol: Option<Protocol>,

    /// Contest ID (for competitive audits)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub contest_id: Option<String>,

    /// Contest link
    #[serde(default, deserialize_with = "de_opt_string")]
    pub contest_link: Option<String>,

    /// Contest prize text
    #[serde(default, deserialize_with = "de_opt_string")]
    pub contest_prize_txt: Option<String>,

    /// Sponsor name
    #[serde(default, deserialize_with = "de_opt_string")]
    pub sponsor_name: Option<String>,

    /// Sponsor link
    #[serde(default, deserialize_with = "de_opt_string")]
    pub sponsor_link: Option<String>,

    /// Number of finders
    #[serde(default, deserialize_with = "de_opt_i32")]
    pub finders_count: Option<i32>,

    /// People who found this issue
    #[serde(default, deserialize_with = "de_vec_lenient")]
    pub issues_issue_finders: Vec<IssueFinder>,

    /// Tags associated with the finding (`issues_issuetagscore`; some docs
    /// incorrectly call this `issues_issuetags`).
    #[serde(default, deserialize_with = "de_vec_lenient")]
    pub issues_issuetagscore: Vec<IssueTagScore>,

    /// Source link (original report)
    #[serde(default, deserialize_with = "de_opt_string")]
    pub source_link: Option<String>,

    /// GitHub link
    #[serde(default, deserialize_with = "de_opt_string")]
    pub github_link: Option<String>,

    /// PDF link
    #[serde(default, deserialize_with = "de_opt_string")]
    pub pdf_link: Option<String>,

    /// PDF page start
    #[serde(default, deserialize_with = "de_opt_i32")]
    pub pdf_page_from: Option<i32>,

    /// Whether bookmarked (always false for API)
    #[serde(default, deserialize_with = "de_bool")]
    pub bookmarked: bool,

    /// Whether read (always false for API)
    #[serde(default, deserialize_with = "de_bool")]
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
            .filter(|h| !h.is_empty())
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

    /// Filter by forked protocol (e.g. "Uniswap V2")
    pub fn forked(mut self, forked: impl Into<FilterValue>) -> Self {
        self.forked.push(forked.into());
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
    /// Used internally by the paginator.
    #[must_use]
    pub fn with_page(&self, page: u32) -> Self {
        Self {
            page,
            ..self.clone()
        }
    }
}

/// Response metadata
///
/// All fields are parsed leniently (numbers or numeric strings); missing or
/// malformed values default to `0` / `None`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Total findings matching the filter
    #[serde(rename = "totalResults", default, deserialize_with = "de_u64")]
    pub total_results: u64,

    /// Current page number
    #[serde(rename = "currentPage", default, deserialize_with = "de_u32")]
    pub current_page: u32,

    /// Results per page
    #[serde(rename = "pageSize", default, deserialize_with = "de_u32")]
    pub page_size: u32,

    /// Total pages available
    #[serde(rename = "totalPages", default, deserialize_with = "de_u32")]
    pub total_pages: u32,

    /// Query execution time in seconds
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub elapsed: Option<f64>,
}

/// Rate limit information
///
/// Available both in the response body (`rateLimit`) and in the
/// `X-RateLimit-Limit` / `X-RateLimit-Remaining` / `X-RateLimit-Reset`
/// response headers. The documented default is 20 requests per 60 seconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per window
    #[serde(default, deserialize_with = "de_u32")]
    pub limit: u32,

    /// Remaining requests in current window
    #[serde(default, deserialize_with = "de_u32")]
    pub remaining: u32,

    /// Unix timestamp when the window resets
    #[serde(default, deserialize_with = "de_u64")]
    pub reset: u64,
}

/// API response wrapper (raw `POST /findings` success body)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiResponse {
    /// List of findings. Individual entries that are not JSON objects are
    /// skipped rather than failing the whole page.
    #[serde(default, deserialize_with = "de_vec_lenient")]
    pub findings: Vec<Finding>,

    /// Response metadata
    #[serde(default, deserialize_with = "de_lenient")]
    pub metadata: ResponseMetadata,

    /// Rate limit information from the body (`None` if absent or malformed)
    #[serde(rename = "rateLimit", default, deserialize_with = "de_opt_lenient")]
    pub rate_limit: Option<RateLimit>,
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

    /// Query execution time in seconds (from `metadata.elapsed`)
    pub elapsed: Option<f64>,

    /// Effective rate limit info: the body's `rateLimit` if present, otherwise
    /// the `X-RateLimit-*` headers, otherwise zeroes.
    pub rate_limit: RateLimit,

    /// Rate limit info parsed from the `X-RateLimit-*` response headers, if any
    pub rate_limit_headers: Option<RateLimit>,
}

impl SearchResults {
    /// Create from API response
    #[must_use]
    pub fn from_response(response: ApiResponse) -> Self {
        let meta = response.metadata;
        let mut total_pages = meta.total_pages;
        // Derive total pages if the server omitted it.
        if total_pages == 0 && meta.total_results > 0 && meta.page_size > 0 {
            total_pages = u32::try_from(meta.total_results.div_ceil(u64::from(meta.page_size)))
                .unwrap_or(u32::MAX);
        }
        Self {
            findings: response.findings,
            total: meta.total_results,
            page: meta.current_page,
            page_size: meta.page_size,
            total_pages,
            elapsed: meta.elapsed,
            rate_limit: response.rate_limit.unwrap_or_default(),
            rate_limit_headers: None,
        }
    }

    /// Attach rate limit info parsed from response headers.
    ///
    /// If the body did not contain `rateLimit`, the header values become the
    /// effective [`SearchResults::rate_limit`].
    #[must_use]
    pub fn with_header_rate_limit(
        mut self,
        body_had_rate_limit: bool,
        headers: Option<RateLimit>,
    ) -> Self {
        if !body_had_rate_limit {
            if let Some(h) = headers {
                self.rate_limit = h;
            }
        }
        self.rate_limit_headers = headers;
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finding_tolerates_odd_fields() {
        let json = serde_json::json!({
            "id": 12345678901234567_u64,
            "slug": "h-01-foo",
            "report_date": {},
            "quality_score": "4.5",
            "general_score": null,
            "finders_count": "3",
            "auditfirm_id": 42,
            "auditfirms_auditfirm": null,
            "protocols_protocol": "unexpected",
            "issues_issue_finders": [
                {"wardens_warden": {"handle": "alice"}},
                {"wardens_warden": null},
                "garbage",
                {"wardens_warden": {"handle": 7}}
            ],
            "issues_issuetagscore": null,
            "pdf_page_from": 1.0,
            "bookmarked": "no"
        });
        let f: Finding = serde_json::from_value(json).unwrap();
        assert_eq!(f.id.as_deref(), Some("12345678901234567"));
        assert_eq!(f.report_date, None);
        assert_eq!(f.quality_score, Some(4.5));
        assert_eq!(f.general_score, None);
        assert_eq!(f.finders_count, Some(3));
        assert_eq!(f.auditfirm_id.as_deref(), Some("42"));
        assert!(f.auditfirms_auditfirm.is_none());
        assert!(f.protocols_protocol.is_none());
        assert_eq!(f.finder_handles(), vec!["alice", "7"]);
        assert!(f.tags().is_empty());
        assert_eq!(f.pdf_page_from, Some(1));
        assert!(!f.bookmarked);
    }

    #[test]
    fn api_response_tolerates_missing_sections() {
        let r: ApiResponse = serde_json::from_str(r#"{"findings":[{"id":"1"}, 5]}"#).unwrap();
        assert_eq!(r.findings.len(), 1);
        assert!(r.rate_limit.is_none());
        assert_eq!(r.metadata.total_results, 0);
    }

    #[test]
    fn total_pages_derived_when_missing() {
        let r: ApiResponse = serde_json::from_str(
            r#"{"findings":[],"metadata":{"totalResults":"101","currentPage":1,"pageSize":50}}"#,
        )
        .unwrap();
        let s = SearchResults::from_response(r);
        assert_eq!(s.total, 101);
        assert_eq!(s.total_pages, 3);
    }

    #[test]
    fn header_rate_limit_used_when_body_missing() {
        let s = SearchResults::from_response(ApiResponse::default()).with_header_rate_limit(
            false,
            Some(RateLimit {
                limit: 20,
                remaining: 19,
                reset: 1_700_000_000,
            }),
        );
        assert_eq!(s.rate_limit.remaining, 19);
        assert!(s.rate_limit_headers.is_some());
    }
}
