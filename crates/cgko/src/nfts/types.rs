//! Types for NFT endpoints

use serde::{Deserialize, Serialize};

/// NFT list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftListItem {
    pub id: String,
    pub contract_address: Option<String>,
    pub name: String,
    pub asset_platform_id: Option<String>,
    pub symbol: Option<String>,
}

/// Full NFT collection data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftCollection {
    pub id: String,
    pub contract_address: Option<String>,
    pub asset_platform_id: Option<String>,
    pub name: String,
    pub symbol: Option<String>,
    pub image: Option<NftImage>,
    pub description: Option<String>,
    pub native_currency: Option<String>,
    pub native_currency_symbol: Option<String>,
    pub floor_price: Option<NftFloorPrice>,
    pub market_cap: Option<NftMarketCap>,
    pub volume_24h: Option<NftVolume>,
    pub floor_price_in_usd_24h_percentage_change: Option<f64>,
    pub floor_price_24h_percentage_change: Option<NftPercentageChange>,
    pub market_cap_24h_percentage_change: Option<NftPercentageChange>,
    pub volume_24h_percentage_change: Option<NftPercentageChange>,
    pub number_of_unique_addresses: Option<u64>,
    pub number_of_unique_addresses_24h_percentage_change: Option<f64>,
    pub total_supply: Option<f64>,
    pub one_day_sales: Option<f64>,
    pub one_day_sales_24h_percentage_change: Option<f64>,
    pub one_day_average_sale_price: Option<f64>,
    pub one_day_average_sale_price_24h_percentage_change: Option<f64>,
    pub links: Option<NftLinks>,
    pub floor_price_7d_percentage_change: Option<NftPercentageChange>,
    pub floor_price_14d_percentage_change: Option<NftPercentageChange>,
    pub floor_price_30d_percentage_change: Option<NftPercentageChange>,
    pub floor_price_60d_percentage_change: Option<NftPercentageChange>,
    pub floor_price_1y_percentage_change: Option<NftPercentageChange>,
    pub explorers: Option<Vec<NftExplorer>>,
}

/// NFT image URLs
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftImage {
    pub small: Option<String>,
}

/// NFT floor price
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftFloorPrice {
    pub native_currency: Option<f64>,
    pub usd: Option<f64>,
}

/// NFT market cap
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftMarketCap {
    pub native_currency: Option<f64>,
    pub usd: Option<f64>,
}

/// NFT volume
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftVolume {
    pub native_currency: Option<f64>,
    pub usd: Option<f64>,
}

/// NFT percentage change
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftPercentageChange {
    pub native_currency: Option<f64>,
    pub usd: Option<f64>,
}

/// NFT links
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftLinks {
    pub homepage: Option<String>,
    pub twitter: Option<String>,
    pub discord: Option<String>,
}

/// NFT explorer
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftExplorer {
    pub name: Option<String>,
    pub link: Option<String>,
}

/// NFT market data item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftMarketItem {
    pub id: String,
    pub contract_address: Option<String>,
    pub asset_platform_id: Option<String>,
    pub name: String,
    pub symbol: Option<String>,
    pub image: Option<NftImage>,
    pub floor_price_in_native_currency: Option<f64>,
    pub floor_price_24h_percentage_change: Option<f64>,
    pub volume_in_usd_24h: Option<f64>,
    pub market_cap_in_native_currency: Option<f64>,
    pub market_cap_24h_percentage_change: Option<f64>,
    pub native_currency: Option<String>,
    pub native_currency_symbol: Option<String>,
    pub h24_volume: Option<f64>,
    pub h24_volume_percentage_change: Option<f64>,
}

/// NFT ticker data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftTicker {
    pub floor_price_in_native_currency: Option<f64>,
    pub h24_volume_in_native_currency: Option<f64>,
    pub native_currency: Option<String>,
    pub native_currency_symbol: Option<String>,
    pub updated_at: Option<String>,
    pub nft_marketplace_id: Option<String>,
}

/// NFT tickers response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftTickersResponse {
    pub tickers: Vec<NftTicker>,
}

/// Options for NFT list query
#[derive(Debug, Clone, Default)]
pub struct NftListOptions {
    pub order: Option<String>,
    pub asset_platform_id: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

impl NftListOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref o) = self.order {
            params.push(format!("order={o}"));
        }
        if let Some(ref ap) = self.asset_platform_id {
            params.push(format!("asset_platform_id={ap}"));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if let Some(p) = self.page {
            params.push(format!("page={p}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// NFT market chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftMarketChart {
    pub floor_price_usd: Option<Vec<(f64, f64)>>,
    pub floor_price_native: Option<Vec<(f64, f64)>>,
    pub h24_volume_usd: Option<Vec<(f64, f64)>>,
    pub h24_volume_native: Option<Vec<(f64, f64)>>,
    pub market_cap_usd: Option<Vec<(f64, f64)>>,
    pub market_cap_native: Option<Vec<(f64, f64)>>,
}
