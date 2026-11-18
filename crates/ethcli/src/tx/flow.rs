//! Token flow analysis
//!
//! Parses Transfer events to construct a token flow graph and compute net flows.

use crate::tx::addresses::{events, get_label};
use crate::tx::types::{NetTokenFlow, TokenFlow, WrapEvent};
use alloy::primitives::{Address, U256};
use alloy::rpc::types::Log;
use std::collections::HashMap;

/// The wrapped native token of a chain (WETH, WXDAI, WMATIC, WBNB)
pub fn wrapped_native_token(chain_id: u64) -> Option<Address> {
    let address = match chain_id {
        1 => "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        10 | 8453 => "0x4200000000000000000000000000000000000006",
        56 => "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c",
        100 => "0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d",
        137 => "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270",
        42161 => "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
        11155111 => "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14",
        _ => return None,
    };
    address.parse().ok()
}

/// Deposits into and withdrawals from `token`, the chain's wrapped native
/// token: `Deposit(address indexed dst, uint256 wad)` and
/// `Withdrawal(address indexed src, uint256 wad)`. Other contracts emit
/// events with the same signatures, so only `token`'s own logs count.
pub fn parse_wraps(logs: &[Log], token: Address) -> Vec<WrapEvent> {
    use alloy::primitives::keccak256;
    let deposit = keccak256(b"Deposit(address,uint256)");
    let withdrawal = keccak256(b"Withdrawal(address,uint256)");

    logs.iter()
        .filter(|log| log.address() == token)
        .filter_map(|log| {
            let topics = log.topics();
            let is_deposit = match topics.first() {
                Some(topic) if *topic == deposit => true,
                Some(topic) if *topic == withdrawal => false,
                _ => return None,
            };
            let account = Address::from_slice(&topics.get(1)?[12..]);
            let data = log.data().data.as_ref();
            (data.len() == 32).then(|| WrapEvent {
                token,
                account,
                amount: U256::from_be_slice(data),
                deposit: is_deposit,
            })
        })
        .collect()
}

/// Every native-token transfer in a `callTracer` trace, top-level call
/// included, as (from, to, amount). A frame that reverted moved nothing, and
/// neither did anything below it. DELEGATECALL and CALLCODE frames repeat
/// their parent's value without moving it, and STATICCALL cannot move any.
pub fn native_transfers_from_trace(root: &serde_json::Value) -> Vec<(Address, Address, U256)> {
    fn walk(frame: &serde_json::Value, out: &mut Vec<(Address, Address, U256)>) {
        if frame.get("error").is_some_and(|e| !e.is_null()) {
            return;
        }
        let kind = frame["type"]
            .as_str()
            .unwrap_or("CALL")
            .to_ascii_uppercase();
        if !matches!(kind.as_str(), "DELEGATECALL" | "CALLCODE" | "STATICCALL") {
            let field = |key: &str| frame[key].as_str();
            if let (Some(from), Some(to), Some(value)) =
                (field("from"), field("to"), field("value"))
            {
                if let (Ok(from), Ok(to), Ok(value)) = (
                    from.parse::<Address>(),
                    to.parse::<Address>(),
                    value.parse::<U256>(),
                ) {
                    if !value.is_zero() {
                        out.push((from, to, value));
                    }
                }
            }
        }
        for child in frame["calls"].as_array().into_iter().flatten() {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out
}

/// The revert data of a failed transaction, from the root frame of its
/// `callTracer` trace. `Some(empty)` means it failed without revert data
/// (out of gas, or a bare `revert()`); `None` means the trace shows no
/// failure at the top level.
pub fn revert_data_from_trace(root: &serde_json::Value) -> Option<Vec<u8>> {
    root.get("error").filter(|e| !e.is_null())?;
    let output = root["output"].as_str().unwrap_or("0x");
    Some(hex::decode(output.strip_prefix("0x").unwrap_or(output)).unwrap_or_default())
}

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

#[cfg(test)]
mod wrap_and_trace_tests {
    use super::*;

    #[test]
    fn trace_walk_skips_reverted_frames_and_calls_that_move_no_value() {
        let a = "0x00000000000000000000000000000000000000aa";
        let b = "0x00000000000000000000000000000000000000bb";
        let c = "0x00000000000000000000000000000000000000cc";
        let trace = serde_json::json!({
            "type": "CALL", "from": a, "to": b, "value": "0xde0b6b3a7640000",
            "calls": [
                {"type": "CALL", "from": b, "to": c, "value": "0x64"},
                // Repeats the parent's value without moving it
                {"type": "DELEGATECALL", "from": b, "to": c, "value": "0xde0b6b3a7640000",
                 "calls": [{"type": "CALL", "from": b, "to": a, "value": "0x1"}]},
                {"type": "STATICCALL", "from": b, "to": c},
                {"type": "CALL", "from": b, "to": c, "value": "0x0"},
                // Reverted: neither it nor its children moved anything
                {"type": "CALL", "from": b, "to": c, "value": "0x5", "error": "execution reverted",
                 "calls": [{"type": "CALL", "from": c, "to": a, "value": "0x5"}]}
            ]
        });
        let transfers = native_transfers_from_trace(&trace);
        let amounts: Vec<u64> = transfers.iter().map(|t| t.2.to::<u128>() as u64).collect();
        assert_eq!(amounts, vec![1_000_000_000_000_000_000, 100, 1]);
        assert_eq!(transfers[1].0, b.parse::<Address>().unwrap());
        assert_eq!(transfers[1].1, c.parse::<Address>().unwrap());

        // A transaction that failed at the top level moved nothing
        let failed = serde_json::json!({
            "type": "CALL", "from": a, "to": b, "value": "0x64", "error": "execution reverted"
        });
        assert!(native_transfers_from_trace(&failed).is_empty());
    }

    #[test]
    fn revert_data_is_read_from_the_root_frame() {
        let reverted = serde_json::json!({
            "type": "CALL", "error": "execution reverted", "output": "0x08c379a0ff"
        });
        assert_eq!(
            revert_data_from_trace(&reverted),
            Some(vec![0x08, 0xc3, 0x79, 0xa0, 0xff])
        );
        // Failed with no data, e.g. out of gas
        let out_of_gas = serde_json::json!({"type": "CALL", "error": "out of gas"});
        assert_eq!(revert_data_from_trace(&out_of_gas), Some(vec![]));
        // A successful transaction, even with a failed inner call
        let ok = serde_json::json!({
            "type": "CALL", "output": "0x01",
            "calls": [{"type": "CALL", "error": "execution reverted", "output": "0xdead"}]
        });
        assert_eq!(revert_data_from_trace(&ok), None);
    }

    #[test]
    fn wrapped_native_token_is_known_for_the_main_chains() {
        assert!(wrapped_native_token(1).is_some());
        assert_eq!(wrapped_native_token(10), wrapped_native_token(8453));
        assert!(wrapped_native_token(999_999).is_none());
    }
}
