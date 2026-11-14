//! Derivatives API endpoints

use super::types::{
    DerivativeExchange, DerivativeExchangeDetail, DerivativeExchangeListItem,
    DerivativeExchangesOptions, DerivativeTicker, DerivativeTickersOptions, UnderlyingAsset,
    UnderlyingAssetData, UnderlyingsOptions,
};
use crate::client::Client;
use crate::error::{pagination, Result};

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

    /// List every derivative ticker, fetching 250 per page
    ///
    /// Stops at the first page shorter than the first one, so a server cap
    /// below 250 is followed rather than taken as the end. Before the
    /// 2026-11-20 cutover `CoinGecko` ignores the paging parameters and
    /// returns every ticker at once; a response longer than a page is taken
    /// as the full list.
    ///
    /// # Errors
    /// Returns a pagination error if a page repeats the previous one (the
    /// server ignored `page`) or more than 1000 pages come back, instead of
    /// looping or returning part of the list.
    pub async fn list_all(&self) -> Result<Vec<DerivativeTicker>> {
        let mut pager = Pager::new(PER_PAGE, MAX_PAGES);
        loop {
            let options = DerivativeTickersOptions::new()
                .page(pager.next_page())
                .per_page(PER_PAGE);
            let batch = self.list_with_options(&options).await?;
            if let Some(all) = pager.push(batch, |t| (t.market.clone(), t.symbol.clone()))? {
                return Ok(all);
            }
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

const PER_PAGE: u32 = 250;
const MAX_PAGES: u32 = 1000;

/// Collects pages of a page-numbered list and decides when it is complete
struct Pager<T> {
    per_page: usize,
    max_pages: u32,
    page: u32,
    /// Length of the first page: the server's real page size
    page_size: Option<usize>,
    items: Vec<T>,
}

impl<T> Pager<T> {
    fn new(per_page: u32, max_pages: u32) -> Self {
        Self {
            per_page: per_page as usize,
            max_pages,
            page: 0,
            page_size: None,
            items: Vec::new(),
        }
    }

    /// The page number to request next
    fn next_page(&self) -> u32 {
        self.page + 1
    }

    /// Add the page just fetched. Returns the full list once it is
    /// complete. `key` identifies an item, to detect a repeated page.
    fn push<K: PartialEq>(
        &mut self,
        batch: Vec<T>,
        key: impl Fn(&T) -> K,
    ) -> Result<Option<Vec<T>>> {
        self.page += 1;
        if batch.len() > self.per_page {
            // The server ignored paging and sent the whole list
            return Ok(Some(batch));
        }
        let page_size = *self.page_size.get_or_insert(batch.len());
        if batch.is_empty() || batch.len() < page_size {
            self.items.extend(batch);
            return Ok(Some(std::mem::take(&mut self.items)));
        }
        // First item of the previous page, if there was one
        let previous = self
            .items
            .len()
            .checked_sub(page_size)
            .and_then(|i| self.items.get(i));
        if previous.is_some() && batch.first().map(&key) == previous.map(&key) {
            return Err(pagination(format!(
                "page {} repeats page {}: the server ignored the page parameter",
                self.page,
                self.page - 1
            )));
        }
        if self.page >= self.max_pages {
            return Err(pagination(format!(
                "stopped after {} pages without reaching the end",
                self.max_pages
            )));
        }
        self.items.extend(batch);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::Pager;

    /// Feed `pages` to a pager; Ok(list) when complete, Err(pages used) if not
    fn run(per_page: u32, max_pages: u32, pages: Vec<Vec<u32>>) -> Result<Vec<u32>, String> {
        let mut pager = Pager::new(per_page, max_pages);
        for page in pages {
            match pager.push(page, |x| *x) {
                Ok(Some(all)) => return Ok(all),
                Ok(None) => {}
                Err(e) => return Err(e.to_string()),
            }
        }
        Err("pager wanted more pages".into())
    }

    #[test]
    fn stops_at_a_short_page() {
        assert_eq!(
            run(3, 10, vec![vec![1, 2, 3], vec![4, 5]]),
            Ok(vec![1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn stops_at_an_empty_page() {
        assert_eq!(run(3, 10, vec![vec![1, 2, 3], vec![]]), Ok(vec![1, 2, 3]));
        assert_eq!(run(3, 10, vec![vec![]]), Ok(vec![]));
    }

    #[test]
    fn follows_a_server_cap_below_per_page() {
        // per_page 5 requested, server caps at 2: keep going until a short page
        assert_eq!(
            run(5, 10, vec![vec![1, 2], vec![3, 4], vec![5]]),
            Ok(vec![1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn whole_list_at_once_replaces_earlier_pages() {
        assert_eq!(
            run(3, 10, vec![vec![1, 2, 3, 4, 5]]),
            Ok(vec![1, 2, 3, 4, 5])
        );
        // Paging stopped working mid-way: no duplicates
        assert_eq!(
            run(3, 10, vec![vec![1, 2, 3], vec![1, 2, 3, 4, 5]]),
            Ok(vec![1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn a_repeated_page_is_an_error_not_a_loop() {
        let err = run(3, 10, vec![vec![1, 2, 3], vec![1, 2, 3]]).unwrap_err();
        assert!(err.contains("repeats page 1"), "{err}");
    }

    #[test]
    fn too_many_pages_is_an_error() {
        let err = run(1, 3, vec![vec![1], vec![2], vec![3], vec![4]]).unwrap_err();
        assert!(err.contains("after 3 pages"), "{err}");
    }
}
