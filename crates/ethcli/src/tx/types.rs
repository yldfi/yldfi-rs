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
    /// Position of the transaction in its block
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_index: Option<u64>,
    /// Price paid per unit of gas, in wei
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_gas_price: Option<String>,
    /// Fee paid by the sender in wei: gas used times the effective gas price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_wei: Option<String>,
    /// The fee in the chain's native token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee: Option<String>,
    /// Net change per address and token over the whole transaction, from
    /// its token transfers, the top-level value and the fee
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub balance_changes: Vec<BalanceChange>,
    /// Block timestamp in seconds since the Unix epoch (with `--full`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    /// Block time as RFC 3339 in UTC (with `--full`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Why a failed transaction reverted (with `--full`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revert: Option<crate::abi::revert::DecodedRevert>,
    /// Where `revert` came from: "trace" is the revert data of the
    /// transaction as it ran; "replay" is from re-running the call against
    /// the parent block's state, which can fail for a different reason
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revert_from: Option<String>,
    /// Things the reader should know about how complete the analysis is
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// Calldata, kept to replay a failed transaction
    #[serde(skip)]
    pub(crate) input: alloy::primitives::Bytes,
    /// Gas limit, kept to replay a failed transaction
    #[serde(skip)]
    pub(crate) gas_limit: u64,
    /// Deposits into and withdrawals from the chain's wrapped native token
    #[serde(skip)]
    pub(crate) wraps: Vec<WrapEvent>,
    /// Every native-token transfer in the transaction, from a call trace
    /// (`None` until a trace has been fetched)
    #[serde(skip)]
    pub(crate) native_transfers: Option<Vec<(Address, Address, U256)>>,
    /// Every native-token transfer made in the transaction, top-level call
    /// included, from a call trace (with `--full`, when tracing works)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_transfer_list: Vec<NativeTransfer>,
}

/// A native-token transfer found in a call trace
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTransfer {
    /// Sender
    pub from: Address,
    /// Recipient
    pub to: Address,
    /// Amount in wei
    pub value_wei: String,
    /// Amount in the native token
    pub value: String,
}

/// A deposit into (wrap) or withdrawal from (unwrap) a wrapped native token.
/// These mint and burn the token without a Transfer event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapEvent {
    /// The wrapped native token
    pub token: Address,
    /// Account that wrapped or unwrapped
    pub account: Address,
    /// Amount, the same in native and wrapped units
    pub amount: U256,
    /// true for a deposit (native in, wrapped out), false for a withdrawal
    pub deposit: bool,
}

/// Net change in one address's balance of one token
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalanceChange {
    /// Address whose balance changed
    pub address: Address,
    /// Known label for the address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Token contract, or `None` for the chain's native token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<Address>,
    /// Token symbol or label, when known
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Token decimals, when known (18 for the native token)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    /// Signed change in the token's smallest unit, e.g. "-1500000"
    pub raw: String,
    /// Signed change scaled by the decimals, when they are known
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    /// true when the figure may be incomplete: a native-token change worked
    /// out without a call trace, so transfers made inside the call are only
    /// known for wrapped-native deposits and withdrawals
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub partial: bool,
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
    /// Token decimals (resolved via `decimals()`); `None` if unknown
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    /// ERC-721 token id (then `amount` is 1 and `decimals` 0)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
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
        let fee = U256::from(receipt.gas_used) * U256::from(receipt.effective_gas_price);

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
            transaction_index: receipt.transaction_index,
            effective_gas_price: Some(receipt.effective_gas_price.to_string()),
            fee_wei: Some(fee.to_string()),
            fee: alloy::primitives::utils::format_units(fee, 18).ok(),
            balance_changes: Vec::new(),
            timestamp: None,
            time: None,
            revert: None,
            revert_from: None,
            notes: Vec::new(),
            input: tx.inner.input().clone(),
            gas_limit: tx.inner.gas_limit(),
            wraps: Vec::new(),
            native_transfers: None,
            native_transfer_list: Vec::new(),
        }
    }
}
