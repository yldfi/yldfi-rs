//! Checkpoint system for resumable fetching

use crate::error::{CheckpointError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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
    /// Size of the NDJSON output, in bytes, when the completed ranges were
    /// last saved. On resume, anything past it was written after the last
    /// checkpoint (or is a torn line) and is cut off, so it isn't duplicated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_len: Option<u64>,
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
            output_len: None,
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
        start_block: u64,
        end_block: Option<u64>,
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

        // The completed ranges only make sense for the same block range
        if self.start_block != start_block || self.end_block != end_block {
            let fmt_end = |e: Option<u64>| e.map_or("latest".to_string(), |n| n.to_string());
            return Err(CheckpointError::Corrupted(format!(
                "Block range mismatch: checkpoint={}-{}, current={}-{}. \
                 Use the original range or delete the checkpoint file",
                self.start_block,
                fmt_end(self.end_block),
                start_block,
                fmt_end(end_block)
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
    /// Save interval (number of ranges). 1: the caller writes and flushes each
    /// range before marking it completed, so saving every time keeps the
    /// checkpoint in step with the output. A crash between the two leaves
    /// rows the checkpoint doesn't record; for NDJSON the saved `output_len`
    /// lets the next resume cut them off (SQLite output dedups instead).
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
            save_interval: 1,
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
            loaded.validate(contract, chain_id, event_signature, start_block, end_block)?;
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

    /// Record the output's size to save with the next completed range
    pub fn set_output_len(&mut self, len: u64) {
        self.checkpoint.output_len = Some(len);
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

/// Get current timestamp (delegated to utils for consistency)
fn current_timestamp() -> u64 {
    crate::utils::unix_timestamp_secs()
}

/// Why a `--resume` run can't safely use its output file, if it can't.
///
/// - Data but no checkpoint: the whole range would be fetched and appended,
///   duplicating every row the file holds.
/// - A checkpoint with logs but no data: the file isn't the one the checkpoint
///   was written with, and ranges marked complete wouldn't be fetched again, so
///   it would end up missing those rows.
#[must_use]
pub fn resume_conflict(output: Option<&Path>, checkpoint: &Path) -> Option<String> {
    let output = output?;
    let has_data = fs::metadata(output).is_ok_and(|m| m.len() > 0);
    if !checkpoint.exists() {
        return has_data.then(|| {
            format!(
                "{} already has data but there is no checkpoint at {}, so --resume would add \
                 every row again. Use a new output file, delete this one, or pass --checkpoint \
                 if the checkpoint is elsewhere",
                output.display(),
                checkpoint.display()
            )
        });
    }
    // An unreadable checkpoint is reported when the fetch loads it
    let logs = Checkpoint::load(checkpoint).map_or(0, |c| c.total_logs);
    (!has_data && logs > 0).then(|| {
        format!(
            "{} is empty or missing but the checkpoint at {} already covers {logs} logs, so \
             --resume would skip them. Use the original output file, or delete the checkpoint \
             to fetch everything again",
            output.display(),
            checkpoint.display()
        )
    })
}

/// Take the lock for a checkpoint for the rest of the run (held while the
/// returned file is open). A second `--resume` of the same run would write the
/// same output and checkpoint, so it is refused instead.
pub fn lock_checkpoint(checkpoint: &Path) -> Result<fs::File> {
    let mut name = checkpoint.as_os_str().to_owned();
    name.push(".lock");
    let lock_path = PathBuf::from(name);
    if let Some(parent) = lock_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        let _ = fs::create_dir_all(parent);
    }
    let file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|e| CheckpointError::WriteError(format!("{}: {}", lock_path.display(), e)))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(fs::TryLockError::WouldBlock) => Err(CheckpointError::WriteError(format!(
            "another `ethcli logs --resume` is using {}; wait for it to finish",
            checkpoint.display()
        ))
        .into()),
        Err(fs::TryLockError::Error(e)) => {
            Err(CheckpointError::WriteError(format!("{}: {}", lock_path.display(), e)).into())
        }
    }
}

/// Before a resumed NDJSON run appends to `output`, cut off what was written
/// after the last checkpoint: rows from ranges the checkpoint doesn't record
/// (they will be fetched again) and any torn last line. Returns a message
/// when something was cut.
///
/// With a recorded size, the file is cut back to it. A checkpoint from an
/// older release has none, so only a torn last line is removed.
pub fn prepare_resume_output(output: &Path, checkpoint: &Path) -> Result<Option<String>> {
    let Ok(meta) = fs::metadata(output) else {
        return Ok(None);
    };
    let len = meta.len();
    let recorded = Checkpoint::load(checkpoint).ok().and_then(|c| c.output_len);
    let target = match recorded {
        Some(n) if n > len => {
            return Err(CheckpointError::Corrupted(format!(
                "{} is {} bytes but the checkpoint at {} recorded {} bytes, so rows from ranges \
                 marked complete are missing. Use the original output file, or delete the \
                 checkpoint to fetch everything again",
                output.display(),
                len,
                checkpoint.display(),
                n
            ))
            .into())
        }
        Some(n) => n,
        None => after_last_newline(output, len)?,
    };
    if target >= len {
        return Ok(None);
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .open(output)
        .map_err(|e| CheckpointError::WriteError(format!("{}: {}", output.display(), e)))?;
    file.set_len(target)
        .map_err(|e| CheckpointError::WriteError(format!("{}: {}", output.display(), e)))?;
    Ok(Some(format!(
        "Removed {} bytes from {} written after the last checkpoint; those blocks will be \
         fetched again",
        len - target,
        output.display()
    )))
}

/// The offset just after the file's last newline (0 if there is none)
fn after_last_newline(path: &Path, len: u64) -> Result<u64> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = fs::File::open(path)
        .map_err(|e| CheckpointError::ReadError(format!("{}: {}", path.display(), e)))?;
    let mut end = len;
    let mut buf = vec![0u8; 64 * 1024];
    while end > 0 {
        let start = end.saturating_sub(buf.len() as u64);
        let n = (end - start) as usize;
        file.seek(SeekFrom::Start(start))
            .and_then(|_| file.read_exact(&mut buf[..n]))
            .map_err(|e| CheckpointError::ReadError(format!("{}: {}", path.display(), e)))?;
        if let Some(i) = buf[..n].iter().rposition(|&b| b == b'\n') {
            return Ok(start + i as u64 + 1);
        }
        end = start;
    }
    Ok(0)
}

#[cfg(test)]
mod resume_output_tests {
    use super::*;

    fn checkpoint_with(dir: &Path, output_len: Option<u64>) -> PathBuf {
        let path = dir.join("run.checkpoint");
        let mut c = Checkpoint::new("0xabc".into(), 1, None, 0, Some(100));
        c.output_len = output_len;
        c.save(&path).unwrap();
        path
    }

    #[test]
    fn cuts_rows_written_after_the_last_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.ndjson");
        fs::write(&out, "{\"a\":1}\n{\"a\":2}\n{\"a\":3}\n").unwrap();
        let cp = checkpoint_with(dir.path(), Some(16));
        let msg = prepare_resume_output(&out, &cp).unwrap();
        assert!(msg.is_some());
        assert_eq!(fs::read_to_string(&out).unwrap(), "{\"a\":1}\n{\"a\":2}\n");
    }

    #[test]
    fn an_old_checkpoint_only_loses_a_torn_line() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.ndjson");
        fs::write(&out, "{\"a\":1}\n{\"a\":2}\n{\"a\":").unwrap();
        let cp = checkpoint_with(dir.path(), None);
        prepare_resume_output(&out, &cp).unwrap();
        assert_eq!(fs::read_to_string(&out).unwrap(), "{\"a\":1}\n{\"a\":2}\n");
    }

    #[test]
    fn a_clean_file_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.ndjson");
        fs::write(&out, "{\"a\":1}\n").unwrap();
        let cp = checkpoint_with(dir.path(), Some(8));
        assert!(prepare_resume_output(&out, &cp).unwrap().is_none());
        assert_eq!(fs::read_to_string(&out).unwrap(), "{\"a\":1}\n");
    }

    #[test]
    fn a_shorter_file_than_recorded_is_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out.ndjson");
        fs::write(&out, "{\"a\":1}\n").unwrap();
        let cp = checkpoint_with(dir.path(), Some(100));
        assert!(prepare_resume_output(&out, &cp).is_err());
    }

    #[test]
    fn a_second_lock_on_the_same_checkpoint_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let cp = dir.path().join("run.checkpoint");
        let first = lock_checkpoint(&cp).unwrap();
        assert!(lock_checkpoint(&cp).is_err());
        drop(first);
        assert!(lock_checkpoint(&cp).is_ok());
    }

    #[test]
    fn checkpoints_without_output_len_still_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("old.checkpoint");
        let c = Checkpoint::new("0xabc".into(), 1, None, 0, Some(100));
        let mut v = serde_json::to_value(&c).unwrap();
        v.as_object_mut().unwrap().remove("output_len");
        fs::write(&path, v.to_string()).unwrap();
        assert_eq!(Checkpoint::load(&path).unwrap().output_len, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn resume_conflict_only_when_data_has_no_checkpoint() {
        let dir = tempfile::tempdir().unwrap();
        let output = dir.path().join("out.ndjson");
        let checkpoint = dir.path().join("fetch.checkpoint");

        // stdout, or no output file yet
        assert_eq!(resume_conflict(None, &checkpoint), None);
        assert_eq!(resume_conflict(Some(&output), &checkpoint), None);

        // An empty file has nothing to duplicate
        fs::write(&output, "").unwrap();
        assert_eq!(resume_conflict(Some(&output), &checkpoint), None);

        // Data and no checkpoint
        fs::write(&output, "{}\n").unwrap();
        let message = resume_conflict(Some(&output), &checkpoint).unwrap();
        assert!(
            message.contains("out.ndjson") && message.contains("--resume"),
            "{message}"
        );

        // Data and a checkpoint: a normal resume
        let mut saved = Checkpoint::new("0xabc".to_string(), 1, None, 1, Some(2));
        saved.save(&checkpoint).unwrap();
        assert_eq!(resume_conflict(Some(&output), &checkpoint), None);

        // A checkpoint that found nothing has nothing to miss
        let other = dir.path().join("other.ndjson");
        assert_eq!(resume_conflict(Some(&other), &checkpoint), None);

        // A checkpoint with logs, and a new or empty output file
        saved.total_logs = 5;
        saved.save(&checkpoint).unwrap();
        let message = resume_conflict(Some(&other), &checkpoint).unwrap();
        assert!(
            message.contains("other.ndjson") && message.contains("5 logs"),
            "{message}"
        );
        fs::write(&other, "").unwrap();
        assert!(resume_conflict(Some(&other), &checkpoint).is_some());
        assert_eq!(resume_conflict(Some(&output), &checkpoint), None);
        assert_eq!(resume_conflict(None, &checkpoint), None);
    }

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
    fn validate_rejects_a_different_block_range() {
        let cp = Checkpoint::new("0xAbC".to_string(), 1, None, 1000, Some(2000));
        assert!(cp.validate("0xabc", 1, None, 1000, Some(2000)).is_ok());
        assert!(cp.validate("0xabc", 1, None, 500, Some(2000)).is_err());
        assert!(cp.validate("0xabc", 1, None, 1000, Some(3000)).is_err());
        assert!(cp.validate("0xabc", 1, None, 1000, None).is_err());
    }

    #[test]
    fn manager_saves_every_completed_range() {
        let file = NamedTempFile::new().unwrap();
        let cp = Checkpoint::new("0x1234".to_string(), 1, None, 0, Some(1000));
        let mut manager = CheckpointManager::new(cp, file.path().to_path_buf());
        manager.mark_completed(0, 99, 5).unwrap();
        // No explicit save: the range must already be on disk
        let loaded = Checkpoint::load(file.path()).unwrap();
        assert_eq!(loaded.completed_ranges, vec![(0, 99)]);
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
