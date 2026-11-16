//! News and insights API endpoints

use super::types::{Insight, InsightsOptions, NewsArticle, NewsOptions};
use crate::client::Client;
use crate::error::Result;

/// News and insights API (Analyst plan and above)
pub struct NewsApi<'a> {
    client: &'a Client,
}

impl<'a> NewsApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the latest crypto news and guides (Analyst plan and above)
    ///
    /// Without a coin ID this returns news only; with one, news and guides.
    /// At most 20 pages of 20 articles.
    pub async fn news(&self, options: &NewsOptions) -> Result<Vec<NewsArticle>> {
        let path = format!("/news{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get the latest coin insights (Analyst plan and above)
    ///
    /// At most 20 pages of 20 insights.
    pub async fn insights(&self, options: &InsightsOptions) -> Result<Vec<Insight>> {
        let path = format!("/insights{}", options.to_query_string());
        self.client.get(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::Mock;

    #[tokio::test]
    async fn news() {
        let mock = Mock::new(
            r#"[{"title":"Implied Volatility Explained","url":"https://example.com/iv",
                "image":"https://example.com/iv.png","author":"Editorial Team",
                "posted_at":"2026-05-25T13:08:37Z","type":"guide","source_name":"CoinGecko",
                "related_coin_ids":["bitcoin","ethereum"]}]"#,
        );
        let r = mock
            .client
            .news()
            .news(
                &NewsOptions::new()
                    .page(2)
                    .per_page(20)
                    .coin_id("bitcoin")
                    .language("zh-tw")
                    .news_type("guides"),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/news?page=2&per_page=20&coin_id=bitcoin&language=zh-tw&type=guides"
        );
        assert_eq!(r[0].article_type.as_deref(), Some("guide"));
        assert_eq!(r[0].related_coin_ids, ["bitcoin", "ethereum"]);
    }

    #[tokio::test]
    async fn insights() {
        let mock = Mock::new(
            r#"[{"title":"Rising memecoin launches","description":"More launches.",
                "related_coin_ids":["wrapped-solana"],"posted_at":"2026-06-15T03:56:19.514666Z"}]"#,
        );
        let r = mock
            .client
            .news()
            .insights(
                &InsightsOptions::new()
                    .coin_id("solana")
                    .range("2026-06-01", "2026-06-15"),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/insights?coin_id=solana&from=2026-06-01&to=2026-06-15"
        );
        assert_eq!(r[0].title, "Rising memecoin launches");
        assert_eq!(r[0].related_coin_ids, ["wrapped-solana"]);
    }
}
