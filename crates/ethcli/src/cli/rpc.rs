//! RPC commands - direct Ethereum RPC calls
//!
//! Commands for reading blockchain state

use crate::config::{Chain, ConfigFile, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use clap::Subcommand;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum RpcCommands {
    /// Call a contract (eth_call) - read-only, no transaction
    ///
    /// Examples:
    ///   ethcli rpc call 0x... 0xa9059cbb...              # Raw calldata
    ///   ethcli rpc call 0x... -s "balanceOf(address)" -a 0xabc...  # With signature
    ///   ethcli rpc call 0x... -s "totalSupply()" -d uint256        # With decode
    Call {
        /// Contract address
        to: String,

        /// Calldata (hex encoded) - use this OR --sig
        data: Option<String>,

        /// Function signature (e.g., "balanceOf(address)")
        #[arg(long, short, conflicts_with = "data")]
        sig: Option<String>,

        /// Function arguments (use with --sig)
        #[arg(long, short = 'a', num_args = 1.., value_delimiter = ' ')]
        args: Option<Vec<String>>,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest")]
        block: String,

        /// Decode output as type (e.g., "uint256", "(address,uint256)")
        #[arg(long, short)]
        decode: Option<String>,
    },

    /// Get block information
    Block {
        /// Block number, hash, or "latest"
        block: String,

        /// Show full transactions
        #[arg(long, short)]
        full: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Read storage slot
    Storage {
        /// Contract address
        address: String,

        /// Storage slot (hex or decimal)
        slot: String,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest")]
        block: String,
    },

    /// Get contract bytecode
    Code {
        /// Contract address
        address: String,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest")]
        block: String,
    },

    /// Get account nonce
    Nonce {
        /// Account address
        address: String,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest")]
        block: String,
    },

    /// Get transaction receipt
    Receipt {
        /// Transaction hash
        hash: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get current chain ID
    ChainId,

    /// Get current block number
    BlockNumber,

    /// Get current gas price
    GasPrice,
}

pub async fn handle(
    action: &RpcCommands,
    chain: Chain,
    rpc_url: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Get RPC endpoint
    let endpoint = if let Some(url) = rpc_url {
        Endpoint::new(EndpointConfig::new(url), 30, None)?
    } else {
        // Use config endpoints
        let config = ConfigFile::load_default()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
            .unwrap_or_default();

        let chain_endpoints: Vec<_> = config
            .endpoints
            .into_iter()
            .filter(|e| e.enabled && e.chain == chain)
            .collect();

        if chain_endpoints.is_empty() {
            return Err(anyhow::anyhow!(
                "No RPC endpoints configured for {}. Add one with: ethcli endpoints add <url>",
                chain.display_name()
            ));
        }
        Endpoint::new(chain_endpoints[0].clone(), 30, None)?
    };

    let provider = endpoint.provider();

    match action {
        RpcCommands::Call {
            to,
            data,
            sig,
            args,
            block,
            decode,
        } => {
            let to_addr =
                Address::from_str(to).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // Build calldata from either raw data or signature + args
            let calldata = if let Some(signature) = sig {
                // Encode from function signature
                let func_args = args.as_deref().unwrap_or(&[]);
                let encoded = super::cast::abi_encode(signature, func_args)?;
                let hex_str = encoded.strip_prefix("0x").unwrap_or(&encoded);
                hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Encoding error: {}", e))?
            } else if let Some(data) = data {
                // Use raw calldata
                let data_hex = data.strip_prefix("0x").unwrap_or(data);
                hex::decode(data_hex).map_err(|e| anyhow::anyhow!("Invalid calldata: {}", e))?
            } else {
                return Err(anyhow::anyhow!(
                    "Calldata or --sig required. Use: rpc call <addr> <calldata> OR rpc call <addr> -s \"func()\""
                ));
            };

            let block_id = parse_block_id(block)?;

            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(to_addr)
                .input(calldata.into());

            let result = provider
                .call(tx)
                .block(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Call failed: {}", e))?;

            if let Some(type_sig) = decode {
                let decoded = decode_output(&result, type_sig)?;
                println!("{}", decoded);
            } else {
                println!("0x{}", hex::encode(&result));
            }
        }

        RpcCommands::Block { block, full, json } => {
            let block_id = parse_block_id(block)?;

            let block_data = if *full {
                provider
                    .get_block(block_id)
                    .full()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get block: {}", e))?
            } else {
                provider
                    .get_block(block_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get block: {}", e))?
            };

            match block_data {
                Some(b) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&b)?);
                    } else {
                        println!("Block {}", b.header.number);
                        println!("{}", "─".repeat(50));
                        println!("Hash:        {:#x}", b.header.hash);
                        println!("Parent:      {:#x}", b.header.parent_hash);
                        println!("Timestamp:   {}", b.header.timestamp);
                        println!("Gas Used:    {}", b.header.gas_used);
                        println!("Gas Limit:   {}", b.header.gas_limit);
                        // Show base fee with decimal precision
                        let base_fee = b.header.base_fee_per_gas.unwrap_or(0);
                        let gwei_div = 1_000_000_000u64;
                        let base_gwei = base_fee / gwei_div;
                        let base_frac = (base_fee % gwei_div) * 1000 / gwei_div;
                        println!("Base Fee:    {}.{:03} gwei", base_gwei, base_frac);
                        println!("Txs:         {}", b.transactions.len());
                        println!("Miner:       {}", b.header.beneficiary.to_checksum(None));
                    }
                }
                None => {
                    println!("Block not found");
                }
            }
        }

        RpcCommands::Storage {
            address,
            slot,
            block,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let slot_val = if slot.starts_with("0x") {
                U256::from_str(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?
            } else {
                U256::from_str(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?
            };

            let block_id = parse_block_id(block)?;

            let value = provider
                .get_storage_at(addr, slot_val)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read storage: {}", e))?;

            println!("{:#x}", value);
        }

        RpcCommands::Code { address, block } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let block_id = parse_block_id(block)?;

            let code = provider
                .get_code_at(addr)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get code: {}", e))?;

            if code.is_empty() {
                println!("0x (no code - EOA or empty contract)");
            } else {
                println!("0x{}", hex::encode(&code));
                if !quiet {
                    eprintln!("\n({} bytes)", code.len());
                }
            }
        }

        RpcCommands::Nonce { address, block } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let block_id = parse_block_id(block)?;

            let nonce = provider
                .get_transaction_count(addr)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

            println!("{}", nonce);
        }

        RpcCommands::Receipt { hash, json } => {
            let tx_hash =
                B256::from_str(hash).map_err(|e| anyhow::anyhow!("Invalid tx hash: {}", e))?;

            let receipt = provider
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get receipt: {}", e))?;

            match receipt {
                Some(r) => {
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&r)?);
                    } else {
                        println!("Transaction Receipt");
                        println!("{}", "─".repeat(50));
                        println!("Hash:        {:#x}", r.transaction_hash);
                        println!("Block:       {}", r.block_number.unwrap_or(0));
                        println!(
                            "Status:      {}",
                            if r.status() { "Success" } else { "Failed" }
                        );
                        println!("Gas Used:    {}", r.gas_used);
                        if let Some(to) = r.to {
                            println!("To:          {}", to.to_checksum(None));
                        }
                        if let Some(addr) = r.contract_address {
                            println!("Created:     {}", addr.to_checksum(None));
                        }
                        println!("Logs:        {}", r.inner.logs().len());
                    }
                }
                None => {
                    println!("Receipt not found (tx may be pending)");
                }
            }
        }

        RpcCommands::ChainId => {
            let chain_id = provider
                .get_chain_id()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get chain ID: {}", e))?;
            println!("{}", chain_id);
        }

        RpcCommands::BlockNumber => {
            let block_num = provider
                .get_block_number()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
            println!("{}", block_num);
        }

        RpcCommands::GasPrice => {
            let gas_price = provider
                .get_gas_price()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?;
            println!("{} gwei", gas_price / 1_000_000_000);
        }
    }

    Ok(())
}

pub fn parse_block_id(block: &str) -> anyhow::Result<alloy::eips::BlockId> {
    use alloy::eips::{BlockId, BlockNumberOrTag};

    match block.to_lowercase().as_str() {
        "latest" => Ok(BlockId::Number(BlockNumberOrTag::Latest)),
        "pending" => Ok(BlockId::Number(BlockNumberOrTag::Pending)),
        "earliest" => Ok(BlockId::Number(BlockNumberOrTag::Earliest)),
        "finalized" => Ok(BlockId::Number(BlockNumberOrTag::Finalized)),
        "safe" => Ok(BlockId::Number(BlockNumberOrTag::Safe)),
        _ => {
            if block.starts_with("0x") && block.len() == 66 {
                // Block hash
                let hash = B256::from_str(block)
                    .map_err(|e| anyhow::anyhow!("Invalid block hash: {}", e))?;
                Ok(BlockId::Hash(hash.into()))
            } else {
                // Block number
                let num: u64 = block
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Invalid block number: {}", e))?;
                Ok(BlockId::Number(BlockNumberOrTag::Number(num)))
            }
        }
    }
}

fn decode_output(data: &[u8], type_sig: &str) -> anyhow::Result<String> {
    use alloy::dyn_abi::DynSolType;

    let ty = DynSolType::parse(type_sig).map_err(|e| anyhow::anyhow!("Invalid type: {}", e))?;

    let decoded = ty
        .abi_decode(data)
        .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;

    Ok(format!("{:?}", decoded))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::eips::{BlockId, BlockNumberOrTag};

    // ==================== parse_block_id tests ====================

    #[test]
    fn test_parse_block_latest() {
        let result = parse_block_id("latest").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Latest));
    }

    #[test]
    fn test_parse_block_latest_uppercase() {
        let result = parse_block_id("LATEST").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Latest));
    }

    #[test]
    fn test_parse_block_pending() {
        let result = parse_block_id("pending").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Pending));
    }

    #[test]
    fn test_parse_block_earliest() {
        let result = parse_block_id("earliest").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Earliest));
    }

    #[test]
    fn test_parse_block_finalized() {
        let result = parse_block_id("finalized").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Finalized));
    }

    #[test]
    fn test_parse_block_safe() {
        let result = parse_block_id("safe").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Safe));
    }

    #[test]
    fn test_parse_block_number() {
        let result = parse_block_id("12345").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(12345)));
    }

    #[test]
    fn test_parse_block_number_zero() {
        let result = parse_block_id("0").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(0)));
    }

    #[test]
    fn test_parse_block_number_large() {
        let result = parse_block_id("21000000").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(21000000)));
    }

    #[test]
    fn test_parse_block_hash() {
        let hash_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = parse_block_id(hash_str).unwrap();
        let expected_hash = B256::from_str(hash_str).unwrap();
        assert_eq!(result, BlockId::Hash(expected_hash.into()));
    }

    #[test]
    fn test_parse_block_invalid_number() {
        let result = parse_block_id("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_block_invalid_hash_length() {
        // Hash that's too short (not 66 chars)
        let result = parse_block_id("0x1234");
        // This should try to parse as a number and fail
        assert!(result.is_err());
    }

    // ==================== decode_output tests ====================

    #[test]
    fn test_decode_output_uint256() {
        // 1000 encoded as uint256
        let data = hex::decode("00000000000000000000000000000000000000000000000000000000000003e8")
            .unwrap();
        let result = decode_output(&data, "uint256").unwrap();
        assert!(result.contains("1000"));
    }

    #[test]
    fn test_decode_output_bool_true() {
        // true encoded as bool
        let data = hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();
        let result = decode_output(&data, "bool").unwrap();
        assert!(result.contains("true"));
    }

    #[test]
    fn test_decode_output_bool_false() {
        // false encoded as bool
        let data = hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();
        let result = decode_output(&data, "bool").unwrap();
        assert!(result.contains("false"));
    }

    #[test]
    fn test_decode_output_address() {
        // Address encoded
        let data = hex::decode("000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap();
        let result = decode_output(&data, "address").unwrap();
        // Should contain the address (case-insensitive check)
        assert!(result
            .to_lowercase()
            .contains("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"));
    }

    #[test]
    fn test_decode_output_invalid_type() {
        let data = vec![0u8; 32];
        let result = decode_output(&data, "invalid_type");
        assert!(result.is_err());
    }
}
