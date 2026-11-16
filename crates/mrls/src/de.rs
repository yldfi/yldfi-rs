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

/// Deserialize an integer that Moralis may send as a number or as a string
/// containing one (an empty string or `"null"` reads as `None`).
pub(crate) fn string_or_integer<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr + TryFrom<i64> + TryFrom<u64>,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let out_of_range = || de::Error::custom("integer out of range");
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(number) => {
            if let Some(v) = number.as_i64() {
                T::try_from(v).map(Some).map_err(|_| out_of_range())
            } else if let Some(v) = number.as_u64() {
                T::try_from(v).map(Some).map_err(|_| out_of_range())
            } else {
                Err(de::Error::custom(format!(
                    "expected an integer, got {number}"
                )))
            }
        }
        serde_json::Value::String(value) => {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }
        value => Err(de::Error::custom(format!(
            "expected an integer or a string containing one, got {value}"
        ))),
    }
}

/// Deserialize a decimal amount Moralis may send as a number or a string,
/// keeping its text so no precision is lost (an empty string or `"null"`
/// reads as `None`).
pub(crate) fn number_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(number) => Ok(Some(number.to_string())),
        serde_json::Value::String(value) => {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        value => Err(de::Error::custom(format!(
            "expected a number or a string containing one, got {value}"
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

#[cfg(test)]
mod tests {
    #[derive(serde::Deserialize)]
    struct Ints {
        #[serde(default, deserialize_with = "super::string_or_integer")]
        a: Option<i32>,
        #[serde(default, deserialize_with = "super::string_or_integer")]
        b: Option<u64>,
    }

    #[test]
    fn string_or_integer_reads_numbers_and_strings() {
        let v: Ints = serde_json::from_str(r#"{"a": "42", "b": 7}"#).unwrap();
        assert_eq!((v.a, v.b), (Some(42), Some(7)));
        let v: Ints = serde_json::from_str(r#"{"a": "", "b": null}"#).unwrap();
        assert_eq!((v.a, v.b), (None, None));
        let v: Ints = serde_json::from_str("{}").unwrap();
        assert_eq!((v.a, v.b), (None, None));
        assert!(serde_json::from_str::<Ints>(r#"{"a": "x"}"#).is_err());
        assert!(serde_json::from_str::<Ints>(r#"{"a": 3000000000}"#).is_err());
    }

    #[derive(serde::Deserialize)]
    struct Amounts {
        #[serde(default, deserialize_with = "super::number_as_string")]
        a: Option<String>,
        #[serde(default, deserialize_with = "super::number_as_string")]
        b: Option<String>,
    }

    #[test]
    fn number_as_string_keeps_text() {
        let v: Amounts =
            serde_json::from_str(r#"{"a": "1.000000000000000001", "b": 12345678901234567890}"#)
                .unwrap();
        assert_eq!(v.a.as_deref(), Some("1.000000000000000001"));
        assert_eq!(v.b.as_deref(), Some("12345678901234567890"));
        let v: Amounts = serde_json::from_str(r#"{"a": "", "b": null}"#).unwrap();
        assert_eq!((v.a, v.b), (None, None));
        assert!(serde_json::from_str::<Amounts>(r#"{"a": true}"#).is_err());
    }
}
