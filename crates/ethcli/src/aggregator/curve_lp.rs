//! On-chain value of a Curve LP token
//!
//! The pool's balances, each coin priced in USD, divided by the LP supply
//! (net asset value). Pools are found through Curve's MetaRegistry, which is on
//! Ethereum only. Coins are priced from Chainlink where a feed exists, else
//! from DefiLlama.
//!
//! This is the primary `price --lp` source on Ethereum. Curve's price indexer
//! (prices.curve.finance) is only a cross-check: it was found 2-10% high on
//! crypto-pool LPs, against the pool's own `lp_price()`.

use super::{
    get_cached_config, normalize::NormalizedPrice, price::fetch_price_from_source, LatencyMeasure,
    PriceSource, SourceResult,
};
use alloy::primitives::{address, Address, U256};
use alloy::providers::Provider;
use alloy::sol;

sol! {
    #[sol(rpc)]
    interface ICurveMetaRegistry {
        function get_pool_from_lp_token(address lp) external view returns (address);
        function get_n_coins(address pool) external view returns (uint256);
        function get_coins(address pool) external view returns (address[8]);
        function get_balances(address pool) external view returns (uint256[8]);
        function get_decimals(address pool) external view returns (uint256[8]);
    }

    #[sol(rpc)]
    interface ILpToken {
        function totalSupply() external view returns (uint256);
        function decimals() external view returns (uint8);
    }
}

/// Curve's MetaRegistry on Ethereum
pub const CURVE_META_REGISTRY: Address = address!("F98B45FA17DE75FB1aD0e7aFD971b0ca00e379fC");

/// The address Curve pools use for native ETH
const NATIVE_ETH: Address = address!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");

/// Source name in results
pub const SOURCE: &str = "curve-onchain";

/// Canonical WETH and WBTC on Ethereum
const WETH: Address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
const WBTC: Address = address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");

/// Chainlink feed for a coin whose own address has no feed: native ETH and
/// the canonical wrapped assets, priced at their underlying. Matched by
/// address only: a token's `symbol()` is whatever its deployer chose, so a
/// fake "WETH" must not get ETH's price.
fn underlying_feed(coin: Address) -> Option<&'static str> {
    match coin {
        c if c == NATIVE_ETH || c == WETH => Some("ETH"),
        c if c == WBTC => Some("BTC"),
        _ => None,
    }
}

/// An error's text with URLs and keys removed (an RPC URL can carry an API key)
fn clean(e: impl std::fmt::Display) -> String {
    crate::error::sanitize_error_message(&e.to_string()).into_owned()
}

/// `amount / 10^decimals` as f64
fn scaled(amount: U256, decimals: u32) -> f64 {
    amount.to_string().parse::<f64>().unwrap_or(f64::NAN) / 10f64.powi(decimals as i32)
}

/// Value per LP token from the pool's balances and coin prices
pub fn nav_per_lp(holdings: &[(f64, f64)], supply: f64) -> Option<f64> {
    if !(supply.is_finite() && supply > 0.0) {
        return None;
    }
    let total: f64 = holdings.iter().map(|(amount, price)| amount * price).sum();
    let nav = total / supply;
    (nav.is_finite() && nav > 0.0).then_some(nav)
}

/// USD price of one pool coin: Chainlink by address, then by feed symbol, then DefiLlama
async fn coin_usd<P: Provider + Clone>(provider: &P, coin: Address) -> Result<f64, String> {
    use crate::chainlink;
    let now = crate::utils::unix_timestamp_secs();
    let usd = chainlink::denominations::USD;

    let mut keys = vec![format!("{coin:#x}")];
    if let Some(feed) = underlying_feed(coin) {
        keys.push(feed.to_string());
    }
    for key in &keys {
        if let Ok(data) = chainlink::fetch_price(provider.clone(), key, "ethereum", usd).await {
            if let Some(price) = data.to_fresh_f64(now) {
                return Ok(price);
            }
        }
    }

    let token = if coin == NATIVE_ETH {
        "ETH".to_string()
    } else {
        format!("{coin:#x}")
    };
    let llama = fetch_price_from_source(&token, "ethereum", PriceSource::Llama).await;
    llama
        .data
        .map(|p| p.usd)
        .ok_or_else(|| format!("no price for pool coin {token}"))
}

/// On-chain value of a Curve LP token on Ethereum
pub async fn fetch_curve_lp_onchain(lp_token: &str, chain: &str) -> SourceResult<NormalizedPrice> {
    let measure = LatencyMeasure::start();
    let fail = |msg: String| SourceResult::error(SOURCE, msg, measure.elapsed_ms());

    if crate::chainlink::normalize_chain(chain) != "ethereum" {
        return fail("on-chain LP pricing is available on Ethereum only".to_string());
    }
    let lp: Address = match lp_token.parse() {
        Ok(a) => a,
        Err(_) => return fail(format!("invalid LP token address: {lp_token}")),
    };
    let Some(config) = get_cached_config().as_ref() else {
        return fail("no RPC endpoint configured".to_string());
    };
    let endpoint = match crate::rpc::selector::get_rpc_endpoint_from_config(
        config,
        crate::config::Chain::Ethereum,
    ) {
        Ok(e) => e,
        Err(e) => return fail(format!("no RPC endpoint: {}", clean(e))),
    };
    let provider = endpoint.provider().clone();

    let registry = ICurveMetaRegistry::new(CURVE_META_REGISTRY, &provider);
    let pool = match registry.get_pool_from_lp_token(lp).call().await {
        Ok(p) if p != Address::ZERO => p,
        Ok(_) => return fail("not a Curve LP token known to the MetaRegistry".to_string()),
        Err(e) => return fail(format!("MetaRegistry lookup failed: {}", clean(e))),
    };

    let token = ILpToken::new(lp, &provider);
    let (n, coins, balances, decimals, supply, lp_decimals) = tokio::join!(
        async { registry.get_n_coins(pool).call().await },
        async { registry.get_coins(pool).call().await },
        async { registry.get_balances(pool).call().await },
        async { registry.get_decimals(pool).call().await },
        async { token.totalSupply().call().await },
        async { token.decimals().call().await },
    );
    let (n, coins, balances, decimals, supply, lp_decimals) =
        match (n, coins, balances, decimals, supply, lp_decimals) {
            (Ok(n), Ok(c), Ok(b), Ok(d), Ok(s), Ok(ld)) => (n, c, b, d, s, ld),
            _ => return fail("could not read the pool from the MetaRegistry".to_string()),
        };
    let n = usize::try_from(n).unwrap_or(0).min(8);
    if n == 0 {
        return fail("the MetaRegistry lists no coins for this pool".to_string());
    }

    let prices =
        futures::future::join_all(coins[..n].iter().map(|c| coin_usd(&provider, *c))).await;
    let mut holdings = Vec::with_capacity(n);
    for i in 0..n {
        let price = match &prices[i] {
            Ok(p) => *p,
            Err(e) => return fail(e.clone()),
        };
        let dec = u32::try_from(decimals[i]).unwrap_or(18);
        holdings.push((scaled(balances[i], dec), price));
    }

    match nav_per_lp(&holdings, scaled(supply, u32::from(lp_decimals))) {
        Some(nav) => SourceResult::success(SOURCE, NormalizedPrice::new(nav), measure.elapsed_ms()),
        None => fail("pool value or LP supply is zero".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nav_divides_value_by_supply() {
        // 1,000 USDC + 0.5 ETH at $2,000 behind 100 LP tokens: $20 each
        let nav = nav_per_lp(&[(1000.0, 1.0), (0.5, 2000.0)], 100.0).unwrap();
        assert!((nav - 20.0).abs() < 1e-9);
    }

    #[test]
    fn nav_rejects_empty_or_invalid_supply() {
        assert!(nav_per_lp(&[(1.0, 1.0)], 0.0).is_none());
        assert!(nav_per_lp(&[(1.0, 1.0)], f64::NAN).is_none());
        assert!(nav_per_lp(&[(0.0, 1.0)], 1.0).is_none());
    }

    #[test]
    fn scaled_handles_decimals() {
        assert!((scaled(U256::from(1_500_000u64), 6) - 1.5).abs() < 1e-12);
        assert!((scaled(U256::from(10u64).pow(U256::from(18)), 18) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn only_canonical_wrapped_coins_use_an_underlying_feed() {
        assert_eq!(underlying_feed(WETH), Some("ETH"));
        assert_eq!(underlying_feed(NATIVE_ETH), Some("ETH"));
        assert_eq!(underlying_feed(WBTC), Some("BTC"));
        // Any other address, whatever its symbol() says
        let other = address!("1111111111111111111111111111111111111111");
        assert_eq!(underlying_feed(other), None);
    }

    #[test]
    fn errors_lose_urls_and_keys() {
        let text = clean(
            "error sending request for url (https://eth.example.com/v2/abcdef1234567890abcdef)",
        );
        assert!(!text.contains("abcdef1234567890abcdef"), "{text}");
    }
}
