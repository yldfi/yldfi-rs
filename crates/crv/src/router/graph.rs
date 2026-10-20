//! Route graph building from pool data
//!
//! Constructs a graph of swap routes by analyzing pool data and creating
//! edges between tokens that can be swapped.

use super::types::{
    eth_wrapper_pairs, PoolType, RouteGraph, RouteStep, SwapParams, SwapType, MIN_TVL_USD,
};
use crate::pools::Pool;

/// Build a route graph from a list of pools
#[must_use]
pub fn build_graph(chain: &str, pools: &[Pool]) -> RouteGraph {
    let mut graph = RouteGraph::new(chain);

    // Add wrapper pairs for special swap type 8 (ETH <-> WETH, etc.)
    if chain.to_lowercase() == "ethereum" || chain.to_lowercase() == "eth" {
        add_wrapper_edges(&mut graph);
    }

    // Add edges from pools
    for pool in pools {
        add_pool_edges(&mut graph, pool);
    }

    graph
}

/// Add edges for wrapper token pairs (ETH/WETH, stETH/wstETH, etc.)
fn add_wrapper_edges(graph: &mut RouteGraph) {
    for pair in eth_wrapper_pairs() {
        // Create a synthetic "pool" for wrapper operations
        let wrapper_address = pair.token_b.clone(); // Use wrapped token as "pool"

        // A -> B (wrap)
        let step_wrap = RouteStep::new(
            format!("wrap_{}", pair.symbol_b),
            &wrapper_address,
            &pair.token_a,
            &pair.token_b,
            SwapParams::new(0, 0, SwapType::Wrap, PoolType::Stable, 2),
            f64::MAX, // Infinite liquidity for wrapping
        );
        graph.add_edge(&pair.token_a, &pair.token_b, step_wrap);

        // B -> A (unwrap)
        let step_unwrap = RouteStep::new(
            format!("unwrap_{}", pair.symbol_b),
            &wrapper_address,
            &pair.token_b,
            &pair.token_a,
            SwapParams::new(0, 0, SwapType::Wrap, PoolType::Stable, 2),
            f64::MAX, // Infinite liquidity for unwrapping
        );
        graph.add_edge(&pair.token_b, &pair.token_a, step_unwrap);
    }
}

/// Extract lowercase coin addresses from a pool's `coins`/`underlyingCoins` array
fn extract_addresses(coins: Option<&Vec<serde_json::Value>>) -> Vec<String> {
    coins
        .map(|coins| {
            coins
                .iter()
                .filter_map(|c| c.get("address").and_then(|a| a.as_str()))
                .map(str::to_lowercase)
                .collect()
        })
        .unwrap_or_default()
}

/// Add edges from a single pool
fn add_pool_edges(graph: &mut RouteGraph, pool: &Pool) {
    // Get pool TVL, skip if too low
    let tvl = pool.usd_total().unwrap_or(0.0);
    if tvl < MIN_TVL_USD {
        return;
    }

    let pool_id = pool.id().unwrap_or_default().to_string();
    let pool_address = pool.address().unwrap_or_default().to_string();
    let lp_token = pool.lp_token_address().unwrap_or_default().to_string();

    // Extract coin addresses
    let coin_addresses = extract_addresses(pool.coins());
    if coin_addresses.is_empty() {
        return;
    }

    let n_coins = coin_addresses.len() as u8;
    let pool_type = determine_pool_type(pool, coin_addresses.len());

    // Swaps between the pool's own `coins` (including a metapool's base LP
    // token) are always a plain `exchange` (swap_type 1)
    add_exchange_edges(
        graph,
        &pool_id,
        &pool_address,
        &coin_addresses,
        SwapType::Exchange,
        pool_type,
        tvl,
    );

    // Underlying coins of stable metapools are swapped with the pool's own
    // `exchange_underlying` (swap_type 2, indices into `underlyingCoins`).
    // Crypto metapools need the zap plus the router's `_pools` argument
    // (swap_type 3), which the encoded `exchange`/`get_dy` calls don't pass,
    // so their underlying coins are not routed.
    if pool.is_meta_pool().unwrap_or(false) && pool_type.is_stable() {
        let underlying =
            extract_addresses(pool.raw().get("underlyingCoins").and_then(|v| v.as_array()));
        if underlying.len() > coin_addresses.len() {
            add_exchange_edges(
                graph,
                &format!("{pool_id}_underlying"),
                &pool_address,
                &underlying,
                SwapType::ExchangeUnderlying,
                pool_type,
                tvl,
            );
        }
    }

    // Add liquidity edges (coin -> LP token, LP token -> coin)
    if !lp_token.is_empty() {
        let lp_token_lower = lp_token.to_lowercase();

        for (i, coin) in coin_addresses.iter().enumerate() {
            // Coin -> LP (add_liquidity)
            let step_add = RouteStep::new(
                format!("{pool_id}_add_{i}"),
                &pool_address,
                coin,
                &lp_token_lower,
                SwapParams::new(i as u8, 0, SwapType::AddLiquidity, pool_type, n_coins),
                tvl,
            );
            graph.add_edge(coin, &lp_token_lower, step_add);

            // LP -> Coin (remove_liquidity_one_coin)
            let step_remove = RouteStep::new(
                format!("{pool_id}_remove_{i}"),
                &pool_address,
                &lp_token_lower,
                coin,
                SwapParams::new(0, i as u8, SwapType::RemoveLiquidity, pool_type, n_coins),
                tvl,
            );
            graph.add_edge(&lp_token_lower, coin, step_remove);
        }
    }
}

/// Add an edge for every ordered pair of `coins`, with `i`/`j` being the
/// positions in `coins`
fn add_exchange_edges(
    graph: &mut RouteGraph,
    pool_id: &str,
    pool_address: &str,
    coins: &[String],
    swap_type: SwapType,
    pool_type: PoolType,
    tvl: f64,
) {
    let n_coins = coins.len() as u8;
    for (i, coin_i) in coins.iter().enumerate() {
        for (j, coin_j) in coins.iter().enumerate() {
            if i == j {
                continue;
            }
            let step = RouteStep::new(
                pool_id,
                pool_address,
                coin_i,
                coin_j,
                SwapParams::new(i as u8, j as u8, swap_type, pool_type, n_coins),
                tvl,
            );
            graph.add_edge(coin_i, coin_j, step);
        }
    }
}

/// Determine the router `pool_type` from the pool's `registryId`
///
/// The pools API has no per-pool crypto/NG flag, so the registry is the
/// signal: `factory-stable-ng`/`factory-twocrypto`/`factory-tricrypto` are the
/// NG factories, `crypto`/`factory-crypto` are crypto v1 (two- or three-coin),
/// everything else (`main`, `factory`, `factory-crvusd`, ...) is stableswap.
fn determine_pool_type(pool: &Pool, n_coins: usize) -> PoolType {
    let raw = pool.raw();
    let registry_id = raw
        .get("registryId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_lowercase();
    let is_crypto = raw
        .get("isCrypto")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let crypto_v1 = || {
        if n_coins >= 3 {
            PoolType::TriCrypto
        } else {
            PoolType::TwoCrypto
        }
    };

    match registry_id.as_str() {
        "factory-stable-ng" => PoolType::StableNg,
        "factory-twocrypto" => PoolType::TwoCryptoNg,
        "factory-tricrypto" => PoolType::TriCryptoNg,
        "crypto" | "factory-crypto" => crypto_v1(),
        _ if is_crypto => crypto_v1(),
        _ => PoolType::Stable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_pool(id: &str, address: &str, coins: Vec<(&str, &str)>, tvl: f64) -> Pool {
        let coins_json: Vec<_> = coins
            .iter()
            .map(|(addr, symbol)| {
                json!({
                    "address": addr,
                    "symbol": symbol
                })
            })
            .collect();

        Pool(json!({
            "id": id,
            "address": address,
            "lpTokenAddress": format!("0xlp_{}", id),
            "coins": coins_json,
            "usdTotal": tvl,
            "isCrypto": false,
            "isFactory": false,
            "isLending": false,
            "isMetaPool": false
        }))
    }

    #[test]
    fn test_build_graph_single_pool() {
        let pools = vec![make_pool(
            "3pool",
            "0x3pool",
            vec![("0xdai", "DAI"), ("0xusdc", "USDC"), ("0xusdt", "USDT")],
            10_000_000.0,
        )];

        let graph = build_graph("ethereum", &pools);

        // Should have DAI, USDC, USDT, and LP token
        assert!(graph.has_token("0xdai"));
        assert!(graph.has_token("0xusdc"));
        assert!(graph.has_token("0xusdt"));
        assert!(graph.has_token("0xlp_3pool"));

        // DAI should have edges to USDC, USDT, and LP
        let dai_edges = graph.get_edges("0xdai").unwrap();
        assert!(dai_edges.len() >= 3);
    }

    #[test]
    fn test_skip_low_tvl_pools() {
        let pools = vec![make_pool(
            "low_tvl",
            "0xlow",
            vec![("0xa", "A"), ("0xb", "B")],
            100.0, // Below MIN_TVL_USD
        )];

        let graph = build_graph("ethereum", &pools);

        // Pool coins shouldn't be in graph (only wrapper pairs)
        // Actually wrapper pairs are added, so we need to check for the specific pool tokens
        assert!(!graph.has_token("0xa"));
        assert!(!graph.has_token("0xb"));
    }

    fn edge<'a>(graph: &'a RouteGraph, from: &str, to: &str) -> &'a RouteStep {
        &graph
            .get_edges(from)
            .unwrap()
            .iter()
            .find(|e| e.step.output_coin == to)
            .unwrap()
            .step
    }

    #[test]
    fn test_pool_type_from_registry() {
        let cases = [
            ("main", 3, PoolType::Stable),
            ("factory", 2, PoolType::Stable),
            ("factory-crvusd", 2, PoolType::Stable),
            ("factory-stable-ng", 2, PoolType::StableNg),
            ("factory-twocrypto", 2, PoolType::TwoCryptoNg),
            ("factory-tricrypto", 3, PoolType::TriCryptoNg),
            ("factory-crypto", 2, PoolType::TwoCrypto),
            ("crypto", 2, PoolType::TwoCrypto),
            ("crypto", 3, PoolType::TriCrypto),
        ];
        for (registry, n, expected) in cases {
            let pool = Pool(json!({ "registryId": registry }));
            assert_eq!(determine_pool_type(&pool, n), expected, "{registry}");
        }

        let pools = vec![Pool(json!({
            "id": "factory-tricrypto-0",
            "address": "0xtri",
            "coins": [{"address": "0xa"}, {"address": "0xb"}, {"address": "0xc"}],
            "usdTotal": 1_000_000.0,
            "registryId": "factory-tricrypto"
        }))];
        let graph = build_graph("ethereum", &pools);
        assert_eq!(
            edge(&graph, "0xa", "0xc").swap_params.to_array(),
            [0, 2, 1, 30, 3]
        );
    }

    #[test]
    fn test_metapool_swap_types() {
        let pools = vec![Pool(json!({
            "id": "factory-v2-0",
            "address": "0xmeta",
            "lpTokenAddress": "0xmeta",
            "registryId": "factory",
            "isMetaPool": true,
            "usdTotal": 1_000_000.0,
            "coins": [{"address": "0xUSDX"}, {"address": "0xBaseLp"}],
            "underlyingCoins": [
                {"address": "0xUSDX"},
                {"address": "0xDai"},
                {"address": "0xUsdc"},
                {"address": "0xUsdt"}
            ]
        }))];
        let graph = build_graph("ethereum", &pools);

        // Direct `coins` swap: plain exchange
        let direct = edge(&graph, "0xusdx", "0xbaselp");
        assert_eq!(direct.swap_params.to_array(), [0, 1, 1, 1, 2]);
        assert_eq!(direct.pool_address, "0xmeta");

        // Underlying swap: exchange_underlying on the metapool itself, with
        // indices into `underlyingCoins`
        let underlying = edge(&graph, "0xusdx", "0xusdc");
        assert_eq!(underlying.swap_params.to_array(), [0, 2, 2, 1, 4]);
        assert_eq!(underlying.pool_address, "0xmeta");
        assert_eq!(
            edge(&graph, "0xdai", "0xusdt").swap_params.swap_type,
            SwapType::ExchangeUnderlying
        );
    }

    #[test]
    fn test_crypto_metapool_underlying_not_routed() {
        let pools = vec![Pool(json!({
            "id": "factory-crypto-0",
            "address": "0xcmeta",
            "registryId": "factory-crypto",
            "isMetaPool": true,
            "usdTotal": 1_000_000.0,
            "coins": [{"address": "0xeurx"}, {"address": "0xbaselp"}],
            "underlyingCoins": [{"address": "0xeurx"}, {"address": "0xdai"}, {"address": "0xusdc"}]
        }))];
        let graph = build_graph("ethereum", &pools);
        assert_eq!(
            edge(&graph, "0xeurx", "0xbaselp").swap_params.to_array(),
            [0, 1, 1, 2, 2]
        );
        assert!(!graph.has_token("0xdai"));
    }

    #[test]
    fn test_sfrxeth_uses_wrap_swap_type() {
        // Router v1.1 handles frxETH <-> sfrxETH under swap_type 8, with the
        // sfrxETH vault as the pool slot
        let graph = build_graph("ethereum", &[]);
        let frxeth = "0x5e8422345238f34275888049021821e8e08caa1f";
        let sfrxeth = "0xac3e018457b222d93114458476f3e3416abbe38f";
        for (from, to) in [(frxeth, sfrxeth), (sfrxeth, frxeth)] {
            let step = edge(&graph, from, to);
            assert_eq!(step.swap_params.swap_type, SwapType::Wrap);
            assert_eq!(step.pool_address, sfrxeth);
        }
    }

    #[test]
    fn test_wrapper_pairs_added() {
        let graph = build_graph("ethereum", &[]);

        // ETH and WETH should be in the graph
        assert!(graph.has_token("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"));
        assert!(graph.has_token("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"));

        // Should have edges between them
        let eth_edges = graph
            .get_edges("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee")
            .unwrap();
        assert!(!eth_edges.is_empty());
    }
}
