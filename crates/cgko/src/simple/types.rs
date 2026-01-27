//! Types for simple price endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Price data for a coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceData {
    #[serde(flatten)]
    pub prices: HashMap<String, f64>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from /simple/price
pub type PricesResponse = HashMap<String, HashMap<String, serde_json::Value>>;

/// Response from /`simple/token_price/{id`}
pub type TokenPricesResponse = HashMap<String, HashMap<String, serde_json::Value>>;

/// Supported vs currencies
pub type SupportedCurrencies = Vec<String>;

/// Options for price queries
#[derive(Debug, Clone, Default)]
pub struct PriceOptions {
    pub include_market_cap: bool,
    pub include_24hr_vol: bool,
    pub include_24hr_change: bool,
    pub include_last_updated_at: bool,
    pub precision: Option<String>,
}

impl PriceOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn full() -> Self {
        Self {
            include_market_cap: true,
            include_24hr_vol: true,
            include_24hr_change: true,
            include_last_updated_at: true,
            precision: None,
        }
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if self.include_market_cap {
            params.push("include_market_cap=true".to_string());
        }
        if self.include_24hr_vol {
            params.push("include_24hr_vol=true".to_string());
        }
        if self.include_24hr_change {
            params.push("include_24hr_change=true".to_string());
        }
        if self.include_last_updated_at {
            params.push("include_last_updated_at=true".to_string());
        }
        if let Some(ref p) = self.precision {
            params.push(format!("precision={p}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("&{}", params.join("&"))
        }
    }
}
