//! Types for collectibles (NFTs)

use serde::{Deserialize, Serialize};

/// Collectibles response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectiblesResponse {
    /// Wallet address
    pub address: String,
    /// Collectible entries
    pub entries: Vec<CollectibleEntry>,
    /// Pagination cursor
    pub next_offset: Option<String>,
    /// Request timestamp
    pub request_time: String,
    /// Response timestamp
    pub response_time: String,
}

/// Collectible entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectibleEntry {
    /// Contract address
    pub contract_address: String,
    /// Token standard (ERC721, ERC1155)
    pub token_standard: String,
    /// Token ID
    pub token_id: String,
    /// Chain name
    pub chain: String,
    /// Chain ID
    pub chain_id: i64,
    /// Collectible name
    pub name: Option<String>,
    /// Collectible symbol
    pub symbol: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Image URL
    pub image_url: Option<String>,
    /// Last sale price
    pub last_sale_price: Option<String>,
    /// Metadata
    pub metadata: Option<CollectibleMetadata>,
    /// Balance (quantity held)
    pub balance: String,
    /// Last acquired timestamp
    pub last_acquired: String,
    /// Spam flag
    pub is_spam: bool,
    /// Spam score (0-100)
    pub spam_score: Option<u8>,
    /// Spam explanations
    pub explanations: Option<Vec<SpamExplanation>>,
}

/// Collectible metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectibleMetadata {
    /// Metadata URI
    pub uri: Option<String>,
    /// Attributes
    pub attributes: Option<Vec<CollectibleAttribute>>,
}

/// Collectible attribute
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectibleAttribute {
    /// Attribute key
    pub key: Option<String>,
    /// Attribute value
    pub value: Option<serde_json::Value>,
    /// Attribute format
    pub format: Option<String>,
}

/// Spam explanation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpamExplanation {
    /// Feature name
    pub feature: Option<String>,
    /// Feature value
    pub value: Option<serde_json::Value>,
    /// Feature score
    pub feature_score: Option<f64>,
    /// Feature weight
    pub feature_weight: Option<f64>,
}

/// Query options for collectibles
#[derive(Debug, Clone, Default)]
pub struct CollectiblesOptions {
    /// Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
    /// Pagination offset
    pub offset: Option<String>,
    /// Results limit (max 2500)
    pub limit: Option<u32>,
    /// Filter spam (default true)
    pub filter_spam: Option<bool>,
    /// Show spam scores
    pub show_spam_scores: Option<bool>,
}

impl CollectiblesOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref chain_ids) = self.chain_ids {
            params.push(format!("chain_ids={chain_ids}"));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(filter_spam) = self.filter_spam {
            params.push(format!("filter_spam={filter_spam}"));
        }
        if let Some(show_spam_scores) = self.show_spam_scores {
            params.push(format!("show_spam_scores={show_spam_scores}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
