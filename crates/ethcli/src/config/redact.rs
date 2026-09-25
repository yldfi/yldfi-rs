//! Redaction of secrets in the config file for display (`ethcli config show`)

use crate::utils::url::{redact_url, url_has_hidden_parts};

/// Placeholder shown instead of a secret value
pub const REDACTED: &str = "<redacted>";

/// Key-name fragments that mark a value as secret
const SECRET_KEY_PARTS: &[&str] = &[
    "key",
    "token",
    "secret",
    "password",
    "passphrase",
    "private",
    "auth",
    "mnemonic",
];

fn is_secret_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    SECRET_KEY_PARTS.iter().any(|part| key.contains(part))
}

fn looks_like_url(value: &str) -> bool {
    let value = value.trim_start();
    ["http://", "https://", "ws://", "wss://"]
        .iter()
        .any(|scheme| value.starts_with(scheme))
}

fn redact_value(key: Option<&str>, value: &mut toml::Value) {
    match value {
        toml::Value::String(s) => {
            if key.is_some_and(is_secret_key) {
                if !s.is_empty() {
                    *s = REDACTED.to_string();
                }
            } else if looks_like_url(s) && url_has_hidden_parts(s) {
                *s = format!("{}/{REDACTED}", redact_url(s));
            }
        }
        toml::Value::Array(items) => {
            for item in items {
                redact_value(key, item);
            }
        }
        toml::Value::Table(table) => redact_table(table),
        _ => {}
    }
}

fn redact_table(table: &mut toml::map::Map<String, toml::Value>) {
    for (key, value) in table.iter_mut() {
        redact_value(Some(key), value);
    }
}

/// Return `content` (a config TOML document) with secrets masked.
///
/// Values whose key name looks secret (`*key*`, `*token*`, `*secret*`, ...)
/// are replaced with [`REDACTED`], and URLs that carry credentials, paths or
/// query strings (where RPC providers put API keys) are cut down to
/// `scheme://host[:port]/<redacted>`. Comments are not preserved.
///
/// # Errors
///
/// Returns an error if `content` is not valid TOML.
pub fn redact_config_toml(content: &str) -> Result<String, toml::de::Error> {
    let mut table: toml::map::Map<String, toml::Value> = toml::from_str(content)?;
    redact_table(&mut table);
    Ok(toml::to_string(&table).unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
etherscan_api_key = "ETHERSCAN-SECRET"
debug_rpc_urls = ["https://user:pw@debug.example.com/v2/DEBUG-SECRET"]

[settings]
concurrency = 5

[[endpoints]]
url = "https://eth-mainnet.g.alchemy.com/v2/ALCHEMY-SECRET"
priority = 5

[[endpoints]]
url = "https://ethereum-rpc.publicnode.com"

[tenderly]
access_key = "TENDERLY-SECRET"
account = "my-account"

[alchemy]
api_key = "ALCHEMY-KEY-SECRET"
notify_token = "NOTIFY-SECRET"

[chainlink]
api_secret = "CL-SECRET"
rest_url = "https://api.dataengine.chain.link"
"#;

    #[test]
    fn masks_every_secret() {
        let out = redact_config_toml(SAMPLE).unwrap();
        for secret in [
            "ETHERSCAN-SECRET",
            "DEBUG-SECRET",
            "user:pw",
            "ALCHEMY-SECRET",
            "TENDERLY-SECRET",
            "ALCHEMY-KEY-SECRET",
            "NOTIFY-SECRET",
            "CL-SECRET",
        ] {
            assert!(!out.contains(secret), "{secret} leaked:\n{out}");
        }
    }

    #[test]
    fn keeps_non_secret_values() {
        let out = redact_config_toml(SAMPLE).unwrap();
        assert!(out.contains("concurrency = 5"));
        assert!(out.contains("account = \"my-account\""));
        // Plain URLs with nothing to hide are left intact
        assert!(out.contains("https://ethereum-rpc.publicnode.com"));
        assert!(out.contains("https://api.dataengine.chain.link"));
        // Keyed URLs keep their host so endpoints stay identifiable
        assert!(out.contains("https://eth-mainnet.g.alchemy.com/<redacted>"));
        assert!(out.contains("https://debug.example.com/<redacted>"));
    }

    #[test]
    fn empty_secret_stays_empty() {
        let out = redact_config_toml("etherscan_api_key = \"\"\n").unwrap();
        assert!(out.contains("etherscan_api_key = \"\""));
    }

    #[test]
    fn invalid_toml_is_an_error() {
        assert!(redact_config_toml("not = [valid").is_err());
    }
}
