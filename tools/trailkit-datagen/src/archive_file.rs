//! Writing gzip-compressed archive files and describing them (size, SHA-256)
//! for the manifest.

use std::{
    fs::File,
    io::{BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use flate2::{Compression, write::GzEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A finished file in the archive, as recorded in the manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveFile {
    /// File name (also the release asset name), e.g. `2026-11-01.prices.ndjson.gz`.
    pub name: String,
    /// Size of the compressed file in bytes.
    pub size_bytes: u64,
    /// Lower-case hex SHA-256 of the compressed file.
    pub sha256: String,
}

/// Streams lines into a gzip file (maximum compression, as measured in the plan).
pub struct GzipFileWriter {
    /// Path of the file being written.
    path: PathBuf,
    /// gzip encoder on top of a buffered file.
    encoder: GzEncoder<BufWriter<File>>,
}

impl GzipFileWriter {
    /// Creates (or truncates) `<directory>/<file_name>`.
    ///
    /// # Errors
    /// Returns an error if the file cannot be created.
    pub fn create(directory: &Path, file_name: &str) -> Result<Self> {
        let path = directory.join(file_name);
        let file = File::create(&path).with_context(|| format!("creating {}", path.display()))?;
        Ok(Self {
            path,
            encoder: GzEncoder::new(BufWriter::new(file), Compression::best()),
        })
    }

    /// Writes `line` followed by a newline.
    ///
    /// # Errors
    /// Returns an error if writing fails.
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        self.encoder
            .write_all(line.as_bytes())
            .and_then(|()| self.encoder.write_all(b"\n"))
            .with_context(|| format!("writing {}", self.path.display()))
    }

    /// Finishes the gzip stream, flushes the file and describes it.
    ///
    /// # Errors
    /// Returns an error if finishing, flushing or hashing fails.
    pub fn finish(self) -> Result<ArchiveFile> {
        let mut buffered = self
            .encoder
            .finish()
            .with_context(|| format!("finishing {}", self.path.display()))?;
        buffered
            .flush()
            .with_context(|| format!("flushing {}", self.path.display()))?;
        describe(&self.path)
    }
}

/// Computes size and SHA-256 of an existing file.
///
/// # Errors
/// Returns an error if the file cannot be read or has no UTF-8 file name.
pub fn describe(path: &Path) -> Result<ArchiveFile> {
    let mut file = File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    let mut size_bytes = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("reading {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size_bytes += read as u64;
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid file name {}", path.display()))?
        .to_owned();
    let sha256 = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(ArchiveFile {
        name,
        size_bytes,
        sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;

    #[test]
    fn writes_readable_gzip_lines_and_hashes_them() -> Result<()> {
        let directory =
            std::env::temp_dir().join(format!("trailkit-datagen-test-{}", std::process::id()));
        std::fs::create_dir_all(&directory)?;
        let mut writer = GzipFileWriter::create(&directory, "sample.ndjson.gz")?;
        writer.write_line(r#"{"id":1}"#)?;
        writer.write_line(r#"{"id":2}"#)?;
        let file = writer.finish()?;

        let mut text = String::new();
        GzDecoder::new(File::open(directory.join(&file.name))?).read_to_string(&mut text)?;
        assert_eq!(text, "{\"id\":1}\n{\"id\":2}\n");
        assert_eq!(file.sha256.len(), 64);
        assert_eq!(file, describe(&directory.join(&file.name))?);

        std::fs::remove_dir_all(&directory)?;
        Ok(())
    }
}
