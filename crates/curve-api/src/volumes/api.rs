//! Volumes and APYs API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// API for Curve volumes and APYs
pub struct VolumesApi<'a> {
    client: &'a Client,
}

impl<'a> VolumesApi<'a> {
    /// Create a new volumes API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all gauges across all chains
    pub async fn get_all_gauges(&self) -> Result<GaugesResponse> {
        self.client.get("/getAllGauges").await
    }

    /// Get total 24h volume for a chain
    pub async fn get_total_volume(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/getAllPoolsVolume/{}", chain);
        self.client.get(&path).await
    }

    /// Get base APYs for pools on a chain
    pub async fn get_base_apys(&self, chain: &str) -> Result<BaseApysResponse> {
        let path = format!("/getBaseApys/{}", chain);
        self.client.get(&path).await
    }

    /// Get volumes from Curve Prices API (more reliable)
    pub async fn get_volumes(&self, chain: &str) -> Result<VolumesResponse> {
        let path = format!("/getVolumes/{}", chain);
        self.client.get(&path).await
    }

    /// Get volumes from subgraph
    pub async fn get_subgraph_data(&self, chain: &str) -> Result<VolumesResponse> {
        let path = format!("/getSubgraphData/{}", chain);
        self.client.get(&path).await
    }

    /// Get factory gauge CRV rewards
    pub async fn get_factory_gauge_rewards(&self, chain: &str) -> Result<serde_json::Value> {
        let path = format!("/getFactoGaugesCrvRewards/{}", chain);
        self.client.get(&path).await
    }

    /// Get crvUSD AMM volumes
    pub async fn get_crvusd_amm_volumes(&self) -> Result<serde_json::Value> {
        self.client.get("/getVolumes/ethereum/crvusd-amms").await
    }
}
