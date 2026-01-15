//! Types for the Resolve API

use serde::{Deserialize, Serialize};

/// ENS domain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnsDomain {
    /// Domain name
    pub name: Option<String>,
    /// Resolved address
    pub address: Option<String>,
    /// Registrant address
    pub registrant_address: Option<String>,
    /// Owner address
    pub owner_address: Option<String>,
    /// Expiration date
    pub expiration_date: Option<String>,
    /// Text records
    pub text_records: Option<serde_json::Value>,
}

/// Resolved domain response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDomain {
    /// Domain name
    pub name: Option<String>,
    /// Resolved address
    pub address: Option<String>,
}

/// Reverse resolve response (address to domain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseResolution {
    /// Address
    pub address: Option<String>,
    /// Domain name
    pub name: Option<String>,
}

/// Domain lookup response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainLookup {
    /// Address
    pub address: Option<String>,
    /// Domains
    pub domains: Option<Vec<DomainInfo>>,
}

/// Domain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    /// Domain name
    pub name: Option<String>,
    /// Domain type (ENS, Unstoppable, etc)
    pub domain_type: Option<String>,
}
