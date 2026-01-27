//! GoPlus Security API commands
//!
//! Query token, address, NFT, and approval security information.

use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GoPlusArgs {
    #[command(subcommand)]
    pub command: GoPlusCommands,
}

#[derive(Subcommand)]
pub enum GoPlusCommands {
    /// Check token security (honeypot, tax, ownership)
    Token {
        /// Token contract address
        address: String,

        /// Chain ID (1=Ethereum, 56=BSC, 137=Polygon, etc.)
        #[arg(long = "chain-id", short = 'i', default_value = "1")]
        chain_id: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check multiple tokens at once (requires auth for >1)
    TokenBatch {
        /// Token contract addresses (comma-separated)
        addresses: String,

        /// Chain ID
        #[arg(long = "chain-id", short = 'i', default_value = "1")]
        chain_id: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check if an address is malicious
    Address {
        /// Address to check
        address: String,

        /// Chain ID (1=Ethereum, 56=BSC, 137=Polygon, etc.)
        #[arg(long = "chain-id", short = 'i', default_value = "1")]
        chain_id: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check NFT collection security
    Nft {
        /// NFT contract address
        address: String,

        /// Chain ID
        #[arg(long = "chain-id", short = 'i', default_value = "1")]
        chain_id: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check approval security (ERC20/721/1155)
    Approval {
        /// Contract address to check approvals for
        address: String,

        /// Chain ID
        #[arg(long = "chain-id", short = 'i', default_value = "1")]
        chain_id: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List supported chains
    Chains,
}

/// Execute GoPlus command
pub async fn execute(args: &GoPlusArgs) -> anyhow::Result<()> {
    // Use authenticated client if credentials are available
    let client = gplus::Client::from_env()?;

    match &args.command {
        GoPlusCommands::Token { address, chain_id, json } => {
            let security = client.token_security(*chain_id, address).await?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&security)?);
                return Ok(());
            }

            println!("Token Security Analysis");
            println!("{}", "=".repeat(50));
            println!(
                "Token:      {} ({})",
                security.token_name.as_deref().unwrap_or("Unknown"),
                security.token_symbol.as_deref().unwrap_or("???")
            );
            println!("Address:    {}", address);
            println!("Chain ID:   {}", chain_id);
            println!();

            // Contract info
            println!("Contract:");
            println!(
                "  Verified:     {}",
                if security.is_verified() { "Yes ✓" } else { "No ✗" }
            );
            println!(
                "  Proxy:        {}",
                if security.is_proxy() { "Yes" } else { "No" }
            );
            println!(
                "  Mintable:     {}",
                if security.is_mintable() { "Yes ⚠️" } else { "No" }
            );
            println!();

            // Trading info
            println!("Trading:");
            println!(
                "  Honeypot:     {}",
                if security.is_honeypot() {
                    "Yes ⚠️ CANNOT SELL"
                } else {
                    "No ✓"
                }
            );
            if let Some(buy) = security.buy_tax_percent() {
                println!("  Buy Tax:      {:.1}%", buy);
            }
            if let Some(sell) = security.sell_tax_percent() {
                let warning = if sell > 10.0 { " ⚠️" } else { "" };
                println!("  Sell Tax:     {:.1}%{}", sell, warning);
            }
            println!();

            // Ownership
            println!("Ownership:");
            if let Some(owner) = &security.owner_address {
                if !owner.is_empty() {
                    let status = if security.is_owner_renounced() {
                        " (renounced ✓)"
                    } else {
                        ""
                    };
                    println!("  Owner:        {}{}", &owner[..20.min(owner.len())], status);
                }
            }
            println!(
                "  Hidden Owner: {}",
                if security.has_hidden_owner() {
                    "Yes ⚠️"
                } else {
                    "No"
                }
            );
            println!();

            // Controls
            println!("Controls:");
            println!(
                "  Pausable:     {}",
                if security.is_transfer_pausable() {
                    "Yes ⚠️"
                } else {
                    "No"
                }
            );
            println!(
                "  Blacklist:    {}",
                if security.can_blacklist() {
                    "Yes ⚠️"
                } else {
                    "No"
                }
            );
            println!(
                "  Anti-whale:   {}",
                if security.has_anti_whale() { "Yes" } else { "No" }
            );
            println!();

            // Summary
            let issues = security.get_issues();
            if issues.is_empty() {
                println!("✓ No major issues detected");
            } else {
                println!("⚠️  Issues detected:");
                for issue in &issues {
                    println!("  - {}", issue);
                }
            }
        }

        GoPlusCommands::TokenBatch { addresses, chain_id, json } => {
            let addrs: Vec<&str> = addresses.split(',').map(|s| s.trim()).collect();
            let results = client.token_security_batch(*chain_id, &addrs).await?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&results)?);
                return Ok(());
            }

            println!(
                "{:<44} {:<10} {:<10} {:<10} {:<10}",
                "Address", "Symbol", "Honeypot", "Sell Tax", "Verified"
            );
            println!("{}", "-".repeat(90));

            for (addr, security) in &results {
                let symbol = security.token_symbol.as_deref().unwrap_or("???");
                let honeypot = if security.is_honeypot() { "YES ⚠️" } else { "No" };
                let sell_tax = security
                    .sell_tax_percent()
                    .map(|t| format!("{:.1}%", t))
                    .unwrap_or_else(|| "-".to_string());
                let verified = if security.is_verified() { "Yes" } else { "No ⚠️" };

                println!(
                    "{:<44} {:<10} {:<10} {:<10} {:<10}",
                    addr, symbol, honeypot, sell_tax, verified
                );
            }
        }

        GoPlusCommands::Address { address, chain_id, json } => {
            let security = client.address_security(*chain_id, address).await?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&security)?);
                return Ok(());
            }

            println!("Address Security Analysis");
            println!("{}", "=".repeat(50));
            println!("Address:    {}", address);
            println!("Chain ID:   {}", chain_id);
            println!();

            let issues = security.get_issues();
            if security.is_malicious() {
                println!("⚠️  MALICIOUS ADDRESS DETECTED");
                for issue in &issues {
                    println!("  - {}", issue);
                }
            } else if !issues.is_empty() {
                println!("⚠️  Potential risks:");
                for issue in &issues {
                    println!("  - {}", issue);
                }
            } else {
                println!("✓ No issues detected");
            }
        }

        GoPlusCommands::Nft { address, chain_id, json } => {
            let security = client.nft_security(*chain_id, address).await?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&security)?);
                return Ok(());
            }

            println!("NFT Security Analysis");
            println!("{}", "=".repeat(50));
            println!(
                "Collection: {} ({})",
                security.nft_name.as_deref().unwrap_or("Unknown"),
                security.nft_symbol.as_deref().unwrap_or("???")
            );
            println!("Address:    {}", address);
            println!("Chain:      {}", chain_id);
            println!();

            println!(
                "Verified:      {}",
                if security.is_verified() { "Yes ✓" } else { "No ⚠️" }
            );
            println!(
                "Open Source:   {}",
                if security.is_open_source() { "Yes" } else { "No ⚠️" }
            );
            println!(
                "Honeypot:      {}",
                if security.is_honeypot() { "Yes ⚠️" } else { "No ✓" }
            );
            println!();

            if let Some(url) = &security.website_url {
                if !url.is_empty() {
                    println!("Website:    {}", url);
                }
            }
            if let Some(twitter) = &security.twitter_url {
                if !twitter.is_empty() {
                    println!("Twitter:    {}", twitter);
                }
            }
            if let Some(discord) = &security.discord_url {
                if !discord.is_empty() {
                    println!("Discord:    {}", discord);
                }
            }
            println!();

            if security.has_risks() {
                println!("⚠️  Risks detected");
            } else {
                println!("✓ No major issues detected");
            }
        }

        GoPlusCommands::Approval { address, chain_id, json } => {
            let security = client.approval_security(*chain_id, address).await?;

            if *json {
                println!("{}", serde_json::to_string_pretty(&security)?);
                return Ok(());
            }

            println!("Approval Security Analysis");
            println!("{}", "=".repeat(50));
            if let Some(name) = &security.contract_name {
                println!("Contract:   {}", name);
            }
            println!("Address:    {}", address);
            println!("Chain ID:   {}", chain_id);
            println!();

            println!(
                "Open Source: {}",
                if security.is_open_source == Some(1) {
                    "Yes ✓"
                } else {
                    "No"
                }
            );
            println!(
                "Proxy:       {}",
                if security.is_proxy == Some(1) {
                    "Yes"
                } else {
                    "No"
                }
            );
            println!();

            if security.is_malicious() {
                println!("⚠️  MALICIOUS CONTRACT");
                if let Some(tag) = &security.tag {
                    println!("  Tag: {}", tag);
                }
                if let Some(behaviors) = &security.malicious_behavior {
                    for b in behaviors {
                        println!("  - {}", b);
                    }
                }
            } else if security.is_trusted() {
                println!("✓ Trusted contract");
            } else if security.is_doubtful() {
                println!("⚠️  Doubtful contract (on doubt list)");
            } else {
                println!("Unknown trust status");
            }
        }

        GoPlusCommands::Chains => {
            println!("Supported Chains:");
            println!("{}", "-".repeat(30));
            println!("{:<20} ID", "Chain");
            println!("{}", "-".repeat(30));
            println!("{:<20} {}", "Ethereum", 1);
            println!("{:<20} {}", "BSC", 56);
            println!("{:<20} {}", "Polygon", 137);
            println!("{:<20} {}", "Arbitrum", 42161);
            println!("{:<20} {}", "Base", 8453);
            println!("{:<20} {}", "Avalanche", 43114);
            println!("{:<20} {}", "Optimism", 10);
            println!("{:<20} {}", "Fantom", 250);
            println!("{:<20} {}", "Cronos", 25);
            println!("{:<20} {}", "Gnosis", 100);
            println!("{:<20} {}", "Heco", 128);
            println!("{:<20} {}", "Linea", 59144);
            println!("{:<20} {}", "Scroll", 534352);
            println!("{:<20} {}", "Mantle", 5000);
            println!("{:<20} {}", "zkSync Era", 324);
            println!("{:<20} {}", "Blast", 81457);
        }
    }

    Ok(())
}
