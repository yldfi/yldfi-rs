//! Token symbol resolution for swap quotes
//!
//! DEX aggregators only accept token addresses, but `ethcli quote` accepts
//! common symbols (ETH, USDC, ...). This module maps symbols to addresses
//! per chain. Native gas tokens resolve to the `0xEeee...EEeE` placeholder.

use super::fetchers::NATIVE_TOKEN;

/// Native gas token symbol(s) for a chain
fn native_symbols(chain_id: u64) -> &'static [&'static str] {
    match chain_id {
        56 => &["BNB"],
        137 => &["POL", "MATIC"],
        43114 => &["AVAX"],
        100 => &["XDAI"],
        250 => &["FTM"],
        5000 => &["MNT"],
        // Ethereum and the ETH-native L2s
        _ => &["ETH"],
    }
}

/// Well-known ERC20 addresses per chain (symbol, address)
fn known_tokens(chain_id: u64) -> &'static [(&'static str, &'static str)] {
    match chain_id {
        1 => &[
            ("WETH", "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"),
            ("USDC", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            ("USDT", "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
            ("DAI", "0x6B175474E89094C44Da98b954EedeAC495271d0F"),
            ("WBTC", "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"),
            ("LINK", "0x514910771AF9Ca656af840dff83E8264EcF986CA"),
            ("UNI", "0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984"),
            ("AAVE", "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9"),
            ("CRV", "0xD533a949740bb3306d119CC777fa900bA034cd52"),
            ("LDO", "0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32"),
            ("WSTETH", "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0"),
        ],
        10 => &[
            ("WETH", "0x4200000000000000000000000000000000000006"),
            ("USDC", "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85"),
            ("USDT", "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58"),
            ("DAI", "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"),
            ("WBTC", "0x68f180fcCe6836688e9084f035309E29Bf0A2095"),
            ("OP", "0x4200000000000000000000000000000000000042"),
        ],
        56 => &[
            ("WBNB", "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c"),
            ("USDC", "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d"),
            ("USDT", "0x55d398326f99059fF775485246999027B3197955"),
            ("WETH", "0x2170Ed0880ac9A755fd29B2688956BD959F933F8"),
        ],
        137 => &[
            ("WPOL", "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),
            ("WMATIC", "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"),
            ("WETH", "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619"),
            ("USDC", "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359"),
            ("USDT", "0xc2132D05D31c914a87C6611C10748AEb04B58e8F"),
            ("DAI", "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063"),
            ("WBTC", "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6"),
        ],
        8453 => &[
            ("WETH", "0x4200000000000000000000000000000000000006"),
            ("USDC", "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            ("DAI", "0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb"),
            ("CBBTC", "0xcbB7C0000aB88B473b1f5aFd9ef808440eed33Bf"),
        ],
        42161 => &[
            ("WETH", "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1"),
            ("USDC", "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"),
            ("USDC.E", "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8"),
            ("USDT", "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9"),
            ("DAI", "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"),
            ("WBTC", "0x2f2a2543B76A4166549F7aaB2e75Bef0aefC5B0f"),
            ("ARB", "0x912CE59144191C1204E64559FE8253a0e49E6548"),
        ],
        43114 => &[
            ("WAVAX", "0xB31f66AA3C1e785363F0875A1B74E27b85FD66c7"),
            ("USDC", "0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E"),
            ("USDT", "0x9702230A8Ea53601f5cD2dc00fDBc13d4dF4A8c7"),
        ],
        _ => &[],
    }
}

/// Resolve a token argument (address or symbol) to an address for `chain_id`.
///
/// Hex addresses are returned unchanged. Symbols are matched
/// case-insensitively; the chain's native gas token resolves to the
/// `0xEeee...EEeE` placeholder that aggregators understand.
///
/// # Errors
///
/// Returns an error if the input is neither a valid address nor a known
/// symbol on this chain.
pub fn resolve_token(input: &str, chain_id: u64) -> anyhow::Result<String> {
    let trimmed = input.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        if trimmed.len() == 42 && trimmed[2..].chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(trimmed.to_string());
        }
        anyhow::bail!("Invalid token address: {trimmed}");
    }

    let upper = trimmed.to_ascii_uppercase();
    if native_symbols(chain_id).contains(&upper.as_str()) {
        return Ok(NATIVE_TOKEN.to_string());
    }
    if let Some((_, addr)) = known_tokens(chain_id).iter().find(|(s, _)| *s == upper) {
        return Ok((*addr).to_string());
    }

    let known: Vec<&str> = native_symbols(chain_id)
        .iter()
        .copied()
        .chain(known_tokens(chain_id).iter().map(|(s, _)| *s))
        .collect();
    anyhow::bail!(
        "Unknown token symbol '{trimmed}' on chain {chain_id}. Use a token address{}",
        if known.len() > 1 {
            format!(" or one of: {}", known.join(", "))
        } else {
            String::new()
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addresses_pass_through() {
        let a = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        assert_eq!(resolve_token(a, 1).unwrap(), a);
        assert!(resolve_token("0x1234", 1).is_err());
    }

    #[test]
    fn symbols_resolve_per_chain() {
        assert_eq!(resolve_token("eth", 1).unwrap(), NATIVE_TOKEN);
        assert_eq!(
            resolve_token("USDC", 1).unwrap(),
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
        assert_eq!(
            resolve_token("usdc", 42161).unwrap(),
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"
        );
        assert_eq!(
            resolve_token("USDC", 8453).unwrap(),
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"
        );
        assert_eq!(resolve_token("ETH", 8453).unwrap(), NATIVE_TOKEN);
        assert_eq!(resolve_token("BNB", 56).unwrap(), NATIVE_TOKEN);
        assert_eq!(resolve_token("POL", 137).unwrap(), NATIVE_TOKEN);
    }

    #[test]
    fn unknown_symbol_errors() {
        let err = resolve_token("NOPE", 1).unwrap_err().to_string();
        assert!(err.contains("Unknown token symbol"), "{err}");
        assert!(resolve_token("USDC", 999_999).is_err());
    }
}
