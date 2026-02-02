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

fn default_unit() -> String {
    "ether".to_string()
}

// --- Logs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogsInput {
    /// Contract address to query logs from
    pub contract: String,
    /// Event signature (e.g., "Transfer(address,address,uint256)")
    pub event: Option<String>,
    /// Starting block number or tag (e.g., "latest", "0x1234")
    pub from_block: Option<String>,
    /// Ending block number or tag
    pub to_block: Option<String>,
    /// Additional topic filters
    pub topics: Option<Vec<String>>,
    /// Chain name (ethereum, polygon, arbitrum, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Transaction ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TxAnalyzeInput {
    /// Transaction hash (with or without 0x prefix).
    /// Partial hashes are supported when block is specified.
    pub hash: String,
    /// Chain name (ethereum, polygon, arbitrum, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number for partial hash resolution.
    /// When provided with a partial tx hash (e.g. from account_txs output),
    /// the transaction will be looked up by matching the prefix within this block.
    #[serde(default)]
    pub block: Option<u64>,
    /// Include execution trace
    #[serde(default)]
    pub trace: bool,
}

// --- Account ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountAddressInput {
    /// Ethereum address (hex format)
    pub address: String,
    /// Chain name (ethereum, polygon, arbitrum, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountBalanceInput {
    /// Ethereum address(es) - supports multiple addresses
    pub addresses: Vec<String>,
    /// Chain name (ethereum, polygon, arbitrum, etc.)
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
    /// Save to file instead of returning in response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSourceInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Save to directory instead of returning in response
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
    pub tokens: Vec<String>,
    /// Holder address(es) to check balance for
    #[serde(default)]
    pub holders: Vec<String>,
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
    /// Target address for the transaction
    pub to: String,
    /// Value in wei (optional)
    pub value: Option<String>,
    /// Transaction data (optional)
    pub data: Option<String>,
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
pub struct CastAbiEncodeInput {
    /// Function signature
    pub sig: String,
    /// Arguments to encode
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastAbiDecodeInput {
    /// Function signature
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
    /// Source to query: all, alchemy, moralis, dsim, uniswap, yearn
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
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
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
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
    #[serde(default)]
    pub show_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteFromInput {
    /// The aggregator to use (open-ocean, kyber-swap, 0x, 1inch, cow-swap, li-fi, velora, enso)
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
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
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
    /// Search query
    pub query: String,
    /// Filter by impact (critical, high, medium, low)
    pub impact: Option<String>,
    /// Maximum results
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SoloditGetInput {
    /// Finding slug/ID
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Uniswap version (v2, v3)
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapTopPoolsInput {
    /// Number of pools to return
    pub limit: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
pub struct LlamaYieldsInput {
    /// Chain name (optional)
    pub chain: Option<String>,
}

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
    /// Amount
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Gas price in Gwei (default: 30)
    #[serde(default = "default_gas_price")]
    pub gas_price: String,
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
    /// Source chain name
    pub from_chain: String,
    /// Source token address
    pub from_token: String,
    /// Destination chain name
    pub to_chain: String,
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
    /// Output file path (returns JSON to stdout if not specified)
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
pub struct BlacklistPathInput {
    /// Source address
    pub from: String,
    /// Destination address
    pub to: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Contract Call ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractCallInput {
    /// Contract address
    pub address: String,
    /// Function signature
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
    /// Transaction hashes to bundle
    pub txs: Vec<String>,
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

// --- Alchemy Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesInput {
    /// Comma-separated token addresses
    pub tokens: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugInput {
    /// Transaction hash
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Protocol name (optional)
    pub protocol: Option<String>,
}

// --- Moralis ---
// Note: Moralis CLI doesn't support --chain flag - chain is handled internally
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressInput {
    /// Wallet or contract address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDomainInput {
    /// Domain to resolve (e.g., ENS, Unstoppable)
    pub domain: String,
}

// --- Dsim ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DsimAddressInput {
    /// Address to query
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DsimTokenInput {
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Dune ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueryInput {
    /// Query ID
    pub query_id: String,
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

// --- Curve Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurvePoolInput {
    /// Pool address or identifier
    pub pool: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CCXT Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtExchangeInput {
    /// Exchange name
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtOhlcvInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
    /// Timeframe (e.g., "1m", "1h", "1d")
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtTradesInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtCompareInput {
    /// Trading symbol
    pub symbol: String,
    /// Comma-separated exchange names (optional)
    pub exchanges: Option<String>,
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongVaultInput {
    /// Vault address
    pub vault: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongPricesInput {
    /// Comma-separated token addresses
    pub tokens: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Amount
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
    /// Route summary from routes command
    pub route_summary: String,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
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
pub struct CowswapOwnerInput {
    /// Owner address
    pub owner: String,
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
pub struct CowswapAuctionInput {
    /// Auction ID
    pub auction_id: String,
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
    /// Chain name
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiGasInput {
    /// Chain ID (e.g., "1" for Ethereum, "137" for Polygon)
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiConnectionsInput {
    /// Source chain
    pub from_chain: String,
    /// Destination chain
    pub to_chain: String,
}

// --- Velora ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraTokenInput {
    /// Token address or symbol
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraTxInput {
    /// Transaction hash
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraChainInput {
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoPriceInput {
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoBalancesInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CCXT ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtTickerInput {
    /// Exchange name (binance, coinbase, etc.)
    pub exchange: String,
    /// Trading symbol (BTC/USDT, ETH/USD, etc.)
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtOrderbookInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
    /// Depth limit
    pub limit: Option<u32>,
}

// --- Simulate ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    /// Enable precise gas estimation (Tenderly)
    #[serde(default)]
    pub estimate_gas: bool,
    /// Generate EIP-2930 access list in response (Tenderly)
    #[serde(default)]
    pub generate_access_list: bool,
    /// Provide access list (JSON format)
    pub access_list: Option<String>,
    /// Simulation type: full, quick, or abi
    pub simulation_type: Option<String>,
    /// Network ID to simulate on
    pub network_id: Option<u64>,
    /// Save simulation to Tenderly (returns simulation ID)
    #[serde(default)]
    pub save: bool,

    // === Tenderly options ===
    /// Tenderly API key (or use TENDERLY_ACCESS_KEY env)
    pub tenderly_key: Option<String>,
    /// Tenderly account slug (or use TENDERLY_ACCOUNT env)
    pub tenderly_account: Option<String>,
    /// Tenderly project slug (or use TENDERLY_PROJECT env)
    pub tenderly_project: Option<String>,

    // === Alchemy options ===
    /// Alchemy API key (or use ALCHEMY_API_KEY env)
    pub alchemy_key: Option<String>,
    /// Alchemy network (e.g., eth-mainnet, polygon-mainnet, arb-mainnet)
    pub alchemy_network: Option<String>,

    // === Debug options ===
    /// Dry run - output request without executing (json, curl, fetch, powershell, url, python, httpie, wget, go, rust, axios)
    pub dry_run: Option<String>,
    /// Show API keys in dry-run output (default: masked with env var placeholders)
    #[serde(default)]
    pub show_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateTxInput {
    /// Transaction hash to simulate
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Simulation backend (cast, tenderly, debug, trace, alchemy)
    pub via: Option<String>,
    /// Show full opcode trace
    #[serde(default)]
    pub trace: bool,
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Endpoints ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsUrlInput {
    /// RPC URL
    pub url: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}
