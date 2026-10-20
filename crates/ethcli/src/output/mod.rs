//! Output writers for different formats

mod csv;
mod json;
mod sqlite;

pub use self::csv::CsvWriter;
pub use self::json::JsonWriter;
pub use self::sqlite::SqliteWriter;

use crate::config::OutputFormat;
use crate::error::{OutputError, Result};
use crate::fetcher::FetchResult;
use std::path::Path;

/// Trait for output writers
pub trait OutputWriter {
    /// Write a batch of logs
    fn write_logs(&mut self, logs: &FetchResult) -> Result<()>;

    /// Finalize output (flush, close, etc.)
    fn finalize(&mut self) -> Result<()>;
}

/// Create an output writer for `logs --resume`.
///
/// A resumed run skips the checkpointed ranges, so it must add to the existing
/// output instead of truncating it. NDJSON is opened in append mode and SQLite
/// already inserts into the existing tables. A JSON array or CSV file can't be
/// extended safely (closing `]`, header/column set), so resuming into one is
/// refused. Writing to stdout is unaffected.
pub fn create_resume_writer(
    format: OutputFormat,
    path: Option<&Path>,
) -> Result<Box<dyn OutputWriter>> {
    match (format, path) {
        (OutputFormat::NdJson, Some(p)) => Ok(Box::new(JsonWriter::append_ndjson(p)?)),
        (OutputFormat::Json | OutputFormat::Csv, Some(p)) => Err(OutputError::FileCreate(format!(
            "{}: --resume can't append to a {} file without losing the earlier results; \
             use --format ndjson or sqlite",
            p.display(),
            if format == OutputFormat::Csv {
                "CSV"
            } else {
                "JSON"
            }
        ))
        .into()),
        _ => create_writer(format, path),
    }
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
                OutputError::FileCreate("SQLite requires output path".to_string())
            })?;
            let writer = SqliteWriter::new(path)?;
            Ok(Box::new(writer))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_refuses_json_and_csv_files_without_truncating() {
        let dir = tempfile::tempdir().unwrap();
        for (format, name) in [
            (OutputFormat::Json, "out.json"),
            (OutputFormat::Csv, "out.csv"),
        ] {
            let path = dir.path().join(name);
            std::fs::write(&path, "earlier results").unwrap();
            assert!(create_resume_writer(format, Some(&path)).is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "earlier results");
        }
    }

    #[test]
    fn resume_allows_ndjson_and_stdout() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.ndjson");
        std::fs::write(&path, "earlier\n").unwrap();
        assert!(create_resume_writer(OutputFormat::NdJson, Some(&path)).is_ok());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "earlier\n");
        assert!(create_resume_writer(OutputFormat::Json, None).is_ok());
    }
}
