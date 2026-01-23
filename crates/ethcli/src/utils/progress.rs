//! Progress indicator utilities with TTY detection
//!
//! Provides consistent progress output that automatically adapts based on
//! whether stdout/stderr is a terminal (TTY) or being piped/redirected.

use std::io::{IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Check if stderr is a terminal (TTY)
///
/// Returns true if stderr is connected to an interactive terminal,
/// false if it's being piped or redirected.
#[inline]
pub fn is_tty() -> bool {
    std::io::stderr().is_terminal()
}

/// Check if stdout is a terminal (TTY)
#[inline]
pub fn is_stdout_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Print a status message to stderr if it's a TTY
///
/// This is useful for progress messages that should only appear
/// in interactive mode, not when output is being piped.
pub fn status(msg: &str) {
    if is_tty() {
        eprintln!("{}", msg);
    }
}

/// Print a status message with formatting if stderr is a TTY
#[macro_export]
macro_rules! status {
    ($($arg:tt)*) => {
        if $crate::utils::progress::is_tty() {
            eprintln!($($arg)*);
        }
    };
}

/// A simple spinner for long-running operations
///
/// Only shows animation if stderr is a TTY.
pub struct Spinner {
    message: String,
    frames: &'static [&'static str],
    frame_idx: usize,
    start: Instant,
    is_tty: bool,
    stopped: Arc<AtomicBool>,
}

impl Spinner {
    /// Default spinner frames
    const DEFAULT_FRAMES: &'static [&'static str] =
        &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// ASCII-only spinner frames (for non-unicode terminals)
    const ASCII_FRAMES: &'static [&'static str] = &["|", "/", "-", "\\"];

    /// Create a new spinner with the given message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            frames: Self::DEFAULT_FRAMES,
            frame_idx: 0,
            start: Instant::now(),
            is_tty: is_tty(),
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Use ASCII-only frames (for terminals that don't support unicode)
    pub fn ascii(mut self) -> Self {
        self.frames = Self::ASCII_FRAMES;
        self
    }

    /// Start the spinner (prints initial message)
    pub fn start(&self) {
        if self.is_tty {
            eprint!("\r{} {}...", self.frames[0], self.message);
            let _ = std::io::stderr().flush();
        }
    }

    /// Update the spinner frame (call periodically)
    pub fn tick(&mut self) {
        if !self.is_tty || self.stopped.load(Ordering::Relaxed) {
            return;
        }

        self.frame_idx = (self.frame_idx + 1) % self.frames.len();
        eprint!("\r{} {}...", self.frames[self.frame_idx], self.message);
        let _ = std::io::stderr().flush();
    }

    /// Update the message while spinning
    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
        if self.is_tty {
            // Clear the line first
            eprint!("\r\x1b[K");
            eprint!("{} {}...", self.frames[self.frame_idx], self.message);
            let _ = std::io::stderr().flush();
        }
    }

    /// Stop the spinner with a success message
    pub fn success(self, message: &str) {
        self.stopped.store(true, Ordering::Relaxed);
        if self.is_tty {
            eprintln!(
                "\r\x1b[K✓ {} ({:.1}s)",
                message,
                self.start.elapsed().as_secs_f64()
            );
        }
    }

    /// Stop the spinner with an error message
    pub fn error(self, message: &str) {
        self.stopped.store(true, Ordering::Relaxed);
        if self.is_tty {
            eprintln!("\r\x1b[K✗ {}", message);
        }
    }

    /// Stop the spinner without a message (clear the line)
    pub fn stop(self) {
        self.stopped.store(true, Ordering::Relaxed);
        if self.is_tty {
            eprint!("\r\x1b[K");
            let _ = std::io::stderr().flush();
        }
    }

    /// Get elapsed time since spinner started
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        // Ensure we clean up the line if dropped without explicit stop
        if self.is_tty && !self.stopped.load(Ordering::Relaxed) {
            eprint!("\r\x1b[K");
            let _ = std::io::stderr().flush();
        }
    }
}

/// A simple progress bar
pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
    width: usize,
    start: Instant,
    is_tty: bool,
    last_draw: Instant,
}

impl ProgressBar {
    /// Create a new progress bar with the given total
    pub fn new(total: u64, message: impl Into<String>) -> Self {
        Self {
            total,
            current: 0,
            message: message.into(),
            width: 30,
            start: Instant::now(),
            is_tty: is_tty(),
            last_draw: Instant::now(),
        }
    }

    /// Set the bar width (default 30)
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Update progress to a specific value
    pub fn set(&mut self, current: u64) {
        self.current = current.min(self.total);
        self.draw(false);
    }

    /// Increment progress by 1
    pub fn inc(&mut self) {
        self.current = (self.current + 1).min(self.total);
        self.draw(false);
    }

    /// Increment progress by a specific amount
    pub fn inc_by(&mut self, amount: u64) {
        self.current = (self.current + amount).min(self.total);
        self.draw(false);
    }

    /// Draw the progress bar (rate limited to avoid flicker)
    fn draw(&mut self, force: bool) {
        if !self.is_tty {
            return;
        }

        // Rate limit drawing to every 50ms unless forced
        if !force && self.last_draw.elapsed() < Duration::from_millis(50) {
            return;
        }
        self.last_draw = Instant::now();

        let percent = if self.total > 0 {
            (self.current as f64 / self.total as f64 * 100.0) as u64
        } else {
            0
        };

        let filled = if self.total > 0 {
            (self.current as f64 / self.total as f64 * self.width as f64) as usize
        } else {
            0
        };

        let empty = self.width.saturating_sub(filled);
        let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

        let elapsed = self.start.elapsed().as_secs_f64();
        let rate = if elapsed > 0.0 {
            self.current as f64 / elapsed
        } else {
            0.0
        };

        let eta = if rate > 0.0 && self.current < self.total {
            let remaining = self.total - self.current;
            format!(" ETA: {:.0}s", remaining as f64 / rate)
        } else {
            String::new()
        };

        eprint!(
            "\r{} [{}] {:>3}% ({}/{}){}",
            self.message, bar, percent, self.current, self.total, eta
        );
        let _ = std::io::stderr().flush();
    }

    /// Finish the progress bar
    pub fn finish(mut self) {
        self.current = self.total;
        self.draw(true);
        if self.is_tty {
            eprintln!(); // New line after completion
        }
    }

    /// Finish with a message
    pub fn finish_with_message(self, message: &str) {
        if self.is_tty {
            eprintln!(
                "\r\x1b[K{} ({:.1}s)",
                message,
                self.start.elapsed().as_secs_f64()
            );
        }
    }
}

/// Print a progress message only if not in quiet mode and stderr is a TTY
///
/// Use this for "Fetching X..." style messages.
pub fn progress_message(quiet: bool, message: impl std::fmt::Display) {
    if !quiet && is_tty() {
        eprintln!("{}...", message);
    }
}

/// Macro for progress messages that respects quiet mode
#[macro_export]
macro_rules! progress {
    ($quiet:expr, $($arg:tt)*) => {
        if !$quiet && $crate::utils::progress::is_tty() {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_tty() {
        // In tests, this will usually be false when piped
        let _ = is_tty();
        let _ = is_stdout_tty();
    }

    #[test]
    fn test_spinner_creation() {
        let spinner = Spinner::new("Loading");
        assert!(!spinner.stopped.load(Ordering::Relaxed));
        spinner.stop();
    }

    #[test]
    fn test_spinner_ascii() {
        let spinner = Spinner::new("Loading").ascii();
        assert_eq!(spinner.frames, Spinner::ASCII_FRAMES);
        spinner.stop();
    }

    #[test]
    fn test_progress_bar_creation() {
        let mut bar = ProgressBar::new(100, "Processing");
        bar.set(50);
        assert_eq!(bar.current, 50);
        bar.inc();
        assert_eq!(bar.current, 51);
        bar.inc_by(10);
        assert_eq!(bar.current, 61);
    }

    #[test]
    fn test_progress_bar_overflow() {
        let mut bar = ProgressBar::new(100, "Test");
        bar.set(150); // Should cap at total
        assert_eq!(bar.current, 100);
    }

    #[test]
    fn test_progress_bar_zero_total() {
        let bar = ProgressBar::new(0, "Empty");
        assert_eq!(bar.total, 0);
    }
}
