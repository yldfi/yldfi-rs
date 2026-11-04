//! Shared request and JSON helpers for the exchange modules

use crate::error::{Error, Result};
use crate::types::Exchange;
use serde_json::Value;

/// Longest error body kept in an error message
const MAX_ERROR_BODY: usize = 300;

/// GET `url` with `query` and return the JSON body.
///
/// A non-2xx status becomes [`Error::Api`], using the exchange's own
/// `code`/`msg` fields when the body has them.
pub(crate) async fn get_json(
    http: &reqwest::Client,
    exchange: Exchange,
    url: &str,
    query: &[(&str, String)],
) -> Result<Value> {
    let response = http
        .get(url)
        .query(query)
        .send()
        .await
        .map_err(|source| Error::Http { exchange, source })?;
    read_json(exchange, response).await
}

/// POST `body` as JSON to `url` and return the JSON body.
pub(crate) async fn post_json(
    http: &reqwest::Client,
    exchange: Exchange,
    url: &str,
    body: &Value,
) -> Result<Value> {
    let response = http
        .post(url)
        .json(body)
        .send()
        .await
        .map_err(|source| Error::Http { exchange, source })?;
    read_json(exchange, response).await
}

async fn read_json(exchange: Exchange, response: reqwest::Response) -> Result<Value> {
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|source| Error::Http { exchange, source })?;
    let json = serde_json::from_str::<Value>(&text);
    if !status.is_success() {
        let (code, message) = match &json {
            Ok(body) => error_fields(body),
            Err(_) => (None, None),
        };
        return Err(Error::Api {
            exchange,
            code: code.unwrap_or_else(|| status.as_u16().to_string()),
            message: message.unwrap_or_else(|| truncate(&text)),
        });
    }
    json.map_err(|e| Error::Parse {
        exchange,
        message: format!("invalid JSON ({e}): {}", truncate(&text)),
    })
}

/// Common error fields: `code` + `msg`/`message` (Binance, Bitget, OKX,
/// Bybit, Lighter), or `error` (Hyperliquid, Kraken, Coinbase)
fn error_fields(body: &Value) -> (Option<String>, Option<String>) {
    let code = body
        .get("code")
        .or_else(|| body.get("retCode"))
        .and_then(scalar);
    let message = ["msg", "message", "retMsg", "error"].iter().find_map(|k| {
        body.get(*k)
            .and_then(|v| scalar(v).or_else(|| first_string(v)))
    });
    (code, message)
}

fn first_string(v: &Value) -> Option<String> {
    v.as_array()?.iter().find_map(scalar)
}

fn truncate(s: &str) -> String {
    let s = s.trim();
    match s.char_indices().nth(MAX_ERROR_BODY) {
        Some((i, _)) => format!("{}…", &s[..i]),
        None => s.to_string(),
    }
}

/// A JSON string or number as a string; `None` for null, empty or other types
pub(crate) fn scalar(v: &Value) -> Option<String> {
    match v {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// `obj[key]` as a string (see [`scalar`])
pub(crate) fn field(obj: &Value, key: &str) -> Option<String> {
    obj.get(key).and_then(scalar)
}

/// `obj[key]` as an integer, accepting a JSON number or a numeric string
pub(crate) fn int_field(obj: &Value, key: &str) -> Option<i64> {
    obj.get(key).and_then(as_i64)
}

/// A JSON number or numeric string as `i64`
pub(crate) fn as_i64(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.parse().ok(),
        _ => None,
    }
}

/// `obj[key]` as a string, or a parse error naming the field
pub(crate) fn require(exchange: Exchange, obj: &Value, key: &str) -> Result<String> {
    field(obj, key).ok_or_else(|| parse_error(exchange, format!("missing field {key:?}")))
}

/// `obj[key]` as an array, or a parse error naming the field
pub(crate) fn array<'a>(exchange: Exchange, obj: &'a Value, key: &str) -> Result<&'a Vec<Value>> {
    obj.get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| parse_error(exchange, format!("missing array {key:?}")))
}

/// Element `i` of a JSON array row as a string
pub(crate) fn cell(exchange: Exchange, row: &Value, i: usize) -> Result<String> {
    row.get(i)
        .and_then(scalar)
        .ok_or_else(|| parse_error(exchange, format!("row is missing column {i}")))
}

pub(crate) fn parse_error(exchange: Exchange, message: impl Into<String>) -> Error {
    Error::Parse {
        exchange,
        message: message.into(),
    }
}

/// `[[price, amount, ...], ...]` order book rows → levels
pub(crate) fn levels(exchange: Exchange, rows: &[Value]) -> Result<Vec<crate::Level>> {
    rows.iter()
        .map(|row| {
            Ok(crate::Level {
                price: cell(exchange, row, 0)?,
                amount: cell(exchange, row, 1)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn error_fields_cover_exchange_shapes() {
        let f = |v: Value| error_fields(&v);
        assert_eq!(
            f(json!({"code": -1121, "msg": "Invalid symbol."})),
            (Some("-1121".into()), Some("Invalid symbol.".into()))
        );
        assert_eq!(
            f(json!({"retCode": 10001, "retMsg": "params error"})),
            (Some("10001".into()), Some("params error".into()))
        );
        assert_eq!(
            f(json!({"error": ["EQuery:Unknown asset pair"], "result": {}})),
            (None, Some("EQuery:Unknown asset pair".into()))
        );
        assert_eq!(
            f(json!({"message": "NotFound"})),
            (None, Some("NotFound".into()))
        );
    }

    #[test]
    fn scalar_accepts_strings_and_numbers_only() {
        assert_eq!(scalar(&json!("1.5")), Some("1.5".into()));
        assert_eq!(scalar(&json!(2)), Some("2".into()));
        assert_eq!(scalar(&json!(0.25)), Some("0.25".into()));
        assert_eq!(scalar(&json!("")), None);
        assert_eq!(scalar(&json!(null)), None);
        assert_eq!(as_i64(&json!("1790352000000")), Some(1_790_352_000_000));
    }
}
