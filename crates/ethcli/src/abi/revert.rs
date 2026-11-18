//! Decoding of revert data
//!
//! A reverted call returns either nothing, a standard `Error(string)`, a
//! compiler `Panic(uint256)`, or a custom error: a 4-byte selector followed
//! by ABI-encoded arguments. This module turns that data into a name and
//! decoded arguments, using (in order) the built-in forms, a contract ABI if
//! one is at hand, and candidate signatures from a signature database.

use alloy::dyn_abi::{DynSolType, DynSolValue, JsonAbiExt};
use alloy::json_abi::JsonAbi;
use alloy::primitives::U256;

const ERROR_STRING_SELECTOR: [u8; 4] = [0x08, 0xc3, 0x79, 0xa0];
const PANIC_SELECTOR: [u8; 4] = [0x4e, 0x48, 0x7b, 0x71];

/// What kind of revert the data describes
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RevertKind {
    /// No revert data at all (`revert()` or `require(cond)` with no message)
    Empty,
    /// `Error(string)`: `revert("...")` or `require(cond, "...")`
    ErrorString,
    /// `Panic(uint256)`: a compiler-inserted check failed
    Panic,
    /// A custom error whose signature was found
    Custom,
    /// A selector no source could name
    Unknown,
}

/// A decoded revert
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DecodedRevert {
    /// Kind of revert
    pub kind: RevertKind,
    /// 4-byte selector, when there is one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// Error signature, e.g. `InsufficientBalance(uint256,uint256)`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Decoded arguments (integers as decimal strings)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<serde_json::Value>,
    /// One-line description for people
    pub message: String,
    /// The revert data as 0x hex
    pub raw: String,
}

/// Meaning of a Solidity panic code
fn panic_meaning(code: U256) -> &'static str {
    match u64::try_from(code).unwrap_or(u64::MAX) {
        0x00 => "generic compiler panic",
        0x01 => "assertion failed",
        0x11 => "arithmetic overflow or underflow",
        0x12 => "division or modulo by zero",
        0x21 => "invalid enum value",
        0x22 => "incorrectly encoded storage byte array",
        0x31 => "pop on an empty array",
        0x32 => "array index out of bounds",
        0x41 => "out of memory or too large an allocation",
        0x51 => "call to an uninitialised internal function",
        _ => "unknown panic code",
    }
}

fn hex_of(data: &[u8]) -> String {
    format!("0x{}", hex::encode(data))
}

/// A decoded value as JSON: integers as decimal strings, addresses
/// checksummed, bytes as 0x hex, arrays and tuples as arrays
fn value_to_json(value: &DynSolValue) -> serde_json::Value {
    use serde_json::Value;
    match value {
        DynSolValue::Bool(b) => Value::Bool(*b),
        DynSolValue::Int(i, _) => Value::String(i.to_string()),
        DynSolValue::Uint(u, _) => Value::String(u.to_string()),
        DynSolValue::Address(a) => Value::String(a.to_checksum(None)),
        DynSolValue::String(s) => Value::String(s.clone()),
        DynSolValue::Bytes(b) => Value::String(hex_of(b)),
        DynSolValue::FixedBytes(word, size) => Value::String(hex_of(&word[..*size])),
        DynSolValue::Function(f) => Value::String(hex_of(f.as_slice())),
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) | DynSolValue::Tuple(items) => {
            Value::Array(items.iter().map(value_to_json).collect())
        }
    }
}

fn args_text(args: &[serde_json::Value]) -> String {
    args.iter()
        .map(|a| match a {
            serde_json::Value::String(s) => s.clone(),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Decode the forms that need no ABI: empty data, `Error(string)` and
/// `Panic(uint256)`. Returns `None` for anything else.
pub fn decode_builtin(data: &[u8]) -> Option<DecodedRevert> {
    if data.is_empty() {
        return Some(DecodedRevert {
            kind: RevertKind::Empty,
            selector: None,
            signature: None,
            args: Vec::new(),
            message: "reverted with no data".to_string(),
            raw: "0x".to_string(),
        });
    }
    if data.len() < 4 {
        return None;
    }
    if data[..4] == ERROR_STRING_SELECTOR {
        if let Ok(DynSolValue::String(reason)) = DynSolType::String.abi_decode(&data[4..]) {
            return Some(DecodedRevert {
                kind: RevertKind::ErrorString,
                selector: Some(hex_of(&data[..4])),
                signature: Some("Error(string)".to_string()),
                message: format!("{reason:?}"),
                args: vec![serde_json::Value::String(reason)],
                raw: hex_of(data),
            });
        }
    }
    if data[..4] == PANIC_SELECTOR {
        if let Ok(DynSolValue::Uint(code, _)) = DynSolType::Uint(256).abi_decode(&data[4..]) {
            return Some(DecodedRevert {
                kind: RevertKind::Panic,
                selector: Some(hex_of(&data[..4])),
                signature: Some("Panic(uint256)".to_string()),
                message: format!("Panic({code:#x}): {}", panic_meaning(code)),
                args: vec![serde_json::Value::String(code.to_string())],
                raw: hex_of(data),
            });
        }
    }
    None
}

/// Decode a custom error against the errors declared in `abi`
pub fn decode_with_abi(data: &[u8], abi: &JsonAbi) -> Option<DecodedRevert> {
    if data.len() < 4 {
        return None;
    }
    let error = abi
        .errors()
        .find(|e| e.selector().as_slice() == &data[..4])?;
    let values = error.abi_decode_input(&data[4..]).ok()?;
    let args: Vec<serde_json::Value> = values.iter().map(value_to_json).collect();
    let named = error
        .inputs
        .iter()
        .zip(&args)
        .map(|(input, arg)| {
            let text = match arg {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            if input.name.is_empty() {
                text
            } else {
                format!("{}={text}", input.name)
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    Some(DecodedRevert {
        kind: RevertKind::Custom,
        selector: Some(hex_of(&data[..4])),
        signature: Some(error.signature()),
        message: format!("{}({named})", error.name),
        args,
        raw: hex_of(data),
    })
}

/// The argument types of `Name(type,type)`, as one tuple type
fn signature_types(signature: &str) -> Option<DynSolType> {
    let open = signature.find('(')?;
    let close = signature.rfind(')')?;
    (close > open).then_some(())?;
    DynSolType::parse(&format!("({})", &signature[open + 1..close])).ok()
}

/// Decode a custom error with a known signature such as
/// `InsufficientBalance(uint256,uint256)`. Returns `None` if the signature's
/// selector is not the data's, or the arguments do not decode as its types.
pub fn decode_with_signature(data: &[u8], signature: &str) -> Option<DecodedRevert> {
    if data.len() < 4 {
        return None;
    }
    let signature = signature.trim();
    let selector = alloy::primitives::keccak256(signature.as_bytes());
    if selector[..4] != data[..4] {
        return None;
    }
    let args: Vec<serde_json::Value> = match signature_types(signature)? {
        DynSolType::Tuple(types) if types.is_empty() => {
            // No arguments: the data must be the selector alone
            (data.len() == 4).then(Vec::new)?
        }
        ty => match ty.abi_decode_params(&data[4..]).ok()? {
            DynSolValue::Tuple(values) => values.iter().map(value_to_json).collect(),
            single => vec![value_to_json(&single)],
        },
    };
    let name = signature.split('(').next().unwrap_or(signature);
    Some(DecodedRevert {
        kind: RevertKind::Custom,
        selector: Some(hex_of(&data[..4])),
        signature: Some(signature.to_string()),
        message: format!("{name}({})", args_text(&args)),
        args,
        raw: hex_of(data),
    })
}

/// A selector nothing could name
pub fn unknown(data: &[u8]) -> DecodedRevert {
    let selector = (data.len() >= 4).then(|| hex_of(&data[..4]));
    DecodedRevert {
        kind: RevertKind::Unknown,
        message: match &selector {
            Some(s) => format!("custom error {s} (signature not found)"),
            None => "revert data too short to hold a selector".to_string(),
        },
        selector,
        signature: None,
        args: Vec::new(),
        raw: hex_of(data),
    }
}

/// Decode revert data using everything available: the built-in forms, then
/// `abi`, then signatures found by `lookup` for the selector.
///
/// `lookup` receives the selector as 0x hex and returns candidate
/// signatures, most likely first; the first whose types decode the
/// arguments wins.
pub async fn decode_revert<F, Fut>(data: &[u8], abi: Option<&JsonAbi>, lookup: F) -> DecodedRevert
where
    F: FnOnce(String) -> Fut,
    Fut: std::future::Future<Output = Option<Vec<String>>>,
{
    if let Some(decoded) = decode_builtin(data) {
        return decoded;
    }
    if let Some(decoded) = abi.and_then(|abi| decode_with_abi(data, abi)) {
        return decoded;
    }
    if data.len() >= 4 {
        if let Some(candidates) = lookup(hex_of(&data[..4])).await {
            if let Some(decoded) = candidates
                .iter()
                .find_map(|signature| decode_with_signature(data, signature))
            {
                return decoded;
            }
        }
    }
    unknown(data)
}

/// Revert data carried by a failed `eth_call`, if this error is a revert.
///
/// `Some(empty)` means the call reverted without data; `None` means the
/// error is not a revert (a transport failure, a rate limit, ...).
pub fn revert_data_of(
    err: &alloy::transports::RpcError<alloy::transports::TransportErrorKind>,
) -> Option<alloy::primitives::Bytes> {
    let payload = err.as_error_resp()?;
    if let Some(data) = payload.as_revert_data() {
        return Some(data);
    }
    payload
        .message
        .to_lowercase()
        .contains("revert")
        .then(alloy::primitives::Bytes::new)
}

/// Decode revert data, looking unknown selectors up in the signature
/// database for `chain`
pub async fn describe(
    data: &[u8],
    abi: Option<&JsonAbi>,
    chain: crate::config::Chain,
) -> DecodedRevert {
    decode_revert(data, abi, |selector| async move {
        let client = crate::etherscan::Client::new(chain, None).ok()?;
        client.lookup_selector_all(&selector).await
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::keccak256;

    fn custom(signature: &str, values: Vec<DynSolValue>) -> Vec<u8> {
        let mut data = keccak256(signature.as_bytes())[..4].to_vec();
        data.extend(DynSolValue::Tuple(values).abi_encode_params());
        data
    }

    #[test]
    fn decodes_the_builtin_forms() {
        let empty = decode_builtin(&[]).unwrap();
        assert_eq!(empty.kind, RevertKind::Empty);

        let mut data = ERROR_STRING_SELECTOR.to_vec();
        data.extend(DynSolValue::String("not owner".into()).abi_encode());
        let error = decode_builtin(&data).unwrap();
        assert_eq!(error.kind, RevertKind::ErrorString);
        assert_eq!(error.message, "\"not owner\"");
        assert_eq!(error.args, vec![serde_json::json!("not owner")]);

        let mut data = PANIC_SELECTOR.to_vec();
        data.extend(DynSolValue::Uint(U256::from(0x11), 256).abi_encode());
        let panic = decode_builtin(&data).unwrap();
        assert_eq!(panic.kind, RevertKind::Panic);
        assert_eq!(
            panic.message,
            "Panic(0x11): arithmetic overflow or underflow"
        );

        assert!(decode_builtin(&[1, 2, 3, 4, 5]).is_none());
        assert!(decode_builtin(&[1, 2]).is_none());
    }

    #[test]
    fn decodes_a_custom_error_from_its_signature() {
        let signature = "InsufficientBalance(uint256,uint256)";
        let data = custom(
            signature,
            vec![
                DynSolValue::Uint(U256::from(5), 256),
                DynSolValue::Uint(U256::from(10), 256),
            ],
        );
        let decoded = decode_with_signature(&data, signature).unwrap();
        assert_eq!(decoded.kind, RevertKind::Custom);
        assert_eq!(decoded.message, "InsufficientBalance(5, 10)");
        assert_eq!(
            decoded.args,
            vec![serde_json::json!("5"), serde_json::json!("10")]
        );

        // A signature with another selector, or types that do not fit
        assert!(decode_with_signature(&data, "Other(uint256,uint256)").is_none());
        assert!(decode_with_signature(&data[..20], signature).is_none());

        // No arguments
        let bare = keccak256(b"Paused()")[..4].to_vec();
        assert_eq!(
            decode_with_signature(&bare, "Paused()").unwrap().message,
            "Paused()"
        );
    }

    #[test]
    fn decodes_a_custom_error_from_an_abi_with_argument_names() {
        let abi: JsonAbi = serde_json::from_str(
            r#"[{"type":"error","name":"TooLow","inputs":[
                {"name":"have","type":"uint256"},{"name":"who","type":"address"}]}]"#,
        )
        .unwrap();
        let data = custom(
            "TooLow(uint256,address)",
            vec![
                DynSolValue::Uint(U256::from(7), 256),
                DynSolValue::Address(alloy::primitives::Address::ZERO),
            ],
        );
        let decoded = decode_with_abi(&data, &abi).unwrap();
        assert_eq!(
            decoded.message,
            "TooLow(have=7, who=0x0000000000000000000000000000000000000000)"
        );
        assert_eq!(
            decoded.signature.as_deref(),
            Some("TooLow(uint256,address)")
        );
        assert!(decode_with_abi(&[9, 9, 9, 9], &abi).is_none());
    }

    #[tokio::test]
    async fn tries_builtin_then_abi_then_lookup_then_gives_up() {
        let signature = "Nope(uint8)";
        let data = custom(signature, vec![DynSolValue::Uint(U256::from(3), 8)]);

        // The lookup offers a wrong candidate first; the fitting one wins
        let decoded = decode_revert(&data, None, |selector| async move {
            assert_eq!(selector.len(), 10);
            Some(vec!["Wrong(string)".to_string(), "Nope(uint8)".to_string()])
        })
        .await;
        assert_eq!(decoded.message, "Nope(3)");

        let unknown = decode_revert(&data, None, |_| async { None }).await;
        assert_eq!(unknown.kind, RevertKind::Unknown);
        assert!(unknown.message.contains("signature not found"));

        let empty =
            decode_revert(&[], None, |_| async { panic!("no lookup for empty data") }).await;
        assert_eq!(empty.kind, RevertKind::Empty);
    }
}
