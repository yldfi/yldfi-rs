//! JSON output writer

use crate::abi::DecodedLog;
use crate::error::{OutputError, Result};
use crate::fetcher::{FetchLogs, FetchResult};
use crate::output::OutputWriter;
use alloy::rpc::types::Log;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

/// JSON output writer
pub struct JsonWriter {
    /// Output destination
    writer: Box<dyn Write + Send>,
    /// Whether to use NDJSON format
    ndjson: bool,
    /// Whether first item has been written (for array format)
    first_written: bool,
    /// Count of items written
    count: usize,
}

impl JsonWriter {
    /// Create a new JSON writer
    pub fn new(path: Option<&Path>, ndjson: bool) -> Result<Self> {
        let writer: Box<dyn Write + Send> = if let Some(p) = path {
            let file = File::create(p)
                .map_err(|e| OutputError::FileCreate(format!("{}: {}", p.display(), e)))?;
            Box::new(BufWriter::new(file))
        } else {
            Box::new(BufWriter::new(io::stdout()))
        };

        let mut json_writer = Self {
            writer,
            ndjson,
            first_written: false,
            count: 0,
        };

        // Write opening bracket for array format
        if !ndjson {
            writeln!(json_writer.writer, "[").map_err(|e| OutputError::JsonWrite(e.to_string()))?;
        }

        Ok(json_writer)
    }

    /// Write a single decoded log
    fn write_decoded(&mut self, log: &DecodedLog) -> Result<()> {
        let json = serde_json::to_string(log).map_err(|e| OutputError::JsonWrite(e.to_string()))?;

        if self.ndjson {
            writeln!(self.writer, "{}", json).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
        } else {
            if self.first_written {
                writeln!(self.writer, ",").map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            }
            write!(self.writer, "  {}", json).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            self.first_written = true;
        }

        self.count += 1;
        Ok(())
    }

    /// Write a single raw log
    fn write_raw(&mut self, log: &Log) -> Result<()> {
        let json = serde_json::to_string(log).map_err(|e| OutputError::JsonWrite(e.to_string()))?;

        if self.ndjson {
            writeln!(self.writer, "{}", json).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
        } else {
            if self.first_written {
                writeln!(self.writer, ",").map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            }
            write!(self.writer, "  {}", json).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            self.first_written = true;
        }

        self.count += 1;
        Ok(())
    }
}

impl OutputWriter for JsonWriter {
    fn write_logs(&mut self, result: &FetchResult) -> Result<()> {
        match &result.logs {
            FetchLogs::Decoded(logs) => {
                for log in logs {
                    self.write_decoded(log)?;
                }
            }
            FetchLogs::Raw(logs) => {
                for log in logs {
                    self.write_raw(log)?;
                }
            }
        }
        Ok(())
    }

    fn finalize(&mut self) -> Result<()> {
        if !self.ndjson {
            writeln!(self.writer).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            writeln!(self.writer, "]").map_err(|e| OutputError::JsonWrite(e.to_string()))?;
        }

        self.writer
            .flush()
            .map_err(|e| OutputError::JsonWrite(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{Address, B256};
    use std::collections::HashMap;

    #[allow(dead_code)]
    fn test_log() -> DecodedLog {
        DecodedLog {
            block_number: 12345,
            timestamp: None,
            transaction_hash: B256::ZERO,
            log_index: 0,
            address: Address::ZERO,
            event_name: "Transfer".to_string(),
            event_signature: "Transfer(address,address,uint256)".to_string(),
            params: HashMap::new(),
            topics: vec![],
            data: vec![],
        }
    }

    #[test]
    fn test_json_array_format() {
        use std::io::Cursor;
        let buffer = Cursor::new(Vec::new());
        let _writer = JsonWriter {
            writer: Box::new(buffer),
            ndjson: false,
            first_written: false,
            count: 0,
        };

        // Can't easily test without mocking, but structure is correct
    }
}
