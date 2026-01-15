//! ENS (Ethereum Name Service) resolution
//!
//! Resolve ENS names to addresses and vice versa

use crate::config::{Chain, ConfigFile, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::{keccak256, Address, B256};
use alloy::providers::Provider;
use clap::Subcommand;
use std::io::Write;
use std::str::FromStr;

// ENS Registry address (same on mainnet and testnets)
const ENS_REGISTRY: &str = "0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e";

// Resolver interface selectors
const ADDR_SELECTOR: [u8; 4] = [0x3b, 0x3b, 0x57, 0xde]; // addr(bytes32)
const NAME_SELECTOR: [u8; 4] = [0x69, 0x1f, 0x34, 0x31]; // name(bytes32)
const RESOLVER_SELECTOR: [u8; 4] = [0x01, 0x78, 0xb8, 0xbf]; // resolver(bytes32)

#[derive(Subcommand)]
pub enum EnsCommands {
    /// Resolve ENS name to address
    Resolve {
        /// ENS name (e.g., "vitalik.eth")
        name: String,
    },

    /// Reverse lookup - address to ENS name
    Lookup {
        /// Ethereum address
        address: String,
    },

    /// Get the resolver for an ENS name
    Resolver {
        /// ENS name
        name: String,
    },

    /// Compute namehash for an ENS name
    Namehash {
        /// ENS name
        name: String,
    },
}

pub async fn handle(
    action: &EnsCommands,
    chain: Chain,
    rpc_url: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Handle namehash first - it's a pure computation, no RPC needed
    if let EnsCommands::Namehash { name } = action {
        let hash = namehash(name);
        println!("{:#x}", hash);
        return Ok(());
    }

    // ENS only works on Ethereum mainnet (and some testnets)
    if chain != Chain::Ethereum {
        return Err(anyhow::anyhow!("ENS is only available on Ethereum mainnet"));
    }

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
        EnsCommands::Resolve { name } => {
            if !quiet {
                eprintln!("Resolving {}...", name);
                let _ = std::io::stderr().flush();
            }

            let address = resolve_name(&provider, name).await?;
            println!("{}", address.to_checksum(None));
        }

        EnsCommands::Lookup { address } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Looking up {}...", address);
                let _ = std::io::stderr().flush();
            }

            match reverse_lookup(&provider, addr).await {
                Ok(name) => println!("{}", name),
                Err(_) => println!("No ENS name found for this address"),
            }
        }

        EnsCommands::Resolver { name } => {
            if !quiet {
                eprintln!("Getting resolver for {}...", name);
                let _ = std::io::stderr().flush();
            }

            let resolver = get_resolver(&provider, name).await?;
            println!("{}", resolver.to_checksum(None));
        }

        EnsCommands::Namehash { .. } => {
            // Already handled above
            unreachable!()
        }
    }

    Ok(())
}

/// Compute the namehash of an ENS name
fn namehash(name: &str) -> B256 {
    let mut node = B256::ZERO;

    if name.is_empty() {
        return node;
    }

    // Split name into labels and process in reverse
    let labels: Vec<&str> = name.split('.').collect();

    for label in labels.into_iter().rev() {
        let label_hash = keccak256(label.as_bytes());
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(node.as_slice());
        combined[32..].copy_from_slice(label_hash.as_slice());
        node = keccak256(combined);
    }

    node
}

/// Get the resolver contract for an ENS name
async fn get_resolver<P: Provider>(provider: &P, name: &str) -> anyhow::Result<Address> {
    let registry = Address::from_str(ENS_REGISTRY)?;
    let node = namehash(name);

    // Call resolver(bytes32)
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&RESOLVER_SELECTOR);
    calldata.extend_from_slice(node.as_slice());

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(registry)
        .input(calldata.into());

    let result = provider
        .call(tx.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get resolver: {}", e))?;

    if result.len() < 32 {
        return Err(anyhow::anyhow!("Invalid resolver response"));
    }

    // Decode address from last 20 bytes of the 32-byte response
    let addr = Address::from_slice(&result[12..32]);

    if addr.is_zero() {
        return Err(anyhow::anyhow!("No resolver set for {}", name));
    }

    Ok(addr)
}

/// Resolve an ENS name to an address
pub async fn resolve_name<P: Provider>(provider: &P, name: &str) -> anyhow::Result<Address> {
    let resolver = get_resolver(provider, name).await?;
    let node = namehash(name);

    // Call addr(bytes32)
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&ADDR_SELECTOR);
    calldata.extend_from_slice(node.as_slice());

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(resolver)
        .input(calldata.into());

    let result = provider
        .call(tx.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to resolve name: {}", e))?;

    if result.len() < 32 {
        return Err(anyhow::anyhow!("Invalid address response"));
    }

    let addr = Address::from_slice(&result[12..32]);

    if addr.is_zero() {
        return Err(anyhow::anyhow!("Name {} not found", name));
    }

    Ok(addr)
}

/// Reverse lookup - find ENS name for an address
async fn reverse_lookup<P: Provider>(provider: &P, address: Address) -> anyhow::Result<String> {
    // Construct the reverse name: <address>.addr.reverse
    let addr_hex = format!("{:x}", address).to_lowercase();
    let reverse_name = format!("{}.addr.reverse", addr_hex);

    let resolver = get_resolver(provider, &reverse_name)
        .await
        .map_err(|e| anyhow::anyhow!("No reverse record set ({})", e))?;

    let node = namehash(&reverse_name);

    // Call name(bytes32)
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&NAME_SELECTOR);
    calldata.extend_from_slice(node.as_slice());

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(resolver)
        .input(calldata.into());

    let result = provider
        .call(tx.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to lookup name: {}", e))?;

    // Decode string from ABI-encoded response
    // Format: offset (32 bytes) + length (32 bytes) + string data
    if result.len() < 64 {
        return Err(anyhow::anyhow!("Invalid name response"));
    }

    let length = u64::from_be_bytes(result[56..64].try_into()?);
    if length == 0 {
        return Err(anyhow::anyhow!("No name set for address"));
    }

    // Bounds check: ensure length doesn't overflow and fits in result buffer
    let length_usize =
        usize::try_from(length).map_err(|_| anyhow::anyhow!("Invalid name length: {}", length))?;
    let end_offset = 64usize
        .checked_add(length_usize)
        .ok_or_else(|| anyhow::anyhow!("Name length overflow: {}", length))?;
    if end_offset > result.len() {
        return Err(anyhow::anyhow!(
            "Invalid name response: length {} exceeds buffer size {}",
            length,
            result.len()
        ));
    }

    let name_bytes = &result[64..end_offset];
    let name = String::from_utf8(name_bytes.to_vec())
        .map_err(|e| anyhow::anyhow!("Invalid name encoding: {}", e))?;

    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ENS namehash test vectors from ENS documentation
    // https://docs.ens.domains/resolution/names#namehash

    #[test]
    fn test_namehash_empty() {
        let hash = namehash("");
        assert_eq!(hash, B256::ZERO);
    }

    #[test]
    fn test_namehash_eth() {
        let hash = namehash("eth");
        assert_eq!(
            format!("{:#x}", hash),
            "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae"
        );
    }

    #[test]
    fn test_namehash_foo_eth() {
        let hash = namehash("foo.eth");
        assert_eq!(
            format!("{:#x}", hash),
            "0xde9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f"
        );
    }

    #[test]
    fn test_namehash_vitalik_eth() {
        // Known namehash for vitalik.eth
        let hash = namehash("vitalik.eth");
        assert_eq!(
            format!("{:#x}", hash),
            "0xee6c4522aab0003e8d14cd40a6af439055fd2577951148c14b6cea9a53475835"
        );
    }

    #[test]
    fn test_namehash_subdomain() {
        // test.foo.eth
        let hash = namehash("test.foo.eth");
        // The namehash should be deterministic
        let hash2 = namehash("test.foo.eth");
        assert_eq!(hash, hash2);
        // And different from parent
        let parent_hash = namehash("foo.eth");
        assert_ne!(hash, parent_hash);
    }

    #[test]
    fn test_namehash_case_sensitive() {
        // ENS namehash is case-sensitive (though normalization happens before)
        let hash1 = namehash("FOO.eth");
        let hash2 = namehash("foo.eth");
        // These should be different because namehash doesn't normalize
        assert_ne!(hash1, hash2);
    }

    // Test selector constants
    #[test]
    fn test_addr_selector() {
        // addr(bytes32) selector
        let expected = keccak256("addr(bytes32)".as_bytes());
        assert_eq!(ADDR_SELECTOR, expected[..4]);
    }

    #[test]
    fn test_name_selector() {
        // name(bytes32) selector
        let expected = keccak256("name(bytes32)".as_bytes());
        assert_eq!(NAME_SELECTOR, expected[..4]);
    }

    #[test]
    fn test_resolver_selector() {
        // resolver(bytes32) selector
        let expected = keccak256("resolver(bytes32)".as_bytes());
        assert_eq!(RESOLVER_SELECTOR, expected[..4]);
    }
}
