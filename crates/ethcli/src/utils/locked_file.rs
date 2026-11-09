//! Locking and atomic writes for the small data files several ethcli
//! processes share (address book, blacklist, signature and token caches).
//!
//! The same pattern as `config.toml`: an exclusive lock on a stable
//! `<file>.lock` next to the file (never renamed or deleted, so every process
//! locks the same inode), and writes that go to a temp file in the same
//! directory, are synced, then renamed over the file. A reader never sees a
//! half-written file, and a read-modify-write under the lock doesn't undo a
//! change another process made meanwhile.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// The stable lock file next to `path`, e.g. `addressbook.toml.lock`
pub fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}

/// Take the exclusive lock for `path`, creating its directory and lock file
/// if needed. The lock is released when the returned file is dropped.
pub fn exclusive_lock(path: &Path) -> std::io::Result<File> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path(path))?;
    lock.lock()?;
    Ok(lock)
}

/// Replace `path` with `content` atomically, with 0600 permissions on Unix.
/// The caller should hold [`exclusive_lock`] if others may write the file.
pub fn atomic_write(path: &Path, content: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let unique = format!(
        "{}.{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let mut temp_name = path.as_os_str().to_owned();
    temp_name.push(".");
    temp_name.push(unique);
    let temp = PathBuf::from(temp_name);

    // Remove the temp file if anything below fails
    struct Guard<'a>(&'a Path, bool);
    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            if !self.1 {
                let _ = std::fs::remove_file(self.0);
            }
        }
    }
    let mut guard = Guard(&temp, false);

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    #[cfg(unix)]
    std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600))?;
    file.write_all(content)?;
    file.sync_all()?;
    drop(file);
    std::fs::rename(&temp, path)?;
    guard.1 = true;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_the_file_and_leaves_no_temp() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.toml");
        std::fs::write(&path, "old").unwrap();
        atomic_write(&path, b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn atomic_write_sets_owner_only_permissions() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.toml");
        atomic_write(&path, b"x").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn exclusive_lock_creates_the_lock_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub").join("data.toml");
        let lock = exclusive_lock(&path).unwrap();
        assert!(lock_path(&path).exists());
        drop(lock);
    }
}
