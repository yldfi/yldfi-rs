//! Token API implementation (RPC methods)

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Token API for ERC-20 token operations via RPC
pub struct TokenApi<'a> {
    client: &'a Client,
}

impl<'a> TokenApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get ERC-20 token balances for an address
    ///
    /// # Arguments
    /// * `address` - Wallet address to query
    ///
    /// # Example
    /// ```ignore
    /// let balances = client.token().get_token_balances("0x123...").await?;
    /// ```
    pub async fn get_token_balances(&self, address: &str) -> Result<RpcTokenBalancesResponse> {
        self.get_token_balances_with_spec(address, TokenSpec::Erc20, None)
            .await
    }

    /// Get token balances for an address with specific token spec
    ///
    /// # Arguments
    /// * `address` - Wallet address to query
    /// * `token_spec` - Which tokens to query
    /// * `options` - Pagination options
    pub async fn get_token_balances_with_spec(
        &self,
        address: &str,
        token_spec: TokenSpec,
        options: Option<RpcTokenBalancesOptions>,
    ) -> Result<RpcTokenBalancesResponse> {
        let mut params = vec![
            serde_json::json!(address),
            serde_json::to_value(&token_spec)?,
        ];

        if let Some(opts) = options {
            params.push(serde_json::to_value(&opts)?);
        }

        self.client.rpc("alchemy_getTokenBalances", params).await
    }

    /// Get token balances for specific token addresses
    ///
    /// # Arguments
    /// * `address` - Wallet address to query
    /// * `token_addresses` - Token contract addresses to query
    pub async fn get_token_balances_for_tokens(
        &self,
        address: &str,
        token_addresses: &[&str],
    ) -> Result<RpcTokenBalancesResponse> {
        let token_spec =
            TokenSpec::Addresses(token_addresses.iter().map(|a| a.to_string()).collect());
        self.get_token_balances_with_spec(address, token_spec, None)
            .await
    }

    /// Get metadata for a token contract
    ///
    /// # Arguments
    /// * `contract_address` - Token contract address
    ///
    /// # Example
    /// ```ignore
    /// let metadata = client.token().get_token_metadata("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").await?;
    /// println!("Token: {} ({})", metadata.name.unwrap(), metadata.symbol.unwrap());
    /// ```
    pub async fn get_token_metadata(&self, contract_address: &str) -> Result<RpcTokenMetadata> {
        self.client
            .rpc("alchemy_getTokenMetadata", vec![contract_address])
            .await
    }

    /// Get token allowance for a spender
    ///
    /// # Arguments
    /// * `contract_address` - Token contract address
    /// * `owner` - Token owner address
    /// * `spender` - Spender address to check allowance for
    ///
    /// # Returns
    /// Allowance amount as a hex-encoded string
    pub async fn get_token_allowance(
        &self,
        contract_address: &str,
        owner: &str,
        spender: &str,
    ) -> Result<String> {
        let params = serde_json::json!({
            "contract": contract_address,
            "owner": owner,
            "spender": spender
        });
        self.client
            .rpc("alchemy_getTokenAllowance", vec![params])
            .await
    }
}
