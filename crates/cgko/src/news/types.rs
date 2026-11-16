//! Types for news and insights endpoints

use crate::query::Query;
use serde::{Deserialize, Serialize};

/// Options for [`super::NewsApi::news`]
#[derive(Debug, Clone, Default)]
pub struct NewsOptions {
    /// Page through results, 1 to 20 (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 20 (API default: 10)
    pub per_page: Option<u32>,
    /// Only news about this coin ID
    pub coin_id: Option<String>,
    /// Language code, e.g. `en` (API default), `de`, `ja`, `zh-tw`
    pub language: Option<String>,
    /// `all` (API default), `news` or `guides`; `guides` needs `coin_id`
    pub news_type: Option<String>,
}

impl NewsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number (1 to 20)
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 20)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Only news about this coin ID
    #[must_use]
    pub fn coin_id(mut self, coin_id: impl Into<String>) -> Self {
        self.coin_id = Some(coin_id.into());
        self
    }

    /// Filter by language code
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// `all`, `news` or `guides`
    #[must_use]
    pub fn news_type(mut self, news_type: impl Into<String>) -> Self {
        self.news_type = Some(news_type.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        Query::new()
            .value("page", self.page)
            .value("per_page", self.per_page)
            .text("coin_id", self.coin_id.as_deref())
            .text("language", self.language.as_deref())
            .text("type", self.news_type.as_deref())
            .finish()
    }
}

/// A news article or guide
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewsArticle {
    pub title: String,
    pub url: Option<String>,
    pub image: Option<String>,
    pub author: Option<String>,
    /// ISO 8601 time
    pub posted_at: Option<String>,
    /// `news` or `guide`
    #[serde(rename = "type")]
    pub article_type: Option<String>,
    pub source_name: Option<String>,
    #[serde(default)]
    pub related_coin_ids: Vec<String>,
}

/// Options for [`super::NewsApi::insights`]
#[derive(Debug, Clone, Default)]
pub struct InsightsOptions {
    /// Page through results, 1 to 20 (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 20 (API default: 10)
    pub per_page: Option<u32>,
    /// Only insights about this coin ID
    pub coin_id: Option<String>,
    /// Start date, `YYYY-MM-DD`
    pub from: Option<String>,
    /// End date, `YYYY-MM-DD`
    pub to: Option<String>,
}

impl InsightsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number (1 to 20)
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 20)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Only insights about this coin ID
    #[must_use]
    pub fn coin_id(mut self, coin_id: impl Into<String>) -> Self {
        self.coin_id = Some(coin_id.into());
        self
    }

    /// Date window, each `YYYY-MM-DD`
    #[must_use]
    pub fn range(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self.to = Some(to.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        Query::new()
            .value("page", self.page)
            .value("per_page", self.per_page)
            .text("coin_id", self.coin_id.as_deref())
            .text("from", self.from.as_deref())
            .text("to", self.to.as_deref())
            .finish()
    }
}

/// A short insight about a coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Insight {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub related_coin_ids: Vec<String>,
    /// ISO 8601 time
    pub posted_at: Option<String>,
}
