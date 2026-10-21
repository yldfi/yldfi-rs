//! Portfolio/Data API implementation

use super::types::{
    AddressNetwork, NftContractsByAddressRequest, NftContractsByAddressResponse,
    NftsByAddressRequest, NftsByAddressResponse, TokenBalancesRequest, TokenBalancesResponse,
    TokensByAddressOptions, TokensByAddressRequest, TokensByAddressResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Portfolio API for multi-chain wallet data
pub struct PortfolioApi<'a> {
    client: &'a Client,
}

impl<'a> PortfolioApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token balances for addresses across multiple networks
    ///
    /// Balances only (no metadata or prices); see [`Self::get_tokens_by_address`]
    /// for the richer endpoint. Networks that failed are reported in
    /// `response.error`, and further pages via `response.data.page_key`.
    ///
    /// # Arguments
    /// * `addresses` - List of (address, networks) tuples
    ///
    /// # Example
    /// ```ignore
    /// let balances = client.portfolio().get_token_balances(&[
    ///     ("0x123...", &["eth-mainnet", "polygon-mainnet"]),
    /// ]).await?;
    /// ```
    pub async fn get_token_balances(
        &self,
        addresses: &[(&str, &[&str])],
    ) -> Result<TokenBalancesResponse> {
        self.get_token_balances_with_page_key(addresses, None).await
    }

    /// Get token balances, continuing from a previous response's `pageKey`
    pub async fn get_token_balances_with_page_key(
        &self,
        addresses: &[(&str, &[&str])],
        page_key: Option<&str>,
    ) -> Result<TokenBalancesResponse> {
        let body = TokenBalancesRequest {
            addresses: address_networks(addresses),
            page_key: page_key.map(std::string::ToString::to_string),
            ..Default::default()
        };
        self.client
            .data_post("assets/tokens/balances/by-address", &body)
            .await
    }

    /// Get tokens held by wallets with balances, metadata and prices
    /// (`assets/tokens/by-address`, "Tokens By Wallet")
    ///
    /// # Arguments
    /// * `addresses` - List of (address, networks) tuples
    /// * `options` - Optional flags and pagination cursor
    pub async fn get_tokens_by_address(
        &self,
        addresses: &[(&str, &[&str])],
        options: &TokensByAddressOptions,
    ) -> Result<TokensByAddressResponse> {
        let body = tokens_by_address_request(addresses, options);
        self.client
            .data_post("assets/tokens/by-address", &body)
            .await
    }

    /// Get NFTs owned by addresses across multiple networks
    ///
    /// # Arguments
    /// * `addresses` - List of (address, networks) tuples
    /// * `with_metadata` - Whether to include NFT metadata
    pub async fn get_nfts_by_address(
        &self,
        addresses: &[(&str, &[&str])],
        with_metadata: bool,
    ) -> Result<NftsByAddressResponse> {
        self.get_nfts_by_address_with_options(addresses, with_metadata, None, None)
            .await
    }

    /// Get NFTs owned by addresses with pagination options
    pub async fn get_nfts_by_address_with_options(
        &self,
        addresses: &[(&str, &[&str])],
        with_metadata: bool,
        page_size: Option<u32>,
        page_key: Option<&str>,
    ) -> Result<NftsByAddressResponse> {
        let body = NftsByAddressRequest {
            addresses: address_networks(addresses),
            with_metadata: Some(with_metadata),
            page_size,
            page_key: page_key.map(std::string::ToString::to_string),
        };
        self.client.data_post("assets/nfts/by-address", &body).await
    }

    /// Get NFT contracts owned by addresses across multiple networks
    ///
    /// # Arguments
    /// * `addresses` - List of (address, networks) tuples
    pub async fn get_nft_contracts_by_address(
        &self,
        addresses: &[(&str, &[&str])],
    ) -> Result<NftContractsByAddressResponse> {
        self.get_nft_contracts_by_address_with_options(addresses, None, None)
            .await
    }

    /// Get NFT contracts owned by addresses with pagination options
    pub async fn get_nft_contracts_by_address_with_options(
        &self,
        addresses: &[(&str, &[&str])],
        page_size: Option<u32>,
        page_key: Option<&str>,
    ) -> Result<NftContractsByAddressResponse> {
        let body = NftContractsByAddressRequest {
            addresses: address_networks(addresses),
            with_metadata: None,
            page_size,
            page_key: page_key.map(std::string::ToString::to_string),
        };
        self.client
            .data_post("assets/nfts/contracts/by-address", &body)
            .await
    }
}

fn address_networks(addresses: &[(&str, &[&str])]) -> Vec<AddressNetwork> {
    addresses
        .iter()
        .map(|(addr, networks)| AddressNetwork {
            address: addr.to_string(),
            networks: networks
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
        })
        .collect()
}

fn tokens_by_address_request(
    addresses: &[(&str, &[&str])],
    options: &TokensByAddressOptions,
) -> TokensByAddressRequest {
    TokensByAddressRequest {
        addresses: address_networks(addresses),
        with_metadata: options.with_metadata,
        with_prices: options.with_prices,
        include_native_tokens: options.include_native_tokens,
        include_erc20_tokens: options.include_erc20_tokens,
        include_block_metadata: options.include_block_metadata,
        page_key: options.page_key.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_by_address_request_takes_wallets_not_token_pairs() {
        let options = TokensByAddressOptions {
            with_prices: Some(true),
            page_key: Some("pk".into()),
            ..Default::default()
        };
        let body = serde_json::to_value(tokens_by_address_request(
            &[(
                "0x0000000000000000000000000000000000000001",
                &["eth-mainnet", "base-mainnet"],
            )],
            &options,
        ))
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "addresses": [{
                    "address": "0x0000000000000000000000000000000000000001",
                    "networks": ["eth-mainnet", "base-mainnet"]
                }],
                "withPrices": true,
                "pageKey": "pk"
            })
        );
    }
}
