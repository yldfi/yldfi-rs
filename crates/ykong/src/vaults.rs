//! Vault-related GraphQL queries

use crate::client::Client;
use crate::error::Result;
use crate::types::{Vault, VaultAccount};
use serde::Deserialize;

/// Vault query builder for filtering vaults
#[derive(Debug, Default, Clone)]
pub struct VaultFilter {
    chain_id: Option<u64>,
    api_version: Option<String>,
    v3: Option<bool>,
    yearn: Option<bool>,
    erc4626: Option<bool>,
    vault_type: Option<i32>,
    risk_level: Option<i32>,
    addresses: Option<Vec<String>>,
}

impl VaultFilter {
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

    /// Filter by API version
    #[must_use]
    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = Some(version.into());
        self
    }

    /// Filter v3 vaults only
    #[must_use]
    pub fn v3(mut self, v3: bool) -> Self {
        self.v3 = Some(v3);
        self
    }

    /// Filter Yearn vaults only
    #[must_use]
    pub fn yearn(mut self, yearn: bool) -> Self {
        self.yearn = Some(yearn);
        self
    }

    /// Filter ERC4626 compliant vaults
    #[must_use]
    pub fn erc4626(mut self, erc4626: bool) -> Self {
        self.erc4626 = Some(erc4626);
        self
    }

    /// Filter by vault type (0 = default, 1 = automated, 2 = multi-strategy)
    #[must_use]
    pub fn vault_type(mut self, vault_type: i32) -> Self {
        self.vault_type = Some(vault_type);
        self
    }

    /// Filter by risk level (1-5)
    #[must_use]
    pub fn risk_level(mut self, level: i32) -> Self {
        self.risk_level = Some(level);
        self
    }

    /// Filter by specific addresses
    #[must_use]
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Build the GraphQL variable declarations, field arguments and
    /// variables for the `vaults` query.
    ///
    /// User-provided values are only ever sent as GraphQL variables, never
    /// interpolated into the query text.
    fn build_args(&self) -> (String, String, serde_json::Value) {
        let mut decls = Vec::new();
        let mut args = Vec::new();
        let mut vars = serde_json::Map::new();
        let mut push = |name: &str, ty: &str, value: serde_json::Value| {
            decls.push(format!("${name}: {ty}"));
            args.push(format!("{name}: ${name}"));
            vars.insert(name.to_string(), value);
        };

        if let Some(chain_id) = self.chain_id {
            push("chainId", "Int", chain_id.into());
        }
        if let Some(ref version) = self.api_version {
            push("apiVersion", "String", version.clone().into());
        }
        if let Some(v3) = self.v3 {
            push("v3", "Boolean", v3.into());
        }
        if let Some(yearn) = self.yearn {
            push("yearn", "Boolean", yearn.into());
        }
        if let Some(erc4626) = self.erc4626 {
            push("erc4626", "Boolean", erc4626.into());
        }
        if let Some(vault_type) = self.vault_type {
            push("vaultType", "Int", vault_type.into());
        }
        if let Some(risk_level) = self.risk_level {
            push("riskLevel", "Int", risk_level.into());
        }
        if let Some(ref addresses) = self.addresses {
            push("addresses", "[String]", addresses.clone().into());
        }

        if args.is_empty() {
            (String::new(), String::new(), serde_json::Value::Null)
        } else {
            (
                format!("({})", decls.join(", ")),
                format!("({})", args.join(", ")),
                serde_json::Value::Object(vars),
            )
        }
    }
}

/// Fields selected for `Vault`
const VAULT_FIELDS: &str = "address name symbol chainId apiVersion decimals v3 yearn erc4626 \
    isShutdown emergencyShutdown vaultType token totalAssets totalSupply pricePerShare \
    depositLimit availableDepositLimit managementFee performanceFee governance guardian \
    management rewards registry inceptTime inceptBlock lastReport activation projectId \
    projectName withdrawalQueue strategies \
    tvl { close blockTime } \
    apy { net weeklyNet monthlyNet inceptionNet grossApr blockTime } \
    fees { managementFee performanceFee } \
    risk { riskLevel } \
    meta { displayName description category isHidden isBoosted } \
    asset { address name symbol decimals }";

/// Query text and variables for `vaults(...)`
fn list_query(filter: &VaultFilter) -> (String, serde_json::Value) {
    let (decls, args, variables) = filter.build_args();
    let query = format!("query Vaults{decls} {{ vaults{args} {{ {VAULT_FIELDS} }} }}");
    (query, variables)
}

/// Query text and variables for `vault(chainId, address)`
fn get_query(chain_id: u64, address: &str) -> (String, serde_json::Value) {
    let query = format!(
        "query Vault($chainId: Int, $address: String) {{ \
         vault(chainId: $chainId, address: $address) {{ {VAULT_FIELDS} }} }}"
    );
    (
        query,
        serde_json::json!({ "chainId": chain_id, "address": address }),
    )
}

/// Vaults API
pub struct VaultsApi<'a> {
    client: &'a Client,
}

impl<'a> VaultsApi<'a> {
    /// Create a new vaults API instance
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all vaults (with optional filter)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// let vaults = client.vaults().list(None).await?;
    /// println!("Found {} vaults", vaults.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, filter: Option<VaultFilter>) -> Result<Vec<Vault>> {
        let (query, variables) = list_query(&filter.unwrap_or_default());

        #[derive(Deserialize)]
        struct Response {
            vaults: Vec<Vault>,
        }

        let response: Response = self.client.query_with_variables(&query, variables).await?;
        Ok(response.vaults)
    }

    /// Get vaults for a specific chain
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// // Get Ethereum mainnet vaults
    /// let vaults = client.vaults().by_chain(1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn by_chain(&self, chain_id: u64) -> Result<Vec<Vault>> {
        self.list(Some(VaultFilter::new().chain_id(chain_id))).await
    }

    /// Get only v3 vaults
    pub async fn v3_vaults(&self) -> Result<Vec<Vault>> {
        self.list(Some(VaultFilter::new().v3(true))).await
    }

    /// Get only Yearn official vaults
    pub async fn yearn_vaults(&self) -> Result<Vec<Vault>> {
        self.list(Some(VaultFilter::new().yearn(true))).await
    }

    /// Get a single vault by address and chain
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// let vault = client.vaults().get(1, "0x...").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, chain_id: u64, address: &str) -> Result<Option<Vault>> {
        let (query, variables) = get_query(chain_id, address);

        #[derive(Deserialize)]
        struct Response {
            vault: Option<Vault>,
        }

        let response: Response = self.client.query_with_variables(&query, variables).await?;
        Ok(response.vault)
    }

    /// Get vault accounts (user positions) for an address
    ///
    /// **DEPRECATED:** The Kong API removed user position queries in 2024.
    /// The old `vaultAccounts(chainId, address)` endpoint no longer exists.
    ///
    /// To get user vault balances, you must query on-chain via vault `balanceOf(user)` calls.
    /// Consider using Alchemy, Moralis, or direct RPC calls instead.
    #[deprecated(
        since = "0.1.2",
        note = "Kong API removed user position queries. Use on-chain balanceOf() calls instead."
    )]
    pub async fn accounts(&self, _chain_id: u64, _address: &str) -> Result<Vec<VaultAccount>> {
        // The Kong API changed in 2024:
        // - Old: vaultAccounts(chainId, address) returned user positions with balances
        // - New: vaultAccounts(chainId, vault) returns AccountRole (role holders, not depositors)
        // - accountVaults(chainId, account) returns vaults where user has ROLES, not deposits
        //
        // User position/balance data must now be fetched on-chain via balanceOf() calls.
        Err(crate::Error::Domain(
            crate::error::DomainError::ApiEndpointRemoved {
                endpoint: "vaultAccounts".to_string(),
                alternative: "Use on-chain balanceOf() calls or ethcli portfolio command"
                    .to_string(),
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_query_sends_filter_as_variables() {
        let filter = VaultFilter::new()
            .chain_id(1)
            .api_version("3.0.2\") { injected }")
            .v3(true)
            .addresses(vec!["0xabc".to_string()]);
        let (query, variables) = list_query(&filter);
        assert!(query.starts_with(
            "query Vaults($chainId: Int, $apiVersion: String, $v3: Boolean, $addresses: [String]) \
             { vaults(chainId: $chainId, apiVersion: $apiVersion, v3: $v3, addresses: $addresses) {"
        ));
        assert!(!query.contains("injected"));
        assert_eq!(
            variables,
            serde_json::json!({
                "chainId": 1,
                "apiVersion": "3.0.2\") { injected }",
                "v3": true,
                "addresses": ["0xabc"],
            })
        );
    }

    #[test]
    fn list_query_without_filter_has_no_variables() {
        let (query, variables) = list_query(&VaultFilter::new());
        assert!(query.starts_with("query Vaults { vaults {"), "{query}");
        assert!(variables.is_null());
    }

    #[test]
    fn get_query_uses_variables() {
        let (query, variables) = get_query(10, "0x\"bad");
        assert!(query.contains("vault(chainId: $chainId, address: $address)"));
        assert!(!query.contains("bad"));
        assert_eq!(
            variables,
            serde_json::json!({ "chainId": 10, "address": "0x\"bad" })
        );
    }
}
