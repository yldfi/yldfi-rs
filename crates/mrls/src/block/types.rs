//! Types for the Block API

use serde::{Deserialize, Serialize};

/// Block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block number
    pub number: Option<String>,
    /// Block hash
    pub hash: Option<String>,
    /// Parent hash
    pub parent_hash: Option<String>,
    /// Nonce
    pub nonce: Option<String>,
    /// SHA3 uncles
    pub sha3_uncles: Option<String>,
    /// Logs bloom
    pub logs_bloom: Option<String>,
    /// Transactions root
    pub transactions_root: Option<String>,
    /// State root
    pub state_root: Option<String>,
    /// Receipts root
    pub receipts_root: Option<String>,
    /// Miner address
    pub miner: Option<String>,
    /// Difficulty
    pub difficulty: Option<String>,
    /// Total difficulty
    pub total_difficulty: Option<String>,
    /// Extra data
    pub extra_data: Option<String>,
    /// Block size
    pub size: Option<String>,
    /// Gas limit
    pub gas_limit: Option<String>,
    /// Gas used
    pub gas_used: Option<String>,
    /// Timestamp
    pub timestamp: Option<String>,
    /// Transaction count
    #[serde(default, deserialize_with = "crate::de::string_or_integer")]
    pub transaction_count: Option<i32>,
    /// Transactions (when included)
    pub transactions: Option<Vec<serde_json::Value>>,
}

/// Date to block response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateToBlock {
    /// Block number
    #[serde(default, deserialize_with = "crate::de::string_or_integer")]
    pub block: Option<i64>,
    /// Date
    pub date: Option<String>,
    /// Timestamp
    #[serde(default, deserialize_with = "crate::de::string_or_integer")]
    pub timestamp: Option<i64>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Hash
    pub hash: Option<String>,
    /// Parent hash
    pub parent_hash: Option<String>,
}

/// Latest block number response
///
/// Reads `{"block": n}` and, as Moralis's spec documents, a bare number or
/// numeric string.
#[derive(Debug, Clone, Serialize)]
pub struct LatestBlock {
    /// Block number
    pub block: Option<i64>,
}

impl<'de> Deserialize<'de> for LatestBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Wrapped {
            #[serde(default, deserialize_with = "crate::de::string_or_integer")]
            block: Option<i64>,
        }
        let value = serde_json::Value::deserialize(deserializer)?;
        let block = if value.is_object() {
            serde_json::from_value::<Wrapped>(value)
                .map_err(serde::de::Error::custom)?
                .block
        } else {
            crate::de::string_or_integer(value).map_err(serde::de::Error::custom)?
        };
        Ok(Self { block })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_block_reads_object_number_and_string() {
        for json in [
            r#"{"block": 21500000}"#,
            "21500000",
            r#""21500000""#,
            r#"{"block": "21500000"}"#,
        ] {
            let b: LatestBlock = serde_json::from_str(json).unwrap();
            assert_eq!(b.block, Some(21_500_000), "{json}");
        }
    }
}
