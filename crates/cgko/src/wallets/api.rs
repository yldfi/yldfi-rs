//! Onchain wallet API endpoints
//!
//! Every endpoint here needs the Analyst plan or above.

use super::types::{
    WalletBalancesOptions, WalletBalancesResponse, WalletPnlOptions, WalletPnlResponse,
    WalletTradesOptions, WalletTradesResponse, WalletTransfersOptions, WalletTransfersResponse,
};
use crate::client::Client;
use crate::error::Result;
use crate::query::Query;

/// Onchain wallet API (Analyst plan and above)
pub struct WalletsApi<'a> {
    client: &'a Client,
}

impl<'a> WalletsApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a wallet's token balances across networks (Analyst plan and
    /// above)
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `networks` - Network IDs, e.g. `["eth", "base"]`; all must be of
    ///   the same VM family (EVM or not), or the request returns `400`.
    ///   `CoinGecko` lists the supported networks in its docs for this
    ///   endpoint.
    pub async fn balances(
        &self,
        address: &str,
        networks: &[&str],
        options: &WalletBalancesOptions,
    ) -> Result<WalletBalancesResponse> {
        let query = options.add_to(networks_query(networks)).finish();
        let path = format!("/onchain/wallets/{address}/balances{query}");
        self.client.get(&path).await
    }

    /// Get a wallet's token transfers on one network (Analyst plan and
    /// above)
    ///
    /// Native-token transfers (e.g. ETH, SOL) are not included. Page with
    /// [`WalletTransfersOptions::cursor`] and `meta.next_cursor`.
    pub async fn transfers(
        &self,
        network: &str,
        address: &str,
        options: &WalletTransfersOptions,
    ) -> Result<WalletTransfersResponse> {
        let path = format!(
            "/onchain/networks/{network}/wallets/{address}/transfers{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get a wallet's DEX trades on one network (Analyst plan and above)
    ///
    /// Page with [`WalletTradesOptions::cursor`] and `meta.next_cursor`.
    pub async fn trades(
        &self,
        network: &str,
        address: &str,
        options: &WalletTradesOptions,
    ) -> Result<WalletTradesResponse> {
        let path = format!(
            "/onchain/networks/{network}/wallets/{address}/trades{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get a wallet's realized and unrealized PnL across networks, per
    /// network and per token (Analyst plan and above)
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `networks` - Network IDs, e.g. `["eth", "base"]`; all must be of
    ///   the same VM family (EVM or not), or the request returns `400`
    pub async fn pnl(
        &self,
        address: &str,
        networks: &[&str],
        options: &WalletPnlOptions,
    ) -> Result<WalletPnlResponse> {
        let query = options.add_to(networks_query(networks)).finish();
        let path = format!("/onchain/wallets/{address}/pnl{query}");
        self.client.get(&path).await
    }
}

fn networks_query(networks: &[&str]) -> Query {
    let networks: Vec<String> = networks.iter().map(|n| (*n).to_string()).collect();
    Query::new().list("networks", Some(&networks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::Mock;

    const ADDR: &str = "0x9642b23ed1e01df1092b92641051881a322f5d4e";

    #[tokio::test]
    async fn balances() {
        let mock = Mock::new(
            r#"{"data":{"id":"0x9642","type":"wallet_balance","attributes":{
                "wallet_address":"0x9642","total_value_usd":"91333695.89","total_holdings":2,
                "networks":[{"network":"eth","value_usd":"91333647.21","holdings":1,
                    "last_updated_at":"2026-09-09T04:53:53Z"}],
                "balances":[{"network":"eth","address":"0xae7a","token_type":"erc20",
                    "name":"Liquid staked Ether 2.0","symbol":"stETH","decimals":18,
                    "coingecko_coin_id":"staked-ether","balance_raw":"1285426654835126578249",
                    "balance":"1285.426654835126578249","price_usd":"2505.01",
                    "value_usd":"3220019.16","h24_price_change_percentage":"1.1",
                    "total_reserve_in_usd":"76309888.57"}]}}}"#,
        );
        let options = WalletBalancesOptions::new()
            .sort("value_usd_asc")
            .token_type("non_native")
            .value_usd_min(10.5)
            .reserve_in_usd_min(1000.0)
            .page(2)
            .per_page(100);
        let r = mock
            .client
            .wallets()
            .balances(ADDR, &["eth", "base"], &options)
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            format!(
                "/onchain/wallets/{ADDR}/balances?networks=eth,base&sort=value_usd_asc\
                 &token_type=non_native&value_usd_min=10.5&reserve_in_usd_min=1000&page=2&per_page=100"
            )
        );
        let a = r.data.attributes.unwrap();
        assert_eq!(a.total_holdings, Some(2));
        assert_eq!(a.networks[0].network, "eth");
        assert_eq!(
            a.balances[0].balance.as_deref(),
            Some("1285.426654835126578249")
        );
    }

    #[tokio::test]
    async fn transfers() {
        let mock = Mock::new(
            r#"{"data":[{"id":"eth_1_0xe3_299_out","type":"transfer","attributes":{
                "block_number":25937833,"tx_hash":"0xe3","direction":"out",
                "token_address":"0xdac1","amount_raw":"7999906000","amount":"7999.906",
                "block_timestamp":"2026-09-09T05:50:11Z","from_address":"0x9642",
                "to_address":"0xf893","name":"Tether USD","symbol":"USDT","decimals":6}}],
                "meta":{"next_cursor":"eyJi"}}"#,
        );
        let r = mock
            .client
            .wallets()
            .transfers(
                "eth",
                ADDR,
                &WalletTransfersOptions::new().direction("out").per_page(1),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            format!("/onchain/networks/eth/wallets/{ADDR}/transfers?direction=out&per_page=1")
        );
        let t = r.data[0].attributes.as_ref().unwrap();
        assert_eq!(t.block_number, Some(25_937_833));
        assert_eq!(t.amount.as_deref(), Some("7999.906"));
        assert_eq!(r.meta.unwrap().next_cursor.as_deref(), Some("eyJi"));
    }

    #[tokio::test]
    async fn trades() {
        let mock = Mock::new(
            r#"{"data":[{"id":"eth_26014653_0x62_70_1789858264","type":"trade","attributes":{
                "pool_address":"0x86ce","pool_dex":"uniswap_v2","block_number":26014653,
                "block_timestamp":"2026-09-19T22:50:59Z","tx_hash":"0x62","tx_from_address":"0x7942",
                "kind":"sell","from_token_address":"0x1241","to_token_address":"0xc02a",
                "from_token_amount":"103506.53","to_token_amount":"0.005","price_from_in_usd":"0.0001",
                "price_to_in_usd":"2628.96","price_from_in_currency_token":"0.00000004",
                "price_to_in_currency_token":"1.0","volume_in_usd":"13.18","is_anomalous":false}}],
                "meta":{"next_cursor":null}}"#,
        );
        let r = mock
            .client
            .wallets()
            .trades(
                "eth",
                ADDR,
                &WalletTradesOptions::new()
                    .range("2026-09-01", "2026-09-20")
                    .include_anomalous(true),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            format!(
                "/onchain/networks/eth/wallets/{ADDR}/trades?from=2026-09-01&to=2026-09-20&include_anomalous=true"
            )
        );
        let t = r.data[0].attributes.as_ref().unwrap();
        assert_eq!(t.pool_dex.as_deref(), Some("uniswap_v2"));
        assert_eq!(t.is_anomalous, Some(false));
        assert!(r.meta.unwrap().next_cursor.is_none());
    }

    #[tokio::test]
    async fn pnl() {
        let mock = Mock::new(
            r#"{"data":{"id":"0x4b3f","type":"wallet_pnl","attributes":{
                "wallet_address":"0x4b3f","total_realized_pnl_usd":"17831328.18",
                "total_unrealized_pnl_usd":"-5265158.43","total_tokens":47,
                "networks":[{"network":"eth","realized_pnl_usd":"13365120.67",
                    "unrealized_pnl_usd":"-1425379.62","tokens":35}],
                "token_stats":[{"network":"eth","address":"0x7f39","name":"wstETH",
                    "symbol":"wstETH","decimals":18,"realized_pnl_usd":"4301337.15",
                    "unrealized_pnl_usd":null,"average_buy_price_usd":"3948.37",
                    "average_sell_price_usd":"3711.76","total_buy_count":13,
                    "total_sell_count":15,"total_buy_token_amount":"1999.07",
                    "total_sell_token_amount":"3285.34","total_buy_usd":"7893095.82",
                    "total_sell_usd":"12194432.97"}]}}}"#,
        );
        let r = mock
            .client
            .wallets()
            .pnl(
                ADDR,
                &["eth"],
                &WalletPnlOptions::new()
                    .sort("total_buy_usd_desc")
                    .page(1)
                    .per_page(300),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            format!(
                "/onchain/wallets/{ADDR}/pnl?networks=eth&sort=total_buy_usd_desc&page=1&per_page=300"
            )
        );
        let a = r.data.attributes.unwrap();
        assert_eq!(a.total_tokens, Some(47));
        assert_eq!(a.networks[0].tokens, Some(35));
        assert_eq!(a.token_stats[0].total_sell_count, Some(15));
        assert!(a.token_stats[0].unrealized_pnl_usd.is_none());
    }
}
