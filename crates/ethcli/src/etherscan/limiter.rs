//! Pacing for explorer API requests, shared by every ethcli process on the
//! machine
//!
//! The explorer allows a few requests per second per key (or per address
//! without one). Several ethcli processes started at once (an agent running
//! `contract source` on ten addresses in parallel) would each stay under the
//! limit and together go over it, so the next free request slot is kept in a
//! small file in the per-user state folder, under an exclusive file lock. Each
//! request takes the next slot and waits for it. If the file cannot be used
//! the request is paced within this process only.

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Requests per second when `ETHCLI_EXPLORER_RPS` is not set
const DEFAULT_RPS: f64 = 3.0;

/// Gap between two requests, with a margin so that clock jitter on the
/// explorer's side does not count two of ours in the same second
fn interval() -> Duration {
    let rps = std::env::var("ETHCLI_EXPLORER_RPS")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| *v > 0.0 && v.is_finite())
        .unwrap_or(DEFAULT_RPS);
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
    // A stored slot far in the future is stale (clock change): ignore it
    let next_free = if next_free > now + 60_000 {
        now
    } else {
        next_free
    };
    let slot = next_free.max(now);
    (slot - now, slot + gap_ms)
}

/// Slot bookkeeping in the shared file. `None` when the file cannot be used
fn shared_wait(gap_ms: u64) -> Option<u64> {
    // Unit tests pace within the process only: they must not touch the
    // machine-wide file other ethcli runs use
    if cfg!(test) {
        return None;
    }
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(pace_file()?)
        .ok()?;
    file.lock().ok()?;
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

/// Whether an explorer error text is a rate-limit answer
pub fn is_rate_limit_text(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("rate limit") || lower.contains("max calls per sec")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_spaced() {
        // Free now: no wait, next slot one gap later
        assert_eq!(take_slot(1_000, 0, 350), (0, 1_350));
        // Taken: wait for it
        assert_eq!(take_slot(1_000, 1_350, 350), (350, 1_700));
        // A stale far-future slot is ignored
        assert_eq!(take_slot(1_000, 1_000_000, 350), (0, 1_350));
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
