//! Treasury API endpoints

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Treasury API (Public companies and governments holding crypto)
pub struct TreasuryApi<'a> {
    client: &'a Client,
}

impl<'a> TreasuryApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all public entities (companies/governments)
    pub async fn entities(&self) -> Result<Vec<EntityListItem>> {
        self.client.get("/entities/list").await
    }

    /// Get public treasury holdings by coin ID
    ///
    /// # Arguments
    /// * `entity` - Entity type: "companies" or "governments"
    /// * `coin_id` - Coin ID (e.g., "bitcoin", "ethereum")
    pub async fn by_coin(&self, entity: &str, coin_id: &str) -> Result<PublicTreasuryByCoin> {
        let path = format!("/{}/public_treasury/{}", entity, coin_id);
        self.client.get(&path).await
    }

    /// Get public treasury holdings by entity ID
    pub async fn by_entity(&self, entity_id: &str) -> Result<PublicTreasuryByEntity> {
        let path = format!("/public_treasury/{}", entity_id);
        self.client.get(&path).await
    }

    /// Get historical holdings chart for an entity's coin holdings
    pub async fn holding_chart(&self, entity_id: &str, coin_id: &str) -> Result<HoldingChart> {
        let path = format!("/public_treasury/{}/{}/holding_chart", entity_id, coin_id);
        self.client.get(&path).await
    }

    /// Get transaction history for an entity
    pub async fn transaction_history(&self, entity_id: &str) -> Result<TransactionHistory> {
        let path = format!("/public_treasury/{}/transaction_history", entity_id);
        self.client.get(&path).await
    }
}
