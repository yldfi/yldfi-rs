//! Checkpoint system for resumable fetching

use crate::error::{CheckpointError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current checkpoint format version
const CHECKPOINT_VERSION: u32 = 1;

/// Checkpoint data for resuming interrupted fetches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Format version
    pub version: u32,
    /// Contract address being fetched
    pub contract: String,
    /// Chain ID
    pub chain_id: u64,
    /// Event signature (if filtering by specific event)
    pub event_signature: Option<String>,
    /// Original start block
    pub start_block: u64,
    /// Original end block (None = latest at start time)
    pub end_block: Option<u64>,
    /// Completed block ranges (start, end) inclusive
    pub completed_ranges: Vec<(u64, u64)>,
    /// Last processed block
    pub last_processed_block: u64,
    /// Total logs fetched so far
    pub total_logs: u64,
    /// Timestamp of last update
    pub last_updated: u64,
    /// Output file path (if any)
    pub output_path: Option<PathBuf>,
}

impl Checkpoint {
    /// Create a new checkpoint
    pub fn new(
        contract: String,
        chain_id: u64,
        event_signature: Option<String>,
        start_block: u64,
        end_block: Option<u64>,
    ) -> Self {
        Self {
            version: CHECKPOINT_VERSION,
            contract,
            chain_id,
            event_signature,
            start_block,
            end_block,
            completed_ranges: Vec::new(),
            last_processed_block: start_block,
            total_logs: 0,
            last_updated: current_timestamp(),
            output_path: None,
        }
    }

    /// Load from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .map_err(|e| CheckpointError::ReadError(format!("{}: {}", path.display(), e)))?;

        let checkpoint: Self = serde_json::from_str(&content)
            .map_err(|e| CheckpointError::Corrupted(format!("Invalid JSON: {}", e)))?;

        // Check version
        if checkpoint.version != CHECKPOINT_VERSION {
            return Err(CheckpointError::VersionMismatch {
                expected: CHECKPOINT_VERSION,
                found: checkpoint.version,
            }
            .into());
        }

        Ok(checkpoint)
    }

    /// Save to file (atomic write)
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| CheckpointError::WriteError(format!("Serialization failed: {}", e)))?;

        // Write to temp file first with unique name to avoid TOCTOU race
        let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));

        fs::write(&temp_path, &content)
            .map_err(|e| CheckpointError::WriteError(format!("Write failed: {}", e)))?;

        // Atomic rename
        if let Err(e) = fs::rename(&temp_path, path) {
            // Clean up temp file on failure
            let _ = fs::remove_file(&temp_path);
            return Err(CheckpointError::WriteError(format!("Rename failed: {}", e)).into());
        }

        Ok(())
    }

    /// Mark a block range as completed
    pub fn mark_completed(&mut self, from: u64, to: u64, logs_count: u64) {
        self.completed_ranges.push((from, to));
        self.total_logs += logs_count;

        if to > self.last_processed_block {
            self.last_processed_block = to;
        }

        self.last_updated = current_timestamp();

        // Merge adjacent ranges to keep the list compact
        self.merge_ranges();
    }

    /// Merge adjacent/overlapping completed ranges
    fn merge_ranges(&mut self) {
        if self.completed_ranges.len() < 2 {
            return;
        }

        // Sort by start block
        self.completed_ranges.sort_by_key(|r| r.0);

        let mut merged = Vec::new();
        let mut current = self.completed_ranges[0];

        for &(start, end) in self.completed_ranges.iter().skip(1) {
            if start <= current.1 + 1 {
                // Overlapping or adjacent - extend current
                current.1 = current.1.max(end);
            } else {
                // Gap - save current and start new
                merged.push(current);
                current = (start, end);
            }
        }
        merged.push(current);

        self.completed_ranges = merged;
    }

    /// Get remaining block ranges to fetch
    pub fn remaining_ranges(&self, end_block: u64) -> Vec<(u64, u64)> {
        if self.completed_ranges.is_empty() {
            return vec![(self.start_block, end_block)];
        }

        let mut remaining = Vec::new();
        let mut current_start = self.start_block;

        for &(completed_start, completed_end) in &self.completed_ranges {
            if current_start < completed_start {
                // Use saturating_sub to prevent underflow when completed_start == 0
                remaining.push((current_start, completed_start.saturating_sub(1)));
            }
            // Use saturating_add to prevent overflow at u64::MAX
            current_start = completed_end.saturating_add(1);

            // If we overflowed to 0, we've covered all possible blocks
            if current_start == 0 && completed_end != 0 {
                return remaining;
            }
        }

        if current_start <= end_block {
            remaining.push((current_start, end_block));
        }

        remaining
    }

    /// Calculate progress as percentage
    pub fn progress_percent(&self, end_block: u64) -> f64 {
        // Use saturating arithmetic throughout to prevent overflow
        let total_blocks = end_block.saturating_sub(self.start_block).saturating_add(1);
        if total_blocks == 0 {
            return 100.0;
        }

        let completed_blocks: u64 = self
            .completed_ranges
            .iter()
            .map(|(s, e)| e.saturating_sub(*s).saturating_add(1))
            .fold(0u64, |acc, x| acc.saturating_add(x));

        (completed_blocks as f64 / total_blocks as f64) * 100.0
    }

    /// Check if fetch is complete
    pub fn is_complete(&self, end_block: u64) -> bool {
        self.remaining_ranges(end_block).is_empty()
    }

    /// Validate checkpoint matches current fetch parameters
    pub fn validate(
        &self,
        contract: &str,
        chain_id: u64,
        event_signature: Option<&str>,
    ) -> Result<()> {
        if self.contract.to_lowercase() != contract.to_lowercase() {
            return Err(CheckpointError::Corrupted(format!(
                "Contract mismatch: checkpoint={}, current={}",
                self.contract, contract
            ))
            .into());
        }

        if self.chain_id != chain_id {
            return Err(CheckpointError::Corrupted(format!(
                "Chain ID mismatch: checkpoint={}, current={}",
                self.chain_id, chain_id
            ))
            .into());
        }

        let checkpoint_sig = self.event_signature.as_deref();
        if checkpoint_sig != event_signature {
            return Err(CheckpointError::Corrupted(format!(
                "Event signature mismatch: checkpoint={:?}, current={:?}",
                checkpoint_sig, event_signature
            ))
            .into());
        }

        Ok(())
    }

    /// Set output path
    pub fn set_output_path(&mut self, path: PathBuf) {
        self.output_path = Some(path);
    }
}

/// Checkpoint manager for automatic saving
pub struct CheckpointManager {
    /// Current checkpoint
    checkpoint: Checkpoint,
    /// File path
    path: PathBuf,
    /// Save interval (number of ranges)
    save_interval: usize,
    /// Ranges since last save
    ranges_since_save: usize,
}

impl CheckpointManager {
    /// Create a new manager
    pub fn new(checkpoint: Checkpoint, path: PathBuf) -> Self {
        Self {
            checkpoint,
            path,
            save_interval: 10,
            ranges_since_save: 0,
        }
    }

    /// Load or create checkpoint
    pub fn load_or_create(
        path: &Path,
        contract: &str,
        chain_id: u64,
        event_signature: Option<&str>,
        start_block: u64,
        end_block: Option<u64>,
    ) -> Result<Self> {
        let checkpoint = if path.exists() {
            let loaded = Checkpoint::load(path)?;
            loaded.validate(contract, chain_id, event_signature)?;
            loaded
        } else {
            Checkpoint::new(
                contract.to_string(),
                chain_id,
                event_signature.map(String::from),
                start_block,
                end_block,
            )
        };

        Ok(Self::new(checkpoint, path.to_path_buf()))
    }

    /// Mark range as completed
    pub fn mark_completed(&mut self, from: u64, to: u64, logs_count: u64) -> Result<()> {
        self.checkpoint.mark_completed(from, to, logs_count);
        self.ranges_since_save += 1;

        // Auto-save at interval
        if self.ranges_since_save >= self.save_interval {
            self.save()?;
        }

        Ok(())
    }

    /// Force save
    pub fn save(&mut self) -> Result<()> {
        self.checkpoint.save(&self.path)?;
        self.ranges_since_save = 0;
        Ok(())
    }

    /// Force save immediately (alias for save)
    pub fn save_now(&mut self) -> Result<()> {
        self.save()
    }

    /// Get remaining ranges
    pub fn remaining_ranges(&self, end_block: u64) -> Vec<(u64, u64)> {
        self.checkpoint.remaining_ranges(end_block)
    }

    /// Get progress
    pub fn progress_percent(&self, end_block: u64) -> f64 {
        self.checkpoint.progress_percent(end_block)
    }

    /// Get total logs fetched
    pub fn total_logs(&self) -> u64 {
        self.checkpoint.total_logs
    }

    /// Check if complete
    pub fn is_complete(&self, end_block: u64) -> bool {
        self.checkpoint.is_complete(end_block)
    }

    /// Get checkpoint reference
    pub fn checkpoint(&self) -> &Checkpoint {
        &self.checkpoint
    }

    /// Delete checkpoint file
    pub fn delete(&self) -> Result<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)
                .map_err(|e| CheckpointError::WriteError(format!("Delete failed: {}", e)))?;
        }
        Ok(())
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_checkpoint_creation() {
        let cp = Checkpoint::new(
            "0x1234".to_string(),
            1,
            Some("Transfer(address,address,uint256)".to_string()),
            1000,
            Some(2000),
        );

        assert_eq!(cp.contract, "0x1234");
        assert_eq!(cp.chain_id, 1);
        assert_eq!(cp.start_block, 1000);
        assert_eq!(cp.end_block, Some(2000));
    }

    #[test]
    fn test_mark_completed() {
        let mut cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(1000));

        cp.mark_completed(0, 100, 50);
        cp.mark_completed(101, 200, 30);

        assert_eq!(cp.total_logs, 80);
        assert_eq!(cp.last_processed_block, 200);
    }

    #[test]
    fn test_merge_ranges() {
        let mut cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(1000));

        cp.mark_completed(0, 100, 10);
        cp.mark_completed(101, 200, 10);
        cp.mark_completed(50, 150, 10); // Overlapping

        // Should merge into single range (0, 200)
        assert_eq!(cp.completed_ranges.len(), 1);
        assert_eq!(cp.completed_ranges[0], (0, 200));
    }

    #[test]
    fn test_remaining_ranges() {
        let mut cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(1000));

        cp.mark_completed(100, 200, 10);
        cp.mark_completed(500, 600, 10);

        let remaining = cp.remaining_ranges(1000);
        assert_eq!(remaining, vec![(0, 99), (201, 499), (601, 1000)]);
    }

    #[test]
    fn test_progress() {
        let mut cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(999));

        cp.mark_completed(0, 499, 10);
        assert!((cp.progress_percent(999) - 50.0).abs() < 0.1);

        cp.mark_completed(500, 999, 10);
        assert!((cp.progress_percent(999) - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_save_load() {
        let mut cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(1000));
        cp.mark_completed(0, 100, 50);

        let temp = NamedTempFile::new().unwrap();
        cp.save(temp.path()).unwrap();

        let loaded = Checkpoint::load(temp.path()).unwrap();
        assert_eq!(loaded.contract, "0x1234");
        assert_eq!(loaded.total_logs, 50);
        assert_eq!(loaded.completed_ranges, vec![(0, 100)]);
    }
}
