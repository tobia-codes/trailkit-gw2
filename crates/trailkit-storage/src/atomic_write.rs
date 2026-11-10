//! Crash-safe file replacement (write a temporary file, flush, rename).

use std::{
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// The step of writing a document file that failed, logged as `stage=`
/// (memo Q166).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteStage {
    /// Creating the folder that holds the file.
    CreateFolder,
    /// Creating or writing the temporary file.
    WriteTemp,
    /// Flushing the temporary file to disk.
    Flush,
    /// Renaming the temporary file over the live file.
    Rename,
    /// Deleting the file.
    Delete,
}

impl WriteStage {
    /// The stage as written to the log, e.g. `write_temp`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateFolder => "create_folder",
            Self::WriteTemp => "write_temp",
            Self::Flush => "flush",
            Self::Rename => "rename",
            Self::Delete => "delete",
        }
    }
}

impl fmt::Display for WriteStage {
    /// Writes [`WriteStage::as_str`].
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A failed [`write_atomically`]: the I/O error and the step it happened in.
#[derive(Debug, thiserror::Error)]
#[error("{stage} failed: {source}")]
pub struct WriteError {
    /// The step that failed.
    pub stage: WriteStage,
    /// The error the operating system reported.
    #[source]
    pub source: io::Error,
}

impl WriteError {
    /// Returns a closure that wraps an I/O error of `stage`, for `map_err`.
    fn at(stage: WriteStage) -> impl FnOnce(io::Error) -> Self {
        move |source| Self { stage, source }
    }
}

impl From<WriteError> for io::Error {
    /// Drops the stage, for callers that only report the I/O error.
    fn from(error: WriteError) -> Self {
        error.source
    }
}

/// Replaces the file at `path` with `content` so that the file is always
/// either the old or the new version, never a partial write.
///
/// The content is written to `<file name>.tmp` in the same directory, flushed
/// to disk and then renamed over `path` (on Windows `std::fs::rename` replaces
/// an existing file). The parent directory must exist.
///
/// # Errors
///
/// Returns a [`WriteError`] with the step that failed if writing, flushing
/// or renaming fails; the temporary file is removed on a best-effort basis in
/// that case and the existing file at `path` is left untouched.
pub fn write_atomically(path: &Path, content: &[u8]) -> Result<(), WriteError> {
    let temporary_path = temporary_path_for(path);
    let result = write_and_flush(&temporary_path, content).and_then(|()| {
        fs::rename(&temporary_path, path).map_err(WriteError::at(WriteStage::Rename))
    });
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
/// Returns a [`WriteError`] at [`WriteStage::WriteTemp`] if creating or
/// writing the file fails, at [`WriteStage::Flush`] if syncing it fails.
fn write_and_flush(path: &Path, content: &[u8]) -> Result<(), WriteError> {
    let mut file = File::create(path).map_err(WriteError::at(WriteStage::WriteTemp))?;
    file.write_all(content)
        .map_err(WriteError::at(WriteStage::WriteTemp))?;
    file.sync_all().map_err(WriteError::at(WriteStage::Flush))
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

        let error = write_atomically(&path, b"new").err();
        assert_eq!(error.map(|error| error.stage), Some(WriteStage::WriteTemp));
        assert_eq!(fs::read(&path)?, b"original");
        Ok(())
    }
}
