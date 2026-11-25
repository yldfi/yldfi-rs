//! Pacing for explorer API requests, shared by every ethcli process on the
//! machine
//!
//! The explorer allows a few requests per second per key (or per address
//! without one). Several ethcli processes started at once (an agent running
//! `contract source` on ten addresses in parallel) would each stay under the
//! limit and together go over it, so the next free request slot is kept in a
//! small file in ethcli's state folder (per user, or per `ETHCLI_STATE_DIR`),
//! under an exclusive file lock. Processes using another state folder, or
//! other machines on the same key, are not paced together. Each request takes
//! the next slot and waits for it. If the file cannot be used (not a regular
//! file, a symlink, a lock held for more than 2 s) the request is paced
//! within this process only.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Requests per second when `ETHCLI_EXPLORER_RPS` is not set
const DEFAULT_RPS: f64 = 3.0;
/// Accepted range for `ETHCLI_EXPLORER_RPS`; anything else uses the default
const MIN_RPS: f64 = 0.1;
const MAX_RPS: f64 = 100.0;
/// Longest wait for the pace file's lock before pacing in-process only
const LOCK_WAIT: Duration = Duration::from_secs(2);

fn rps_from(v: Option<&str>) -> f64 {
    v.and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && (MIN_RPS..=MAX_RPS).contains(v))
        .unwrap_or(DEFAULT_RPS)
}

/// Gap between two requests, with a margin so that clock jitter on the
/// explorer's side does not count two of ours in the same second
fn interval() -> Duration {
    let rps = rps_from(std::env::var("ETHCLI_EXPLORER_RPS").ok().as_deref());
    Duration::from_secs_f64(1.0 / rps) + Duration::from_millis(30)
}

fn pace_file() -> Option<PathBuf> {
    let dir = crate::cli::skills::state_dir();
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir.join("explorer-pace"))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Take the next slot from `next_free` (ms since the epoch) and return how
/// long to wait for it and the new value to store
fn take_slot(now: u64, next_free: u64, gap_ms: u64) -> (u64, u64) {
    // A stored slot further ahead than 50 gaps (or a minute) is stale (a
    // clock change, a crashed writer): ignore it
    let horizon = gap_ms.saturating_mul(50).max(60_000);
    let next_free = if next_free > now.saturating_add(horizon) {
        now
    } else {
        next_free
    };
    let slot = next_free.max(now);
    (slot - now, slot + gap_ms)
}

/// Open the pace file: never through a symlink, only a regular file, created
/// readable by the owner only
fn open_pace_file(path: &std::path::Path) -> Option<std::fs::File> {
    if let Ok(meta) = std::fs::symlink_metadata(path) {
        if !meta.file_type().is_file() {
            return None;
        }
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let file = opts.open(path).ok()?;
    file.metadata().ok().filter(|m| m.is_file())?;
    Some(file)
}

/// Take the file's lock, waiting at most `LOCK_WAIT`
fn lock_with_deadline(file: &std::fs::File) -> bool {
    let deadline = std::time::Instant::now() + LOCK_WAIT;
    loop {
        match file.try_lock() {
            Ok(()) => return true,
            Err(std::fs::TryLockError::WouldBlock) if std::time::Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(_) => return false,
        }
    }
}

/// Slot bookkeeping in the shared file. `None` when the file cannot be used
fn shared_wait(gap_ms: u64) -> Option<u64> {
    // Unit tests pace within the process only: they must not touch the
    // machine-wide file other ethcli runs use
    if cfg!(test) {
        return None;
    }
    let mut file = open_pace_file(&pace_file()?)?;
    if !lock_with_deadline(&file) {
        return None;
    }
    let mut text = String::new();
    let _ = file.read_to_string(&mut text);
    let stored = text.trim().parse::<u64>().unwrap_or(0);
    let (wait, next) = take_slot(now_ms(), stored, gap_ms);
    let written = file
        .seek(SeekFrom::Start(0))
        .and_then(|_| file.set_len(0))
        .and_then(|_| file.write_all(next.to_string().as_bytes()));
    let _ = file.unlock();
    written.ok().map(|_| wait)
}

static LOCAL_NEXT: Mutex<u64> = Mutex::new(0);

fn local_wait(gap_ms: u64) -> u64 {
    let mut next = LOCAL_NEXT.lock().unwrap_or_else(|e| e.into_inner());
    let (wait, new_next) = take_slot(now_ms(), *next, gap_ms);
    *next = new_next;
    wait
}

/// Wait for this process's turn to send an explorer request
pub async fn acquire() {
    let gap_ms = interval().as_millis() as u64;
    let wait = tokio::task::spawn_blocking(move || shared_wait(gap_ms))
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| local_wait(gap_ms));
    if wait > 0 {
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }
}

/// Wait before retry `attempt` (0-based) after a rate-limit answer: 1, 2,
/// 4, 8 s, plus up to half a second of jitter so processes that were refused
/// together do not retry together
pub fn backoff(attempt: u32) -> Duration {
    let base = Duration::from_secs(1u64 << attempt.min(4));
    base + Duration::from_millis(rand::random_range(0..500))
}

/// Whether an explorer error text is a rate-limit answer worth waiting out.
/// A daily limit is not: retrying within seconds cannot help
pub fn is_rate_limit_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    (lower.contains("rate limit") || lower.contains("max calls per sec"))
        && !lower.contains("daily")
        && !lower.contains("per day")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rps_is_validated() {
        assert_eq!(rps_from(None), DEFAULT_RPS);
        assert_eq!(rps_from(Some("5")), 5.0);
        for bad in ["0", "-1", "NaN", "inf", "0.0001", "1000", "x"] {
            assert_eq!(rps_from(Some(bad)), DEFAULT_RPS, "{bad}");
        }
    }

    /// A symlink or a directory is never used as the pace file
    #[test]
    fn pace_file_refuses_links_and_directories() {
        let dir = std::env::temp_dir().join(format!("ethcli-pace-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("target");
        std::fs::write(&target, "keep").unwrap();
        #[cfg(unix)]
        {
            let link = dir.join("link");
            std::os::unix::fs::symlink(&target, &link).unwrap();
            assert!(open_pace_file(&link).is_none());
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
        }
        assert!(open_pace_file(&dir).is_none());
        let real = dir.join("pace");
        assert!(open_pace_file(&real).is_some());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&real).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn daily_limits_are_not_retried() {
        assert!(!is_rate_limit_text("Max daily rate limit reached"));
    }

    #[test]
    fn slots_are_spaced() {
        // Free now: no wait, next slot one gap later
        assert_eq!(take_slot(1_000, 0, 350), (0, 1_350));
        // Taken: wait for it
        assert_eq!(take_slot(1_000, 1_350, 350), (350, 1_700));
        // A stale far-future slot is ignored
        assert_eq!(take_slot(1_000, 1_000_000, 350), (0, 1_350));
        // At a low rate a slot 50 gaps ahead is still honoured
        assert_eq!(take_slot(0, 400_000, 10_000).0, 400_000);
    }

    #[test]
    fn backoff_grows() {
        assert!(backoff(0) >= Duration::from_secs(1) && backoff(0) < Duration::from_millis(1500));
        assert!(backoff(3) >= Duration::from_secs(8));
    }

    #[test]
    fn rate_limit_texts() {
        assert!(is_rate_limit_text(
            "NOTOK: Max calls per sec rate limit reached (3/sec)"
        ));
        assert!(is_rate_limit_text("Max rate limit reached"));
        assert!(!is_rate_limit_text("Contract source code not verified"));
    }
}
