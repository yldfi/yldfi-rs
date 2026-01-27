//! Types for GoPlus Security API responses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// GoPlus API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response<T> {
    /// Response code (1 = success)
    pub code: i32,
    /// Response message
    pub message: String,
    /// Response data
    pub result: Option<T>,
}

impl<T> Response<T> {
    /// Check if response was successful
    pub fn is_success(&self) -> bool {
        self.code == 1
    }
}

/// Token security response - map of address to security info
pub type TokenSecurityResponse = HashMap<String, TokenSecurity>;

/// DEX trading information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct DexInfo {
    /// DEX name
    pub name: Option<String>,
    /// Liquidity in the pool
    pub liquidity: Option<String>,
    /// Trading pair
    pub pair: Option<String>,
}

/// Holder information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct HolderInfo {
    /// Holder address
    pub address: Option<String>,
    /// Holder tag/label
    pub tag: Option<String>,
    /// Whether this is a contract
    pub is_contract: Option<i32>,
    /// Balance held
    pub balance: Option<String>,
    /// Percentage of total supply
    pub percent: Option<String>,
    /// Whether address is locked
    pub is_locked: Option<i32>,
    /// Lock details if locked
    pub locked_detail: Option<Vec<LockedDetail>>,
}

/// Lock detail information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LockedDetail {
    /// Amount locked
    pub amount: Option<String>,
    /// End time of lock
    pub end_time: Option<String>,
    /// Opt out deadline
    pub opt_out_deadline: Option<String>,
}

/// Fake token detection info
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FakeToken {
    /// Whether it's a true token
    pub true_token_address: Option<String>,
    /// Value (1 = is fake)
    pub value: Option<i32>,
}

/// Token security information from GoPlus API
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct TokenSecurity {
    // === Basic Info ===
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Total supply
    pub total_supply: Option<String>,
    /// Number of holders
    pub holder_count: Option<String>,

    // === Contract Info ===
    /// Whether source code is open/verified
    #[serde(default)]
    pub is_open_source: Option<String>,
    /// Whether this is a proxy contract
    #[serde(default)]
    pub is_proxy: Option<String>,
    /// Whether contract can self-destruct
    #[serde(default)]
    pub selfdestruct: Option<String>,
    /// Whether contract makes external calls
    #[serde(default)]
    pub external_call: Option<String>,

    // === Ownership ===
    /// Owner address
    pub owner_address: Option<String>,
    /// Owner balance
    pub owner_balance: Option<String>,
    /// Owner percentage of supply
    pub owner_percent: Option<String>,
    /// Whether owner can change balances
    #[serde(default)]
    pub owner_change_balance: Option<String>,
    /// Creator address
    pub creator_address: Option<String>,
    /// Creator balance
    pub creator_balance: Option<String>,
    /// Creator percentage of supply
    pub creator_percent: Option<String>,
    /// Whether owner can take back ownership
    #[serde(default)]
    pub can_take_back_ownership: Option<String>,
    /// Whether contract has hidden owners
    #[serde(default)]
    pub hidden_owner: Option<String>,

    // === Trading Risks ===
    /// Whether token is a honeypot (cannot sell)
    #[serde(default)]
    pub is_honeypot: Option<String>,
    /// Buy tax percentage
    pub buy_tax: Option<String>,
    /// Sell tax percentage
    pub sell_tax: Option<String>,
    /// Whether token cannot be bought
    #[serde(default)]
    pub cannot_buy: Option<String>,
    /// Whether selling all tokens is restricted
    #[serde(default)]
    pub cannot_sell_all: Option<String>,
    /// Whether tax/slippage can be modified
    #[serde(default)]
    pub slippage_modifiable: Option<String>,
    /// Whether personal slippage can be set per address
    #[serde(default)]
    pub personal_slippage_modifiable: Option<String>,
    /// Whether trading has cooldown mechanism
    #[serde(default)]
    pub trading_cooldown: Option<String>,
    /// Whether transfer can be paused
    #[serde(default)]
    pub transfer_pausable: Option<String>,

    // === Token Controls ===
    /// Whether tokens can be minted
    #[serde(default)]
    pub is_mintable: Option<String>,
    /// Whether contract can blacklist addresses
    #[serde(default)]
    pub is_blacklisted: Option<String>,
    /// Whether contract has whitelist function
    #[serde(default)]
    pub is_whitelisted: Option<String>,
    /// Whether anti-whale mechanism exists
    #[serde(default)]
    pub is_anti_whale: Option<String>,
    /// Whether anti-whale can be modified
    #[serde(default)]
    pub anti_whale_modifiable: Option<String>,

    // === DEX & Liquidity ===
    /// Whether token is listed on DEX
    #[serde(default)]
    pub is_in_dex: Option<String>,
    /// DEX information
    #[serde(default)]
    pub dex: Option<Vec<DexInfo>>,
    /// LP holder count
    pub lp_holder_count: Option<String>,
    /// LP total supply
    pub lp_total_supply: Option<String>,
    /// LP holders info
    #[serde(default)]
    pub lp_holders: Option<Vec<HolderInfo>>,

    // === Holders ===
    /// Top holders info
    #[serde(default)]
    pub holders: Option<Vec<HolderInfo>>,

    // === Scam Detection ===
    /// Whether it's a fake/counterfeit token
    #[serde(default)]
    pub is_true_token: Option<String>,
    /// Fake token details
    #[serde(default)]
    pub fake_token: Option<FakeToken>,
    /// Whether it's an airdrop scam
    #[serde(default)]
    pub is_airdrop_scam: Option<String>,
    /// Count of honeypots by same creator
    pub honeypot_with_same_creator: Option<String>,
    /// Trust list status
    #[serde(default)]
    pub trust_list: Option<String>,
    /// Other potential risks description
    pub other_potential_risks: Option<String>,
    /// Additional notes
    pub note: Option<String>,
}

impl TokenSecurity {
    /// Parse a "0"/"1" string field as bool
    fn parse_bool(value: Option<&String>) -> bool {
        value.is_some_and(|v| v == "1")
    }

    /// Parse a string field as f64 percentage (0-100)
    fn parse_percent(value: Option<&String>) -> Option<f64> {
        value.and_then(|v| v.parse::<f64>().ok()).map(|v| v * 100.0)
    }

    /// Check if token is a honeypot
    pub fn is_honeypot(&self) -> bool {
        Self::parse_bool(self.is_honeypot.as_ref())
    }

    /// Check if contract is verified/open source
    pub fn is_verified(&self) -> bool {
        Self::parse_bool(self.is_open_source.as_ref())
    }

    /// Check if contract is a proxy
    pub fn is_proxy(&self) -> bool {
        Self::parse_bool(self.is_proxy.as_ref())
    }

    /// Check if token is mintable
    pub fn is_mintable(&self) -> bool {
        Self::parse_bool(self.is_mintable.as_ref())
    }

    /// Check if transfers can be paused
    pub fn is_transfer_pausable(&self) -> bool {
        Self::parse_bool(self.transfer_pausable.as_ref())
    }

    /// Check if owner can blacklist addresses
    pub fn can_blacklist(&self) -> bool {
        Self::parse_bool(self.is_blacklisted.as_ref())
    }

    /// Check if there's a hidden owner
    pub fn has_hidden_owner(&self) -> bool {
        Self::parse_bool(self.hidden_owner.as_ref())
    }

    /// Check if anti-whale mechanism exists
    pub fn has_anti_whale(&self) -> bool {
        Self::parse_bool(self.is_anti_whale.as_ref())
    }

    /// Check if owner can change balances
    pub fn owner_can_change_balance(&self) -> bool {
        Self::parse_bool(self.owner_change_balance.as_ref())
    }

    /// Check if it's an airdrop scam
    pub fn is_airdrop_scam(&self) -> bool {
        Self::parse_bool(self.is_airdrop_scam.as_ref())
    }

    /// Get buy tax as percentage (0-100)
    pub fn buy_tax_percent(&self) -> Option<f64> {
        Self::parse_percent(self.buy_tax.as_ref())
    }

    /// Get sell tax as percentage (0-100)
    pub fn sell_tax_percent(&self) -> Option<f64> {
        Self::parse_percent(self.sell_tax.as_ref())
    }

    /// Check if sell tax is high (> 10%)
    pub fn has_high_sell_tax(&self) -> bool {
        self.sell_tax_percent().is_some_and(|t| t > 10.0)
    }

    /// Check if owner is renounced (address is zero)
    pub fn is_owner_renounced(&self) -> bool {
        self.owner_address
            .as_ref()
            .is_some_and(|addr| addr == "0x0000000000000000000000000000000000000000" || addr.is_empty())
    }

    /// Check if token has any major red flags
    pub fn has_major_risks(&self) -> bool {
        self.is_honeypot()
            || self.has_high_sell_tax()
            || self.is_airdrop_scam()
            || Self::parse_bool(self.cannot_buy.as_ref())
            || Self::parse_bool(self.cannot_sell_all.as_ref())
    }

    /// Get list of detected issues
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();

        if self.is_honeypot() {
            issues.push("Cannot sell (honeypot)".to_string());
        }
        if self.has_high_sell_tax() {
            issues.push(format!(
                "High sell tax ({:.0}%)",
                self.sell_tax_percent().unwrap_or(0.0)
            ));
        }
        if self.is_airdrop_scam() {
            issues.push("Airdrop scam".to_string());
        }
        if Self::parse_bool(self.cannot_buy.as_ref()) {
            issues.push("Cannot buy".to_string());
        }
        if Self::parse_bool(self.cannot_sell_all.as_ref()) {
            issues.push("Cannot sell all".to_string());
        }
        if self.is_mintable() {
            issues.push("Mintable".to_string());
        }
        if self.is_transfer_pausable() {
            issues.push("Transfers pausable".to_string());
        }
        if self.can_blacklist() {
            issues.push("Can blacklist addresses".to_string());
        }
        if self.has_hidden_owner() {
            issues.push("Hidden owner".to_string());
        }
        if self.owner_can_change_balance() {
            issues.push("Owner can change balances".to_string());
        }

        issues
    }
}

/// Supported chains for token security API
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chain {
    Ethereum = 1,
    BscMainnet = 56,
    Polygon = 137,
    Arbitrum = 42161,
    Base = 8453,
    Avalanche = 43114,
    Optimism = 10,
    Fantom = 250,
    Cronos = 25,
    Gnosis = 100,
    Heco = 128,
    Linea = 59144,
    Scroll = 534352,
    Mantle = 5000,
    ZkSyncEra = 324,
    Blast = 81457,
}

impl Chain {
    /// Get chain ID
    pub fn id(&self) -> u64 {
        *self as u64
    }

    /// Try to create from chain ID
    pub fn from_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Ethereum),
            56 => Some(Self::BscMainnet),
            137 => Some(Self::Polygon),
            42161 => Some(Self::Arbitrum),
            8453 => Some(Self::Base),
            43114 => Some(Self::Avalanche),
            10 => Some(Self::Optimism),
            250 => Some(Self::Fantom),
            25 => Some(Self::Cronos),
            100 => Some(Self::Gnosis),
            128 => Some(Self::Heco),
            59144 => Some(Self::Linea),
            534352 => Some(Self::Scroll),
            5000 => Some(Self::Mantle),
            324 => Some(Self::ZkSyncEra),
            81457 => Some(Self::Blast),
            _ => None,
        }
    }

    /// Check if chain ID is supported
    pub fn is_supported(id: u64) -> bool {
        Self::from_id(id).is_some()
    }
}

// ==================== Address Security ====================

/// Address security response - map of address to security info
pub type AddressSecurityResponse = HashMap<String, AddressSecurity>;

/// Address security information (malicious address detection)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AddressSecurity {
    /// Whether the address is a contract
    #[serde(default)]
    pub is_contract: Option<String>,
    /// Cybercrime involvement
    #[serde(default)]
    pub cybercrime: Option<String>,
    /// Money laundering involvement
    #[serde(default)]
    pub money_laundering: Option<String>,
    /// Number of malicious contracts created
    #[serde(default)]
    pub number_of_malicious_contracts_created: Option<String>,
    /// Financial crime involvement
    #[serde(default)]
    pub financial_crime: Option<String>,
    /// Darkweb transactions
    #[serde(default)]
    pub darkweb_transactions: Option<String>,
    /// Phishing activities
    #[serde(default)]
    pub phishing_activities: Option<String>,
    /// Fake KYC
    #[serde(default)]
    pub fake_kyc: Option<String>,
    /// Blacklist doubt
    #[serde(default)]
    pub blacklist_doubt: Option<String>,
    /// Stealing attack
    #[serde(default)]
    pub stealing_attack: Option<String>,
    /// Blackmail activities
    #[serde(default)]
    pub blackmail_activities: Option<String>,
    /// Sanctioned status
    #[serde(default)]
    pub sanctioned: Option<String>,
    /// Malicious mining activities
    #[serde(default)]
    pub malicious_mining_activities: Option<String>,
    /// Mixer usage
    #[serde(default)]
    pub mixer: Option<String>,
    /// Honeypot related address
    #[serde(default)]
    pub honeypot_related_address: Option<String>,
    /// Data source
    pub data_source: Option<String>,
}

impl AddressSecurity {
    fn parse_bool(value: Option<&String>) -> bool {
        value.is_some_and(|v| v == "1")
    }

    /// Check if address is flagged as malicious
    pub fn is_malicious(&self) -> bool {
        Self::parse_bool(self.cybercrime.as_ref())
            || Self::parse_bool(self.money_laundering.as_ref())
            || Self::parse_bool(self.financial_crime.as_ref())
            || Self::parse_bool(self.phishing_activities.as_ref())
            || Self::parse_bool(self.stealing_attack.as_ref())
            || Self::parse_bool(self.sanctioned.as_ref())
            || Self::parse_bool(self.honeypot_related_address.as_ref())
    }

    /// Get list of detected issues
    pub fn get_issues(&self) -> Vec<String> {
        let mut issues = Vec::new();
        if Self::parse_bool(self.cybercrime.as_ref()) {
            issues.push("Cybercrime".to_string());
        }
        if Self::parse_bool(self.money_laundering.as_ref()) {
            issues.push("Money laundering".to_string());
        }
        if Self::parse_bool(self.financial_crime.as_ref()) {
            issues.push("Financial crime".to_string());
        }
        if Self::parse_bool(self.phishing_activities.as_ref()) {
            issues.push("Phishing".to_string());
        }
        if Self::parse_bool(self.stealing_attack.as_ref()) {
            issues.push("Stealing attack".to_string());
        }
        if Self::parse_bool(self.sanctioned.as_ref()) {
            issues.push("Sanctioned".to_string());
        }
        if Self::parse_bool(self.honeypot_related_address.as_ref()) {
            issues.push("Honeypot related".to_string());
        }
        if Self::parse_bool(self.mixer.as_ref()) {
            issues.push("Mixer usage".to_string());
        }
        if Self::parse_bool(self.darkweb_transactions.as_ref()) {
            issues.push("Darkweb transactions".to_string());
        }
        issues
    }
}

// ==================== NFT Security ====================

/// NFT security response - map of address to security info
pub type NftSecurityResponse = HashMap<String, NftSecurity>;

/// NFT privilege info (burn, mint, self destruct, transfer without approval)
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NftPrivilegeInfo {
    /// Owner address
    pub owner_address: Option<String>,
    /// Value (-1 = no owner, 0 = no privilege, 1 = has privilege)
    pub value: Option<i32>,
    /// Owner type (blackhole, contract, eoa, etc.)
    pub owner_type: Option<String>,
}

/// NFT collection security information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NftSecurity {
    /// NFT contract address
    pub nft_address: Option<String>,
    /// NFT name
    pub nft_name: Option<String>,
    /// NFT symbol
    pub nft_symbol: Option<String>,
    /// NFT description
    pub nft_description: Option<String>,
    /// NFT ERC standard (erc721, erc1155)
    pub nft_erc: Option<String>,
    /// Number of items in collection
    pub nft_items: Option<u64>,
    /// Number of owners
    pub nft_owner_number: Option<u64>,
    /// Website URL
    pub website_url: Option<String>,
    /// Discord URL
    pub discord_url: Option<String>,
    /// Twitter URL
    pub twitter_url: Option<String>,
    /// Medium URL
    pub medium_url: Option<String>,
    /// GitHub URL
    pub github_url: Option<String>,
    /// Telegram URL
    pub telegram_url: Option<String>,
    /// Creator address
    pub creator_address: Option<String>,
    /// Create block number
    pub create_block_number: Option<u64>,
    /// Whether it's verified (0 = no, 1 = yes)
    #[serde(default)]
    pub nft_verified: Option<i32>,
    /// Whether it's open source (0 = no, 1 = yes)
    #[serde(default)]
    pub nft_open_source: Option<i32>,
    /// Whether it's a proxy (0 = no, 1 = yes)
    #[serde(default)]
    pub nft_proxy: Option<i32>,
    /// Whether minting is restricted (0 = no, 1 = yes)
    #[serde(default)]
    pub restricted_approval: Option<i32>,
    /// Trust list (0 = no, 1 = yes)
    #[serde(default)]
    pub trust_list: Option<i32>,
    /// Malicious NFT contract (0 = no, 1 = yes)
    #[serde(default)]
    pub malicious_nft_contract: Option<i32>,
    /// Privileged burn info
    #[serde(default)]
    pub privileged_burn: Option<NftPrivilegeInfo>,
    /// Privileged minting info
    #[serde(default)]
    pub privileged_minting: Option<NftPrivilegeInfo>,
    /// Self destruct info
    #[serde(default)]
    pub self_destruct: Option<NftPrivilegeInfo>,
    /// Transfer without approval info
    #[serde(default)]
    pub transfer_without_approval: Option<NftPrivilegeInfo>,
    /// Red check mark
    pub red_check_mark: Option<i32>,
    /// Metadata frozen
    pub metadata_frozen: Option<i32>,
    /// Oversupply minting
    pub oversupply_minting: Option<i32>,
    /// Trading volume 24h
    pub traded_volume_24h: Option<f64>,
    /// Total volume
    pub total_volume: Option<f64>,
    /// Highest price
    pub highest_price: Option<f64>,
    /// Lowest price 24h
    pub lowest_price_24h: Option<f64>,
    /// Average price 24h
    pub average_price_24h: Option<f64>,
    /// Sales 24h
    pub sales_24h: Option<u64>,
}

impl NftSecurity {
    /// Check if NFT is verified
    pub fn is_verified(&self) -> bool {
        self.nft_verified == Some(1)
    }

    /// Check if NFT contract is open source
    pub fn is_open_source(&self) -> bool {
        self.nft_open_source == Some(1)
    }

    /// Check if NFT is a malicious contract
    pub fn is_malicious(&self) -> bool {
        self.malicious_nft_contract == Some(1)
    }

    /// Check if NFT is a honeypot (uses malicious_nft_contract flag)
    pub fn is_honeypot(&self) -> bool {
        self.is_malicious()
    }

    /// Check if has privileged burn capability
    pub fn has_privileged_burn(&self) -> bool {
        self.privileged_burn
            .as_ref()
            .is_some_and(|p| p.value == Some(1))
    }

    /// Check if has privileged minting capability
    pub fn has_privileged_minting(&self) -> bool {
        self.privileged_minting
            .as_ref()
            .is_some_and(|p| p.value == Some(1))
    }

    /// Check if has self destruct capability
    pub fn has_self_destruct(&self) -> bool {
        self.self_destruct
            .as_ref()
            .is_some_and(|p| p.value == Some(1))
    }

    /// Check if NFT has risks
    pub fn has_risks(&self) -> bool {
        self.is_malicious()
            || self.has_privileged_burn()
            || self.has_self_destruct()
    }
}

// ==================== Approval Security ====================

/// Approval security response - map of address to security info
pub type ApprovalSecurityResponse = HashMap<String, ApprovalSecurity>;

/// Contract scan results
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ContractScan {
    /// Owner info
    #[serde(default)]
    pub owner: Option<serde_json::Value>,
    /// Privilege withdraw (-1 = unknown, 0 = no, 1 = yes)
    pub privilege_withdraw: Option<i32>,
    /// Withdraw missing (-1 = unknown, 0 = no, 1 = yes)
    pub withdraw_missing: Option<i32>,
    /// Blacklist (-1 = unknown, 0 = no, 1 = yes)
    pub blacklist: Option<i32>,
    /// Self destruct (-1 = unknown, 0 = no, 1 = yes)
    pub selfdestruct: Option<i32>,
    /// Approval abuse (-1 = unknown, 0 = no, 1 = yes)
    pub approval_abuse: Option<i32>,
}

/// Risky approval info
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RiskyApproval {
    /// Risk description
    pub risk: Option<String>,
    /// Risk value (0 = no risk, 1 = has risk)
    pub value: Option<i32>,
}

/// Approval security information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ApprovalSecurity {
    /// Contract name
    pub contract_name: Option<String>,
    /// Whether it's a contract (0 = no, 1 = yes)
    #[serde(default)]
    pub is_contract: Option<i32>,
    /// Contract creator
    pub creator_address: Option<String>,
    /// Deploy timestamp
    pub deployed_time: Option<u64>,
    /// Whether it's open source (0 = no, 1 = yes)
    #[serde(default)]
    pub is_open_source: Option<i32>,
    /// Whether it's a proxy (0 = no, 1 = yes)
    #[serde(default)]
    pub is_proxy: Option<i32>,
    /// Trust list status (0 = no, 1 = yes)
    #[serde(default)]
    pub trust_list: Option<i32>,
    /// Doubt list status (0 = no, 1 = yes)
    #[serde(default)]
    pub doubt_list: Option<i32>,
    /// Malicious behavior
    pub malicious_behavior: Option<Vec<String>>,
    /// Tag (e.g., "Fake_Phishing")
    pub tag: Option<String>,
    /// Contract scan results
    pub contract_scan: Option<ContractScan>,
    /// Risky approval info
    pub risky_approval: Option<RiskyApproval>,
}

impl ApprovalSecurity {
    /// Check if contract is malicious
    pub fn is_malicious(&self) -> bool {
        self.doubt_list == Some(1)
            || self
                .malicious_behavior
                .as_ref()
                .is_some_and(|b| !b.is_empty())
            || self.risky_approval.as_ref().is_some_and(|r| r.value == Some(1))
    }

    /// Check if contract is trusted
    pub fn is_trusted(&self) -> bool {
        self.trust_list == Some(1)
    }

    /// Check if contract is on doubt list
    pub fn is_doubtful(&self) -> bool {
        self.doubt_list == Some(1)
    }
}
