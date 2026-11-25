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
    /// Blockscout, with the host used and the match type
    /// (`exact`/`partial`/`unknown`)
    Blockscout(String, String),
}

impl Origin {
    /// How the verified source matches the deployed code: `exact` (code and
    /// metadata hash), `partial` (code only), `unknown`, or `not_reported`
    /// (Etherscan does not say)
    pub fn match_type(&self) -> &str {
        match self {
            Origin::Etherscan => "not_reported",
            Origin::Sourcify(kind) => match kind.as_str() {
                "exact_match" => "exact",
                "match" => "partial",
                _ => "unknown",
            },
            Origin::Blockscout(_, kind) => kind,
        }
    }
}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::Etherscan => write!(f, "Etherscan"),
            Origin::Sourcify(kind) => write!(f, "Sourcify ({kind})"),
            Origin::Blockscout(host, kind) => write!(f, "Blockscout ({host}, {kind} match)"),
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
    /// The verifier's compiler settings (solc standard-JSON `settings`:
    /// optimizer, evmVersion, viaIR, libraries), when it gives them
    pub settings: Option<serde_json::Value>,
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
        let source_code = match &self.settings {
            Some(settings) => SourceCodeMetadata::Metadata {
                language: None,
                sources,
                settings: Some(settings.clone()),
            },
            None => SourceCodeMetadata::Sources(sources),
        };
        Metadata {
            source_code,
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
    match fetch_json(http, url).await {
        Fetched::Json(v) => Some(v),
        _ => None,
    }
}

/// What a verifier answered
enum Fetched {
    Json(serde_json::Value),
    /// HTTP 404: it has no record of the address
    NotFound,
    /// No usable answer (rate limit, timeout, server error): says nothing
    /// about whether the contract is verified
    Unavailable(String),
}

/// GET a verifier URL, retrying once after a rate-limit (429) or server
/// error answer
async fn fetch_json(http: &reqwest::Client, url: &str) -> Fetched {
    let mut last = String::new();
    for attempt in 0..2 {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(1500)).await;
        }
        let response = match http.get(url).timeout(Duration::from_secs(20)).send().await {
            Ok(r) => r,
            Err(e) => {
                last = if e.is_timeout() {
                    "timed out".into()
                } else {
                    "request failed".into()
                };
                continue;
            }
        };
        let status = response.status();
        if status == reqwest::StatusCode::NOT_FOUND {
            return Fetched::NotFound;
        }
        if !status.is_success() {
            last = format!("HTTP {status}");
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                continue;
            }
            return Fetched::Unavailable(last);
        }
        return match response.json().await {
            Ok(v) => Fetched::Json(v),
            Err(_) => Fetched::Unavailable("unreadable response".into()),
        };
    }
    Fetched::Unavailable(last)
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
        settings: settings.is_object().then(|| settings.clone()),
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
        origin: Origin::Blockscout(host.to_string(), blockscout_match_type(value).match_type),
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
        settings: value["compiler_settings"]
            .is_object()
            .then(|| value["compiler_settings"].clone()),
    })
}

/// What the Sourcify / Blockscout lookup found
#[derive(Debug)]
pub enum Fallback {
    Found(Box<Verified>),
    /// Every verifier asked answered, and none has it verified
    NotVerified,
    /// At least one verifier gave no usable answer and none has it: whether
    /// it is verified is unknown. The text says which and why
    Unavailable(String),
}

/// Look the contract up on Sourcify, then Blockscout
pub async fn find_verified(http: &reqwest::Client, chain_id: u64, address: &str) -> Fallback {
    let mut unavailable = Vec::new();
    let url = format!("{SOURCIFY_API}/{chain_id}/{address}?fields=abi,sources,compilation");
    match fetch_json(http, &url).await {
        Fetched::Json(v) => {
            if let Some(found) = parse_sourcify(&v) {
                return Fallback::Found(Box::new(found));
            }
        }
        Fetched::NotFound => {}
        Fetched::Unavailable(why) => unavailable.push(format!("Sourcify: {why}")),
    }
    if let Some(host) = blockscout_host(chain_id) {
        let url = format!("https://{host}/api/v2/smart-contracts/{address}");
        match fetch_json(http, &url).await {
            Fetched::Json(v) => {
                if let Some(found) = parse_blockscout(&v, host) {
                    return Fallback::Found(Box::new(found));
                }
                // No verification keys at all is not a "no"
                if v["is_verified"].as_bool().is_none() && !v.is_null() {
                    unavailable.push("Blockscout: no verification status in its answer".into());
                }
            }
            Fetched::NotFound => {}
            Fetched::Unavailable(why) => unavailable.push(format!("Blockscout: {why}")),
        }
    }
    if unavailable.is_empty() {
        Fallback::NotVerified
    } else {
        Fallback::Unavailable(unavailable.join("; "))
    }
}

/// What Sourcify said about a contract's storage layout
#[derive(Debug)]
pub enum LayoutLookup {
    /// The `storageLayout` object
    Found(serde_json::Value),
    /// Verified, but Sourcify has no layout for it (older compilers, Vyper)
    NoLayout,
    /// Not verified on Sourcify
    NotVerified,
    /// Sourcify did not answer usefully
    Unavailable(String),
}

/// Fetch a contract's compiler storage layout from Sourcify
pub async fn storage_layout(http: &reqwest::Client, chain_id: u64, address: &str) -> LayoutLookup {
    let url = format!("{SOURCIFY_API}/{chain_id}/{address}?fields=storageLayout");
    let response = match http.get(&url).timeout(Duration::from_secs(20)).send().await {
        Ok(r) => r,
        Err(e) => return LayoutLookup::Unavailable(format!("request failed: {e}")),
    };
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return LayoutLookup::NotVerified;
    }
    if !response.status().is_success() {
        return LayoutLookup::Unavailable(format!("HTTP {}", response.status()));
    }
    match response.json::<serde_json::Value>().await {
        Ok(v) if v["storageLayout"].is_object() => LayoutLookup::Found(v["storageLayout"].clone()),
        Ok(v) if v["match"].is_null() => LayoutLookup::NotVerified,
        Ok(_) => LayoutLookup::NoLayout,
        Err(e) => LayoutLookup::Unavailable(format!("bad response: {e}")),
    }
}

/// Sourcify server base URL (`SOURCIFY_SERVER_URL` overrides it, for tests
/// against a mock server)
pub fn sourcify_base() -> String {
    std::env::var("SOURCIFY_SERVER_URL")
        .ok()
        .filter(|u| !u.is_empty())
        .map(|u| u.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "https://sourcify.dev/server".to_string())
}

/// Blockscout base URL for a chain (`BLOCKSCOUT_BASE_URL` overrides it, for
/// tests)
fn blockscout_base(chain_id: u64) -> Option<String> {
    if let Ok(url) = std::env::var("BLOCKSCOUT_BASE_URL") {
        if !url.is_empty() {
            return Some(url.trim_end_matches('/').to_string());
        }
    }
    blockscout_host(chain_id).map(|h| format!("https://{h}"))
}

/// How a verified contract matched its source, from one verifier
#[derive(Debug, Clone, serde::Serialize)]
pub struct VerificationMatch {
    /// `sourcify` or `blockscout`
    pub source: String,
    /// `exact` (bytecode and metadata hash match), `partial` (bytecode
    /// matches, metadata differs), `not_verified` or `unknown`
    pub match_type: String,
    /// The verifier's own fields, unchanged
    pub detail: serde_json::Value,
}

/// Classify a Sourcify v2 `contract` response (`match`, `runtimeMatch`,
/// `creationMatch`)
pub fn sourcify_match_type(value: &serde_json::Value) -> VerificationMatch {
    let kind = match value["match"].as_str() {
        Some("exact_match") => "exact",
        Some("match") => "partial",
        Some(_) => "unknown",
        None => "not_verified",
    };
    VerificationMatch {
        source: "sourcify".into(),
        match_type: kind.into(),
        detail: serde_json::json!({
            "match": value["match"],
            "runtimeMatch": value["runtimeMatch"],
            "creationMatch": value["creationMatch"],
        }),
    }
}

/// Classify a Blockscout v2 `smart-contracts` response. Blockscout leaves
/// the verification keys out for unverified contracts, so missing is
/// `unknown`, not `not_verified`.
pub fn blockscout_match_type(value: &serde_json::Value) -> VerificationMatch {
    let kind = match (
        value["is_verified"].as_bool(),
        value["is_fully_verified"].as_bool(),
        value["is_partially_verified"].as_bool(),
    ) {
        (Some(true), Some(true), _) => "exact",
        (Some(true), _, Some(true)) => "partial",
        (Some(true), _, _) => "unknown",
        (Some(false), _, _) => "not_verified",
        (None, ..) => "unknown",
    };
    VerificationMatch {
        source: "blockscout".into(),
        match_type: kind.into(),
        detail: serde_json::json!({
            "is_verified": value["is_verified"],
            "is_fully_verified": value["is_fully_verified"],
            "is_partially_verified": value["is_partially_verified"],
            "is_verified_via_eth_bytecode_db": value["is_verified_via_eth_bytecode_db"],
            "is_verified_via_sourcify": value["is_verified_via_sourcify"],
            "verified_twin_address_hash": value["verified_twin_address_hash"],
        }),
    }
}

/// Sourcify's verdict for an address (`None` when Sourcify did not answer)
pub async fn sourcify_lookup(
    http: &reqwest::Client,
    chain_id: u64,
    address: &str,
) -> Option<VerificationMatch> {
    let url = format!("{}/v2/contract/{chain_id}/{address}", sourcify_base());
    let response = http
        .get(&url)
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .ok()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Some(sourcify_match_type(&serde_json::Value::Null));
    }
    if !response.status().is_success() {
        return None;
    }
    let value: serde_json::Value = response.json().await.ok()?;
    Some(sourcify_match_type(&value))
}

/// Blockscout's `smart-contracts` record, with the base URL used (`None` on
/// a chain without a usable Blockscout, or when it did not answer)
pub async fn blockscout_lookup(
    http: &reqwest::Client,
    chain_id: u64,
    address: &str,
) -> Option<(String, serde_json::Value)> {
    let base = blockscout_base(chain_id)?;
    let url = format!("{base}/api/v2/smart-contracts/{address}");
    let value = get_json(http, &url).await?;
    Some((base, value))
}

/// Ask Sourcify to verify `address` against contracts whose runtime code is
/// similar. This WRITES: on success Sourcify publishes a verified match for
/// the address. Returns the job, polled for up to `wait`.
pub async fn sourcify_submit_similarity(
    http: &reqwest::Client,
    chain_id: u64,
    address: &str,
    wait: Duration,
) -> Result<serde_json::Value, String> {
    let base = sourcify_base();
    let url = format!("{base}/v2/verify/similarity/{chain_id}/{address}");
    let response = http
        .post(&url)
        .json(&serde_json::json!({}))
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;
    let status = response.status();
    let body: serde_json::Value = response.json().await.unwrap_or(serde_json::Value::Null);
    if !status.is_success() {
        let message = body["message"]
            .as_str()
            .or(body["customCode"].as_str())
            .unwrap_or("no message");
        return Err(format!("Sourcify answered HTTP {status}: {message}"));
    }
    let Some(id) = body["verificationId"].as_str().map(str::to_string) else {
        return Ok(body);
    };
    let deadline = std::time::Instant::now() + wait;
    let mut job = body;
    while std::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(1500)).await;
        let Some(v) = get_json(http, &format!("{base}/v2/verify/{id}")).await else {
            continue;
        };
        job = v;
        if job["isJobCompleted"].as_bool() == Some(true) {
            break;
        }
    }
    job["verificationId"] = serde_json::json!(id);
    Ok(job)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn match_types() {
        let m =
            sourcify_match_type(&json!({"match": "exact_match", "runtimeMatch": "exact_match"}));
        assert_eq!(m.match_type, "exact");
        assert_eq!(
            sourcify_match_type(&json!({"match": "match"})).match_type,
            "partial"
        );
        assert_eq!(sourcify_match_type(&json!({})).match_type, "not_verified");
        let b = blockscout_match_type(
            &json!({"is_verified": true, "is_fully_verified": false, "is_partially_verified": true}),
        );
        assert_eq!(b.match_type, "partial");
        assert_eq!(
            blockscout_match_type(&json!({"creation_bytecode": "0x"})).match_type,
            "unknown"
        );
    }

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
        assert_eq!(
            found.origin.to_string(),
            "Blockscout (eth.blockscout.com, unknown match)"
        );
        assert!(parse_blockscout(&json!({"is_verified": false}), "h").is_none());
    }
}
