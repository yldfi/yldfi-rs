//! Derivatives API endpoints

use super::types::{
    DerivativeExchange, DerivativeExchangeDetail, DerivativeExchangeListItem,
    DerivativeExchangesOptions, DerivativeTicker, DerivativeTickersOptions, UnderlyingAsset,
    UnderlyingAssetData, UnderlyingsOptions,
};
use crate::client::Client;
use crate::error::Result;

/// Derivatives API
pub struct DerivativesApi<'a> {
    client: &'a Client,
}

impl<'a> DerivativesApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List derivative tickers (first page only)
    ///
    /// Since 2026-11-20 02:00 UTC `CoinGecko` pages `/derivatives`, so this
    /// returns only the first 100 tickers. Use [`Self::list_all`] for every
    /// ticker, or [`Self::list_with_options`] to pick a page.
    pub async fn list(&self) -> Result<Vec<DerivativeTicker>> {
        self.client.get("/derivatives").await
    }

    /// List one page of derivative tickers
    pub async fn list_with_options(
        &self,
        options: &DerivativeTickersOptions,
    ) -> Result<Vec<DerivativeTicker>> {
        let path = format!("/derivatives{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// List every derivative ticker, fetching 250 per page until a short
    /// page comes back
    ///
    /// Before the 2026-11-20 cutover `CoinGecko` ignores the paging
    /// parameters and returns every ticker at once; a response longer than
    /// one page is taken as the full list.
    pub async fn list_all(&self) -> Result<Vec<DerivativeTicker>> {
        const PER_PAGE: u32 = 250;
        let mut all = Vec::new();
        let mut page = 1;
        loop {
            let options = DerivativeTickersOptions::new()
                .page(page)
                .per_page(PER_PAGE);
            let batch = self.list_with_options(&options).await?;
            if is_last_page(batch.len(), PER_PAGE) {
                if page == 1 {
                    return Ok(batch);
                }
                all.extend(batch);
                return Ok(all);
            }
            all.extend(batch);
            page += 1;
        }
    }

    /// List supported derivatives underlying assets (coins and RWAs)
    ///
    /// # Arguments
    /// * `underlying_type` - Optional filter: `coin` or `rwa`
    pub async fn underlyings_list(
        &self,
        underlying_type: Option<&str>,
    ) -> Result<Vec<UnderlyingAsset>> {
        let path = match underlying_type {
            Some(t) => format!("/derivatives/underlyings/list?underlying_type={t}"),
            None => "/derivatives/underlyings/list".to_string(),
        };
        self.client.get(&path).await
    }

    /// List derivatives underlying assets with price, open interest,
    /// funding rate and volume (Analyst plan and above)
    pub async fn underlyings(
        &self,
        options: &UnderlyingsOptions,
    ) -> Result<Vec<UnderlyingAssetData>> {
        let path = format!("/derivatives/underlyings{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// List derivatives exchanges
    pub async fn exchanges(&self) -> Result<Vec<DerivativeExchange>> {
        self.client.get("/derivatives/exchanges").await
    }

    /// List derivatives exchanges with options
    pub async fn exchanges_with_options(
        &self,
        options: &DerivativeExchangesOptions,
    ) -> Result<Vec<DerivativeExchange>> {
        let path = format!("/derivatives/exchanges{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get derivatives exchange by ID
    pub async fn exchange(&self, id: &str) -> Result<DerivativeExchangeDetail> {
        let path = format!("/derivatives/exchanges/{id}");
        self.client.get(&path).await
    }

    /// Get derivatives exchange by ID with tickers
    pub async fn exchange_with_tickers(
        &self,
        id: &str,
        include_tickers: &str,
    ) -> Result<DerivativeExchangeDetail> {
        let path = format!("/derivatives/exchanges/{id}?include_tickers={include_tickers}");
        self.client.get(&path).await
    }

    /// List derivatives exchanges (id and name only)
    pub async fn exchanges_list(&self) -> Result<Vec<DerivativeExchangeListItem>> {
        self.client.get("/derivatives/exchanges/list").await
    }
}

/// Whether a page of `len` results is the last one. A page longer than
/// `per_page` means the server ignored paging and sent everything.
fn is_last_page(len: usize, per_page: u32) -> bool {
    len != per_page as usize
}

#[cfg(test)]
mod tests {
    use super::is_last_page;

    #[test]
    fn last_page_detection() {
        assert!(!is_last_page(250, 250));
        assert!(is_last_page(249, 250));
        assert!(is_last_page(0, 250));
        // Server ignored paging and returned every ticker
        assert!(is_last_page(28_301, 250));
    }
}
