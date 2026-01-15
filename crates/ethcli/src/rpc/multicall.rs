//! Multicall3 support for batching multiple eth_call requests
//!
//! Uses the Multicall3 contract (0xcA11bde05977b3631167028862bE2a173976CA11)
//! which is deployed on all major EVM chains at the same address.

use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::Provider;
use alloy::sol;
use alloy::sol_types::SolCall;

/// Multicall3 contract address (same on all chains)
pub const MULTICALL3_ADDRESS: Address =
    alloy::primitives::address!("cA11bde05977b3631167028862bE2a173976CA11");

// Define the Multicall3 interface using alloy's sol! macro
sol! {
    /// A single call to make
    #[derive(Debug)]
    struct Call3 {
        address target;
        bool allowFailure;
        bytes callData;
    }

    /// Result of a single call
    #[derive(Debug)]
    struct Result {
        bool success;
        bytes returnData;
    }

    /// Multicall3 aggregate function
    #[derive(Debug)]
    function aggregate3(Call3[] calldata calls) external payable returns (Result[] memory returnData);
}

/// A builder for constructing multicall batches
#[derive(Debug, Default)]
pub struct MulticallBuilder {
    calls: Vec<Call3>,
}

impl MulticallBuilder {
    /// Create a new multicall builder
    pub fn new() -> Self {
        Self { calls: Vec::new() }
    }

    /// Add a call to the batch
    pub fn add_call(mut self, target: Address, calldata: Bytes, allow_failure: bool) -> Self {
        self.calls.push(Call3 {
            target,
            allowFailure: allow_failure,
            callData: calldata,
        });
        self
    }

    /// Add a call that can fail
    pub fn add_call_allow_failure(self, target: Address, calldata: Bytes) -> Self {
        self.add_call(target, calldata, true)
    }

    /// Get the number of calls in the batch
    pub fn len(&self) -> usize {
        self.calls.len()
    }

    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty()
    }

    /// Execute the multicall and return results
    pub async fn execute<P: Provider>(self, provider: &P) -> anyhow::Result<Vec<MulticallResult>> {
        self.execute_with_retry(provider, 0).await
    }

    /// Execute the multicall with retry logic
    pub async fn execute_with_retry<P: Provider>(
        self,
        provider: &P,
        max_retries: u32,
    ) -> anyhow::Result<Vec<MulticallResult>> {
        use std::time::Duration;

        if self.calls.is_empty() {
            return Ok(Vec::new());
        }

        // Encode the aggregate3 call
        let call = aggregate3Call { calls: self.calls };
        let calldata = call.abi_encode();

        // Make the eth_call with retry
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(MULTICALL3_ADDRESS)
            .input(calldata.into());

        let mut attempts = 0;
        let max_attempts = max_retries + 1;
        let mut last_error = None;

        while attempts < max_attempts {
            attempts += 1;

            match provider.call(tx.clone()).await {
                Ok(result) => {
                    // Decode the results
                    let decoded: Vec<Result> = aggregate3Call::abi_decode_returns(&result)
                        .map_err(|e| anyhow::anyhow!("Failed to decode multicall result: {}", e))?;

                    return Ok(decoded
                        .into_iter()
                        .map(|r| MulticallResult {
                            success: r.success,
                            data: r.returnData,
                        })
                        .collect());
                }
                Err(e) => {
                    let err_str = e.to_string().to_lowercase();
                    let is_retryable = err_str.contains("timeout")
                        || err_str.contains("connection")
                        || err_str.contains("temporarily")
                        || err_str.contains("503")
                        || err_str.contains("502")
                        || err_str.contains("504")
                        || err_str.contains("429")
                        || err_str.contains("rate");

                    if !is_retryable || attempts >= max_attempts {
                        return Err(anyhow::anyhow!("Multicall failed: {}", e));
                    }

                    last_error = Some(e);

                    // Exponential backoff: 100ms * 2^(attempt-1) with jitter
                    let base_delay = 100u64 * (1 << (attempts - 1));
                    let jitter = (rand::random::<u64>() % 50) + 1;
                    let delay = Duration::from_millis(base_delay + jitter);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(anyhow::anyhow!(
            "Multicall failed after {} attempts: {}",
            attempts,
            last_error.map(|e| e.to_string()).unwrap_or_default()
        ))
    }
}

/// Result of a single call in the multicall batch
#[derive(Debug, Clone)]
pub struct MulticallResult {
    /// Whether the call succeeded
    pub success: bool,
    /// The return data (empty if failed)
    pub data: Bytes,
}

impl MulticallResult {
    /// Try to decode as uint256 (for totalSupply, balanceOf, getEthBalance)
    pub fn decode_uint256(&self) -> Option<U256> {
        if !self.success || self.data.len() < 32 {
            return None;
        }
        Some(U256::from_be_slice(&self.data[..32]))
    }

    /// Try to decode as uint8 (for decimals)
    pub fn decode_uint8(&self) -> Option<u8> {
        if !self.success || self.data.is_empty() {
            return None;
        }
        Some(self.data[self.data.len() - 1])
    }

    /// Try to decode as a string (for name/symbol)
    pub fn decode_string(&self) -> Option<String> {
        if !self.success || self.data.len() < 64 {
            return None;
        }

        let data = &self.data[..];

        // Check for dynamic string encoding (offset at 0x20)
        if data.len() >= 64 {
            let offset_bytes: [u8; 8] = data[24..32].try_into().ok()?;
            let offset = u64::from_be_bytes(offset_bytes) as usize;

            if offset == 32 && data.len() >= 64 {
                let length_bytes: [u8; 8] = data[56..64].try_into().ok()?;
                let length = u64::from_be_bytes(length_bytes) as usize;

                if data.len() >= 64 + length {
                    let string_data = &data[64..64 + length];
                    return String::from_utf8(string_data.to_vec()).ok();
                }
            }
        }

        // Try bytes32 encoding (fixed-size string)
        let trimmed: Vec<u8> = data[0..32]
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect();

        if !trimmed.is_empty() {
            return String::from_utf8(trimmed).ok();
        }

        None
    }
}

/// Common function selectors
pub mod selectors {
    use alloy::primitives::{Address, Bytes};

    /// name() selector: 0x06fdde03
    pub fn name() -> Bytes {
        Bytes::from(vec![0x06, 0xfd, 0xde, 0x03])
    }

    /// symbol() selector: 0x95d89b41
    pub fn symbol() -> Bytes {
        Bytes::from(vec![0x95, 0xd8, 0x9b, 0x41])
    }

    /// decimals() selector: 0x313ce567
    pub fn decimals() -> Bytes {
        Bytes::from(vec![0x31, 0x3c, 0xe5, 0x67])
    }

    /// totalSupply() selector: 0x18160ddd
    pub fn total_supply() -> Bytes {
        Bytes::from(vec![0x18, 0x16, 0x0d, 0xdd])
    }

    /// balanceOf(address) selector: 0x70a08231
    pub fn balance_of(address: Address) -> Bytes {
        let mut data = vec![0x70, 0xa0, 0x82, 0x31];
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(address.as_slice());
        Bytes::from(data)
    }

    /// Multicall3's getEthBalance(address) selector: 0x4d2301cc
    pub fn get_eth_balance(address: Address) -> Bytes {
        let mut data = vec![0x4d, 0x23, 0x01, 0xcc];
        data.extend_from_slice(&[0u8; 12]);
        data.extend_from_slice(address.as_slice());
        Bytes::from(data)
    }
}
