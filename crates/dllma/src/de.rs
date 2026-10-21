//! Shared serde helpers for loosely typed `DefiLlama` responses
//!
//! These go through `serde_json::Value` rather than `#[serde(flatten)]`:
//! with `serde_json/arbitrary_precision` enabled (ethcli pulls it in
//! workspace-wide) buffered numbers don't deserialize into primitives.

use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer};

/// Read a JSON number or numeric string as `u64`
pub(crate) fn value_to_u64(v: &serde_json::Value) -> Option<u64> {
    match v {
        serde_json::Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().filter(|f| *f >= 0.0).map(|f| f as u64)),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Read a JSON number or numeric string as `f64`
pub(crate) fn value_to_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

/// Deserialize an optional `u64` sent as either a number or a numeric string
pub(crate) fn opt_flex_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(deserializer)?;
    match v {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => value_to_u64(&v)
            .map(Some)
            .ok_or_else(|| D::Error::custom(format!("expected unsigned integer, got {v}"))),
    }
}

/// Deserialize a value that may arrive either as JSON or as a string holding JSON
pub(crate) fn json_or_json_string<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(s) => serde_json::from_str(&s).map_err(D::Error::custom),
        v => serde_json::from_value(v).map_err(D::Error::custom),
    }
}

/// Deserialize an optional string that may also be sent as a bare number
pub(crate) fn opt_string_or_number<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(v) => Err(D::Error::custom(format!(
            "expected string or number, got {v}"
        ))),
    }
}
