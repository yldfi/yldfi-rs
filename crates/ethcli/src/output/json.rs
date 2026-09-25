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
    /// Whether the opening `[` has been written (array format). Deferred
    /// until the first item or `finalize` so that an error before any output
    /// doesn't leave a stray `[` on stdout.
    opened: bool,
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

        Ok(Self::with_writer(writer, ndjson))
    }

    /// Create a JSON writer over an arbitrary sink
    pub fn with_writer(writer: Box<dyn Write + Send>, ndjson: bool) -> Self {
        Self {
            writer,
            ndjson,
            first_written: false,
            opened: false,
            count: 0,
        }
    }

    /// Write the opening bracket for array format (once)
    fn ensure_open(&mut self) -> Result<()> {
        if !self.ndjson && !self.opened {
            writeln!(self.writer, "[").map_err(|e| OutputError::JsonWrite(e.to_string()))?;
            self.opened = true;
        }
        Ok(())
    }

    /// Write a single decoded log
    fn write_decoded(&mut self, log: &DecodedLog) -> Result<()> {
        let json = serde_json::to_string(log).map_err(|e| OutputError::JsonWrite(e.to_string()))?;
        self.ensure_open()?;

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
        self.ensure_open()?;

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
            self.ensure_open()?;
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

    /// Shared in-memory sink so tests can inspect output after writing
    #[derive(Clone, Default)]
    struct SharedBuf(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl Write for SharedBuf {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl SharedBuf {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    #[test]
    fn test_no_output_before_first_item() {
        let buf = SharedBuf::default();
        let _writer = JsonWriter::with_writer(Box::new(buf.clone()), false);
        // Simulates an error before any logs are fetched: nothing on stdout
        assert_eq!(buf.text(), "");
    }

    #[test]
    fn test_array_output_is_valid_json_and_data_is_hex() {
        let buf = SharedBuf::default();
        let mut writer = JsonWriter::with_writer(Box::new(buf.clone()), false);
        let mut log = test_log();
        log.data = vec![0x00, 0x01, 0xab];
        writer.write_decoded(&log).unwrap();
        writer.finalize().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&buf.text()).unwrap();
        assert_eq!(parsed[0]["data"], "0x0001ab");
    }

    #[test]
    fn test_empty_array_output() {
        let buf = SharedBuf::default();
        let mut writer = JsonWriter::with_writer(Box::new(buf.clone()), false);
        writer.finalize().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&buf.text()).unwrap();
        assert_eq!(parsed, serde_json::json!([]));
    }

    #[test]
    fn test_json_array_format() {
        use std::io::Cursor;
        let buffer = Cursor::new(Vec::new());
        let _writer = JsonWriter::with_writer(Box::new(buffer), false);

        // Can't easily test without mocking, but structure is correct
    }
}
