//! Transaction analysis types

use alloy::consensus::Transaction as TxTrait;
use alloy::primitives::{Address, B256, U256};
use alloy::rpc::types::{Log, Transaction, TransactionReceipt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete transaction analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// Transaction hash
    pub hash: B256,
    /// Block number
    pub block_number: u64,
    /// From address
    pub from: Address,
    /// To address (None for contract creation)
    pub to: Option<Address>,
    /// Value transferred in wei
    pub value: U256,
    /// Gas used
    pub gas_used: u64,
    /// Transaction status (true = success)
    pub status: bool,
    /// Contracts involved with labels
    pub contracts: Vec<ContractInfo>,
    /// Analyzed events
    pub events: Vec<AnalyzedEvent>,
    /// Token flows
    pub token_flows: Vec<TokenFlow>,
    /// Decoded function call (if available)
    pub function_call: Option<FunctionCall>,
}

/// Information about a contract involved in the transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInfo {
    /// Contract address
    pub address: Address,
    /// Human-readable label (e.g., "WETH", "Uniswap V3 Router")
    pub label: Option<String>,
    /// Contract type/category
    pub category: ContractCategory,
}

/// Category of contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContractCategory {
    /// ERC20 token
    Token,
    /// DEX/AMM
    Dex,
    /// Lending protocol
    Lending,
    /// Staking/Yield
    Staking,
    /// Bridge
    Bridge,
    /// NFT (ERC721/1155)
    Nft,
    /// Known protocol (other)
    Protocol,
    /// Unknown contract
    Unknown,
    /// Externally owned account (not a contract)
    Eoa,
}

/// Analyzed event with decoded parameters and labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedEvent {
    /// Log index
    pub log_index: u64,
    /// Contract address that emitted the event
    pub address: Address,
    /// Contract label (if known)
    pub address_label: Option<String>,
    /// Event name (if decoded)
    pub name: Option<String>,
    /// Event signature (canonical form)
    pub signature: Option<String>,
    /// Decoded parameters
    pub params: HashMap<String, EventParam>,
    /// Raw topic0
    pub topic0: B256,
    /// Whether this is a Transfer event
    pub is_transfer: bool,
}

/// Event parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventParam {
    Address(String),
    Uint(String),
    Int(String),
    Bool(bool),
    Bytes(String),
    String(String),
    Array(Vec<EventParam>),
}

/// Token transfer flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenFlow {
    /// Token contract address
    pub token: Address,
    /// Token symbol/label
    pub token_label: Option<String>,
    /// From address
    pub from: Address,
    /// From label
    pub from_label: Option<String>,
    /// To address
    pub to: Address,
    /// To label
    pub to_label: Option<String>,
    /// Amount transferred (as string to preserve precision)
    pub amount: String,
    /// Log index where this transfer occurred
    pub log_index: u64,
}

/// Net token flow summary for an address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetTokenFlow {
    /// Token address
    pub token: Address,
    /// Token label
    pub token_label: Option<String>,
    /// Net change (positive = received, negative = sent)
    pub net_change: String,
    /// Whether this address received tokens
    pub is_inflow: bool,
}

/// Decoded function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// Function selector (4 bytes)
    pub selector: String,
    /// Function name (if decoded)
    pub name: Option<String>,
    /// Function signature (if decoded)
    pub signature: Option<String>,
    /// Decoded parameters
    pub params: Vec<FunctionParam>,
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub ty: String,
    /// Parameter value
    pub value: String,
}

/// Raw transaction data before analysis
#[derive(Debug, Clone)]
pub struct RawTxData {
    /// The transaction
    pub tx: Transaction,
    /// The transaction receipt
    pub receipt: TransactionReceipt,
    /// Logs from receipt
    pub logs: Vec<Log>,
}

impl TransactionAnalysis {
    /// Create analysis from raw transaction data
    pub fn new(data: &RawTxData) -> Self {
        let tx = &data.tx;
        let receipt = &data.receipt;

        Self {
            hash: *tx.inner.tx_hash(),
            block_number: receipt.block_number.unwrap_or(0),
            from: tx.inner.signer(),
            to: tx.inner.to(),
            value: tx.inner.value(),
            gas_used: receipt.gas_used,
            status: receipt.status(),
            contracts: Vec::new(),
            events: Vec::new(),
            token_flows: Vec::new(),
            function_call: None,
        }
    }
}
