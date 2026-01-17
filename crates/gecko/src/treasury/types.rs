//! Types for treasury endpoints

use serde::{Deserialize, Serialize};

/// Entity list item (companies/governments that hold crypto)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityListItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
}

/// Public treasury holdings by coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicTreasuryByCoin {
    pub total_holdings: Option<f64>,
    pub total_value_usd: Option<f64>,
    pub market_cap_dominance: Option<f64>,
    pub companies: Vec<CompanyHolding>,
}

/// Company holding data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompanyHolding {
    pub name: String,
    pub symbol: Option<String>,
    pub country: Option<String>,
    pub total_holdings: Option<f64>,
    pub total_entry_value_usd: Option<f64>,
    pub total_current_value_usd: Option<f64>,
    pub percentage_of_total_supply: Option<f64>,
}

/// Public treasury by entity
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicTreasuryByEntity {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    pub holdings: Vec<EntityHolding>,
}

/// Entity holding
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityHolding {
    pub coin_id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub total_holdings: Option<f64>,
    pub total_value_usd: Option<f64>,
}

/// Holding chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldingChart {
    /// Vec of [timestamp, holdings]
    pub holdings: Vec<(u64, f64)>,
}

/// Transaction history
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionHistory {
    pub transactions: Vec<TreasuryTransaction>,
}

/// Treasury transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreasuryTransaction {
    pub coin_id: Option<String>,
    pub symbol: Option<String>,
    pub amount: Option<f64>,
    pub value_usd: Option<f64>,
    pub transaction_type: Option<String>,
    pub timestamp: Option<u64>,
}
