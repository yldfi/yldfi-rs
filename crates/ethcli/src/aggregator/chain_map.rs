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
        _ => chain_lower,
    }
}

/// Chain ID for a chain name or provider slug (`eth-mainnet`, `polygon-pos`)
pub fn chain_id_from_name(chain: &str) -> Option<u64> {
    yldfi_common::Chain::from_name(chain).map(|c| c.id())
}

/// Chains Moralis has stopped serving, with the date support ended.
const MORALIS_DROPPED_CHAINS: &[(&[&str], &str, &str)] = &[
    (
        &["fantom", "ftm", "fantom-mainnet", "250"],
        "Fantom",
        "2026-05-29",
    ),
    (&["moonbeam", "1284"], "Moonbeam", "2026-09-25"),
    (&["moonriver", "1285"], "Moonriver", "2026-09-25"),
    (&["lisk", "1135"], "Lisk", "2026-09-25"),
];

/// Explain why Moralis can't be queried for `chain`, if it has dropped it.
pub fn moralis_dropped_chain(chain: &str) -> Option<String> {
    let chain = chain.to_lowercase();
    MORALIS_DROPPED_CHAINS
        .iter()
        .find(|(aliases, _, _)| aliases.contains(&chain.as_str()))
        .map(|(_, name, date)| format!("Moralis dropped {name} support on {date}"))
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
        assert!(moralis_dropped_chain("Fantom").is_some());
        assert!(moralis_dropped_chain("moonbeam").is_some());
        assert!(moralis_dropped_chain("ethereum").is_none());
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
    fn test_chain_id_from_name() {
        assert_eq!(chain_id_from_name("ethereum"), Some(1));
        assert_eq!(chain_id_from_name("eth-mainnet"), Some(1));
        assert_eq!(chain_id_from_name("polygon-pos"), Some(137));
        assert_eq!(chain_id_from_name("arbitrum-mainnet"), Some(42161));
        assert_eq!(chain_id_from_name("op-mainnet"), Some(10));
        assert_eq!(chain_id_from_name("bnb"), Some(56));
        assert_eq!(chain_id_from_name("gnosis"), Some(100));
        assert_eq!(chain_id_from_name("unknown"), None);
        assert_eq!(chain_id_from_name(""), None);
    }
}
