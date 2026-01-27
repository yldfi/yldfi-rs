//! Token blacklist management commands
//!
//! Allows users to add, remove, and list blacklisted tokens.
//! Includes automated scam detection based on contract verification status.

use crate::abi::AbiFetcher;
use crate::config::{Chain, TokenBlacklist};
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct BlacklistArgs {
    #[command(subcommand)]
    pub command: BlacklistCommands,
}

#[derive(Subcommand)]
pub enum BlacklistCommands {
    /// Add a token to the blacklist
    Add {
        /// Token contract address
        address: String,

        /// Token symbol (for display)
        #[arg(long, short)]
        symbol: Option<String>,

        /// Reason for blacklisting
        #[arg(long, short)]
        reason: Option<String>,

        /// Chain where this token exists
        #[arg(long, short)]
        chain: Option<String>,
    },

    /// Remove a token from the blacklist
    Remove {
        /// Token contract address
        address: String,
    },

    /// List all blacklisted tokens (with Etherscan links)
    List {
        /// Show full Etherscan URLs
        #[arg(long, short = 'l')]
        links: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check if a token is blacklisted
    Check {
        /// Token contract address
        address: String,
    },

    /// Scan a token for scam indicators (unverified, suspicious name)
    Scan {
        /// Token contract address
        address: String,

        /// Chain to check on
        #[arg(long, short, default_value = "ethereum")]
        chain: String,

        /// Automatically add to blacklist if detected as scam
        #[arg(long, short = 'a')]
        auto_blacklist: bool,
    },

    /// Scan all tokens from portfolio for scams
    ScanPortfolio {
        /// Wallet address to scan
        address: String,

        /// Chain to check on
        #[arg(long, short, default_value = "ethereum")]
        chain: String,

        /// Automatically blacklist detected scam tokens
        #[arg(long, short = 'a')]
        auto_blacklist: bool,

        /// Only show unverified/suspicious tokens
        #[arg(long)]
        suspicious_only: bool,
    },

    /// Show the blacklist file path
    Path,
}

/// Security check result from honeypot.is and GoPlus APIs
#[derive(Debug, Clone, Default)]
pub struct SecurityResult {
    // Honeypot detection
    pub is_honeypot: bool,
    pub buy_tax: Option<f64>,
    pub sell_tax: Option<f64>,

    // Contract controls
    pub transfer_pausable: bool,
    pub can_blacklist: bool,
    pub hidden_owner: bool,
    pub anti_whale: bool,

    // Ownership status
    pub owner_address: Option<String>,
    pub is_owner_renounced: bool,

    // Liquidity info
    pub is_open_source: bool,
    pub is_proxy: bool,
    pub is_mintable: bool,

    // Risk issues
    pub issues: Vec<String>,
}

impl SecurityResult {
    /// Check if high sell tax (>10%)
    pub fn has_high_sell_tax(&self) -> bool {
        self.sell_tax.is_some_and(|t| t > 10.0)
    }
}

/// Result of scanning a token for scam indicators
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub address: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub is_verified: bool,
    pub is_proxy: bool,
    pub contract_name: Option<String>,
    pub implementation: Option<String>,
    pub security: Option<SecurityResult>,
    pub warnings: Vec<String>,
}

impl ScanResult {
    /// Check if this token is likely a scam
    pub fn is_likely_scam(&self) -> bool {
        // Primary: unverified contract
        if !self.is_verified {
            return true;
        }
        // Secondary: honeypot detected or high sell tax
        if let Some(sec) = &self.security {
            if sec.is_honeypot || sec.has_high_sell_tax() {
                return true;
            }
        }
        false
    }

    /// Get a reason string for why this is flagged
    pub fn reason(&self) -> String {
        let mut reasons = Vec::new();

        if !self.is_verified {
            if self.is_proxy {
                reasons.push("Unverified proxy".to_string());
            } else {
                reasons.push("Unverified contract".to_string());
            }
        }

        if let Some(sec) = &self.security {
            if sec.is_honeypot {
                reasons.push("Honeypot".to_string());
            }
            if sec.has_high_sell_tax() {
                reasons.push(format!("High sell tax ({:.0}%)", sec.sell_tax.unwrap_or(0.0)));
            }
            reasons.extend(sec.issues.clone());
        }

        reasons.extend(self.warnings.clone());

        if reasons.is_empty() {
            "Unknown".to_string()
        } else {
            reasons.join(", ")
        }
    }

    /// Get detailed security info for display
    pub fn security_details(&self) -> Option<String> {
        self.security.as_ref().map(|sec| {
            let mut details = Vec::new();
            if let Some(buy) = sec.buy_tax {
                if buy > 0.0 {
                    details.push(format!("Buy tax: {:.1}%", buy));
                }
            }
            if let Some(sell) = sec.sell_tax {
                if sell > 0.0 {
                    details.push(format!("Sell tax: {:.1}%", sell));
                }
            }
            if sec.transfer_pausable {
                details.push("Pausable".to_string());
            }
            if sec.can_blacklist {
                details.push("Can blacklist".to_string());
            }
            if sec.hidden_owner {
                details.push("Hidden owner".to_string());
            }
            if sec.is_mintable {
                details.push("Mintable".to_string());
            }
            if !sec.is_owner_renounced {
                if let Some(owner) = &sec.owner_address {
                    if owner != "0x0000000000000000000000000000000000000000" {
                        details.push("Owner not renounced".to_string());
                    }
                }
            }
            details.join(", ")
        })
    }
}

/// Known legitimate protocol contract name patterns (case-insensitive)
/// These are whitelisted even if they have unusual ownership patterns
const SAFE_PROTOCOL_PATTERNS: &[&str] = &[
    // Yearn ecosystem
    "yearn", "yvault", "ystrategy", "yvtoken", "v3 vault",
    // Curve ecosystem
    "curve", "crv", "vyper_contract", "gauge", "minter",
    // Convex ecosystem
    "convex", "cvx", "basepool", "booster", "rewardpool",
    // Lido ecosystem
    "lido", "steth", "wsteth",
    // Compound ecosystem
    "compound", "ctoken", "cerc20", "comptroller",
    // Aave ecosystem
    "aave", "atoken", "debttoken", "pool",
    // Other major DeFi
    "uniswap", "sushiswap", "balancer", "makerdao", "dsr", "dai",
    "origin", "oeth", "ousd",
    "frax", "sfrax", "frxeth", "sfrxeth",
    "prisma", "mkusd",
    "eigen", "eigenlayer",
    "rocket", "reth",
    "coinbase", "cbeth",
];

/// Check if a contract name suggests it's a known safe protocol
fn is_known_safe_protocol(contract_name: Option<&str>, symbol: Option<&str>) -> bool {
    let check_text = |text: &str| -> bool {
        let lower = text.to_lowercase();
        SAFE_PROTOCOL_PATTERNS.iter().any(|p| lower.contains(p))
    };

    if let Some(name) = contract_name {
        if check_text(name) {
            return true;
        }
    }
    if let Some(sym) = symbol {
        if check_text(sym) {
            return true;
        }
    }
    false
}

/// Check security using GoPlus API via the gplus crate
async fn check_security(chain_id: u64, address: &str) -> Option<SecurityResult> {
    // Only supported chains
    if !gplus::is_chain_supported(chain_id) {
        return None;
    }

    let client = gplus::Client::new().ok()?;
    let token = client.token_security(chain_id, address).await.ok()?;

    Some(SecurityResult {
        is_honeypot: token.is_honeypot(),
        buy_tax: token.buy_tax_percent(),
        sell_tax: token.sell_tax_percent(),
        transfer_pausable: token.is_transfer_pausable(),
        can_blacklist: token.can_blacklist(),
        hidden_owner: token.has_hidden_owner(),
        anti_whale: token.has_anti_whale(),
        owner_address: token.owner_address.clone(),
        is_owner_renounced: token.is_owner_renounced(),
        is_open_source: token.is_verified(),
        is_proxy: token.is_proxy(),
        is_mintable: token.is_mintable(),
        issues: token.get_issues(),
    })
}

/// Scan a token for scam indicators using Etherscan verification and honeypot detection
pub async fn scan_token(
    chain: Chain,
    address: &str,
    api_key: Option<String>,
) -> anyhow::Result<ScanResult> {
    let fetcher = AbiFetcher::new(api_key)?;

    // Get contract metadata (verification status)
    let metadata = fetcher.get_contract_metadata(chain, address).await?;

    // Get token metadata (name, symbol) via RPC
    let token_meta = fetcher.get_token_metadata_rpc(chain, address).await.ok();

    let name = token_meta.as_ref().and_then(|t| t.name.clone()).unwrap_or_default();
    let symbol = token_meta.as_ref().and_then(|t| t.symbol.clone()).unwrap_or_default();

    let mut warnings = Vec::new();

    // Check if this is a known safe protocol (whitelist)
    let is_known_safe = is_known_safe_protocol(metadata.name.as_deref(), Some(&symbol));

    // Check for proxy without visible implementation (only warn if not known safe)
    if !is_known_safe && metadata.is_proxy && metadata.implementation.is_none() {
        warnings.push("Proxy without visible implementation".to_string());
    }

    // If it's a proxy, check if implementation is verified
    if let Some(impl_addr) = &metadata.implementation {
        let impl_meta = fetcher.get_contract_metadata(chain, impl_addr).await.ok();
        if let Some(im) = impl_meta {
            if !im.is_verified && !is_known_safe {
                warnings.push("Proxy implementation not verified".to_string());
            }
        }
    }

    // Check security status using GoPlus API
    let chain_id = chain.chain_id();
    let security = check_security(chain_id, address).await;

    // For known safe protocols, override is_verified if they're recognized DeFi
    let effective_verified = if is_known_safe && !metadata.is_verified {
        // Known protocols might use proxy patterns that appear unverified
        // but the symbol/name indicates legitimacy
        true
    } else {
        metadata.is_verified
    };

    Ok(ScanResult {
        address: address.to_string(),
        symbol: if symbol.is_empty() { None } else { Some(symbol) },
        name: if name.is_empty() { None } else { Some(name) },
        is_verified: effective_verified,
        is_proxy: metadata.is_proxy,
        contract_name: metadata.name,
        implementation: metadata.implementation,
        security,
        warnings,
    })
}

/// Get Etherscan URL for a token
fn etherscan_url(chain: &str, address: &str) -> String {
    let base = match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => "https://etherscan.io",
        "polygon" | "matic" => "https://polygonscan.com",
        "arbitrum" | "arb" => "https://arbiscan.io",
        "optimism" | "op" => "https://optimistic.etherscan.io",
        "base" => "https://basescan.org",
        "bsc" | "bnb" => "https://bscscan.com",
        "avalanche" | "avax" => "https://snowtrace.io",
        "fantom" | "ftm" => "https://ftmscan.com",
        "gnosis" | "xdai" => "https://gnosisscan.io",
        _ => "https://etherscan.io",
    };
    format!("{}/token/{}", base, address)
}

/// Execute blacklist command
pub async fn execute(args: &BlacklistArgs) -> anyhow::Result<()> {
    match &args.command {
        BlacklistCommands::Add {
            address,
            symbol,
            reason,
            chain,
        } => {
            let mut blacklist = TokenBlacklist::load_default();
            blacklist
                .add(address, symbol.clone(), reason.clone(), chain.clone())
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            let display = symbol.as_deref().unwrap_or(address);
            println!("Added {} to blacklist", display);

            if let Some(r) = reason {
                println!("  Reason: {}", r);
            }
        }

        BlacklistCommands::Remove { address } => {
            let mut blacklist = TokenBlacklist::load_default();

            // Get symbol before removing for display
            let symbol = blacklist
                .get(address)
                .and_then(|e| e.symbol.clone())
                .unwrap_or_else(|| address.to_string());

            if blacklist.remove(address).map_err(|e| anyhow::anyhow!("{}", e))? {
                println!("Removed {} from blacklist", symbol);
            } else {
                println!("Token {} was not in blacklist", address);
            }
        }

        BlacklistCommands::List { links, json } => {
            let blacklist = TokenBlacklist::load_default();

            if blacklist.is_empty() {
                if *json {
                    println!("[]");
                } else {
                    println!("No tokens in blacklist");
                    println!();
                    println!("Add tokens with: ethcli blacklist add <address> --symbol <SYM>");
                }
                return Ok(());
            }

            if *json {
                // Output as JSON array
                let entries: Vec<_> = blacklist.list().iter().map(|e| {
                    serde_json::json!({
                        "address": e.address,
                        "symbol": e.symbol,
                        "reason": e.reason,
                        "chain": e.chain,
                        "etherscan_url": etherscan_url(e.chain.as_deref().unwrap_or("ethereum"), &e.address)
                    })
                }).collect();
                println!("{}", serde_json::to_string_pretty(&entries)?);
                return Ok(());
            }

            println!("Blacklisted tokens ({}):", blacklist.len());
            println!("{}", "-".repeat(if *links { 120 } else { 80 }));

            if *links {
                println!(
                    "{:<12} {:<44} {:<40} Etherscan",
                    "Symbol", "Address", "Reason"
                );
            } else {
                println!(
                    "{:<12} {:<44} Reason",
                    "Symbol", "Address"
                );
            }
            println!("{}", "-".repeat(if *links { 120 } else { 80 }));

            for entry in blacklist.list() {
                let symbol = entry.symbol.as_deref().unwrap_or("-");
                let reason = entry.reason.as_deref().unwrap_or("");
                let chain = entry.chain.as_deref().unwrap_or("ethereum");
                let chain_suffix = format!(" ({})", chain);

                // Truncate reason for display
                let reason_display = if reason.len() > 30 {
                    format!("{}...", &reason[..27])
                } else {
                    reason.to_string()
                };

                if *links {
                    let url = etherscan_url(chain, &entry.address);
                    println!(
                        "{:<12} {:<44} {:<40} {}",
                        symbol, entry.address, format!("{}{}", reason_display, chain_suffix), url
                    );
                } else {
                    println!(
                        "{:<12} {:<44} {}{}",
                        symbol, entry.address, reason_display, chain_suffix
                    );
                }
            }
            println!("{}", "-".repeat(if *links { 120 } else { 80 }));
        }

        BlacklistCommands::Check { address } => {
            let blacklist = TokenBlacklist::load_default();

            if let Some(entry) = blacklist.get(address) {
                let symbol = entry.symbol.as_deref().unwrap_or("Unknown");
                let chain = entry.chain.as_deref().unwrap_or("ethereum");
                println!("Token {} ({}) is BLACKLISTED", symbol, address);
                if let Some(reason) = &entry.reason {
                    println!("  Reason: {}", reason);
                }
                println!("  Etherscan: {}", etherscan_url(chain, address));
            } else {
                println!("Token {} is NOT blacklisted", address);
            }
        }

        BlacklistCommands::Scan { address, chain, auto_blacklist } => {
            execute_scan(address, chain, *auto_blacklist).await?;
        }

        BlacklistCommands::ScanPortfolio { address, chain, auto_blacklist, suspicious_only } => {
            execute_scan_portfolio(address, chain, *auto_blacklist, *suspicious_only).await?;
        }

        BlacklistCommands::Path => {
            println!("{}", TokenBlacklist::default_path().display());
        }
    }

    Ok(())
}

/// Print token analysis section
fn print_token_analysis(result: &ScanResult, chain_str: &str) {
    println!();
    println!("Token Analysis:");
    println!("  Address:    {}", result.address);
    if let Some(sym) = &result.symbol {
        println!("  Symbol:     {}", sym);
    }
    if let Some(name) = &result.name {
        println!("  Name:       {}", name);
    }
    if let Some(contract) = &result.contract_name {
        println!("  Contract:   {}", contract);
    }
    println!(
        "  Verified:   {}",
        if result.is_verified { "YES ✓" } else { "NO ✗" }
    );
    println!("  Proxy:      {}", if result.is_proxy { "Yes" } else { "No" });
    if result.is_proxy {
        if let Some(impl_addr) = &result.implementation {
            println!("  Impl:       {}", impl_addr);
        }
    }
    println!("  Etherscan:  {}", etherscan_url(chain_str, &result.address));
}

/// Print security analysis section
fn print_security_analysis(security: &SecurityResult) {
    println!();
    println!("Security Analysis:");
    println!(
        "  Honeypot:    {}",
        if security.is_honeypot { "YES ⚠️" } else { "No ✓" }
    );
    if let Some(buy) = security.buy_tax {
        println!("  Buy Tax:     {:.1}%", buy);
    }
    if let Some(sell) = security.sell_tax {
        let warning = if sell > 10.0 { " ⚠️" } else { "" };
        println!("  Sell Tax:    {:.1}%{}", sell, warning);
    }
    if let Some(owner) = &security.owner_address {
        if !owner.is_empty() {
            let status = if security.is_owner_renounced {
                " (renounced ✓)"
            } else {
                ""
            };
            let display = if owner.len() >= 10 { &owner[..10] } else { owner };
            println!("  Owner:       {}{}", display, status);
        }
    }
    if security.is_mintable {
        println!("  Mintable:    Yes ⚠️");
    }
    print_security_flags(security);
}

/// Print security flags (pausable, blacklist, etc.)
fn print_security_flags(security: &SecurityResult) {
    if security.transfer_pausable
        || security.can_blacklist
        || security.hidden_owner
        || security.anti_whale
    {
        println!("  Flags:");
        if security.transfer_pausable {
            println!("    - Transfers can be paused");
        }
        if security.can_blacklist {
            println!("    - Owner can blacklist addresses");
        }
        if security.hidden_owner {
            println!("    - Hidden owner");
        }
        if security.anti_whale {
            println!("    - Anti-whale mechanics");
        }
    }
}

/// Handle scam detection result and optional auto-blacklist
fn handle_scam_result(
    result: &ScanResult,
    address: &str,
    chain_str: &str,
    auto_blacklist: bool,
) -> anyhow::Result<()> {
    println!("⚠️  LIKELY SCAM: {}", result.reason());

    if auto_blacklist {
        let mut blacklist = TokenBlacklist::load_default();
        let already_listed = blacklist.is_blacklisted(address);

        if already_listed {
            println!("   Already in blacklist");
        } else {
            blacklist
                .add(
                    address,
                    result.symbol.clone(),
                    Some(result.reason()),
                    Some(chain_str.to_string()),
                )
                .map_err(|e| anyhow::anyhow!("{}", e))?;
            println!("   ✓ Added to blacklist");
        }
    } else {
        println!("   Use --auto-blacklist to add automatically");
    }
    Ok(())
}

/// Execute scan command
async fn execute_scan(address: &str, chain_str: &str, auto_blacklist: bool) -> anyhow::Result<()> {
    let chain = chain_str.parse::<Chain>().unwrap_or(Chain::Ethereum);
    let api_key = std::env::var("ETHERSCAN_API_KEY").ok();

    println!("Scanning token {}...", address);

    let result = scan_token(chain, address, api_key).await?;

    print_token_analysis(&result, chain_str);

    if let Some(sec) = &result.security {
        print_security_analysis(sec);
    }
    println!();

    if result.is_likely_scam() {
        handle_scam_result(&result, address, chain_str, auto_blacklist)?;
    } else {
        println!("✓ Token appears legitimate");
        if let Some(details) = result.security_details() {
            if !details.is_empty() {
                println!("  Note: {}", details);
            }
        }
    }

    Ok(())
}

/// Token info extracted from portfolio
struct TokenInfo {
    address: String,
    symbol: String,
}

/// Scan a single token and handle the result
async fn scan_single_token(
    token: &TokenInfo,
    chain: Chain,
    chain_str: &str,
    api_key: Option<String>,
    blacklist: &mut TokenBlacklist,
    auto_blacklist: bool,
    suspicious_only: bool,
) -> anyhow::Result<(bool, bool)> {
    // Returns (is_scam, is_verified)

    match scan_token(chain, &token.address, api_key).await {
        Ok(result) => {
            if result.is_likely_scam() {
                let sym = result.symbol.as_deref().unwrap_or(&token.symbol);
                let addr_short = &token.address[..10.min(token.address.len())];
                println!("⚠️  {} ({}) - {}", sym, addr_short, result.reason());
                println!("    {}", etherscan_url(chain_str, &token.address));

                if auto_blacklist {
                    blacklist
                        .add(
                            &token.address,
                            result.symbol.clone(),
                            Some(result.reason()),
                            Some(chain_str.to_string()),
                        )
                        .map_err(|e| anyhow::anyhow!("{}", e))?;
                    println!("    ✓ Added to blacklist");
                }
                Ok((true, false))
            } else {
                if !suspicious_only {
                    let sym = result.symbol.as_deref().unwrap_or(&token.symbol);
                    let addr_short = &token.address[..10.min(token.address.len())];
                    println!("✓  {} ({}) - Verified", sym, addr_short);
                }
                Ok((false, true))
            }
        }
        Err(e) => {
            let addr_short = &token.address[..10.min(token.address.len())];
            eprintln!("   Error scanning {}: {}", addr_short, e);
            Ok((false, false))
        }
    }
}

/// Print portfolio scan summary
fn print_scan_summary(verified_count: u32, scam_count: u32, auto_blacklist: bool) {
    println!();
    println!("Scan complete:");
    println!("  Verified tokens: {}", verified_count);
    println!("  Suspicious tokens: {}", scam_count);
    if auto_blacklist && scam_count > 0 {
        println!("  Added to blacklist: {}", scam_count);
    }
}

/// Execute scan-portfolio command
async fn execute_scan_portfolio(
    wallet_address: &str,
    chain_str: &str,
    auto_blacklist: bool,
    suspicious_only: bool,
) -> anyhow::Result<()> {
    use crate::aggregator::portfolio::fetch_portfolio_all;

    let chain = chain_str.parse::<Chain>().unwrap_or(Chain::Ethereum);
    let api_key = std::env::var("ETHERSCAN_API_KEY").ok();

    println!("Fetching portfolio for {}...", wallet_address);

    let chains = [chain_str];
    let portfolio_result = fetch_portfolio_all(wallet_address, &chains).await;
    let portfolio = portfolio_result.aggregated;

    if portfolio.tokens.is_empty() {
        eprintln!("No portfolio data available");
        return Ok(());
    }

    // Collect unique token addresses (exclude native ETH)
    let tokens: Vec<TokenInfo> = portfolio
        .tokens
        .iter()
        .filter(|t| t.address.to_lowercase() != "native" && t.address.starts_with("0x"))
        .map(|t| TokenInfo {
            address: t.address.clone(),
            symbol: t.symbol.clone(),
        })
        .collect();

    println!("Found {} tokens to scan", tokens.len());
    println!();

    let mut scam_count = 0u32;
    let mut verified_count = 0u32;
    let mut blacklist = TokenBlacklist::load_default();

    for token in &tokens {
        if blacklist.is_blacklisted(&token.address) {
            continue;
        }

        // Rate limit between API calls
        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;

        let (is_scam, is_verified) = scan_single_token(
            token,
            chain,
            chain_str,
            api_key.clone(),
            &mut blacklist,
            auto_blacklist,
            suspicious_only,
        )
        .await?;

        if is_scam {
            scam_count += 1;
        }
        if is_verified {
            verified_count += 1;
        }
    }

    print_scan_summary(verified_count, scam_count, auto_blacklist);

    Ok(())
}
