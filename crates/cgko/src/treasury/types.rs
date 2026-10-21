//! Types for treasury endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub symbol: Option<String>,
    pub country: Option<String>,
    pub website_url: Option<String>,
    pub twitter_screen_name: Option<String>,
    pub total_treasury_value_usd: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    /// Market cap to net asset value ratio
    pub m_nav: Option<f64>,
    pub total_asset_value_per_share_usd: Option<f64>,
    #[serde(default)]
    pub holdings: Vec<EntityHolding>,
}

/// Entity holding
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntityHolding {
    pub coin_id: String,
    pub amount: Option<f64>,
    pub percentage_of_total_supply: Option<f64>,
    pub amount_per_share: Option<f64>,
    pub entity_value_usd_percentage: Option<f64>,
    pub current_value_usd: Option<f64>,
    pub total_entry_value_usd: Option<f64>,
    pub average_entry_value_usd: Option<f64>,
    pub unrealized_pnl: Option<f64>,
    /// Holding amount change keyed by period (`7d`, `14d`, `30d`, `90d`, `1y`, `ytd`)
    pub holding_amount_change: Option<HashMap<String, f64>>,
    /// Holding change percentage keyed by period
    pub holding_change_percentage: Option<HashMap<String, f64>>,
}

/// Holding chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoldingChart {
    /// Vec of [timestamp, holdings]
    #[serde(default)]
    pub holdings: Vec<(u64, f64)>,
    /// Vec of [timestamp, holding value in USD]
    #[serde(default)]
    pub holding_value_in_usd: Vec<(u64, f64)>,
}

/// Transaction history
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionHistory {
    pub transactions: Vec<TreasuryTransaction>,
}

/// Treasury transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreasuryTransaction {
    /// Transaction date (Unix ms)
    pub date: Option<u64>,
    pub source_url: Option<String>,
    pub coin_id: Option<String>,
    /// `buy` or `sell`
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub holding_net_change: Option<f64>,
    pub transaction_value_usd: Option<f64>,
    pub holding_balance: Option<f64>,
    pub average_entry_value_usd: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_holdings_match_spec() {
        let json = r#"{"name": "Example Co", "id": "example-co", "type": "company",
            "symbol": "EXCO.US", "country": "US", "website_url": "https://example.com/",
            "twitter_screen_name": "example", "total_treasury_value_usd": 6438.5,
            "unrealized_pnl": 51.3, "m_nav": 1.03, "total_asset_value_per_share_usd": 179.7,
            "holdings": [{"coin_id": "bitcoin", "amount": 843.0, "percentage_of_total_supply": 4.0,
                "amount_per_share": 0.0023, "entity_value_usd_percentage": 100.0,
                "current_value_usd": 6438.5, "total_entry_value_usd": 6387.0,
                "average_entry_value_usd": 75.6, "unrealized_pnl": 51.3,
                "holding_amount_change": {"7d": 0.0, "ytd": 171.0},
                "holding_change_percentage": {"7d": 0.0, "ytd": 25.4}}]}"#;
        let e: PublicTreasuryByEntity = serde_json::from_str(json).unwrap();
        assert_eq!(e.m_nav, Some(1.03));
        let h = &e.holdings[0];
        assert_eq!(h.amount, Some(843.0));
        assert_eq!(h.holding_amount_change.as_ref().unwrap()["ytd"], 171.0);
    }

    #[test]
    fn holding_chart_and_transactions_match_spec() {
        let chart: HoldingChart = serde_json::from_str(
            r#"{"holdings": [[1748736000000, 580.0]], "holding_value_in_usd": [[1748736000000, 6081.6]]}"#,
        )
        .unwrap();
        assert_eq!(chart.holding_value_in_usd[0], (1_748_736_000_000, 6081.6));

        let history: TransactionHistory = serde_json::from_str(
            r#"{"transactions": [{"date": 1779062400000, "source_url": "https://example.com/filing.pdf",
                "coin_id": "bitcoin", "type": "buy", "holding_net_change": 248.0,
                "transaction_value_usd": 2014.0, "holding_balance": 843.0,
                "average_entry_value_usd": 80.9}]}"#,
        )
        .unwrap();
        let tx = &history.transactions[0];
        assert_eq!(tx.transaction_type.as_deref(), Some("buy"));
        assert_eq!(tx.holding_balance, Some(843.0));
    }
}
