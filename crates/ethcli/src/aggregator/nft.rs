//! NFT aggregation from multiple sources
//!
//! Fetches NFT holdings from Alchemy, Moralis, and Dune SIM in parallel
//! and merges results.

use super::{normalize_chain_for_source, AggregatedResult, SourceResult};
use crate::config::ConfigFile;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// NFT source selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NftSource {
    All,
    Alchemy,
    Moralis,
    DuneSim,
}

impl NftSource {
    pub fn name(&self) -> &'static str {
        match self {
            NftSource::All => "all",
            NftSource::Alchemy => "alchemy",
            NftSource::Moralis => "moralis",
            NftSource::DuneSim => "dsim",
        }
    }

    pub fn all_sources() -> Vec<NftSource> {
        vec![NftSource::Alchemy, NftSource::Moralis, NftSource::DuneSim]
    }
}

/// NFT entry from aggregation (renamed to avoid conflict with normalize.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftEntry {
    /// Contract address (checksummed)
    pub contract_address: String,
    /// Token ID
    pub token_id: String,
    /// Chain (normalized)
    pub chain: String,
    /// NFT name
    pub name: Option<String>,
    /// Collection name
    pub collection_name: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Token type (ERC721, ERC1155)
    pub token_type: Option<String>,
    /// Image URL
    pub image_url: Option<String>,
    /// Thumbnail URL
    pub thumbnail_url: Option<String>,
    /// Metadata URL
    pub metadata_url: Option<String>,
    /// Balance (for ERC1155)
    pub balance: u64,
    /// Floor price in ETH
    pub floor_price_eth: Option<f64>,
    /// Floor price in USD
    pub floor_price_usd: Option<f64>,
    /// Is spam
    pub is_spam: Option<bool>,
    /// Is verified collection
    pub is_verified: Option<bool>,
    /// Sources that found this NFT
    pub found_in: Vec<String>,
}

impl NftEntry {
    pub fn builder() -> NftEntryBuilder {
        NftEntryBuilder::default()
    }

    /// Create unique key for deduplication
    pub fn unique_key(&self) -> String {
        format!(
            "{}:{}:{}",
            self.chain.to_lowercase(),
            self.contract_address.to_lowercase(),
            self.token_id
        )
    }
}

#[derive(Default)]
pub struct NftEntryBuilder {
    contract_address: String,
    token_id: String,
    chain: String,
    name: Option<String>,
    collection_name: Option<String>,
    symbol: Option<String>,
    token_type: Option<String>,
    image_url: Option<String>,
    thumbnail_url: Option<String>,
    metadata_url: Option<String>,
    balance: u64,
    floor_price_eth: Option<f64>,
    floor_price_usd: Option<f64>,
    is_spam: Option<bool>,
    is_verified: Option<bool>,
    found_in: Vec<String>,
}

impl NftEntryBuilder {
    pub fn contract_address(mut self, v: impl Into<String>) -> Self {
        self.contract_address = v.into();
        self
    }

    pub fn token_id(mut self, v: impl Into<String>) -> Self {
        self.token_id = v.into();
        self
    }

    pub fn chain(mut self, v: impl Into<String>) -> Self {
        self.chain = v.into();
        self
    }

    pub fn name(mut self, v: Option<String>) -> Self {
        self.name = v;
        self
    }

    pub fn collection_name(mut self, v: Option<String>) -> Self {
        self.collection_name = v;
        self
    }

    pub fn symbol(mut self, v: Option<String>) -> Self {
        self.symbol = v;
        self
    }

    pub fn token_type(mut self, v: Option<String>) -> Self {
        self.token_type = v;
        self
    }

    pub fn image_url(mut self, v: Option<String>) -> Self {
        self.image_url = v;
        self
    }

    pub fn thumbnail_url(mut self, v: Option<String>) -> Self {
        self.thumbnail_url = v;
        self
    }

    pub fn metadata_url(mut self, v: Option<String>) -> Self {
        self.metadata_url = v;
        self
    }

    pub fn balance(mut self, v: u64) -> Self {
        self.balance = v;
        self
    }

    pub fn floor_price_eth(mut self, v: Option<f64>) -> Self {
        self.floor_price_eth = v;
        self
    }

    pub fn floor_price_usd(mut self, v: Option<f64>) -> Self {
        self.floor_price_usd = v;
        self
    }

    pub fn is_spam(mut self, v: Option<bool>) -> Self {
        self.is_spam = v;
        self
    }

    pub fn is_verified(mut self, v: Option<bool>) -> Self {
        self.is_verified = v;
        self
    }

    pub fn source(mut self, v: impl Into<String>) -> Self {
        self.found_in.push(v.into());
        self
    }

    pub fn build(self) -> NftEntry {
        NftEntry {
            contract_address: self.contract_address,
            token_id: self.token_id,
            chain: self.chain,
            name: self.name,
            collection_name: self.collection_name,
            symbol: self.symbol,
            token_type: self.token_type,
            image_url: self.image_url,
            thumbnail_url: self.thumbnail_url,
            metadata_url: self.metadata_url,
            balance: self.balance,
            floor_price_eth: self.floor_price_eth,
            floor_price_usd: self.floor_price_usd,
            is_spam: self.is_spam,
            is_verified: self.is_verified,
            found_in: self.found_in,
        }
    }
}

/// Aggregated NFT result (renamed to avoid conflict with normalize.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftResult {
    /// Total NFT count
    pub nft_count: usize,
    /// Merged NFTs (deduplicated by contract+tokenId+chain)
    pub nfts: Vec<NftEntry>,
    /// Chains covered
    pub chains_covered: Vec<String>,
    /// Estimated total value in USD
    pub estimated_value_usd: Option<f64>,
}

/// Fetch NFTs from all sources in parallel
pub async fn fetch_nfts_all(
    address: &str,
    chains: &[&str],
) -> AggregatedResult<Vec<NftEntry>, NftResult> {
    fetch_nfts_parallel(address, chains, &NftSource::all_sources()).await
}

/// Fetch NFTs from specified sources in parallel
pub async fn fetch_nfts_parallel(
    address: &str,
    chains: &[&str],
    sources: &[NftSource],
) -> AggregatedResult<Vec<NftEntry>, NftResult> {
    let start = Instant::now();

    // Build futures for each source
    let futures: Vec<_> = sources
        .iter()
        .map(|source| {
            let address = address.to_string();
            let chains: Vec<String> = chains.iter().map(|s| s.to_string()).collect();
            async move {
                let source_start = Instant::now();
                let result = match source {
                    NftSource::Alchemy => {
                        fetch_nfts_alchemy(
                            &address,
                            &chains.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        )
                        .await
                    }
                    NftSource::Moralis => {
                        fetch_nfts_moralis(
                            &address,
                            &chains.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        )
                        .await
                    }
                    NftSource::DuneSim => {
                        fetch_nfts_dsim(
                            &address,
                            &chains.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
                        )
                        .await
                    }
                    NftSource::All => {
                        // Should not be called directly
                        Err(anyhow::anyhow!("All is not a direct source"))
                    }
                };
                (
                    source.name().to_string(),
                    result,
                    source_start.elapsed().as_millis() as u64,
                )
            }
        })
        .collect();

    // Execute all in parallel
    let results = join_all(futures).await;

    // Build source results and collect successful NFT lists
    let mut source_results = Vec::new();
    let mut all_nfts: Vec<NftEntry> = Vec::new();

    for (source_name, result, latency) in results {
        match result {
            Ok(nfts) => {
                all_nfts.extend(nfts.clone());
                source_results.push(SourceResult {
                    source: source_name,
                    data: Some(nfts),
                    raw: None,
                    error: None,
                    latency_ms: latency,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                });
            }
            Err(e) => {
                source_results.push(SourceResult {
                    source: source_name,
                    data: None,
                    raw: None,
                    error: Some(e.to_string()),
                    latency_ms: latency,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                });
            }
        }
    }

    // Merge and deduplicate NFTs
    let merged_nfts = merge_nfts(all_nfts);

    // Calculate aggregation
    let chains_covered: Vec<String> = merged_nfts
        .iter()
        .map(|n| n.chain.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let estimated_value: f64 = merged_nfts.iter().filter_map(|n| n.floor_price_usd).sum();

    let aggregation = NftResult {
        nft_count: merged_nfts.len(),
        nfts: merged_nfts,
        chains_covered,
        estimated_value_usd: if estimated_value > 0.0 {
            Some(estimated_value)
        } else {
            None
        },
    };

    let sources_succeeded = source_results.iter().filter(|s| s.is_success()).count();

    AggregatedResult {
        aggregated: aggregation,
        sources: source_results,
        sources_queried: sources.len(),
        sources_succeeded,
        total_latency_ms: start.elapsed().as_millis() as u64,
    }
}

/// Merge NFTs from multiple sources, deduplicating by contract+tokenId+chain
fn merge_nfts(nfts: Vec<NftEntry>) -> Vec<NftEntry> {
    let mut merged: HashMap<String, NftEntry> = HashMap::new();

    for nft in nfts {
        let key = nft.unique_key();

        if let Some(existing) = merged.get_mut(&key) {
            // Merge sources
            for source in &nft.found_in {
                if !existing.found_in.contains(source) {
                    existing.found_in.push(source.clone());
                }
            }

            // Merge optional fields (prefer non-None values)
            if existing.name.is_none() && nft.name.is_some() {
                existing.name = nft.name;
            }
            if existing.collection_name.is_none() && nft.collection_name.is_some() {
                existing.collection_name = nft.collection_name;
            }
            if existing.image_url.is_none() && nft.image_url.is_some() {
                existing.image_url = nft.image_url;
            }
            if existing.thumbnail_url.is_none() && nft.thumbnail_url.is_some() {
                existing.thumbnail_url = nft.thumbnail_url;
            }
            if existing.floor_price_eth.is_none() && nft.floor_price_eth.is_some() {
                existing.floor_price_eth = nft.floor_price_eth;
            }
            if existing.floor_price_usd.is_none() && nft.floor_price_usd.is_some() {
                existing.floor_price_usd = nft.floor_price_usd;
            }
            if existing.is_spam.is_none() && nft.is_spam.is_some() {
                existing.is_spam = nft.is_spam;
            }
            if existing.is_verified.is_none() && nft.is_verified.is_some() {
                existing.is_verified = nft.is_verified;
            }

            // Take higher balance (for ERC1155)
            if nft.balance > existing.balance {
                existing.balance = nft.balance;
            }
        } else {
            merged.insert(key, nft);
        }
    }

    // Sort by estimated value (highest first), then by collection name
    let mut result: Vec<NftEntry> = merged.into_values().collect();
    result.sort_by(|a, b| {
        let val_cmp = b
            .floor_price_usd
            .unwrap_or(0.0)
            .partial_cmp(&a.floor_price_usd.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal);
        if val_cmp == std::cmp::Ordering::Equal {
            a.collection_name.cmp(&b.collection_name)
        } else {
            val_cmp
        }
    });

    result
}

/// Convert chain name to Alchemy Network enum
fn chain_to_alchemy_network(chain: &str) -> Option<alcmy::Network> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" | "eth-mainnet" => Some(alcmy::Network::EthMainnet),
        "polygon" | "matic" | "polygon-mainnet" => Some(alcmy::Network::PolygonMainnet),
        "arbitrum" | "arb" | "arb-mainnet" => Some(alcmy::Network::ArbitrumMainnet),
        "optimism" | "opt" | "opt-mainnet" => Some(alcmy::Network::OptMainnet),
        "base" | "base-mainnet" => Some(alcmy::Network::BaseMainnet),
        "zksync" | "zksync-mainnet" => Some(alcmy::Network::ZksyncMainnet),
        "linea" | "linea-mainnet" => Some(alcmy::Network::LineaMainnet),
        "scroll" | "scroll-mainnet" => Some(alcmy::Network::ScrollMainnet),
        "blast" | "blast-mainnet" => Some(alcmy::Network::BlastMainnet),
        "mantle" | "mantle-mainnet" => Some(alcmy::Network::MantleMainnet),
        "zora" | "zora-mainnet" => Some(alcmy::Network::ZoraMainnet),
        "bnb" | "bsc" | "bnb-mainnet" => Some(alcmy::Network::Bnb),
        "avalanche" | "avax" | "avax-mainnet" => Some(alcmy::Network::Avalanche),
        "fantom" | "ftm" | "fantom-mainnet" => Some(alcmy::Network::Fantom),
        "gnosis" | "xdai" | "gnosis-mainnet" => Some(alcmy::Network::Gnosis),
        _ => None,
    }
}

/// Fetch NFTs from Alchemy
async fn fetch_nfts_alchemy(address: &str, chains: &[&str]) -> anyhow::Result<Vec<NftEntry>> {
    // Get API key from config first, then fall back to env var
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| c.alchemy.as_ref())
        .map(|a| a.api_key.clone())
        .or_else(|| std::env::var("ALCHEMY_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("ALCHEMY_API_KEY not set in config or environment"))?;

    let mut all_nfts = Vec::new();

    for chain in chains {
        let network = match chain_to_alchemy_network(chain) {
            Some(n) => n,
            None => {
                eprintln!("Alchemy: unsupported chain '{}'", chain);
                continue;
            }
        };

        let client = match alcmy::Client::new(&api_key, network) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Alchemy: Failed to create client for '{}': {}", chain, e);
                continue;
            }
        };

        match client.nft().get_nfts_for_owner(address).await {
            Ok(response) => {
                for nft in response.owned_nfts {
                    // Get image URL from various sources
                    let image_url = nft
                        .image
                        .as_ref()
                        .and_then(|i| i.cached_url.clone().or(i.original_url.clone()));
                    let thumbnail_url = nft.image.as_ref().and_then(|i| i.thumbnail_url.clone());

                    // Get floor price from contract metadata
                    let floor_price = nft
                        .contract
                        .opensea_metadata
                        .as_ref()
                        .and_then(|o| o.floor_price);

                    // Get collection name
                    let collection_name = nft
                        .collection
                        .as_ref()
                        .and_then(|c| c.name.clone())
                        .or_else(|| {
                            nft.contract
                                .opensea_metadata
                                .as_ref()
                                .and_then(|o| o.collection_name.clone())
                        });

                    // Parse balance (default to 1 for ERC721)
                    let balance: u64 = nft
                        .balance
                        .as_ref()
                        .and_then(|b: &String| b.parse::<u64>().ok())
                        .unwrap_or(1);

                    // Get token type
                    let token_type = nft
                        .token_type
                        .map(|t| format!("{:?}", t))
                        .or_else(|| nft.contract.token_type.map(|t| format!("{:?}", t)));

                    let normalized = NftEntry::builder()
                        .contract_address(nft.contract.address.clone())
                        .token_id(nft.token_id.clone())
                        .chain(chain.to_string())
                        .name(nft.name.clone())
                        .collection_name(collection_name)
                        .symbol(nft.contract.symbol.clone())
                        .token_type(token_type)
                        .image_url(image_url)
                        .thumbnail_url(thumbnail_url)
                        .metadata_url(nft.token_uri.clone())
                        .balance(balance)
                        .floor_price_eth(floor_price)
                        .is_spam(None) // Alchemy requires separate spam check
                        .is_verified(None)
                        .source("alchemy")
                        .build();

                    all_nfts.push(normalized);
                }
            }
            Err(e) => {
                eprintln!("Alchemy NFT fetch error for {}: {}", chain, e);
            }
        }
    }

    Ok(all_nfts)
}

/// Fetch NFTs from Moralis
async fn fetch_nfts_moralis(address: &str, chains: &[&str]) -> anyhow::Result<Vec<NftEntry>> {
    // Get API key from config first, then fall back to env var
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| c.moralis.as_ref())
        .map(|m| m.api_key.clone())
        .or_else(|| std::env::var("MORALIS_API_KEY").ok())
        .ok_or_else(|| anyhow::anyhow!("MORALIS_API_KEY not set in config or environment"))?;

    let client = mrls::Client::new(&api_key)?;
    let mut all_nfts = Vec::new();

    for chain in chains {
        let network = normalize_chain_for_source("moralis", chain);
        let query = mrls::nft::NftQuery::new().chain(&network);

        match client.nft().get_wallet_nfts(address, Some(&query)).await {
            Ok(response) => {
                for nft in response.result {
                    // Parse balance
                    let balance: u64 = nft
                        .amount
                        .as_ref()
                        .and_then(|b: &String| b.parse::<u64>().ok())
                        .unwrap_or(1);

                    let normalized = NftEntry::builder()
                        .contract_address(nft.token_address.clone().unwrap_or_default())
                        .token_id(nft.token_id.clone().unwrap_or_default())
                        .chain(chain.to_string())
                        .name(nft.name.clone())
                        .collection_name(nft.name.clone()) // Moralis uses same field
                        .symbol(nft.symbol.clone())
                        .token_type(nft.contract_type.clone())
                        .image_url(None) // Moralis requires metadata parsing
                        .thumbnail_url(None)
                        .metadata_url(nft.token_uri.clone())
                        .balance(balance)
                        .floor_price_eth(nft.floor_price)
                        .floor_price_usd(nft.floor_price_usd)
                        .is_spam(nft.possible_spam)
                        .is_verified(nft.verified_collection)
                        .source("moralis")
                        .build();

                    all_nfts.push(normalized);
                }
            }
            Err(e) => {
                eprintln!("Moralis NFT fetch error for {}: {}", chain, e);
            }
        }
    }

    Ok(all_nfts)
}

/// Fetch NFTs from Dune SIM (Collectibles API)
async fn fetch_nfts_dsim(address: &str, chains: &[&str]) -> anyhow::Result<Vec<NftEntry>> {
    // Get API key from config first, then fall back to env var
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| {
            c.dune_sim
                .as_ref()
                .map(|d| d.api_key.clone())
                .or_else(|| c.dune.as_ref().map(|d| d.api_key.clone()))
        })
        .or_else(|| {
            std::env::var("DUNE_SIM_API_KEY")
                .or_else(|_| std::env::var("DUNE_API_KEY"))
                .ok()
        })
        .ok_or_else(|| anyhow::anyhow!("DUNE_SIM_API_KEY not set in config or environment"))?;

    let client = dnsim::Client::new(&api_key)?;
    let mut all_nfts = Vec::new();

    // Dune SIM doesn't support chain filter in the same way, it returns all chains
    // We'll filter by requested chains after fetching
    let requested_chains: std::collections::HashSet<String> = chains
        .iter()
        .map(|c| normalize_chain_for_source("dsim", c).to_lowercase())
        .collect();

    // Dune SIM collectibles endpoint - fetches all chains, we filter client-side
    match client.collectibles().get(address).await {
        Ok(response) => {
            for collectible in response.entries {
                // Filter by requested chains
                let collectible_chain = collectible.chain.to_lowercase();
                if !requested_chains.is_empty() && !requested_chains.contains(&collectible_chain) {
                    continue;
                }

                // Parse balance (it's a String in the response)
                let balance: u64 = collectible.balance.parse().unwrap_or(1);

                let normalized = NftEntry::builder()
                    .contract_address(collectible.contract_address.clone())
                    .token_id(collectible.token_id.clone())
                    .chain(collectible.chain.clone())
                    .name(collectible.name.clone())
                    .collection_name(None) // Not provided in Dune SIM response
                    .symbol(collectible.symbol.clone())
                    .token_type(Some(collectible.token_standard.clone()))
                    .image_url(collectible.image_url.clone())
                    .thumbnail_url(None) // Not provided in Dune SIM response
                    .metadata_url(None)
                    .balance(balance)
                    .floor_price_eth(None)
                    .floor_price_usd(None) // Not provided in Dune SIM response
                    .is_spam(Some(collectible.is_spam))
                    .is_verified(None)
                    .source("dsim")
                    .build();

                all_nfts.push(normalized);
            }
        }
        Err(e) => {
            eprintln!("Dune SIM collectibles fetch error: {}", e);
        }
    }

    Ok(all_nfts)
}
