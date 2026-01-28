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
    /// Transaction hash (with or without 0x prefix)
    pub hash: String,
    /// Chain name (ethereum, polygon, arbitrum, etc.)
    #[serde(default = "default_chain")]
    pub chain: String,
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
pub struct AccountTxsInput {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Maximum number of transactions to return
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
    /// Token contract address
    pub token: String,
    /// Wallet address to check balance for
    pub address: String,
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
    /// Wallet address
    pub address: String,
    /// Chain name (optional, all chains if omitted)
    pub chain: Option<String>,
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
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Amount to swap (in source token units)
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Slippage tolerance in basis points (optional)
    pub slippage: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteCompareInput {
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Amount to swap
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Transfer category (external, internal, erc20, erc721, erc1155)
    pub category: Option<String>,
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
    /// Protocol name (optional)
    pub protocol: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressNameInput {
    /// Address name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressFileInput {
    /// File path for import/export
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressSearchInput {
    /// Search query
    pub query: String,
}

// --- Blacklist ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistAddInput {
    /// Address to blacklist
    pub address: String,
    /// Reason for blacklisting (optional)
    pub reason: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressInput {
    /// Wallet or contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
    /// Contract address
    pub contract: String,
    /// Function signature
    pub sig: String,
    /// Function arguments
    pub args: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Simulation provider (tenderly, alchemy)
    pub via: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateTxInput {
    /// Transaction hash to simulate
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
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
pub struct EndpointsOptimizeInput {
    /// Optional specific RPC URL to optimize (if omitted, optimizes all)
    pub url: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}
