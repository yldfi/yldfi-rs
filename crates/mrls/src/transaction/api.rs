//! Transaction API client

use super::types::{Transaction, VerboseTransaction};
use crate::client::Client;
use crate::error::Result;
use crate::wallet::PaginatedResponse;
use serde::Serialize;

/// Query parameters for transaction endpoints
#[derive(Debug, Default, Serialize)]
pub struct TransactionQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_internal_transactions: Option<bool>,
}

impl TransactionQuery {
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
    pub fn include_internal_transactions(mut self, include: bool) -> Self {
        self.include_internal_transactions = Some(include);
        self
    }
}

/// API for transaction operations
pub struct TransactionApi<'a> {
    client: &'a Client,
}

impl<'a> TransactionApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, tx_hash: &str, chain: Option<&str>) -> Result<Transaction> {
        let path = format!("/transaction/{tx_hash}");
        if let Some(chain) = chain {
            let query = TransactionQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get verbose transaction with decoded data and internal transactions
    pub async fn get_transaction_verbose(
        &self,
        tx_hash: &str,
        query: Option<&TransactionQuery>,
    ) -> Result<VerboseTransaction> {
        let path = format!("/transaction/{tx_hash}/verbose");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get transactions for an address
    pub async fn get_wallet_transactions(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<PaginatedResponse<Transaction>> {
        let path = format!("/{address}");
        if let Some(chain) = chain {
            let query = TransactionQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get verbose transactions for an address
    pub async fn get_wallet_transactions_verbose(
        &self,
        address: &str,
        query: Option<&TransactionQuery>,
    ) -> Result<PaginatedResponse<VerboseTransaction>> {
        let path = format!("/{address}/verbose");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }
}
