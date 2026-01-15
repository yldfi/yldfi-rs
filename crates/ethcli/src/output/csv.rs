//! CSV output writer

use crate::abi::{DecodedLog, DecodedValue};
use crate::error::{OutputError, Result};
use crate::fetcher::{FetchLogs, FetchResult};
use crate::output::OutputWriter;
use alloy::rpc::types::Log;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// CSV output writer
pub struct CsvWriter {
    /// CSV writer
    writer: csv::Writer<Box<dyn Write + Send>>,
    /// Known column names (sorted for consistent output)
    columns: Vec<String>,
    /// Whether header has been written
    header_written: bool,
    /// Buffered rows (before header is determined)
    buffer: Vec<DecodedLog>,
    /// Max rows to buffer before writing header
    max_buffer: usize,
    /// Columns that appeared after header was written (with count of dropped values)
    dropped_columns: HashMap<String, usize>,
}

impl CsvWriter {
    /// Create a new CSV writer
    pub fn new(path: Option<&Path>) -> Result<Self> {
        let output: Box<dyn Write + Send> = if let Some(p) = path {
            let file = File::create(p)
                .map_err(|e| OutputError::FileCreate(format!("{}: {}", p.display(), e)))?;
            Box::new(BufWriter::new(file))
        } else {
            Box::new(BufWriter::new(io::stdout()))
        };

        let writer = csv::Writer::from_writer(output);

        Ok(Self {
            writer,
            columns: Vec::new(),
            header_written: false,
            buffer: Vec::new(),
            max_buffer: 1000, // Buffer more rows to determine schema
            dropped_columns: HashMap::new(),
        })
    }

    /// Collect all unique column names from a log
    fn collect_columns(&mut self, log: &DecodedLog) {
        for key in log.params.keys() {
            if !self.columns.contains(key) {
                self.columns.push(key.clone());
            }
        }
    }

    /// Write header row
    fn write_header(&mut self) -> Result<()> {
        // Sort columns for consistent output
        self.columns.sort();

        // Build header
        let mut header = vec![
            "block_number".to_string(),
            "transaction_hash".to_string(),
            "log_index".to_string(),
            "address".to_string(),
            "event_name".to_string(),
        ];
        header.extend(self.columns.clone());

        self.writer
            .write_record(&header)
            .map_err(|e| OutputError::CsvWrite(e.to_string()))?;

        self.header_written = true;
        Ok(())
    }

    /// Write a single decoded log as a row
    fn write_row(&mut self, log: &DecodedLog) -> Result<()> {
        let mut row = vec![
            log.block_number.to_string(),
            format!("{:#x}", log.transaction_hash),
            log.log_index.to_string(),
            format!("{:#x}", log.address),
            log.event_name.clone(),
        ];

        // Add parameter values in column order
        for col in &self.columns {
            let value = log
                .params
                .get(col)
                .map(Self::value_to_string)
                .unwrap_or_default();
            row.push(value);
        }

        self.writer
            .write_record(&row)
            .map_err(|e| OutputError::CsvWrite(e.to_string()))?;

        Ok(())
    }

    /// Escape a string value to prevent CSV formula injection
    /// Values starting with =, +, -, @, tab, or carriage return can be interpreted
    /// as formulas by spreadsheet applications (Excel, Google Sheets, etc.)
    /// Also handles leading whitespace that could bypass basic checks.
    fn escape_formula_injection(s: &str) -> String {
        if s.is_empty() {
            return s.to_string();
        }

        // Check both the first char and the first non-whitespace char
        // to prevent bypass via leading whitespace (e.g., " =formula")
        let first_char = s.chars().next().unwrap_or(' ');
        let first_non_ws = s.trim_start().chars().next().unwrap_or(' ');

        if matches!(first_char, '=' | '+' | '-' | '@' | '\t' | '\r')
            || matches!(first_non_ws, '=' | '+' | '-' | '@' | '\t' | '\r')
        {
            // Prefix with single quote to prevent formula interpretation
            format!("'{}", s)
        } else {
            s.to_string()
        }
    }

    /// Convert a decoded value to string
    fn value_to_string(value: &DecodedValue) -> String {
        let raw = match value {
            DecodedValue::Address(s) => s.clone(),
            DecodedValue::Uint(s) => s.clone(),
            DecodedValue::Int(s) => s.clone(),
            DecodedValue::Bool(b) => b.to_string(),
            DecodedValue::Bytes(s) => s.clone(),
            DecodedValue::String(s) => s.clone(),
            DecodedValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                format!("[{}]", items.join(","))
            }
            DecodedValue::Tuple(arr) => {
                let items: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                format!("({})", items.join(","))
            }
        };
        // Apply formula injection protection
        Self::escape_formula_injection(&raw)
    }

    /// Flush buffer and write header
    fn flush_buffer(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        // Collect columns from all buffered rows
        for log in &self.buffer {
            let mut new_cols: Vec<String> = log
                .params
                .keys()
                .filter(|k| !self.columns.contains(*k))
                .cloned()
                .collect();
            self.columns.append(&mut new_cols);
        }

        // Write header
        self.write_header()?;

        // Write buffered rows
        let buffer = std::mem::take(&mut self.buffer);
        for log in buffer {
            self.write_row(&log)?;
        }

        Ok(())
    }

    /// Write raw log (limited CSV support)
    fn write_raw_log(&mut self, log: &Log) -> Result<()> {
        if !self.header_written {
            self.writer
                .write_record([
                    "block_number",
                    "transaction_hash",
                    "log_index",
                    "address",
                    "topics",
                    "data",
                ])
                .map_err(|e| OutputError::CsvWrite(e.to_string()))?;
            self.header_written = true;
        }

        let topics: Vec<String> = log.topics().iter().map(|t| format!("{:#x}", t)).collect();

        self.writer
            .write_record(&[
                log.block_number.unwrap_or(0).to_string(),
                format!("{:#x}", log.transaction_hash.unwrap_or_default()),
                log.log_index.unwrap_or(0).to_string(),
                format!("{:#x}", log.address()),
                topics.join(";"),
                format!("0x{}", hex::encode(&log.data().data)),
            ])
            .map_err(|e| OutputError::CsvWrite(e.to_string()))?;

        Ok(())
    }
}

impl OutputWriter for CsvWriter {
    fn write_logs(&mut self, result: &FetchResult) -> Result<()> {
        match &result.logs {
            FetchLogs::Decoded(logs) => {
                for log in logs {
                    if !self.header_written {
                        // Buffer rows until we have enough to determine columns
                        self.collect_columns(log);
                        self.buffer.push(log.clone());

                        if self.buffer.len() >= self.max_buffer {
                            self.flush_buffer()?;
                        }
                    } else {
                        // Track new columns that appeared after header was written
                        for key in log.params.keys() {
                            if !self.columns.contains(key) {
                                let count = self.dropped_columns.entry(key.clone()).or_insert(0);
                                *count += 1;
                                // Warn immediately on first occurrence of a new column
                                if *count == 1 {
                                    tracing::warn!(
                                        "CSV: New column '{}' appeared after header was written - values will be dropped. \
                                         Consider using JSON or SQLite output for dynamic schemas.",
                                        key
                                    );
                                }
                            }
                        }

                        self.write_row(log)?;
                    }
                }
            }
            FetchLogs::Raw(logs) => {
                for log in logs {
                    self.write_raw_log(log)?;
                }
            }
        }
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        // Flush any remaining buffer
        if !self.header_written && !self.buffer.is_empty() {
            self.flush_buffer()?;
        }

        self.writer
            .flush()
            .map_err(|e| OutputError::CsvWrite(e.to_string()))?;

        // Report dropped columns (data loss warning)
        if !self.dropped_columns.is_empty() {
            let total_dropped: usize = self.dropped_columns.values().sum();
            tracing::error!(
                "CSV schema change detected: {} values in {} columns were dropped because they appeared after the header was written",
                total_dropped,
                self.dropped_columns.len()
            );
            for (col, count) in &self.dropped_columns {
                tracing::error!("  - Column '{}': {} values dropped", col, count);
            }
            tracing::warn!(
                "Consider using JSON or SQLite output format for events with dynamic schemas"
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_to_string() {
        assert_eq!(
            CsvWriter::value_to_string(&DecodedValue::Uint("1000".to_string())),
            "1000"
        );

        assert_eq!(
            CsvWriter::value_to_string(&DecodedValue::Bool(true)),
            "true"
        );

        assert_eq!(
            CsvWriter::value_to_string(&DecodedValue::Array(vec![
                DecodedValue::Uint("1".to_string()),
                DecodedValue::Uint("2".to_string()),
            ])),
            "[1,2]"
        );
    }
}
