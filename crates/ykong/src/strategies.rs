//! Strategy-related GraphQL queries

use crate::client::Client;
use crate::error::Result;
use crate::types::Strategy;
use serde::Deserialize;

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

    /// Build the GraphQL arguments string
    fn build_args(&self) -> String {
        let mut args = Vec::new();

        if let Some(chain_id) = self.chain_id {
            args.push(format!("chainId: {chain_id}"));
        }
        if let Some(ref vault) = self.vault {
            args.push(format!("vault: \"{vault}\""));
        }
        if let Some(v3) = self.v3 {
            args.push(format!("v3: {v3}"));
        }
        if let Some(ref addresses) = self.addresses {
            let addr_str: Vec<String> = addresses.iter().map(|a| format!("\"{a}\"")).collect();
            args.push(format!("addresses: [{}]", addr_str.join(", ")));
        }

        if args.is_empty() {
            String::new()
        } else {
            format!("({})", args.join(", "))
        }
    }
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
        let args = filter.unwrap_or_default().build_args();
        let query = format!(
            r"{{
                strategies{args} {{
                    address
                    name
                    chainId
                    apiVersion
                    vault
                    v3
                    activation
                    inceptTime
                    inceptBlock
                    lastReport
                    totalDebt
                    totalGain
                    totalLoss
                    performanceFee
                    debtRatio
                    estimatedTotalAssets
                    isActive
                    isShutdown
                    keeper
                    strategist
                    risk {{ riskLevel riskGroup }}
                    apy {{ net weeklyNet monthlyNet }}
                    tvl {{ close blockNumber blockTime }}
                }}
            }}"
        );

        #[derive(Deserialize)]
        struct Response {
            strategies: Vec<Strategy>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.strategies)
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
            r#"{{
                strategy(chainId: {chain_id}, address: "{address}") {{
                    address
                    name
                    chainId
                    apiVersion
                    vault
                    v3
                    activation
                    inceptTime
                    inceptBlock
                    lastReport
                    totalDebt
                    totalGain
                    totalLoss
                    performanceFee
                    debtRatio
                    estimatedTotalAssets
                    isActive
                    isShutdown
                    keeper
                    strategist
                    risk {{ riskLevel riskGroup tvlImpact auditScore codeReviewScore complexityScore longevityImpact protocolSafetyScore teamKnowledgeScore testingScore }}
                    apy {{ net weeklyNet monthlyNet inceptionNet grossApr }}
                    tvl {{ close blockNumber blockTime }}
                }}
            }}"#
        );

        #[derive(Deserialize)]
        struct Response {
            strategy: Option<Strategy>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.strategy)
    }
}
