//! SQLite output writer

use crate::abi::{DecodedLog, DecodedValue};
use crate::error::{OutputError, Result};
use crate::fetcher::{FetchLogs, FetchResult};
use crate::output::OutputWriter;
use alloy::rpc::types::Log;
use rusqlite::{params, Connection};
use std::path::Path;

use std::collections::HashMap as StdHashMap;

/// SQLite output writer
pub struct SqliteWriter {
    /// Database connection
    conn: Connection,
    /// Known columns (original names)
    columns: Vec<String>,
    /// Mapping from original column name to sanitized column name (handles collisions)
    column_name_map: StdHashMap<String, String>,
    /// Set of sanitized names in use (to detect collisions)
    sanitized_names: std::collections::HashSet<String>,
    /// Table created
    table_created: bool,
    /// Batch buffer
    buffer: Vec<DecodedLog>,
    /// Batch size for inserts
    batch_size: usize,
}

impl SqliteWriter {
    /// Create a new SQLite writer
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).map_err(OutputError::Sqlite)?;

        // Enable WAL mode for better performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .map_err(OutputError::Sqlite)?;

        Ok(Self {
            conn,
            columns: Vec::new(),
            column_name_map: StdHashMap::new(),
            sanitized_names: std::collections::HashSet::new(),
            table_created: false,
            buffer: Vec::new(),
            batch_size: 1000,
        })
    }

    /// Create the events table
    fn create_table(&mut self) -> Result<()> {
        // Base columns
        let mut create_sql = String::from(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_number INTEGER NOT NULL,
                transaction_hash TEXT NOT NULL,
                log_index INTEGER NOT NULL,
                address TEXT NOT NULL,
                event_name TEXT NOT NULL,
                event_signature TEXT NOT NULL,
                topics TEXT,
                data BLOB",
        );

        // Add dynamic columns with collision-safe names
        for col in &self.columns.clone() {
            let safe_col = self.get_sanitized_column_name(col);
            create_sql.push_str(&format!(",\n                {} TEXT", safe_col));
        }

        create_sql.push_str(
            "
            );
            CREATE INDEX IF NOT EXISTS idx_block ON events(block_number);
            CREATE INDEX IF NOT EXISTS idx_address ON events(address);
            CREATE INDEX IF NOT EXISTS idx_event ON events(event_name);",
        );

        self.conn
            .execute_batch(&create_sql)
            .map_err(OutputError::Sqlite)?;
        self.table_created = true;

        Ok(())
    }

    /// Sanitize column name for SQL (static version for basic sanitization)
    fn sanitize_column_name_basic(name: &str) -> String {
        // Replace non-alphanumeric with underscore, prefix with param_ to avoid reserved words
        let sanitized: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        format!("param_{}", sanitized)
    }

    /// Get or create a unique sanitized column name for an original name
    /// Handles collisions by appending a numeric suffix
    fn get_sanitized_column_name(&mut self, original_name: &str) -> String {
        // Return existing mapping if we have one
        if let Some(sanitized) = self.column_name_map.get(original_name) {
            return sanitized.clone();
        }

        // Generate base sanitized name
        let base_sanitized = Self::sanitize_column_name_basic(original_name);

        // Check for collision and find unique name
        let unique_name = if self.sanitized_names.contains(&base_sanitized) {
            // Collision detected - find unique suffix
            let mut suffix = 1u32;
            loop {
                let candidate = format!("{}_{}", base_sanitized, suffix);
                if !self.sanitized_names.contains(&candidate) {
                    tracing::warn!(
                        "Column name collision detected: '{}' and another column both sanitize to '{}'. \
                         Using '{}' for '{}'.",
                        original_name,
                        base_sanitized,
                        candidate,
                        original_name
                    );
                    break candidate;
                }
                suffix += 1;
            }
        } else {
            base_sanitized
        };

        // Register the mapping
        self.sanitized_names.insert(unique_name.clone());
        self.column_name_map
            .insert(original_name.to_string(), unique_name.clone());

        unique_name
    }

    /// Add a column if it doesn't exist
    fn ensure_column(&mut self, name: &str) -> Result<()> {
        if self.columns.contains(&name.to_string()) {
            return Ok(());
        }

        let safe_col = self.get_sanitized_column_name(name);

        if self.table_created {
            // ALTER TABLE to add column
            self.conn
                .execute(
                    &format!("ALTER TABLE events ADD COLUMN {} TEXT", safe_col),
                    [],
                )
                .map_err(OutputError::Sqlite)?;
        }

        self.columns.push(name.to_string());
        Ok(())
    }

    /// Collect columns from logs
    fn collect_columns(&mut self, logs: &[DecodedLog]) {
        for log in logs {
            for key in log.params.keys() {
                if !self.columns.contains(key) {
                    self.columns.push(key.clone());
                }
            }
        }
    }

    /// Insert a batch of logs
    fn insert_batch(&mut self, logs: Vec<DecodedLog>) -> Result<()> {
        if logs.is_empty() {
            return Ok(());
        }

        // Ensure all columns exist
        for log in &logs {
            for key in log.params.keys() {
                self.ensure_column(key)?;
            }
        }

        // Build INSERT statement
        let mut cols = vec![
            "block_number",
            "transaction_hash",
            "log_index",
            "address",
            "event_name",
            "event_signature",
            "topics",
            "data",
        ];

        // Use the collision-safe column name mapping
        let param_cols: Vec<String> = self
            .columns
            .iter()
            .map(|c| {
                self.column_name_map
                    .get(c)
                    .cloned()
                    .unwrap_or_else(|| Self::sanitize_column_name_basic(c))
            })
            .collect();

        for col in &param_cols {
            cols.push(col);
        }

        let placeholders: Vec<&str> = (0..cols.len()).map(|_| "?").collect();
        let sql = format!(
            "INSERT INTO events ({}) VALUES ({})",
            cols.join(", "),
            placeholders.join(", ")
        );

        let tx = self.conn.transaction().map_err(OutputError::Sqlite)?;

        {
            let mut stmt = tx.prepare(&sql).map_err(OutputError::Sqlite)?;

            for log in &logs {
                let topics_json = match serde_json::to_string(&log.topics) {
                    Ok(json) => json,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to serialize topics for log in tx {:#x}: {}",
                            log.transaction_hash,
                            e
                        );
                        "[]".to_string()
                    }
                };

                let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![
                    Box::new(log.block_number as i64),
                    Box::new(format!("{:#x}", log.transaction_hash)),
                    Box::new(log.log_index as i64),
                    Box::new(format!("{:#x}", log.address)),
                    Box::new(log.event_name.clone()),
                    Box::new(log.event_signature.clone()),
                    Box::new(topics_json),
                    Box::new(log.data.clone()),
                ];

                // Add parameter values
                for col_name in &self.columns {
                    let value = log
                        .params
                        .get(col_name)
                        .map(Self::value_to_string)
                        .unwrap_or_default();
                    values.push(Box::new(value));
                }

                let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|v| v.as_ref()).collect();
                stmt.execute(params.as_slice())
                    .map_err(OutputError::Sqlite)?;
            }
        }

        tx.commit().map_err(OutputError::Sqlite)?;
        Ok(())
    }

    /// Convert value to string for storage
    fn value_to_string(value: &DecodedValue) -> String {
        match value {
            DecodedValue::Address(s) => s.clone(),
            DecodedValue::Uint(s) => s.clone(),
            DecodedValue::Int(s) => s.clone(),
            DecodedValue::Bool(b) => b.to_string(),
            DecodedValue::Bytes(s) => s.clone(),
            DecodedValue::String(s) => s.clone(),
            DecodedValue::Array(arr) => serde_json::to_string(arr).unwrap_or_else(|e| {
                tracing::warn!("Failed to serialize array value: {}", e);
                "[serialization error]".to_string()
            }),
            DecodedValue::Tuple(arr) => serde_json::to_string(arr).unwrap_or_else(|e| {
                tracing::warn!("Failed to serialize tuple value: {}", e);
                "[serialization error]".to_string()
            }),
        }
    }

    /// Write raw logs to a simpler table
    fn write_raw_logs(&mut self, logs: &[Log]) -> Result<()> {
        // Create raw logs table
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS raw_logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                block_number INTEGER,
                transaction_hash TEXT,
                log_index INTEGER,
                address TEXT NOT NULL,
                topic0 TEXT,
                topic1 TEXT,
                topic2 TEXT,
                topic3 TEXT,
                data BLOB
            );
            CREATE INDEX IF NOT EXISTS idx_raw_block ON raw_logs(block_number);
            CREATE INDEX IF NOT EXISTS idx_raw_address ON raw_logs(address);",
            )
            .map_err(OutputError::Sqlite)?;

        let tx = self.conn.transaction().map_err(OutputError::Sqlite)?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO raw_logs (block_number, transaction_hash, log_index, address, topic0, topic1, topic2, topic3, data)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            ).map_err(OutputError::Sqlite)?;

            for log in logs {
                let topics = log.topics();
                stmt.execute(params![
                    log.block_number.map(|n| n as i64),
                    log.transaction_hash.map(|h| format!("{:#x}", h)),
                    log.log_index.map(|i| i as i64),
                    format!("{:#x}", log.address()),
                    topics.first().map(|t| format!("{:#x}", t)),
                    topics.get(1).map(|t| format!("{:#x}", t)),
                    topics.get(2).map(|t| format!("{:#x}", t)),
                    topics.get(3).map(|t| format!("{:#x}", t)),
                    log.data().data.to_vec(),
                ])
                .map_err(OutputError::Sqlite)?;
            }
        }

        tx.commit().map_err(OutputError::Sqlite)?;
        Ok(())
    }
}

impl OutputWriter for SqliteWriter {
    fn write_logs(&mut self, result: &FetchResult) -> Result<()> {
        match &result.logs {
            FetchLogs::Decoded(logs) => {
                // Collect columns from first batch
                if !self.table_created {
                    self.collect_columns(logs);
                    self.create_table()?;
                }

                // Buffer logs
                self.buffer.extend(logs.iter().cloned());

                // Flush if batch size reached
                if self.buffer.len() >= self.batch_size {
                    let batch = std::mem::take(&mut self.buffer);
                    self.insert_batch(batch)?;
                }
            }
            FetchLogs::Raw(logs) => {
                self.write_raw_logs(logs)?;
            }
        }
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        // Flush remaining buffer
        if !self.buffer.is_empty() {
            // Take buffer first to avoid borrow conflict
            let batch = std::mem::take(&mut self.buffer);
            if !self.table_created {
                self.collect_columns(&batch);
                self.create_table()?;
            }
            self.insert_batch(batch)?;
        }

        // Optimize database
        self.conn
            .execute_batch("PRAGMA optimize;")
            .map_err(OutputError::Sqlite)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_column_name_basic() {
        assert_eq!(
            SqliteWriter::sanitize_column_name_basic("from"),
            "param_from"
        );
        assert_eq!(
            SqliteWriter::sanitize_column_name_basic("token-id"),
            "param_token_id"
        );
    }

    #[test]
    fn test_column_name_collision_handling() {
        // Note: We can't easily test the collision handling without a full writer,
        // but we can verify that the basic sanitization produces identical results
        // for names that should collide
        let name1 = "a-b";
        let name2 = "a_b";
        let sanitized1 = SqliteWriter::sanitize_column_name_basic(name1);
        let sanitized2 = SqliteWriter::sanitize_column_name_basic(name2);
        assert_eq!(sanitized1, sanitized2); // These collide
    }

    #[test]
    fn test_value_to_string_address() {
        let value = DecodedValue::Address("0x1234567890abcdef".to_string());
        assert_eq!(SqliteWriter::value_to_string(&value), "0x1234567890abcdef");
    }

    #[test]
    fn test_value_to_string_uint() {
        let value = DecodedValue::Uint("12345678901234567890".to_string());
        assert_eq!(
            SqliteWriter::value_to_string(&value),
            "12345678901234567890"
        );
    }

    #[test]
    fn test_value_to_string_bool() {
        let value_true = DecodedValue::Bool(true);
        let value_false = DecodedValue::Bool(false);
        assert_eq!(SqliteWriter::value_to_string(&value_true), "true");
        assert_eq!(SqliteWriter::value_to_string(&value_false), "false");
    }

    #[test]
    fn test_value_to_string_array() {
        let value = DecodedValue::Array(vec![
            DecodedValue::Uint("1".to_string()),
            DecodedValue::Uint("2".to_string()),
        ]);
        let result = SqliteWriter::value_to_string(&value);
        // Should be valid JSON
        assert!(result.starts_with('['));
        assert!(result.contains("\"1\""));
        assert!(result.contains("\"2\""));
    }

    #[test]
    fn test_value_to_string_tuple() {
        let value = DecodedValue::Tuple(vec![
            DecodedValue::Address("0xabc".to_string()),
            DecodedValue::Uint("123".to_string()),
        ]);
        let result = SqliteWriter::value_to_string(&value);
        // Should be valid JSON
        assert!(result.starts_with('['));
        assert!(result.contains("0xabc"));
    }

    #[test]
    fn test_value_to_string_nested() {
        let value = DecodedValue::Array(vec![DecodedValue::Tuple(vec![
            DecodedValue::Address("0x123".to_string()),
            DecodedValue::Bool(true),
        ])]);
        let result = SqliteWriter::value_to_string(&value);
        // Should be valid JSON with nested structure
        assert!(result.contains("0x123"));
        assert!(result.contains("true"));
    }
}
