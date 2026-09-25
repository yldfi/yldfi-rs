//! On-chain token metadata helpers (decimals lookup via RPC)

use crate::config::Chain;
use alloy::primitives::Address;
use alloy::providers::{Provider, ProviderBuilder};

/// Placeholder addresses aggregators use for the chain's native token
const NATIVE_PLACEHOLDERS: [&str; 2] = [
    "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
    "0x0000000000000000000000000000000000000000",
];

/// Whether `token` is a native-token placeholder address
#[must_use]
pub fn is_native_placeholder(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    NATIVE_PLACEHOLDERS.contains(&lower.as_str())
}

/// Fetch ERC20 `decimals()` for `token` on `chain_id` via the configured RPC.
///
/// Native-token placeholders resolve to 18 without an RPC call.
///
/// # Errors
///
/// Returns an error if the address is invalid, no RPC endpoint is configured
/// for the chain, or the call fails / returns malformed data.
pub async fn fetch_token_decimals(chain_id: u64, token: &str) -> anyhow::Result<u8> {
    if is_native_placeholder(token) {
        return Ok(18);
    }
    let address: Address = token
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid token address '{token}': {e}"))?;
    let chain = Chain::from_chain_id(chain_id);
    let url = crate::rpc::selector::get_rpc_url(chain)?;
    let provider = ProviderBuilder::new().connect_http(
        url.parse()
            .map_err(|_| anyhow::anyhow!("Invalid RPC URL configured for chain {chain_id}"))?,
    );
    // decimals() selector
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(alloy::primitives::Bytes::from_static(&[0x31, 0x3c, 0xe5, 0x67]).into());
    // Retry briefly: this often runs alongside many parallel aggregator
    // requests and shared RPC endpoints may rate-limit (HTTP 429).
    let mut attempt = 0u64;
    let result = loop {
        match provider.call(tx.clone()).await {
            Ok(r) => break r,
            Err(e) if attempt >= 2 => {
                anyhow::bail!("decimals() call failed for {token}: {e}");
            }
            Err(_) => {
                attempt += 1;
                tokio::time::sleep(std::time::Duration::from_millis(300 * attempt)).await;
            }
        }
    };
    decode_decimals(&result)
        .ok_or_else(|| anyhow::anyhow!("decimals() returned malformed data for {token}"))
}

/// Decode an ABI-encoded `uint8` return value.
fn decode_decimals(data: &[u8]) -> Option<u8> {
    if data.len() < 32 || data[..31].iter().any(|b| *b != 0) {
        return None;
    }
    Some(data[31])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_placeholders_are_detected() {
        assert!(is_native_placeholder(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE"
        ));
        assert!(is_native_placeholder(
            "0x0000000000000000000000000000000000000000"
        ));
        assert!(!is_native_placeholder(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        ));
    }

    #[test]
    fn decode_decimals_validates_encoding() {
        let mut word = [0u8; 32];
        word[31] = 6;
        assert_eq!(decode_decimals(&word), Some(6));
        word[0] = 1;
        assert_eq!(decode_decimals(&word), None);
        assert_eq!(decode_decimals(&[6]), None);
    }

    #[tokio::test]
    async fn native_decimals_need_no_rpc() {
        assert_eq!(
            fetch_token_decimals(42161, "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE")
                .await
                .unwrap(),
            18
        );
    }
}
