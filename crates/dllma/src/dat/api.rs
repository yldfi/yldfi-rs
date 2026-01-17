//! DAT API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::*;

/// DAT (Digital Asset Treasury) API client (Pro only)
pub struct DatApi<'a> {
    client: &'a Client,
}

impl<'a> DatApi<'a> {
    /// Create a new DAT API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all institutions with Digital Asset Treasury data
    ///
    /// **Requires Pro API key**
    ///
    /// Returns comprehensive data about institutions holding digital assets,
    /// including mNAV calculations (realized, realistic, maximum).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let data = client.dat().institutions().await?;
    /// println!("Tracking {} companies", data.total_companies.unwrap_or(0));
    /// for (ticker, meta) in data.institution_metadata.iter().take(5) {
    ///     println!("{}: ${:.0}M in crypto",
    ///         ticker,
    ///         meta.total_usd_value.unwrap_or(0.0) / 1_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn institutions(&self) -> Result<DatInstitutionsResponse> {
        self.client.get_pro("/dat/institutions").await
    }

    /// Get detailed DAT data for a specific institution
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `symbol` - Institution ticker symbol (e.g., "MSTR" for MicroStrategy)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let mstr = client.dat().institution("MSTR").await?;
    /// println!("{}: ${:.0}M in crypto holdings",
    ///     mstr.name.as_deref().unwrap_or("Unknown"),
    ///     mstr.total_usd_value.unwrap_or(0.0) / 1_000_000.0);
    /// if let Some(mnav) = mstr.realized_m_nav {
    ///     println!("Realized mNAV: {:.2}x", mnav);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn institution(&self, symbol: &str) -> Result<InstitutionDetail> {
        let path = format!("/dat/institutions/{}", symbol);
        self.client.get_pro(&path).await
    }
}
