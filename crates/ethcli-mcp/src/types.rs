//! Input types for MCP tools
//!
//! These structs define the JSON Schema for tool parameters.
//! All types derive JsonSchema for MCP protocol compatibility.

use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// INPUT TYPES (with JSON Schema for MCP)
// =============================================================================

fn default_chain() -> String {
    "ethereum".to_string()
}

fn default_moralis_chain() -> String {
    "eth".to_string()
}

fn default_network() -> String {
    "eth-mainnet".to_string()
}

fn default_solana_network() -> String {
    "solana-mainnet".to_string()
}

fn default_unit() -> String {
    "ether".to_string()
}

// --- Logs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// Unknown fields are errors so callers still sending removed fields find out
#[serde(deny_unknown_fields)]
pub struct LogsInput {
    /// Contract address to query logs from
    pub contract: String,
    /// Event signature (e.g., "Transfer(address,address,uint256)")
    pub event: Option<String>,
    /// Starting block number or tag (e.g., "latest", "0x1234")
    pub from_block: Option<String>,
    /// Ending block number or tag
    pub to_block: Option<String>,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Transaction ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TxAnalyzeInput {
    /// Transaction hash (with or without 0x prefix).
    /// Partial hashes are supported when block is specified.
    pub hash: String,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number for partial hash resolution.
    /// When provided with a partial tx hash (e.g. from account_txs output),
    /// the transaction will be looked up by matching the prefix within this block.
    #[serde(default)]
    pub block: Option<u64>,
    /// Also fetch a call trace, the block's time and, for a failed transaction,
    /// the reason it reverted (up to three more requests). The fee, the
    /// position in the block and balance_changes are always returned, but
    /// native-token entries carry "partial": true without the trace and may be
    /// wrong for contracts or accounts paid inside the call. Set this when the
    /// native-token amounts matter. For a failed transaction, check
    /// revert_from: "trace" is the real reason; "replay" is from re-running the
    /// call on the parent block and can differ from it.
    #[serde(default)]
    pub full: bool,
}

// --- Account ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountAddressInput {
    /// Ethereum address (hex format)
    pub address: String,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountBalanceInput {
    /// Ethereum address(es) - supports multiple addresses
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Ethereum address (legacy singular form)
    pub address: Option<String>,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountTxsInput {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
    /// Sort order: "asc" or "desc" (default: desc)
    pub sort: Option<String>,
    /// First block to include (inclusive; default: 0)
    pub start_block: Option<u64>,
    /// Last block to include (inclusive; default: latest)
    pub end_block: Option<u64>,
    /// Fetch every page (1,000 rows per request), walking past the
    /// explorer's 10,000-row window. Ignores page and limit
    #[serde(default)]
    pub all: bool,
    /// With all, stop after this many requests (default 10). Tool output over
    /// 1 MB is cut off, so 1-2 pages is the useful range here
    #[serde(default)]
    pub max_pages: Option<u32>,
}

#[cfg(test)]
mod account_history_range_tests {
    use super::AccountTxsInput;

    #[test]
    fn old_clients_can_omit_bounds_and_schema_exposes_new_bounds() {
        let input: AccountTxsInput = serde_json::from_value(
            serde_json::json!({"address":"0x0000000000000000000000000000000000000000"}),
        )
        .unwrap();
        assert_eq!(input.start_block, None);
        assert_eq!(input.end_block, None);
        let schema = serde_json::to_value(schemars::schema_for!(AccountTxsInput)).unwrap();
        assert!(schema["properties"].get("start_block").is_some());
        assert!(schema["properties"].get("end_block").is_some());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc20Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
    /// Fetch every page (1,000 rows per request), walking past the
    /// explorer's 10,000-row window. Ignores page and limit
    #[serde(default)]
    pub all: bool,
    /// With all, stop after this many requests (default 10). Tool output over
    /// 1 MB is cut off, so 1-2 pages is the useful range here
    #[serde(default)]
    pub max_pages: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountLabelsInput {
    /// Addresses to look up (up to 100)
    pub addresses: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc721Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc1155Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInternalTxsInput {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
    /// Fetch every page (1,000 rows per request), walking past the
    /// explorer's 10,000-row window. Ignores page and limit
    #[serde(default)]
    pub all: bool,
    /// With all, stop after this many requests (default 10). Tool output over
    /// 1 MB is cut off, so 1-2 pages is the useful range here
    #[serde(default)]
    pub max_pages: Option<u32>,
}

// --- Contract ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAddressInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAbiInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Disabled over MCP; omit to return the ABI in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSourceInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Disabled over MCP; omit to return source data in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSelectorsInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Lookup function signatures from 4byte.directory
    #[serde(default)]
    pub lookup: bool,
    /// If proxy detected, extract selectors from the implementation contract instead
    #[serde(default)]
    pub follow_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractDisassembleInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Maximum number of opcodes to show
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractOpcodesInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAnalyzeInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Include full opcode disassembly in output
    #[serde(default)]
    pub include_disassembly: bool,
    /// Limit number of opcodes in disassembly (requires include_disassembly)
    pub limit: Option<u32>,
    /// Lookup function signatures from 4byte.directory
    #[serde(default)]
    pub lookup: bool,
    /// If proxy detected, analyze the implementation contract instead
    #[serde(default)]
    pub follow_proxy: bool,
    /// Include selector -> handler offset dispatcher mapping
    #[serde(default)]
    pub dispatcher: bool,
    /// Include handler guard/check heuristics for auth, calldata hash validation, storage checks, and external calls
    #[serde(default)]
    pub checks: bool,
}

// --- Token ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenInfoInput {
    /// Token contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenHoldersInput {
    /// Token contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Maximum number of holders to return
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceInput {
    /// Token contract address(es) or labels. Use "eth" for native ETH balance.
    #[serde(default)]
    pub tokens: Vec<String>,
    /// Token contract address or label (legacy singular form)
    pub token: Option<String>,
    /// Holder address(es) to check balance for
    #[serde(default)]
    pub holders: Vec<String>,
    /// Holder address (legacy singular form)
    pub address: Option<String>,
    /// Get balances for all addresses with this tag (can combine with holders)
    pub tag: Option<String>,
    /// Show zero balances (hidden by default when multiple holders)
    #[serde(default)]
    pub show_zero: bool,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Gas ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GasOracleInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GasEstimateInput {
    /// Gas price in gwei to estimate confirmation time for
    pub gwei: u64,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Signature Lookup ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SigLookupInput {
    /// 4-byte function selector (e.g., "0xa9059cbb") or event topic
    pub selector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SigErrorInput {
    /// Revert data as 0x hex (selector plus encoded arguments), or a bare 4-byte error selector
    pub data: String,
}

// --- Cast (Conversions) ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastToWeiInput {
    /// Amount to convert
    pub amount: String,
    /// Unit (ether, gwei, or wei)
    #[serde(default = "default_unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastFromWeiInput {
    /// Wei amount to convert
    pub wei: String,
    /// Target unit (ether or gwei)
    #[serde(default = "default_unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastValueInput {
    /// Value to convert
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastSigInput {
    /// Function signature (e.g., "transfer(address,uint256)")
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastIndexInput {
    /// Solidity type of the mapping key (e.g., "address", "uint256", "bytes32")
    pub key_type: String,
    /// Key value
    pub key: String,
    /// Slot of the mapping (decimal or 0x hex). For a nested mapping, pass the previous result
    pub slot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastAbiEncodeInput {
    /// Function signature
    pub sig: String,
    /// Arguments to encode
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastAbiDecodeInput {
    /// Type signature for raw ABI data, or function signature for calldata with selector
    /// (e.g. "uint256", "(address,uint256)", or "transfer(address,uint256)")
    pub sig: String,
    /// Encoded data to decode
    pub data: String,
}

// --- RPC ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcCallInput {
    /// Contract address
    pub to: String,
    /// Calldata (hex encoded)
    pub data: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number or tag
    pub block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcBlockInput {
    /// Block number, hash, or tag (latest, pending, etc.)
    pub block: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Show full transactions (not just hashes)
    #[serde(default)]
    pub full: bool,
    /// Custom RPC URL (overrides default)
    pub rpc_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcStorageInput {
    /// Contract address
    pub address: String,
    /// Storage slot
    pub slot: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcAddressInput {
    /// Address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcHashInput {
    /// Transaction hash
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcMulticallCall {
    /// Contract address
    pub address: String,
    /// Full signature with return types, e.g. "balanceOf(address)(uint256)".
    /// Results are decoded only when return types are given
    pub sig: String,
    /// Arguments, one string each (no spaces inside an argument)
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcMulticallInput {
    /// Calls to run. All are read at the same block
    pub calls: Vec<RpcMulticallCall>,
    /// Block number, tag or a time ("2024-01-31", "@1706700000", "7d"), or a
    /// comma-separated list of block numbers to repeat the calls at (default: latest)
    #[serde(default)]
    pub block: Option<String>,
    /// Repeat the calls every STEP blocks: "FROM:TO:STEP" (inclusive). Use instead of `block`
    #[serde(default)]
    pub block_range: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcBlockAtInput {
    /// A date ("2024-01-31", "2024-01-31T12:00:00Z"), a Unix timestamp
    /// written as "@1706700000", or an age ("7d", "12h", "30m"). A plain
    /// date is 00:00 UTC at the start of that day: for the end of a day pass
    /// the next day's date or a full time ("2024-01-31T23:59:59Z")
    pub time: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcBalanceInput {
    /// Account address
    pub address: String,
    /// Block number, "latest", or a time ("2024-01-31", "@1706700000", "7d") (default: latest).
    /// A plain date is 00:00 UTC at the start of that day, i.e. the state before anything on
    /// that date; for an end-of-day balance pass the next day's date or "2024-01-31T23:59:59Z"
    #[serde(default)]
    pub block: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- ENS ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsResolveInput {
    /// ENS name (e.g., "vitalik.eth")
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsLookupInput {
    /// Ethereum address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsNamehashInput {
    /// ENS name to hash
    pub name: String,
}

// --- Price ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriceInput {
    /// Token symbol or address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Portfolio ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioInput {
    /// Wallet address(es) to query. Can specify multiple.
    /// If empty/omitted, uses "me" from address book.
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Query all addresses with this tag from address book
    pub tag: Option<String>,
    /// Aggregate all wallets into a single combined view
    #[serde(default)]
    pub aggregate: bool,
    /// Chain(s) to query (default: ethereum)
    pub chain: Option<String>,
    /// Source to query: all, alchemy, moralis, uniswap, yearn
    pub source: Option<String>,
    /// Minimum USD value to show (filter small balances)
    pub min_value: Option<f64>,
    /// Show per-source breakdown
    #[serde(default)]
    pub show_sources: bool,
    /// Include blacklisted tokens (normally filtered out)
    #[serde(default)]
    pub show_blacklisted: bool,
}

// --- NFTs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NftsInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Yields ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct YieldsInput {
    /// Protocol name (optional)
    pub protocol: Option<String>,
    /// Chain name (optional)
    pub chain: Option<String>,
}

// --- Quote ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteBestInput {
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw).
    /// E.g., decimals=18 converts "1.5" to "1500000000000000000"
    pub decimals: Option<u8>,
    /// Wallet that will sign the swap. Transaction data is built for it and pays the output
    /// to it, so only pass the user's own wallet, never an example address. Omit it for prices:
    /// CowSwap, LI.FI and Enso are then priced for a placeholder (marked `placeholder_sender`)
    /// and return no transaction data.
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in the CLI's table output. It does not change the JSON this tool
    /// returns: `quote_from` JSON carries transaction data whenever the source built it for a
    /// real `sender`, and `quote_best`/`quote_compare` JSON never does.
    #[serde(default)]
    pub show_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteCompareInput {
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw)
    pub decimals: Option<u8>,
    /// Wallet that will sign the swap. Transaction data is built for it and pays the output
    /// to it, so only pass the user's own wallet, never an example address. Omit it for prices:
    /// CowSwap, LI.FI and Enso are then priced for a placeholder (marked `placeholder_sender`)
    /// and return no transaction data.
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in the CLI's table output. It does not change the JSON this tool
    /// returns: `quote_from` JSON carries transaction data whenever the source built it for a
    /// real `sender`, and `quote_best`/`quote_compare` JSON never does.
    #[serde(default)]
    pub show_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteFromInput {
    /// The aggregator to use (openocean, kyberswap, 0x, 1inch, cowswap, lifi, velora,
    /// velora-delta, enso, propamm). velora-delta is a gasless order: its output is net of gas.
    /// propamm is a market-maker pool price from a block builder's snapshot: conditional, it
    /// holds only in blocks from builders carrying the maker's update.
    pub source: String,
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw)
    pub decimals: Option<u8>,
    /// Wallet that will sign the swap. Transaction data is built for it and pays the output
    /// to it, so only pass the user's own wallet, never an example address. Omit it for prices:
    /// CowSwap, LI.FI and Enso are then priced for a placeholder (marked `placeholder_sender`)
    /// and return no transaction data.
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in the CLI's table output. It does not change the JSON this tool
    /// returns: `quote_from` JSON carries transaction data whenever the source built it for a
    /// real `sender`, and `quote_best`/`quote_compare` JSON never does.
    #[serde(default)]
    pub show_tx: bool,
}

// --- Chainlink ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkPriceInput {
    /// Token symbol (e.g., "ETH", "BTC")
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number for historical price
    pub block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkFeedInput {
    /// Token symbol
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkOraclesInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsFeedInput {
    /// Feed ID (hex string, e.g., "0x00036...")
    pub feed_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsReportInput {
    /// Feed ID (hex string)
    pub feed_id: String,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsBulkInput {
    /// Comma-separated feed IDs (hex strings)
    pub feed_ids: String,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsHistoryInput {
    /// Feed ID (hex string)
    pub feed_id: String,
    /// Start timestamp in seconds
    pub timestamp: u64,
    /// Number of reports to fetch
    #[serde(default = "default_streams_limit")]
    pub limit: u32,
}

fn default_streams_limit() -> u32 {
    10
}

// --- GoPlus Security ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoplusInput {
    /// Address to check
    pub address: String,
    /// Chain ID (1 for Ethereum, 137 for Polygon, etc.)
    pub chain_id: u64,
}

// --- Solodit ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SoloditSearchInput {
    /// Keywords to search in title and content (empty string = no keyword filter)
    pub query: String,
    /// Filter by impact: HIGH, MEDIUM, LOW, GAS (comma-separated)
    pub impact: Option<String>,
    /// Results per page (1-100, default 20)
    pub limit: Option<u32>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Filter by audit firm(s), comma-separated (e.g. "Cyfrin,Sherlock")
    pub firm: Option<String>,
    /// Filter by tag(s), comma-separated (e.g. "Reentrancy,Oracle")
    pub tag: Option<String>,
    /// Filter by protocol name (partial match)
    pub protocol: Option<String>,
    /// Filter by protocol category(ies), comma-separated (e.g. "DeFi,Lending")
    pub protocol_category: Option<String>,
    /// Filter by forked protocol(s), comma-separated
    pub forked: Option<String>,
    /// Filter by language(s), comma-separated (Solidity, Rust, Cairo, Vyper, Move)
    pub language: Option<String>,
    /// Filter by finder/auditor handle (partial match)
    pub finder: Option<String>,
    /// Report date period: 30, 60, 90, alltime, or after (with reported_after)
    pub reported: Option<String>,
    /// Only findings reported after this ISO date (e.g. 2024-01-01)
    pub reported_after: Option<String>,
    /// Sort field: recency, quality, rarity
    pub sort: Option<String>,
    /// Sort direction: desc or asc
    pub sort_dir: Option<String>,
    /// Minimum quality score (0-5)
    pub min_quality: Option<u32>,
    /// Minimum rarity score (0-5)
    pub min_rarity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SoloditGetInput {
    /// Finding slug, ID, or Solodit URL (best-effort lookup via search)
    pub slug: String,
}

// --- Uniswap ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPoolInput {
    /// Pool address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapEthPriceInput {
    /// Uniswap version (v2, v3, v4)
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapTopPoolsInput {
    /// Number of pools to return
    pub limit: Option<u32>,
    /// Uniswap version
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPositionsInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Uniswap version
    pub version: Option<String>,
}

// --- Curve ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurvePoolsInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainVolumeInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Start timestamp (unix seconds, default: end - 30 days)
    #[serde(default)]
    pub start: Option<u64>,
    /// End timestamp (unix seconds, default: now)
    #[serde(default)]
    pub end: Option<u64>,
    /// Aggregation interval: "hour", "day" (default) or "week". Window max 300x interval.
    #[serde(default)]
    pub interval: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveRouteInput {
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Alchemy ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- CoinGecko ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinInput {
    /// CoinGecko coin ID
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoPriceInput {
    /// Comma-separated coin IDs
    pub ids: String,
    /// Target currencies (default: usd)
    pub vs_currencies: Option<String>,
}

// --- DefiLlama ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaTvlInput {
    /// Protocol name
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaYieldsInput {}

// --- DEX Aggregators ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchQuoteInput {
    /// Source token address
    pub src: String,
    /// Destination token address
    pub dst: String,
    /// Amount in smallest unit
    pub amount: String,
    /// Chain ID
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxQuoteInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Taker address
    pub taker: String,
    /// Use the AllowanceHolder flow (approve `allowanceTarget`, no Permit2 signature) instead of Permit2
    #[serde(default)]
    pub allowance_holder: bool,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanQuoteInput {
    /// Input token address
    pub in_token: String,
    /// Output token address
    pub out_token: String,
    /// Amount in smallest units (wei) for quotes; for reverse quotes, the
    /// human-readable desired output amount (e.g. "1")
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Gas price in Gwei (default: 30)
    #[serde(default = "default_gas_price")]
    pub gas_price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanReverseQuoteInput {
    /// Token to sell
    pub in_token: String,
    /// Token to receive
    pub out_token: String,
    /// Desired output amount, human-readable (e.g. "1")
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

fn default_gas_price() -> String {
    "30".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapQuoteInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Amount
    pub amount: String,
    /// From address
    pub from: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiQuoteInput {
    /// Source chain ID (e.g. 1 for Ethereum, 42161 for Arbitrum)
    pub from_chain: u64,
    /// Source token address
    pub from_token: String,
    /// Destination chain ID (e.g. 10 for Optimism, 8453 for Base)
    pub to_chain: u64,
    /// Destination token address
    pub to_token: String,
    /// Amount
    pub amount: String,
    /// Sender address
    pub from_address: String,
}

// --- Pyth ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythPriceInput {
    /// Symbol or price feed ID
    pub symbols: String,
    /// Unix timestamp (seconds): return the first price published at or after this time instead of the latest
    #[serde(default)]
    pub at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythSearchInput {
    /// Search query
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythFeedsInput {
    /// Asset type filter (optional)
    pub asset_type: Option<String>,
}

// --- Address Book ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressAddInput {
    /// Name for the address
    pub name: String,
    /// Ethereum address
    pub address: String,
    /// Description or notes (optional)
    pub description: Option<String>,
    /// Tags for categorization (optional, can specify multiple)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Mark address as chain-specific (e.g., "polygon", "arbitrum")
    pub for_chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressNameInput {
    /// Address name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressImportInput {
    /// Path to JSON file to import
    pub file: String,
    /// Overwrite existing entries with same label
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressExportInput {
    /// Disabled over MCP; omit to return JSON in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressSearchInput {
    /// Search query
    pub query: String,
}

// --- Blacklist ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistAddInput {
    /// Token contract address to blacklist
    pub address: String,
    /// Token symbol for display (optional)
    pub symbol: Option<String>,
    /// Reason for blacklisting (optional)
    pub reason: Option<String>,
    /// Chain where this token exists (optional)
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistAddressInput {
    /// Address to check
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistScanInput {
    /// Address to scan
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistPathInput {}

// --- Contract Call ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractCallInput {
    /// Contract address
    pub address: String,
    /// Function name or full signature (e.g. "balanceOf", "balanceOf(address)" or "balanceOf(address)(uint256)"). A full signature with return types also works on unverified contracts
    pub sig: String,
    /// Arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number or "latest" (default: latest)
    pub block: Option<String>,
    /// Custom RPC URL (overrides config)
    pub rpc_url: Option<String>,
    /// Format output for human readability (commas in numbers, token decimals)
    #[serde(default)]
    pub human: bool,
}

// --- Cast Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastComputeAddressInput {
    /// Deployer address
    pub deployer: String,
    /// Nonce
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastCreate2Input {
    /// Deployer address
    pub deployer: String,
    /// Salt
    pub salt: String,
    /// Init code hash
    pub init_code_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastConcatInput {
    /// Values to concatenate
    pub values: Vec<String>,
}

// --- Simulate Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateBundleInput {
    /// JSON string of transaction array (e.g., '[{"from":"0x...","to":"0x...","data":"0x..."}]')
    pub txs: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateIdInput {
    /// Simulation ID
    pub id: String,
}

// --- Tenderly ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenderlySimulateInput {
    /// Contract address
    pub contract: String,
    /// Transaction data
    pub data: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Tenderly VNets ---

fn default_network_id() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsCreateInput")]
pub struct TenderlyVnetsCreateInput {
    /// Unique slug identifier for the VNet
    pub slug: String,
    /// Display name for the VNet
    pub name: String,
    /// Network ID to fork from (1 = mainnet, 137 = polygon, etc.)
    #[serde(default = "default_network_id")]
    pub network_id: u64,
    /// Block number to fork from (latest if not specified)
    #[serde(default)]
    pub block_number: Option<u64>,
    /// Custom chain ID for the VNet
    #[serde(default)]
    pub chain_id: Option<u64>,
    /// Enable state sync with parent network
    #[serde(default)]
    pub sync_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsIdInput")]
pub struct TenderlyVnetsIdInput {
    /// VNet ID (UUID)
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsDeleteInput")]
pub struct TenderlyVnetsDeleteInput {
    /// VNet ID(s) to delete (provide one or more IDs)
    #[serde(default)]
    pub ids: Vec<String>,
    /// Delete all Virtual TestNets
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsUpdateInput")]
pub struct TenderlyVnetsUpdateInput {
    /// VNet ID (UUID)
    pub id: String,
    /// New display name
    #[serde(default)]
    pub name: Option<String>,
    /// New slug
    #[serde(default)]
    pub slug: Option<String>,
    /// Enable/disable state sync
    #[serde(default)]
    pub sync_state: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsForkInput")]
pub struct TenderlyVnetsForkInput {
    /// Source VNet ID to fork from
    pub source: String,
    /// New slug identifier for the fork
    pub slug: String,
    /// Display name for the fork
    pub name: String,
    /// Block number to fork at (latest if not specified)
    #[serde(default)]
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsTransactionsInput")]
pub struct TenderlyVnetsTransactionsInput {
    /// VNet ID (UUID)
    pub id: String,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsSendInput")]
pub struct TenderlyVnetsSendInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Transaction data (hex encoded)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (default: 0)
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsGetTransactionInput")]
pub struct TenderlyVnetsGetTransactionInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Transaction hash
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsSimulateInput")]
pub struct TenderlyVnetsSimulateInput {
    /// VNet ID to simulate against
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient/contract address (omit for contract deployment)
    #[serde(default)]
    pub to: Option<String>,
    /// Calldata (hex encoded)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (default: 0)
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<u64>,
    /// Gas price (legacy transactions, in wei)
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Max fee per gas (EIP-1559, in wei)
    #[serde(default)]
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (EIP-1559, in wei)
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<String>,
}

// --- Tenderly VNets Admin ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBalanceInput")]
pub struct TenderlyVnetsAdminBalanceInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Account address
    pub address: String,
    /// Amount (wei, or use suffix: 1eth, 100gwei)
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminErc20Input")]
pub struct TenderlyVnetsAdminErc20Input {
    /// VNet ID (UUID)
    pub vnet: String,
    /// ERC20 token contract address
    pub token: String,
    /// Wallet address
    pub wallet: String,
    /// Token amount in smallest unit (only for set-erc20-balance, omit for set-max)
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminTimeInput")]
pub struct TenderlyVnetsAdminTimeInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Seconds to advance (increase-time) or Unix timestamp (set-timestamp)
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBlocksInput")]
pub struct TenderlyVnetsAdminBlocksInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Number of blocks to skip
    pub blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminVnetInput")]
pub struct TenderlyVnetsAdminVnetInput {
    /// VNet ID (UUID)
    pub vnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminRevertInput")]
pub struct TenderlyVnetsAdminRevertInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Snapshot ID to revert to
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminStorageInput")]
pub struct TenderlyVnetsAdminStorageInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Contract address
    pub address: String,
    /// Storage slot (32-byte hex or decimal, auto-padded)
    pub slot: String,
    /// Value to set (32-byte hex or decimal, auto-padded)
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminCodeInput")]
pub struct TenderlyVnetsAdminCodeInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Address to set code at
    pub address: String,
    /// Bytecode (hex string)
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSendTxInput")]
pub struct TenderlyVnetsAdminSendTxInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address (optional for contract creation)
    #[serde(default)]
    pub to: Option<String>,
    /// Transaction data (hex)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
}

// --- Tenderly VNet Admin RPC Simulation ---

fn default_block_tag() -> String {
    "latest".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSimulateTxInput")]
pub struct TenderlyVnetsAdminSimulateTxInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address (optional for contract creation)
    #[serde(default)]
    pub to: Option<String>,
    /// Transaction data (hex calldata)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (hex or decimal)
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit (hex or decimal)
    #[serde(default)]
    pub gas: Option<String>,
    /// Block tag or hex number (default: "latest")
    #[serde(default = "default_block_tag")]
    pub block: String,
    /// State overrides as JSON string: {"0xaddr": {"balance": "0x...", "stateDiff": {"0x0": "0x1"}}}
    #[serde(default)]
    pub state_overrides: Option<String>,
    /// Block overrides as JSON string: {"time": "0x...", "baseFee": "0x..."}
    #[serde(default)]
    pub block_overrides: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSimulateBundleInput")]
pub struct TenderlyVnetsAdminSimulateBundleInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Array of transactions as JSON string: [{"from":"0x...","to":"0x...","data":"0x..."}]
    pub txs: String,
    /// State overrides as JSON string
    #[serde(default)]
    pub state_overrides: Option<String>,
    /// Block overrides as JSON string
    #[serde(default)]
    pub block_overrides: Option<String>,
}

// --- Tenderly Actions Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyActionsIdsInput")]
pub struct TenderlyActionsIdsInput {
    /// Action ID(s)
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyActionsGetCallInput")]
pub struct TenderlyActionsGetCallInput {
    /// Action ID
    pub id: String,
    /// Execution ID
    pub execution_id: String,
}

// --- Tenderly Alerts Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyAlertsUpdateInput")]
pub struct TenderlyAlertsUpdateInput {
    /// Alert ID
    pub id: String,
    /// Alert name (required)
    pub name: String,
    /// Alert type, as Tenderly's simple_type (required, e.g., "successful_tx", "failed_tx", "log_emitted", "function_called", "erc20_transfer_log_emitted")
    pub alert_type: String,
    /// Network ID (e.g., "1" for mainnet)
    #[serde(default)]
    pub network: Option<String>,
    /// Target contract addresses (repeatable)
    #[serde(default)]
    pub addresses: Option<Vec<String>>,
}

// --- Tenderly Contracts Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsRemoveTagInput")]
pub struct TenderlyContractsRemoveTagInput {
    /// Contract address
    pub address: String,
    /// Network name
    #[serde(default)]
    pub network: Option<String>,
    /// Tag to remove
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsBulkTagInput")]
pub struct TenderlyContractsBulkTagInput {
    /// Tag to apply
    pub tag: String,
    /// Contract IDs to tag
    pub contract_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsEncodeStateInput")]
pub struct TenderlyContractsEncodeStateInput {
    /// Network ID (default: 1)
    #[serde(default)]
    pub network: Option<String>,
    /// Block number to encode against (default: latest)
    #[serde(default)]
    pub block_number: Option<String>,
    /// Named-variable state overrides as JSON, keyed by contract address, e.g.
    /// {"0xToken": {"value": {"balances[0xHolder]": "1000"}}}
    pub state_json: String,
}

// --- Tenderly VNets Admin Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBatchBalanceInput")]
pub struct TenderlyVnetsAdminBatchBalanceInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Addresses to set/add balances for
    pub addresses: Vec<String>,
    /// Amount (wei, or use suffix: 10eth, 100gwei)
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminCreateAccessListInput")]
pub struct TenderlyVnetsAdminCreateAccessListInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Transaction data (hex calldata)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Block tag or hex number (default: "latest")
    #[serde(default)]
    pub block: Option<String>,
}

// --- Alchemy Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesInput {
    /// Comma-separated token addresses
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugInput {
    /// Transaction hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftMetadataInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftContractInput {
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftIsHolderInput {
    /// Wallet address to check
    pub address: String,
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenMetadataInput {
    /// Token contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenAllowancesInput {
    /// Token contract address
    pub contract: String,
    /// Token owner address
    pub owner: String,
    /// Spender address
    pub spender: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersToInput {
    /// Destination address
    pub address: String,
    /// Starting block number or tag
    #[serde(default)]
    pub from_block: Option<String>,
    /// Ending block number or tag
    #[serde(default)]
    pub to_block: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesByAddressInput {
    /// Comma-separated token contract addresses
    pub addresses: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy NFT (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftContractOwnerInput {
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftAddressInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftForContractInput {
    /// Contract address
    pub contract: String,
    /// Starting token for pagination
    pub start_token: Option<String>,
    /// Maximum number of NFTs to return
    pub limit: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftForCollectionInput {
    /// Collection slug
    pub slug: String,
    /// Starting token for pagination
    pub start_token: Option<String>,
    /// Maximum number of NFTs to return
    pub limit: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyChainOnlyInput {
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Token (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenBalancesForTokensInput {
    /// Wallet address to query
    pub address: String,
    /// Comma-separated token contract addresses
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Transfers (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersAllInput {
    /// Address to query (both sent and received)
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Portfolio (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioTokenInfoInput {
    /// Comma-separated tokens in network:address format (e.g., eth-mainnet:0x...)
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioNftsInput {
    /// Address to query
    pub address: String,
    /// Include NFT metadata (default: true)
    #[serde(default = "default_true")]
    pub with_metadata: bool,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioNftContractsInput {
    /// Address to query
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Prices (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesBySymbolInput {
    /// Comma-separated token symbols
    pub symbols: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesHistoricalBySymbolInput {
    /// Token symbol (e.g., "ETH")
    pub symbol: String,
    /// Start time (ISO 8601 or Unix timestamp)
    pub start_time: String,
    /// End time (ISO 8601 or Unix timestamp)
    pub end_time: String,
    /// Time interval: 5m, 1h, 1d (default: 1h)
    pub interval: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesHistoricalByAddressInput {
    /// Token contract address
    pub address: String,
    /// Start time (ISO 8601 or Unix timestamp)
    pub start_time: String,
    /// End time (ISO 8601 or Unix timestamp)
    pub end_time: String,
    /// Time interval: 5m, 1h, 1d (default: 1h)
    pub interval: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Debug (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugTraceCallInput {
    /// Recipient address
    pub to: String,
    /// Block number or tag (default: "latest")
    pub block: Option<String>,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugBlockHashInput {
    /// Block hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugBlockInput {
    /// Block number (hex or decimal) or tag
    pub block: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Trace ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceBlockInput {
    /// Block number (hex or tag)
    pub block: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceCallInput {
    /// Recipient address
    pub to: String,
    /// Block number or tag (default: "latest")
    pub block: Option<String>,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceGetInput {
    /// Transaction hash
    pub hash: String,
    /// Trace indices (comma-separated, e.g., "0,1")
    pub indices: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceRawTxInput {
    /// Raw signed transaction (hex)
    pub raw_tx: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceReplayBlockInput {
    /// Block number (hex or tag)
    pub block: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceReplayTxInput {
    /// Transaction hash
    pub hash: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceTxInput {
    /// Transaction hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceFilterInput {
    /// From block (hex or tag)
    pub from_block: Option<String>,
    /// To block (hex or tag)
    pub to_block: Option<String>,
    /// Filter by sender addresses (comma-separated)
    pub from_address: Option<String>,
    /// Filter by recipient addresses (comma-separated)
    pub to_address: Option<String>,
    /// Skip first N traces
    pub after: Option<u32>,
    /// Maximum traces to return
    pub count: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Bundler ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBundlerEstimateGasInput {
    /// UserOperation as JSON string
    pub user_op_json: String,
    /// Entry point address (default: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789)
    pub entry_point: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBundlerHashInput {
    /// UserOperation hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Gas Manager ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyGasManagerPolicyInput {
    /// Policy ID
    pub policy_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Notify ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNotifyWebhookInput {
    /// Webhook ID
    pub webhook_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Beacon ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconBlockIdInput {
    /// Block ID (slot number, "head", "finalized", "justified", or block root hash)
    pub block_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconStateIdInput {
    /// State ID (slot number, "head", "finalized", "justified", or state root hash)
    pub state_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconValidatorsInput {
    /// State ID (slot number, "head", "finalized", "justified", or state root hash)
    pub state_id: String,
    /// Comma-separated validator indices or pubkeys (this or `status` is required)
    pub id: Option<String>,
    /// Comma-separated validator statuses, e.g. active_ongoing,pending_queued (this or `id` is required)
    pub status: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconValidatorBalancesInput {
    /// State ID (slot number, "head", "finalized", "justified", or state root hash)
    pub state_id: String,
    /// Comma-separated validator indices or pubkeys
    pub id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconStateEpochInput {
    /// State ID (slot number, "head", "finalized", "justified", or state root hash)
    pub state_id: String,
    /// Epoch (defaults to the state's epoch)
    pub epoch: Option<u64>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconValidatorInput {
    /// State ID
    pub state_id: String,
    /// Validator ID (index or pubkey)
    pub validator_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconDutiesInput {
    /// Epoch number
    pub epoch: String,
    /// Comma-separated validator indices
    pub validators: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconEpochInput {
    /// Epoch number
    pub epoch: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Solana ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAssetInput {
    /// Asset ID (mint address)
    pub id: String,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAssetsInput {
    /// Comma-separated asset IDs
    pub ids: String,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaOwnerInput {
    /// Owner address
    pub owner: String,
    /// Page number (default: 1)
    pub page: Option<u32>,
    /// Results per page (default: 100)
    pub limit: Option<u32>,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAddressInput {
    /// Address (creator, authority, etc.)
    pub address: String,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaGroupInput {
    /// Group key (e.g., "collection")
    pub group_key: String,
    /// Group value (e.g., collection address)
    pub group_value: String,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaTokenAccountsInput {
    /// Owner address
    pub owner: Option<String>,
    /// Mint address
    pub mint: Option<String>,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaMintInput {
    /// Mint address
    pub mint: String,
    /// Alchemy Solana network (solana-mainnet or solana-devnet). Defaults to solana-mainnet.
    #[serde(default = "default_solana_network")]
    pub network: String,
}

// --- Gecko Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoNftInput {
    /// NFT collection ID
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainInput {
    /// Network name (e.g., "ethereum")
    pub network: String,
    /// Token address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoTokenPriceInput {
    /// Platform ID (e.g., "ethereum", "polygon-pos")
    pub platform: String,
    /// Comma-separated contract addresses
    pub addresses: String,
    /// Target currencies (comma-separated, default: usd)
    #[serde(default)]
    pub vs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsListInput {
    /// Include platform contract addresses
    #[serde(default)]
    pub with_platforms: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsMarketsInput {
    /// Target currency (e.g., "usd")
    #[serde(default)]
    pub vs_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsChartInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Days of data (1, 7, 14, 30, 90, 180, 365, max)
    #[serde(default)]
    pub days: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsOhlcInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Days of data (1, 7, 14, 30, 90, 180, 365)
    #[serde(default)]
    pub days: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsHistoryInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Date in dd-mm-yyyy format
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsTopMoversInput {
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Duration: 1h, 24h, 7d, 14d, 30d, 60d, 1y
    #[serde(default)]
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoContractInput {
    /// Platform ID (e.g., "ethereum")
    pub platform: String,
    /// Contract address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoSearchInput {
    /// Search query
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainNetworkInput {
    /// Network ID (e.g., "eth", "polygon_pos")
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainOptionalNetworkInput {
    /// Network ID (optional, all networks if omitted)
    #[serde(default)]
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainTokenPriceInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Comma-separated token addresses
    pub addresses: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainPoolOhlcvInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Pool address
    pub address: String,
    /// Timeframe: minute, hour, day (default: hour)
    #[serde(default)]
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainTokenPriceMultiInput {
    /// Up to 50 network:address pairs, comma-separated (e.g. "eth:0xc02a...,base:0x4200...")
    pub tokens: String,
    /// Include market cap
    #[serde(default)]
    pub market_cap: bool,
    /// Include 24h volume
    #[serde(default)]
    pub volume: bool,
    /// Include 24h price change
    #[serde(default)]
    pub price_change: bool,
    /// Include total reserve in USD
    #[serde(default)]
    pub reserve: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainTokensMultiInput {
    /// Up to 50 network:address pairs, comma-separated (e.g. "eth:0xc02a...,base:0x4200...")
    pub tokens: String,
    /// Include each token's top pools
    #[serde(default)]
    pub top_pools: bool,
    /// Include pool composition (implies top_pools)
    #[serde(default)]
    pub composition: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainPoolTradesInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Pool address
    pub address: String,
    /// Range start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `to`; Analyst plan)
    #[serde(default)]
    pub from: Option<String>,
    /// Range end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `from`)
    #[serde(default)]
    pub to: Option<String>,
    /// Lookback without a range: 1d (default), 7d, 30d (7d/30d: Analyst plan)
    #[serde(default)]
    pub period: Option<String>,
    /// Trade side: base (default), quote or a token address
    #[serde(default)]
    pub token: Option<String>,
    /// Only trades worth more than this many USD
    #[serde(default)]
    pub min_volume_usd: Option<f64>,
    /// meta.next_cursor from the previous page (Analyst plan)
    #[serde(default)]
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (Analyst plan)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainTokenTradesInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Token address
    pub address: String,
    /// Range start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `to`; Analyst plan)
    #[serde(default)]
    pub from: Option<String>,
    /// Range end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `from`)
    #[serde(default)]
    pub to: Option<String>,
    /// Lookback without a range: 1d (default), 7d, 30d (7d/30d: Analyst plan)
    #[serde(default)]
    pub period: Option<String>,
    /// Only trades worth more than this many USD
    #[serde(default)]
    pub min_volume_usd: Option<f64>,
    /// meta.next_cursor from the previous page (Analyst plan)
    #[serde(default)]
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (Analyst plan)
    #[serde(default)]
    pub per_page: Option<u32>,
}

// --- Gecko RWAs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasListInput {
    /// Filter: stock, commodity or etf
    #[serde(default)]
    pub asset_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasMarketsInput {
    /// Filter: stock, commodity or etf
    #[serde(default)]
    pub asset_type: Option<String>,
    /// RWA IDs (comma-separated)
    #[serde(default)]
    pub ids: Option<String>,
    /// Issuer ID (see gecko_rwas_issuers)
    #[serde(default)]
    pub issuer: Option<String>,
    /// market_cap_desc (default), market_cap_asc, volume_desc, volume_asc, id_asc, id_desc
    #[serde(default)]
    pub order: Option<String>,
    /// Results per page, 1 to 250 (default 100)
    #[serde(default)]
    pub per_page: Option<u32>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasGetInput {
    /// RWA ID (e.g., "1-800-flowers-com")
    pub id: String,
    /// Also include its tokens, tokenized and perpetuals market data
    #[serde(default)]
    pub full: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasIdInput {
    /// RWA issuer ID (see gecko_rwas_issuers)
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasTickersInput {
    /// RWA ID
    pub id: String,
    /// Page number (100 tickers per page)
    #[serde(default)]
    pub page: Option<u32>,
    /// volume_desc (default), volume_asc, last_traded_at_desc, last_traded_at_asc
    #[serde(default)]
    pub order: Option<String>,
    /// Include 2% order book depth
    #[serde(default)]
    pub depth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoRwasChartInput {
    /// RWA ID
    pub id: String,
    /// Days of data, a number or "max" (default: 30)
    #[serde(default)]
    pub days: Option<String>,
    /// Data interval: 5m, hourly or daily (default: chosen from days)
    #[serde(default)]
    pub interval: Option<String>,
}

// --- Gecko Wallets ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoWalletsBalancesInput {
    /// Wallet address
    pub address: String,
    /// Network IDs, comma-separated, all EVM or all non-EVM (e.g., "eth,base")
    pub networks: String,
    /// value_usd_desc (default) or value_usd_asc
    #[serde(default)]
    pub sort: Option<String>,
    /// Minimum holding value in USD
    #[serde(default)]
    pub min_value_usd: Option<f64>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page, 1 to 100 (default 50)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoWalletsTransfersInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Wallet address
    pub address: String,
    /// Only this token contract address
    #[serde(default)]
    pub token: Option<String>,
    /// Window start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `to`; default: last 7 days)
    #[serde(default)]
    pub from: Option<String>,
    /// Window end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `from`)
    #[serde(default)]
    pub to: Option<String>,
    /// in (received) or out (sent); both when omitted
    #[serde(default)]
    pub direction: Option<String>,
    /// meta.next_cursor from the previous page
    #[serde(default)]
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (default 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoWalletsTradesInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Wallet address
    pub address: String,
    /// Only trades involving this token contract address
    #[serde(default)]
    pub token: Option<String>,
    /// Window start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `to`; default: last 7 days)
    #[serde(default)]
    pub from: Option<String>,
    /// Window end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (needs `from`)
    #[serde(default)]
    pub to: Option<String>,
    /// meta.next_cursor from the previous page
    #[serde(default)]
    pub cursor: Option<String>,
    /// Results per page, 1 to 300 (default 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoWalletsPnlInput {
    /// Wallet address
    pub address: String,
    /// Network IDs, comma-separated, all EVM or all non-EVM (e.g., "eth,base")
    pub networks: String,
    /// realized_pnl_usd_desc (default), unrealized_pnl_usd_desc, total_buy_usd_desc, total_sell_usd_desc
    #[serde(default)]
    pub sort: Option<String>,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page, 1 to 300 (default 100)
    #[serde(default)]
    pub per_page: Option<u32>,
}

// --- Gecko News & Insights ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoNewsInput {
    /// Only news about this coin ID (also returns guides)
    #[serde(default)]
    pub coin: Option<String>,
    /// Language code, e.g. en (default), de, ja, zh-tw
    #[serde(default)]
    pub language: Option<String>,
    /// all (default), news or guides (guides needs coin)
    #[serde(default)]
    pub news_type: Option<String>,
    /// Page, 1 to 20
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page, 1 to 20
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoInsightsInput {
    /// Only insights about this coin ID
    #[serde(default)]
    pub coin: Option<String>,
    /// Start date (YYYY-MM-DD)
    #[serde(default)]
    pub from: Option<String>,
    /// End date (YYYY-MM-DD)
    #[serde(default)]
    pub to: Option<String>,
    /// Page, 1 to 20
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page, 1 to 20
    #[serde(default)]
    pub per_page: Option<u32>,
}

// --- GoPlus Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoplusBatchInput {
    /// Comma-separated addresses
    pub addresses: String,
    /// Chain ID
    pub chain_id: u64,
}

// --- Solodit Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
pub struct SoloditSlugInput {
    /// Finding slug/ID
    pub slug: String,
}

// --- Llama Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaCoinsInput {
    /// Addresses in format "chain:address" (comma-separated)
    pub addresses: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaProtocolInput {
    /// Protocol slug (e.g., "uniswap", "aave")
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaYieldsEarnInput {
    /// Chain slugs, lowercase, comma-separated (any match; e.g. "ethereum,base")
    #[serde(default)]
    pub chain: Option<String>,
    /// Protocol slugs, lowercase, comma-separated (any match; e.g. "aave-v3")
    #[serde(default)]
    pub protocol: Option<String>,
    /// true: only all-stablecoin pools; false: only the rest
    #[serde(default)]
    pub stablecoin: Option<bool>,
    /// true: only single-asset pools; false: only the rest
    #[serde(default)]
    pub single_asset: Option<bool>,
    /// Minimum TVL in USD
    #[serde(default)]
    pub min_tvl: Option<f64>,
    /// Page number (1-based)
    #[serde(default)]
    pub page: Option<u32>,
    /// Page size, 1 to 300 (default 100)
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaYieldsIdInput {
    /// Earn pool ID
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaYieldsHistoryInput {
    /// Earn pool ID (borrow market ID for llama_yields_borrow_history)
    pub id: String,
    /// History window: 30d, 90d or max (default: full series)
    #[serde(default)]
    pub range: Option<String>,
}

// --- Moralis ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisChainOnlyInput {
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressInput {
    /// Wallet or contract address
    pub address: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisWalletInsightInput {
    /// Wallet address
    pub address: String,
    /// Only these chains, comma-separated (e.g. "eth,base"; default: every supported chain)
    #[serde(default)]
    pub chains: Option<String>,
    /// Include a per-chain breakdown
    #[serde(default)]
    pub chain_breakdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisTokenScoreHistoryInput {
    /// Token contract address
    pub address: String,
    /// Timeframe: 1d, 7d (default) or 30d
    #[serde(default)]
    pub timeframe: Option<String>,
    /// Blockchain chain (EVM only, e.g., eth, polygon, bsc, arbitrum, base). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDomainInput {
    /// Domain to resolve (e.g., ENS, Unstoppable)
    pub domain: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisSearchInput {
    /// Search query string
    pub query: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftMetadataInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressesInput {
    /// Comma-separated addresses
    pub addresses: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftContractInput {
    /// NFT contract address
    pub contract: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftTokenInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisMultipleNftsInput {
    /// Comma-separated token_address:token_id pairs
    pub tokens: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisTxHashInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisTxVerboseInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Include internal transactions
    #[serde(default)]
    pub include_internal: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisWalletVerboseInput {
    /// Wallet address
    pub address: String,
    /// Include internal transactions
    #[serde(default)]
    pub include_internal: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisBlockInput {
    /// Block number or hash
    pub block_number_or_hash: String,
    /// Include internal transactions in the response
    #[serde(default)]
    pub include_transactions: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDateInput {
    /// Date in ISO 8601 format (e.g. 2023-01-01T00:00:00Z)
    pub date: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDefiPairPriceInput {
    /// Token 0 address
    pub token0: String,
    /// Token 1 address
    pub token1: String,
    /// DEX exchange name (e.g., uniswapv2, sushiswap)
    pub exchange: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDefiPairAddressInput {
    /// Token 0 address
    pub token0: String,
    /// Token 1 address
    pub token1: String,
    /// DEX exchange name
    pub exchange: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisProtocolPositionsInput {
    /// Wallet address
    pub address: String,
    /// Protocol name (e.g., aave, uniswap-v3)
    pub protocol: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAnalyticsTimeseriesInput {
    /// Comma-separated token addresses
    pub addresses: String,
    /// Timeframe (1d, 7d or 30d). Defaults to 1d.
    pub timeframe: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisEntityIdInput {
    /// Entity ID
    pub entity_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisCategoryIdInput {
    /// Category ID
    pub category_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

// --- Dune ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueryInput {
    /// Query ID
    pub query_id: String,
    /// Query parameters as a JSON object, e.g. {"address": "0x..."}
    #[serde(default)]
    pub params: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneSqlInput {
    /// SQL query
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneExecutionInput {
    /// Execution ID
    pub execution_id: String,
}

// --- Dune Queries CRUD ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesCreateInput {
    /// Query name
    pub name: String,
    /// SQL query body
    pub sql: String,
    /// Query description
    pub description: Option<String>,
    /// Make query private
    #[serde(default)]
    pub private: bool,
    /// Comma-separated tags
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesGetInput {
    /// Query ID
    pub query_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesUpdateInput {
    /// Query ID to update
    pub query_id: String,
    /// New query name
    pub name: Option<String>,
    /// New SQL query body
    pub sql: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New comma-separated tags
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesIdInput {
    /// Query ID
    pub query_id: String,
}

// --- Dune Tables CRUD ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesCreateInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
    /// JSON schema definition for the table columns
    pub schema_json: String,
    /// Table description
    pub description: Option<String>,
    /// Make table private
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesUploadCsvInput {
    /// Table name
    pub table_name: String,
    /// CSV data content (or file path if --file is used)
    pub data: String,
    /// Table description
    pub description: Option<String>,
    /// Make table private
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesGetInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesInsertInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
    /// JSON data to insert (array of row objects)
    pub data_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesNamespaceTableInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
}

// --- Dune Materialized Views ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsUpsertInput {
    /// Materialized view name
    pub name: String,
    /// Source query ID
    pub query_id: String,
    /// Cron schedule expression (e.g., "0 */6 * * *")
    pub cron: Option<String>,
    /// Expiration timestamp (ISO 8601)
    pub expires_at: Option<String>,
    /// Make materialized view private
    #[serde(default)]
    pub private: bool,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsNameInput {
    /// Materialized view name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsRefreshInput {
    /// Materialized view name
    pub name: String,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

// --- Dune Pipelines ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DunePipelinesExecuteInput {
    /// Pipeline definition as JSON, e.g. {"nodes":[{"id":1,"query_execution":{"query_id":123}},{"id":2,"dependencies":[1],"materialized_view_refresh":{"name":"dune.team.result_x"}}]}
    pub pipeline_json: String,
    /// Parameters as JSON string
    pub params: Option<String>,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DunePipelinesStatusInput {
    /// Pipeline execution ID
    pub execution_id: String,
}

// --- Curve Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveRouterEncodeInput {
    /// Input token address
    pub from: String,
    /// Output token address
    pub to: String,
    /// Input amount (raw, no decimals)
    pub amount: String,
    /// Minimum output amount (raw, no decimals)
    pub min_out: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainRegistryInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Registry ID (e.g., "main", "factory", "factory-crypto")
    pub registry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveOptionalChainInput {
    /// Chain name (optional, all chains if omitted)
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainAddressInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Token or contract address
    pub address: String,
}

/// Input for `curve_trades` (the trades endpoint takes no time range)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveTradesInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Pool contract address
    pub address: String,
    /// Main token address (pool coin)
    pub main_token: String,
    /// Reference token address (pool coin being priced)
    pub reference_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurvePoolPairInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Pool contract address
    pub address: String,
    /// Main token address (pool coin; OHLC prices are denominated in it)
    pub main_token: String,
    /// Reference token address (pool coin being priced)
    pub reference_token: String,
    /// Start timestamp (unix seconds, optional; default end - 7 days)
    pub start: Option<u64>,
    /// End timestamp (unix seconds, optional; default now)
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainAddressTimeRangeInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Token or contract address
    pub address: String,
    /// Start timestamp (unix seconds)
    pub start: Option<u64>,
    /// End timestamp (unix seconds)
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveDaoLockersInput {
    /// Number of top lockers to return
    pub top: Option<u32>,
}

// --- CEX Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexExchangeInput {
    /// Exchange: binance, bitget, okx, bybit, coinbase, kraken, hyperliquid, lighter
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexOhlcvInput {
    /// Exchange: binance, bitget, okx, bybit, coinbase, kraken, hyperliquid, lighter
    pub exchange: String,
    /// Trading symbol: BASE/QUOTE for spot (BTC/USDT, ETH/USD), BASE/USDC:USDC for hyperliquid and lighter perps
    pub symbol: String,
    /// Timeframe (e.g., "1m", "1h", "1d")
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexTradesInput {
    /// Exchange: binance, bitget, okx, bybit, coinbase, kraken, hyperliquid, lighter
    pub exchange: String,
    /// Trading symbol: BASE/QUOTE for spot (BTC/USDT, ETH/USD), BASE/USDC:USDC for hyperliquid and lighter perps
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexCompareInput {
    /// Trading symbol: BASE/QUOTE for spot (BTC/USDT, ETH/USD), BASE/USDC:USDC for hyperliquid and lighter perps
    pub symbol: String,
}

// --- Uniswap Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPoolAddressInput {
    /// Pool address
    pub pool: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

/// A pool on the Ethereum mainnet Uniswap subgraph
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapSubgraphPoolInput {
    /// Pool address (Ethereum mainnet)
    pub pool: String,
    /// Uniswap version (v2, v3, v4)
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapBalanceInput {
    /// Wallet address
    pub address: String,
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Kong ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongChainInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongTvlInput {
    /// Vault or strategy address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongReportsInput {
    /// Report type: "vault" or "strategy"
    pub report_type: String,
    /// Vault or strategy address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongPricesInput {
    /// Token contract address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

// --- 1inch Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchSwapInput {
    /// Source token address
    pub src: String,
    /// Destination token address
    pub dst: String,
    /// Amount
    pub amount: String,
    /// Sender address
    pub from: String,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchChainInput {
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchAllowanceInput {
    /// Token address
    pub token: String,
    /// Owner address
    pub owner: String,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchApproveInput {
    /// Token address
    pub token: String,
    /// Amount (optional, max if omitted)
    pub amount: Option<String>,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

// --- OpenOcean Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanSwapInput {
    /// Input token address
    pub in_token: String,
    /// Output token address
    pub out_token: String,
    /// Amount in smallest units (wei)
    pub amount: String,
    /// Account address
    pub account: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- KyberSwap ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KyberRoutesInput {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KyberBuildInput {
    /// Source token address
    pub token_in: String,
    /// Destination token address
    pub token_out: String,
    /// Amount in smallest units (wei)
    pub amount_in: String,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
    /// Route summary JSON from routes command
    pub route_summary: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KyberSwapInput {
    /// Source token address
    pub token_in: String,
    /// Destination token address
    pub token_out: String,
    /// Amount in smallest units (wei)
    pub amount_in: String,
    /// Sender address (input tokens are taken from it)
    pub sender: String,
    /// Recipient address (defaults to sender)
    #[serde(default)]
    pub recipient: Option<String>,
    /// Slippage tolerance in basis points (default: 50 = 0.5%)
    #[serde(default)]
    pub slippage_bps: Option<u32>,
    /// Exclude RFQ sources
    #[serde(default)]
    pub exclude_rfq: bool,
    /// KyberSwap partner client ID (falls back to KYBERSWAP_CLIENT_ID; the endpoint is partner-only)
    #[serde(default)]
    pub client_id: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- 0x Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxPriceInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Taker address (wallet executing the swap)
    pub taker: String,
    /// Use the AllowanceHolder flow (approve `allowanceTarget`, no Permit2 signature) instead of Permit2
    #[serde(default)]
    pub allowance_holder: bool,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxChainsInput {
    /// List the Gasless API's chains instead of the Swap API's
    #[serde(default)]
    pub gasless: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxGaslessInput {
    /// Sell token address (ERC-20; the native token is not supported)
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Sell amount in smallest units
    pub sell_amount: String,
    /// Taker address (wallet executing the swap)
    pub taker: String,
    /// Slippage in basis points (default: 100 = 1%)
    #[serde(default)]
    pub slippage_bps: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxGaslessStatusInput {
    /// Trade hash (`trade.hash` from zerox_gasless_quote)
    pub trade_hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CoW Swap Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOrderInput {
    /// Order UID
    pub order_uid: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOrdersInput {
    /// Owner address
    pub owner: String,
    /// Pagination offset
    pub offset: Option<u64>,
    /// Max orders to return, 1-1000 (API default: 10)
    pub limit: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapTradesInput {
    /// Owner address
    pub owner: String,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<u64>,
    /// Max trades to return, 1-1000 (API default: 10)
    #[serde(default)]
    pub limit: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOrderTradesInput {
    /// Order UID
    pub order_uid: String,
    /// Pagination offset
    #[serde(default)]
    pub offset: Option<u64>,
    /// Max trades to return, 1-1000 (API default: 10)
    #[serde(default)]
    pub limit: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapAuctionInput {
    /// Auction ID (omit both auction_id and tx_hash for the latest competition)
    #[serde(default)]
    pub auction_id: Option<String>,
    /// Settlement transaction hash (alternative to auction_id)
    #[serde(default)]
    pub tx_hash: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapTokenInput {
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapTxHashInput {
    /// Settlement transaction hash
    pub tx_hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOrdersByUidsInput {
    /// Order UIDs (up to 128; unknown UIDs are left out of the result)
    pub uids: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapAppDataInput {
    /// appData hash (32-byte hex)
    pub app_data_hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapAddressInput {
    /// User address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CowSwap Order Management ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapCreateOrderInput {
    /// Source token address
    pub sell_token: String,
    /// Destination token address
    pub buy_token: String,
    /// Sell amount in smallest units (wei)
    pub sell_amount: String,
    /// Buy amount in smallest units (wei)
    pub buy_amount: String,
    /// Order expiration (Unix timestamp)
    pub valid_to: u64,
    /// Order creator address
    pub from: String,
    /// Receiver address
    pub receiver: String,
    /// Signed order data (hex)
    pub signature: String,
    /// Order kind: "sell" or "buy"
    #[serde(default = "default_order_kind")]
    pub kind: String,
    /// Signing scheme: "eip712", "eip1271", or "presign"
    #[serde(default = "default_signing_scheme")]
    pub signing_scheme: String,
    /// Fee amount in sell token (smallest units)
    #[serde(default)]
    pub fee_amount: Option<String>,
    /// App data hash (32 bytes hex)
    #[serde(default)]
    pub app_data: Option<String>,
    /// Allow partial fills
    #[serde(default)]
    pub partially_fillable: bool,
    /// Quote ID reference
    pub quote_id: Option<i64>,
    /// Chain name or ID (ethereum, gnosis, arbitrum, base, polygon, bnb, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
}

fn default_order_kind() -> String {
    "sell".to_string()
}

fn default_signing_scheme() -> String {
    "eip712".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapCancelOrderInput {
    /// Order UID to cancel
    pub uid: String,
    /// EIP-712 signature proving ownership
    pub signature: String,
    /// Chain name or ID (ethereum, gnosis, arbitrum, base, polygon, bnb, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapCancelOrdersInput {
    /// Order UIDs to cancel (up to 128)
    pub uids: Vec<String>,
    /// Signature of `OrderCancellations(bytes[] orderUids)` from the orders' owner
    pub signature: String,
    /// Signing scheme: "eip712" or "ethsign"
    #[serde(default)]
    pub signing_scheme: Option<String>,
    /// Chain name or ID (ethereum, gnosis, arbitrum, base, polygon, bnb, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- LiFi Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiStatusInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Bridge name (optional)
    pub bridge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiChainInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    pub chain: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiGasInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    pub chain_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiTokensInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_lifi_chain_id")]
    pub chain_id: u64,
}

fn default_lifi_chain_id() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiConnectionsInput {
    /// Source chain ID (required by LI.FI)
    pub from_chain: u64,
    /// Destination chain ID (required by LI.FI)
    pub to_chain: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiStepTransactionInput {
    /// Step JSON from a route response
    pub step_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiTokenInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    pub chain_id: u64,
    /// Token address or symbol (e.g. USDC)
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiQuoteToAmountInput {
    /// Source chain ID (e.g. 1 for Ethereum, 42161 for Arbitrum)
    pub from_chain: u64,
    /// Source token address
    pub from_token: String,
    /// Destination chain ID (e.g. 10 for Optimism, 8453 for Base)
    pub to_chain: u64,
    /// Destination token address
    pub to_token: String,
    /// Amount to receive, in the destination token's smallest units
    pub to_amount: String,
    /// Sender address
    pub from_address: String,
    /// Receiver address (defaults to sender)
    #[serde(default)]
    pub to_address: Option<String>,
    /// Slippage tolerance in percent (e.g. 0.5 = 0.5%; default 0.5)
    #[serde(default)]
    pub slippage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiContractCallsInput {
    /// Source chain ID (e.g. 1 for Ethereum, 42161 for Arbitrum)
    pub from_chain: u64,
    /// Source token address
    pub from_token: String,
    /// Destination chain ID
    pub to_chain: u64,
    /// Token the contract calls need on the destination chain
    pub to_token: String,
    /// Amount of `to_token` the calls need, in smallest units
    pub to_amount: String,
    /// Sender address
    pub from_address: String,
    /// Contract calls as a JSON array: [{"fromAmount":"...","fromTokenAddress":"0x...","toContractAddress":"0x...","toContractCallData":"0x...","toContractGasLimit":"..."}]
    pub calls: String,
    /// Where bridged tokens go if a call fails (defaults to sender)
    #[serde(default)]
    pub fallback_address: Option<String>,
    /// Slippage tolerance in percent (e.g. 0.5 = 0.5%; default 0.5)
    #[serde(default)]
    pub slippage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiTransfersInput {
    /// Sending or receiving wallet address
    pub wallet: String,
    /// Transfer status: all, done, pending, failed (API default: done)
    #[serde(default)]
    pub status: Option<String>,
    /// Page size (API default: 10)
    #[serde(default)]
    pub limit: Option<u32>,
    /// Cursor from a previous response's `next`
    #[serde(default)]
    pub next: Option<String>,
    /// Cursor from a previous response's `previous` (not with `next`)
    #[serde(default)]
    pub previous: Option<String>,
    /// Oldest timestamp to include, unix seconds (default: last 30 days)
    #[serde(default)]
    pub from_timestamp: Option<u64>,
    /// Newest timestamp to include, unix seconds
    #[serde(default)]
    pub to_timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiGasStatusInput {
    /// Transaction hash that started the LI.Fuel refuel
    pub tx_hash: String,
}

// --- Velora ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraPriceInput {
    /// Source token address
    pub src_token: String,
    /// Destination token address
    pub dest_token: String,
    /// Amount in smallest units (wei)
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraTxInput {
    /// User address (wallet executing the swap)
    pub user_address: String,
    /// Price route JSON from price command
    pub price_route: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Slippage in basis points (e.g., 100 = 1%)
    pub slippage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraSwapInput {
    /// Source token address
    pub src_token: String,
    /// Destination token address
    pub dest_token: String,
    /// Amount in smallest units (wei)
    pub amount: String,
    /// User address (wallet executing the swap)
    pub user_address: String,
    /// Side: SELL (default) or BUY
    #[serde(default)]
    pub side: Option<String>,
    /// Slippage in basis points, 1-5000 (default: 100 = 1%)
    #[serde(default)]
    pub slippage: Option<u32>,
    /// Receiver address (defaults to user address)
    #[serde(default)]
    pub receiver: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Enso ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoRouteInput {
    /// From token address
    pub from_token: String,
    /// To token address
    pub to_token: String,
    /// Amount
    pub amount: String,
    /// Sender address (required by Enso API)
    pub from_address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoPriceInput {
    /// Token address
    pub token: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoBalancesInput {
    /// Wallet address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoBundleInput {
    /// Sender address
    pub from_address: String,
    /// Actions JSON array: [{"protocol":"...","action":"...","args":{...}}]
    pub actions_json: String,
    /// Chain ID (e.g., "1" for Ethereum)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    /// Routing strategy: router, delegate, ensowallet
    pub routing_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoQuoteInput {
    /// From token address
    pub from_token: String,
    /// To token address
    pub to_token: String,
    /// Amount in smallest units (wei)
    pub amount: String,
    /// Sender address (optional for quotes)
    #[serde(default)]
    pub from_address: Option<String>,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    /// Aggregators to leave out, comma-separated (e.g. "1inch,0x")
    #[serde(default)]
    pub ignore_aggregators: Option<String>,
    /// Latency/quality trade-off: light, medium, high, max
    #[serde(default)]
    pub effort: Option<String>,
    /// Swaps the quote may use: any, rfq, amm
    #[serde(default)]
    pub swap_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoPricesInput {
    /// Token addresses (max 100)
    pub tokens: Vec<String>,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoTokensInput {
    /// Only this chain ID (omit to search every chain)
    #[serde(default)]
    pub chain_id: Option<String>,
    /// Only this project (e.g. aave)
    #[serde(default)]
    pub project: Option<String>,
    /// Only this protocol slug (e.g. aave-v3)
    #[serde(default)]
    pub protocol: Option<String>,
    /// Only these token addresses (comma-separated)
    #[serde(default)]
    pub address: Option<String>,
    /// Only these symbols (comma-separated)
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token type: defi or base
    #[serde(default)]
    pub token_type: Option<String>,
    /// Include symbol, name and logos
    #[serde(default)]
    pub include_metadata: bool,
    /// Page number
    #[serde(default)]
    pub page: Option<u64>,
    /// Page size (max 1000)
    #[serde(default)]
    pub page_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoProtocolsInput {
    /// Only protocols on this chain ID (omit for every chain)
    #[serde(default)]
    pub chain_id: Option<String>,
    /// Only the protocol with this slug (e.g. aave-v3)
    #[serde(default)]
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoApproveInput {
    /// Token address
    pub token: String,
    /// Amount in smallest units
    pub amount: String,
    /// Token owner (sender) address
    pub from_address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    /// Routing strategy (approve the smart wallet with ensowallet/delegate; default: router)
    #[serde(default)]
    pub routing_strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoWalletInput {
    /// Owner address
    pub from_address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    /// Routing strategy (router, delegate, ensowallet)
    #[serde(default)]
    pub routing_strategy: Option<String>,
}

fn default_chain_id() -> String {
    "1".to_string()
}

// --- CEX ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexTickerInput {
    /// Exchange: binance, bitget, okx, bybit, coinbase, kraken, hyperliquid, lighter
    pub exchange: String,
    /// Trading symbol: BASE/QUOTE for spot (BTC/USDT, ETH/USD), BASE/USDC:USDC for hyperliquid and lighter perps
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CexOrderbookInput {
    /// Exchange: binance, bitget, okx, bybit, coinbase, kraken, hyperliquid, lighter
    pub exchange: String,
    /// Trading symbol: BASE/QUOTE for spot (BTC/USDT, ETH/USD), BASE/USDC:USDC for hyperliquid and lighter perps
    pub symbol: String,
    /// Depth limit
    pub limit: Option<u32>,
}

// --- Simulate ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// Unknown fields are errors so callers still sending removed fields find out
#[serde(deny_unknown_fields)]
pub struct SimulateCallInput {
    /// Target contract address
    pub contract: String,
    /// Function signature (e.g., "transfer(address,uint256)")
    pub sig: Option<String>,
    /// Raw calldata (hex encoded, alternative to sig)
    pub data: Option<String>,
    /// Function arguments (used with sig).
    /// For array types (e.g., address[], uint256[]), pass the array in brackets WITHOUT quotes:
    ///   - address[]: `[0x1234...,0x5678...]` (no quotes, no spaces)
    ///   - uint256[]: `[1,2,3]`
    ///
    /// Example for swapExactETHForTokens(uint256,address[],address,uint256):
    /// `args: ["0", "[0xC02aaA39...,0xA0b86991...]", "0xRecipient", "999999999"]`
    #[serde(default)]
    pub args: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,

    // === Transaction parameters ===
    /// Sender address (default: zero address)
    pub from: Option<String>,
    /// Value to send in wei (default: 0)
    pub value: Option<String>,
    /// Block number or tag (latest, pending, etc.)
    pub block: Option<String>,
    /// Gas limit
    pub gas: Option<String>,
    /// Gas price in wei
    pub gas_price: Option<String>,

    // === State overrides (format: "address=value" or "address:slot=value") ===
    /// Set balance overrides (format: address=wei, can specify multiple)
    #[serde(default)]
    pub balance_overrides: Vec<String>,
    /// Set storage overrides (format: address:slot=value, can specify multiple)
    #[serde(default)]
    pub storage_overrides: Vec<String>,
    /// Set code overrides (format: address=bytecode, can specify multiple)
    #[serde(default)]
    pub code_overrides: Vec<String>,
    /// Set nonce overrides (format: address=nonce, can specify multiple)
    #[serde(default)]
    pub nonce_overrides: Vec<String>,

    // === Block overrides ===
    /// Override block timestamp (unix seconds)
    pub block_timestamp: Option<u64>,
    /// Override block number
    pub block_number_override: Option<u64>,
    /// Override block gas limit
    pub block_gas_limit: Option<String>,
    /// Override block coinbase/miner address
    pub block_coinbase: Option<String>,
    /// Override block difficulty
    pub block_difficulty: Option<String>,
    /// Override block base fee per gas (wei)
    pub block_base_fee: Option<String>,
    /// Transaction index within the block
    pub transaction_index: Option<u32>,

    // === L2 options (Optimism, etc.) ===
    /// L1 block number (for L2 simulations)
    pub l1_block_number: Option<u64>,
    /// L1 timestamp (for L2 simulations)
    pub l1_timestamp: Option<u64>,
    /// L1 message sender (for L2 cross-chain simulations)
    pub l1_message_sender: Option<String>,
    /// Mark as deposit transaction (Optimism Bedrock)
    #[serde(default)]
    pub deposit_tx: bool,
    /// Mark as system transaction (Optimism Bedrock)
    #[serde(default)]
    pub system_tx: bool,

    // === Simulation options ===
    /// Simulation backend (cast, anvil, tenderly, debug, trace, alchemy)
    pub via: Option<String>,
    /// RPC URL (for debug backend)
    pub rpc_url: Option<String>,
    /// Show execution trace
    #[serde(default)]
    pub trace: bool,
    /// Decode internal functions in Foundry traces
    #[serde(default)]
    pub decode_internal: bool,
    /// Disable address labels in Foundry traces
    #[serde(default)]
    pub disable_labels: bool,
    /// Label addresses in Foundry traces (format: address:label)
    #[serde(default)]
    pub labels: Vec<String>,
    /// EVM version for Foundry trace execution
    pub evm_version: Option<String>,
    /// Use local Foundry project artifacts for trace decoding
    #[serde(default)]
    pub with_local_artifacts: bool,
    /// RPC timeout in seconds for Foundry calls
    pub rpc_timeout: Option<u64>,
    /// Disable automatic proxy detection in Foundry RPC clients
    #[serde(default)]
    pub no_proxy: bool,
    /// Additional Anvil fork RPC URLs
    #[serde(default)]
    pub fork_urls: Vec<String>,
    /// Anvil fork block number or negative offset from latest
    pub fork_block_number: Option<String>,
    /// Anvil fork state after a specific transaction hash
    pub fork_transaction_hash: Option<String>,
    /// Anvil fork chain ID for offline-start mode
    pub fork_chain_id: Option<u64>,
    /// Anvil hardfork to use (e.g., prague, cancun)
    pub hardfork: Option<String>,
    /// Anvil network family to enable (ethereum, optimism, tempo)
    pub network: Option<String>,
    /// Disable Anvil fork RPC rate limiting
    #[serde(default)]
    pub no_rate_limit: bool,
    /// Disable Anvil fork storage caching
    #[serde(default)]
    pub no_storage_caching: bool,
    /// Anvil upstream fork RPC timeout in milliseconds
    pub fork_timeout_ms: Option<u64>,
    /// Anvil upstream fork retry count
    pub fork_retries: Option<u32>,
    /// Disable Anvil block gas limit checks
    #[serde(default)]
    pub disable_block_gas_limit: bool,
    /// Enable Anvil transaction gas limit checks
    #[serde(default)]
    pub enable_tx_gas_limit: bool,
    /// Enable precise gas estimation (Tenderly)
    #[serde(default)]
    pub estimate_gas: bool,
    /// Generate EIP-2930 access list in response (Tenderly)
    #[serde(default)]
    pub generate_access_list: bool,
    /// Access list as inline JSON
    pub access_list: Option<String>,
    /// Simulation type: full, quick, or abi
    pub simulation_type: Option<String>,
    /// Network ID to simulate on
    pub network_id: Option<u64>,
    /// Save simulation to Tenderly (returns simulation ID)
    #[serde(default)]
    pub save: bool,

    // === Tenderly options ===
    /// Tenderly account slug (or use TENDERLY_ACCOUNT env)
    pub tenderly_account: Option<String>,
    /// Tenderly project slug (or use TENDERLY_PROJECT env)
    pub tenderly_project: Option<String>,

    // === Alchemy options ===
    /// Alchemy network (e.g., eth-mainnet, polygon-mainnet, arb-mainnet)
    pub alchemy_network: Option<String>,

    // === Debug options ===
    /// Dry run - output request without executing (json, curl, fetch, powershell, url, python, httpie, wget, go, rust, axios)
    pub dry_run: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
// Unknown fields are errors so callers still sending removed fields find out
#[serde(deny_unknown_fields)]
pub struct SimulateTxInput {
    /// Transaction hash to simulate
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Simulation backend (cast, debug, trace, alchemy). `tenderly` is not
    /// supported for historical transactions (Tenderly has no trace-by-hash API).
    pub via: Option<String>,
    /// RPC URL
    pub rpc_url: Option<String>,
    /// Show full opcode trace
    #[serde(default)]
    pub trace: bool,
    /// Open interactive Foundry debugger
    #[serde(default)]
    pub debug: bool,
    /// Only execute with previous-block state for faster Foundry replay
    #[serde(default)]
    pub quick: bool,
    /// Decode internal functions in Foundry traces
    #[serde(default)]
    pub decode_internal: bool,
    /// Limit Foundry trace depth
    pub trace_depth: Option<u32>,
    /// Replay system transactions before the target transaction
    #[serde(default)]
    pub replay_system_txs: bool,
    /// Disable address labels in Foundry traces
    #[serde(default)]
    pub disable_labels: bool,
    /// Label addresses in Foundry traces (format: address:label)
    #[serde(default)]
    pub labels: Vec<String>,
    /// EVM version for Foundry replay
    pub evm_version: Option<String>,
    /// Use local Foundry project artifacts for trace decoding
    #[serde(default)]
    pub with_local_artifacts: bool,
    /// Disable automatic proxy detection in Foundry RPC clients
    #[serde(default)]
    pub no_proxy: bool,
    /// RPC timeout in seconds for Foundry replay
    pub rpc_timeout: Option<u64>,
    /// Enable transaction gas limit checks in Foundry replay
    #[serde(default)]
    pub enable_tx_gas_limit: bool,
    /// Disable block gas limit checks in Foundry replay
    #[serde(default)]
    pub disable_block_gas_limit: bool,
    /// Print raw callTracer JSON instead of the decoded call tree (debug backend)
    #[serde(default)]
    pub raw: bool,
}

// --- Config ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigKeyInput {
    /// API key value
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigTenderlyInput {
    /// Tenderly account name
    pub account: String,
    /// Tenderly project name
    pub project: String,
    /// Tenderly API key
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigChainlinkInput {
    /// Chainlink API key (client ID)
    pub key: String,
    /// Chainlink user secret (client secret)
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDebugRpcInput {
    /// RPC URL for debug mode
    pub url: String,
}

// --- Endpoints ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsHealthInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of probe requests per endpoint
    #[serde(default = "default_endpoint_health_probes")]
    pub probes: u32,
}

fn default_endpoint_health_probes() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsUrlInput {
    /// RPC URL
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsAddInput {
    /// RPC URL to add
    pub url: String,
    /// Chain this endpoint serves (auto-detected if not specified)
    pub chain: Option<String>,
    /// Skip optimization (just add with defaults)
    #[serde(default)]
    pub no_optimize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsOptimizeInput {
    /// Optional specific RPC URL to optimize (if omitted, optimizes all)
    pub url: Option<String>,
    /// Only optimize endpoints for this chain (ignored when url is set; omit for all chains)
    pub chain: Option<String>,
}

// --- Chainlist ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistSearchInput {
    /// Search query (chain name, short name, or chain ID)
    pub query: String,
    /// Include testnet chains in results
    #[serde(default)]
    pub testnets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistChainInput {
    /// Chain name or numeric chain ID (e.g., "gnosis", "fantom", "100")
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistAddInput {
    /// Chain name or numeric chain ID (e.g., "gnosis", "fantom", "100")
    pub chain: String,
    /// Maximum number of endpoints to add (default 5)
    #[serde(default = "default_chainlist_max")]
    pub max: usize,
}

fn default_chainlist_max() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistListInput {
    /// Include testnet chains in results
    #[serde(default)]
    pub testnets: bool,
}
