//! Subgraph client for historical Uniswap data
//!
//! This module provides access to indexed historical data via The Graph's
//! Uniswap subgraphs. Requires an API key from The Graph Studio.

use reqwest::Client as HttpClient;

/// Uniswap protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UniswapVersion {
    /// Uniswap V2
    V2,
    /// Uniswap V3
    #[default]
    V3,
    /// Uniswap V4
    V4,
}
use serde::de::DeserializeOwned;
use serde_json::json;
use url::Url;
use yldfi_common::http::HttpClientConfig;

use crate::error::{subgraph_error, subgraph_key_required, Result};
use crate::types::{
    GraphQLResponse, LiquidityPositionV2, PairDataV2, PoolData, PoolDataV4, PoolDayData, Position,
    PositionV4, Swap,
};

/// Subgraph IDs for Uniswap on various chains
pub mod subgraph_ids {
    // === V2 Subgraphs ===
    /// Ethereum Mainnet V2
    pub const MAINNET_V2: &str = "EYCKATKGBKLWvSfwvBjzfCBmGwYNdVkduYXVivCsLRFu";

    // === V3 Subgraphs ===
    /// Ethereum Mainnet V3
    pub const MAINNET_V3: &str = "5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV";
    /// Arbitrum V3
    pub const ARBITRUM_V3: &str = "FbCGRftH4a3yZugY7TnbYgPJVEv2LvMT6oF1fxPe9aJM";
    /// Optimism V3
    pub const OPTIMISM_V3: &str = "Cghf4LfVqPiFw6fp6Y5X5Ubc8UpmUhSfJL82zwiBFLaj";
    /// Polygon V3
    pub const POLYGON_V3: &str = "3hCPRGf4z88VC5rsBKU5AA9FBBq5nF3jbKJG7VZCbhjm";
    /// Base V3
    pub const BASE_V3: &str = "GqzP4Xaehti8KSfQmv3ZctFSjnSUYZ4En5NRsiTbvZpz";
    /// BSC V3
    pub const BSC_V3: &str = "F85MNzUGYqgSHSHRGgeVMNsdnW1KtZSVgFULumXRZTw2";

    // === V4 Subgraphs (Official Uniswap deployments) ===
    /// Ethereum Mainnet V4
    pub const MAINNET_V4: &str = "DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G";
    /// Arbitrum V4 (placeholder - check docs.uniswap.org for official ID)
    pub const ARBITRUM_V4: &str = "DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G";
    /// Base V4 (placeholder - check docs.uniswap.org for official ID)
    pub const BASE_V4: &str = "DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G";
    /// Polygon V4 (placeholder - check docs.uniswap.org for official ID)
    pub const POLYGON_V4: &str = "DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G";
}

/// The Graph gateway base URL
const GRAPH_GATEWAY: &str = "https://gateway.thegraph.com/api";

/// Configuration for the subgraph client
#[derive(Debug, Clone)]
pub struct SubgraphConfig {
    /// The Graph API key (required)
    pub api_key: String,
    /// Subgraph ID to query
    pub subgraph_id: String,
    /// Uniswap protocol version
    pub version: UniswapVersion,
    /// HTTP client configuration
    pub http: HttpClientConfig,
}

impl SubgraphConfig {
    // === V2 Configs ===

    /// Create config for Ethereum mainnet V2
    pub fn mainnet_v2(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::MAINNET_V2.to_string(),
            version: UniswapVersion::V2,
            http: HttpClientConfig::default(),
        }
    }

    // === V3 Configs ===

    /// Create config for Ethereum mainnet V3
    pub fn mainnet_v3(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::MAINNET_V3.to_string(),
            version: UniswapVersion::V3,
            http: HttpClientConfig::default(),
        }
    }

    /// Create config for Arbitrum V3
    pub fn arbitrum_v3(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::ARBITRUM_V3.to_string(),
            version: UniswapVersion::V3,
            http: HttpClientConfig::default(),
        }
    }

    /// Create config for Optimism V3
    pub fn optimism_v3(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::OPTIMISM_V3.to_string(),
            version: UniswapVersion::V3,
            http: HttpClientConfig::default(),
        }
    }

    /// Create config for Base V3
    pub fn base_v3(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::BASE_V3.to_string(),
            version: UniswapVersion::V3,
            http: HttpClientConfig::default(),
        }
    }

    // === V4 Configs ===

    /// Create config for Ethereum mainnet V4
    pub fn mainnet_v4(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::MAINNET_V4.to_string(),
            version: UniswapVersion::V4,
            http: HttpClientConfig::default(),
        }
    }

    /// Create config for Arbitrum V4
    pub fn arbitrum_v4(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::ARBITRUM_V4.to_string(),
            version: UniswapVersion::V4,
            http: HttpClientConfig::default(),
        }
    }

    /// Create config for Base V4
    pub fn base_v4(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::BASE_V4.to_string(),
            version: UniswapVersion::V4,
            http: HttpClientConfig::default(),
        }
    }

    /// Set a custom subgraph ID
    #[must_use]
    pub fn with_subgraph_id(mut self, id: impl Into<String>) -> Self {
        self.subgraph_id = id.into();
        self
    }
}

/// Client for querying Uniswap subgraphs
#[derive(Debug, Clone)]
pub struct SubgraphClient {
    http: HttpClient,
    endpoint: String,
    version: UniswapVersion,
}

impl SubgraphClient {
    /// Create a new subgraph client
    pub fn new(config: SubgraphConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(subgraph_key_required());
        }

        let http = yldfi_common::build_client(&config.http)?;
        let endpoint = format!(
            "{}/{}/subgraphs/id/{}",
            GRAPH_GATEWAY, config.api_key, config.subgraph_id
        );

        // Validate URL
        let _ = Url::parse(&endpoint)?;

        Ok(Self {
            http,
            endpoint,
            version: config.version,
        })
    }

    /// Get the Uniswap version this client is configured for
    #[must_use] 
    pub fn version(&self) -> UniswapVersion {
        self.version
    }

    /// Execute a raw GraphQL query
    pub async fn query<T: DeserializeOwned>(&self, query: &str) -> Result<T> {
        self.query_with_variables(query, serde_json::Value::Null)
            .await
    }

    /// Execute a GraphQL query with variables
    pub async fn query_with_variables<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<T> {
        let body = json!({
            "query": query,
            "variables": variables
        });

        let response = self
            .http
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| subgraph_error(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            return Err(subgraph_error(format!("HTTP {status}: {body}")));
        }

        let gql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| subgraph_error(format!("Parse error: {e}")))?;

        if let Some(err) = gql_response.first_error() {
            return Err(subgraph_error(err));
        }

        gql_response
            .data
            .ok_or_else(|| subgraph_error("No data in response"))
    }

    /// Get current ETH price in USD
    pub async fn get_eth_price(&self) -> Result<f64> {
        // V2 uses `ethPrice`, V3/V4 use `ethPriceUSD`
        let (query, field_name) = match self.version {
            UniswapVersion::V2 => (
                r"
                query {
                    bundles(first: 1) {
                        id
                        ethPrice
                    }
                }
                ",
                "ethPrice",
            ),
            UniswapVersion::V3 | UniswapVersion::V4 => (
                r"
                query {
                    bundles(first: 1) {
                        id
                        ethPriceUSD
                    }
                }
                ",
                "ethPriceUSD",
            ),
        };

        // Use a flexible response type that can handle both field names
        let data: serde_json::Value = self.query(query).await?;

        data.get("bundles")
            .and_then(|b| b.get(0))
            .and_then(|bundle| bundle.get(field_name))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| subgraph_error("Failed to get ETH price"))
    }

    /// Get top pools by TVL
    pub async fn get_top_pools(&self, limit: u32) -> Result<Vec<PoolData>> {
        #[derive(serde::Deserialize)]
        struct Response {
            pools: Vec<PoolData>,
        }

        let query = format!(
            r"
            query {{
                pools(
                    first: {limit}
                    orderBy: totalValueLockedUSD
                    orderDirection: desc
                ) {{
                    id
                    token0 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    token1 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    feeTier
                    totalValueLockedUSD
                    volumeUSD
                    feesUSD
                    txCount
                }}
            }}
        "
        );

        let data: Response = self.query(&query).await?;
        Ok(data.pools)
    }

    /// Get top V2 pairs by TVL (for V2 subgraphs)
    pub async fn get_top_pairs(&self, limit: u32) -> Result<Vec<PairDataV2>> {
        #[derive(serde::Deserialize)]
        struct Response {
            pairs: Vec<PairDataV2>,
        }

        let query = format!(
            r"
            query {{
                pairs(
                    first: {limit}
                    orderBy: reserveUSD
                    orderDirection: desc
                ) {{
                    id
                    token0 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    token1 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    reserve0
                    reserve1
                    totalSupply
                    reserveUSD
                    volumeUSD
                    txCount
                }}
            }}
        "
        );

        let data: Response = self.query(&query).await?;
        Ok(data.pairs)
    }

    /// Get top V4 pools by TVL (for V4 subgraphs)
    pub async fn get_top_pools_v4(&self, limit: u32) -> Result<Vec<PoolDataV4>> {
        #[derive(serde::Deserialize)]
        struct Response {
            pools: Vec<PoolDataV4>,
        }

        let query = format!(
            r"
            query {{
                pools(
                    first: {limit}
                    orderBy: totalValueLockedUSD
                    orderDirection: desc
                ) {{
                    id
                    token0 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    token1 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    feeTier
                    hooks
                    totalValueLockedUSD
                    volumeUSD
                    feesUSD
                    txCount
                }}
            }}
        "
        );

        let data: Response = self.query(&query).await?;
        Ok(data.pools)
    }

    /// Get pool by address
    pub async fn get_pool(&self, address: &str) -> Result<Option<PoolData>> {
        #[derive(serde::Deserialize)]
        struct Response {
            pool: Option<PoolData>,
        }

        let query = format!(
            r#"
            query {{
                pool(id: "{}") {{
                    id
                    token0 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    token1 {{
                        id
                        symbol
                        name
                        decimals
                    }}
                    feeTier
                    totalValueLockedUSD
                    volumeUSD
                    feesUSD
                    txCount
                }}
            }}
        "#,
            address.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.pool)
    }

    /// Get recent swaps for a pool
    pub async fn get_swaps(&self, pool_address: &str, limit: u32) -> Result<Vec<Swap>> {
        #[derive(serde::Deserialize)]
        struct Response {
            swaps: Vec<Swap>,
        }

        let query = format!(
            r#"
            query {{
                swaps(
                    first: {}
                    orderBy: timestamp
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    id
                    transaction {{ id }}
                    timestamp
                    pool {{ id }}
                    sender
                    recipient
                    amount0
                    amount1
                    amountUSD
                }}
            }}
        "#,
            limit,
            pool_address.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.swaps)
    }

    /// Get daily data for a pool
    pub async fn get_pool_day_data(
        &self,
        pool_address: &str,
        days: u32,
    ) -> Result<Vec<PoolDayData>> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename = "poolDayDatas")]
            pool_day_datas: Vec<PoolDayData>,
        }

        let query = format!(
            r#"
            query {{
                poolDayDatas(
                    first: {}
                    orderBy: date
                    orderDirection: desc
                    where: {{ pool: "{}" }}
                ) {{
                    date
                    pool {{ id }}
                    volumeUSD
                    tvlUSD
                    feesUSD
                    open
                    high
                    low
                    close
                }}
            }}
        "#,
            days,
            pool_address.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.pool_day_datas)
    }

    /// Get V3 LP positions for a wallet address
    pub async fn get_positions(&self, owner: &str) -> Result<Vec<Position>> {
        #[derive(serde::Deserialize)]
        struct Response {
            positions: Vec<Position>,
        }

        let query = format!(
            r#"
            query {{
                positions(
                    first: 100
                    where: {{ owner: "{}", liquidity_gt: "0" }}
                ) {{
                    id
                    owner
                    pool {{
                        id
                        token0 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        token1 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        feeTier
                        tick
                        sqrtPrice
                        token0Price
                        token1Price
                    }}
                    liquidity
                    depositedToken0
                    depositedToken1
                    withdrawnToken0
                    withdrawnToken1
                    collectedFeesToken0
                    collectedFeesToken1
                    tickLower {{
                        tickIdx
                    }}
                    tickUpper {{
                        tickIdx
                    }}
                }}
            }}
        "#,
            owner.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.positions)
    }

    /// Get V2 LP positions for a wallet address (ERC-20 LP tokens)
    pub async fn get_positions_v2(&self, owner: &str) -> Result<Vec<LiquidityPositionV2>> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename = "liquidityPositions")]
            liquidity_positions: Vec<LiquidityPositionV2>,
        }

        let query = format!(
            r#"
            query {{
                liquidityPositions(
                    first: 100
                    where: {{ user: "{}", liquidityTokenBalance_gt: "0" }}
                ) {{
                    id
                    user {{
                        id
                    }}
                    pair {{
                        id
                        token0 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        token1 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        reserve0
                        reserve1
                        totalSupply
                        reserveUSD
                    }}
                    liquidityTokenBalance
                }}
            }}
        "#,
            owner.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.liquidity_positions)
    }

    /// Get V4 LP positions for a wallet address
    pub async fn get_positions_v4(&self, owner: &str) -> Result<Vec<PositionV4>> {
        #[derive(serde::Deserialize)]
        struct Response {
            positions: Vec<PositionV4>,
        }

        let query = format!(
            r#"
            query {{
                positions(
                    first: 100
                    where: {{ owner: "{}", liquidity_gt: "0" }}
                ) {{
                    id
                    owner
                    pool {{
                        id
                        token0 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        token1 {{
                            id
                            symbol
                            name
                            decimals
                        }}
                        fee
                        hooks
                        totalValueLockedUSD
                    }}
                    liquidity
                    tickLower
                    tickUpper
                }}
            }}
        "#,
            owner.to_lowercase()
        );

        let data: Response = self.query(&query).await?;
        Ok(data.positions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = SubgraphConfig::mainnet_v3("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.subgraph_id, subgraph_ids::MAINNET_V3);
    }

    #[test]
    fn test_empty_api_key_fails() {
        let config = SubgraphConfig::mainnet_v3("");
        let result = SubgraphClient::new(config);
        assert!(result.is_err());
    }
}
