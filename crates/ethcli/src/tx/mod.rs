//! Transaction analysis module
//!
//! Provides tools for analyzing Ethereum transactions, including:
//! - Fetching transaction data and receipts
//! - Decoding events with known signatures
//! - Labeling known contracts and tokens
//! - Tracking token flows

pub mod addresses;
pub mod analyzer;
pub mod flow;
pub mod types;

pub use analyzer::{format_analysis, TxAnalyzer};
pub use types::{AnalyzedEvent, ContractInfo, TokenFlow, TransactionAnalysis};
