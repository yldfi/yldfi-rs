//! Transaction analyzer
//!
//! Coordinates fetching and analyzing Ethereum transactions.

use crate::abi::AbiFetcher;
use crate::config::Chain;
use crate::error::Result;
use crate::rpc::RpcPool;
use crate::tx::addresses::{events, get_contract_info, get_label};
use crate::tx::flow::parse_transfers;
use crate::tx::types::{
    AnalyzedEvent, ContractCategory, ContractInfo, EventParam, FunctionCall, FunctionParam,
    RawTxData, TransactionAnalysis,
};
use alloy::consensus::Transaction as TxTrait;
use alloy::primitives::U256;
use alloy::primitives::{Address, B256};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

/// Convert U256 to f64 with ETH decimals (HIGH-001 fix)
///
/// Handles arbitrarily large values by:
/// 1. Using u128 conversion when possible for precision
/// 2. Falling back to string conversion for very large values
/// 3. Saturating at f64::MAX to prevent overflow/panic
fn u256_to_eth_f64(value: &U256) -> f64 {
    const ETH_SCALE: f64 = 1e18;

    // Try u128 first for better precision (covers values up to ~340 undecillion wei)
    if let Ok(as_u128) = TryInto::<u128>::try_into(*value) {
        return (as_u128 as f64) / ETH_SCALE;
    }

    // For values > u128::MAX, convert via string (loses precision but handles any size)
    let s = value.to_string();
    match s.parse::<f64>() {
        Ok(v) if v.is_finite() => v / ETH_SCALE,
        _ => f64::MAX, // Saturate on overflow or parse failure
    }
}

/// Format token amount with decimals (LOW-002 fix)
///
/// Safely formats large token amounts without panicking on edge cases.
/// Returns a human-readable string with 2 decimal places.
fn format_token_amount(amount_str: &str, decimals: usize) -> String {
    // Handle empty or non-numeric input
    if amount_str.is_empty() || !amount_str.chars().all(|c| c.is_ascii_digit()) {
        return "0.00".to_string();
    }

    if amount_str.len() > decimals {
        let int_part = &amount_str[..amount_str.len() - decimals];
        let frac_part = &amount_str[amount_str.len() - decimals..];
        // Take first 2 decimal places
        let frac_display = if frac_part.len() >= 2 {
            &frac_part[..2]
        } else {
            frac_part
        };
        format!("{}.{}", int_part, frac_display)
    } else {
        // Amount is less than 1 whole token
        let padding = "0".repeat(decimals - amount_str.len());
        let padded = format!("{}{}", padding, amount_str);
        let frac_display = if padded.len() >= 2 {
            &padded[..2]
        } else {
            &padded
        };
        format!("0.{}", frac_display)
    }
}

/// Transaction analyzer
pub struct TxAnalyzer {
    /// RPC pool
    pool: RpcPool,
    /// Chain
    chain: Chain,
    /// ABI fetcher for Etherscan lookups (shared to avoid HTTP client recreation - PERF-002 fix)
    abi_fetcher: Arc<AbiFetcher>,
}

impl TxAnalyzer {
    /// Create a new analyzer
    pub fn new(pool: RpcPool, chain: Chain) -> Result<Self> {
        Ok(Self {
            pool,
            chain,
            abi_fetcher: Arc::new(AbiFetcher::new(None)?),
        })
    }

    /// Create a new analyzer with a shared AbiFetcher (PERF-002 fix)
    ///
    /// Use this when analyzing multiple transactions to avoid recreating
    /// HTTP clients for each transaction.
    pub fn with_fetcher(pool: RpcPool, chain: Chain, abi_fetcher: Arc<AbiFetcher>) -> Self {
        Self {
            pool,
            chain,
            abi_fetcher,
        }
    }

    /// Get a reference to the ABI fetcher for sharing with other analyzers
    pub fn abi_fetcher(&self) -> Arc<AbiFetcher> {
        Arc::clone(&self.abi_fetcher)
    }

    /// Analyze a transaction by hash (basic analysis)
    pub async fn analyze(&self, hash: &str) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, false).await
    }

    /// Analyze a transaction with enrichment from Etherscan (slower but more detailed)
    pub async fn analyze_enriched(&self, hash: &str) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, true).await
    }

    /// Internal analyze method
    async fn analyze_internal(&self, hash: &str, enrich: bool) -> Result<TransactionAnalysis> {
        // Parse transaction hash
        let hash = B256::from_str(hash)
            .map_err(|e| format!("Invalid transaction hash '{}': {}", hash, e))?;

        // Fetch raw data
        let raw = self.fetch_raw_data(hash).await?;

        // Build analysis
        let mut analysis = TransactionAnalysis::new(&raw);

        // Analyze contracts involved
        analysis.contracts = self.analyze_contracts(&raw);

        // Analyze events
        analysis.events = self.analyze_events(&raw);

        // Parse token flows
        analysis.token_flows = parse_transfers(&raw.logs);

        // Label token flows
        for flow in &mut analysis.token_flows {
            if flow.token_label.is_none() {
                flow.token_label = get_label(&flow.token).map(String::from);
            }
            if flow.from_label.is_none() {
                flow.from_label = get_label(&flow.from).map(String::from);
            }
            if flow.to_label.is_none() {
                flow.to_label = get_label(&flow.to).map(String::from);
            }
        }

        // Decode function call if there's input data
        if let Some(to) = analysis.to {
            let input = raw.tx.inner.input();
            if input.len() >= 4 {
                analysis.function_call = self.decode_function_call(&to, input, enrich).await;
            }
        }

        // Enrich unknown contracts if requested
        if enrich {
            self.enrich_contracts(&mut analysis.contracts).await;
            self.enrich_token_flows(&mut analysis.token_flows).await;
        }

        Ok(analysis)
    }

    /// Decode function call from input data
    async fn decode_function_call(
        &self,
        to: &Address,
        input: &[u8],
        fetch_abi: bool,
    ) -> Option<FunctionCall> {
        if input.len() < 4 {
            return None;
        }

        let selector = format!("0x{}", hex::encode(&input[..4]));

        if fetch_abi {
            // Try to decode using Etherscan ABI
            if let Some(decoded) = self
                .abi_fetcher
                .decode_function_call(self.chain, &format!("{:#x}", to), input)
                .await
            {
                return Some(FunctionCall {
                    selector: decoded.selector,
                    name: decoded.name,
                    signature: decoded.signature,
                    params: decoded
                        .params
                        .into_iter()
                        .map(|(name, ty, value)| FunctionParam { name, ty, value })
                        .collect(),
                });
            }
        }

        // Try 4byte.directory lookup as fallback
        if let Some(sig) = self.abi_fetcher.lookup_selector(&selector).await {
            let name = sig.split('(').next().map(String::from);
            return Some(FunctionCall {
                selector,
                name,
                signature: Some(sig),
                params: Vec::new(),
            });
        }

        // Return unknown function with just selector
        Some(FunctionCall {
            selector,
            name: None,
            signature: None,
            params: Vec::new(),
        })
    }

    /// Enrich unknown contracts with Etherscan metadata
    async fn enrich_contracts(&self, contracts: &mut [ContractInfo]) {
        use futures::stream::{self, StreamExt};

        // MED-004 fix: Limit concurrency to prevent resource exhaustion
        const MAX_CONCURRENT_REQUESTS: usize = 10;

        // Collect contracts that need lookup
        let lookup_indices: Vec<usize> = contracts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.label.is_none() && c.address != Address::ZERO)
            .map(|(i, _)| i)
            .collect();

        if lookup_indices.is_empty() {
            return;
        }

        // Fetch metadata with bounded concurrency
        let results: Vec<_> = stream::iter(lookup_indices.iter().map(|&i| {
            let address = contracts[i].address;
            async move {
                let result = self
                    .abi_fetcher
                    .get_contract_metadata(self.chain, &format!("{:#x}", address))
                    .await;
                (i, result)
            }
        }))
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect()
        .await;

        // Apply results
        for (i, result) in results {
            if let Ok(metadata) = result {
                if let Some(name) = metadata.name {
                    contracts[i].label = Some(name);
                    // If verified, it's likely a contract (not EOA)
                    if metadata.is_verified && contracts[i].category == ContractCategory::Unknown {
                        contracts[i].category = ContractCategory::Protocol;
                    }
                }
            }
        }
    }

    /// Enrich token flows with token metadata
    async fn enrich_token_flows(&self, flows: &mut [crate::tx::types::TokenFlow]) {
        use futures::stream::{self, StreamExt};

        // MED-004 fix: Limit concurrency to prevent resource exhaustion
        const MAX_CONCURRENT_REQUESTS: usize = 10;

        // Collect unique tokens that need lookup
        let mut tokens_to_lookup: HashSet<Address> = HashSet::new();
        for flow in flows.iter() {
            if flow.token_label.is_none() {
                tokens_to_lookup.insert(flow.token);
            }
        }

        if tokens_to_lookup.is_empty() {
            return;
        }

        // Lookup token metadata with bounded concurrency
        let tokens: Vec<Address> = tokens_to_lookup.into_iter().collect();
        let results: Vec<_> = stream::iter(tokens.iter().map(|&token| async move {
            let result = self
                .abi_fetcher
                .get_token_metadata_rpc(self.chain, &format!("{:#x}", token))
                .await;
            (token, result)
        }))
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect()
        .await;

        // Build symbol map from results
        let mut token_symbols: HashMap<Address, String> = HashMap::new();
        for (token, result) in results {
            if let Ok(metadata) = result {
                if let Some(symbol) = metadata.symbol {
                    token_symbols.insert(token, symbol);
                }
            }
        }

        // Apply to flows
        for flow in flows.iter_mut() {
            if flow.token_label.is_none() {
                if let Some(symbol) = token_symbols.get(&flow.token) {
                    flow.token_label = Some(symbol.clone());
                }
            }
        }
    }

    /// Fetch raw transaction data
    async fn fetch_raw_data(&self, hash: B256) -> Result<RawTxData> {
        // Fetch transaction and receipt in parallel
        let (tx_result, receipt_result) = tokio::join!(
            self.pool.get_transaction(hash),
            self.pool.get_transaction_receipt(hash)
        );

        let tx = tx_result?.ok_or_else(|| format!("Transaction not found: {:#x}", hash))?;

        let receipt = receipt_result?.ok_or_else(|| format!("Receipt not found: {:#x}", hash))?;

        let logs = receipt.inner.logs().to_vec();

        Ok(RawTxData { tx, receipt, logs })
    }

    /// Analyze contracts involved in the transaction
    fn analyze_contracts(&self, raw: &RawTxData) -> Vec<ContractInfo> {
        let mut addresses = HashSet::new();

        // Add from and to - access via Transaction trait from inner
        addresses.insert(raw.tx.inner.signer());
        if let Some(to) = raw.tx.inner.to() {
            addresses.insert(to);
        }

        // Add all addresses from logs
        for log in &raw.logs {
            addresses.insert(log.address());

            // Also extract addresses from indexed topics
            for topic in log.topics().iter().skip(1) {
                // Check if this looks like an address (first 12 bytes are zeros)
                if topic.0[..12] == [0u8; 12] {
                    let addr = Address::from_slice(&topic.0[12..]);
                    if addr != Address::ZERO {
                        addresses.insert(addr);
                    }
                }
            }
        }

        // Build contract info for each address
        let mut contracts: Vec<ContractInfo> = addresses
            .into_iter()
            .map(|addr| get_contract_info(&addr))
            .collect();

        // Sort by category then by label
        contracts.sort_by(|a, b| {
            // LOW-006: Explicit exhaustive match - adding new ContractCategory variants
            // will cause a compile error here, ensuring the sort order is updated.
            let cat_ord = |c: &ContractCategory| match c {
                ContractCategory::Token => 0,
                ContractCategory::Dex => 1,
                ContractCategory::Lending => 2,
                ContractCategory::Staking => 3,
                ContractCategory::Bridge => 4,
                ContractCategory::Nft => 5,
                ContractCategory::Protocol => 6,
                ContractCategory::Unknown => 7,
                ContractCategory::Eoa => 8,
            };

            let ord = cat_ord(&a.category).cmp(&cat_ord(&b.category));
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }

            // Then by label (known first)
            match (&a.label, &b.label) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(a_l), Some(b_l)) => a_l.as_str().cmp(b_l.as_str()),
                (None, None) => a.address.cmp(&b.address),
            }
        });

        contracts
    }

    /// Analyze events in the transaction
    fn analyze_events(&self, raw: &RawTxData) -> Vec<AnalyzedEvent> {
        raw.logs
            .iter()
            .map(|log| {
                let topic0 = log.topics().first().copied().unwrap_or_default();
                let event_name = events::get_event_name(&topic0);
                let is_transfer = topic0 == events::TRANSFER;

                // Decode parameters based on known event signatures
                let params = self.decode_event_params(log, &topic0);

                AnalyzedEvent {
                    log_index: log.log_index.unwrap_or(0),
                    address: log.address(),
                    address_label: get_label(&log.address()).map(String::from),
                    name: event_name.map(String::from),
                    signature: event_name.map(|n| {
                        if n == "Transfer" {
                            "Transfer(address,address,uint256)".to_string()
                        } else if n == "Approval" {
                            "Approval(address,address,uint256)".to_string()
                        } else if n == "Deposit" {
                            "Deposit(address,uint256)".to_string()
                        } else if n == "Withdrawal" {
                            "Withdrawal(address,uint256)".to_string()
                        } else {
                            n.to_string()
                        }
                    }),
                    params,
                    topic0,
                    is_transfer,
                }
            })
            .collect()
    }

    /// Decode event parameters
    fn decode_event_params(
        &self,
        log: &alloy::rpc::types::Log,
        topic0: &B256,
    ) -> HashMap<String, EventParam> {
        // PERF-010 fix: pre-allocate for typical event parameter count (3-4)
        let mut params = HashMap::with_capacity(4);
        let topics = log.topics();

        if *topic0 == events::TRANSFER && topics.len() >= 3 {
            // Transfer(address indexed from, address indexed to, uint256 value)
            let from = Address::from_slice(&topics[1].0[12..]);
            let to = Address::from_slice(&topics[2].0[12..]);

            params.insert(
                "from".to_string(),
                EventParam::Address(format!("{:#x}", from)),
            );
            params.insert("to".to_string(), EventParam::Address(format!("{:#x}", to)));

            let data = &log.data().data;
            if data.len() >= 32 {
                let value = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("value".to_string(), EventParam::Uint(value.to_string()));
            } else {
                // LOW-001 fix: Log malformed event data for debugging
                tracing::debug!(
                    "Malformed Transfer event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::APPROVAL && topics.len() >= 3 {
            // Approval(address indexed owner, address indexed spender, uint256 value)
            let owner = Address::from_slice(&topics[1].0[12..]);
            let spender = Address::from_slice(&topics[2].0[12..]);

            params.insert(
                "owner".to_string(),
                EventParam::Address(format!("{:#x}", owner)),
            );
            params.insert(
                "spender".to_string(),
                EventParam::Address(format!("{:#x}", spender)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let value = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("value".to_string(), EventParam::Uint(value.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Approval event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::DEPOSIT && topics.len() >= 2 {
            // Deposit(address indexed dst, uint256 wad)
            let dst = Address::from_slice(&topics[1].0[12..]);
            params.insert(
                "dst".to_string(),
                EventParam::Address(format!("{:#x}", dst)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let wad = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("wad".to_string(), EventParam::Uint(wad.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Deposit event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::WITHDRAWAL && topics.len() >= 2 {
            // Withdrawal(address indexed src, uint256 wad)
            let src = Address::from_slice(&topics[1].0[12..]);
            params.insert(
                "src".to_string(),
                EventParam::Address(format!("{:#x}", src)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let wad = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("wad".to_string(), EventParam::Uint(wad.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Withdrawal event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        }

        params
    }
}

/// Format transaction analysis for display
pub fn format_analysis(analysis: &TransactionAnalysis) -> String {
    // LOW-005 fix: Pre-allocate for typical output size
    let mut output = String::with_capacity(2048);

    // Header
    output.push_str(&format!("Transaction: {:#x}\n", analysis.hash));
    output.push_str(&format!("Block: {}\n", analysis.block_number));
    output.push_str(&format!("From: {:#x}\n", analysis.from));

    if let Some(to) = analysis.to {
        let label = get_label(&to)
            .map(|l| format!(" ({})", l))
            .unwrap_or_default();
        output.push_str(&format!("To: {:#x}{}\n", to, label));
    } else {
        output.push_str("To: Contract Creation\n");
    }

    // HIGH-001 fix: Use safe U256 to f64 conversion
    output.push_str(&format!(
        "Value: {} wei ({:.6} ETH)\n",
        analysis.value,
        u256_to_eth_f64(&analysis.value)
    ));
    output.push_str(&format!("Gas Used: {}\n", analysis.gas_used));
    output.push_str(&format!(
        "Status: {}\n",
        if analysis.status { "Success" } else { "Failed" }
    ));

    // Contracts
    output.push_str("\nContracts Involved:\n");
    for contract in &analysis.contracts {
        let label = contract
            .label
            .as_ref()
            .map(|l| format!(" {}", l))
            .unwrap_or_else(|| format!(" ({:?})", contract.category));
        output.push_str(&format!("  {:#x}{}\n", contract.address, label));
    }

    // Events summary
    output.push_str(&format!("\nEvents ({}):\n", analysis.events.len()));
    for (i, event) in analysis.events.iter().enumerate().take(20) {
        let name = event.name.as_deref().unwrap_or("Unknown");
        let contract = event
            .address_label
            .as_ref()
            .map(|l| format!("({})", l))
            .unwrap_or_default();
        output.push_str(&format!("  [{}] {} {}\n", i, name, contract));
    }
    if analysis.events.len() > 20 {
        output.push_str(&format!("  ... and {} more\n", analysis.events.len() - 20));
    }

    // Token flows
    if !analysis.token_flows.is_empty() {
        output.push_str(&format!(
            "\nToken Transfers ({}):\n",
            analysis.token_flows.len()
        ));
        for flow in analysis.token_flows.iter().take(15) {
            let token = flow.token_label.as_deref().unwrap_or("???");
            let from_short = format!("{:#x}", flow.from);
            let from_short = if from_short.len() > 12 {
                &from_short[..12]
            } else {
                &from_short
            };
            let to_short = format!("{:#x}", flow.to);
            let to_short = if to_short.len() > 12 {
                &to_short[..12]
            } else {
                &to_short
            };

            // LOW-002 fix: Use safe token amount formatting
            let amount_display = format_token_amount(&flow.amount, 18);

            output.push_str(&format!(
                "  {} {}... → {}... {} {}\n",
                token, from_short, to_short, amount_display, token
            ));
        }
        if analysis.token_flows.len() > 15 {
            output.push_str(&format!(
                "  ... and {} more\n",
                analysis.token_flows.len() - 15
            ));
        }
    }

    // Function call
    if let Some(func) = &analysis.function_call {
        output.push_str("\nFunction Call:\n");
        let name = func.name.as_deref().unwrap_or("Unknown");
        output.push_str(&format!("  {} ({})\n", name, func.selector));
        if let Some(sig) = &func.signature {
            output.push_str(&format!("  Signature: {}\n", sig));
        }
        if !func.params.is_empty() {
            output.push_str("  Parameters:\n");
            for param in func.params.iter().take(10) {
                output.push_str(&format!(
                    "    {} ({}): {}\n",
                    param.name, param.ty, param.value
                ));
            }
            if func.params.len() > 10 {
                output.push_str(&format!("    ... and {} more\n", func.params.len() - 10));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::types::{AnalyzedEvent, FunctionCall, FunctionParam};
    use alloy::primitives::{address, b256, B256, U256};
    use std::collections::HashMap;

    fn make_test_analysis() -> TransactionAnalysis {
        TransactionAnalysis {
            hash: b256!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
            block_number: 18_500_000,
            from: address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
            to: Some(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")),
            value: U256::from(1_000_000_000_000_000_000u128), // 1 ETH
            gas_used: 21000,
            status: true,
            contracts: vec![ContractInfo {
                address: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                label: Some("WETH".to_string()),
                category: ContractCategory::Token,
            }],
            events: vec![],
            token_flows: vec![],
            function_call: None,
        }
    }

    #[test]
    fn test_format_analysis_basic() {
        let analysis = make_test_analysis();
        let output = format_analysis(&analysis);

        assert!(output.contains("Transaction: 0x1234"));
        assert!(output.contains("Block: 18500000"));
        assert!(output.contains("Status: Success"));
        assert!(output.contains("WETH"));
    }

    #[test]
    fn test_format_analysis_failed_tx() {
        let mut analysis = make_test_analysis();
        analysis.status = false;
        let output = format_analysis(&analysis);

        assert!(output.contains("Status: Failed"));
    }

    #[test]
    fn test_format_analysis_contract_creation() {
        let mut analysis = make_test_analysis();
        analysis.to = None;
        let output = format_analysis(&analysis);

        assert!(output.contains("Contract Creation"));
    }

    // LOW-007: Edge case tests

    #[test]
    fn test_u256_to_eth_f64_normal() {
        // 1 ETH = 1e18 wei
        let one_eth = U256::from(1_000_000_000_000_000_000u128);
        let result = u256_to_eth_f64(&one_eth);
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_u256_to_eth_f64_small() {
        // 1 wei
        let one_wei = U256::from(1u64);
        let result = u256_to_eth_f64(&one_wei);
        assert!(result > 0.0 && result < 1e-17);
    }

    #[test]
    fn test_u256_to_eth_f64_zero() {
        let zero = U256::ZERO;
        let result = u256_to_eth_f64(&zero);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_u256_to_eth_f64_large() {
        // Value larger than u128::MAX
        let large = U256::MAX;
        let result = u256_to_eth_f64(&large);
        // Should not panic, and should return a large positive number
        assert!(result > 0.0);
        assert!(result.is_finite() || result == f64::MAX);
    }

    #[test]
    fn test_format_token_amount_normal() {
        // 1.23 tokens (with 18 decimals)
        let amount = "1230000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "1.23");
    }

    #[test]
    fn test_format_token_amount_small() {
        // 0.01 tokens
        let amount = "10000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "0.01");
    }

    #[test]
    fn test_format_token_amount_very_small() {
        // Very small amount (less than 0.01)
        let amount = "1000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "0.00");
    }

    #[test]
    fn test_format_token_amount_empty() {
        let result = format_token_amount("", 18);
        assert_eq!(result, "0.00");
    }

    #[test]
    fn test_format_token_amount_invalid() {
        let result = format_token_amount("not_a_number", 18);
        assert_eq!(result, "0.00");
    }

    #[test]
    fn test_format_token_amount_large() {
        // 1 million tokens
        let amount = "1000000000000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "1000000.00");
    }

    #[test]
    fn test_format_analysis_many_events() {
        let mut analysis = make_test_analysis();
        // Add 25 events (more than the 20 limit)
        for i in 0..25 {
            analysis.events.push(AnalyzedEvent {
                log_index: i as u64,
                address: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                address_label: Some("WETH".to_string()),
                name: Some("Transfer".to_string()),
                signature: Some("Transfer(address,address,uint256)".to_string()),
                params: HashMap::new(),
                topic0: B256::ZERO,
                is_transfer: true,
            });
        }

        let output = format_analysis(&analysis);
        assert!(output.contains("... and 5 more"));
    }

    #[test]
    fn test_format_analysis_many_token_flows() {
        use crate::tx::types::TokenFlow;

        let mut analysis = make_test_analysis();
        // Add 20 token flows (more than the 15 limit)
        for i in 0..20 {
            analysis.token_flows.push(TokenFlow {
                token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
                token_label: Some("USDC".to_string()),
                from: address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
                from_label: None,
                to: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                to_label: None,
                amount: "1000000".to_string(),
                log_index: i as u64,
            });
        }

        let output = format_analysis(&analysis);
        assert!(output.contains("... and 5 more"));
    }

    #[test]
    fn test_format_analysis_with_function_call() {
        let mut analysis = make_test_analysis();
        analysis.function_call = Some(FunctionCall {
            selector: "0xa9059cbb".to_string(),
            name: Some("transfer".to_string()),
            signature: Some("transfer(address,uint256)".to_string()),
            params: vec![
                FunctionParam {
                    name: "to".to_string(),
                    ty: "address".to_string(),
                    value: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
                },
                FunctionParam {
                    name: "amount".to_string(),
                    ty: "uint256".to_string(),
                    value: "1000000000000000000".to_string(),
                },
            ],
        });

        let output = format_analysis(&analysis);
        assert!(output.contains("Function Call:"));
        assert!(output.contains("transfer"));
        assert!(output.contains("0xa9059cbb"));
        assert!(output.contains("Parameters:"));
    }

    #[test]
    fn test_format_analysis_empty_contracts() {
        let mut analysis = make_test_analysis();
        analysis.contracts.clear();
        let output = format_analysis(&analysis);

        // Should still have the "Contracts Involved:" header
        assert!(output.contains("Contracts Involved:"));
    }

    #[test]
    fn test_format_analysis_large_value() {
        let mut analysis = make_test_analysis();
        // Set a very large value (> u128::MAX would overflow, but let's test a large u128)
        analysis.value = U256::from(u128::MAX);
        let output = format_analysis(&analysis);

        // Should not panic and should contain ETH formatting
        assert!(output.contains("ETH"));
    }
}
