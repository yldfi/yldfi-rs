//! Serde helpers: lenient deserializers for Moralis numeric fields and
//! query-parameter serializers

use serde::de::{self, Deserialize, Deserializer, Visitor};

/// Deserialize a value that could be either a float, integer, or a string containing a number.
/// Moralis API sometimes returns numeric values as strings.
pub(crate) fn string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| de::Error::custom("number cannot be represented as f64")),
        serde_json::Value::String(value) => {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }
        value => Err(de::Error::custom(format!(
            "expected a float, integer, or string containing a number, got {value}"
        ))),
    }
}

/// Deserialize a value that could be either a string or an integer.
/// Moralis API sometimes returns block numbers and similar fields as integers
/// rather than strings.
pub(crate) fn string_or_int<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrIntVisitor;

    impl<'de> Visitor<'de> for StringOrIntVisitor {
        type Value = Option<String>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a string or an integer")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v.to_string()))
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrIntVisitor)
}

/// `skip_serializing_if` helper: skip unless `Some(true)`
#[allow(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]
pub(crate) fn not_true(value: &Option<bool>) -> bool {
    *value != Some(true)
}

/// Serialize `Some(true)` as the `include=internal_transactions` query value
#[allow(clippy::ref_option, clippy::trivially_copy_pass_by_ref)]
pub(crate) fn include_internal_transactions<S>(
    _value: &Option<bool>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str("internal_transactions")
}
