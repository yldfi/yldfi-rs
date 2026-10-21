//! Token flow analysis
//!
//! Parses Transfer events to construct a token flow graph and compute net flows.

use crate::tx::addresses::{events, get_label};
use crate::tx::types::{NetTokenFlow, TokenFlow};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Log;
use std::collections::HashMap;

/// Parse Transfer events from logs
pub fn parse_transfers(logs: &[Log]) -> Vec<TokenFlow> {
    let mut flows = Vec::new();

    for log in logs {
        // Check if this is a Transfer event
        if let Some(topic0) = log.topics().first() {
            if *topic0 == events::TRANSFER {
                if let Some(transfer) = parse_transfer_log(log) {
                    flows.push(transfer);
                }
            }
        }
    }

    flows
}

/// Parse a single Transfer log
fn parse_transfer_log(log: &Log) -> Option<TokenFlow> {
    let topics = log.topics();

    // Transfer(address indexed from, address indexed to, uint256 value)
    // topics[0] = event signature
    // topics[1] = from (indexed)
    // topics[2] = to (indexed)
    // data = value

    if topics.len() < 3 {
        return None;
    }

    // Extract from and to addresses from topics
    let from = Address::from_slice(&topics[1].0[12..]);
    let to = Address::from_slice(&topics[2].0[12..]);

    let token = log.address();

    // ERC-721 Transfer has the same topic0 but a third indexed param, the
    // token id, and no data: one NFT moved, not an amount of a token
    if let Some(token_id) = topics.get(3) {
        return Some(TokenFlow {
            token,
            token_label: get_label(&token).map(String::from),
            from,
            from_label: get_label(&from).map(String::from),
            to,
            to_label: get_label(&to).map(String::from),
            amount: "1".to_string(),
            decimals: Some(0),
            token_id: Some(U256::from_be_bytes(token_id.0).to_string()),
            log_index: log.log_index.unwrap_or(0),
        });
    }

    // Extract value from data
    let data = &log.data().data;
    let amount = if data.len() >= 32 {
        U256::from_be_slice(&data[..32])
    } else {
        U256::ZERO
    };

    Some(TokenFlow {
        token,
        token_label: get_label(&token).map(String::from),
        from,
        from_label: get_label(&from).map(String::from),
        to,
        to_label: get_label(&to).map(String::from),
        amount: amount.to_string(),
        decimals: None,
        token_id: None,
        log_index: log.log_index.unwrap_or(0),
    })
}

/// Compute net token flows for an address
pub fn compute_net_flows(flows: &[TokenFlow], address: &Address) -> Vec<NetTokenFlow> {
    // Map: token -> (inflow, outflow)
    let mut balances: HashMap<Address, (U256, U256)> = HashMap::new();
    let mut labels: HashMap<Address, Option<String>> = HashMap::new();

    for flow in flows {
        let token = flow.token;
        let amount = U256::from_str_radix(&flow.amount, 10).unwrap_or(U256::ZERO);

        // Track token label
        labels
            .entry(token)
            .or_insert_with(|| flow.token_label.clone());

        // Track inflows and outflows
        let entry = balances.entry(token).or_insert((U256::ZERO, U256::ZERO));

        if flow.to == *address {
            entry.0 = entry.0.saturating_add(amount); // inflow
        }
        if flow.from == *address {
            entry.1 = entry.1.saturating_add(amount); // outflow
        }
    }

    // Convert to NetTokenFlow
    balances
        .into_iter()
        .filter_map(|(token, (inflow, outflow))| {
            if inflow == U256::ZERO && outflow == U256::ZERO {
                return None;
            }

            let (net_change, is_inflow) = if inflow >= outflow {
                (inflow.saturating_sub(outflow), true)
            } else {
                (outflow.saturating_sub(inflow), false)
            };

            // Skip if net is zero
            if net_change == U256::ZERO {
                return None;
            }

            let sign = if is_inflow { "+" } else { "-" };

            Some(NetTokenFlow {
                token,
                token_label: labels.get(&token).cloned().flatten(),
                net_change: format!("{}{}", sign, net_change),
                is_inflow,
            })
        })
        .collect()
}

/// Get unique tokens involved in flows
pub fn unique_tokens(flows: &[TokenFlow]) -> Vec<(Address, Option<String>)> {
    let mut seen = HashMap::new();

    for flow in flows {
        seen.entry(flow.token)
            .or_insert_with(|| flow.token_label.clone());
    }

    seen.into_iter().collect()
}

/// Get unique addresses involved in flows (both from and to)
pub fn unique_addresses(flows: &[TokenFlow]) -> Vec<Address> {
    let mut seen = std::collections::HashSet::new();

    for flow in flows {
        seen.insert(flow.from);
        seen.insert(flow.to);
    }

    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {

    fn transfer_log(topics: Vec<alloy::primitives::B256>, data: Vec<u8>) -> Log {
        Log {
            inner: alloy::primitives::Log::new_unchecked(
                Address::from_slice(&[9u8; 20]),
                topics,
                data.into(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn erc721_transfer_is_one_nft_not_an_amount() {
        let from = alloy::primitives::B256::left_padding_from(&[1u8; 20]);
        let to = alloy::primitives::B256::left_padding_from(&[2u8; 20]);
        let id = alloy::primitives::B256::from(U256::from(1234u64));
        let flows = parse_transfers(&[transfer_log(vec![events::TRANSFER, from, to, id], vec![])]);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].amount, "1");
        assert_eq!(flows[0].token_id.as_deref(), Some("1234"));
        assert_eq!(flows[0].decimals, Some(0));

        // ERC-20: value in data, no token id
        let flows = parse_transfers(&[transfer_log(
            vec![events::TRANSFER, from, to],
            U256::from(5u64).to_be_bytes::<32>().to_vec(),
        )]);
        assert_eq!(flows[0].amount, "5");
        assert!(flows[0].token_id.is_none());
    }

    use super::*;

    #[test]
    fn test_compute_net_flows() {
        let addr = Address::from_slice(&[1u8; 20]);
        let token = Address::from_slice(&[2u8; 20]);
        let other = Address::from_slice(&[3u8; 20]);

        let flows = vec![
            TokenFlow {
                token,
                token_label: Some("TEST".to_string()),
                from: other,
                from_label: None,
                to: addr,
                to_label: None,
                amount: "1000".to_string(),
                decimals: None,
                token_id: None,
                log_index: 0,
            },
            TokenFlow {
                token,
                token_label: Some("TEST".to_string()),
                from: addr,
                from_label: None,
                to: other,
                to_label: None,
                amount: "400".to_string(),
                decimals: None,
                token_id: None,
                log_index: 1,
            },
        ];

        let net = compute_net_flows(&flows, &addr);
        assert_eq!(net.len(), 1);
        assert!(net[0].is_inflow);
        assert_eq!(net[0].net_change, "+600");
    }
}
