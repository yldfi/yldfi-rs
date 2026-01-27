//! Solodit API commands
//!
//! Search and explore smart contract security vulnerabilities from Solodit.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct SoloditArgs {
    #[command(subcommand)]
    pub command: SoloditCommands,
}

/// Search parameters struct to reduce enum variant size
#[derive(Args)]
pub struct SearchArgs {
    /// Keywords to search for
    pub keywords: String,

    /// Filter by impact level (HIGH, MEDIUM, LOW, GAS)
    #[arg(long, short = 'i', value_delimiter = ',')]
    pub impact: Option<Vec<String>>,

    /// Filter by audit firm name
    #[arg(long, short = 'f', value_delimiter = ',')]
    pub firm: Option<Vec<String>>,

    /// Filter by vulnerability tag (Reentrancy, Oracle, etc.)
    #[arg(long, short = 't', value_delimiter = ',')]
    pub tag: Option<Vec<String>>,

    /// Filter by protocol name (partial match)
    #[arg(long)]
    pub protocol: Option<String>,

    /// Filter by protocol category (DeFi, NFT, Bridge, etc.)
    #[arg(long, value_delimiter = ',')]
    pub protocol_category: Option<Vec<String>>,

    /// Filter by programming language (Solidity, Rust, etc.)
    #[arg(long)]
    pub language: Option<String>,

    /// Filter by finder/auditor handle (partial match)
    #[arg(long)]
    pub finder: Option<String>,

    /// Minimum number of finders
    #[arg(long)]
    pub min_finders: Option<u32>,

    /// Maximum number of finders
    #[arg(long)]
    pub max_finders: Option<u32>,

    /// Filter by report date period (30, 60, 90 days, or "all")
    #[arg(long)]
    pub reported: Option<String>,

    /// Filter by reports after this date (ISO format: 2024-01-01)
    #[arg(long)]
    pub reported_after: Option<String>,

    /// Page number (1-indexed)
    #[arg(long, default_value = "1")]
    pub page: u32,

    /// Results per page (max 100)
    #[arg(long, default_value = "20")]
    pub limit: u32,

    /// Sort by: recency, quality, rarity
    #[arg(long, default_value = "recency")]
    pub sort: String,

    /// Sort direction: asc or desc (default: desc)
    #[arg(long, default_value = "desc")]
    pub sort_dir: String,

    /// Minimum quality score (0-5)
    #[arg(long)]
    pub min_quality: Option<u32>,

    /// Minimum rarity score (0-5)
    #[arg(long)]
    pub min_rarity: Option<u32>,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum SoloditCommands {
    /// Search for vulnerability findings
    Search(Box<SearchArgs>),

    /// Get a specific finding by slug
    Get {
        /// Finding slug (from search results or URL)
        slug: String,

        /// Output format
        #[arg(long, short = 'o', default_value = "table")]
        format: OutputFormat,
    },

    /// Check current API rate limit
    RateLimit,

    /// List common vulnerability tags
    Tags,

    /// List common audit firms
    Firms,
}

/// Execute Solodit command
pub async fn execute(args: &SoloditArgs) -> anyhow::Result<()> {
    use crate::config::ConfigFile;
    use secrecy::ExposeSecret;

    // Try environment variable first, then config file
    let api_key = std::env::var("SOLODIT_API_KEY").ok().or_else(|| {
        ConfigFile::load_default()
            .ok()
            .flatten()
            .and_then(|c| c.solodit)
            .map(|s| s.api_key.expose_secret().to_string())
    });

    let api_key = api_key.ok_or_else(|| {
        anyhow::anyhow!(
            "SOLODIT_API_KEY not set. Get an API key from https://solodit.cyfrin.io (Profile > API Keys)\n\
             Set via: ethcli config set-solodit <key>"
        )
    })?;

    let client = sldt::Client::new(&api_key)
        .map_err(|e| anyhow::anyhow!("Failed to create Solodit client: {}", e))?;

    match &args.command {
        SoloditCommands::Search(search_args) => {
            let SearchArgs {
                keywords,
                impact,
                firm,
                tag,
                protocol,
                protocol_category,
                language,
                finder,
                min_finders,
                max_finders,
                reported,
                reported_after,
                page,
                limit,
                sort,
                sort_dir,
                min_quality,
                min_rarity,
                format,
            } = search_args.as_ref();

            let mut filter = sldt::SearchFilter::new(keywords)
                .page(*page)
                .page_size(*limit);

            // Apply impact filters
            if let Some(impacts) = impact {
                for i in impacts {
                    let impact = match i.to_uppercase().as_str() {
                        "HIGH" => sldt::Impact::High,
                        "MEDIUM" => sldt::Impact::Medium,
                        "LOW" => sldt::Impact::Low,
                        "GAS" => sldt::Impact::Gas,
                        _ => continue,
                    };
                    filter = filter.impact(impact);
                }
            }

            // Apply firm filters
            if let Some(firms) = firm {
                for f in firms {
                    filter = filter.firm(f.as_str());
                }
            }

            // Apply tag filters
            if let Some(tags) = tag {
                for t in tags {
                    filter = filter.tag(t.as_str());
                }
            }

            // Apply protocol filter
            if let Some(p) = protocol {
                filter = filter.protocol(p);
            }

            // Apply protocol category filters
            if let Some(categories) = protocol_category {
                for c in categories {
                    filter = filter.protocol_category(c.as_str());
                }
            }

            // Apply language filter
            if let Some(lang) = language {
                filter = filter.language(lang.as_str());
            }

            // Apply finder/user filter
            if let Some(user) = finder {
                filter = filter.user(user);
            }

            // Apply finder count range
            if min_finders.is_some() || max_finders.is_some() {
                filter = filter.finders_range(*min_finders, *max_finders);
            }

            // Apply reported date filter
            if let Some(period) = reported {
                let period = match period.to_lowercase().as_str() {
                    "30" => sldt::ReportedPeriod::Days30,
                    "60" => sldt::ReportedPeriod::Days60,
                    "90" => sldt::ReportedPeriod::Days90,
                    _ => sldt::ReportedPeriod::AllTime,
                };
                filter = filter.reported(period);
            }

            // Apply reported after date
            if let Some(date) = reported_after {
                filter = filter.reported_after(date);
            }

            // Apply sort field
            filter = match sort.to_lowercase().as_str() {
                "quality" => filter.sort_by_quality(),
                "rarity" => filter.sort_by_rarity(),
                _ => filter.sort_by_recency(),
            };

            // Apply sort direction
            filter = match sort_dir.to_lowercase().as_str() {
                "asc" | "ascending" => filter.ascending(),
                _ => filter.descending(),
            };

            // Apply quality filter
            if let Some(q) = min_quality {
                filter = filter.min_quality(*q);
            }

            // Apply rarity filter
            if let Some(r) = min_rarity {
                filter = filter.min_rarity(*r);
            }

            let results = client.search_with_filter(filter).await?;

            if matches!(format, OutputFormat::Json | OutputFormat::Ndjson) {
                // Build JSON response
                let response = serde_json::json!({
                    "findings": results.findings,
                    "total": results.total,
                    "page": results.page,
                    "page_size": results.page_size,
                    "total_pages": results.total_pages,
                    "rate_limit": {
                        "remaining": results.rate_limit.remaining,
                        "limit": results.rate_limit.limit,
                        "reset": results.rate_limit.reset
                    }
                });
                if matches!(format, OutputFormat::Json) {
                    println!("{}", serde_json::to_string_pretty(&response)?);
                } else {
                    println!("{}", serde_json::to_string(&response)?);
                }
                return Ok(());
            }

            // Human-readable output
            println!("Search Results: \"{}\"", keywords);
            println!("{}", "=".repeat(60));
            println!(
                "Found {} results (page {}/{}) - Rate limit: {}/{}",
                results.total,
                results.page,
                results.total_pages,
                results.rate_limit.remaining,
                results.rate_limit.limit
            );
            println!();

            if results.findings.is_empty() {
                println!("No findings match your search criteria.");
                return Ok(());
            }

            for finding in &results.findings {
                let impact = finding.impact_level();
                let impact_badge = match impact {
                    sldt::Impact::High => "[HIGH]",
                    sldt::Impact::Medium => "[MED]",
                    sldt::Impact::Low => "[LOW]",
                    sldt::Impact::Gas => "[GAS]",
                    sldt::Impact::Unknown => "[???]",
                };

                println!(
                    "{} {}",
                    impact_badge,
                    finding.title.as_deref().unwrap_or("Untitled")
                );

                if let Some(firm) = finding.firm() {
                    print!("    Firm: {}", firm);
                }
                if let Some(protocol) = finding.protocol() {
                    print!(" | Protocol: {}", protocol);
                }
                println!();

                let tags = finding.tags();
                if !tags.is_empty() {
                    println!("    Tags: {}", tags.join(", "));
                }

                if let Some(url) = finding.solodit_url() {
                    println!("    URL: {}", url);
                }

                println!();
            }

            if results.has_more() {
                println!(
                    "Showing page {} of {}. Use --page {} to see more.",
                    results.page,
                    results.total_pages,
                    results.page + 1
                );
            }
        }

        SoloditCommands::Get { slug, format } => {
            let finding = client.get_by_slug(slug).await?;

            if matches!(format, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&finding)?);
                return Ok(());
            }
            if matches!(format, OutputFormat::Ndjson) {
                println!("{}", serde_json::to_string(&finding)?);
                return Ok(());
            }

            println!("Finding Details");
            println!("{}", "=".repeat(60));
            println!(
                "Title:    {}",
                finding.title.as_deref().unwrap_or("Untitled")
            );
            println!("Impact:   {}", finding.impact_level());

            if let Some(firm) = finding.firm() {
                println!("Firm:     {}", firm);
            }
            if let Some(protocol) = finding.protocol() {
                println!("Protocol: {}", protocol);
            }

            let tags = finding.tags();
            if !tags.is_empty() {
                println!("Tags:     {}", tags.join(", "));
            }

            if let Some(q) = finding.quality_score {
                println!("Quality:  {:.1}/5", q);
            }
            if let Some(r) = finding.general_score {
                println!("Rarity:   {:.1}/5", r);
            }

            let finders = finding.finder_handles();
            if !finders.is_empty() {
                println!("Finders:  {}", finders.join(", "));
            }

            if let Some(url) = finding.solodit_url() {
                println!("URL:      {}", url);
            }

            if let Some(source) = &finding.source_link {
                println!("Source:   {}", source);
            }

            println!();
            println!("Content:");
            println!("{}", "-".repeat(60));
            if let Some(content) = &finding.content {
                // Truncate long content for display
                let display_content = if content.len() > 2000 {
                    format!(
                        "{}...\n\n[Content truncated. Use --format json for full content]",
                        &content[..2000]
                    )
                } else {
                    content.clone()
                };
                println!("{}", display_content);
            } else {
                println!("(No content available)");
            }
        }

        SoloditCommands::RateLimit => {
            let rate_limit = client.check_rate_limit().await?;
            println!("Solodit API Rate Limit Status");
            println!("{}", "=".repeat(40));
            println!("Remaining: {}/{}", rate_limit.remaining, rate_limit.limit);
            println!("Resets at: {} (Unix timestamp)", rate_limit.reset);

            // Convert to human-readable time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after UNIX epoch")
                .as_secs();
            if rate_limit.reset > now {
                let secs_remaining = rate_limit.reset - now;
                println!("Resets in: {} seconds", secs_remaining);
            }
        }

        SoloditCommands::Tags => {
            println!("Common Solodit Vulnerability Tags");
            println!("{}", "=".repeat(40));
            println!();
            let tags = [
                "Reentrancy",
                "Oracle",
                "Access Control",
                "Integer Overflow/Underflow",
                "Front-running",
                "Logic Error",
                "DOS",
                "Price Manipulation",
                "Flash Loan",
                "Griefing",
                "Signature Malleability",
                "Timestamp Dependence",
                "Unchecked Return Value",
                "Gas Optimization",
            ];
            for tag in tags {
                println!("  - {}", tag);
            }
            println!();
            println!("Use --tag <name> to filter by tag");
        }

        SoloditCommands::Firms => {
            println!("Common Solodit Audit Firms");
            println!("{}", "=".repeat(40));
            println!();
            let firms = [
                "Cyfrin",
                "Sherlock",
                "Code4rena",
                "Trail of Bits",
                "OpenZeppelin",
                "Consensys Diligence",
                "Pashov Audit Group",
                "Spearbit",
                "Hacken",
                "CertiK",
                "Chainsecurity",
                "Quantstamp",
            ];
            for firm in firms {
                println!("  - {}", firm);
            }
            println!();
            println!("Use --firm <name> to filter by audit firm");
        }
    }

    Ok(())
}
