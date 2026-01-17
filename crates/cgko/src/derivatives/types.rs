//! Types for derivatives endpoints

use serde::{Deserialize, Serialize};

/// Derivative ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeTicker {
    pub market: Option<String>,
    pub symbol: Option<String>,
    pub index_id: Option<String>,
    pub price: Option<String>,
    pub price_percentage_change_24h: Option<f64>,
    pub contract_type: Option<String>,
    pub index: Option<f64>,
    pub basis: Option<f64>,
    pub spread: Option<f64>,
    pub funding_rate: Option<f64>,
    pub open_interest: Option<f64>,
    pub volume_24h: Option<f64>,
    pub last_traded_at: Option<u64>,
    pub expired_at: Option<String>,
}

/// Derivatives exchange list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeListItem {
    pub id: String,
    pub name: String,
}

/// Derivatives exchange data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchange {
    pub name: String,
    pub id: String,
    pub open_interest_btc: Option<f64>,
    pub trade_volume_24h_btc: Option<String>,
    pub number_of_perpetual_pairs: Option<u32>,
    pub number_of_futures_pairs: Option<u32>,
    pub image: Option<String>,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// Detailed derivatives exchange data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeDetail {
    pub name: String,
    pub open_interest_btc: Option<f64>,
    pub trade_volume_24h_btc: Option<String>,
    pub number_of_perpetual_pairs: Option<u32>,
    pub number_of_futures_pairs: Option<u32>,
    pub image: Option<String>,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub tickers: Option<Vec<DerivativeExchangeTicker>>,
}

/// Derivative exchange ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeTicker {
    pub symbol: Option<String>,
    pub base: Option<String>,
    pub target: Option<String>,
    pub trade_url: Option<String>,
    pub contract_type: Option<String>,
    pub last: Option<f64>,
    pub h24_percentage_change: Option<f64>,
    pub index: Option<f64>,
    pub index_basis_percentage: Option<f64>,
    pub bid_ask_spread: Option<f64>,
    pub funding_rate: Option<f64>,
    pub open_interest_usd: Option<f64>,
    pub h24_volume: Option<f64>,
    pub converted_volume: Option<ConvertedVolume>,
    pub converted_last: Option<ConvertedLast>,
    pub last_traded: Option<u64>,
    pub expired_at: Option<String>,
}

/// Converted volume
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertedVolume {
    pub btc: Option<f64>,
    pub eth: Option<f64>,
    pub usd: Option<f64>,
}

/// Converted last price
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertedLast {
    pub btc: Option<f64>,
    pub eth: Option<f64>,
    pub usd: Option<f64>,
}

/// Options for derivatives query
#[derive(Debug, Clone, Default)]
pub struct DerivativesOptions {
    pub include_tickers: Option<String>,
}

impl DerivativesOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn all_tickers() -> Self {
        Self {
            include_tickers: Some("all".to_string()),
        }
    }

    pub fn unexpired_tickers() -> Self {
        Self {
            include_tickers: Some("unexpired".to_string()),
        }
    }

    pub fn to_query_string(&self) -> String {
        match &self.include_tickers {
            Some(t) => format!("?include_tickers={}", t),
            None => String::new(),
        }
    }
}

/// Options for derivatives exchanges query
#[derive(Debug, Clone, Default)]
pub struct DerivativeExchangesOptions {
    pub order: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

impl DerivativeExchangesOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref o) = self.order {
            params.push(format!("order={}", o));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={}", pp));
        }
        if let Some(p) = self.page {
            params.push(format!("page={}", p));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
