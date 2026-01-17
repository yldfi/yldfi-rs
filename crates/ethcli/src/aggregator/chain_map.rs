//! Chain name normalization across different API services
//!
//! Each API service uses different chain naming conventions.
//! This module provides mappings to normalize chain names.

/// Normalize chain name for a specific API source
pub fn normalize_chain_for_source(source: &str, chain: &str) -> String {
    let chain_lower = chain.to_lowercase();

    match source.to_lowercase().as_str() {
        "alchemy" | "alcmy" => match chain_lower.as_str() {
            "ethereum" | "eth" | "mainnet" => "eth-mainnet".to_string(),
            "polygon" | "matic" => "polygon-mainnet".to_string(),
            "arbitrum" | "arb" => "arb-mainnet".to_string(),
            "optimism" | "op" => "opt-mainnet".to_string(),
            "base" => "base-mainnet".to_string(),
            "zksync" => "zksync-mainnet".to_string(),
            "linea" => "linea-mainnet".to_string(),
            "scroll" => "scroll-mainnet".to_string(),
            "blast" => "blast-mainnet".to_string(),
            "mantle" => "mantle-mainnet".to_string(),
            "bnb" | "bsc" => "bnb-mainnet".to_string(),
            "avalanche" | "avax" => "avax-mainnet".to_string(),
            "fantom" | "ftm" => "fantom-mainnet".to_string(),
            "gnosis" | "xdai" => "gnosis-mainnet".to_string(),
            _ => chain_lower,
        },
        "moralis" | "mrls" => match chain_lower.as_str() {
            "ethereum" | "mainnet" | "eth-mainnet" => "eth".to_string(),
            "polygon" | "matic" | "polygon-mainnet" => "polygon".to_string(),
            "arbitrum" | "arb" | "arb-mainnet" => "arbitrum".to_string(),
            "optimism" | "op" | "opt-mainnet" => "optimism".to_string(),
            "base" | "base-mainnet" => "base".to_string(),
            "bnb" | "bsc" | "bnb-mainnet" => "bsc".to_string(),
            "avalanche" | "avax" | "avax-mainnet" => "avalanche".to_string(),
            "fantom" | "ftm" | "fantom-mainnet" => "fantom".to_string(),
            "gnosis" | "xdai" | "gnosis-mainnet" => "gnosis".to_string(),
            _ => chain_lower,
        },
        "defillama" | "llama" => match chain_lower.as_str() {
            "eth" | "eth-mainnet" | "mainnet" => "ethereum".to_string(),
            "matic" | "polygon-mainnet" => "polygon".to_string(),
            "arb" | "arb-mainnet" => "arbitrum".to_string(),
            "op" | "opt-mainnet" => "optimism".to_string(),
            "base-mainnet" => "base".to_string(),
            "bnb-mainnet" => "bsc".to_string(),
            "avax" | "avax-mainnet" => "avax".to_string(),
            "ftm" | "fantom-mainnet" => "fantom".to_string(),
            "xdai" | "gnosis-mainnet" => "gnosis".to_string(),
            _ => chain_lower,
        },
        "coingecko" | "gecko" => match chain_lower.as_str() {
            "eth" | "eth-mainnet" | "mainnet" => "ethereum".to_string(),
            "polygon" | "matic" | "polygon-mainnet" => "polygon-pos".to_string(),
            "arbitrum" | "arb" | "arb-mainnet" => "arbitrum-one".to_string(),
            "op" | "opt-mainnet" => "optimistic-ethereum".to_string(),
            "base-mainnet" => "base".to_string(),
            "bnb" | "bnb-mainnet" => "binance-smart-chain".to_string(),
            "avax" | "avax-mainnet" => "avalanche".to_string(),
            "ftm" | "fantom-mainnet" => "fantom".to_string(),
            "xdai" | "gnosis-mainnet" => "xdai".to_string(),
            _ => chain_lower,
        },
        "curve" | "crv" => match chain_lower.as_str() {
            "eth" | "eth-mainnet" | "mainnet" => "ethereum".to_string(),
            "matic" | "polygon-mainnet" => "polygon".to_string(),
            "arb" | "arb-mainnet" => "arbitrum".to_string(),
            "op" | "opt-mainnet" => "optimism".to_string(),
            "base-mainnet" => "base".to_string(),
            "avax" | "avax-mainnet" => "avalanche".to_string(),
            "ftm" | "fantom-mainnet" => "fantom".to_string(),
            "xdai" | "gnosis-mainnet" => "gnosis".to_string(),
            _ => chain_lower,
        },
        "dunesim" | "dsim" => match chain_lower.as_str() {
            "eth-mainnet" | "mainnet" => "ethereum".to_string(),
            "polygon-mainnet" | "matic" => "polygon".to_string(),
            "arb-mainnet" | "arb" => "arbitrum".to_string(),
            "opt-mainnet" | "op" => "optimism".to_string(),
            "base-mainnet" => "base".to_string(),
            "bnb-mainnet" => "bnb".to_string(),
            _ => chain_lower,
        },
        _ => chain_lower,
    }
}

/// Get the chain ID for a given chain name
pub fn chain_name_to_id(chain: &str) -> Option<u64> {
    let chain_lower = chain.to_lowercase();
    match chain_lower.as_str() {
        "ethereum" | "eth" | "eth-mainnet" | "mainnet" => Some(1),
        "polygon" | "matic" | "polygon-mainnet" | "polygon-pos" => Some(137),
        "arbitrum" | "arb" | "arb-mainnet" | "arbitrum-one" => Some(42161),
        "optimism" | "op" | "opt-mainnet" | "optimistic-ethereum" => Some(10),
        "base" | "base-mainnet" => Some(8453),
        "zksync" | "zksync-mainnet" => Some(324),
        "linea" | "linea-mainnet" => Some(59144),
        "scroll" | "scroll-mainnet" => Some(534352),
        "blast" | "blast-mainnet" => Some(81457),
        "mantle" | "mantle-mainnet" => Some(5000),
        "bnb" | "bsc" | "bnb-mainnet" | "binance-smart-chain" => Some(56),
        "avalanche" | "avax" | "avax-mainnet" => Some(43114),
        "fantom" | "ftm" | "fantom-mainnet" => Some(250),
        "gnosis" | "xdai" | "gnosis-mainnet" => Some(100),
        _ => None,
    }
}

/// Get canonical chain name from chain ID
pub fn chain_id_to_name(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        1 => Some("ethereum"),
        137 => Some("polygon"),
        42161 => Some("arbitrum"),
        10 => Some("optimism"),
        8453 => Some("base"),
        324 => Some("zksync"),
        59144 => Some("linea"),
        534352 => Some("scroll"),
        81457 => Some("blast"),
        5000 => Some("mantle"),
        56 => Some("bnb"),
        43114 => Some("avalanche"),
        250 => Some("fantom"),
        100 => Some("gnosis"),
        _ => None,
    }
}

/// Normalize an address to checksummed format
pub fn normalize_address(address: &str) -> String {
    // Simple implementation - just lowercase with 0x prefix
    // In production, you'd want to use proper EIP-55 checksumming
    let addr = address.trim();
    if addr.starts_with("0x") || addr.starts_with("0X") {
        format!("0x{}", &addr[2..].to_lowercase())
    } else {
        format!("0x{}", addr.to_lowercase())
    }
}

/// Convert a DefiLlama coin identifier to an address
/// DefiLlama uses format like "ethereum:0x..." or "polygon:0x..."
pub fn llama_coin_to_address(coin: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = coin.split(':').collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), normalize_address(parts[1])))
    } else {
        None
    }
}

/// Build a DefiLlama coin identifier from chain and address
pub fn address_to_llama_coin(chain: &str, address: &str) -> String {
    let normalized_chain = normalize_chain_for_source("llama", chain);
    format!("{}:{}", normalized_chain, normalize_address(address))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_chain_for_alchemy() {
        assert_eq!(
            normalize_chain_for_source("alchemy", "ethereum"),
            "eth-mainnet"
        );
        assert_eq!(
            normalize_chain_for_source("alchemy", "polygon"),
            "polygon-mainnet"
        );
        assert_eq!(
            normalize_chain_for_source("alchemy", "arbitrum"),
            "arb-mainnet"
        );
    }

    #[test]
    fn test_normalize_chain_for_moralis() {
        assert_eq!(normalize_chain_for_source("moralis", "ethereum"), "eth");
        assert_eq!(normalize_chain_for_source("moralis", "polygon"), "polygon");
        assert_eq!(normalize_chain_for_source("moralis", "eth-mainnet"), "eth");
    }

    #[test]
    fn test_normalize_chain_for_gecko() {
        assert_eq!(normalize_chain_for_source("gecko", "ethereum"), "ethereum");
        assert_eq!(
            normalize_chain_for_source("gecko", "polygon"),
            "polygon-pos"
        );
        assert_eq!(
            normalize_chain_for_source("gecko", "arbitrum"),
            "arbitrum-one"
        );
    }

    #[test]
    fn test_chain_name_to_id() {
        assert_eq!(chain_name_to_id("ethereum"), Some(1));
        assert_eq!(chain_name_to_id("eth-mainnet"), Some(1));
        assert_eq!(chain_name_to_id("polygon"), Some(137));
        assert_eq!(chain_name_to_id("unknown"), None);
    }

    #[test]
    fn test_normalize_address() {
        assert_eq!(
            normalize_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
        assert_eq!(
            normalize_address("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
    }

    #[test]
    fn test_llama_coin_conversion() {
        let (chain, addr) =
            llama_coin_to_address("ethereum:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        assert_eq!(chain, "ethereum");
        assert_eq!(addr, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");

        let coin =
            address_to_llama_coin("eth-mainnet", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert_eq!(coin, "ethereum:0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    }
}
