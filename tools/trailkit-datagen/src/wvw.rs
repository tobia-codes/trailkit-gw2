//! World vs. World match snapshots: daily as part of the archive and shortly
//! before the weekly EU/NA resets (data-pipeline D15, D29).

use std::path::Path;

use anyhow::{Result, bail};
use jiff::civil::Date;

use crate::{
    archive_file::{ArchiveFile, GzipFileWriter},
    gw2_api::{ApiClient, ApiResponse},
    raw_json::array_to_lines,
};

/// Endpoint returning every current match.
const MATCHES_PATH: &str = "/v2/wvw/matches?ids=all";

/// Region whose weekly reset a snapshot precedes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Region {
    /// Europe (reset Friday 18:00 UTC).
    Eu,
    /// North America (reset Saturday 02:00 UTC).
    Na,
}

impl Region {
    /// Lower-case region code used in file names.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Eu => "eu",
            Self::Na => "na",
        }
    }
}

/// Fetches all matches as compact NDJSON lines (one match per line).
///
/// # Errors
/// Returns an error if the request fails or the body is not a JSON array.
pub fn fetch_lines(client: &mut ApiClient) -> Result<Vec<String>> {
    match client.get(MATCHES_PATH)? {
        ApiResponse::Found(body) => array_to_lines(&body),
        ApiResponse::NotFound => bail!("GET {MATCHES_PATH}: 404"),
    }
}

/// Writes the pre-reset snapshot `<date>.wvw-<region>-reset.ndjson.gz`.
///
/// # Errors
/// Returns an error if fetching or writing fails.
pub fn write_reset_snapshot(directory: &Path, date: Date, region: Region) -> Result<ArchiveFile> {
    let lines = fetch_lines(&mut ApiClient::new())?;
    let mut writer = GzipFileWriter::create(
        directory,
        &format!("{date}.wvw-{}-reset.ndjson.gz", region.code()),
    )?;
    for line in &lines {
        writer.write_line(line)?;
    }
    writer.finish()
}
