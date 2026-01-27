//! Route finding algorithm using depth-first search
//!
//! Implements a DFS-based pathfinding algorithm that tracks routes by both
//! minimum TVL and path length to find optimal swap paths.

use super::types::{Route, RouteGraph, RouteStep, MAX_ROUTES_PER_CRITERION, MAX_SEARCH_DEPTH};
use std::collections::HashSet;

/// Found routes with metadata for sorting
#[derive(Debug, Clone)]
struct CandidateRoute {
    route: Route,
    /// Score based on TVL (higher is better)
    tvl_score: f64,
}

impl CandidateRoute {
    fn new(route: Route) -> Self {
        let tvl_score = route.min_tvl;
        Self { route, tvl_score }
    }
}

/// Find all routes between two tokens
#[must_use] 
pub fn find_routes(graph: &RouteGraph, from: &str, to: &str) -> Vec<Route> {
    let from_lower = from.to_lowercase();
    let to_lower = to.to_lowercase();

    // Early exit if tokens don't exist in graph
    if !graph.has_token(&from_lower) || !graph.has_token(&to_lower) {
        return Vec::new();
    }

    // Track best routes by TVL
    let mut best_by_tvl: Vec<CandidateRoute> = Vec::with_capacity(MAX_ROUTES_PER_CRITERION);
    // Track best routes by length (shorter is better)
    let mut best_by_length: Vec<CandidateRoute> = Vec::with_capacity(MAX_ROUTES_PER_CRITERION);

    // DFS state
    let mut visited: HashSet<String> = HashSet::new();
    let mut current_path: Vec<RouteStep> = Vec::new();

    // Start DFS
    visited.insert(from_lower.clone());
    dfs(
        graph,
        &from_lower,
        &to_lower,
        &mut visited,
        &mut current_path,
        &mut best_by_tvl,
        &mut best_by_length,
        0,
    );

    // Merge and deduplicate results
    merge_routes(best_by_tvl, best_by_length)
}

/// Depth-first search for routes
#[allow(clippy::too_many_arguments)]
fn dfs(
    graph: &RouteGraph,
    current: &str,
    target: &str,
    visited: &mut HashSet<String>,
    current_path: &mut Vec<RouteStep>,
    best_by_tvl: &mut Vec<CandidateRoute>,
    best_by_length: &mut Vec<CandidateRoute>,
    depth: usize,
) {
    // Check depth limit
    if depth >= MAX_SEARCH_DEPTH {
        return;
    }

    // Get edges from current token
    let edges = match graph.get_edges(current) {
        Some(e) => e,
        None => return,
    };

    for edge in edges {
        let next_token = edge.step.output_coin.to_lowercase();

        // Check if we found the target
        if next_token == target {
            // Build complete route
            let mut route = Route::new(
                current_path
                    .first().map_or_else(|| current.to_string(), |s| s.input_coin.clone()),
                target.to_string(),
            );

            // Add existing path
            for step in current_path.iter() {
                route.push(step.clone());
            }
            // Add final step
            route.push(edge.step.clone());

            // Try to add to best routes
            let candidate = CandidateRoute::new(route);
            try_add_to_best(best_by_tvl, candidate.clone(), true);
            try_add_to_best(best_by_length, candidate, false);

            continue;
        }

        // Skip if already visited (avoid cycles)
        if visited.contains(&next_token) {
            continue;
        }

        // Prune: skip if this path can't beat existing routes
        if should_prune(current_path, &edge.step, best_by_tvl, depth) {
            continue;
        }

        // Recurse
        visited.insert(next_token.clone());
        current_path.push(edge.step.clone());

        dfs(
            graph,
            &next_token,
            target,
            visited,
            current_path,
            best_by_tvl,
            best_by_length,
            depth + 1,
        );

        current_path.pop();
        visited.remove(&next_token);
    }
}

/// Try to add a candidate route to the best routes list
fn try_add_to_best(best: &mut Vec<CandidateRoute>, candidate: CandidateRoute, by_tvl: bool) {
    if best.len() < MAX_ROUTES_PER_CRITERION {
        best.push(candidate);
        sort_candidates(best, by_tvl);
    } else {
        // Check if better than worst
        let dominated = if by_tvl {
            candidate.tvl_score > best.last().map_or(0.0, |r| r.tvl_score)
        } else {
            candidate.route.len() < best.last().map_or(usize::MAX, |r| r.route.len())
        };

        if dominated {
            best.pop();
            best.push(candidate);
            sort_candidates(best, by_tvl);
        }
    }
}

/// Sort candidates by criterion
fn sort_candidates(candidates: &mut [CandidateRoute], by_tvl: bool) {
    if by_tvl {
        // Higher TVL is better
        candidates.sort_by(|a, b| {
            b.tvl_score
                .partial_cmp(&a.tvl_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        // Shorter path is better
        candidates.sort_by(|a, b| a.route.len().cmp(&b.route.len()));
    }
}

/// Check if current path should be pruned
fn should_prune(
    current_path: &[RouteStep],
    next_step: &RouteStep,
    best_by_tvl: &[CandidateRoute],
    depth: usize,
) -> bool {
    // Don't prune if we don't have enough routes yet
    if best_by_tvl.len() < MAX_ROUTES_PER_CRITERION / 2 {
        return false;
    }

    // Calculate min TVL if we take this step
    let path_min_tvl = current_path
        .iter()
        .map(|s| s.tvl_usd)
        .fold(f64::MAX, f64::min);
    let potential_min_tvl = path_min_tvl.min(next_step.tvl_usd);

    // Get worst TVL in best routes
    let worst_best_tvl = best_by_tvl.last().map_or(0.0, |r| r.tvl_score);

    // Prune if we can't beat the worst best route and we're already deep
    if depth >= 2 && potential_min_tvl < worst_best_tvl * 0.1 {
        return true;
    }

    false
}

/// Merge routes from different criteria and deduplicate
fn merge_routes(mut by_tvl: Vec<CandidateRoute>, by_length: Vec<CandidateRoute>) -> Vec<Route> {
    // Add length-based routes that aren't already in TVL list
    for candidate in by_length {
        let is_duplicate = by_tvl
            .iter()
            .any(|r| routes_equal(&r.route, &candidate.route));
        if !is_duplicate {
            by_tvl.push(candidate);
        }
    }

    // Sort final list: prioritize shorter routes with good TVL
    by_tvl.sort_by(|a, b| {
        // First by length (shorter is better)
        match a.route.len().cmp(&b.route.len()) {
            std::cmp::Ordering::Equal => {
                // Then by TVL (higher is better)
                b.tvl_score
                    .partial_cmp(&a.tvl_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }
            other => other,
        }
    });

    // Return the routes
    by_tvl.into_iter().map(|c| c.route).collect()
}

/// Check if two routes are equal (same steps)
fn routes_equal(a: &Route, b: &Route) -> bool {
    if a.steps.len() != b.steps.len() {
        return false;
    }

    a.steps.iter().zip(b.steps.iter()).all(|(step_a, step_b)| {
        step_a.pool_address.to_lowercase() == step_b.pool_address.to_lowercase()
            && step_a.input_coin.to_lowercase() == step_b.input_coin.to_lowercase()
            && step_a.output_coin.to_lowercase() == step_b.output_coin.to_lowercase()
    })
}

/// Find the single best route between two tokens
#[must_use] 
pub fn find_best_route(graph: &RouteGraph, from: &str, to: &str) -> Option<Route> {
    let routes = find_routes(graph, from, to);
    routes.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::types::{PoolType, SwapParams, SwapType};

    fn make_test_graph() -> RouteGraph {
        let mut graph = RouteGraph::new("test");

        // Create a simple graph: A -> B -> C, A -> C (direct)
        let step_ab = RouteStep::new(
            "pool_ab",
            "0xpool_ab",
            "0xa",
            "0xb",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            10_000.0,
        );
        graph.add_edge("0xa", "0xb", step_ab);

        let step_bc = RouteStep::new(
            "pool_bc",
            "0xpool_bc",
            "0xb",
            "0xc",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            5_000.0,
        );
        graph.add_edge("0xb", "0xc", step_bc);

        let step_ac = RouteStep::new(
            "pool_ac",
            "0xpool_ac",
            "0xa",
            "0xc",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Crypto, 2),
            2_000.0, // Lower TVL but direct
        );
        graph.add_edge("0xa", "0xc", step_ac);

        graph
    }

    #[test]
    fn test_find_routes_direct() {
        let graph = make_test_graph();
        let routes = find_routes(&graph, "0xa", "0xc");

        assert!(!routes.is_empty());
        // Should find both direct (A->C) and indirect (A->B->C) routes
        assert!(routes.iter().any(|r| r.steps.len() == 1));
        assert!(routes.iter().any(|r| r.steps.len() == 2));
    }

    #[test]
    fn test_find_routes_no_path() {
        let graph = make_test_graph();
        let routes = find_routes(&graph, "0xa", "0xnonexistent");

        assert!(routes.is_empty());
    }

    #[test]
    fn test_find_best_route() {
        let graph = make_test_graph();
        let best = find_best_route(&graph, "0xa", "0xc");

        assert!(best.is_some());
        // Best route should be the direct one (shorter)
        let best = best.unwrap();
        assert_eq!(best.steps.len(), 1);
    }

    #[test]
    fn test_no_cycles() {
        let mut graph = RouteGraph::new("test");

        // Create a cycle: A -> B -> C -> A
        let step_ab = RouteStep::new(
            "pool_ab",
            "0xpool_ab",
            "0xa",
            "0xb",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        );
        graph.add_edge("0xa", "0xb", step_ab);

        let step_bc = RouteStep::new(
            "pool_bc",
            "0xpool_bc",
            "0xb",
            "0xc",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        );
        graph.add_edge("0xb", "0xc", step_bc);

        let step_ca = RouteStep::new(
            "pool_ca",
            "0xpool_ca",
            "0xc",
            "0xa",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        );
        graph.add_edge("0xc", "0xa", step_ca);

        // Also add edge to target D from A and C
        let step_ad = RouteStep::new(
            "pool_ad",
            "0xpool_ad",
            "0xa",
            "0xd",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        );
        graph.add_edge("0xa", "0xd", step_ad);

        // Should find path without infinite loop
        let routes = find_routes(&graph, "0xa", "0xd");
        assert!(!routes.is_empty());
        // Path should not revisit A
        for route in &routes {
            let mut seen: HashSet<String> = HashSet::new();
            seen.insert("0xa".to_string());
            for step in &route.steps {
                assert!(
                    !seen.contains(&step.output_coin.to_lowercase())
                        || step.output_coin.to_lowercase() == "0xd"
                );
                seen.insert(step.output_coin.to_lowercase());
            }
        }
    }
}
