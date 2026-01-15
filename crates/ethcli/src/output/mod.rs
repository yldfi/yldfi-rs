//! Output writers for different formats

mod csv;
mod json;
mod sqlite;

pub use self::csv::CsvWriter;
pub use self::json::JsonWriter;
pub use self::sqlite::SqliteWriter;

use crate::config::OutputFormat;
use crate::error::Result;
use crate::fetcher::FetchResult;
use std::path::Path;

/// Trait for output writers
pub trait OutputWriter {
    /// Write a batch of logs
    fn write_logs(&mut self, logs: &FetchResult) -> Result<()>;

    /// Finalize output (flush, close, etc.)
    fn finalize(&mut self) -> Result<()>;
}

/// Create an output writer based on format and path
pub fn create_writer(format: OutputFormat, path: Option<&Path>) -> Result<Box<dyn OutputWriter>> {
    match format {
        OutputFormat::Json => {
            let writer = JsonWriter::new(path, false)?;
            Ok(Box::new(writer))
        }
        OutputFormat::NdJson => {
            let writer = JsonWriter::new(path, true)?;
            Ok(Box::new(writer))
        }
        OutputFormat::Csv => {
            let writer = CsvWriter::new(path)?;
            Ok(Box::new(writer))
        }
        OutputFormat::Sqlite => {
            let path = path.ok_or_else(|| {
                crate::error::OutputError::FileCreate("SQLite requires output path".to_string())
            })?;
            let writer = SqliteWriter::new(path)?;
            Ok(Box::new(writer))
        }
    }
}
