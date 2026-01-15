//! Resolve API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// API for domain resolution
pub struct ResolveApi<'a> {
    client: &'a Client,
}

impl<'a> ResolveApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Resolve a domain to an address (supports ENS, Unstoppable Domains, etc)
    pub async fn resolve_domain(&self, domain: &str) -> Result<ResolvedDomain> {
        let path = format!("/resolve/{}", domain);
        self.client.get(&path).await
    }

    /// Reverse resolve an address to a domain
    pub async fn reverse_resolve(&self, address: &str) -> Result<ReverseResolution> {
        let path = format!("/resolve/{}/reverse", address);
        self.client.get(&path).await
    }

    /// Get all domains for an address
    pub async fn get_address_domains(&self, address: &str) -> Result<DomainLookup> {
        let path = format!("/resolve/{}/domain", address);
        self.client.get(&path).await
    }

    /// Get ENS domain details
    pub async fn get_ens_domain(&self, domain: &str) -> Result<EnsDomain> {
        let path = format!("/resolve/ens/{}", domain);
        self.client.get(&path).await
    }
}
