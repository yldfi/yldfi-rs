//! ENS (Ethereum Name Service) resolution
//!
//! Resolve ENS names to addresses and vice versa

use crate::config::{Chain, EndpointConfig};
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
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Reverse lookup - address to ENS name
    Lookup {
        /// Ethereum address
        #[arg(value_name = "ADDRESS")]
        address: String,
    },

    /// Get the resolver for an ENS name
    Resolver {
        /// ENS name
        #[arg(value_name = "NAME")]
        name: String,
    },

    /// Compute namehash for an ENS name
    Namehash {
        /// ENS name
        #[arg(value_name = "NAME")]
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

    // An explicit --rpc-url is used alone; otherwise try the ranked endpoints,
    // moving on when one is rate limited or unreachable
    let candidates = if let Some(url) = rpc_url {
        vec![Endpoint::new(EndpointConfig::new(url), 30, None)?]
    } else {
        crate::rpc::candidate_endpoints(
            chain,
            &crate::rpc::SelectionOptions::default(),
            crate::rpc::MAX_FAILOVER_ATTEMPTS,
        )?
    };
    if candidates.is_empty() {
        return Err(anyhow::anyhow!(
            "No RPC endpoints configured for {}. Add one with: ethcli endpoints add <url>",
            chain.display_name()
        ));
    }

    crate::rpc::with_failover(candidates, |endpoint| async move {
        run_on_endpoint(action, &endpoint, quiet).await
    })
    .await
}

/// Run an RPC-backed ENS command against one endpoint
async fn run_on_endpoint(
    action: &EnsCommands,
    endpoint: &Endpoint,
    quiet: bool,
) -> anyhow::Result<()> {
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

            match reverse_lookup(&provider, addr).await? {
                ReverseName::Verified(name) => println!("{}", name),
                ReverseName::NotSet => println!("No ENS name found for this address"),
                ReverseName::Unverified {
                    claimed,
                    resolves_to,
                } => {
                    println!("No verified ENS name found for this address");
                    if !quiet {
                        let target = match resolves_to {
                            Some(a) if a == addr => "it is not in normalised form".to_string(),
                            Some(a) => format!("it resolves to {}", a.to_checksum(None)),
                            None => "it does not resolve".to_string(),
                        };
                        eprintln!(
                            "Warning: ignoring the reverse record {:?}: {}",
                            claimed, target
                        );
                    }
                }
            }
        }

        EnsCommands::Resolver { name } => {
            if !quiet {
                eprintln!("Getting resolver for {}...", name);
                let _ = std::io::stderr().flush();
            }

            let resolver = get_resolver(&provider, name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("No resolver set for {}", name))?;
            println!("{}", resolver.to_checksum(None));
        }

        EnsCommands::Namehash { .. } => {
            // Already handled above
            unreachable!()
        }
    }

    Ok(())
}

/// Normalise an ENS name before hashing: trim, lowercase and NFC.
///
/// This covers the common ENSIP-15 cases (case folding, composed characters),
/// so "Vitalik.eth" hashes to the same node as in other clients. ENSIP-15's
/// emoji, confusable and disallowed-character rules are not applied.
fn normalize_name(name: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    name.trim().to_lowercase().nfc().collect()
}

/// True if a name has no control or bidi-override characters, which could make
/// an on-chain string render as something else in a terminal
fn is_displayable(name: &str) -> bool {
    !name.chars().any(|c| {
        c.is_control()
            || ('\u{202a}'..='\u{202e}').contains(&c)
            || ('\u{2066}'..='\u{2069}').contains(&c)
    })
}

/// Compute the namehash of an ENS name (normalised first)
fn namehash(name: &str) -> B256 {
    let mut node = B256::ZERO;

    let name = normalize_name(name);
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

/// Get the resolver contract for an ENS name (`None` if none is set)
async fn get_resolver<P: Provider>(provider: &P, name: &str) -> anyhow::Result<Option<Address>> {
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

    Ok((!addr.is_zero()).then_some(addr))
}

/// Resolve an ENS name to an address.
///
/// Supports ENSIP-10 wildcard resolution (a resolver set on a parent name,
/// e.g. `*.base.eth`, `*.cb.id`) and EIP-3668 CCIP-read offchain lookups.
pub async fn resolve_name<P: Provider>(provider: &P, name: &str) -> anyhow::Result<Address> {
    use alloy::sol_types::{SolCall, SolValue};

    let (resolver, exact) = find_resolver(provider, name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No resolver set for {}", name))?;
    let node = namehash(name);

    // Call addr(bytes32)
    let mut addr_call = Vec::with_capacity(36);
    addr_call.extend_from_slice(&ADDR_SELECTOR);
    addr_call.extend_from_slice(node.as_slice());

    // A resolver found on a parent name must be an extended (ENSIP-10)
    // resolver; one set on the name itself may be either kind
    let extended = !exact || supports_extended_resolver(provider, resolver).await;
    let result = if extended {
        let call = ens_abi::resolveCall {
            name: dns_encode(name)?.into(),
            data: addr_call.into(),
        };
        let out = ccip_call(provider, resolver, call.abi_encode()).await?;
        alloy::primitives::Bytes::abi_decode(&out)
            .map_err(|e| anyhow::anyhow!("Invalid resolve() response: {}", e))?
    } else {
        ccip_call(provider, resolver, addr_call).await?
    };

    if result.len() < 32 {
        return Err(anyhow::anyhow!("Name {} not found", name));
    }

    let addr = Address::from_slice(&result[12..32]);

    if addr.is_zero() {
        return Err(anyhow::anyhow!("Name {} not found", name));
    }

    Ok(addr)
}

mod ens_abi {
    alloy::sol! {
        error OffchainLookup(address sender, string[] urls, bytes callData, bytes4 callbackFunction, bytes extraData);
        function resolve(bytes name, bytes data) external view returns (bytes);
        function supportsInterface(bytes4 interfaceID) external view returns (bool);
    }
}

/// IExtendedResolver interface id (resolve(bytes,bytes))
const EXTENDED_RESOLVER_INTERFACE: [u8; 4] = [0x90, 0x61, 0xb9, 0x23];

/// Find the resolver for `name`, walking up to parent names (ENSIP-10).
/// Returns the resolver and whether it was set on `name` itself.
async fn find_resolver<P: Provider>(
    provider: &P,
    name: &str,
) -> anyhow::Result<Option<(Address, bool)>> {
    let name = normalize_name(name);
    let mut current = name.as_str();
    loop {
        if let Some(resolver) = get_resolver(provider, current).await? {
            return Ok(Some((resolver, current == name)));
        }
        match current.split_once('.') {
            Some((_, parent)) if !parent.is_empty() => current = parent,
            _ => return Ok(None),
        }
    }
}

/// Whether a resolver implements IExtendedResolver (ERC-165)
async fn supports_extended_resolver<P: Provider>(provider: &P, resolver: Address) -> bool {
    use alloy::sol_types::SolCall;
    let call = ens_abi::supportsInterfaceCall {
        interfaceID: EXTENDED_RESOLVER_INTERFACE.into(),
    };
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(resolver)
        .input(call.abi_encode().into());
    match provider.call(tx).await {
        Ok(out) => out.len() >= 32 && out[31] == 1,
        Err(_) => false,
    }
}

/// DNS wire-format encoding of a (normalised) name, as resolve(bytes,bytes) takes
fn dns_encode(name: &str) -> anyhow::Result<Vec<u8>> {
    let name = normalize_name(name);
    let mut out = Vec::with_capacity(name.len() + 2);
    for label in name.split('.').filter(|l| !l.is_empty()) {
        let len = u8::try_from(label.len())
            .map_err(|_| anyhow::anyhow!("ENS label too long: {}", label))?;
        out.push(len);
        out.extend_from_slice(label.as_bytes());
    }
    out.push(0);
    Ok(out)
}

/// eth_call that follows EIP-3668 OffchainLookup reverts through the
/// gateway URLs the contract names (https only), up to 4 hops.
async fn ccip_call<P: Provider>(
    provider: &P,
    to: Address,
    data: Vec<u8>,
) -> anyhow::Result<alloy::primitives::Bytes> {
    use alloy::sol_types::{SolError, SolValue};

    let mut data = data;
    for _ in 0..4 {
        let tx = alloy::rpc::types::TransactionRequest::default()
            .to(to)
            .input(data.clone().into());
        let err = match provider.call(tx).await {
            Ok(out) => return Ok(out),
            Err(e) => e,
        };
        let revert = err.as_error_resp().and_then(|p| p.as_revert_data());
        let lookup = revert
            .as_deref()
            .filter(|r| r.starts_with(&ens_abi::OffchainLookup::SELECTOR))
            .and_then(|r| ens_abi::OffchainLookup::abi_decode(r).ok());
        let Some(lookup) = lookup else {
            return Err(anyhow::anyhow!("ENS call failed: {}", err));
        };
        if lookup.sender != to {
            anyhow::bail!("CCIP-read: OffchainLookup sender does not match the resolver");
        }
        let response = ccip_fetch(&lookup.urls, lookup.sender, &lookup.callData).await?;
        let mut next = lookup.callbackFunction.to_vec();
        next.extend((response, lookup.extraData).abi_encode_params());
        data = next;
    }
    anyhow::bail!("CCIP-read: too many offchain lookups")
}

/// Query CCIP-read gateways in order until one answers
async fn ccip_fetch(
    urls: &[String],
    sender: Address,
    call_data: &[u8],
) -> anyhow::Result<alloy::primitives::Bytes> {
    let client = crate::utils::get_shared_http_client().map_err(|e| anyhow::anyhow!(e))?;
    let sender_hex = format!("{:#x}", sender);
    let data_hex = format!("0x{}", hex::encode(call_data));
    let mut last_err = anyhow::anyhow!("CCIP-read: no gateway URLs");
    for url in urls {
        if !url.starts_with("https://") {
            last_err = anyhow::anyhow!("CCIP-read: refusing non-https gateway");
            continue;
        }
        let url = url.replace("{sender}", &sender_hex);
        let request = if url.contains("{data}") {
            client.get(url.replace("{data}", &data_hex))
        } else {
            client
                .post(&url)
                .json(&serde_json::json!({"data": data_hex, "sender": sender_hex}))
        };
        let result = async {
            let response = request
                .timeout(std::time::Duration::from_secs(15))
                .send()
                .await?
                .error_for_status()?;
            let body: serde_json::Value = response.json().await?;
            let hex_data = body["data"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("gateway response has no data"))?;
            Ok::<_, anyhow::Error>(hex::decode(hex_data.trim_start_matches("0x"))?.into())
        }
        .await;
        match result {
            Ok(bytes) => return Ok(bytes),
            Err(e) => {
                last_err = anyhow::anyhow!(
                    "CCIP-read gateway failed: {}",
                    crate::utils::url::redact_urls_in_text(&e.to_string())
                )
            }
        }
    }
    Err(last_err)
}

/// Outcome of a reverse lookup
#[derive(Debug, PartialEq, Eq)]
enum ReverseName {
    /// The reverse record's name resolves back to the address
    Verified(String),
    /// No reverse record
    NotSet,
    /// The reverse record names something that doesn't resolve back to the
    /// address. Anyone can set any reverse name, so it must not be shown as
    /// the address's name.
    Unverified {
        claimed: String,
        resolves_to: Option<Address>,
    },
}

/// Reverse lookup - find the ENS name for an address, verified by forward resolution
async fn reverse_lookup<P: Provider>(
    provider: &P,
    address: Address,
) -> anyhow::Result<ReverseName> {
    let Some(claimed) = reverse_record(provider, address).await? else {
        return Ok(ReverseName::NotSet);
    };
    let resolves_to = resolve_name(provider, &claimed).await.ok();
    // A name that isn't already in normal form (mixed case, decomposed or
    // control/bidi characters) can display differently from what it hashes to.
    if resolves_to == Some(address)
        && claimed == normalize_name(&claimed)
        && is_displayable(&claimed)
    {
        Ok(ReverseName::Verified(claimed))
    } else {
        Ok(ReverseName::Unverified {
            claimed,
            resolves_to,
        })
    }
}

/// Read the (unverified) reverse record for an address
async fn reverse_record<P: Provider>(
    provider: &P,
    address: Address,
) -> anyhow::Result<Option<String>> {
    // Construct the reverse name: <address>.addr.reverse
    let addr_hex = format!("{:x}", address).to_lowercase();
    let reverse_name = format!("{}.addr.reverse", addr_hex);

    let Some(resolver) = get_resolver(provider, &reverse_name).await? else {
        return Ok(None);
    };

    let node = namehash(&reverse_name);

    // Call name(bytes32)
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(&NAME_SELECTOR);
    calldata.extend_from_slice(node.as_slice());

    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(resolver)
        .input(calldata.into());

    let result = match provider.call(tx.clone()).await {
        Ok(r) => r,
        // A resolver without name() support reverts: there is no name
        Err(e) if e.to_string().to_lowercase().contains("revert") => return Ok(None),
        Err(e) => return Err(anyhow::anyhow!("Failed to lookup name: {}", e)),
    };

    // Decode string from ABI-encoded response
    // Format: offset (32 bytes) + length (32 bytes) + string data
    if result.len() < 64 {
        return Err(anyhow::anyhow!("Invalid name response"));
    }

    let length = u64::from_be_bytes(result[56..64].try_into()?);
    if length == 0 {
        return Ok(None);
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

    Ok(Some(name))
}

#[cfg(test)]
mod tests {

    #[test]
    fn dns_encoding() {
        assert_eq!(
            dns_encode("Jesse.Base.eth").unwrap(),
            b"\x05jesse\x04base\x03eth\x00".to_vec()
        );
        assert!(dns_encode(&format!("{}.eth", "a".repeat(300))).is_err());
    }

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
    fn test_namehash_normalizes_case_and_composition() {
        assert_eq!(namehash("Vitalik.ETH"), namehash("vitalik.eth"));
        assert_eq!(namehash(" vitalik.eth "), namehash("vitalik.eth"));
        // "é" as e + combining acute (NFD) and as one code point (NFC)
        assert_eq!(namehash("cafe\u{301}.eth"), namehash("caf\u{e9}.eth"));
    }

    /// Minimal ENS over JSON-RPC: one resolver for every node, whose reverse
    /// record says `claimed` and whose addr() answers `forward`.
    struct FakeEns {
        claimed: &'static str,
        forward: Address,
    }

    impl wiremock::Respond for FakeEns {
        fn respond(&self, req: &wiremock::Request) -> wiremock::ResponseTemplate {
            use alloy::sol_types::SolValue;
            let body: serde_json::Value = serde_json::from_slice(&req.body).unwrap();
            let tx = &body["params"][0];
            let input = tx["input"].as_str().or(tx["data"].as_str()).unwrap();
            let input = hex::decode(input.trim_start_matches("0x")).unwrap();
            let resolver = Address::repeat_byte(0x11);
            let out = match &input[..4] {
                s if s == RESOLVER_SELECTOR => resolver.abi_encode(),
                s if s == NAME_SELECTOR => self.claimed.to_string().abi_encode(),
                s if s == ADDR_SELECTOR => self.forward.abi_encode(),
                _ => panic!("unexpected call"),
            };
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": body["id"],
                "result": format!("0x{}", hex::encode(out)),
            }))
        }
    }

    async fn lookup_with(claimed: &'static str, forward: Address, address: Address) -> ReverseName {
        let server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("POST"))
            .respond_with(FakeEns { claimed, forward })
            .mount(&server)
            .await;
        let endpoint = Endpoint::new(EndpointConfig::new(server.uri()), 5, None).unwrap();
        reverse_lookup(endpoint.provider(), address).await.unwrap()
    }

    #[tokio::test]
    async fn reverse_lookup_rejects_a_spoofed_reverse_record() {
        let attacker = Address::repeat_byte(0xaa);
        let vitalik = Address::repeat_byte(0xbb);
        assert_eq!(
            lookup_with("vitalik.eth", vitalik, attacker).await,
            ReverseName::Unverified {
                claimed: "vitalik.eth".to_string(),
                resolves_to: Some(vitalik),
            }
        );
    }

    #[tokio::test]
    async fn reverse_lookup_rejects_unnormalised_or_bidi_names() {
        let owner = Address::repeat_byte(0xbb);
        for claimed in ["Vitalik.eth", "vitalik\u{202e}hte.eth"] {
            assert!(matches!(
                lookup_with(claimed, owner, owner).await,
                ReverseName::Unverified { .. }
            ));
        }
    }

    #[tokio::test]
    async fn reverse_lookup_accepts_a_name_that_resolves_back() {
        let owner = Address::repeat_byte(0xbb);
        assert_eq!(
            lookup_with("vitalik.eth", owner, owner).await,
            ReverseName::Verified("vitalik.eth".to_string())
        );
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
