//! Token API client

use super::types::*;
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
    pub fn include_spam(mut self, include: bool) -> Self {
        self.include_spam = Some(include);
        self
    }
}

/// API for token operations
pub struct TokenApi<'a> {
    client: &'a Client,
}

impl<'a> TokenApi<'a> {
    /// Create a new token API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token metadata
    pub async fn get_metadata(&self, address: &str, chain: Option<&str>) -> Result<TokenMetadata> {
        let path = format!("/erc20/metadata");
        let query = TokenQuery::new();
        let query = if let Some(c) = chain {
            query.chain(c)
        } else {
            query
        };

        #[derive(Serialize)]
        struct MetadataQuery {
            addresses: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            chain: Option<String>,
        }

        let q = MetadataQuery {
            addresses: vec![address.to_string()],
            chain: query.chain,
        };

        let results: Vec<TokenMetadata> = self.client.get_with_query(&path, &q).await?;
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
        let path = format!("/erc20/{}/price", address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token transfers for an address
    pub async fn get_transfers(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<Vec<TokenTransfer>> {
        let path = format!("/{}/erc20/transfers", address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token pairs (DEX liquidity pools)
    pub async fn get_pairs(&self, address: &str, chain: Option<&str>) -> Result<Vec<TokenPair>> {
        let path = format!("/erc20/{}/pairs", address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get top token holders
    pub async fn get_holders(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<TokenHoldersResponse> {
        let path = format!("/erc20/{}/owners", address);
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
        let path = format!("/erc20/{}/swaps", address);
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
        let path = format!("/wallets/{}/swaps", address);
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
        let path = format!("/pairs/{}/swaps", pair_address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token stats
    pub async fn get_stats(&self, address: &str, chain: Option<&str>) -> Result<TokenStats> {
        let path = format!("/erc20/{}/stats", address);
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
            chain: chain.map(|s| s.to_string()),
        };

        self.client.get_with_query("/tokens/search", &q).await
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
    pub async fn get_pair_ohlcv(
        &self,
        pair_address: &str,
        chain: Option<&str>,
    ) -> Result<Vec<PairOhlcv>> {
        let path = format!("/pairs/{}/ohlcv", pair_address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get pair stats
    pub async fn get_pair_stats(
        &self,
        pair_address: &str,
        chain: Option<&str>,
    ) -> Result<PairStats> {
        let path = format!("/pairs/{}/stats", pair_address);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get new tokens on an exchange
    pub async fn get_exchange_new_tokens(
        &self,
        exchange_name: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<NewToken>> {
        let path = format!("/erc20/exchange/{}/new", exchange_name);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get bonding tokens on an exchange (e.g., pump.fun)
    pub async fn get_exchange_bonding_tokens(
        &self,
        exchange_name: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<NewToken>> {
        let path = format!("/erc20/exchange/{}/bonding", exchange_name);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get graduated tokens on an exchange
    pub async fn get_exchange_graduated_tokens(
        &self,
        exchange_name: &str,
        chain: Option<&str>,
    ) -> Result<TokenResponse<NewToken>> {
        let path = format!("/erc20/exchange/{}/graduated", exchange_name);
        if let Some(chain) = chain {
            let query = TokenQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }
}
