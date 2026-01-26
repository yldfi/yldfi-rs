//! Type definitions for Kong API responses

use serde::{Deserialize, Deserializer, Serialize};

/// Helper module for deserializing values that can be either strings or integers
mod string_or_int {
    use serde::{Deserialize, Deserializer};

    /// Deserialize a value that might be a string or an integer into a String
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrInt {
            String(String),
            Int(i64),
            UInt(u64),
            Float(f64),
        }

        Ok(Option::<StringOrInt>::deserialize(deserializer)?.map(|v| match v {
            StringOrInt::String(s) => s,
            StringOrInt::Int(i) => i.to_string(),
            StringOrInt::UInt(u) => u.to_string(),
            StringOrInt::Float(f) => f.to_string(),
        }))
    }
}

/// Helper to deserialize flexible numeric fields
fn deserialize_flexible_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    string_or_int::deserialize(deserializer)
}

/// A Yearn vault
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vault {
    /// Vault contract address
    pub address: String,
    /// Vault name
    pub name: Option<String>,
    /// Vault symbol
    pub symbol: Option<String>,
    /// Chain ID (1 = Ethereum, 137 = Polygon, etc.)
    pub chain_id: u64,
    /// API version (e.g., "3.0.4" for v3 vaults)
    pub api_version: Option<String>,
    /// Vault decimals (can be string or int from API)
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub decimals: Option<String>,
    /// Whether this is a v3 vault
    pub v3: Option<bool>,
    /// Whether this is a Yearn vault
    pub yearn: Option<bool>,
    /// Whether the vault is ERC4626 compliant
    pub erc4626: Option<bool>,
    /// Whether the vault is shutdown
    pub is_shutdown: Option<bool>,
    /// Emergency shutdown status
    pub emergency_shutdown: Option<bool>,
    /// Vault type (0 = default, 1 = automated, 2 = multi-strategy)
    pub vault_type: Option<i32>,
    /// Underlying asset token address
    pub token: Option<String>,
    /// Total assets in the vault (raw)
    #[serde(default)]
    pub total_assets: Option<String>,
    /// Total supply of vault shares (raw)
    #[serde(default)]
    pub total_supply: Option<String>,
    /// Price per share (raw)
    #[serde(default)]
    pub price_per_share: Option<String>,
    /// Deposit limit (raw)
    #[serde(default)]
    pub deposit_limit: Option<String>,
    /// Available deposit limit (raw)
    #[serde(default)]
    pub available_deposit_limit: Option<String>,
    /// Management fee (basis points)
    #[serde(default)]
    pub management_fee: Option<String>,
    /// Performance fee (basis points)
    #[serde(default)]
    pub performance_fee: Option<String>,
    /// Governance address
    pub governance: Option<String>,
    /// Guardian address
    pub guardian: Option<String>,
    /// Management address
    pub management: Option<String>,
    /// Rewards address
    pub rewards: Option<String>,
    /// Registry address
    pub registry: Option<String>,
    /// Inception timestamp (returned as string from API)
    #[serde(default)]
    pub incept_time: Option<String>,
    /// Inception block (returned as string from API)
    #[serde(default)]
    pub incept_block: Option<String>,
    /// Last report timestamp (returned as string from API)
    #[serde(default)]
    pub last_report: Option<String>,
    /// Activation timestamp (returned as string from API)
    #[serde(default)]
    pub activation: Option<String>,
    /// Project ID
    pub project_id: Option<String>,
    /// Project name
    pub project_name: Option<String>,
    /// Withdrawal queue
    #[serde(default)]
    pub withdrawal_queue: Option<Vec<String>>,
    /// Strategy addresses
    #[serde(default)]
    pub strategies: Option<Vec<String>>,
    /// TVL data
    pub tvl: Option<SparklinePoint>,
    /// APY data
    pub apy: Option<Apy>,
    /// Fee structure
    pub fees: Option<Fees>,
    /// Risk score
    pub risk: Option<RiskScore>,
    /// Vault metadata
    pub meta: Option<VaultMeta>,
    /// Underlying asset info
    pub asset: Option<Erc20>,
}

/// A Yearn strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Strategy {
    /// Strategy contract address
    pub address: String,
    /// Strategy name
    pub name: Option<String>,
    /// Chain ID
    pub chain_id: u64,
    /// API version
    pub api_version: Option<String>,
    /// Vault address this strategy belongs to
    pub vault: Option<String>,
    /// Whether this is a v3 strategy
    pub v3: Option<bool>,
    /// Activation timestamp
    #[serde(default)]
    pub activation: Option<u64>,
    /// Inception timestamp
    #[serde(default)]
    pub incept_time: Option<u64>,
    /// Inception block
    #[serde(default)]
    pub incept_block: Option<u64>,
    /// Last report timestamp
    #[serde(default)]
    pub last_report: Option<u64>,
    /// Total debt
    #[serde(default)]
    pub total_debt: Option<String>,
    /// Total gain
    #[serde(default)]
    pub total_gain: Option<String>,
    /// Total loss
    #[serde(default)]
    pub total_loss: Option<String>,
    /// Performance fee
    #[serde(default)]
    pub performance_fee: Option<String>,
    /// Debt ratio
    #[serde(default)]
    pub debt_ratio: Option<String>,
    /// Estimated total assets
    #[serde(default)]
    pub estimated_total_assets: Option<String>,
    /// Whether the strategy is active
    pub is_active: Option<bool>,
    /// Whether the strategy is shutdown
    pub is_shutdown: Option<bool>,
    /// Keeper address
    pub keeper: Option<String>,
    /// Strategist address
    pub strategist: Option<String>,
    /// Risk score
    pub risk: Option<RiskScore>,
    /// APY data
    pub apy: Option<Apy>,
    /// TVL data
    pub tvl: Option<SparklinePoint>,
}

/// APY (Annual Percentage Yield) data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Apy {
    /// Net APY (annualized)
    pub net: Option<f64>,
    /// Weekly net APY
    pub weekly_net: Option<f64>,
    /// Monthly net APY
    pub monthly_net: Option<f64>,
    /// Inception net APY
    pub inception_net: Option<f64>,
    /// Gross APR
    pub gross_apr: Option<f64>,
    /// Price per share at measurement
    #[serde(default)]
    pub price_per_share: Option<String>,
    /// Block number of measurement (returned as string from API)
    pub block_number: Option<String>,
    /// Block timestamp (returned as string from API)
    pub block_time: Option<String>,
}

/// TVL sparkline point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SparklinePoint {
    /// Value in USD
    pub close: Option<f64>,
    /// Block number (returned as string from API)
    pub block_number: Option<String>,
    /// Block timestamp (returned as string from API)
    pub block_time: Option<String>,
}

/// Fee structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fees {
    /// Management fee (annual, basis points)
    pub management_fee: Option<f64>,
    /// Performance fee (basis points)
    pub performance_fee: Option<f64>,
}

/// Risk score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RiskScore {
    /// Overall risk level (1-5, lower is safer)
    pub risk_level: Option<i32>,
}

/// Vault metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMeta {
    /// Display name
    pub display_name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Whether the vault is hidden
    pub is_hidden: Option<bool>,
    /// Whether the vault is boosted
    pub is_boosted: Option<bool>,
}

/// ERC20 token info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20 {
    /// Token address
    pub address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals (can be string or int from API)
    #[serde(default, deserialize_with = "deserialize_flexible_string")]
    pub decimals: Option<String>,
}

/// Token price info from Kong API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    /// Token address
    pub address: String,
    /// Chain ID
    pub chain_id: u64,
    /// Price in USD
    pub price_usd: f64,
    /// Price source (e.g., "defillama", "coingecko")
    pub price_source: String,
    /// Block number
    pub block_number: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// TVL data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tvl {
    /// Chain ID
    pub chain_id: u64,
    /// Address (vault or strategy)
    pub address: String,
    /// TVL value in USD
    pub value: f64,
    /// Price in USD at the time
    pub price_usd: Option<f64>,
    /// Price source
    pub price_source: String,
    /// Period (day, week, month)
    pub period: String,
    /// Block number
    pub block_number: u64,
    /// Timestamp
    pub time: Option<u64>,
}

/// Vault report (harvest event)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultReport {
    /// Chain ID
    pub chain_id: u64,
    /// Vault address
    pub address: String,
    /// Event name
    pub event_name: String,
    /// Strategy address
    pub strategy: String,
    /// Gain amount (raw)
    pub gain: String,
    /// Loss amount (raw)
    pub loss: String,
    /// Debt paid (raw)
    pub debt_paid: Option<String>,
    /// Total gain (raw)
    pub total_gain: Option<String>,
    /// Total loss (raw)
    pub total_loss: Option<String>,
    /// Total debt (raw)
    pub total_debt: Option<String>,
    /// Debt added (raw)
    pub debt_added: Option<String>,
    /// Debt ratio
    pub debt_ratio: Option<String>,
    /// Current debt (raw)
    pub current_debt: Option<String>,
    /// Protocol fees (raw)
    pub protocol_fees: Option<String>,
    /// Total fees (raw)
    pub total_fees: Option<String>,
    /// Total refunds (raw)
    pub total_refunds: Option<String>,
    /// Gain in USD
    pub gain_usd: Option<f64>,
    /// Loss in USD
    pub loss_usd: Option<f64>,
    /// Debt paid in USD
    pub debt_paid_usd: Option<f64>,
    /// Total gain in USD
    pub total_gain_usd: Option<f64>,
    /// Total loss in USD
    pub total_loss_usd: Option<f64>,
    /// Total debt in USD
    pub total_debt_usd: Option<f64>,
    /// Debt added in USD
    pub debt_added_usd: Option<f64>,
    /// Current debt in USD
    pub current_debt_usd: Option<f64>,
    /// Protocol fees in USD
    pub protocol_fees_usd: Option<f64>,
    /// Total fees in USD
    pub total_fees_usd: Option<f64>,
    /// Total refunds in USD
    pub total_refunds_usd: Option<f64>,
    /// Price in USD at report time
    pub price_usd: Option<f64>,
    /// Price source
    pub price_source: Option<String>,
    /// APR data
    pub apr: Option<ReportApr>,
    /// Block number
    pub block_number: u64,
    /// Block timestamp
    pub block_time: u64,
    /// Log index
    pub log_index: u64,
    /// Transaction hash
    pub transaction_hash: String,
}

/// Strategy report (harvest event)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StrategyReport {
    /// Chain ID
    pub chain_id: u64,
    /// Strategy address
    pub address: String,
    /// Event name
    pub event_name: String,
    /// Profit amount (raw)
    pub profit: String,
    /// Loss amount (raw)
    pub loss: String,
    /// Debt payment (raw)
    pub debt_payment: Option<String>,
    /// Debt outstanding (raw)
    pub debt_outstanding: Option<String>,
    /// Protocol fees (raw)
    pub protocol_fees: Option<String>,
    /// Performance fees (raw)
    pub performance_fees: Option<String>,
    /// APR data
    pub apr: Option<ReportApr>,
    /// Profit in USD
    pub profit_usd: Option<f64>,
    /// Loss in USD
    pub loss_usd: Option<f64>,
    /// Debt payment in USD
    pub debt_payment_usd: Option<f64>,
    /// Debt outstanding in USD
    pub debt_outstanding_usd: Option<f64>,
    /// Protocol fees in USD
    pub protocol_fees_usd: Option<f64>,
    /// Performance fees in USD
    pub performance_fees_usd: Option<f64>,
    /// Price in USD at report time
    pub price_usd: Option<f64>,
    /// Price source
    pub price_source: Option<String>,
    /// Block number
    pub block_number: u64,
    /// Block timestamp
    pub block_time: u64,
    /// Log index
    pub log_index: u64,
    /// Transaction hash
    pub transaction_hash: String,
}

/// APR data from a report
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportApr {
    /// Gross APR
    pub gross: Option<f64>,
    /// Net APR
    pub net: Option<f64>,
    /// Forward APR
    pub forward: Option<f64>,
}

/// TVL timeseries entry (legacy format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TvlEntry {
    /// Address (vault or strategy)
    pub address: Option<String>,
    /// Chain ID
    pub chain_id: Option<u64>,
    /// TVL in USD
    pub tvl: Option<f64>,
    /// Block number
    pub block_number: Option<u64>,
    /// Block timestamp
    pub block_time: Option<u64>,
    /// Period type (day, week, month)
    pub period: Option<String>,
}

/// Vault account (user position)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultAccount {
    /// Account address
    pub address: String,
    /// Vault address
    pub vault: String,
    /// Chain ID
    pub chain_id: u64,
    /// Share balance (raw)
    #[serde(default)]
    pub balance: Option<String>,
    /// Deposit amount (raw)
    #[serde(default)]
    pub deposits: Option<String>,
    /// Withdrawal amount (raw)
    #[serde(default)]
    pub withdrawals: Option<String>,
    /// Profit/loss (raw)
    #[serde(default)]
    pub profit: Option<String>,
}

/// GraphQL response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse<T> {
    /// Response data
    pub data: Option<T>,
    /// GraphQL errors
    pub errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    /// Error message
    pub message: String,
    /// Error locations
    pub locations: Option<Vec<GraphQLLocation>>,
    /// Error extensions
    pub extensions: Option<serde_json::Value>,
}

/// GraphQL error location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLLocation {
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}
