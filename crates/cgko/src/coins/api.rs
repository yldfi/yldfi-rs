//! Coins API endpoints

use super::types::{
    CoinContractData, CoinData, CoinHistory, CoinListItem, CoinMarket, CoinTickers, MarketChart,
    MarketsOptions, OhlcData, RecentlyAddedCoin, SupplyChart, TopMoversResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Coins API
pub struct CoinsApi<'a> {
    client: &'a Client,
}

impl<'a> CoinsApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all coins (id, name, symbol)
    pub async fn list(&self) -> Result<Vec<CoinListItem>> {
        self.client.get("/coins/list").await
    }

    /// List coins with platform contract addresses
    pub async fn list_with_platforms(&self) -> Result<Vec<CoinListItem>> {
        self.client.get("/coins/list?include_platform=true").await
    }

    /// Get coin market data
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> cgko::error::Result<()> {
    /// let client = cgko::Client::new()?;
    /// let markets = client.coins().markets("usd").await?;
    /// for coin in markets.iter().take(5) {
    ///     println!("{}: ${:.2}", coin.name, coin.current_price.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn markets(&self, vs_currency: &str) -> Result<Vec<CoinMarket>> {
        let path = format!("/coins/markets?vs_currency={vs_currency}");
        self.client.get(&path).await
    }

    /// Get coin market data with options
    pub async fn markets_with_options(
        &self,
        vs_currency: &str,
        options: &MarketsOptions,
    ) -> Result<Vec<CoinMarket>> {
        let path = format!(
            "/coins/markets?vs_currency={}{}",
            vs_currency,
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Get coin data by ID
    pub async fn get(&self, id: &str) -> Result<CoinData> {
        let path = format!("/coins/{id}");
        self.client.get(&path).await
    }

    /// Get coin tickers
    pub async fn tickers(&self, id: &str) -> Result<CoinTickers> {
        let path = format!("/coins/{id}/tickers");
        self.client.get(&path).await
    }

    /// Get coin market chart
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `vs_currency` - Target currency
    /// * `days` - Data range (1, 7, 14, 30, 90, 180, 365, "max")
    pub async fn market_chart(
        &self,
        id: &str,
        vs_currency: &str,
        days: &str,
    ) -> Result<MarketChart> {
        let path = format!("/coins/{id}/market_chart?vs_currency={vs_currency}&days={days}");
        self.client.get(&path).await
    }

    /// Get coin market chart by date range
    pub async fn market_chart_range(
        &self,
        id: &str,
        vs_currency: &str,
        from: u64,
        to: u64,
    ) -> Result<MarketChart> {
        let path =
            format!("/coins/{id}/market_chart/range?vs_currency={vs_currency}&from={from}&to={to}");
        self.client.get(&path).await
    }

    /// Get OHLC data
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `vs_currency` - Target currency
    /// * `days` - Data range (1, 7, 14, 30, 90, 180, 365)
    pub async fn ohlc(&self, id: &str, vs_currency: &str, days: u32) -> Result<OhlcData> {
        let path = format!("/coins/{id}/ohlc?vs_currency={vs_currency}&days={days}");
        self.client.get(&path).await
    }

    /// Get coin historical data for a specific date
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `date` - Date in dd-mm-yyyy format
    pub async fn history(&self, id: &str, date: &str) -> Result<CoinHistory> {
        let path = format!("/coins/{id}/history?date={date}");
        self.client.get(&path).await
    }

    /// Get coin historical data with localization option
    pub async fn history_with_localization(
        &self,
        id: &str,
        date: &str,
        localization: bool,
    ) -> Result<CoinHistory> {
        let path = format!("/coins/{id}/history?date={date}&localization={localization}");
        self.client.get(&path).await
    }

    /// Get top gainers and losers
    ///
    /// # Arguments
    /// * `vs_currency` - Target currency (e.g., "usd")
    /// * `duration` - Time duration: "1h", "24h", "7d", "14d", "30d", "60d", "1y"
    pub async fn top_gainers_losers(
        &self,
        vs_currency: &str,
        duration: &str,
    ) -> Result<TopMoversResponse> {
        let path =
            format!("/coins/top_gainers_losers?vs_currency={vs_currency}&duration={duration}");
        self.client.get(&path).await
    }

    /// Get recently added coins
    pub async fn recently_added(&self) -> Result<Vec<RecentlyAddedCoin>> {
        self.client.get("/coins/list/new").await
    }

    /// Get OHLC data by date range (Pro API only)
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `vs_currency` - Target currency
    /// * `from` - Unix timestamp start
    /// * `to` - Unix timestamp end
    pub async fn ohlc_range(
        &self,
        id: &str,
        vs_currency: &str,
        from: u64,
        to: u64,
    ) -> Result<OhlcData> {
        let path = format!("/coins/{id}/ohlc/range?vs_currency={vs_currency}&from={from}&to={to}");
        self.client.get(&path).await
    }

    /// Get coin data by contract address
    pub async fn by_contract(
        &self,
        platform_id: &str,
        contract_address: &str,
    ) -> Result<CoinContractData> {
        let path = format!("/coins/{platform_id}/contract/{contract_address}");
        self.client.get(&path).await
    }

    /// Get market chart by contract address
    pub async fn contract_market_chart(
        &self,
        platform_id: &str,
        contract_address: &str,
        vs_currency: &str,
        days: &str,
    ) -> Result<MarketChart> {
        let path = format!(
            "/coins/{platform_id}/contract/{contract_address}/market_chart?vs_currency={vs_currency}&days={days}"
        );
        self.client.get(&path).await
    }

    /// Get market chart by contract address within date range
    pub async fn contract_market_chart_range(
        &self,
        platform_id: &str,
        contract_address: &str,
        vs_currency: &str,
        from: u64,
        to: u64,
    ) -> Result<MarketChart> {
        let path = format!(
            "/coins/{platform_id}/contract/{contract_address}/market_chart/range?vs_currency={vs_currency}&from={from}&to={to}"
        );
        self.client.get(&path).await
    }

    /// Get historical circulating supply chart (Enterprise API only)
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `days` - Data range (any number, "max")
    pub async fn circulating_supply_chart(&self, id: &str, days: &str) -> Result<SupplyChart> {
        let path = format!("/coins/{id}/circulating_supply_chart?days={days}");
        self.client.get(&path).await
    }

    /// Get historical circulating supply chart by date range (Enterprise API only)
    pub async fn circulating_supply_chart_range(
        &self,
        id: &str,
        from: u64,
        to: u64,
    ) -> Result<SupplyChart> {
        let path = format!("/coins/{id}/circulating_supply_chart/range?from={from}&to={to}");
        self.client.get(&path).await
    }

    /// Get historical total supply chart (Enterprise API only)
    ///
    /// # Arguments
    /// * `id` - Coin ID
    /// * `days` - Data range (any number, "max")
    pub async fn total_supply_chart(&self, id: &str, days: &str) -> Result<SupplyChart> {
        let path = format!("/coins/{id}/total_supply_chart?days={days}");
        self.client.get(&path).await
    }

    /// Get historical total supply chart by date range (Enterprise API only)
    pub async fn total_supply_chart_range(
        &self,
        id: &str,
        from: u64,
        to: u64,
    ) -> Result<SupplyChart> {
        let path = format!("/coins/{id}/total_supply_chart/range?from={from}&to={to}");
        self.client.get(&path).await
    }
}
