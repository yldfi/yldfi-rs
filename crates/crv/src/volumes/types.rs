//! Types for the Volumes and APYs API

use serde::{Deserialize, Serialize};

/// Response for gauge data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugesResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Gauge data
    pub data: serde_json::Value,
}

/// Response for volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumesResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Volume data
    pub data: VolumesData,
}

/// Volume data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumesData {
    /// Pool volumes
    pub pools: Option<Vec<PoolVolume>>,
    /// Chain-wide volume totals
    #[serde(default)]
    pub total_volumes: Option<TotalVolumes>,
}

/// Chain-wide volume totals (`data.totalVolumes`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalVolumes {
    /// Total stable-pool volume (USD, 24h)
    #[serde(default)]
    pub total_stable_volume: Option<f64>,
    /// Total crypto-pool volume (USD, 24h)
    #[serde(default)]
    pub total_crypto_volume: Option<f64>,
    /// Total volume (USD, 24h)
    #[serde(default)]
    pub total_volume: Option<f64>,
    /// Crypto share of volume (percent)
    #[serde(default)]
    pub crypto_volume_share_pcent: Option<f64>,
}

/// Volume data for a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolVolume {
    /// Pool address
    pub address: String,
    /// Pool registry type (e.g. "main", "factory-crypto")
    #[serde(default, rename = "type")]
    pub pool_type: Option<String>,
    /// 24h volume in USD (API field `volumeUSD`)
    #[serde(default, rename = "volumeUSD")]
    pub volume_usd: Option<f64>,
    /// Latest daily base APY, in percent
    #[serde(default)]
    pub latest_daily_apy_pcent: Option<f64>,
    /// Latest weekly base APY, in percent
    #[serde(default)]
    pub latest_weekly_apy_pcent: Option<f64>,
    /// APY contribution from LSTs, in percent
    #[serde(default)]
    pub included_apy_pcent_from_lsts: Option<f64>,
    /// Pool virtual price (raw, 1e18-scaled)
    #[serde(default)]
    pub virtual_price: Option<serde_json::Value>,
}

/// Response for base APYs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseApysResponse {
    /// Whether the request was successful
    pub success: bool,
    /// APY data
    pub data: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volumes_parse_live_field_names() {
        let json = r#"{"success":true,"data":{"pools":[{"address":"0xbEbc","type":"main",
            "volumeUSD":8279244.44,"latestDailyApyPcent":1.5,"latestWeeklyApyPcent":2.5,
            "includedApyPcentFromLsts":0,"virtualPrice":1039823717404894000}],
            "totalVolumes":{"totalStableVolume":75663374.71,"totalCryptoVolume":8295841.73,
            "totalVolume":83959216.44,"cryptoVolumeSharePcent":9.88}}}"#;
        let r: VolumesResponse = serde_json::from_str(json).unwrap();
        let p = &r.data.pools.as_ref().unwrap()[0];
        assert_eq!(p.volume_usd, Some(8_279_244.44));
        assert_eq!(p.latest_daily_apy_pcent, Some(1.5));
        assert_eq!(p.latest_weekly_apy_pcent, Some(2.5));
        assert_eq!(
            r.data.total_volumes.as_ref().unwrap().total_volume,
            Some(83_959_216.44)
        );
        let out = serde_json::to_value(&r).unwrap();
        assert_eq!(out["data"]["pools"][0]["volumeUSD"], 8_279_244.44);
    }
}
