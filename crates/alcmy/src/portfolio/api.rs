//! Portfolio/Data API implementation

use super::types::*;
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
        let body = TokenBalancesRequest {
            addresses: addresses
                .iter()
                .map(|(addr, networks)| AddressNetwork {
                    address: addr.to_string(),
                    networks: networks.iter().map(|n| n.to_string()).collect(),
                })
                .collect(),
        };
        self.client
            .data_post("assets/tokens/balances/by-address", &body)
            .await
    }

    /// Get token info for multiple tokens
    ///
    /// # Arguments
    /// * `tokens` - List of (network, address) tuples
    pub async fn get_token_info(&self, tokens: &[(&str, &str)]) -> Result<TokenInfoResponse> {
        let body = TokenInfoRequest {
            addresses: tokens
                .iter()
                .map(|(network, address)| TokenAddressInfo {
                    network: network.to_string(),
                    address: address.to_string(),
                })
                .collect(),
        };
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
            addresses: addresses
                .iter()
                .map(|(addr, networks)| AddressNetwork {
                    address: addr.to_string(),
                    networks: networks.iter().map(|n| n.to_string()).collect(),
                })
                .collect(),
            with_metadata: Some(with_metadata),
            page_size,
            page_key: page_key.map(|k| k.to_string()),
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
            addresses: addresses
                .iter()
                .map(|(addr, networks)| AddressNetwork {
                    address: addr.to_string(),
                    networks: networks.iter().map(|n| n.to_string()).collect(),
                })
                .collect(),
            page_size,
            page_key: page_key.map(|k| k.to_string()),
        };
        self.client
            .data_post("assets/nfts/contracts/by-address", &body)
            .await
    }
}
