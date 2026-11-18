//! World vs. World match snapshots: daily as part of the archive and shortly
//! before the weekly EU/NA resets (data-pipeline D15, D29).

use std::path::Path;

use anyhow::{Context, Result, bail};
use jiff::{Timestamp, civil::Date};
use serde::Deserialize;

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

    /// Prefix of this region's match ids (`1-…` NA, `2-…` EU).
    const fn match_id_prefix(self) -> &'static str {
        match self {
            Self::Eu => "2-",
            Self::Na => "1-",
        }
    }
}

/// The fields of one match object needed to check the match period.
#[derive(Debug, Deserialize)]
struct MatchPeriod {
    /// Match id such as `2-1`; the first digit is the region.
    id: String,
    /// End of the match week (RFC 3339, the next reset).
    end_time: String,
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

/// Checks that `lines` still hold the region's pre-reset matches (D42).
///
/// A delayed run after the reset would otherwise save the new week's matches
/// under the pre-reset name. The check uses the API's own `end_time`, not
/// the runner clock alone: every match of `region` must end after `now`.
///
/// # Errors
/// Returns an error if a line is not a match object with `id`/`end_time`, an
/// `end_time` is not a timestamp, the region has no match, or a match of the
/// region has already ended (the reset has passed).
fn ensure_before_reset(lines: &[String], region: Region, now: Timestamp) -> Result<()> {
    let mut region_matches = 0_usize;
    for line in lines {
        let period: MatchPeriod =
            serde_json::from_str(line).context("WvW match without id/end_time")?;
        if !period.id.starts_with(region.match_id_prefix()) {
            continue;
        }
        let end: Timestamp = period
            .end_time
            .parse()
            .with_context(|| format!("match {}: invalid end_time", period.id))?;
        if end <= now {
            bail!(
                "match {} ended at {end}, before {now}: the {} reset has passed, no snapshot written",
                period.id,
                region.code()
            );
        }
        region_matches += 1;
    }
    if region_matches == 0 {
        bail!("no {} match in the API response", region.code());
    }
    Ok(())
}

/// Writes the pre-reset snapshot `<date>.wvw-<region>-reset.ndjson.gz`.
///
/// Writes nothing if the reset has already passed (D42).
///
/// # Errors
/// Returns an error if fetching fails, the matches are already from after
/// the reset (see [`ensure_before_reset`]) or writing fails.
pub fn write_reset_snapshot(directory: &Path, date: Date, region: Region) -> Result<ArchiveFile> {
    let lines = fetch_lines(&mut ApiClient::new())?;
    ensure_before_reset(&lines, region, Timestamp::now())?;
    let mut writer = GzipFileWriter::create(
        directory,
        &format!("{date}.wvw-{}-reset.ndjson.gz", region.code()),
    )?;
    for line in &lines {
        writer.write_line(line)?;
    }
    writer.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One EU and one NA match, both ending at their next reset.
    fn lines() -> Vec<String> {
        vec![
            r#"{"id":"1-1","start_time":"2026-11-14T02:00:00Z","end_time":"2026-11-21T02:00:00Z"}"#
                .to_owned(),
            r#"{"id":"2-1","start_time":"2026-11-13T18:00:00Z","end_time":"2026-11-20T18:00:00Z"}"#
                .to_owned(),
        ]
    }

    /// Parses a test timestamp.
    fn at(text: &str) -> Timestamp {
        text.parse().expect("valid test timestamp")
    }

    #[test]
    fn accepts_run_before_reset() {
        ensure_before_reset(&lines(), Region::Eu, at("2026-11-20T17:23:00Z")).unwrap();
        ensure_before_reset(&lines(), Region::Na, at("2026-11-21T01:23:00Z")).unwrap();
    }

    #[test]
    fn rejects_run_after_reset() {
        let error =
            ensure_before_reset(&lines(), Region::Eu, at("2026-11-20T22:12:00Z")).unwrap_err();
        assert!(error.to_string().contains("eu reset has passed"), "{error}");
        assert!(ensure_before_reset(&lines(), Region::Na, at("2026-11-21T02:00:00Z")).is_err());
    }

    #[test]
    fn ignores_other_region() {
        // EU has reset, NA has not: the NA snapshot is still valid.
        ensure_before_reset(&lines(), Region::Na, at("2026-11-20T19:00:00Z")).unwrap();
    }

    #[test]
    fn rejects_missing_region_and_bad_fields() {
        let only_na = vec![lines()[0].clone()];
        assert!(ensure_before_reset(&only_na, Region::Eu, at("2026-11-20T17:00:00Z")).is_err());
        let bad_time = vec![r#"{"id":"2-1","end_time":"soon"}"#.to_owned()];
        assert!(ensure_before_reset(&bad_time, Region::Eu, at("2026-11-20T17:00:00Z")).is_err());
        let no_end = vec![r#"{"id":"2-1"}"#.to_owned()];
        assert!(ensure_before_reset(&no_end, Region::Eu, at("2026-11-20T17:00:00Z")).is_err());
    }
}
