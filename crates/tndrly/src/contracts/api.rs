//! Contract API operations

use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Contract API client
pub struct ContractsApi<'a> {
    client: &'a Client,
}

impl<'a> ContractsApi<'a> {
    /// Create a new Contract API client
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

    /// Update a contract
    pub async fn update(
        &self,
        network_id: &str,
        address: &str,
        request: &UpdateContractRequest,
    ) -> Result<Contract> {
        self.client
            .patch(
                &format!(
                    "/contract/{}/{}",
                    encode_path_segment(network_id),
                    encode_path_segment(address)
                ),
                request,
            )
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

    /// Verify a contract
    ///
    /// Submits source code for verification. If successful, the contract
    /// will show as verified and its ABI will be available.
    pub async fn verify(&self, request: &VerifyContractRequest) -> Result<VerificationResult> {
        self.client.post("/contract/verify", request).await
    }

    /// Encode state overrides for use in simulations
    ///
    /// This converts human-readable state overrides into the format
    /// expected by the simulation API.
    pub async fn encode_state(&self, request: &EncodeStateRequest) -> Result<EncodeStateResponse> {
        self.client.post("/contract/encode-states", request).await
    }

    /// Add a tag to a contract
    ///
    /// # Note
    ///
    /// This operation is not atomic. It fetches the current tags, modifies them,
    /// and updates the contract. If another client modifies the tags between these
    /// operations, their changes may be overwritten (TOCTOU race condition).
    /// For concurrent access, consider using the direct `update()` method with
    /// your own synchronization.
    pub async fn add_tag(&self, network_id: &str, address: &str, tag: &str) -> Result<Contract> {
        let contract = self.get(network_id, address).await?;
        let mut tags = contract.tags();
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
        let request = UpdateContractRequest::new().tags(tags);
        self.update(network_id, address, &request).await
    }

    /// Remove a tag from a contract
    ///
    /// # Note
    ///
    /// This operation is not atomic. See [`add_tag`](Self::add_tag) for details.
    pub async fn remove_tag(&self, network_id: &str, address: &str, tag: &str) -> Result<Contract> {
        let contract = self.get(network_id, address).await?;
        let tags: Vec<String> = contract.tags().into_iter().filter(|t| t != tag).collect();
        let request = UpdateContractRequest::new().tags(tags);
        self.update(network_id, address, &request).await
    }

    /// Get the ABI for a contract
    ///
    /// Returns None if the contract is not verified or ABI is not available.
    /// Note: ABI data is not included in the standard contract response.
    /// You may need to fetch it separately from the verification endpoint.
    pub async fn abi(
        &self,
        _network_id: &str,
        _address: &str,
    ) -> Result<Option<serde_json::Value>> {
        // ABI is not included in the standard contract response
        // This would require a separate API call to fetch ABI data
        Ok(None)
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
    fn test_verify_contract_request() {
        let request = VerifyContractRequest::new(
            "1",
            "0x1234",
            "MyContract",
            "pragma solidity ^0.8.0;",
            "v0.8.19+commit.7dd6d404",
        )
        .optimization(true, 200)
        .evm_version("paris");

        assert_eq!(request.network_id, "1");
        assert!(request.optimization.is_some());
        assert_eq!(request.evm_version, Some("paris".to_string()));
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

    #[test]
    fn test_verify_contract_request_serialization() {
        // Verify JSON structure for contract verification
        let request = VerifyContractRequest::new(
            "1",
            "0x1234",
            "MyContract",
            "pragma solidity ^0.8.0;",
            "v0.8.19+commit.7dd6d404",
        )
        .optimization(true, 200)
        .evm_version("paris");

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["network_id"], "1");
        assert_eq!(json["address"], "0x1234");
        assert_eq!(json["contract_name"], "MyContract");
        assert_eq!(json["compiler_version"], "v0.8.19+commit.7dd6d404");

        // Verify optimization is nested correctly
        assert!(json["optimization"].is_object());
        assert_eq!(json["optimization"]["enabled"], true);
        assert_eq!(json["optimization"]["runs"], 200);

        assert_eq!(json["evm_version"], "paris");
    }
}
