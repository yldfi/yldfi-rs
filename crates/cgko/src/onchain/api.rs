//! Onchain/GeckoTerminal API endpoints

use super::types::{
    DexesResponse, HoldersChartResponse, LiquidityEventsOptions, LiquidityEventsResponse,
    MegafilterOptions, NetworkTokensResponse, NetworksResponse, OhlcvOptions, OhlcvResponse,
    OnchainCategoriesResponse, PoolInfoResponse, PoolResponse, PoolsResponse,
    SimpleTokenPriceResponse, TokenHoldersResponse, TokenInfoResponse, TokenOhlcvResponse,
    TokenPriceOptions, TokenPriceResponse, TokenResponse, TokenTradersResponse,
    TokenTradesResponse, TokensMultiNetworkOptions, TokensResponse, TradesOptions, TradesResponse,
};
use crate::client::Client;
use crate::error::Result;
use crate::query::Query;

/// Onchain (`GeckoTerminal`) API
pub struct OnchainApi<'a> {
    client: &'a Client,
}

impl<'a> OnchainApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List supported networks
    pub async fn networks(&self) -> Result<NetworksResponse> {
        self.client.get("/onchain/networks").await
    }

    /// List DEXes on a network
    pub async fn dexes(&self, network: &str) -> Result<DexesResponse> {
        let path = format!("/onchain/networks/{network}/dexes");
        self.client.get(&path).await
    }

    /// Get trending pools across all networks
    pub async fn trending_pools(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/networks/trending_pools").await
    }

    /// Get trending pools on a network
    pub async fn trending_pools_network(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/trending_pools");
        self.client.get(&path).await
    }

    /// Get top pools on a network
    pub async fn top_pools(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/pools");
        self.client.get(&path).await
    }

    /// Get pool data
    pub async fn pool(&self, network: &str, address: &str) -> Result<PoolResponse> {
        let path = format!("/onchain/networks/{network}/pools/{address}");
        self.client.get(&path).await
    }

    /// Get new pools on a network
    pub async fn new_pools(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/new_pools");
        self.client.get(&path).await
    }

    /// Get token data
    pub async fn token(&self, network: &str, address: &str) -> Result<TokenResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{address}");
        self.client.get(&path).await
    }

    /// Get token price
    pub async fn token_price(
        &self,
        network: &str,
        addresses: &[&str],
    ) -> Result<TokenPriceResponse> {
        let path = format!(
            "/onchain/simple/networks/{}/token_price/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get token prices on one network, with optional market cap, 24h
    /// volume, 24h price change and total reserve
    ///
    /// The response maps are keyed by token address.
    pub async fn token_price_with_options(
        &self,
        network: &str,
        addresses: &[&str],
        options: &TokenPriceOptions,
    ) -> Result<SimpleTokenPriceResponse> {
        let path = format!(
            "/onchain/simple/networks/{}/token_price/{}{}",
            network,
            addresses.join(","),
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get token prices across networks in one request (Enterprise plan
    /// only)
    ///
    /// # Arguments
    /// * `tokens` - Up to 50 `network_id:token_address` pairs, e.g.
    ///   `eth:0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2`; networks can be
    ///   mixed. The response maps are keyed by the same pairs.
    ///
    /// An unknown network fails the whole request with `400`; a token
    /// address that cannot be resolved is left out of the response.
    pub async fn token_price_multi_network(
        &self,
        tokens: &[&str],
        options: &TokenPriceOptions,
    ) -> Result<SimpleTokenPriceResponse> {
        let tokens = to_strings(tokens);
        let query = options
            .add_to(Query::new().list("tokens", Some(&tokens)))
            .finish();
        let path = format!("/onchain/simple/token_price/multi{query}");
        self.client.get(&path).await
    }

    /// Get token data across networks in one request (Enterprise plan only)
    ///
    /// # Arguments
    /// * `tokens` - Up to 50 `network_id:token_address` pairs; networks can
    ///   be mixed. Each token's `attributes.network` names its network.
    ///
    /// An unknown network fails the whole request with `400`; a token
    /// address that cannot be resolved is left out of the response.
    pub async fn tokens_multi_network(
        &self,
        tokens: &[&str],
        options: &TokensMultiNetworkOptions,
    ) -> Result<NetworkTokensResponse> {
        let tokens = to_strings(tokens);
        let query = options
            .add_to(Query::new().list("tokens", Some(&tokens)))
            .finish();
        let path = format!("/onchain/tokens/multi{query}");
        self.client.get(&path).await
    }

    /// Get pools for a token
    pub async fn token_pools(&self, network: &str, token_address: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/pools");
        self.client.get(&path).await
    }

    /// Get pool OHLCV data
    ///
    /// # Arguments
    /// * `network` - Network ID
    /// * `pool_address` - Pool address
    /// * `timeframe` - "minute", "hour", or "day"
    pub async fn pool_ohlcv(
        &self,
        network: &str,
        pool_address: &str,
        timeframe: &str,
    ) -> Result<OhlcvResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/ohlcv/{timeframe}");
        self.client.get(&path).await
    }

    /// Get pool OHLCV with options (candle size, count up to 1000, end
    /// time, currency, inverted chart, empty intervals)
    ///
    /// # Arguments
    /// * `timeframe` - `day`, `hour`, `minute` or `second`
    pub async fn pool_ohlcv_with_options(
        &self,
        network: &str,
        pool_address: &str,
        timeframe: &str,
        options: &OhlcvOptions,
    ) -> Result<OhlcvResponse> {
        let options = OhlcvOptions {
            include_inactive_source: None,
            ..options.clone()
        };
        let path = format!(
            "/onchain/networks/{network}/pools/{pool_address}/ohlcv/{timeframe}{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get pool trades
    ///
    /// Since 2026-10-20 `CoinGecko` leaves out trades it flags as anomalous;
    /// use [`Self::pool_trades_with_anomalous`] to include them.
    pub async fn pool_trades(&self, network: &str, pool_address: &str) -> Result<TradesResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/trades");
        self.client.get(&path).await
    }

    /// Get pool trades, choosing whether to include trades `CoinGecko`
    /// flags as anomalous (`include_anomalous`; each trade then carries
    /// `is_anomalous`)
    pub async fn pool_trades_with_anomalous(
        &self,
        network: &str,
        pool_address: &str,
        include_anomalous: bool,
    ) -> Result<TradesResponse> {
        let path = format!(
            "/onchain/networks/{network}/pools/{pool_address}/trades?include_anomalous={include_anomalous}"
        );
        self.client.get(&path).await
    }

    /// Get pool trades with options: paging (pass `meta.next_cursor` back
    /// via [`TradesOptions::cursor`]), lookback period, minimum USD volume,
    /// token side and anomalous trades. Paging and the lookback period need
    /// the Analyst plan or above.
    pub async fn pool_trades_with_options(
        &self,
        network: &str,
        pool_address: &str,
        options: &TradesOptions,
    ) -> Result<TradesResponse> {
        let path = format!(
            "/onchain/networks/{network}/pools/{pool_address}/trades{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get pool trades between two times (Analyst plan and above)
    ///
    /// # Arguments
    /// * `from`, `to` - Window ends, both inclusive, as an ISO date
    ///   (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or a Unix timestamp; at most 30
    ///   days apart and not in the future
    ///
    /// Page with [`TradesOptions::cursor`] and `meta.next_cursor`.
    /// [`TradesOptions::trading_period`] does not apply here and is not sent.
    pub async fn pool_trades_range(
        &self,
        network: &str,
        pool_address: &str,
        from: &str,
        to: &str,
        options: &TradesOptions,
    ) -> Result<TradesResponse> {
        let options = TradesOptions {
            trading_period: None,
            ..options.clone()
        };
        let path = format!(
            "/onchain/networks/{network}/pools/{pool_address}/trades/range{}",
            range_query(from, to, &options)
        );
        self.client.get(&path).await
    }

    /// Get add and remove liquidity events for a pool (Analyst plan and
    /// above)
    ///
    /// Paginate by passing `meta.next_cursor` back as the cursor option.
    /// `CoinGecko` currently supports only some Solana DEXes; other pools
    /// return `400`.
    pub async fn pool_liquidity_events(
        &self,
        network: &str,
        pool_address: &str,
        options: &LiquidityEventsOptions,
    ) -> Result<LiquidityEventsResponse> {
        let path = format!(
            "/onchain/networks/{network}/pools/{pool_address}/liquidity_events{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Search pools
    pub async fn search_pools(&self, query: &str) -> Result<PoolsResponse> {
        self.client.get(&search_pools_path(query)).await
    }

    /// Get new pools across all networks
    pub async fn new_pools_all(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/networks/new_pools").await
    }

    /// Get top pools across all networks
    ///
    /// CoinGecko has no `/onchain/networks/pools` endpoint (it returns 404).
    /// This now redirects to [`Self::megafilter`] sorted by 24h USD volume,
    /// which requires an Analyst plan or above. Use [`Self::top_pools`] for
    /// a single network on any plan.
    #[deprecated(
        since = "0.1.5",
        note = "`/onchain/networks/pools` does not exist; use `top_pools(network)` or `megafilter(&MegafilterOptions::new().with_sort(\"h24_volume_usd_desc\"))`"
    )]
    pub async fn top_pools_all(&self) -> Result<PoolsResponse> {
        self.megafilter(&MegafilterOptions::new().with_sort("h24_volume_usd_desc"))
            .await
    }

    /// Get multiple pools by addresses
    pub async fn pools_multi(&self, network: &str, addresses: &[&str]) -> Result<PoolsResponse> {
        let path = format!(
            "/onchain/networks/{}/pools/multi/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get multiple tokens
    pub async fn tokens_multi(&self, network: &str, addresses: &[&str]) -> Result<TokensResponse> {
        let path = format!(
            "/onchain/networks/{}/tokens/multi/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get token info (detailed)
    pub async fn token_info(&self, network: &str, address: &str) -> Result<TokenInfoResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{address}/info");
        self.client.get(&path).await
    }

    /// Get pools for a specific DEX on a network
    pub async fn dex_pools(&self, network: &str, dex: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/dexes/{dex}/pools");
        self.client.get(&path).await
    }

    /// Get most recently updated tokens across all networks
    pub async fn recently_updated_tokens(&self) -> Result<TokensResponse> {
        self.client
            .get("/onchain/tokens/info_recently_updated")
            .await
    }

    /// Get pool token metadata
    pub async fn pool_info(&self, network: &str, pool_address: &str) -> Result<PoolInfoResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/info");
        self.client.get(&path).await
    }

    /// Get top token holders (Pro API only)
    pub async fn token_holders(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenHoldersResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/top_holders");
        self.client.get(&path).await
    }

    /// Get top token traders (Pro API only)
    pub async fn token_traders(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenTradersResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/top_traders");
        self.client.get(&path).await
    }

    /// Get historical token holders chart (Pro API only)
    pub async fn token_holders_chart(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<HoldersChartResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/holders_chart");
        self.client.get(&path).await
    }

    /// Get token OHLCV data (Pro API only)
    ///
    /// # Arguments
    /// * `network` - Network ID
    /// * `token_address` - Token address
    /// * `timeframe` - "minute", "hour", or "day"
    pub async fn token_ohlcv(
        &self,
        network: &str,
        token_address: &str,
        timeframe: &str,
    ) -> Result<TokenOhlcvResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/ohlcv/{timeframe}");
        self.client.get(&path).await
    }

    /// Get token OHLCV with options (candle size, count up to 1000, end
    /// time, currency, empty intervals, inactive pools)
    ///
    /// # Arguments
    /// * `timeframe` - `day`, `hour`, `minute` or `second`
    pub async fn token_ohlcv_with_options(
        &self,
        network: &str,
        token_address: &str,
        timeframe: &str,
        options: &OhlcvOptions,
    ) -> Result<TokenOhlcvResponse> {
        let options = OhlcvOptions {
            token: None,
            ..options.clone()
        };
        let path = format!(
            "/onchain/networks/{network}/tokens/{token_address}/ohlcv/{timeframe}{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get token trades across all pools (Pro API only)
    ///
    /// Since 2026-10-20 `CoinGecko` leaves out trades it flags as anomalous;
    /// use [`Self::token_trades_with_anomalous`] to include them.
    pub async fn token_trades(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenTradesResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/trades");
        self.client.get(&path).await
    }

    /// Get token trades across all pools, choosing whether to include
    /// trades `CoinGecko` flags as anomalous (Pro API only)
    pub async fn token_trades_with_anomalous(
        &self,
        network: &str,
        token_address: &str,
        include_anomalous: bool,
    ) -> Result<TokenTradesResponse> {
        let path = format!(
            "/onchain/networks/{network}/tokens/{token_address}/trades?include_anomalous={include_anomalous}"
        );
        self.client.get(&path).await
    }

    /// Get token trades across all pools with options (Pro API only):
    /// paging (pass `meta.next_cursor` back via [`TradesOptions::cursor`]),
    /// lookback period, minimum USD volume and anomalous trades.
    /// [`TradesOptions::token`] does not apply here and is not sent.
    pub async fn token_trades_with_options(
        &self,
        network: &str,
        token_address: &str,
        options: &TradesOptions,
    ) -> Result<TokenTradesResponse> {
        let options = TradesOptions {
            token: None,
            ..options.clone()
        };
        let path = format!(
            "/onchain/networks/{network}/tokens/{token_address}/trades{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get token trades across all pools between two times (Analyst plan
    /// and above)
    ///
    /// # Arguments
    /// * `from`, `to` - Window ends, both inclusive, as an ISO date
    ///   (`YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`) or a Unix timestamp; at most 30
    ///   days apart and not in the future
    ///
    /// Page with [`TradesOptions::cursor`] and `meta.next_cursor`.
    /// [`TradesOptions::token`] and [`TradesOptions::trading_period`] do not
    /// apply here and are not sent.
    pub async fn token_trades_range(
        &self,
        network: &str,
        token_address: &str,
        from: &str,
        to: &str,
        options: &TradesOptions,
    ) -> Result<TokenTradesResponse> {
        let options = TradesOptions {
            token: None,
            trading_period: None,
            ..options.clone()
        };
        let path = format!(
            "/onchain/networks/{network}/tokens/{token_address}/trades/range{}",
            range_query(from, to, &options)
        );
        self.client.get(&path).await
    }

    /// Advanced pool filtering with megafilter (Pro API only)
    pub async fn megafilter(&self, options: &MegafilterOptions) -> Result<PoolsResponse> {
        let path = format!("/onchain/pools/megafilter{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get trending search pools (Pro API only)
    pub async fn trending_search_pools(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/pools/trending_search").await
    }

    /// Get `GeckoTerminal` categories (Pro API only)
    pub async fn categories(&self) -> Result<OnchainCategoriesResponse> {
        self.client.get("/onchain/categories").await
    }

    /// Get pools by category (Pro API only)
    pub async fn category_pools(&self, category_id: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/categories/{category_id}/pools");
        self.client.get(&path).await
    }
}

fn to_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_string()).collect()
}

/// `?from=..&to=..` followed by the trade options
fn range_query(from: &str, to: &str, options: &TradesOptions) -> String {
    options
        .add_to(Query::new().text("from", Some(from)).text("to", Some(to)))
        .finish()
}

fn search_pools_path(query: &str) -> String {
    format!(
        "/onchain/search/pools?query={}",
        Client::encode_query(query)
    )
}

#[cfg(test)]
mod search_tests {
    use super::*;

    #[test]
    fn search_pools_query_is_url_encoded() {
        assert_eq!(
            search_pools_path("weth/usdc #1"),
            "/onchain/search/pools?query=weth%2Fusdc+%231"
        );
    }
}

#[cfg(test)]
mod mock_tests {
    use super::*;
    use crate::mock::Mock;

    const WETH: &str = "eth:0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

    #[tokio::test]
    async fn token_price_multi_network() {
        let mock = Mock::new(
            r#"{"data":{"id":"1965b339","type":"simple_token_price","attributes":{
                "token_prices":{"eth:0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2":"2472.55",
                    "solana:6Nwa":"0.00725052986758138039267632966543134822451730159307662487"},
                "market_cap_usd":{"eth:0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2":"5102178003.8",
                    "solana:6Nwa":null},
                "last_trade_timestamp":{"solana:6Nwa":"1789028068"}}}}"#,
        );
        let options = TokenPriceOptions::new()
            .include_market_cap(true)
            .mcap_fdv_fallback(false)
            .include_24hr_vol(true)
            .include_24hr_price_change(true)
            .include_total_reserve_in_usd(true)
            .include_inactive_source(true);
        let r = mock
            .client
            .onchain()
            .token_price_multi_network(&[WETH, "solana:6Nwa"], &options)
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/onchain/simple/token_price/multi?tokens=eth%3A0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2,solana%3A6Nwa\
             &include_market_cap=true&mcap_fdv_fallback=false&include_24hr_vol=true\
             &include_24hr_price_change=true&include_total_reserve_in_usd=true&include_inactive_source=true"
        );
        let a = r.data.attributes.unwrap();
        // Full precision: prices stay strings
        assert_eq!(
            a.token_prices.unwrap()["solana:6Nwa"].as_deref(),
            Some("0.00725052986758138039267632966543134822451730159307662487")
        );
        assert!(a.market_cap_usd.unwrap()["solana:6Nwa"].is_none());
        assert!(a.h24_volume_usd.is_none());
    }

    #[tokio::test]
    async fn token_price_with_options() {
        let mock = Mock::new(
            r#"{"data":{"id":"49dd","type":"simple_token_price","attributes":{
                "token_prices":{"0xc02a":"2084.57"},"h24_volume_usd":{"0xc02a":"313441016.14"}}}}"#,
        );
        let r = mock
            .client
            .onchain()
            .token_price_with_options(
                "eth",
                &["0xc02a", "0xdac1"],
                &TokenPriceOptions::new().include_24hr_vol(true),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/onchain/simple/networks/eth/token_price/0xc02a,0xdac1?include_24hr_vol=true"
        );
        let a = r.data.attributes.unwrap();
        assert_eq!(
            a.h24_volume_usd.unwrap()["0xc02a"].as_deref(),
            Some("313441016.14")
        );
    }

    #[tokio::test]
    async fn tokens_multi_network() {
        let mock = Mock::new(
            r#"{"data":[{"id":"solana_2g4L","type":"token","attributes":{"network":"solana",
                "address":"2g4L","name":"SORACAT","symbol":"SORACAT","decimals":6,
                "image_url":null,"coingecko_coin_id":null,"total_supply":"1000000000000000.0",
                "normalized_total_supply":"1000000000.0","price_usd":"0.000006746080385",
                "fdv_usd":"6746.08","total_reserve_in_usd":"5234.61","volume_usd":{"h24":"0.0"},
                "market_cap_usd":null,"launchpad_details":{"graduation_percentage":2.16,
                    "completed":false,"completed_at":null,"migrated_destination_pool_address":null},
                "last_trade_timestamp":"1759502562"},
                "relationships":{"top_pools":{"data":[{"id":"solana_7Dyo","type":"pool"}]}}}],
                "included":[{"id":"solana_7Dyo","type":"pool","attributes":{"address":"7Dyo"}}]}"#,
        );
        let r = mock
            .client
            .onchain()
            .tokens_multi_network(
                &["solana:2g4L"],
                &TokensMultiNetworkOptions::new()
                    .include_top_pools()
                    .include_composition(true)
                    .include_inactive_source(false),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/onchain/tokens/multi?tokens=solana%3A2g4L&include=top_pools\
             &include_composition=true&include_inactive_source=false"
        );
        let a = r.data[0].attributes.as_ref().unwrap();
        assert_eq!(a.network.as_deref(), Some("solana"));
        assert_eq!(a.normalized_total_supply.as_deref(), Some("1000000000.0"));
        let launchpad = a.launchpad_details.as_ref().unwrap();
        assert_eq!(launchpad.graduation_percentage, Some(2.16));
        assert_eq!(launchpad.completed, Some(false));
        assert_eq!(r.included.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn pool_trades_range() {
        let mock = Mock::new(
            r#"{"data":[{"id":"eth_25928873_0x2d17_167_1788825063","type":"trade","attributes":{
                "block_number":25928873,"tx_hash":"0x2d17","tx_from_address":"0x0000",
                "from_token_amount":"0.475091","to_token_amount":"0.00019",
                "price_from_in_currency_token":"0.0004","price_to_in_currency_token":"1.0",
                "price_from_in_usd":"0.9997","price_to_in_usd":"2492.43",
                "block_timestamp":"2026-09-07T23:50:47Z","kind":"sell","volume_in_usd":"0.47",
                "from_token_address":"0xdac1","to_token_address":"0xc02a","is_anomalous":false}}],
                "meta":{"next_cursor":"eyJi"}}"#,
        );
        let options = TradesOptions::new()
            .trading_period("7d")
            .token("quote")
            .per_page(300);
        let r = mock
            .client
            .onchain()
            .pool_trades_range("eth", "0x06da", "2026-09-01", "2026-09-08T12:00", &options)
            .await
            .unwrap();
        // trading_period does not apply to a range and is dropped
        assert_eq!(
            mock.request(),
            "/onchain/networks/eth/pools/0x06da/trades/range?from=2026-09-01&to=2026-09-08T12%3A00&token=quote&per_page=300"
        );
        assert_eq!(
            r.data[0].attributes.as_ref().unwrap().block_number,
            Some(25_928_873)
        );
        assert_eq!(r.meta.unwrap().next_cursor.as_deref(), Some("eyJi"));
    }

    #[tokio::test]
    async fn token_trades_range() {
        let mock = Mock::new(
            r#"{"data":[{"id":"eth_25928919_0xe527_73_1788825611","type":"trade","attributes":{
                "pool_address":"0x0fb0","pool_dex":"uniswap-v4-ethereum","block_number":25928919,
                "tx_hash":"0xe527","kind":"sell","volume_in_usd":"2490.40","is_anomalous":false}}],
                "meta":{"next_cursor":null}}"#,
        );
        let options = TradesOptions::new()
            .token("base")
            .trading_period("30d")
            .trade_volume_in_usd_greater_than(1000.0)
            .include_anomalous(true)
            .cursor("abc");
        let r = mock
            .client
            .onchain()
            .token_trades_range("eth", "0xdac1", "1788300000", "1788900000", &options)
            .await
            .unwrap();
        // token and trading_period do not apply here and are dropped
        assert_eq!(
            mock.request(),
            "/onchain/networks/eth/tokens/0xdac1/trades/range?from=1788300000&to=1788900000\
             &trade_volume_in_usd_greater_than=1000&include_anomalous=true&cursor=abc"
        );
        assert_eq!(
            r.data[0].attributes.as_ref().unwrap().kind.as_deref(),
            Some("sell")
        );
        assert!(r.meta.unwrap().next_cursor.is_none());
    }
}
