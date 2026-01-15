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
use alloy::primitives::{Address, B256};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

/// Transaction analyzer
pub struct TxAnalyzer {
    /// RPC pool
    pool: RpcPool,
    /// Chain
    chain: Chain,
    /// ABI fetcher for Etherscan lookups
    abi_fetcher: AbiFetcher,
}

impl TxAnalyzer {
    /// Create a new analyzer
    pub fn new(pool: RpcPool, chain: Chain) -> Result<Self> {
        Ok(Self {
            pool,
            chain,
            abi_fetcher: AbiFetcher::new(None)?,
        })
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
        for contract in contracts.iter_mut() {
            // Skip if already labeled
            if contract.label.is_some() {
                continue;
            }

            // Skip zero address
            if contract.address == Address::ZERO {
                continue;
            }

            // Try to get metadata from Etherscan
            if let Ok(metadata) = self
                .abi_fetcher
                .get_contract_metadata(self.chain, &format!("{:#x}", contract.address))
                .await
            {
                if let Some(name) = metadata.name {
                    contract.label = Some(name);
                    // If verified, it's likely a contract (not EOA)
                    if metadata.is_verified && contract.category == ContractCategory::Unknown {
                        contract.category = ContractCategory::Protocol;
                    }
                }
            }
        }
    }

    /// Enrich token flows with token metadata
    async fn enrich_token_flows(&self, flows: &mut [crate::tx::types::TokenFlow]) {
        // Collect unique tokens that need lookup
        let mut tokens_to_lookup: HashSet<Address> = HashSet::new();
        for flow in flows.iter() {
            if flow.token_label.is_none() {
                tokens_to_lookup.insert(flow.token);
            }
        }

        // Lookup token metadata for each
        let mut token_symbols: HashMap<Address, String> = HashMap::new();
        for token in tokens_to_lookup {
            if let Ok(metadata) = self
                .abi_fetcher
                .get_token_metadata_rpc(self.chain, &format!("{:#x}", token))
                .await
            {
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
        let mut params = HashMap::new();
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
            }
        }

        params
    }
}

/// Format transaction analysis for display
pub fn format_analysis(analysis: &TransactionAnalysis) -> String {
    let mut output = String::new();

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

    output.push_str(&format!(
        "Value: {} wei ({:.6} ETH)\n",
        analysis.value,
        analysis.value.to_string().parse::<f64>().unwrap_or(0.0) / 1e18
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

            // Format amount nicely
            let amount = &flow.amount;
            let amount_display = if amount.len() > 18 {
                format!(
                    "{}.{}",
                    &amount[..amount.len() - 18],
                    &amount[amount.len() - 18..amount.len() - 16]
                )
            } else {
                format!("0.{}", amount)
            };

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
