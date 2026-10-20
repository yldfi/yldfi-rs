//! Strategy-related GraphQL queries

use crate::client::Client;
use crate::error::Result;
use crate::types::Strategy;
use serde::Deserialize;

/// Fields selected for `Strategy` (validated against the live Kong schema:
/// `Strategy` has no `activation`, `totalGain`, `totalLoss` or `debtRatio`;
/// `risk` is `RiskScoreLegacy`; `tvl` is a `SparklinePoint`).
const STRATEGY_FIELDS: &str = "address name chainId apiVersion vault v3 inceptTime inceptBlock \
    lastReport totalDebt performanceFee estimatedTotalAssets isActive isShutdown keeper strategist \
    risk { label auditScore codeReviewScore complexityScore protocolSafetyScore teamKnowledgeScore testingScore } \
    apy { net weeklyNet monthlyNet inceptionNet grossApr } \
    tvl { close blockTime }";

/// Strategy query builder for filtering strategies
#[derive(Debug, Default, Clone)]
pub struct StrategyFilter {
    chain_id: Option<u64>,
    vault: Option<String>,
    v3: Option<bool>,
    addresses: Option<Vec<String>>,
}

impl StrategyFilter {
    /// Create a new filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by chain ID
    #[must_use]
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Filter by vault address
    #[must_use]
    pub fn vault(mut self, vault: impl Into<String>) -> Self {
        self.vault = Some(vault.into());
        self
    }

    /// Filter v3 strategies only
    #[must_use]
    pub fn v3(mut self, v3: bool) -> Self {
        self.v3 = Some(v3);
        self
    }

    /// Filter by specific addresses
    #[must_use]
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Build the GraphQL arguments string.
    ///
    /// Kong's `strategies` query only accepts `chainId`, `apiVersion` and
    /// `erc4626`; vault filtering uses `vaultStrategies(chainId, vault)` and
    /// `v3`/`addresses` are applied client-side (see [`Self::retain`]).
    fn build_args(&self) -> String {
        match self.chain_id {
            Some(_) => "(chainId: $chainId)".to_string(),
            None => String::new(),
        }
    }

    /// Variable declarations and variables matching [`Self::build_args`]
    fn build_variables(&self) -> (&'static str, serde_json::Value) {
        match self.chain_id {
            Some(chain_id) => (
                "($chainId: Int)",
                serde_json::json!({ "chainId": chain_id }),
            ),
            None => ("", serde_json::Value::Null),
        }
    }

    /// Apply the client-side parts of the filter
    fn retain(&self, strategies: &mut Vec<Strategy>) {
        if let Some(v3) = self.v3 {
            strategies.retain(|s| s.v3.unwrap_or(false) == v3);
        }
        if let Some(ref addresses) = self.addresses {
            let wanted: Vec<String> = addresses.iter().map(|a| a.to_lowercase()).collect();
            strategies.retain(|s| wanted.contains(&s.address.to_lowercase()));
        }
    }
}

/// Query text and variables for `strategies(...)`
fn list_query(filter: &StrategyFilter) -> (String, serde_json::Value) {
    let (decls, variables) = filter.build_variables();
    let args = filter.build_args();
    let query = format!("query Strategies{decls} {{ strategies{args} {{ {STRATEGY_FIELDS} }} }}");
    (query, variables)
}

/// Strategies API
pub struct StrategiesApi<'a> {
    client: &'a Client,
}

impl<'a> StrategiesApi<'a> {
    /// Create a new strategies API instance
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all strategies (with optional filter)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// let strategies = client.strategies().list(None).await?;
    /// println!("Found {} strategies", strategies.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, filter: Option<StrategyFilter>) -> Result<Vec<Strategy>> {
        let filter = filter.unwrap_or_default();

        let mut strategies = if let Some(ref vault) = filter.vault {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct Response {
                vault_strategies: Vec<Strategy>,
            }
            let chain_id = vault_chain_id(&filter)?;
            let query = format!(
                "query VaultStrategies($chainId: Int, $vault: String) {{ \
                 vaultStrategies(chainId: $chainId, vault: $vault) {{ {STRATEGY_FIELDS} }} }}"
            );
            let variables = serde_json::json!({ "chainId": chain_id, "vault": vault });
            let response: Response = self.client.query_with_variables(&query, variables).await?;
            response.vault_strategies
        } else {
            #[derive(Deserialize)]
            struct Response {
                strategies: Vec<Strategy>,
            }
            let (query, variables) = list_query(&filter);
            let response: Response = self.client.query_with_variables(&query, variables).await?;
            response.strategies
        };

        filter.retain(&mut strategies);
        Ok(strategies)
    }

    /// Get strategies for a specific chain
    pub async fn by_chain(&self, chain_id: u64) -> Result<Vec<Strategy>> {
        self.list(Some(StrategyFilter::new().chain_id(chain_id)))
            .await
    }

    /// Get strategies for a specific vault
    pub async fn by_vault(&self, chain_id: u64, vault: &str) -> Result<Vec<Strategy>> {
        self.list(Some(StrategyFilter::new().chain_id(chain_id).vault(vault)))
            .await
    }

    /// Get a single strategy by address and chain
    pub async fn get(&self, chain_id: u64, address: &str) -> Result<Option<Strategy>> {
        let query = format!(
            "query Strategy($chainId: Int, $address: String) {{ \
             strategy(chainId: $chainId, address: $address) {{ {STRATEGY_FIELDS} }} }}"
        );
        let variables = serde_json::json!({ "chainId": chain_id, "address": address });

        #[derive(Deserialize)]
        struct Response {
            strategy: Option<Strategy>,
        }

        let response: Response = self.client.query_with_variables(&query, variables).await?;
        Ok(response.strategy)
    }
}

/// Chain ID for a `vaultStrategies` query; a vault address is only unique
/// per chain, so the filter must name one
fn vault_chain_id(filter: &StrategyFilter) -> Result<u64> {
    filter.chain_id.ok_or_else(|| {
        crate::error::invalid_argument("filtering strategies by vault requires a chain ID")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vault_filter_requires_chain_id() {
        assert!(vault_chain_id(&StrategyFilter::new().vault("0xv")).is_err());
        assert_eq!(
            vault_chain_id(&StrategyFilter::new().chain_id(137).vault("0xv")).unwrap(),
            137
        );
    }

    #[test]
    fn only_supported_server_args_are_sent() {
        let f = StrategyFilter::new().chain_id(1).vault("0xv").v3(true);
        assert_eq!(f.build_args(), "(chainId: $chainId)");
        assert_eq!(StrategyFilter::new().build_args(), "");
    }

    #[test]
    fn list_query_uses_variables() {
        let (query, variables) = list_query(&StrategyFilter::new().chain_id(137));
        assert!(
            query.starts_with("query Strategies($chainId: Int) { strategies(chainId: $chainId) {"),
            "{query}"
        );
        assert_eq!(variables, serde_json::json!({ "chainId": 137 }));

        let (query, variables) = list_query(&StrategyFilter::new());
        assert!(
            query.starts_with("query Strategies { strategies {"),
            "{query}"
        );
        assert!(variables.is_null());
    }

    #[test]
    fn strategy_fields_exclude_removed_fields() {
        for bad in [
            "activation",
            "totalGain",
            "totalLoss",
            "debtRatio",
            "riskLevel",
            "blockNumber",
        ] {
            assert!(!STRATEGY_FIELDS.contains(bad), "{bad}");
        }
    }
}
