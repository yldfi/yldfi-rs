//! Etherscan API chain support checks
//!
//! Etherscan's multichain (V2) API has dropped support for some chains that
//! ethcli still supports over RPC. Requests for those chains are rejected
//! locally with a clear message instead of making a request that is bound to
//! fail with an opaque error.
//!
//! Source: <https://docs.etherscan.io/changelog>

use chrono::{NaiveDate, Utc};

use crate::error::AbiError;

/// A chain whose Etherscan API support has ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EtherscanSunset {
    /// EIP-155 chain ID
    pub chain_id: u64,
    /// Human-readable chain name
    pub name: &'static str,
    /// Date (YYYY-MM-DD) from which Etherscan no longer serves the chain
    pub effective: &'static str,
}

/// Chains for which the Etherscan API no longer works.
pub const ETHERSCAN_SUNSET_CHAINS: &[EtherscanSunset] = &[
    EtherscanSunset {
        chain_id: 1923,
        name: "Swell",
        effective: "2026-02-25",
    },
    EtherscanSunset {
        chain_id: 1924,
        name: "Swell Testnet",
        effective: "2026-02-25",
    },
    EtherscanSunset {
        chain_id: 534_352,
        name: "Scroll",
        effective: "2026-04-16",
    },
    EtherscanSunset {
        chain_id: 534_351,
        name: "Scroll Sepolia",
        effective: "2026-04-16",
    },
    EtherscanSunset {
        chain_id: 1284,
        name: "Moonbeam",
        effective: "2026-07-31",
    },
    EtherscanSunset {
        chain_id: 1285,
        name: "Moonriver",
        effective: "2026-07-31",
    },
    EtherscanSunset {
        chain_id: 1287,
        name: "Moonbase Alpha",
        effective: "2026-07-31",
    },
    EtherscanSunset {
        chain_id: 204,
        name: "opBNB",
        effective: "2026-10-12",
    },
    EtherscanSunset {
        chain_id: 5611,
        name: "opBNB Testnet",
        effective: "2026-10-12",
    },
    EtherscanSunset {
        chain_id: 33_139,
        name: "ApeChain",
        effective: "2026-10-31",
    },
    EtherscanSunset {
        chain_id: 33_111,
        name: "ApeChain Curtis",
        effective: "2026-10-31",
    },
    EtherscanSunset {
        chain_id: 81_457,
        name: "Blast",
        effective: "2026-10-31",
    },
    EtherscanSunset {
        chain_id: 168_587_773,
        name: "Blast Sepolia",
        effective: "2026-10-31",
    },
];

impl EtherscanSunset {
    /// Whether the sunset has taken effect on `today`.
    pub fn is_effective(&self, today: NaiveDate) -> bool {
        // Entries are compile-time constants covered by a test, so a parse
        // failure can only mean a typo; treat it as already effective.
        NaiveDate::parse_from_str(self.effective, "%Y-%m-%d").map_or(true, |d| today >= d)
    }
}

/// Return the sunset entry for `chain_id` if the Etherscan API no longer
/// supports it.
pub fn etherscan_sunset(chain_id: u64) -> Option<&'static EtherscanSunset> {
    etherscan_sunset_on(chain_id, Utc::now().date_naive())
}

/// Like [`etherscan_sunset`], evaluated for a given (UTC) date. Announced
/// sunsets with a future date are ignored until that date.
pub fn etherscan_sunset_on(chain_id: u64, today: NaiveDate) -> Option<&'static EtherscanSunset> {
    ETHERSCAN_SUNSET_CHAINS
        .iter()
        .find(|c| c.chain_id == chain_id && c.is_effective(today))
}

/// Fail fast if the Etherscan API does not support `chain_id`.
///
/// RPC-based functionality for these chains is unaffected; only Etherscan
/// API calls (ABIs, source code, account history, gas oracle, ...) are
/// rejected.
pub fn ensure_etherscan_supported(chain_id: u64) -> Result<(), AbiError> {
    match etherscan_sunset(chain_id) {
        Some(s) => Err(AbiError::EtherscanUnsupportedChain(format!(
            "{} (chain ID {}): Etherscan ended API support on {} \
             (https://docs.etherscan.io/changelog). RPC-based commands still work for this chain; \
             use the chain's own explorer or an RPC endpoint for this data",
            s.name, s.chain_id, s.effective
        ))),
        None => Ok(()),
    }
}

/// Build a readable error string from an Etherscan `status != "1"` response,
/// calling out plan/tier restrictions explicitly.
pub fn describe_etherscan_error(message: &str, result: &str) -> String {
    let lower = result.to_ascii_lowercase();
    let plan_restricted = lower.contains("free api access is not supported")
        || lower.contains("upgrade your api plan")
        || lower.contains("api pro")
        || lower.contains("requires a lite plan")
        || lower.contains("not available on the free tier")
        || (lower.contains("free tier") && lower.contains("unavailable"));
    let shared_pool_exhausted = lower.contains("community free api limit");

    if shared_pool_exhausted {
        format!(
            "Etherscan free-tier pool exhausted: {result} \
             (free calls on some chains, including Arbitrum from 2026-11-01, share one pool \
             across all free users; wait for the reset or use a paid key, see https://etherscan.io/apis)"
        )
    } else if plan_restricted {
        format!(
            "Etherscan plan restriction: {result} \
             (your API key's plan does not cover this chain/endpoint; see https://etherscan.io/apis)"
        )
    } else {
        format!("{message}: {result}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sunset_chains_fail_fast() {
        for id in [534_352u64, 1284, 1285, 204] {
            let err = ensure_etherscan_supported(id).unwrap_err().to_string();
            assert!(err.contains("Etherscan"), "{err}");
            assert!(err.contains(&id.to_string()), "{err}");
        }
    }

    #[test]
    fn sunset_dates_parse_and_gate() {
        for c in ETHERSCAN_SUNSET_CHAINS {
            assert!(
                NaiveDate::parse_from_str(c.effective, "%Y-%m-%d").is_ok(),
                "bad date for {}: {}",
                c.name,
                c.effective
            );
        }
        let before = NaiveDate::from_ymd_opt(2026, 10, 30).unwrap();
        let on = NaiveDate::from_ymd_opt(2026, 10, 31).unwrap();
        assert!(etherscan_sunset_on(81_457, before).is_none());
        assert!(etherscan_sunset_on(81_457, on).is_some());
        assert!(etherscan_sunset_on(33_139, on).is_some());
        assert!(etherscan_sunset_on(204, before).is_some());
        assert!(etherscan_sunset_on(1, on).is_none());
    }

    #[test]
    fn supported_chains_pass() {
        for id in [1u64, 10, 137, 8453, 42161, 59144] {
            assert!(ensure_etherscan_supported(id).is_ok());
        }
    }

    #[test]
    fn plan_restriction_is_called_out() {
        let msg = describe_etherscan_error(
            "NOTOK",
            "Free API access is not supported for this chain. Please upgrade your api plan for full chain coverage. https://etherscan.io/apis",
        );
        assert!(msg.starts_with("Etherscan plan restriction"), "{msg}");

        let msg = describe_etherscan_error(
            "NOTOK",
            "Community Free API limit reached. Resets 2026-11-02 12:00:00 UTC.",
        );
        assert!(msg.contains("pool exhausted"), "{msg}");
        assert!(msg.contains("Arbitrum"), "{msg}");

        let msg = describe_etherscan_error("NOTOK", "Invalid address format");
        assert_eq!(msg, "NOTOK: Invalid address format");
    }
}
