//! Full transaction fees on rollups: L2 execution fee plus L1 data fee
//!
//! - OP-stack chains (Optimism, Base, ...) charge the L1 data fee on top of
//!   `gasUsed × effectiveGasPrice`. Receipts carry it as `l1Fee`, along with
//!   the inputs it was computed from (`l1GasUsed`, `l1GasPrice`,
//!   `l1BaseFeeScalar`, `l1BlobBaseFee`, `l1BlobBaseFeeScalar`).
//! - Arbitrum folds the L1 portion into `gasUsed`; receipts report it as
//!   `gasUsedForL1`, so the total is still `gasUsed × effectiveGasPrice` and
//!   the split is `gasUsedForL1 × price` (L1) and the rest (execution).
//! - On L1 chains there is no L1 data fee: it is reported as not applicable
//!   (`null`), not zero.

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// GasPriceOracle predeploy on OP-stack chains
pub const OP_GAS_PRICE_ORACLE: Address =
    alloy::primitives::address!("420000000000000000000000000000000000000F");
/// NodeInterface precompile on Arbitrum chains
pub const ARB_NODE_INTERFACE: Address =
    alloy::primitives::address!("00000000000000000000000000000000000000C8");

/// How a chain charges for posting data to L1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeModel {
    /// No L1 data fee (L1 chains and sidechains)
    L1,
    /// OP-stack: separate `l1Fee` charged on top of execution
    OpStack,
    /// Arbitrum Nitro: L1 portion included in `gasUsed`
    Arbitrum,
}

impl FeeModel {
    pub fn for_chain_id(chain_id: u64) -> Self {
        match chain_id {
            // Optimism, Base, Zora, Mode, Fraxtal, Unichain, World Chain,
            // Ink, Soneium, BOB, Lisk, Blast
            10 | 8453 | 7777777 | 34443 | 252 | 130 | 480 | 57073 | 1868 | 60808 | 1135 | 81457 => {
                FeeModel::OpStack
            }
            // Arbitrum One, Arbitrum Nova
            42161 | 42170 => FeeModel::Arbitrum,
            _ => FeeModel::L1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            FeeModel::L1 => "l1",
            FeeModel::OpStack => "op-stack",
            FeeModel::Arbitrum => "arbitrum",
        }
    }

    pub fn is_l2(&self) -> bool {
        !matches!(self, FeeModel::L1)
    }
}

/// A fee split into execution and L1 data parts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeBreakdown {
    /// "l1", "op-stack" or "arbitrum"
    pub model: String,
    /// L2 execution fee in wei
    pub execution_fee_wei: String,
    /// L1 data fee in wei; `null` when not applicable (L1 chain)
    pub l1_fee_wei: Option<String>,
    /// Execution plus L1 data fee, in wei
    pub total_fee_wei: String,
    /// The same three in the native token
    pub execution_fee: String,
    pub l1_fee: Option<String>,
    pub total_fee: String,
    /// L2 gas units (on Arbitrum, excluding `l1_gas_used`)
    pub execution_gas: u64,
    /// Price per L2 gas unit in wei
    pub gas_price_wei: String,
    /// OP-stack: L1 gas units for the data. Arbitrum: L2 gas units charged
    /// for the L1 data (`gasUsedForL1`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_gas_used: Option<u64>,
    /// OP-stack: L1 base fee used for the data fee, in wei
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_gas_price_wei: Option<String>,
    /// OP-stack: L1 blob base fee, in wei
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_blob_base_fee_wei: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_base_fee_scalar: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub l1_blob_base_fee_scalar: Option<u64>,
    /// Fees in USD, when a native token price was available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usd: Option<FeeUsd>,
}

/// USD value of a [`FeeBreakdown`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeUsd {
    /// Native token price used
    pub price_usd: f64,
    /// Where the price came from
    pub source: String,
    pub execution_usd: f64,
    pub l1_usd: Option<f64>,
    pub total_usd: f64,
}

/// Parse a JSON-RPC quantity (hex string, or a number)
pub fn quantity(v: Option<&Value>) -> Option<U256> {
    match v? {
        Value::String(s) => {
            let s = s.trim();
            match s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
                Some("") => Some(U256::ZERO),
                Some(hex) => U256::from_str_radix(hex, 16).ok(),
                None => U256::from_str_radix(s, 10).ok(),
            }
        }
        Value::Number(n) => n.as_u64().map(U256::from),
        _ => None,
    }
}

fn to_u64(v: Option<U256>) -> Option<u64> {
    v.and_then(|x| u64::try_from(x).ok())
}

/// Wei to the native token with 18 decimals
pub fn format_native(wei: U256) -> String {
    alloy::primitives::utils::format_units(wei, 18).unwrap_or_else(|_| wei.to_string())
}

/// Lossy wei to native-token float, for USD values
pub fn wei_to_f64(wei: U256) -> f64 {
    format_native(wei).parse::<f64>().unwrap_or(0.0)
}

impl FeeBreakdown {
    /// Build from parts; `l1_fee` `None` means not applicable
    pub fn new(
        model: FeeModel,
        execution_gas: u64,
        gas_price: U256,
        execution_fee: U256,
        l1_fee: Option<U256>,
    ) -> Self {
        let total = execution_fee.saturating_add(l1_fee.unwrap_or(U256::ZERO));
        FeeBreakdown {
            model: model.name().to_string(),
            execution_fee_wei: execution_fee.to_string(),
            l1_fee_wei: l1_fee.map(|f| f.to_string()),
            total_fee_wei: total.to_string(),
            execution_fee: format_native(execution_fee),
            l1_fee: l1_fee.map(format_native),
            total_fee: format_native(total),
            execution_gas,
            gas_price_wei: gas_price.to_string(),
            l1_gas_used: None,
            l1_gas_price_wei: None,
            l1_blob_base_fee_wei: None,
            l1_base_fee_scalar: None,
            l1_blob_base_fee_scalar: None,
            usd: None,
        }
    }

    /// Split the fee of a mined transaction from its raw JSON receipt.
    ///
    /// Returns `None` when `gasUsed` or `effectiveGasPrice` is missing.
    pub fn from_receipt_json(chain_id: u64, receipt: &Value) -> Option<Self> {
        let model = FeeModel::for_chain_id(chain_id);
        let gas_used = quantity(receipt.get("gasUsed"))?;
        let price = quantity(receipt.get("effectiveGasPrice"))?;
        let gas_used_u64 = u64::try_from(gas_used).ok()?;
        match model {
            FeeModel::L1 => Some(Self::new(
                model,
                gas_used_u64,
                price,
                gas_used.saturating_mul(price),
                None,
            )),
            FeeModel::OpStack => {
                // Deposit transactions carry no l1Fee: they pay nothing on L2
                let l1_fee = quantity(receipt.get("l1Fee")).unwrap_or(U256::ZERO);
                let mut b = Self::new(
                    model,
                    gas_used_u64,
                    price,
                    gas_used.saturating_mul(price),
                    Some(l1_fee),
                );
                b.l1_gas_used = to_u64(quantity(receipt.get("l1GasUsed")));
                b.l1_gas_price_wei = quantity(receipt.get("l1GasPrice")).map(|v| v.to_string());
                b.l1_blob_base_fee_wei =
                    quantity(receipt.get("l1BlobBaseFee")).map(|v| v.to_string());
                b.l1_base_fee_scalar = to_u64(quantity(receipt.get("l1BaseFeeScalar")));
                b.l1_blob_base_fee_scalar = to_u64(quantity(receipt.get("l1BlobBaseFeeScalar")));
                Some(b)
            }
            FeeModel::Arbitrum => {
                let for_l1 = quantity(receipt.get("gasUsedForL1"))
                    .unwrap_or(U256::ZERO)
                    .min(gas_used);
                let exec_gas = gas_used - for_l1;
                let mut b = Self::new(
                    model,
                    u64::try_from(exec_gas).ok()?,
                    price,
                    exec_gas.saturating_mul(price),
                    Some(for_l1.saturating_mul(price)),
                );
                b.l1_gas_used = to_u64(Some(for_l1));
                Some(b)
            }
        }
    }

    pub fn total_wei(&self) -> U256 {
        U256::from_str_radix(&self.total_fee_wei, 10).unwrap_or(U256::ZERO)
    }

    fn wei(s: &str) -> U256 {
        U256::from_str_radix(s, 10).unwrap_or(U256::ZERO)
    }

    /// Attach USD values at `price_usd` per native token
    pub fn set_usd(&mut self, price_usd: f64, source: impl Into<String>) {
        let usd = |s: &str| wei_to_f64(Self::wei(s)) * price_usd;
        self.usd = Some(FeeUsd {
            price_usd,
            source: source.into(),
            execution_usd: usd(&self.execution_fee_wei),
            l1_usd: self.l1_fee_wei.as_deref().map(usd),
            total_usd: usd(&self.total_fee_wei),
        });
    }

    /// Human-readable lines, e.g. for `tx` and `gas cost`
    pub fn lines(&self, symbol: &str) -> Vec<String> {
        let usd = |v: Option<f64>| match v {
            Some(v) => format!(" (${})", format_usd(v)),
            None => String::new(),
        };
        let u = self.usd.as_ref();
        let mut out = vec![format!(
            "  Execution: {} {symbol}{}  [{} gas × {} wei]",
            self.execution_fee,
            usd(u.map(|u| u.execution_usd)),
            self.execution_gas,
            self.gas_price_wei
        )];
        match &self.l1_fee {
            Some(l1) => {
                let detail = match (self.model.as_str(), self.l1_gas_used) {
                    ("arbitrum", Some(g)) => format!("  [{g} L2 gas for L1 data]"),
                    (_, Some(g)) => format!("  [{g} L1 gas]"),
                    _ => String::new(),
                };
                out.push(format!(
                    "  L1 data:   {l1} {symbol}{}{detail}",
                    usd(u.and_then(|u| u.l1_usd))
                ));
            }
            None => out.push("  L1 data:   n/a (not a rollup)".to_string()),
        }
        out.push(format!(
            "  Total:     {} {symbol}{}",
            self.total_fee,
            usd(u.map(|u| u.total_usd))
        ));
        if let Some(u) = u {
            out.push(format!(
                "  Price:     ${} per {symbol} ({})",
                format_usd(u.price_usd),
                u.source
            ));
        }
        out
    }
}

/// USD with enough digits for sub-cent fees
pub fn format_usd(v: f64) -> String {
    if v.abs() >= 1.0 {
        format!("{v:.2}")
    } else if v.abs() >= 0.01 {
        format!("{v:.4}")
    } else {
        format!("{v:.6}")
    }
}

/// Native token USD price for fee conversion.
///
/// With `at` (Unix time) the DefiLlama historical price at that time;
/// otherwise the median of the current-price aggregation. Returns the price
/// and a description of its source.
pub async fn native_usd_price(
    chain: crate::config::Chain,
    at: Option<u64>,
) -> Result<(f64, String), String> {
    use crate::aggregator::price::{fetch_prices_parallel, symbol_to_coingecko_id};
    use crate::aggregator::PriceSource;

    let symbol = chain.native_symbol();
    if symbol == "???" {
        return Err(format!(
            "unknown native token for chain {}",
            chain.chain_id()
        ));
    }
    if let Some(at) = at {
        let token = dllma::coins::Token::coingecko(symbol_to_coingecko_id(symbol));
        let client =
            crate::aggregator::price::get_llama_client().map_err(|e| format!("DefiLlama: {e}"))?;
        let resp = client
            .coins()
            .historical_with_search_width(at, std::slice::from_ref(&token), "6h")
            .await
            .map_err(|e| format!("DefiLlama historical price: {e}"))?;
        let coin = resp
            .coins
            .get(&token.format())
            .ok_or_else(|| format!("DefiLlama has no {symbol} price near the block time"))?;
        return Ok((coin.price, "DefiLlama, at block time".to_string()));
    }
    let sources = [
        PriceSource::Gecko,
        PriceSource::Llama,
        PriceSource::Chainlink,
        PriceSource::Pyth,
        PriceSource::Cex,
    ];
    let result = fetch_prices_parallel(symbol, "ethereum", &sources).await;
    let ok: Vec<String> = result
        .sources
        .iter()
        .filter(|r| r.data.is_some())
        .map(|r| r.source.to_string())
        .collect();
    let median = result.aggregated.median_usd;
    if ok.is_empty() || median <= 0.0 {
        return Err(format!("no {symbol} price source answered"));
    }
    Ok((median, format!("median of {}", ok.join(", "))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn model_by_chain() {
        assert_eq!(FeeModel::for_chain_id(1), FeeModel::L1);
        assert_eq!(FeeModel::for_chain_id(10), FeeModel::OpStack);
        assert_eq!(FeeModel::for_chain_id(8453), FeeModel::OpStack);
        assert_eq!(FeeModel::for_chain_id(42161), FeeModel::Arbitrum);
        assert!(!FeeModel::for_chain_id(137).is_l2());
    }

    #[test]
    fn quantities() {
        assert_eq!(quantity(Some(&json!("0x10"))), Some(U256::from(16)));
        assert_eq!(quantity(Some(&json!("0x"))), Some(U256::ZERO));
        assert_eq!(quantity(Some(&json!(7))), Some(U256::from(7)));
        assert_eq!(quantity(Some(&json!("12"))), Some(U256::from(12)));
        assert_eq!(quantity(None), None);
        assert_eq!(quantity(Some(&json!(null))), None);
    }

    #[test]
    fn op_stack_receipt_adds_l1_fee() {
        // A live Base receipt (fields as returned by the node)
        let r = json!({
            "gasUsed": "0x45b17", "effectiveGasPrice": "0x30a32c0",
            "l1GasPrice": "0x5984c1f", "l1GasUsed": "0xa68", "l1Fee": "0x5cc1dd7d",
            "l1BaseFeeScalar": "0x8dd", "l1BlobBaseFee": "0x55cd2e",
            "l1BlobBaseFeeScalar": "0x101c12"
        });
        let b = FeeBreakdown::from_receipt_json(8453, &r).unwrap();
        let exec = 0x45b17u128 * 0x30a32c0u128;
        let l1 = 0x5cc1dd7du128;
        assert_eq!(b.model, "op-stack");
        assert_eq!(b.execution_fee_wei, exec.to_string());
        assert_eq!(b.l1_fee_wei, Some(l1.to_string()));
        assert_eq!(b.total_fee_wei, (exec + l1).to_string());
        assert_eq!(b.l1_gas_used, Some(0xa68));
        assert_eq!(b.l1_base_fee_scalar, Some(0x8dd));
        assert_eq!(b.l1_blob_base_fee_scalar, Some(0x101c12));
        assert_eq!(b.l1_gas_price_wei, Some(0x5984c1fu64.to_string()));
    }

    #[test]
    fn op_stack_deposit_without_l1_fee() {
        let r = json!({"gasUsed": "0xb4e5", "effectiveGasPrice": "0x0"});
        let b = FeeBreakdown::from_receipt_json(10, &r).unwrap();
        assert_eq!(b.total_fee_wei, "0");
        assert_eq!(b.l1_fee_wei.as_deref(), Some("0"));
    }

    #[test]
    fn arbitrum_receipt_splits_gas_used() {
        let r = json!({
            "gasUsed": "0x13077", "effectiveGasPrice": "0x1312d00",
            "gasUsedForL1": "0x25a"
        });
        let b = FeeBreakdown::from_receipt_json(42161, &r).unwrap();
        let price = 0x1312d00u128;
        assert_eq!(b.model, "arbitrum");
        assert_eq!(b.execution_gas, 0x13077 - 0x25a);
        assert_eq!(b.l1_fee_wei, Some((0x25au128 * price).to_string()));
        // The total is gasUsed × price: the L1 part is not added on top
        assert_eq!(b.total_fee_wei, (0x13077u128 * price).to_string());
        assert_eq!(b.l1_gas_used, Some(0x25a));
    }

    #[test]
    fn l1_chain_has_no_l1_fee() {
        let r = json!({"gasUsed": "0x5208", "effectiveGasPrice": "0x4a817c800"});
        let b = FeeBreakdown::from_receipt_json(1, &r).unwrap();
        assert_eq!(b.model, "l1");
        assert_eq!(b.l1_fee_wei, None);
        assert_eq!(b.l1_fee, None);
        assert_eq!(b.total_fee_wei, "420000000000000");
        let v = serde_json::to_value(&b).unwrap();
        assert!(v["l1_fee_wei"].is_null());
        assert!(b.lines("ETH").iter().any(|l| l.contains("n/a")));
    }

    #[test]
    fn missing_fields_give_none() {
        assert!(FeeBreakdown::from_receipt_json(10, &json!({"gasUsed": "0x1"})).is_none());
    }

    #[test]
    fn usd_values() {
        let mut b = FeeBreakdown::new(
            FeeModel::OpStack,
            100_000,
            U256::from(1_000_000u64),
            U256::from(100_000_000_000u64),             // 1e-7 ETH
            Some(U256::from(1_000_000_000_000_000u64)), // 0.001 ETH
        );
        b.set_usd(3000.0, "test");
        let u = b.usd.as_ref().unwrap();
        assert!((u.execution_usd - 0.0003).abs() < 1e-9);
        assert!((u.l1_usd.unwrap() - 3.0).abs() < 1e-9);
        assert!((u.total_usd - 3.0003).abs() < 1e-9);
        assert_eq!(b.total_fee, "0.001000100000000000");
        assert_eq!(format_usd(3.0003), "3.00");
        assert_eq!(format_usd(0.0003), "0.000300");
    }
}
