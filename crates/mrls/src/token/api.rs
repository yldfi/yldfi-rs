//! Token API client

use super::types::{
    GetMultiplePricesRequest, PairOhlcvResponse, PairStats, TokenCategory, TokenHoldersResponse,
    TokenHoldersSummary, TokenMetadata, TokenPair, TokenPairsResponse, TokenPrice, TokenResponse,
    TokenSearchResponse, TokenSearchResult, TokenSwap, TokenTransfer, TokenTransferResponse,
    TopGainersResponse, TrendingToken,
};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for token endpoints
#[derive(Debug, Default, Serialize)]
pub struct TokenQuery {
    /// Chain to query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Include spam tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_spam: Option<bool>,
}

impl TokenQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chain
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    /// Include spam tokens
    #[must_use]
    #[allow(dead_code)]
    pub fn include_spam(mut self, include: bool) -> Self {
        self.include_spam = Some(include);
        self
    }
}

/// Query parameters for pair OHLCV endpoints
///
/// Moralis requires `timeframe`, `currency`, `fromDate` and `toDate`; any left
/// unset are filled in by [`TokenApi::get_pair_ohlcv`] (1d, usd, last 7 days).
#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PairOhlcvQuery {
    /// Chain to query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Timeframe (e.g., 1h, 4h, 1d, 1w)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    /// Price currency (`usd` or `native`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    /// From date (unix seconds or ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    /// To date (unix seconds or ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl PairOhlcvQuery {
    /// Create a new query
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chain
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    /// Set timeframe
    #[must_use]
    pub fn timeframe(mut self, timeframe: impl Into<String>) -> Self {
        self.timeframe = Some(timeframe.into());
        self
    }

    /// Set currency (`usd` or `native`)
    #[must_use]
    pub fn currency(mut self, currency: impl Into<String>) -> Self {
        self.currency = Some(currency.into());
        self
    }

    /// Set from_date
    #[must_use]
    pub fn from_date(mut self, from_date: impl Into<String>) -> Self {
        self.from_date = Some(from_date.into());
        self
    }

    /// Set to_date
    #[must_use]
    pub fn to_date(mut self, to_date: impl Into<String>) -> Self {
        self.to_date = Some(to_date.into());
        self
    }

    /// Set limit
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set pagination cursor
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Fill in the parameters Moralis requires: timeframe `1d`, currency `usd`
    /// and a date range of the last 7 days ending now (unix seconds).
    fn with_required_defaults(&self, now_secs: u64) -> Self {
        let mut q = self.clone();
        q.timeframe.get_or_insert_with(|| "1d".to_string());
        q.currency.get_or_insert_with(|| "usd".to_string());
        q.to_date.get_or_insert_with(|| now_secs.to_string());
        q.from_date
            .get_or_insert_with(|| now_secs.saturating_sub(7 * 86_400).to_string());
        q
    }
}

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// API for token operations
pub struct TokenApi<'a> {
    client: &'a Client,
}

impl<'a> TokenApi<'a> {
    /// Create a new token API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token metadata
    pub async fn get_metadata(&self, address: &str, chain: Option<&str>) -> Result<TokenMetadata> {
        // `addresses` is an array param; a single value is sent as a plain
        // pair. A pair list lets reqwest URL-encode the values.
        let mut query = vec![("addresses", address)];
        if let Some(chain) = chain {
            query.push(("chain", chain));
        }

        let results: Vec<TokenMetadata> = self
            .client
            .get_with_query("/erc20/metadata", &query)
            .await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| crate::error::Error::Api {
                status: 404,
                message: "Token not found".to_string(),
            })
    }

    /// Get token price
    pub async fn get_price(&self, address: &str, chain: Option<&str>) -> Result<TokenPrice> {
        let path = format!("/erc20/{address}/price");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token transfers for an address
    ///
    /// Moralis API returns transfers wrapped in a paginated response object.
    pub async fn get_transfers(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<Vec<TokenTransfer>> {
        let path = format!("/{address}/erc20/transfers");
        let response: TokenTransferResponse = if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await?
        } else {
            self.client.get(&path).await?
        };
        Ok(response.result)
    }

    /// Get token pairs (DEX liquidity pools)
    pub async fn get_pairs(&self, address: &str, chain: Option<&str>) -> Result<Vec<TokenPair>> {
        let path = format!("/erc20/{address}/pairs");
        let response: TokenPairsResponse = if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await?
        } else {
            self.client.get(&path).await?
        };
        Ok(response.pairs)
    }

    /// Get top token holders
    pub async fn get_holders(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenHoldersResponse> {
        let path = format!("/erc20/{address}/owners");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token swaps
    pub async fn get_swaps(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<TokenSwap>> {
        let path = format!("/erc20/{address}/swaps");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get wallet swaps
    pub async fn get_wallet_swaps(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<TokenSwap>> {
        let path = format!("/wallets/{address}/swaps");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get pair swaps
    pub async fn get_pair_swaps(
        &self,
        pair_address: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<TokenSwap>> {
        let path = format!("/pairs/{pair_address}/swaps");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Search tokens
    pub async fn search(
        &self,
        query_str: &str,
        chain: Option<&str>,
    ) -> Result<Vec<TokenSearchResult>> {
        #[derive(Serialize)]
        struct SearchQuery {
            query: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            chain: Option<String>,
        }

        let q = SearchQuery {
            query: query_str.to_string(),
            chain: chain.map(std::string::ToString::to_string),
        };

        // Moralis wraps search results in a `{ result, total }` envelope
        let response: TokenSearchResponse =
            self.client.get_with_query("/tokens/search", &q).await?;
        Ok(response.result)
    }

    /// Get trending tokens
    pub async fn get_trending(&self, chain: Option<&str>) -> Result<Vec<TrendingToken>> {
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query("/tokens/trending", &query).await
        } else {
            self.client.get("/tokens/trending").await
        }
    }

    /// Get token categories
    pub async fn get_categories(&self) -> Result<Vec<TokenCategory>> {
        self.client.get("/tokens/categories").await
    }

    /// Get pair OHLCV data
    ///
    /// The Moralis API requires `timeframe`, `currency`, `fromDate` and `toDate`.
    /// Any of these not set on `query` default to `1d`, `usd` and the last 7 days.
    pub async fn get_pair_ohlcv(
        &self,
        pair_address: &str,
        query: Option<&PairOhlcvQuery>,
    ) -> Result<PairOhlcvResponse> {
        let path = format!("/pairs/{pair_address}/ohlcv");
        let q = query
            .cloned()
            .unwrap_or_default()
            .with_required_defaults(unix_now_secs());
        self.client.get_with_query(&path, &q).await
    }

    /// Get pair stats
    pub async fn get_pair_stats(
        &self,
        pair_address: &str,
        chain: Option<&str>,
    ) -> Result<PairStats> {
        let path = format!("/pairs/{pair_address}/stats");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get multiple token prices (batch)
    pub async fn get_multiple_prices(
        &self,
        request: &GetMultiplePricesRequest,
        chain: Option<&str>,
    ) -> Result<Vec<TokenPrice>> {
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client
                .post_with_query("/erc20/prices", request, &query)
                .await
        } else {
            self.client.post("/erc20/prices", request).await
        }
    }

    /// Get contract transfers for a token (not wallet transfers)
    pub async fn get_contract_transfers(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<TokenTransfer>> {
        let path = format!("/erc20/{address}/transfers");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get holders summary for a token
    pub async fn get_holders_summary(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenHoldersSummary> {
        let path = format!("/erc20/{address}/holders");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get top traders/gainers for a token
    ///
    /// Moralis returns the token's metadata with the traders in `result`.
    pub async fn get_top_gainers(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TopGainersResponse> {
        let path = format!("/erc20/{address}/top-gainers");
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query_pairs(q: &PairOhlcvQuery) -> Vec<(String, String)> {
        let req = reqwest::Client::new()
            .get("https://example.invalid/pairs/0xpair/ohlcv")
            .query(q)
            .build()
            .unwrap();
        req.url()
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    #[test]
    fn test_pair_ohlcv_query_fills_required_params() {
        let now = 1_700_000_000;
        let q = PairOhlcvQuery::new()
            .chain("eth")
            .with_required_defaults(now);
        let pairs = query_pairs(&q);
        let get = |k: &str| {
            pairs
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("chain"), Some("eth"));
        assert_eq!(get("timeframe"), Some("1d"));
        assert_eq!(get("currency"), Some("usd"));
        assert_eq!(get("toDate"), Some("1700000000"));
        assert_eq!(get("fromDate"), Some("1699395200"));
    }

    #[test]
    fn test_pair_ohlcv_query_keeps_caller_values() {
        let q = PairOhlcvQuery::new()
            .timeframe("1h")
            .currency("native")
            .from_date("2024-01-01T00:00:00Z")
            .to_date("2024-01-02T00:00:00Z")
            .cursor("abc")
            .with_required_defaults(1_700_000_000);
        let pairs = query_pairs(&q);
        assert!(pairs.contains(&("timeframe".into(), "1h".into())));
        assert!(pairs.contains(&("currency".into(), "native".into())));
        assert!(pairs.contains(&("fromDate".into(), "2024-01-01T00:00:00Z".into())));
        assert!(pairs.contains(&("toDate".into(), "2024-01-02T00:00:00Z".into())));
        assert!(pairs.contains(&("cursor".into(), "abc".into())));
    }
}
