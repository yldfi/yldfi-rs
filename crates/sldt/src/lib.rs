//! # sldt
//!
//! Unofficial Rust client for the Solodit smart contract vulnerability database.
//!
//! [Solodit](https://solodit.cyfrin.io) aggregates security vulnerabilities and findings
//! from audits across the web3 ecosystem. This crate provides programmatic access to
//! search and retrieve vulnerability reports.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> sldt::Result<()> {
//! // Create client with your API key (get from solodit.cyfrin.io)
//! let client = sldt::Client::new("sk_your_api_key_here")?;
//!
//! // Search for findings
//! let results = client.search("reentrancy").await?;
//! for finding in results.findings {
//!     println!("[{}] {}", finding.impact_level(), finding.title.unwrap_or_default());
//! }
//!
//! println!("Rate limit remaining: {}", results.rate_limit.remaining);
//! # Ok(())
//! # }
//! ```
//!
//! ## Filtered Search
//!
//! ```no_run
//! # async fn example() -> sldt::Result<()> {
//! use sldt::{SearchFilter, Impact};
//!
//! let client = sldt::Client::new("sk_your_api_key")?;
//!
//! let filter = SearchFilter::new("flash loan")
//!     .impact(Impact::High)
//!     .impact(Impact::Medium)
//!     .firm("Cyfrin")
//!     .page_size(50)
//!     .sort_by_quality();
//!
//! let results = client.search_with_filter(filter).await?;
//! println!("Found {} total results", results.total);
//! # Ok(())
//! # }
//! ```
//!
//! ## Pagination
//!
//! ```no_run
//! # async fn example() -> sldt::Result<()> {
//! let client = sldt::Client::new("sk_your_api_key")?;
//! let mut paginator = client.paginate(sldt::SearchFilter::new("oracle"));
//!
//! while let Some(findings) = paginator.next_page().await? {
//!     for finding in findings {
//!         println!("{}", finding.title.unwrap_or_default());
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Terms of Service
//!
//! This is an unofficial client. When using Solodit data, you must comply with
//! [Solodit's Terms of Service](https://solodit.cyfrin.io/terms-of-service):
//!
//! - **Permitted**: Integrating data into products, derivative works, commercial use
//! - **Prohibited**: Raw data redistribution, database mirroring, competing services
//! - **Required**: Attribution when publishing research or derivative works
//!
//! ## Disclaimer
//!
//! This crate is not affiliated with or endorsed by Cyfrin or Solodit.

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, FindingPaginator, BASE_URL};
pub use error::{Error, Result};
pub use types::{
    ApiResponse, AuditFirm, FilterValue, Finding, Impact, IssueFinder, IssueTag, IssueTagScore,
    Protocol, ProtocolCategory, ProtocolCategoryScore, RateLimit, ReportedPeriod, ResponseMetadata,
    SearchFilter, SearchResults, SortDirection, SortField, Warden,
};
