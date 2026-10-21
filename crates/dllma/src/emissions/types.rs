//! Types for token emissions/unlocks data (Pro)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token emissions summary (`/api/emissions`)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionsSummary {
    /// Protocol name
    pub name: String,
    /// Token identifier (e.g. `coingecko:<id>`)
    pub token: Option<String>,
    /// `DefiLlama` protocol ID
    pub protocol_id: Option<String>,
    /// Token `CoinGecko` ID (use as the slug for [`super::EmissionsApi::get`])
    #[serde(rename = "gecko_id")]
    pub gecko_id: Option<String>,
    /// Source documents for the schedule
    #[serde(default)]
    pub sources: Vec<String>,
    /// Circulating supply
    pub circ_supply: Option<f64>,
    /// Circulating supply in 30 days
    #[serde(rename = "circSupply30d")]
    pub circ_supply_30d: Option<f64>,
    /// Total locked
    pub total_locked: Option<f64>,
    /// Max supply
    pub max_supply: Option<f64>,
    /// Market cap in USD
    pub mcap: Option<f64>,
    /// Tokens unlocked per day
    pub unlocks_per_day: Option<f64>,
    /// Next unlock event
    pub next_event: Option<NextUnlock>,
    /// Unlock events
    #[serde(default)]
    pub events: Vec<UnlockEvent>,
}

/// Next unlock event
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextUnlock {
    /// Unlock date (Unix timestamp)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub date: Option<u64>,
    /// Number of tokens to unlock
    pub to_unlock: Option<f64>,
}

/// Token unlock event
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockEvent {
    /// Event description template
    pub description: Option<String>,
    /// Event date (Unix timestamp)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub timestamp: Option<u64>,
    /// Tokens unlocked, one entry per recipient
    #[serde(default)]
    pub no_of_tokens: Vec<f64>,
    /// Category (e.g. insiders, noncirculating)
    pub category: Option<String>,
    /// Unlock type (cliff, linear, etc.)
    pub unlock_type: Option<String>,
}

/// Detailed emission schedule (`/api/emission/{protocol}`)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionDetail {
    /// Schedule data (accepted both as an object and as a JSON-encoded string)
    #[serde(deserialize_with = "crate::de::json_or_json_string")]
    pub body: EmissionBody,
    /// Last modification time of the data
    pub last_modified: Option<String>,
}

/// Emission schedule body
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionBody {
    /// Documented (from sources) unlock data
    pub documented_data: Option<EmissionData>,
    /// Token and schedule metadata
    pub metadata: Option<EmissionMetadata>,
}

/// Unlock series and allocation
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionData {
    /// Unlock series, one per allocation label
    #[serde(default)]
    pub data: Vec<EmissionSeries>,
    /// Token allocation by category
    pub token_allocation: Option<TokenAllocation>,
}

/// Unlock series for one allocation label
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmissionSeries {
    /// Allocation label (e.g. "Core Contributors")
    pub label: String,
    /// Data points
    #[serde(default)]
    pub data: Vec<EmissionPoint>,
}

/// Unlock data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionPoint {
    /// Unix timestamp
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub timestamp: Option<u64>,
    /// Cumulative tokens unlocked
    pub unlocked: Option<f64>,
    /// Tokens emitted in this period
    pub raw_emission: Option<f64>,
    /// Tokens burned
    pub burned: Option<f64>,
}

/// Token allocation by category (e.g. insiders, airdrop, farming)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TokenAllocation {
    /// Current allocation
    #[serde(default)]
    pub current: HashMap<String, f64>,
    /// Final allocation
    #[serde(default, rename = "final")]
    pub final_allocation: HashMap<String, f64>,
    /// Progress towards the final allocation
    #[serde(default)]
    pub progress: HashMap<String, f64>,
}

/// Emission metadata
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionMetadata {
    /// Protocol name
    pub name: Option<String>,
    /// Token identifier (e.g. `coingecko:<id>`)
    pub token: Option<String>,
    /// Token `CoinGecko` ID
    #[serde(rename = "gecko_id")]
    pub gecko_id: Option<String>,
    /// Total supply covered by the schedule
    pub total: Option<f64>,
    /// Chain identifier
    pub chain: Option<String>,
    /// Chain display name
    pub chain_name: Option<String>,
    /// Protocol category
    pub protocol_category: Option<String>,
    /// Protocol ID
    pub p_id: Option<String>,
    /// Notes
    #[serde(default)]
    pub notes: Vec<String>,
    /// Source documents
    #[serde(default)]
    pub sources: Vec<String>,
    /// Protocol IDs
    #[serde(default)]
    pub protocol_ids: Vec<String>,
    /// `DefiLlama` IDs
    #[serde(default)]
    pub defillama_ids: Vec<String>,
    /// Allocation labels keyed by category
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = r#"{"documentedData":{"data":[{"label":"Team","data":[
        {"timestamp":1700000000,"unlocked":100,"rawEmission":100,"burned":0}]}],
        "tokenAllocation":{"current":{"insiders":10.5},"final":{"insiders":20},
        "progress":{"insiders":52.5}}},
        "metadata":{"token":"coingecko:foo","sources":["https://example.com/doc"],
        "protocolIds":["1"],"total":1000,"chain":"ethereum","name":"Foo","gecko_id":"foo",
        "defillamaIds":["1"],"categories":{"insiders":["Team"]},"protocolCategory":"Dexs",
        "chainName":"Ethereum","pId":"1","notes":[]}}"#;

    fn check(d: &EmissionDetail) {
        let data = d.body.documented_data.as_ref().unwrap();
        assert_eq!(data.data[0].label, "Team");
        assert_eq!(data.data[0].data[0].timestamp, Some(1_700_000_000));
        assert_eq!(data.data[0].data[0].raw_emission, Some(100.0));
        let alloc = data.token_allocation.as_ref().unwrap();
        assert_eq!(alloc.final_allocation["insiders"], 20.0);
        let meta = d.body.metadata.as_ref().unwrap();
        assert_eq!(meta.gecko_id.as_deref(), Some("foo"));
        assert_eq!(meta.p_id.as_deref(), Some("1"));
        assert_eq!(meta.categories["insiders"], vec!["Team"]);
        assert_eq!(d.last_modified.as_deref(), Some("2024-01-01T00:00:00.000Z"));
    }

    #[test]
    fn emission_detail_object_body() {
        let json = format!(r#"{{"body":{BODY},"lastModified":"2024-01-01T00:00:00.000Z"}}"#);
        check(&serde_json::from_str(&json).unwrap());
    }

    #[test]
    fn emission_detail_string_body() {
        let wrapped = serde_json::json!({
            "body": BODY,
            "lastModified": "2024-01-01T00:00:00.000Z",
        });
        check(&serde_json::from_value(wrapped).unwrap());
    }

    #[test]
    fn emissions_list_parses() {
        let json = r#"[{"token":"coingecko:foo","sources":["https://example.com/doc"],
            "protocolId":"1","name":"Foo","circSupply":293.5,"circSupply30d":300,
            "totalLocked":81.5,"maxSupply":375,"gecko_id":"foo",
            "events":[{"description":"A cliff","timestamp":1700000000,"noOfTokens":[120],
            "category":"noncirculating","unlockType":"cliff"}],
            "nextEvent":{"date":1800000000,"toUnlock":81.5},"unlocksPerDay":0,"mcap":1000}]"#;
        let l: Vec<EmissionsSummary> = serde_json::from_str(json).unwrap();
        let e = &l[0];
        assert_eq!(e.gecko_id.as_deref(), Some("foo"));
        assert_eq!(e.circ_supply_30d, Some(300.0));
        assert_eq!(e.next_event.as_ref().unwrap().date, Some(1_800_000_000));
        assert_eq!(e.events[0].no_of_tokens, vec![120.0]);
        assert_eq!(e.events[0].unlock_type.as_deref(), Some("cliff"));
    }
}
