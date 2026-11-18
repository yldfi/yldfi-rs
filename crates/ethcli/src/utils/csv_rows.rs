//! CSV for list-shaped JSON output
//!
//! Turns rows of JSON objects into CSV with one column per field. Nested
//! objects become `parent.child` columns; arrays are written as JSON text.
//! Columns named in `first` come first, in that order; the rest follow in
//! the order they appear (alphabetical for objects built with `json!`).

use std::fmt::Write;

/// Flatten one value into `(column, text)` cells under `prefix`
fn flatten(prefix: &str, value: &serde_json::Value, out: &mut Vec<(String, String)>) {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let column = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(&column, child, out);
            }
        }
        Value::Null => out.push((prefix.to_string(), String::new())),
        Value::String(text) => out.push((prefix.to_string(), text.clone())),
        Value::Array(_) => out.push((prefix.to_string(), value.to_string())),
        other => out.push((prefix.to_string(), other.to_string())),
    }
}

/// A cell made safe for spreadsheets: text that a spreadsheet would run as a
/// formula (starting with `=`, `+`, `-`, `@`, tab or carriage return) gets a
/// leading `'`, unless it is a plain number such as `-3` or `+1.5`. Values can
/// come from contracts (token symbols, revert messages), so this matters.
fn spreadsheet_safe(text: &str) -> std::borrow::Cow<'_, str> {
    let risky = text.starts_with(['=', '+', '-', '@', '\t', '\r']);
    if risky && text.parse::<f64>().is_err() {
        std::borrow::Cow::Owned(format!("'{text}"))
    } else {
        std::borrow::Cow::Borrowed(text)
    }
}

/// Rows of JSON values as CSV text, with a header line. `first` lists the
/// columns to put first (names not present are skipped).
pub fn json_rows_to_csv(rows: &[serde_json::Value], first: &[&str]) -> anyhow::Result<String> {
    let flat: Vec<Vec<(String, String)>> = rows
        .iter()
        .map(|row| {
            let mut cells = Vec::new();
            flatten("", row, &mut cells);
            cells
        })
        .collect();

    let mut columns: Vec<String> = Vec::new();
    for name in first {
        if flat
            .iter()
            .any(|cells| cells.iter().any(|(c, _)| c == name))
        {
            columns.push(name.to_string());
        }
    }
    for cells in &flat {
        for (column, _) in cells {
            if !columns.contains(column) {
                columns.push(column.clone());
            }
        }
    }

    if columns.is_empty() {
        return Ok(String::new());
    }
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer.write_record(&columns)?;
    for cells in &flat {
        let record: Vec<std::borrow::Cow<'_, str>> = columns
            .iter()
            .map(|column| {
                cells
                    .iter()
                    .find(|(c, _)| c == column)
                    .map(|(_, text)| spreadsheet_safe(text))
                    .unwrap_or_default()
            })
            .collect();
        writer.write_record(record.iter().map(|cell| cell.as_ref()))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|e| anyhow::anyhow!("Failed to write CSV: {e}"))?;
    let mut text = String::from_utf8(bytes)?;
    if !text.ends_with('\n') {
        let _ = writeln!(text);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn columns_follow_first_appearance_and_nesting_flattens() {
        let rows = vec![
            json!({"hash": "0x1", "value": "5", "meta": {"name": "A"}, "logs": [1, 2]}),
            json!({"hash": "0x2", "extra": true, "value": null}),
        ];
        let csv = json_rows_to_csv(&rows, &["value", "hash", "missing"]).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "value,hash,logs,meta.name,extra");
        assert_eq!(lines[1], "5,0x1,\"[1,2]\",A,");
        assert_eq!(lines[2], ",0x2,,,true");
    }

    #[test]
    fn formula_text_is_neutralised_but_numbers_are_not() {
        let rows = vec![
            json!({"symbol": "=HYPERLINK(\"x\")", "change": "-3000000", "a": "+1.5", "b": "@x"}),
        ];
        let csv = json_rows_to_csv(&rows, &["symbol", "change", "a", "b"]).unwrap();
        let line = csv.lines().nth(1).unwrap();
        assert!(line.starts_with("\"'=HYPERLINK(\"\"x\"\")\""), "{line}");
        assert!(line.contains(",-3000000,"), "{line}");
        assert!(line.contains(",+1.5,"), "{line}");
        assert!(line.ends_with(",'@x"), "{line}");
    }

    #[test]
    fn no_rows_gives_an_empty_header() {
        assert_eq!(json_rows_to_csv(&[], &["a"]).unwrap(), "");
    }
}
