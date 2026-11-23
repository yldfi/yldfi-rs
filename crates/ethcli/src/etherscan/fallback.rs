//! Verified source and ABI from Sourcify and Blockscout
//!
//! Used when Etherscan does not have a contract verified, or cannot be used
//! (no key, plan restriction, rate limit, unsupported chain). Sourcify serves
//! every chain it verifies on; Blockscout is tried only where its public API
//! answers scripted requests.

use alloy::json_abi::JsonAbi;
use foundry_block_explorers::contract::{Metadata, SourceCodeEntry, SourceCodeMetadata};
use std::collections::HashMap;
use std::time::Duration;

const SOURCIFY_API: &str = "https://sourcify.dev/server/v2/contract";

/// Where a contract's verified source or ABI came from
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    Etherscan,
    /// Sourcify, with its match kind (`exact_match` or `match`)
    Sourcify(String),
    /// Blockscout, with the host used
    Blockscout(String),
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::Etherscan => write!(f, "Etherscan"),
            Origin::Sourcify(kind) => write!(f, "Sourcify ({kind})"),
            Origin::Blockscout(host) => write!(f, "Blockscout ({host})"),
        }
    }
}

/// A verified contract found outside Etherscan
#[derive(Debug, Clone)]
pub struct Verified {
    pub origin: Origin,
    pub abi: JsonAbi,
    pub name: String,
    pub compiler_version: String,
    pub optimization_used: bool,
    pub runs: u64,
    pub evm_version: String,
    pub sources: HashMap<String, String>,
}

impl Verified {
    /// The same shape Etherscan's `getsourcecode` gives, so callers handle
    /// every origin the same way
    pub fn to_metadata(&self) -> Metadata {
        let sources = self
            .sources
            .iter()
            .map(|(path, content)| (path.clone(), SourceCodeEntry::from(content.clone())))
            .collect();
        Metadata {
            source_code: SourceCodeMetadata::Sources(sources),
            abi: serde_json::to_string(&self.abi).unwrap_or_default(),
            contract_name: self.name.clone(),
            compiler_version: self.compiler_version.clone(),
            optimization_used: u64::from(self.optimization_used),
            runs: self.runs,
            constructor_arguments: Default::default(),
            evm_version: self.evm_version.clone(),
            library: String::new(),
            license_type: String::new(),
            proxy: 0,
            implementation: None,
            swarm_source: String::new(),
        }
    }
}

/// Blockscout hosts whose public API answers scripted requests
fn blockscout_host(chain_id: u64) -> Option<&'static str> {
    match chain_id {
        1 => Some("eth.blockscout.com"),
        10 => Some("explorer.optimism.io"),
        _ => None,
    }
}

async fn get_json(http: &reqwest::Client, url: &str) -> Option<serde_json::Value> {
    let response = http
        .get(url)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    response.json().await.ok()
}

/// Parse a Sourcify v2 `contract` response
pub fn parse_sourcify(value: &serde_json::Value) -> Option<Verified> {
    let kind = value["match"].as_str()?.to_string();
    let abi: JsonAbi = serde_json::from_value(value["abi"].clone()).ok()?;
    let compilation = &value["compilation"];
    let settings = &compilation["compilerSettings"];
    let sources = value["sources"]
        .as_object()?
        .iter()
        .filter_map(|(path, entry)| Some((path.clone(), entry["content"].as_str()?.to_string())))
        .collect();
    Some(Verified {
        origin: Origin::Sourcify(kind),
        abi,
        name: compilation["name"].as_str().unwrap_or_default().to_string(),
        compiler_version: compilation["compilerVersion"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        optimization_used: settings["optimizer"]["enabled"].as_bool().unwrap_or(false),
        runs: settings["optimizer"]["runs"].as_u64().unwrap_or(0),
        evm_version: settings["evmVersion"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        sources,
    })
}

/// Parse a Blockscout v2 `smart-contracts` response
pub fn parse_blockscout(value: &serde_json::Value, host: &str) -> Option<Verified> {
    if value["is_verified"].as_bool() != Some(true) {
        return None;
    }
    let abi: JsonAbi = serde_json::from_value(value["abi"].clone()).ok()?;
    let name = value["name"].as_str().unwrap_or_default().to_string();
    let mut sources = HashMap::new();
    if let Some(code) = value["source_code"].as_str() {
        let path = value["file_path"]
            .as_str()
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("{name}.sol"));
        sources.insert(path, code.to_string());
    }
    for extra in value["additional_sources"].as_array().into_iter().flatten() {
        if let (Some(path), Some(code)) =
            (extra["file_path"].as_str(), extra["source_code"].as_str())
        {
            sources.insert(path.to_string(), code.to_string());
        }
    }
    Some(Verified {
        origin: Origin::Blockscout(host.to_string()),
        abi,
        name,
        compiler_version: value["compiler_version"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        optimization_used: value["optimization_enabled"].as_bool().unwrap_or(false),
        runs: value["optimization_runs"].as_u64().unwrap_or(0),
        evm_version: value["evm_version"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        sources,
    })
}

/// Look the contract up on Sourcify, then Blockscout. `None` when neither
/// has it verified (or neither answered).
pub async fn find_verified(
    http: &reqwest::Client,
    chain_id: u64,
    address: &str,
) -> Option<Verified> {
    let url = format!("{SOURCIFY_API}/{chain_id}/{address}?fields=abi,sources,compilation");
    if let Some(found) = get_json(http, &url).await.as_ref().and_then(parse_sourcify) {
        return Some(found);
    }
    let host = blockscout_host(chain_id)?;
    let url = format!("https://{host}/api/v2/smart-contracts/{address}");
    get_json(http, &url)
        .await
        .as_ref()
        .and_then(|v| parse_blockscout(v, host))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const ABI: &str = r#"[{"type":"function","name":"owner","inputs":[],"outputs":[{"name":"","type":"address"}],"stateMutability":"view"}]"#;

    #[test]
    fn sourcify_response_becomes_metadata() {
        let value = json!({
            "match": "exact_match",
            "abi": serde_json::from_str::<serde_json::Value>(ABI).unwrap(),
            "sources": {"contracts/A.sol": {"content": "contract A {}"}},
            "compilation": {"name": "A", "compilerVersion": "0.8.20+commit.a1b79de6",
                "compilerSettings": {"optimizer": {"enabled": true, "runs": 200}, "evmVersion": "paris"}}
        });
        let found = parse_sourcify(&value).unwrap();
        assert_eq!(found.origin.to_string(), "Sourcify (exact_match)");
        assert_eq!(found.abi.functions().count(), 1);
        let metadata = found.to_metadata();
        assert_eq!(metadata.contract_name, "A");
        assert_eq!(metadata.optimization_used, 1);
        assert_eq!(metadata.runs, 200);
        assert_eq!(
            metadata.source_code.sources()["contracts/A.sol"].content,
            "contract A {}"
        );

        assert!(parse_sourcify(&json!({"match": null, "chainId": "1"})).is_none());
    }

    #[test]
    fn blockscout_response_needs_verification_and_keeps_all_files() {
        let value = json!({
            "is_verified": true, "name": "B", "file_path": "src/B.sol", "source_code": "contract B {}",
            "additional_sources": [{"file_path": "src/Lib.sol", "source_code": "library L {}"}],
            "abi": serde_json::from_str::<serde_json::Value>(ABI).unwrap(),
            "compiler_version": "v0.8.19", "optimization_enabled": false, "evm_version": "default"
        });
        let found = parse_blockscout(&value, "eth.blockscout.com").unwrap();
        assert_eq!(found.sources.len(), 2);
        assert_eq!(found.origin.to_string(), "Blockscout (eth.blockscout.com)");
        assert!(parse_blockscout(&json!({"is_verified": false}), "h").is_none());
    }
}
