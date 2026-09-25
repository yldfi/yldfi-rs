//! Contract API operations

use super::types::{
    AddContractRequest, BulkTagRequest, BulkTagResponse, Contract, DeleteTagRequest,
    EncodeStateRequest, EncodeStateResponse, ListContractsQuery, RenameContractRequest,
};
use crate::client::{encode_path_segment, Client};
use crate::error::{self, Result};

/// Build a Tenderly contract ID (`eth:{network_id}:{address}`) as used by `POST /tag`
#[must_use]
pub fn contract_id(network_id: &str, address: &str) -> String {
    format!("eth:{}:{}", network_id, address.to_lowercase())
}

/// Contract API client
pub struct ContractsApi<'a> {
    client: &'a Client,
}

impl<'a> ContractsApi<'a> {
    /// Create a new Contract API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Add a contract to the project
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = AddContractRequest::new("1", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    ///     .display_name("USDC")
    ///     .tag("stablecoin")
    ///     .tag("defi");
    ///
    /// let contract = client.contracts().add(&request).await?;
    /// ```
    pub async fn add(&self, request: &AddContractRequest) -> Result<Contract> {
        self.client.post("/address", request).await
    }

    /// List contracts in the project
    ///
    /// Returns a vector of Contracts directly (API returns a raw array).
    pub async fn list(&self, query: Option<ListContractsQuery>) -> Result<Vec<Contract>> {
        match query {
            Some(q) => self.client.get_with_query("/contracts", &q).await,
            None => self.client.get("/contracts").await,
        }
    }

    /// Get a contract by network and address
    ///
    /// This is the primary method for retrieving contract information.
    pub async fn get(&self, network_id: &str, address: &str) -> Result<Contract> {
        self.client
            .get(&format!(
                "/contract/{}/{}",
                encode_path_segment(network_id),
                encode_path_segment(address)
            ))
            .await
    }

    /// Delete a contract
    pub async fn delete(&self, network_id: &str, address: &str) -> Result<()> {
        self.client
            .delete(&format!(
                "/contract/{}/{}",
                encode_path_segment(network_id),
                encode_path_segment(address)
            ))
            .await
    }

    /// Encode state overrides for use in simulations
    ///
    /// Calls `POST /contracts/encode-states`, converting human-readable
    /// (named variable) state overrides into the raw storage-slot format
    /// expected by the simulation API.
    pub async fn encode_state(&self, request: &EncodeStateRequest) -> Result<EncodeStateResponse> {
        self.client.post("/contracts/encode-states", request).await
    }

    /// Add a tag to a contract and return the updated contract
    ///
    /// Uses `POST /tag` (see [`bulk_tag`](Self::bulk_tag)) with the contract ID
    /// `eth:{network_id}:{address}`, then re-fetches the contract.
    pub async fn add_tag(&self, network_id: &str, address: &str, tag: &str) -> Result<Contract> {
        self.bulk_tag(tag, vec![contract_id(network_id, address)])
            .await?;
        self.get(network_id, address).await
    }

    /// Remove a tag from a contract and return the updated contract
    ///
    /// Uses `DELETE /contract/{network}/{address}/tag` (see
    /// [`delete_tag`](Self::delete_tag)), then re-fetches the contract.
    pub async fn remove_tag(&self, network_id: &str, address: &str, tag: &str) -> Result<Contract> {
        self.delete_tag(network_id, address, tag).await?;
        self.get(network_id, address).await
    }

    /// Get the ABI for a contract
    ///
    /// # Errors
    ///
    /// Always returns an [`InvalidParam`](crate::error::DomainError::InvalidParam)
    /// error: the public Tenderly REST API (OpenAPI spec) has no endpoint that
    /// returns a contract ABI. Fetch ABIs from a block explorer (e.g. Etherscan)
    /// instead.
    #[deprecated(
        since = "0.3.9",
        note = "Tenderly's public API has no contract ABI endpoint; this always errors"
    )]
    pub async fn abi(
        &self,
        _network_id: &str,
        _address: &str,
    ) -> Result<Option<serde_json::Value>> {
        Err(error::invalid_param(
            "Tenderly's public API does not provide a contract ABI endpoint; \
             fetch the ABI from a block explorer (e.g. Etherscan) instead",
        ))
    }

    /// Rename a contract
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.contracts().rename("1", "0x1234...", "My New Contract Name").await?;
    /// ```
    pub async fn rename(
        &self,
        network_id: &str,
        address: &str,
        display_name: impl Into<String>,
    ) -> Result<()> {
        let request = RenameContractRequest {
            display_name: display_name.into(),
        };
        self.client
            .post_no_response(
                &format!(
                    "/contract/{}/{}/rename",
                    encode_path_segment(network_id),
                    encode_path_segment(address)
                ),
                &request,
            )
            .await
    }

    /// Add a tag to multiple contracts at once
    ///
    /// # Example
    ///
    /// ```ignore
    /// let contract_ids = vec![
    ///     "eth:1:0x1234...".to_string(),
    ///     "eth:1:0x5678...".to_string(),
    /// ];
    /// client.contracts().bulk_tag("v1.0.0", contract_ids).await?;
    /// ```
    pub async fn bulk_tag(
        &self,
        tag: impl Into<String>,
        contract_ids: Vec<String>,
    ) -> Result<BulkTagResponse> {
        let request = BulkTagRequest {
            tag: tag.into(),
            contract_ids,
        };
        self.client.post("/tag", &request).await
    }

    /// Delete a tag from a contract
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.contracts().delete_tag("1", "0x1234...", "old-tag").await?;
    /// ```
    pub async fn delete_tag(
        &self,
        network_id: &str,
        address: &str,
        tag: impl Into<String>,
    ) -> Result<()> {
        let request = DeleteTagRequest { tag: tag.into() };
        self.client
            .delete_with_body(
                &format!(
                    "/contract/{}/{}/tag",
                    encode_path_segment(network_id),
                    encode_path_segment(address)
                ),
                &request,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::StateOverrideInput;

    #[test]
    fn test_add_contract_request() {
        let request = AddContractRequest::new("1", "0x1234")
            .display_name("My Contract")
            .tag("defi")
            .tag("trading");

        assert_eq!(request.network_id, "1");
        assert_eq!(request.address, "0x1234");
        assert_eq!(request.display_name, Some("My Contract".to_string()));
        assert_eq!(
            request.tags,
            Some(vec!["defi".to_string(), "trading".to_string()])
        );
    }

    #[test]
    fn test_state_override_input() {
        let override_input = StateOverrideInput::new()
            .balance("1000000000000000000")
            .storage("0x0", "0x1")
            .nonce(10);

        assert_eq!(
            override_input.balance,
            Some("1000000000000000000".to_string())
        );
        assert!(override_input.storage.is_some());
        assert_eq!(override_input.nonce, Some(10));
    }

    #[test]
    fn test_encode_state_request_serialization() {
        // Field names must match POST /contracts/encode-states in the OpenAPI spec
        let mut overrides = std::collections::HashMap::new();
        overrides.insert(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(),
            StateOverrideInput::new().value("balances[0xabc]", "1000"),
        );
        let request = EncodeStateRequest::new("1", overrides).block_number("-1");
        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["networkID"], "1");
        assert_eq!(json["blockNumber"], "-1");
        assert_eq!(
            json["stateOverrides"]["0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"]["value"]
                ["balances[0xabc]"],
            "1000"
        );
        assert!(json.get("network_id").is_none());
    }

    #[test]
    fn test_encode_state_response_accepts_state_overrides_key() {
        let resp: EncodeStateResponse =
            serde_json::from_str(r#"{"stateOverrides":{"0xabc":{"value":{"0x0":"0x1"}}}}"#)
                .unwrap();
        assert_eq!(resp.encoded_state["0xabc"]["value"]["0x0"], "0x1");
    }

    #[test]
    fn test_contract_id_format() {
        assert_eq!(contract_id("1", "0xABCdef"), "eth:1:0xabcdef");
    }

    #[test]
    fn test_add_contract_request_serialization() {
        // Verify JSON structure matches Tenderly API expectations
        let request = AddContractRequest::new("1", "0x1234")
            .display_name("My Contract")
            .tag("defi");

        let json = serde_json::to_value(&request).unwrap();

        // Verify field names serialize correctly
        assert_eq!(json["network_id"], "1");
        assert_eq!(json["address"], "0x1234");
        assert_eq!(json["display_name"], "My Contract");
        assert!(json["tags"].is_array());
        assert_eq!(json["tags"][0], "defi");
    }
}
