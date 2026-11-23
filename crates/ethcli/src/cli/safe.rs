//! Read-only Safe (Gnosis Safe) commands
//!
//! `info` reads the Safe's state on chain (one Multicall3 request plus three
//! storage reads), so it needs no API. `queue`, `tx` and `history` read the
//! public Safe Transaction Service. Nothing here proposes, signs or executes.

use crate::abi::AbiFetcher;
use crate::cli::contract::{dyn_value_to_json, storage_address};
use crate::cli::OutputFormat;
use crate::config::{Chain, EndpointConfig};
use crate::rpc::multicall::MulticallBuilder;
use crate::rpc::Endpoint;
use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::primitives::{keccak256, Address, Bytes, B256, U256};
use alloy::providers::Provider;
use clap::Subcommand;
use futures::StreamExt;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Selector of `multiSend(bytes)` (MultiSend and MultiSendCallOnly)
const MULTISEND_SELECTOR: [u8; 4] = [0x8d, 0x80, 0xff, 0x0a];
/// `keccak256("guard_manager.guard.address")`
const GUARD_SLOT: &str = "0x4a204f620c8c5ccdca3fd54d003badd85ba500436a431f0cbda4f558c93c34c8";
/// `keccak256("fallback_manager.handler.address")`
const FALLBACK_HANDLER_SLOT: &str =
    "0x6c9a6c4a39284e37ed1cf53d337577d14212a4870fb976a4366c693b939918d5";
/// Safe library deployments that a Safe is expected to DELEGATECALL, from
/// safe-global/safe-deployments (canonical, eip155 and zksync variants).
/// `true` marks MultiSend / MultiSendCallOnly, whose `multiSend(bytes)`
/// payload is unpacked.
const KNOWN_DELEGATE_TARGETS: &[(&str, &str, bool)] = &[
    (
        "0x8D29bE29923b68abfDD21e541b9374737B49cdAD",
        "MultiSend 1.1.1",
        true,
    ),
    (
        "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761",
        "MultiSend 1.3.0",
        true,
    ),
    (
        "0x998739BFdAAdde7C933B942a68053933098f9EDa",
        "MultiSend 1.3.0 (eip155)",
        true,
    ),
    (
        "0x0dFcccB95225ffB03c6FBB2559B530C2B7C8A912",
        "MultiSend 1.3.0 (zksync)",
        true,
    ),
    (
        "0x40A2aCCbd92BCA938b02010E17A5b8929b49130D",
        "MultiSendCallOnly 1.3.0",
        true,
    ),
    (
        "0xA1dabEF33b3B82c7814B6D82A79e50F4AC44102B",
        "MultiSendCallOnly 1.3.0 (eip155)",
        true,
    ),
    (
        "0xf220D3b4DFb23C4ade8C88E526C1353AbAcbC38F",
        "MultiSendCallOnly 1.3.0 (zksync)",
        true,
    ),
    (
        "0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526",
        "MultiSend 1.4.1",
        true,
    ),
    (
        "0x309D0B190FeCCa8e1D5D8309a16F7e3CB133E885",
        "MultiSend 1.4.1 (zksync)",
        true,
    ),
    (
        "0x9641d764fc13c8B624c04430C7356C1C7C8102e2",
        "MultiSendCallOnly 1.4.1",
        true,
    ),
    (
        "0x0408EF011960d02349d50286D20531229BCef773",
        "MultiSendCallOnly 1.4.1 (zksync)",
        true,
    ),
    (
        "0x218543288004CD07832472D464648173c77D7eB7",
        "MultiSend 1.5.0",
        true,
    ),
    (
        "0xA83c336B20401Af773B6219BA5027174338D1836",
        "MultiSendCallOnly 1.5.0",
        true,
    ),
    (
        "0xA65387F16B013cf2Af4605Ad8aA5ec25a2cbA3a2",
        "SignMessageLib 1.3.0",
        false,
    ),
    (
        "0x98FFBBF51bb33A056B08ddf711f289936AafF717",
        "SignMessageLib 1.3.0 (eip155)",
        false,
    ),
    (
        "0x357147caf9C0cCa67DfA0CF5369318d8193c8407",
        "SignMessageLib 1.3.0 (zksync)",
        false,
    ),
    (
        "0xd53cd0aB83D845Ac265BE939c57F53AD838012c9",
        "SignMessageLib 1.4.1",
        false,
    ),
    (
        "0xAca1ec0a1A575CDCCF1DC3d5d296202Eb6061888",
        "SignMessageLib 1.4.1 (zksync)",
        false,
    ),
    (
        "0x7cbB62EaA69F79e6873cD1ecB2392971036cFAa4",
        "CreateCall 1.3.0",
        false,
    ),
    (
        "0xB19D6FFc2182150F8Eb585b79D4ABcd7C5640A9d",
        "CreateCall 1.3.0 (eip155)",
        false,
    ),
    (
        "0xcB8e5E438c5c2b45FbE17B02Ca9aF91509a8ad56",
        "CreateCall 1.3.0 (zksync)",
        false,
    ),
    (
        "0x9b35Af71d77eaf8d7e40252370304687390A1A52",
        "CreateCall 1.4.1",
        false,
    ),
    (
        "0xAAA566Fe7978bB0fb0B5362B7ba23038f4428D8f",
        "CreateCall 1.4.1 (zksync)",
        false,
    ),
];

/// The name of a known Safe library and whether it is a MultiSend
pub fn known_library(to: Address) -> Option<(&'static str, bool)> {
    KNOWN_DELEGATE_TARGETS
        .iter()
        .find(|(a, _, _)| a.parse::<Address>().ok() == Some(to))
        .map(|(_, name, multisend)| (*name, *multisend))
}

/// A warning when `operation` is DELEGATECALL to an address that is not a
/// known Safe library: that code runs with full control of the Safe
pub fn delegatecall_warning(operation: u8, to: Address) -> Option<String> {
    (operation == 1 && known_library(to).is_none()).then(|| {
        format!(
            "DELEGATECALL to {} which is not a known Safe library: it runs with full control of the Safe's storage and funds",
            to.to_checksum(None)
        )
    })
}

/// Concurrent signature/ABI lookups when decoding several transactions
const DECODE_CONCURRENCY: usize = 4;

#[derive(Subcommand)]
pub enum SafeCommands {
    /// Owners, threshold, nonce, version, modules, guard and fallback handler, read on chain
    Info {
        /// Safe address
        safe: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Pending transactions (nonce >= the on-chain nonce) with who has and has not signed
    Queue {
        /// Safe address
        safe: String,

        /// Maximum transactions to list
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// One transaction by safeTxHash, calldata decoded and MultiSend batches unpacked
    Tx {
        /// safeTxHash (the Safe's EIP-712 hash, not the chain transaction hash)
        safe_tx_hash: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Executed transactions, newest first
    History {
        /// Safe address
        safe: String,

        /// Maximum transactions to list
        #[arg(long, default_value = "20")]
        limit: u32,

        /// Skip this many transactions (for paging)
        #[arg(long, default_value = "0")]
        offset: u32,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },
}

pub async fn handle(
    action: &SafeCommands,
    chain: Chain,
    rpc_url: Option<String>,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        SafeCommands::Info { safe, format } => {
            let safe = parse_address(safe)?;
            let state = read_state(chain, rpc_url.as_deref(), safe).await?;
            let out = json!({
                "chain": chain.name(),
                "address": safe.to_checksum(None),
                "block": state.block.to_string(),
                "version": state.version,
                "nonce": state.nonce.map(|n| n.to_string()),
                "threshold": state.threshold.to_string(),
                "owners": state.owners.as_ref().map(|o| o.iter().map(|a| a.to_checksum(None)).collect::<Vec<_>>()),
                "modules": state.modules.as_ref().map(|m| m.iter().map(|a| a.to_checksum(None)).collect::<Vec<_>>()),
                "modules_truncated": state.modules_truncated,
                "guard": state.guard.map(|a| a.to_checksum(None)),
                "fallback_handler": state.fallback_handler.map(|a| a.to_checksum(None)),
                "singleton": state.singleton.map(|a| a.to_checksum(None)),
            });
            if format.is_table() {
                print_info(&out);
            } else {
                print_json(&out, *format)?;
            }
        }

        SafeCommands::Queue {
            safe,
            limit,
            format,
        } => {
            let safe = parse_address(safe)?;
            let service = Service::new(chain)?;
            let state = read_state(chain, rpc_url.as_deref(), safe).await?;
            let (Some(owners), Some(current_nonce)) = (state.owners.clone(), state.nonce) else {
                anyhow::bail!("could not read the Safe's owners or nonce on chain; cannot tell who has signed");
            };
            let path = format!(
                "/api/v2/safes/{}/multisig-transactions/?executed=false&nonce__gte={}&ordering=nonce&limit={}",
                safe.to_checksum(None),
                current_nonce,
                limit
            );
            let page = service.get(&path).await?;
            let results = page["results"].as_array().cloned().unwrap_or_default();
            let fetcher = AbiFetcher::new(api_key)?;
            let decoded = decode_many(&fetcher, chain, &results).await;

            // Several proposals at one nonce are alternatives: only one can run
            let mut per_nonce: HashMap<String, usize> = HashMap::new();
            for tx in &results {
                *per_nonce.entry(text(&tx["nonce"])).or_default() += 1;
            }

            let items: Vec<Value> = results
                .iter()
                .zip(decoded)
                .map(|(tx, call)| {
                    let mut item = summarize(tx, call);
                    let s = signers(tx, &owners);
                    let conflicting = per_nonce.get(&text(&tx["nonce"])).copied().unwrap_or(1) > 1;
                    let complete = is_ready(&s, state.threshold);
                    let at_current = text(&tx["nonce"]) == current_nonce.to_string();
                    item["confirmations"] = json!(s.signed.len());
                    item["threshold"] = json!(state.threshold.to_string());
                    item["signatures_complete"] = json!(complete);
                    item["ready"] = json!(complete && at_current);
                    item["signed"] = json!(s.signed);
                    item["not_signed"] = json!(s.not_signed);
                    item["stale_signers"] = json!(s.stale);
                    item["conflicting_proposals"] = json!(conflicting);
                    let mut warnings = integrity_warnings(tx, chain.chain_id(), Some(safe), None);
                    if complete && !at_current {
                        warnings.push(format!(
                            "fully signed but nonce {} is not the current nonce {current_nonce}: it cannot run until earlier nonces execute",
                            text(&tx["nonce"])
                        ));
                    }
                    if conflicting {
                        warnings.push("other proposals share this nonce: only one of them can execute".to_string());
                    }
                    item["warnings"] = json!(warnings);
                    item
                })
                .collect();
            let out = json!({
                "chain": chain.name(),
                "safe": safe.to_checksum(None),
                "nonce": current_nonce.to_string(),
                "threshold": state.threshold.to_string(),
                "owners": owners.len(),
                "count": items.len(),
                "total": page["count"],
                "transactions": items,
            });
            if format.is_table() {
                print_queue(&out);
            } else {
                print_json(&out, *format)?;
            }
        }

        SafeCommands::Tx {
            safe_tx_hash,
            format,
        } => {
            let hash: B256 = safe_tx_hash
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid safeTxHash: {safe_tx_hash}"))?;
            let service = Service::new(chain)?;
            let tx = service
                .get(&format!("/api/v2/multisig-transactions/{hash:#x}/"))
                .await?;
            let fetcher = AbiFetcher::new(api_key)?;
            let to = parse_address(&text(&tx["to"]))?;
            let data = hex_bytes(&tx["data"]);
            let operation = tx["operation"].as_u64().unwrap_or(0) as u8;
            let mut call = decode_call(&fetcher, chain, to, &data).await;
            if let Some(inner) = nested_payload(operation, to, &data) {
                match decode_batch(&fetcher, chain, &inner, 1).await {
                    Ok(calls) => call["multisend"] = json!(calls),
                    Err(e) => call["multisend_error"] = json!(e.to_string()),
                }
            }
            let warnings = top_level_warnings(operation, to, &data);
            if !warnings.is_empty() {
                call["batch_warnings"] = json!(warnings);
            }

            let mut out = json!({
                "chain": chain.name(),
                "safe": tx["safe"],
                "safe_tx_hash": tx["safeTxHash"],
                "nonce": text(&tx["nonce"]),
                "to": tx["to"],
                "value": text(&tx["value"]),
                "operation": operation_name(tx["operation"].as_u64().unwrap_or(0) as u8),
                "data": tx["data"],
                "call": call,
                "delegatecall_warning": delegatecall_warning(tx["operation"].as_u64().unwrap_or(0) as u8, to),
                "submitted": tx["submissionDate"],
                "proposer": tx["proposer"],
                "executed": tx["isExecuted"],
                "successful": tx["isSuccessful"],
                "execution_date": tx["executionDate"],
                "transaction_hash": tx["transactionHash"],
                "executor": tx["executor"],
                "confirmations_required": tx["confirmationsRequired"],
                "refund": refund_fields(&tx),
                "warnings": integrity_warnings(&tx, chain.chain_id(), None, Some(hash)),
            });
            let signed: Vec<Value> = confirmations(&tx).into_iter().map(|a| json!(a)).collect();
            if let Some(w) = refund_warning(&tx) {
                if let Some(list) = out["warnings"].as_array_mut() {
                    list.push(json!(w));
                }
            }
            out["confirmations"] = json!(signed.len());
            out["signed"] = json!(signed);
            if tx["isExecuted"].as_bool() != Some(true) {
                // Who still has to sign is only meaningful while it is pending,
                // against the current owners
                if let Ok(safe) = parse_address(&text(&tx["safe"])) {
                    match read_state(chain, rpc_url.as_deref(), safe).await {
                        Ok(SafeState {
                            owners: Some(owners),
                            nonce: Some(current_nonce),
                            threshold,
                            ..
                        }) => {
                            let state_threshold = threshold;
                            let s = signers(&tx, &owners);
                            let complete = is_ready(&s, state_threshold);
                            let at_current = text(&tx["nonce"]) == current_nonce.to_string();
                            out["confirmations"] = json!(s.signed.len());
                            out["signatures_complete"] = json!(complete);
                            out["ready"] = json!(complete && at_current);
                            if complete && !at_current {
                                if let Some(list) = out["warnings"].as_array_mut() {
                                    list.push(json!(format!(
                                        "fully signed but nonce {} is not the current nonce {current_nonce}",
                                        text(&tx["nonce"])
                                    )));
                                }
                            }
                            out["signed"] = json!(s.signed);
                            out["not_signed"] = json!(s.not_signed);
                            out["stale_signers"] = json!(s.stale);
                            out["threshold"] = json!(state_threshold.to_string());
                            out["safe_nonce"] = json!(current_nonce.to_string());
                        }
                        Ok(_) => {
                            if !quiet {
                                eprintln!("Note: on-chain owners or nonce not read; signer split not shown");
                            }
                        }
                        Err(e) => {
                            if !quiet {
                                eprintln!("Note: on-chain owners not read: {e}");
                            }
                        }
                    }
                }
            }
            if format.is_table() {
                print_tx(&out);
            } else {
                print_json(&out, *format)?;
            }
        }

        SafeCommands::History {
            safe,
            limit,
            offset,
            format,
        } => {
            let safe = parse_address(safe)?;
            let service = Service::new(chain)?;
            let path = format!(
                "/api/v2/safes/{}/multisig-transactions/?executed=true&ordering=-nonce&limit={}&offset={}",
                safe.to_checksum(None),
                limit,
                offset
            );
            let page = service.get(&path).await?;
            let results = page["results"].as_array().cloned().unwrap_or_default();
            let fetcher = AbiFetcher::new(api_key)?;
            let decoded = decode_many(&fetcher, chain, &results).await;
            let items: Vec<Value> = results
                .iter()
                .zip(decoded)
                .map(|(tx, call)| {
                    let mut item = summarize(tx, call);
                    item["successful"] = tx["isSuccessful"].clone();
                    item["execution_date"] = tx["executionDate"].clone();
                    item["transaction_hash"] = tx["transactionHash"].clone();
                    item["confirmations"] = json!(confirmations(tx).len());
                    item["warnings"] =
                        json!(integrity_warnings(tx, chain.chain_id(), Some(safe), None));
                    item
                })
                .collect();
            let out = json!({
                "chain": chain.name(),
                "safe": safe.to_checksum(None),
                "count": items.len(),
                "total": page["count"],
                "offset": offset,
                "transactions": items,
            });
            if format.is_table() {
                print_history(&out);
            } else {
                print_json(&out, *format)?;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// On-chain state
// ---------------------------------------------------------------------------

struct SafeState {
    block: u64,
    version: Option<String>,
    /// `None` when the read failed
    nonce: Option<U256>,
    threshold: U256,
    /// `None` when the read failed
    owners: Option<Vec<Address>>,
    /// `None` when neither getModulesPaginated nor getModules answered
    modules: Option<Vec<Address>>,
    /// More than [`MODULE_PAGE`] modules are enabled; only the first page is listed
    modules_truncated: bool,
    guard: Option<Address>,
    fallback_handler: Option<Address>,
    singleton: Option<Address>,
}

fn selector(signature: &str) -> Bytes {
    keccak256(signature.as_bytes())[..4].to_vec().into()
}

async fn read_state(
    chain: Chain,
    rpc_url: Option<&str>,
    safe: Address,
) -> anyhow::Result<SafeState> {
    let endpoint = match rpc_url {
        Some(url) => Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?,
        None => crate::rpc::selector::get_rpc_endpoint(chain)?,
    };
    let provider = endpoint.provider();
    let block = provider.get_block_number().await?;
    let at = alloy::eips::BlockId::number(block);

    // getModulesPaginated(SENTINEL, MODULE_PAGE): the first page of enabled
    // modules; Safe 1.0.0 only has getModules()
    let mut modules_call = selector("getModulesPaginated(address,uint256)").to_vec();
    modules_call.extend(
        DynSolValue::Tuple(vec![
            DynSolValue::Address(SENTINEL),
            DynSolValue::Uint(U256::from(MODULE_PAGE), 256),
        ])
        .abi_encode_params(),
    );
    let results = MulticallBuilder::new()
        .add_call(safe, selector("getOwners()"), true)
        .add_call(safe, selector("getThreshold()"), true)
        .add_call(safe, selector("nonce()"), true)
        .add_call(safe, selector("VERSION()"), true)
        .add_call(safe, modules_call.into(), true)
        .add_call(safe, selector("getModules()"), true)
        .execute_at(provider, at)
        .await?;
    let ok = |i: usize| {
        results
            .get(i)
            .is_some_and(|r| r.success && !r.data.is_empty())
    };

    let threshold = ok(1)
        .then(|| results[1].decode_uint256())
        .flatten()
        .ok_or_else(|| {
            anyhow::anyhow!("{safe} does not answer getThreshold(): not a Safe on {chain}")
        })?;
    let owners = ok(0)
        .then(|| {
            decode_addresses(
                &DynSolType::Array(Box::new(DynSolType::Address)),
                &results[0].data,
            )
        })
        .flatten();
    let nonce = ok(2).then(|| results[2].decode_uint256()).flatten();
    let version = ok(3).then(|| results[3].decode_string()).flatten();
    let (modules, modules_truncated) = match ok(4)
        .then(|| decode_modules_page(&results[4].data))
        .flatten()
    {
        Some((list, next)) => {
            let truncated = next != SENTINEL && !next.is_zero();
            (Some(list), truncated)
        }
        None => (
            ok(5)
                .then(|| {
                    decode_addresses(
                        &DynSolType::Array(Box::new(DynSolType::Address)),
                        &results[5].data,
                    )
                })
                .flatten(),
            false,
        ),
    };

    let slot = |s: &str| s.parse::<B256>().expect("valid slot");
    let (guard, fallback_handler, singleton) = tokio::join!(
        storage_address(provider, safe, slot(GUARD_SLOT), at),
        storage_address(provider, safe, slot(FALLBACK_HANDLER_SLOT), at),
        storage_address(provider, safe, B256::ZERO, at),
    );
    Ok(SafeState {
        block,
        version,
        nonce,
        threshold,
        owners,
        modules,
        modules_truncated,
        guard,
        fallback_handler,
        singleton,
    })
}

/// The modules sentinel (`address(0x1)`)
const SENTINEL: Address =
    Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
/// Modules read per `getModulesPaginated` page
const MODULE_PAGE: u64 = 100;

/// `getModulesPaginated` return: (modules, next)
fn decode_modules_page(data: &[u8]) -> Option<(Vec<Address>, Address)> {
    let ty = DynSolType::Tuple(vec![
        DynSolType::Array(Box::new(DynSolType::Address)),
        DynSolType::Address,
    ]);
    let DynSolValue::Tuple(items) = ty.abi_decode_params(data).ok()? else {
        return None;
    };
    let list = match items.first()? {
        DynSolValue::Array(a) => a.iter().filter_map(|v| v.as_address()).collect(),
        _ => return None,
    };
    let next = items.get(1)?.as_address()?;
    Some((list, next))
}

/// The addresses in an `address[]` return, or the first `address[]` of a tuple
fn decode_addresses(ty: &DynSolType, data: &[u8]) -> Option<Vec<Address>> {
    let value = match ty {
        DynSolType::Tuple(_) => ty.abi_decode_params(data).ok()?,
        _ => ty.abi_decode(data).ok()?,
    };
    let list = match value {
        DynSolValue::Tuple(mut items) if !items.is_empty() => items.swap_remove(0),
        other => other,
    };
    match list {
        DynSolValue::Array(items) => Some(items.iter().filter_map(|v| v.as_address()).collect()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Safe Transaction Service
// ---------------------------------------------------------------------------

/// The Transaction Service's short name for a chain, as used in
/// `https://api.safe.global/tx-service/<name>`
pub fn service_chain_name(chain_id: u64) -> Option<&'static str> {
    Some(match chain_id {
        1 => "eth",
        10 => "oeth",
        56 => "bnb",
        100 => "gno",
        130 => "unichain",
        137 => "pol",
        146 => "sonic",
        324 => "zksync",
        480 => "wc",
        1313161554 => "aurora",
        8453 => "base",
        42161 => "arb1",
        42220 => "celo",
        43114 => "avax",
        57073 => "ink",
        59144 => "linea",
        80094 => "berachain",
        534352 => "scr",
        43111 => "hemi",
        84532 => "basesep",
        11155111 => "sep",
        _ => return None,
    })
}

struct Service {
    base: String,
    client: reqwest::Client,
    api_key: Option<String>,
}

impl Service {
    fn new(chain: Chain) -> anyhow::Result<Self> {
        let name = service_chain_name(chain.chain_id()).ok_or_else(|| {
            anyhow::anyhow!(
                "The Safe Transaction Service has no known endpoint for chain {} ({}); `safe info` still works on any chain",
                chain.name(),
                chain.chain_id()
            )
        })?;
        let base = std::env::var("SAFE_TX_SERVICE_URL")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("https://api.safe.global/tx-service/{name}"));
        Ok(Self {
            base: base.trim_end_matches('/').to_string(),
            client: crate::utils::get_shared_http_client()
                .map_err(|e| anyhow::anyhow!(e))?
                .clone(),
            api_key: std::env::var("SAFE_API_KEY").ok().filter(|s| !s.is_empty()),
        })
    }

    async fn get(&self, path: &str) -> anyhow::Result<Value> {
        let mut request = self.client.get(format!("{}{}", self.base, path));
        if let Some(key) = &self.api_key {
            request = request.bearer_auth(key);
        }
        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;
        if status.as_u16() == 404 {
            anyhow::bail!("Safe Transaction Service: not found (wrong chain, or not indexed)");
        }
        if status.as_u16() == 429 {
            anyhow::bail!(
                "Safe Transaction Service rate limit reached; set SAFE_API_KEY (from developer.safe.global) for a higher limit"
            );
        }
        if !status.is_success() {
            let snippet =
                crate::abi::revert::safe_text(&body.chars().take(300).collect::<String>());
            anyhow::bail!("Safe Transaction Service returned {status}: {snippet}");
        }
        Ok(serde_json::from_str(&body)?)
    }
}

// ---------------------------------------------------------------------------
// Decoding
// ---------------------------------------------------------------------------

/// One call inside a MultiSend batch
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiSendCall {
    /// 0 = call, 1 = delegatecall
    pub operation: u8,
    pub to: Address,
    pub value: U256,
    pub data: Vec<u8>,
}

/// The packed `transactions` bytes, when `data` is a `multiSend(bytes)` call
pub fn multisend_payload(to: Address, data: &[u8]) -> Option<Vec<u8>> {
    if !known_library(to).is_some_and(|(_, multisend)| multisend) {
        return None;
    }
    if data.len() < 4 || data[..4] != MULTISEND_SELECTOR {
        return None;
    }
    match DynSolType::Bytes.abi_decode_params(&data[4..]).ok()? {
        DynSolValue::Bytes(b) => Some(b),
        _ => None,
    }
}

/// Unpack MultiSend's packed encoding: for each call
/// `operation (1) | to (20) | value (32) | data length (32) | data`
pub fn unpack_multisend(packed: &[u8]) -> anyhow::Result<Vec<MultiSendCall>> {
    let mut calls = Vec::new();
    let mut i = 0usize;
    while i < packed.len() {
        let header_end = i + 85;
        if header_end > packed.len() {
            anyhow::bail!(
                "MultiSend data truncated in the header of call {}",
                calls.len()
            );
        }
        let operation = packed[i];
        let to = Address::from_slice(&packed[i + 1..i + 21]);
        let value = U256::from_be_slice(&packed[i + 21..i + 53]);
        let len = U256::from_be_slice(&packed[i + 53..i + 85]);
        let len: usize = len
            .try_into()
            .ok()
            .filter(|l: &usize| {
                header_end
                    .checked_add(*l)
                    .is_some_and(|e| e <= packed.len())
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "MultiSend data truncated in the body of call {}",
                    calls.len()
                )
            })?;
        calls.push(MultiSendCall {
            operation,
            to,
            value,
            data: packed[header_end..header_end + len].to_vec(),
        });
        i = header_end + len;
    }
    Ok(calls)
}

fn operation_name(op: u8) -> &'static str {
    match op {
        0 => "call",
        1 => "delegatecall",
        _ => "unknown",
    }
}

/// Decode a signature's arguments from calldata (everything after the selector)
pub fn decode_with_signature(signature: &str, data: &[u8]) -> Option<Vec<Value>> {
    let args = signature.get(signature.find('(')?..)?;
    let ty = DynSolType::parse(args).ok()?;
    let DynSolValue::Tuple(values) = ty.abi_decode_params(data.get(4..)?).ok()? else {
        return None;
    };
    let DynSolType::Tuple(types) = ty else {
        return None;
    };
    Some(
        values
            .iter()
            .zip(types.iter())
            .enumerate()
            .map(|(i, (v, t))| {
                json!({"name": format!("arg{i}"), "type": t.sol_type_name(), "value": dyn_value_to_json(v)})
            })
            .collect(),
    )
}

/// Decode one call: the target's verified ABI first, then the signature
/// database (best guess, marked as such)
async fn decode_call(fetcher: &AbiFetcher, chain: Chain, to: Address, data: &[u8]) -> Value {
    let mut call = decode_call_inner(fetcher, chain, to, data).await;
    if let Some((name, _)) = known_library(to) {
        call["library"] = json!(name);
    } else if data.len() >= 4 && data[..4] == MULTISEND_SELECTOR {
        call["warning"] = json!(
            "unknown contract using the multiSend selector: not a known MultiSend deployment, batch not unpacked"
        );
    }
    call
}

async fn decode_call_inner(fetcher: &AbiFetcher, chain: Chain, to: Address, data: &[u8]) -> Value {
    if data.len() < 4 {
        return json!({"kind": if data.is_empty() { "transfer" } else { "raw" }});
    }
    let selector = format!("0x{}", hex::encode(&data[..4]));
    if let Some(decoded) = fetcher
        .decode_function_call(chain, &format!("{to:#x}"), data)
        .await
    {
        return json!({
            "selector": decoded.selector,
            "function": decoded.name,
            "signature": decoded.signature,
            "source": "abi",
            "params": decoded.params.into_iter().map(|(name, ty, value)| json!({"name": name, "type": ty, "value": value})).collect::<Vec<_>>(),
        });
    }
    if data[..4] == MULTISEND_SELECTOR && known_library(to).is_some_and(|(_, m)| m) {
        return json!({"selector": selector, "function": "multiSend", "signature": "multiSend(bytes)", "source": "builtin"});
    }
    if let Some(signature) = fetcher.lookup_selector(&selector).await {
        let params = decode_with_signature(&signature, data);
        return json!({
            "selector": selector,
            "function": signature.split('(').next(),
            "signature": signature,
            "source": "signature_database",
            "params": params,
        });
    }
    json!({"selector": selector, "source": "none"})
}

/// How deep nested MultiSend batches are unpacked
const MAX_BATCH_DEPTH: usize = 4;

/// The inner payload when a call DELEGATECALLs a known MultiSend
fn nested_payload(operation: u8, to: Address, data: &[u8]) -> Option<Vec<u8>> {
    (operation == 1)
        .then(|| multisend_payload(to, data))
        .flatten()
}

/// Warnings for every call inside a MultiSend batch sent to `to`, at any
/// depth: DELEGATECALLs to unknown contracts, nested batches that cannot be
/// unpacked, and batches nested deeper than [`MAX_BATCH_DEPTH`]. `depth` is
/// the level of the batch in `data` (1 for the transaction's own batch).
pub fn batch_warnings(to: Address, data: &[u8], depth: usize) -> Vec<String> {
    let mut out = Vec::new();
    let Some(packed) = multisend_payload(to, data) else {
        return out;
    };
    scan_batch(&packed, depth, "", &mut out);
    out
}

fn scan_batch(packed: &[u8], depth: usize, path: &str, out: &mut Vec<String>) {
    let calls =
        match unpack_multisend(packed) {
            Ok(calls) => calls,
            Err(e) => {
                out.push(format!(
                "batch{} at depth {depth} could not be unpacked ({e}); its calls were not checked",
                if path.is_empty() { String::new() } else { format!(" {path}") }
            ));
                return;
            }
        };
    for (i, c) in calls.iter().enumerate() {
        let here = if path.is_empty() {
            format!("[{i}]")
        } else {
            format!("{path}[{i}]")
        };
        if let Some(w) = delegatecall_warning(c.operation, c.to) {
            out.push(format!("call {here}: {w}"));
        }
        if c.operation == 1 && c.data.len() >= 4 && c.data[..4] == MULTISEND_SELECTOR {
            match nested_payload(c.operation, c.to, &c.data) {
                Some(_) if depth >= MAX_BATCH_DEPTH => out.push(format!(
                    "call {here}: MultiSend nested deeper than {MAX_BATCH_DEPTH} levels; its calls were not checked"
                )),
                Some(inner) => scan_batch(&inner, depth + 1, &here, out),
                None if known_library(c.to).is_some_and(|(_, m)| m) => out.push(format!(
                    "call {here}: nested MultiSend payload could not be decoded; its calls were not checked"
                )),
                None => {}
            }
        }
    }
}

/// Decode a MultiSend batch, unpacking batches that are DELEGATECALLed from
/// inside it, down to [`MAX_BATCH_DEPTH`]
fn decode_batch<'a>(
    fetcher: &'a AbiFetcher,
    chain: Chain,
    packed: &'a [u8],
    depth: usize,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Vec<Value>>> + Send + 'a>> {
    Box::pin(async move {
        let calls = unpack_multisend(packed)?;
        let mut out = Vec::with_capacity(calls.len());
        for c in calls {
            let mut decoded = decode_call(fetcher, chain, c.to, &c.data).await;
            if let Some(inner) = nested_payload(c.operation, c.to, &c.data) {
                if depth >= MAX_BATCH_DEPTH {
                    decoded["multisend_error"] = json!(format!(
                        "nested deeper than {MAX_BATCH_DEPTH} levels; not unpacked"
                    ));
                } else {
                    match decode_batch(fetcher, chain, &inner, depth + 1).await {
                        Ok(nested) => decoded["multisend"] = json!(nested),
                        Err(e) => decoded["multisend_error"] = json!(e.to_string()),
                    }
                }
            }
            out.push(json!({
                "operation": operation_name(c.operation),
                "to": c.to.to_checksum(None),
                "value": c.value.to_string(),
                "data": format!("0x{}", hex::encode(&c.data)),
                "call": decoded,
                "delegatecall_warning": delegatecall_warning(c.operation, c.to),
            }));
        }
        Ok(out)
    })
}

async fn decode_many(fetcher: &AbiFetcher, chain: Chain, txs: &[Value]) -> Vec<Value> {
    futures::stream::iter(txs.iter().map(|tx| async move {
        let to = parse_address(&text(&tx["to"])).unwrap_or_default();
        let data = hex_bytes(&tx["data"]);
        let operation = tx["operation"].as_u64().unwrap_or(0) as u8;
        let mut call = decode_call(fetcher, chain, to, &data).await;
        if let Some(inner) = nested_payload(operation, to, &data) {
            if let Ok(calls) = unpack_multisend(&inner) {
                call["multisend_calls"] = json!(calls.len());
            }
        }
        let warnings = top_level_warnings(operation, to, &data);
        if !warnings.is_empty() {
            call["batch_warnings"] = json!(warnings);
        }
        call
    }))
    .buffered(DECODE_CONCURRENCY)
    .collect()
    .await
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_address(s: &str) -> anyhow::Result<Address> {
    s.trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid address: {s}"))
}

fn text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn hex_bytes(v: &Value) -> Vec<u8> {
    v.as_str()
        .and_then(|s| hex::decode(s.trim_start_matches("0x")).ok())
        .unwrap_or_default()
}

fn confirmations(tx: &Value) -> Vec<String> {
    tx["confirmations"]
        .as_array()
        .map(|list| list.iter().map(|c| text(&c["owner"])).collect())
        .unwrap_or_default()
}

/// Confirmations split against the current on-chain owners
#[derive(Debug, Default, PartialEq, Eq)]
struct Signers {
    /// Distinct current owners who confirmed, checksummed
    signed: Vec<String>,
    /// Current owners who have not confirmed
    not_signed: Vec<String>,
    /// Confirmations from addresses that are not current owners; they do not count
    stale: Vec<String>,
}

fn signers(tx: &Value, owners: &[Address]) -> Signers {
    let mut out = Signers::default();
    let mut confirmed: Vec<Address> = Vec::new();
    for raw in confirmations(tx) {
        match raw.trim().parse::<Address>() {
            Ok(a) if owners.contains(&a) => {
                if !confirmed.contains(&a) {
                    confirmed.push(a);
                    out.signed.push(a.to_checksum(None));
                }
            }
            Ok(a) => {
                let a = a.to_checksum(None);
                if !out.stale.contains(&a) {
                    out.stale.push(a);
                }
            }
            Err(_) => out.stale.push(raw),
        }
    }
    out.not_signed = owners
        .iter()
        .filter(|o| !confirmed.contains(o))
        .map(|o| o.to_checksum(None))
        .collect();
    out
}

/// Ready to execute: distinct current owners who confirmed reach the
/// threshold. Stale signers never count; a zero threshold is never ready.
fn is_ready(s: &Signers, threshold: U256) -> bool {
    !threshold.is_zero() && U256::from(s.signed.len()) >= threshold
}

fn summarize(tx: &Value, call: Value) -> Value {
    let op = tx["operation"].as_u64().unwrap_or(0) as u8;
    let warning = parse_address(&text(&tx["to"]))
        .ok()
        .and_then(|to| delegatecall_warning(op, to));
    json!({
        "delegatecall_warning": warning,
        "nonce": text(&tx["nonce"]),
        "safe_tx_hash": tx["safeTxHash"],
        "to": tx["to"],
        "value": text(&tx["value"]),
        "operation": operation_name(tx["operation"].as_u64().unwrap_or(0) as u8),
        "call": call,
        "submitted": tx["submissionDate"],
        "proposer": tx["proposer"],
        "refund": refund_fields(tx),
        "refund_warning": refund_warning(tx),
    })
}

/// The gas-refund fields of a Safe transaction. With a non-zero gasPrice
/// the Safe pays `(gasUsed + baseGas) * gasPrice` of gasToken (ETH when
/// zero) to refundReceiver (tx.origin when zero) on execution.
fn refund_fields(tx: &Value) -> Value {
    json!({
        "safe_tx_gas": text(&tx["safeTxGas"]),
        "base_gas": text(&tx["baseGas"]),
        "gas_price": text(&tx["gasPrice"]),
        "gas_token": tx["gasToken"],
        "refund_receiver": tx["refundReceiver"],
    })
}

fn is_zero_text(v: &Value) -> bool {
    let t = text(v);
    let t = t.trim();
    t.is_empty()
        || t.trim_start_matches("0x")
            .trim_start_matches('0')
            .is_empty()
}

/// A warning when the transaction pays a gas refund out of the Safe
pub fn refund_warning(tx: &Value) -> Option<String> {
    let price_set = !is_zero_text(&tx["gasPrice"]);
    let receiver_set = !is_zero_text(&tx["refundReceiver"]);
    if !price_set && !receiver_set {
        return None;
    }
    let token = if is_zero_text(&tx["gasToken"]) {
        "ETH".to_string()
    } else {
        text(&tx["gasToken"])
    };
    let receiver = if receiver_set {
        text(&tx["refundReceiver"])
    } else {
        "the executor (tx.origin)".to_string()
    };
    Some(if price_set {
        format!(
            "gas refund: the Safe pays (gasUsed + baseGas {}) * gasPrice {} in {token} to {receiver}",
            text(&tx["baseGas"]),
            text(&tx["gasPrice"])
        )
    } else {
        format!("refundReceiver is set to {receiver} (gasPrice is 0, so no refund is paid unless it changes)")
    })
}

const SAFE_TX_TYPEHASH: &str = "SafeTx(address to,uint256 value,bytes data,uint8 operation,uint256 safeTxGas,uint256 baseGas,uint256 gasPrice,address gasToken,address refundReceiver,uint256 nonce)";

/// The EIP-712 safeTxHash for the fields in a service record, under both
/// domain layouts: with chainId (Safe >= 1.3.0) and without (older Safes)
pub fn compute_safe_tx_hashes(tx: &Value, chain_id: u64) -> Option<[B256; 2]> {
    let addr = |k: &str| -> Option<Address> {
        let t = text(&tx[k]);
        if t.is_empty() {
            Some(Address::ZERO)
        } else {
            t.parse().ok()
        }
    };
    let uint = |k: &str| -> Option<U256> {
        let t = text(&tx[k]);
        if t.is_empty() {
            Some(U256::ZERO)
        } else {
            t.parse().ok()
        }
    };
    let safe = addr("safe")?;
    let data = match tx["data"].as_str() {
        None => Vec::new(),
        Some(d) => hex::decode(d.trim_start_matches("0x")).ok()?,
    };
    let operation = tx["operation"].as_u64()?;
    let struct_hash = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::FixedBytes(keccak256(SAFE_TX_TYPEHASH.as_bytes()), 32),
            DynSolValue::Address(addr("to")?),
            DynSolValue::Uint(uint("value")?, 256),
            DynSolValue::FixedBytes(keccak256(&data), 32),
            DynSolValue::Uint(U256::from(operation), 8),
            DynSolValue::Uint(uint("safeTxGas")?, 256),
            DynSolValue::Uint(uint("baseGas")?, 256),
            DynSolValue::Uint(uint("gasPrice")?, 256),
            DynSolValue::Address(addr("gasToken")?),
            DynSolValue::Address(addr("refundReceiver")?),
            DynSolValue::Uint(uint("nonce")?, 256),
        ])
        .abi_encode_params(),
    );
    let domain_with_chain = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::FixedBytes(
                keccak256("EIP712Domain(uint256 chainId,address verifyingContract)".as_bytes()),
                32,
            ),
            DynSolValue::Uint(U256::from(chain_id), 256),
            DynSolValue::Address(safe),
        ])
        .abi_encode_params(),
    );
    let domain_legacy = keccak256(
        DynSolValue::Tuple(vec![
            DynSolValue::FixedBytes(
                keccak256("EIP712Domain(address verifyingContract)".as_bytes()),
                32,
            ),
            DynSolValue::Address(safe),
        ])
        .abi_encode_params(),
    );
    let digest = |domain: B256| {
        let mut buf = vec![0x19, 0x01];
        buf.extend_from_slice(domain.as_slice());
        buf.extend_from_slice(struct_hash.as_slice());
        keccak256(&buf)
    };
    Some([digest(domain_with_chain), digest(domain_legacy)])
}

/// Checks that the service record is the one asked for and that its
/// safeTxHash matches its own fields
pub fn integrity_warnings(
    tx: &Value,
    chain_id: u64,
    expected_safe: Option<Address>,
    requested_hash: Option<B256>,
) -> Vec<String> {
    let mut out = Vec::new();
    let returned: Option<B256> = text(&tx["safeTxHash"]).parse().ok();
    if let Some(requested) = requested_hash {
        if returned != Some(requested) {
            out.push(format!(
                "the service returned safeTxHash {} for requested {requested:#x}",
                text(&tx["safeTxHash"])
            ));
        }
    }
    if let Some(expected) = expected_safe {
        if text(&tx["safe"]).parse::<Address>().ok() != Some(expected) {
            out.push(format!(
                "the service returned a transaction of Safe {} for {}",
                text(&tx["safe"]),
                expected.to_checksum(None)
            ));
        }
    }
    match compute_safe_tx_hashes(tx, chain_id) {
        Some(hashes) if returned.is_some_and(|h| hashes.contains(&h)) => {}
        Some(hashes) => out.push(format!(
            "safeTxHash does not match the transaction's fields (recomputed {:#x}); the service data cannot be trusted",
            hashes[0]
        )),
        None => out.push("could not recompute safeTxHash from the service fields".to_string()),
    }
    out
}

/// Warnings about the transaction's own call: a CALL to a MultiSend (not a
/// normal batch), a DELEGATECALLed batch that cannot be decoded, and every
/// finding inside the batch
pub fn top_level_warnings(operation: u8, to: Address, data: &[u8]) -> Vec<String> {
    let is_multisend_call = data.len() >= 4
        && data[..4] == MULTISEND_SELECTOR
        && known_library(to).is_some_and(|(_, m)| m);
    if !is_multisend_call {
        return Vec::new();
    }
    if operation != 1 {
        return vec![
            "CALL (not DELEGATECALL) to a MultiSend library: not a normal Safe batch; MultiSend reverts, MultiSendCallOnly would run the inner calls from the library rather than the Safe. Batch not unpacked".to_string(),
        ];
    }
    if multisend_payload(to, data).is_none() {
        return vec!["multiSend calldata to a known MultiSend could not be decoded; its calls were not checked".to_string()];
    }
    batch_warnings(to, data, 1)
}

/// Text from the service or a contract, made safe to print in a terminal
fn st(v: &Value) -> String {
    crate::abi::revert::safe_text(&text(v))
}

fn safe_text_s(s: &str) -> String {
    crate::abi::revert::safe_text(s)
}

fn print_json(out: &Value, format: OutputFormat) -> anyhow::Result<()> {
    if format.is_ndjson() {
        println!("{}", serde_json::to_string(out)?);
    } else {
        println!("{}", serde_json::to_string_pretty(out)?);
    }
    Ok(())
}

fn method_label(call: &Value) -> String {
    use crate::abi::revert::safe_text;
    let mut label = match (call["function"].as_str(), call["selector"].as_str()) {
        (Some(f), _) => safe_text(f),
        (None, Some(s)) => safe_text(s),
        _ => safe_text(call["kind"].as_str().unwrap_or("-")),
    };
    if let Some(n) = call["multisend_calls"].as_u64() {
        label.push_str(&format!(" [{n} calls]"));
    }
    if call["source"] == "signature_database" {
        label.push('?');
    }
    label
}

fn print_info(out: &Value) {
    let s = |v: &Value| {
        v.as_str()
            .map(crate::abi::revert::safe_text)
            .unwrap_or("-".to_string())
    };
    println!(
        "Safe:             {} ({})",
        s(&out["address"]),
        s(&out["chain"])
    );
    println!("Version:          {}", s(&out["version"]));
    println!(
        "Threshold:        {} of {} owners",
        s(&out["threshold"]),
        out["owners"]
            .as_array()
            .map_or("unknown".to_string(), |a| a.len().to_string())
    );
    println!(
        "Nonce:            {}",
        out["nonce"]
            .as_str()
            .map(safe_text_s)
            .unwrap_or("unknown (read failed)".to_string())
    );
    println!("Singleton:        {}", s(&out["singleton"]));
    println!("Guard:            {}", s(&out["guard"]));
    println!("Fallback handler: {}", s(&out["fallback_handler"]));
    println!("Block:            {}", s(&out["block"]));
    match out["owners"].as_array() {
        Some(owners) => {
            println!("Owners:");
            for o in owners {
                println!("  {}", s(o));
            }
        }
        None => println!("Owners:           unknown (read failed)"),
    }
    match out["modules"].as_array() {
        None => println!(
            "Modules:          unknown (read failed): check modules before trusting this Safe"
        ),
        Some(modules) if modules.is_empty() => println!("Modules:          none"),
        Some(modules) => {
            println!("Modules:");
            for m in modules {
                println!("  {}", s(m));
            }
        }
    }
    if out["modules_truncated"] == true {
        println!("WARNING: more than {MODULE_PAGE} modules are enabled; only the first {MODULE_PAGE} are listed");
    }
}

fn print_queue(out: &Value) {
    let txs = out["transactions"].as_array().cloned().unwrap_or_default();
    println!(
        "Safe {} on {}: nonce {}, threshold {} of {}; {} pending",
        st(&out["safe"]),
        st(&out["chain"]),
        st(&out["nonce"]),
        st(&out["threshold"]),
        out["owners"],
        txs.len()
    );
    for tx in &txs {
        println!();
        println!(
            "nonce {}  {}/{} signed{}{}  {}",
            st(&tx["nonce"]),
            tx["confirmations"],
            st(&tx["threshold"]),
            if tx["ready"] == true {
                "  READY"
            } else if tx["signatures_complete"] == true {
                "  SIGNED (not current nonce)"
            } else {
                ""
            },
            if tx["conflicting_proposals"] == true {
                "  (other proposals at this nonce)"
            } else {
                ""
            },
            st(&tx["safe_tx_hash"]),
        );
        println!(
            "  {} -> {}  value {}  {}",
            method_label(&tx["call"]),
            st(&tx["to"]),
            st(&tx["value"]),
            st(&tx["operation"])
        );
        let join = |v: &Value| {
            v.as_array()
                .map(|a| a.iter().map(st).collect::<Vec<_>>().join(", "))
                .unwrap_or_default()
        };
        if let Some(w) = tx["delegatecall_warning"].as_str() {
            println!("  WARNING: {}", safe_text_s(w));
        }
        if let Some(w) = tx["call"]["warning"].as_str() {
            println!("  WARNING: {}", safe_text_s(w));
        }
        print_batch_warnings(&tx["call"], "  ");
        print_list_warnings(tx, "  ");
        print_refund(tx, "  ");
        println!("  signed:     {}", join(&tx["signed"]));
        if tx["stale_signers"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
        {
            println!(
                "  stale (not owners, not counted): {}",
                join(&tx["stale_signers"])
            );
        }
        println!("  not signed: {}", join(&tx["not_signed"]));
    }
}

fn print_call(call: &Value, indent: &str) {
    if let Some(w) = call["warning"].as_str() {
        println!("{indent}WARNING: {}", safe_text_s(w));
    }
    if let Some(lib) = call["library"].as_str() {
        println!("{indent}library:   {}", safe_text_s(lib));
    }
    println!("{indent}function:  {}", method_label(call));
    if let Some(sig) = call["signature"].as_str() {
        println!(
            "{indent}signature: {} ({})",
            crate::abi::revert::safe_text(sig),
            st(&call["source"])
        );
    }
    for p in call["params"].as_array().into_iter().flatten() {
        let value = match &p["value"] {
            Value::String(s) => crate::abi::revert::safe_text(s),
            other => other.to_string(),
        };
        println!(
            "{indent}  {} {} = {}",
            st(&p["type"]),
            st(&p["name"]),
            value
        );
    }
}

fn print_tx(out: &Value) {
    println!("safeTxHash: {}", st(&out["safe_tx_hash"]));
    println!("Safe:       {} ({})", st(&out["safe"]), st(&out["chain"]));
    println!("Nonce:      {}", st(&out["nonce"]));
    println!(
        "To:         {}  ({})",
        st(&out["to"]),
        st(&out["operation"])
    );
    println!("Value:      {}", st(&out["value"]));
    if let Some(w) = out["delegatecall_warning"].as_str() {
        println!("WARNING:    {}", safe_text_s(w));
    }
    print_list_warnings(out, "");
    print_refund(out, "");
    print_batch_warnings(&out["call"], "");
    print_call(&out["call"], "  ");
    print_batch(&out["call"], "");
    println!(
        "Status:     {}",
        if out["executed"] == true {
            format!(
                "executed {} ({}), tx {}",
                st(&out["execution_date"]),
                if out["successful"] == true {
                    "success"
                } else {
                    "failed"
                },
                st(&out["transaction_hash"])
            )
        } else {
            "pending".to_string()
        }
    );
    match out["threshold"].as_str() {
        Some(threshold) => println!(
            "Signed:     {} of {} (on-chain threshold; service says {}){}",
            out["confirmations"],
            safe_text_s(threshold),
            st(&out["confirmations_required"]),
            if out["ready"] == true {
                "  READY"
            } else if out["signatures_complete"] == true {
                "  SIGNED (not current nonce)"
            } else {
                ""
            }
        ),
        None => println!(
            "Signed:     {} (service says {} required; on-chain threshold not read)",
            out["confirmations"],
            st(&out["confirmations_required"])
        ),
    }
    for s in out["signed"].as_array().into_iter().flatten() {
        println!("  {}", st(s));
    }
    if let Some(stale) = out["stale_signers"].as_array().filter(|a| !a.is_empty()) {
        println!("Stale (not current owners, not counted):");
        for s in stale {
            println!("  {}", st(s));
        }
    }
    if let Some(missing) = out["not_signed"].as_array() {
        println!("Not signed:");
        for s in missing {
            println!("  {}", st(s));
        }
    }
}

fn print_list_warnings(item: &Value, indent: &str) {
    for w in item["warnings"].as_array().into_iter().flatten() {
        println!("{indent}WARNING: {}", st(w));
    }
    if let Some(w) = item["refund_warning"].as_str() {
        println!("{indent}WARNING: {}", safe_text_s(w));
    }
}

fn print_refund(item: &Value, indent: &str) {
    let r = &item["refund"];
    if r.is_null() {
        return;
    }
    println!(
        "{indent}refund: safeTxGas {} baseGas {} gasPrice {} gasToken {} refundReceiver {}",
        st(&r["safe_tx_gas"]),
        st(&r["base_gas"]),
        st(&r["gas_price"]),
        st(&r["gas_token"]),
        st(&r["refund_receiver"])
    );
}

fn print_batch_warnings(call: &Value, indent: &str) {
    for w in call["batch_warnings"].as_array().into_iter().flatten() {
        println!("{indent}WARNING: {}", st(w));
    }
}

/// A MultiSend batch and any batches nested inside it
fn print_batch(call: &Value, indent: &str) {
    if let Some(e) = call["multisend_error"].as_str() {
        println!("{indent}WARNING: batch not unpacked: {}", safe_text_s(e));
    }
    let Some(calls) = call["multisend"].as_array() else {
        return;
    };
    println!("{indent}MultiSend: {} calls", calls.len());
    for (i, c) in calls.iter().enumerate() {
        println!(
            "{indent}  [{i}] {} {} value {}",
            st(&c["operation"]),
            st(&c["to"]),
            st(&c["value"])
        );
        if let Some(w) = c["delegatecall_warning"].as_str() {
            println!("{indent}      WARNING: {}", safe_text_s(w));
        }
        let inner = format!("{indent}      ");
        print_call(&c["call"], &inner);
        print_batch(&c["call"], &format!("{indent}    "));
    }
}

fn print_history(out: &Value) {
    println!(
        "Safe {} on {}: {} of {} executed transactions (offset {})",
        st(&out["safe"]),
        st(&out["chain"]),
        out["count"],
        out["total"],
        out["offset"]
    );
    println!(
        "{:<7} {:<20} {:<4} {:<32} {:<42} TX",
        "NONCE", "EXECUTED", "OK", "METHOD", "TO"
    );
    for tx in out["transactions"].as_array().into_iter().flatten() {
        let date: String = st(&tx["execution_date"]).chars().take(19).collect();
        let mut method = method_label(&tx["call"]);
        if method.chars().count() > 32 {
            method = method.chars().take(31).collect();
            method.push('…');
        }
        println!(
            "{:<7} {:<20} {:<4} {:<32} {:<42} {}",
            st(&tx["nonce"]),
            date,
            if tx["successful"] == true {
                "yes"
            } else {
                "no"
            },
            method,
            st(&tx["to"]),
            st(&tx["transaction_hash"])
        );
        if let Some(w) = tx["delegatecall_warning"].as_str() {
            println!("        WARNING: {}", safe_text_s(w));
        }
        print_batch_warnings(&tx["call"], "        ");
        print_list_warnings(tx, "        ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack(op: u8, to: Address, value: u64, data: &[u8]) -> Vec<u8> {
        let mut out = vec![op];
        out.extend_from_slice(to.as_slice());
        out.extend_from_slice(&U256::from(value).to_be_bytes::<32>());
        out.extend_from_slice(&U256::from(data.len()).to_be_bytes::<32>());
        out.extend_from_slice(data);
        out
    }

    fn transfer_calldata() -> Vec<u8> {
        // transfer(0x00..01, 1000)
        let mut data = vec![0xa9, 0x05, 0x9c, 0xbb];
        data.extend(
            DynSolValue::Tuple(vec![
                DynSolValue::Address(Address::with_last_byte(1)),
                DynSolValue::Uint(U256::from(1000), 256),
            ])
            .abi_encode_params(),
        );
        data
    }

    #[test]
    fn unpacks_multisend_and_decodes_inner_calls() {
        let usdc: Address = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            .parse()
            .unwrap();
        let other = Address::with_last_byte(0x42);
        let mut packed = pack(0, usdc, 0, &transfer_calldata());
        packed.extend(pack(1, other, 5, &[]));

        // multiSend(bytes) calldata around it
        let mut calldata = MULTISEND_SELECTOR.to_vec();
        calldata.extend(
            DynSolValue::Tuple(vec![DynSolValue::Bytes(packed.clone())]).abi_encode_params(),
        );

        let ms: Address = "0x40A2aCCbd92BCA938b02010E17A5b8929b49130D"
            .parse()
            .unwrap();
        // Same calldata sent to an unknown contract is not unpacked
        assert!(multisend_payload(Address::with_last_byte(9), &calldata).is_none());
        let payload = multisend_payload(ms, &calldata).expect("multisend");
        assert_eq!(payload, packed);
        let calls = unpack_multisend(&payload).unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].operation, 0);
        assert_eq!(calls[0].to, usdc);
        assert_eq!(calls[0].value, U256::ZERO);
        assert_eq!(calls[0].data, transfer_calldata());
        assert_eq!(calls[1].operation, 1);
        assert_eq!(calls[1].to, other);
        assert_eq!(calls[1].value, U256::from(5));
        assert!(calls[1].data.is_empty());

        let params = decode_with_signature("transfer(address,uint256)", &calls[0].data).unwrap();
        assert_eq!(params[0]["type"], "address");
        assert_eq!(
            params[0]["value"],
            "0x0000000000000000000000000000000000000001"
        );
        assert_eq!(params[1]["type"], "uint256");
        assert_eq!(params[1]["value"], "1000");
    }

    #[test]
    fn rejects_truncated_multisend() {
        let mut packed = pack(0, Address::with_last_byte(1), 0, &[1, 2, 3, 4]);
        packed.pop();
        assert!(unpack_multisend(&packed).is_err());
        assert!(unpack_multisend(&packed[..40]).is_err());
        assert!(unpack_multisend(&[]).unwrap().is_empty());
    }

    #[test]
    fn non_multisend_calldata_is_not_unpacked() {
        let ms: Address = "0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761"
            .parse()
            .unwrap();
        assert!(multisend_payload(ms, &transfer_calldata()).is_none());
        assert!(multisend_payload(ms, &[]).is_none());
    }

    #[test]
    fn maps_service_chain_names() {
        assert_eq!(service_chain_name(1), Some("eth"));
        assert_eq!(service_chain_name(42161), Some("arb1"));
        assert_eq!(service_chain_name(137), Some("pol"));
        assert_eq!(service_chain_name(999_999), None);
    }

    #[test]
    fn signers_split_owners_and_ignore_stale_and_duplicates() {
        let a = Address::with_last_byte(0xaa);
        let b = Address::with_last_byte(0xbb);
        let stranger = Address::with_last_byte(0xcc);
        let tx = json!({"confirmations": [
            {"owner": a.to_checksum(None).to_lowercase()},
            {"owner": a.to_checksum(None)},
            {"owner": stranger.to_checksum(None)},
        ]});
        let s = signers(&tx, &[a, b]);
        assert_eq!(s.signed, vec![a.to_checksum(None)]);
        assert_eq!(s.not_signed, vec![b.to_checksum(None)]);
        assert_eq!(s.stale, vec![stranger.to_checksum(None)]);
        // threshold 2: one owner plus a duplicate and a stale signer is not ready
        assert!(!is_ready(&s, U256::from(2)));
        assert!(is_ready(&s, U256::from(1)));
        assert!(!is_ready(&s, U256::ZERO));
    }

    #[test]
    fn known_libraries_and_delegatecall_warning() {
        let ms: Address = "0x9641d764fc13c8B624c04430C7356C1C7C8102e2"
            .parse()
            .unwrap();
        let sign: Address = "0xd53cd0aB83D845Ac265BE939c57F53AD838012c9"
            .parse()
            .unwrap();
        assert_eq!(known_library(ms).map(|(_, m)| m), Some(true));
        assert_eq!(known_library(sign).map(|(_, m)| m), Some(false));
        for (addr, _, _) in KNOWN_DELEGATE_TARGETS {
            assert!(addr.parse::<Address>().is_ok(), "{addr}");
        }
        assert!(delegatecall_warning(1, ms).is_none());
        assert!(delegatecall_warning(1, sign).is_none());
        assert!(delegatecall_warning(0, Address::with_last_byte(7)).is_none());
        assert!(delegatecall_warning(1, Address::with_last_byte(7)).is_some());
    }

    #[test]
    fn method_label_truncation_is_char_safe() {
        let call = json!({"function": "é".repeat(40), "source": "abi"});
        let mut method = method_label(&call);
        if method.chars().count() > 32 {
            method = method.chars().take(31).collect();
            method.push('…');
        }
        assert_eq!(method.chars().count(), 32);
    }

    #[test]
    fn table_text_strips_escapes() {
        let v = json!("evil\u{1b}[31mred");
        assert!(!st(&v).contains('\u{1b}'));
    }

    fn multisend_call(packed: Vec<u8>) -> Vec<u8> {
        let mut data = MULTISEND_SELECTOR.to_vec();
        data.extend(DynSolValue::Tuple(vec![DynSolValue::Bytes(packed)]).abi_encode_params());
        data
    }

    #[test]
    fn nested_unknown_delegatecall_is_warned() {
        let ms: Address = "0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526"
            .parse()
            .unwrap();
        let evil = Address::with_last_byte(0xee);
        // level 3: delegatecall to an unknown contract
        let level3 = pack(1, evil, 0, &[0xde, 0xad, 0xbe, 0xef]);
        // level 2: delegatecall to MultiSend wrapping level 3
        let level2 = pack(1, ms, 0, &multisend_call(level3));
        // level 1 (the tx's batch): a plain call, then a delegatecall to MultiSend wrapping level 2
        let mut level1 = pack(0, Address::with_last_byte(1), 0, &[]);
        level1.extend(pack(1, ms, 0, &multisend_call(level2)));
        let warnings = batch_warnings(ms, &multisend_call(level1), 1);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].starts_with("call [1][0][0]:"),
            "{}",
            warnings[0]
        );
        assert!(warnings[0].contains(&evil.to_checksum(None)));
    }

    #[test]
    fn depth_limit_is_warned() {
        let ms: Address = "0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526"
            .parse()
            .unwrap();
        let mut packed = pack(0, Address::with_last_byte(1), 0, &[]);
        for _ in 0..MAX_BATCH_DEPTH {
            packed = pack(1, ms, 0, &multisend_call(packed));
        }
        let warnings = batch_warnings(ms, &multisend_call(packed), 1);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains("nested deeper than"),
            "{}",
            warnings[0]
        );
    }

    #[test]
    fn broken_nested_batch_is_warned() {
        let ms: Address = "0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526"
            .parse()
            .unwrap();
        let mut bad = pack(0, Address::with_last_byte(1), 0, &[1, 2, 3]);
        bad.pop();
        let level1 = pack(1, ms, 0, &multisend_call(bad));
        let warnings = batch_warnings(ms, &multisend_call(level1), 1);
        assert_eq!(warnings.len(), 1, "{warnings:?}");
        assert!(
            warnings[0].contains("could not be unpacked"),
            "{}",
            warnings[0]
        );
    }

    fn live_queued_tx() -> Value {
        json!({
            "safe": "0xFEB4acf3df3cDEA7399794D0869ef76A6EfAff52",
            "to": "0x40A2aCCbd92BCA938b02010E17A5b8929b49130D",
            "value": "0",
            "data": "0x8d80ff0a0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000009900a0b86991c6218b36c1d19d4a2e9eb0ce3606eb4800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044a9059cbb000000000000000000000000b9d026f3f383872922d2c3bebfbfa2affa06ec24000000000000000000000000000000000000000000000000000000001dcd650000000000000000",
            "operation": 1,
            "safeTxGas": "0",
            "baseGas": "0",
            "gasPrice": "0",
            "gasToken": "0x0000000000000000000000000000000000000000",
            "refundReceiver": "0x0000000000000000000000000000000000000000",
            "nonce": "2308",
            "safeTxHash": "0x4865c6e169a13fcf0e80f37122728e051857fb18737e11a5633390375fc52e0e",
        })
    }

    #[test]
    fn recomputes_safe_tx_hash_of_a_real_transaction() {
        let tx = live_queued_tx();
        let hash: B256 = text(&tx["safeTxHash"]).parse().unwrap();
        let safe: Address = text(&tx["safe"]).parse().unwrap();
        assert_eq!(compute_safe_tx_hashes(&tx, 1).unwrap()[0], hash);
        assert!(integrity_warnings(&tx, 1, Some(safe), Some(hash)).is_empty());
        assert!(refund_warning(&tx).is_none());
    }

    #[test]
    fn flags_tampered_or_mismatched_service_records() {
        let mut tx = live_queued_tx();
        let hash: B256 = text(&tx["safeTxHash"]).parse().unwrap();
        // a field changed under the same hash
        tx["value"] = json!("1");
        let w = integrity_warnings(&tx, 1, None, Some(hash));
        assert!(w.iter().any(|w| w.contains("does not match")), "{w:?}");
        // a different hash and Safe than requested
        let tx = live_queued_tx();
        let other_hash = B256::repeat_byte(7);
        let other_safe = Address::with_last_byte(5);
        let w = integrity_warnings(&tx, 1, Some(other_safe), Some(other_hash));
        assert!(w.iter().any(|w| w.contains("returned safeTxHash")), "{w:?}");
        assert!(w.iter().any(|w| w.contains("of Safe")), "{w:?}");
        // wrong chain id: the hash does not match
        assert!(!integrity_warnings(&tx, 10, None, None).is_empty());
    }

    #[test]
    fn warns_on_gas_refunds() {
        let mut tx = live_queued_tx();
        tx["gasPrice"] = json!("1000000000");
        tx["baseGas"] = json!("50000");
        let w = refund_warning(&tx).unwrap();
        assert!(w.contains("ETH") && w.contains("tx.origin"), "{w}");
        let mut tx = live_queued_tx();
        tx["refundReceiver"] = json!("0x00000000000000000000000000000000000000ee");
        assert!(refund_warning(&tx).is_some());
        let r = refund_fields(&live_queued_tx());
        assert_eq!(r["gas_price"], "0");
        assert_eq!(r["safe_tx_gas"], "0");
    }

    #[test]
    fn top_level_multisend_checks() {
        let ms: Address = "0x38869bf66a61cF6bDB996A6aE40D5853Fd43B526"
            .parse()
            .unwrap();
        let batch = multisend_call(pack(0, Address::with_last_byte(1), 0, &[]));
        // normal delegatecall batch: no warnings
        assert!(top_level_warnings(1, ms, &batch).is_empty());
        // CALL to MultiSend: warned, not unpacked
        let w = top_level_warnings(0, ms, &batch);
        assert!(w[0].contains("CALL (not DELEGATECALL)"), "{w:?}");
        assert!(nested_payload(0, ms, &batch).is_none());
        // undecodable multiSend calldata: warned
        let cut = &batch[..batch.len() - 40];
        let w = top_level_warnings(1, ms, cut);
        assert!(w[0].contains("could not be decoded"), "{w:?}");
    }

    #[test]
    fn decodes_module_pages() {
        let page = |next: Address| {
            DynSolValue::Tuple(vec![
                DynSolValue::Array(vec![DynSolValue::Address(Address::with_last_byte(9))]),
                DynSolValue::Address(next),
            ])
            .abi_encode_params()
        };
        let (list, next) = decode_modules_page(&page(SENTINEL)).unwrap();
        assert_eq!(list, vec![Address::with_last_byte(9)]);
        assert_eq!(next, SENTINEL);
        let (_, next) = decode_modules_page(&page(Address::with_last_byte(9))).unwrap();
        assert!(next != SENTINEL && !next.is_zero());
        assert!(decode_modules_page(&[1, 2, 3]).is_none());
    }
}
