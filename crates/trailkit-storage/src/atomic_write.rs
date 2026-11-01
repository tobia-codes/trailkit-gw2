//! Crash-safe file replacement (write a temporary file, flush, rename).

use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Replaces the file at `path` with `content` so that the file is always
/// either the old or the new version, never a partial write.
///
/// The content is written to `<file name>.tmp` in the same directory, flushed
/// to disk and then renamed over `path` (on Windows `std::fs::rename` replaces
/// an existing file). The parent directory must exist.
///
/// # Errors
///
/// Returns the underlying I/O error if writing, flushing or renaming fails;
/// the temporary file is removed on a best-effort basis in that case and the
/// existing file at `path` is left untouched.
pub fn write_atomically(path: &Path, content: &[u8]) -> io::Result<()> {
    let temporary_path = temporary_path_for(path);
    let result =
        write_and_flush(&temporary_path, content).and_then(|()| fs::rename(&temporary_path, path));
    if result.is_err() {
        // Best effort: a leftover temporary file is harmless and overwritten next time.
        let _ = fs::remove_file(&temporary_path);
    }
    result
}

/// Writes `content` to a newly created file at `path` and flushes it to disk.
///
/// # Errors
///
/// Returns the I/O error of creating, writing or syncing the file.
fn write_and_flush(path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content)?;
    file.sync_all()
}

/// Returns the temporary file path used while replacing `path`
/// (`notes.json` → `notes.json.tmp`).
fn temporary_path_for(path: &Path) -> PathBuf {
    let mut file_name = path.file_name().unwrap_or_default().to_os_string();
    file_name.push(".tmp");
    path.with_file_name(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_replaces_file_without_leftovers() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("notes.json");

        write_atomically(&path, b"first")?;
        write_atomically(&path, b"second")?;

        assert_eq!(fs::read(&path)?, b"second");
        assert!(!temporary_path_for(&path).exists());
        Ok(())
    }

    #[test]
    fn failure_leaves_existing_file_untouched() -> io::Result<()> {
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("notes.json");
        write_atomically(&path, b"original")?;
        // A directory in place of the temporary file makes the write fail.
        fs::create_dir(temporary_path_for(&path))?;

        assert!(write_atomically(&path, b"new").is_err());
        assert_eq!(fs::read(&path)?, b"original");
        Ok(())
    }
}
