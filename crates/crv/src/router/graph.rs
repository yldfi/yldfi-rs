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
            SwapParams::new(0, 0, SwapType::Wrap, PoolType::Main, 2),
            f64::MAX, // Infinite liquidity for wrapping
        );
        graph.add_edge(&pair.token_a, &pair.token_b, step_wrap);

        // B -> A (unwrap)
        let step_unwrap = RouteStep::new(
            format!("unwrap_{}", pair.symbol_b),
            &wrapper_address,
            &pair.token_b,
            &pair.token_a,
            SwapParams::new(0, 0, SwapType::Wrap, PoolType::Main, 2),
            f64::MAX, // Infinite liquidity for unwrapping
        );
        graph.add_edge(&pair.token_b, &pair.token_a, step_unwrap);
    }
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

    // Get coins
    let coins = match pool.coins() {
        Some(c) => c,
        None => return,
    };

    // Extract coin addresses
    let coin_addresses: Vec<String> = coins
        .iter()
        .filter_map(|c| c.get("address").and_then(|a| a.as_str()))
        .map(str::to_lowercase)
        .collect();

    if coin_addresses.is_empty() {
        return;
    }

    // Determine pool type
    let pool_type = determine_pool_type(pool);
    let is_lending = pool
        .raw()
        .get("isLending")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let is_meta = pool.is_meta_pool().unwrap_or(false);

    let n_coins = coin_addresses.len() as u8;

    // Create edges between all coin pairs in the pool
    for (i, coin_i) in coin_addresses.iter().enumerate() {
        for (j, coin_j) in coin_addresses.iter().enumerate() {
            if i == j {
                continue;
            }

            // Determine swap type based on pool characteristics
            let swap_type = if is_lending {
                SwapType::ExchangeUnderlying
            } else if is_meta {
                SwapType::ExchangeUnderlyingZap
            } else {
                SwapType::Exchange
            };

            let step = RouteStep::new(
                &pool_id,
                &pool_address,
                coin_i,
                coin_j,
                SwapParams::new(i as u8, j as u8, swap_type, pool_type, n_coins),
                tvl,
            );

            graph.add_edge(coin_i, coin_j, step);
        }
    }

    // Add liquidity edges (coin -> LP token, LP token -> coin)
    if !lp_token.is_empty() {
        let lp_token_lower = lp_token.to_lowercase();

        for (i, coin) in coin_addresses.iter().enumerate() {
            // Coin -> LP (add_liquidity)
            let add_swap_type = if is_lending {
                SwapType::AddLiquidityUnderlying
            } else {
                SwapType::AddLiquidity
            };

            let step_add = RouteStep::new(
                format!("{pool_id}_add_{i}"),
                &pool_address,
                coin,
                &lp_token_lower,
                SwapParams::new(i as u8, 0, add_swap_type, pool_type, n_coins),
                tvl,
            );
            graph.add_edge(coin, &lp_token_lower, step_add);

            // LP -> Coin (remove_liquidity_one_coin)
            let remove_swap_type = if is_lending {
                SwapType::RemoveLiquidityUnderlying
            } else {
                SwapType::RemoveLiquidity
            };

            let step_remove = RouteStep::new(
                format!("{pool_id}_remove_{i}"),
                &pool_address,
                &lp_token_lower,
                coin,
                SwapParams::new(0, i as u8, remove_swap_type, pool_type, n_coins),
                tvl,
            );
            graph.add_edge(&lp_token_lower, coin, step_remove);
        }
    }
}

/// Determine pool type from pool metadata
fn determine_pool_type(pool: &Pool) -> PoolType {
    let raw = pool.raw();

    // Check various flags in the pool data
    let is_crypto = raw
        .get("isCrypto")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let is_factory = raw
        .get("isFactory")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let is_lending = raw
        .get("isLending")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    // Also check registry ID
    let registry_id = raw.get("registryId").and_then(|v| v.as_str());

    if is_lending {
        PoolType::Lending
    } else if is_factory && is_crypto {
        PoolType::FactoryCrypto
    } else if is_factory {
        PoolType::Factory
    } else if is_crypto {
        PoolType::Crypto
    } else if let Some(reg) = registry_id {
        // Infer from registry ID
        if reg.contains("crypto") {
            PoolType::Crypto
        } else if reg.contains("factory") {
            if reg.contains("crypto") {
                PoolType::FactoryCrypto
            } else {
                PoolType::Factory
            }
        } else {
            PoolType::Main
        }
    } else {
        PoolType::Main
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
