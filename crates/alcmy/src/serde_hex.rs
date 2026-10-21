//! Serde helpers for JSON-RPC hex quantities

use serde::Serializer;

/// Serialize an optional `u64` as a `0x`-prefixed hex quantity (e.g. `"0x1"`).
///
/// Use together with `skip_serializing_if = "Option::is_none"`.
#[allow(clippy::ref_option)]
pub(crate) fn opt_u64_as_hex<S: Serializer>(
    value: &Option<u64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    match value {
        Some(n) => serializer.serialize_str(&format!("{n:#x}")),
        None => serializer.serialize_none(),
    }
}
