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
use serde_json::{json, Value};
use url::Url;
use yldfi_common::api::{sanitize_error_body, SecretApiKey};
use yldfi_common::http::HttpClientConfig;

use crate::error::{subgraph_error, subgraph_key_required, Result};
use crate::types::{
    GraphQLResponse, LiquidityPositionV2, PairDataV2, PairDayDataV2, PoolData, PoolDataV4,
    PoolDayData, Position, PositionV4, Swap,
};

/// Subgraph IDs for Uniswap on various chains
pub mod subgraph_ids {
    // === V2 Subgraphs ===
    /// Ethereum Mainnet V2 (as listed in Uniswap's docs,
    /// `content/ecosystem/subgraphs/overview.mdx`)
    pub const MAINNET_V2: &str = "A3Np3RQbaBA6oKJgiwDJeo5T3zrYfGHPWFYayMwtNDum";

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

    // === V4 Subgraphs ===
    //
    // Uniswap's docs list only the mainnet V4 subgraph. The non-mainnet IDs
    // below are published on The Graph Network by the same publisher account
    // (0x29ff57f9730fe51cf107fa3e6348250596100662) as the docs-listed mainnet
    // deployment, and were checked to index the matching network.

    /// Ethereum Mainnet V4 (`uniswap-v4-ethereum`, listed in Uniswap's docs)
    pub const MAINNET_V4: &str = "DiYPVdygkfjDWhbxGSqAQxwBKmfKnkWQojqeM2rkLb3G";
    /// Arbitrum One V4 (`uniswap-v4-arbitrum`)
    pub const ARBITRUM_V4: &str = "G5TsTKNi8yhPSV7kycaE23oWbqv9zzNqR49FoEQjzq1r";
    /// Base V4 (`uniswap-v4-base`)
    pub const BASE_V4: &str = "HNCFA9TyBqpo5qpe6QreQABAA1kV8g46mhkCcicu6v2R";
    /// Polygon PoS V4 (`uniswap-v4-polygon`)
    pub const POLYGON_V4: &str = "CwpebM66AH5uqS5sreKij8yEkkPcHvmyEs7EwFtdM5ND";
}

/// The Graph gateway base URL
const GRAPH_GATEWAY: &str = "https://gateway.thegraph.com/api";

/// Configuration for the subgraph client
///
/// `Debug` output redacts the API key.
#[derive(Clone)]
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

impl std::fmt::Debug for SubgraphConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphConfig")
            .field("api_key", &"[REDACTED]")
            .field("subgraph_id", &self.subgraph_id)
            .field("version", &self.version)
            .field("http", &self.http)
            .finish()
    }
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

    /// Create config for Polygon V4
    pub fn polygon_v4(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            subgraph_id: subgraph_ids::POLYGON_V4.to_string(),
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
///
/// The Graph gateway takes the API key as a URL path segment, so the stored
/// endpoint contains it. `Debug` output and errors redact it.
#[derive(Clone)]
pub struct SubgraphClient {
    http: HttpClient,
    endpoint: String,
    api_key: SecretApiKey,
    version: UniswapVersion,
}

impl std::fmt::Debug for SubgraphClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubgraphClient")
            .field("endpoint", &self.redact(&self.endpoint))
            .field("version", &self.version)
            .finish_non_exhaustive()
    }
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
            api_key: SecretApiKey::new(config.api_key),
            version: config.version,
        })
    }

    /// Client pointed at an arbitrary endpoint (for tests against a mock server)
    #[cfg(test)]
    pub(crate) fn for_test_endpoint(endpoint: &str, version: UniswapVersion) -> Self {
        Self {
            http: HttpClient::new(),
            endpoint: endpoint.to_string(),
            api_key: SecretApiKey::new("test-key"),
            version,
        }
    }

    /// Replace any occurrence of the API key in `text`
    fn redact(&self, text: &str) -> String {
        text.replace(self.api_key.expose(), "[REDACTED]")
    }

    /// Error message for a reqwest error, without the request URL (which
    /// contains the API key)
    fn transport_error(&self, e: reqwest::Error) -> String {
        self.redact(&e.without_url().to_string())
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
            .map_err(|e| subgraph_error(self.transport_error(e)))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let body = response.text().await.unwrap_or_default();
            let body = sanitize_error_body(&self.redact(&body));
            return Err(subgraph_error(format!("HTTP {status}: {body}")));
        }

        let gql_response: GraphQLResponse<T> = response
            .json()
            .await
            .map_err(|e| subgraph_error(format!("Parse error: {}", self.transport_error(e))))?;

        if let Some(err) = gql_response.first_error() {
            return Err(subgraph_error(self.redact(err)));
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

        let query = r"
            query Pool($id: ID!) {
                pool(id: $id) {
                    id
                    token0 {
                        id
                        symbol
                        name
                        decimals
                    }
                    token1 {
                        id
                        symbol
                        name
                        decimals
                    }
                    feeTier
                    totalValueLockedUSD
                    volumeUSD
                    feesUSD
                    txCount
                }
            }
        ";
        let variables = serde_json::json!({ "id": address.to_lowercase() });

        let data: Response = self.query_with_variables(query, variables).await?;
        Ok(data.pool)
    }

    /// Get recent swaps for a pool
    ///
    /// The query is version-aware: V2 swaps are keyed by `pair` and report
    /// in/out amounts (mapped to signed `amount0`/`amount1` from the pool's
    /// perspective); V4 swaps have no `recipient` field and report `origin`
    /// (the EOA that initiated the transaction) instead.
    pub async fn get_swaps(&self, pool_address: &str, limit: u32) -> Result<Vec<Swap>> {
        #[derive(serde::Deserialize)]
        struct Response {
            swaps: Vec<Swap>,
        }

        let (query, variables) = swaps_query(self.version, pool_address, limit);
        let data: Response = self.query_with_variables(&query, variables).await?;
        Ok(data.swaps)
    }

    /// Get daily data for a pool, newest first
    ///
    /// V3/V4 query `poolDayDatas`; V2 queries `pairDayDatas` (no OHLC, fees
    /// derived as 0.3% of volume).
    pub async fn get_pool_day_data(
        &self,
        pool_address: &str,
        days: u32,
    ) -> Result<Vec<PoolDayData>> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename = "poolDayDatas", default)]
            pool_day_datas: Vec<PoolDayData>,
            #[serde(rename = "pairDayDatas", default)]
            pair_day_datas: Vec<PairDayDataV2>,
        }

        let (query, variables) = pool_day_data_query(self.version, pool_address, days);
        let data: Response = self.query_with_variables(&query, variables).await?;
        Ok(data
            .pool_day_datas
            .into_iter()
            .chain(data.pair_day_datas.into_iter().map(PoolDayData::from))
            .collect())
    }

    /// Get V3 LP positions for a wallet address
    pub async fn get_positions(&self, owner: &str) -> Result<Vec<Position>> {
        #[derive(serde::Deserialize)]
        struct Response {
            positions: Vec<Position>,
        }

        let query = r"
            query Positions($where: Position_filter) {
                positions(
                    first: 100
                    where: $where
                ) {
                    id
                    owner
                    pool {
                        id
                        token0 {
                            id
                            symbol
                            name
                            decimals
                        }
                        token1 {
                            id
                            symbol
                            name
                            decimals
                        }
                        feeTier
                        tick
                        sqrtPrice
                        token0Price
                        token1Price
                    }
                    liquidity
                    depositedToken0
                    depositedToken1
                    withdrawnToken0
                    withdrawnToken1
                    collectedFeesToken0
                    collectedFeesToken1
                    tickLower {
                        tickIdx
                    }
                    tickUpper {
                        tickIdx
                    }
                }
            }
        ";
        let variables = serde_json::json!({
            "where": { "owner": owner.to_lowercase(), "liquidity_gt": "0" }
        });

        let data: Response = self.query_with_variables(query, variables).await?;
        Ok(data.positions)
    }

    /// Get V2 LP positions for a wallet address (ERC-20 LP tokens)
    pub async fn get_positions_v2(&self, owner: &str) -> Result<Vec<LiquidityPositionV2>> {
        #[derive(serde::Deserialize)]
        struct Response {
            #[serde(rename = "liquidityPositions")]
            liquidity_positions: Vec<LiquidityPositionV2>,
        }

        let query = r"
            query LiquidityPositions($where: LiquidityPosition_filter) {
                liquidityPositions(
                    first: 100
                    where: $where
                ) {
                    id
                    user {
                        id
                    }
                    pair {
                        id
                        token0 {
                            id
                            symbol
                            name
                            decimals
                        }
                        token1 {
                            id
                            symbol
                            name
                            decimals
                        }
                        reserve0
                        reserve1
                        totalSupply
                        reserveUSD
                    }
                    liquidityTokenBalance
                }
            }
        ";
        let variables = serde_json::json!({
            "where": { "user": owner.to_lowercase(), "liquidityTokenBalance_gt": "0" }
        });

        let data: Response = self.query_with_variables(query, variables).await?;
        Ok(data.liquidity_positions)
    }

    /// Get V4 LP positions (position NFTs) owned by a wallet address
    ///
    /// The V4 subgraph `Position` entity only tracks ownership
    /// (`id`, `tokenId`, `owner`, `origin`, `createdAtTimestamp`); pool and
    /// liquidity details live on-chain in the `PositionManager`.
    pub async fn get_positions_v4(&self, owner: &str) -> Result<Vec<PositionV4>> {
        #[derive(serde::Deserialize)]
        struct Response {
            positions: Vec<PositionV4>,
        }

        let (query, variables) = positions_v4_query(owner);
        let data: Response = self.query_with_variables(query, variables).await?;
        Ok(data.positions)
    }
}

/// Build the swaps query and variables for a subgraph version
///
/// The pool/pair address is sent as a GraphQL variable (`Swap_filter`),
/// never interpolated into the query text.
/// Version-aware daily data query: `pairDayDatas` for V2, `poolDayDatas`
/// otherwise
fn pool_day_data_query(version: UniswapVersion, pool_address: &str, days: u32) -> (String, Value) {
    let pool = pool_address.to_lowercase();
    let (entity, filter_type, filter, fields) = match version {
        UniswapVersion::V2 => (
            "pairDayDatas",
            "PairDayData_filter",
            serde_json::json!({ "pairAddress": pool }),
            "date pairAddress dailyVolumeUSD reserveUSD",
        ),
        UniswapVersion::V3 | UniswapVersion::V4 => (
            "poolDayDatas",
            "PoolDayData_filter",
            serde_json::json!({ "pool": pool }),
            "date pool { id } volumeUSD tvlUSD feesUSD open high low close",
        ),
    };
    let query = format!(
        "query DayData($where: {filter_type}) {{ {entity}(first: {days}, orderBy: date, \
         orderDirection: desc, where: $where) {{ {fields} }} }}"
    );
    (query, serde_json::json!({ "where": filter }))
}

fn swaps_query(version: UniswapVersion, pool_address: &str, limit: u32) -> (String, Value) {
    let pool = pool_address.to_lowercase();
    let (filter, fields) = match version {
        UniswapVersion::V2 => (
            serde_json::json!({ "pair": pool }),
            "id transaction { id } timestamp pair { id } sender to from \
             amount0In amount1In amount0Out amount1Out amountUSD",
        ),
        UniswapVersion::V3 => (
            serde_json::json!({ "pool": pool }),
            "id transaction { id } timestamp pool { id } sender recipient origin \
             amount0 amount1 amountUSD",
        ),
        UniswapVersion::V4 => (
            serde_json::json!({ "pool": pool }),
            "id transaction { id } timestamp pool { id } sender origin \
             amount0 amount1 amountUSD",
        ),
    };
    let query = format!(
        "query Swaps($where: Swap_filter) {{ swaps(first: {limit}, orderBy: timestamp, \
         orderDirection: desc, where: $where) {{ {fields} }} }}"
    );
    (query, serde_json::json!({ "where": filter }))
}

/// V4 positions query (fields per Uniswap/v4-subgraph schema.graphql)
const POSITIONS_V4_QUERY: &str = "query PositionsV4($where: Position_filter) { \
    positions(first: 100, orderBy: createdAtTimestamp, orderDirection: desc, where: $where) \
    { id tokenId owner origin createdAtTimestamp } }";

/// Build the V4 positions query and variables
fn positions_v4_query(owner: &str) -> (&'static str, Value) {
    (
        POSITIONS_V4_QUERY,
        serde_json::json!({ "where": { "owner": owner.to_lowercase() } }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_data_query_is_version_aware() {
        let (v2, vars) = pool_day_data_query(UniswapVersion::V2, "0xABC", 7);
        assert!(v2.contains("pairDayDatas(first: 7") && v2.contains("PairDayData_filter"));
        assert!(v2.contains("dailyVolumeUSD") && !v2.contains("poolDayDatas"));
        assert_eq!(
            vars,
            serde_json::json!({ "where": { "pairAddress": "0xabc" } })
        );
        let (v3, vars) = pool_day_data_query(UniswapVersion::V3, "0xABC", 7);
        assert!(v3.contains("poolDayDatas(first: 7") && v3.contains("feesUSD"));
        assert_eq!(vars, serde_json::json!({ "where": { "pool": "0xabc" } }));
    }

    const KEY: &str = "graph-secret-key-789";

    /// A client whose endpoint embeds `KEY` like the real gateway URL, but
    /// points at `base`.
    fn client_at(base: &str) -> SubgraphClient {
        SubgraphClient {
            http: HttpClient::new(),
            endpoint: format!("{base}/{KEY}/subgraphs/id/test"),
            api_key: SecretApiKey::new(KEY),
            version: UniswapVersion::V3,
        }
    }

    #[test]
    fn test_debug_redacts_api_key() {
        let config = SubgraphConfig::mainnet_v3(KEY);
        let client = SubgraphClient::new(config.clone()).unwrap();
        for debug in [format!("{config:?}"), format!("{client:?}")] {
            assert!(!debug.contains(KEY), "{debug}");
            assert!(debug.contains("[REDACTED]"), "{debug}");
        }
    }

    #[tokio::test]
    async fn test_transport_error_omits_api_key() {
        // Nothing listens on port 1, so this fails with a connect error whose
        // Display would include the URL
        let client = client_at("http://127.0.0.1:1");
        let err = client
            .query::<serde_json::Value>("{ x }")
            .await
            .unwrap_err();
        assert!(!err.to_string().contains(KEY), "{err}");
        assert!(!format!("{err:?}").contains(KEY), "{err:?}");
    }

    #[tokio::test]
    async fn test_http_and_parse_errors_omit_api_key() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let client = client_at(&server.uri());

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(500).set_body_string(format!("no such path /{KEY}/x")),
            )
            .up_to_n_times(1)
            .mount(&server)
            .await;
        let err = client
            .query::<serde_json::Value>("{ x }")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("HTTP 500"), "{err}");
        assert!(!err.to_string().contains(KEY), "{err}");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;
        let err = client
            .query::<serde_json::Value>("{ x }")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Parse error"), "{err}");
        assert!(!err.to_string().contains(KEY), "{err}");
    }

    #[test]
    fn test_config_creation() {
        let config = SubgraphConfig::mainnet_v3("test-key");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.subgraph_id, subgraph_ids::MAINNET_V3);
    }

    #[test]
    fn test_v4_queries_use_v4_schema_fields() {
        let (q, vars) = swaps_query(UniswapVersion::V4, "0xABC", 5);
        assert!(!q.contains("recipient"), "{q}");
        assert!(q.contains("origin"));
        assert!(q.contains("where: $where") && !q.contains("0xabc"), "{q}");
        assert_eq!(vars, serde_json::json!({ "where": { "pool": "0xabc" } }));

        let (p, vars) = positions_v4_query("0xD8dA");
        for bad in ["liquidity", "tickLower", "pool {", "liquidity_gt"] {
            assert!(!p.contains(bad), "V4 Position has no field {bad}: {p}");
        }
        assert!(p.contains("tokenId") && p.contains("$where: Position_filter"));
        assert_eq!(vars, serde_json::json!({ "where": { "owner": "0xd8da" } }));
    }

    #[test]
    fn test_swap_deserializes_all_versions() {
        let v4: Swap = serde_json::from_str(
            r#"{"id":"1","transaction":{"id":"0xt"},"timestamp":"1","pool":{"id":"0xp"},
            "sender":"0xs","origin":"0xo","amount0":"-1.5","amount1":"4000","amountUSD":"4000"}"#,
        )
        .unwrap();
        assert_eq!(v4.recipient, None);
        assert_eq!(v4.origin.as_deref(), Some("0xo"));

        let v2: Swap = serde_json::from_str(
            r#"{"id":"2","transaction":{"id":"0xt"},"timestamp":"1","pair":{"id":"0xpair"},
            "sender":"0xs","to":"0xr","from":"0xf","amount0In":"0","amount1In":"2",
            "amount0Out":"5000","amount1Out":"0","amountUSD":"5000"}"#,
        )
        .unwrap();
        assert_eq!(v2.pool.id, "0xpair");
        assert_eq!(v2.recipient.as_deref(), Some("0xr"));
        assert_eq!(v2.amount0, "-5000");
        assert_eq!(v2.amount1, "2");

        let v4_pos: PositionV4 = serde_json::from_str(
            r#"{"id":"123","tokenId":"123","owner":"0xo","origin":"0xo","createdAtTimestamp":"1700000000"}"#,
        )
        .unwrap();
        assert_eq!(v4_pos.token_id, "123");
    }

    #[test]
    fn test_v2_swaps_query_uses_pair() {
        let (q, vars) = swaps_query(UniswapVersion::V2, "0xB4e1\" } }", 3);
        assert!(q.contains("$where: Swap_filter") && q.contains("first: 3"));
        assert!(q.contains("amount0In") && !q.contains("recipient"));
        assert!(!q.contains("0xb4e1"), "{q}");
        assert_eq!(
            vars,
            serde_json::json!({ "where": { "pair": "0xb4e1\" } }" } })
        );
    }

    #[test]
    fn test_v4_chain_ids_are_distinct() {
        // Non-mainnet V4 configs must not silently fall back to Ethereum data.
        let ids = [
            subgraph_ids::MAINNET_V4,
            subgraph_ids::ARBITRUM_V4,
            subgraph_ids::BASE_V4,
            subgraph_ids::POLYGON_V4,
        ];
        for (i, a) in ids.iter().enumerate() {
            for b in &ids[i + 1..] {
                assert_ne!(a, b);
            }
        }
        let polygon = SubgraphConfig::polygon_v4("k");
        assert_eq!(polygon.subgraph_id, subgraph_ids::POLYGON_V4);
        assert_eq!(polygon.version, UniswapVersion::V4);
    }

    #[test]
    fn test_empty_api_key_fails() {
        let config = SubgraphConfig::mainnet_v3("");
        let result = SubgraphClient::new(config);
        assert!(result.is_err());
    }
}
