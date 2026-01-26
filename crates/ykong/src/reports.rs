//! Vault and strategy report queries from Kong API

use crate::client::Client;
use crate::error::Result;
use crate::types::{StrategyReport, VaultReport};
use serde::Deserialize;

/// Reports API for vault and strategy performance data
pub struct ReportsApi<'a> {
    client: &'a Client,
}

impl<'a> ReportsApi<'a> {
    /// Create a new reports API instance
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get vault reports (harvest events)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// let reports = client.reports().vault_reports(1, "0x...").await?;
    /// for report in reports {
    ///     if let Some(gain) = report.gain_usd {
    ///         println!("Harvest gain: ${:.2}", gain);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn vault_reports(&self, chain_id: u64, address: &str) -> Result<Vec<VaultReport>> {
        let query = format!(
            r#"{{
                vaultReports(chainId: {}, address: "{}") {{
                    chainId
                    address
                    eventName
                    strategy
                    gain
                    loss
                    debtPaid
                    totalGain
                    totalLoss
                    totalDebt
                    debtAdded
                    debtRatio
                    currentDebt
                    protocolFees
                    totalFees
                    totalRefunds
                    gainUsd
                    lossUsd
                    debtPaidUsd
                    totalGainUsd
                    totalLossUsd
                    totalDebtUsd
                    debtAddedUsd
                    currentDebtUsd
                    protocolFeesUsd
                    totalFeesUsd
                    totalRefundsUsd
                    priceUsd
                    priceSource
                    apr {{ gross net forward }}
                    blockNumber
                    blockTime
                    logIndex
                    transactionHash
                }}
            }}"#,
            chain_id, address
        );

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            vault_reports: Vec<VaultReport>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.vault_reports)
    }

    /// Get strategy reports (harvest events)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> ykong::error::Result<()> {
    /// use ykong::Client;
    ///
    /// let client = Client::new()?;
    /// let reports = client.reports().strategy_reports(1, "0x...").await?;
    /// for report in reports {
    ///     if let Some(profit) = report.profit_usd {
    ///         println!("Strategy profit: ${:.2}", profit);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn strategy_reports(
        &self,
        chain_id: u64,
        address: &str,
    ) -> Result<Vec<StrategyReport>> {
        let query = format!(
            r#"{{
                strategyReports(chainId: {}, address: "{}") {{
                    chainId
                    address
                    eventName
                    profit
                    loss
                    debtPayment
                    debtOutstanding
                    protocolFees
                    performanceFees
                    apr {{ gross net forward }}
                    profitUsd
                    lossUsd
                    debtPaymentUsd
                    debtOutstandingUsd
                    protocolFeesUsd
                    performanceFeesUsd
                    priceUsd
                    priceSource
                    blockNumber
                    blockTime
                    logIndex
                    transactionHash
                }}
            }}"#,
            chain_id, address
        );

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Response {
            strategy_reports: Vec<StrategyReport>,
        }

        let response: Response = self.client.query(&query).await?;
        Ok(response.strategy_reports)
    }

    /// Get the latest vault report
    pub async fn latest_vault_report(
        &self,
        chain_id: u64,
        address: &str,
    ) -> Result<Option<VaultReport>> {
        let reports = self.vault_reports(chain_id, address).await?;
        Ok(reports.into_iter().next())
    }

    /// Get the latest strategy report
    pub async fn latest_strategy_report(
        &self,
        chain_id: u64,
        address: &str,
    ) -> Result<Option<StrategyReport>> {
        let reports = self.strategy_reports(chain_id, address).await?;
        Ok(reports.into_iter().next())
    }

    /// Calculate total gains from vault reports
    pub async fn vault_total_gains_usd(&self, chain_id: u64, address: &str) -> Result<f64> {
        let reports = self.vault_reports(chain_id, address).await?;
        let total = reports
            .iter()
            .filter_map(|r| r.gain_usd)
            .sum();
        Ok(total)
    }

    /// Calculate total gains from strategy reports
    pub async fn strategy_total_profits_usd(&self, chain_id: u64, address: &str) -> Result<f64> {
        let reports = self.strategy_reports(chain_id, address).await?;
        let total = reports
            .iter()
            .filter_map(|r| r.profit_usd)
            .sum();
        Ok(total)
    }
}
