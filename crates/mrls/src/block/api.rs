//! Block API client

use super::types::{Block, LatestBlock, DateToBlock};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for block endpoints
#[derive(Debug, Default, Serialize)]
pub struct BlockQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_transactions: Option<bool>,
}

impl BlockQuery {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    #[must_use]
    pub fn include_transactions(mut self, include: bool) -> Self {
        self.include_transactions = Some(include);
        self
    }
}

/// API for block operations
pub struct BlockApi<'a> {
    client: &'a Client,
}

impl<'a> BlockApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get block by number or hash
    pub async fn get_block(
        &self,
        block_number_or_hash: &str,
        query: Option<&BlockQuery>,
    ) -> Result<Block> {
        let path = format!("/block/{block_number_or_hash}");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get latest block number for a chain
    pub async fn get_latest_block_number(&self, chain: &str) -> Result<LatestBlock> {
        let path = format!("/latestBlockNumber/{chain}");
        self.client.get(&path).await
    }

    /// Get block number by date
    pub async fn date_to_block(&self, date: &str, chain: Option<&str>) -> Result<DateToBlock> {
        #[derive(Serialize)]
        struct DateQuery {
            date: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            chain: Option<String>,
        }

        let query = DateQuery {
            date: date.to_string(),
            chain: chain.map(std::string::ToString::to_string),
        };

        self.client.get_with_query("/dateToBlock", &query).await
    }
}
