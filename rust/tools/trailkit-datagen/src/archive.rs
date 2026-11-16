//! The daily archive run (data-pipeline D4, D33, D35): captures every data
//! set that today's manifest does not list as complete, writes the files and
//! updates the manifest after each data set.
//!
//! The first run of the day starts with an empty directory and captures
//! everything; the catch-up run starts with the day's manifest downloaded into
//! the directory and only repeats missing, partial or failed data sets.

use std::path::Path;

use anyhow::{Context, Result};
use jiff::{Timestamp, civil::Date, tz::TimeZone};
use serde::Deserialize;

use crate::{
    archive_file::GzipFileWriter,
    gem_exchange,
    gw2_api::{ApiClient, ApiResponse, MAX_IDS_PER_REQUEST, ids_path},
    manifest::{Dataset, DatasetEntry, DatasetOutcome, Manifest, Run},
    orderbook,
    raw_json::array_to_lines,
    wvw,
};

/// Log a progress line every this many batches.
const PROGRESS_EVERY_BATCHES: usize = 25;

/// Response of `/v2/build`.
#[derive(Debug, Deserialize)]
struct Build {
    /// Current game build id.
    id: u32,
}

/// Turns one batch response body into archive lines.
type BatchTransform = fn(&str) -> Result<Vec<String>>;

/// Today's date in UTC (the archive day).
#[must_use]
pub fn today_utc() -> Date {
    Timestamp::now().to_zoned(TimeZone::UTC).date()
}

/// Runs the archive for today into `directory` and returns the updated manifest.
///
/// Network problems never abort the run; they end up as `partial`/`failed`
/// entries in the manifest.
///
/// # Errors
/// Returns an error only for local problems: the directory, a file or the
/// manifest cannot be read or written.
pub fn run(directory: &Path) -> Result<Manifest> {
    std::fs::create_dir_all(directory)
        .with_context(|| format!("creating {}", directory.display()))?;
    let started_at = Timestamp::now();
    let date = today_utc();
    let mut manifest = Manifest::load_or_new(directory, date)?;
    let pending = manifest.incomplete_datasets();
    if pending.is_empty() {
        log::info!("archive {date} is already complete");
        return Ok(manifest);
    }
    log::info!("archive {date}: capturing {pending:?}");

    let mut client = ApiClient::new();
    let gw2_build = fetch_build(&mut client)
        .inspect_err(|error| log::warn!("game build unavailable: {error:#}"))
        .ok();

    for &dataset in &pending {
        let captured_at = Timestamp::now();
        let outcome = capture(&mut client, directory, date, dataset)?;
        log_outcome(dataset, &outcome);
        manifest.datasets.insert(
            dataset,
            DatasetEntry {
                captured_at,
                outcome,
            },
        );
        manifest.save(directory)?;
    }

    manifest.runs.push(Run {
        started_at,
        finished_at: Timestamp::now(),
        tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        gw2_build,
        datasets: pending,
    });
    manifest.save(directory)?;
    Ok(manifest)
}

/// Fetches the current game build id.
///
/// # Errors
/// Returns an error if the request fails or the body is unexpected.
fn fetch_build(client: &mut ApiClient) -> Result<u32> {
    match client.get("/v2/build")? {
        ApiResponse::Found(body) => Ok(serde_json::from_str::<Build>(&body)?.id),
        ApiResponse::NotFound => anyhow::bail!("GET /v2/build: 404"),
    }
}

/// Captures one data set.
///
/// # Errors
/// Returns an error only for local I/O problems.
fn capture(
    client: &mut ApiClient,
    directory: &Path,
    date: Date,
    dataset: Dataset,
) -> Result<DatasetOutcome> {
    let file_name = dataset.file_name(date);
    match dataset {
        Dataset::Prices => capture_by_ids(
            client,
            directory,
            &file_name,
            "/v2/commerce/prices",
            array_to_lines,
        ),
        Dataset::Orderbook => capture_by_ids(
            client,
            directory,
            &file_name,
            "/v2/commerce/listings",
            orderbook::condense_response,
        ),
        Dataset::Wvw => match wvw::fetch_lines(client) {
            Ok(lines) => write_complete(directory, &file_name, &lines),
            Err(error) => Ok(failed(&error)),
        },
        Dataset::GemExchange => match gem_exchange::fetch(client) {
            Ok(document) => write_complete(directory, &file_name, &[document]),
            Err(error) => Ok(failed(&error)),
        },
    }
}

/// Captures an endpoint that lists its ids and serves them in `?ids=` batches.
/// Ids of batches that fail after all retries are recorded as `failed_ids`;
/// ids the API no longer knows (404 / missing in a 206) are simply absent.
///
/// # Errors
/// Returns an error only for local I/O problems.
fn capture_by_ids(
    client: &mut ApiClient,
    directory: &Path,
    file_name: &str,
    endpoint: &str,
    transform: BatchTransform,
) -> Result<DatasetOutcome> {
    let ids = match client.get_ids(endpoint) {
        Ok(ids) => ids,
        Err(error) => return Ok(failed(&error)),
    };
    let mut writer = GzipFileWriter::create(directory, file_name)?;
    let mut records = 0_u64;
    let mut failed_ids = Vec::new();
    let batch_count = ids.len().div_ceil(MAX_IDS_PER_REQUEST);
    for (index, batch) in ids.chunks(MAX_IDS_PER_REQUEST).enumerate() {
        let lines = match client.get(&ids_path(endpoint, batch)) {
            Ok(ApiResponse::Found(body)) => transform(&body),
            Ok(ApiResponse::NotFound) => Ok(Vec::new()),
            Err(error) => Err(error),
        };
        match lines {
            Ok(lines) => {
                for line in &lines {
                    writer.write_line(line)?;
                }
                records += lines.len() as u64;
            }
            Err(error) => {
                log::warn!("{endpoint}: batch {} failed: {error:#}", index + 1);
                failed_ids.extend_from_slice(batch);
            }
        }
        if (index + 1) % PROGRESS_EVERY_BATCHES == 0 {
            log::info!("{endpoint}: {}/{batch_count} batches", index + 1);
        }
    }
    let file = writer.finish()?;

    if !ids.is_empty() && failed_ids.len() == ids.len() {
        let path = directory.join(&file.name);
        std::fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
        return Ok(DatasetOutcome::Failed {
            error: format!("all {batch_count} batches of {endpoint} failed"),
        });
    }
    Ok(if failed_ids.is_empty() {
        DatasetOutcome::Complete { file, records }
    } else {
        DatasetOutcome::Partial {
            file,
            records,
            failed_ids,
        }
    })
}

/// Writes `lines` as a complete data set file.
///
/// # Errors
/// Returns an error if the file cannot be written.
fn write_complete(directory: &Path, file_name: &str, lines: &[String]) -> Result<DatasetOutcome> {
    let mut writer = GzipFileWriter::create(directory, file_name)?;
    for line in lines {
        writer.write_line(line)?;
    }
    Ok(DatasetOutcome::Complete {
        file: writer.finish()?,
        records: lines.len() as u64,
    })
}

/// A `failed` outcome carrying the full error chain.
fn failed(error: &anyhow::Error) -> DatasetOutcome {
    DatasetOutcome::Failed {
        error: format!("{error:#}"),
    }
}

/// Logs a one-line summary of a captured data set.
fn log_outcome(dataset: Dataset, outcome: &DatasetOutcome) {
    match outcome {
        DatasetOutcome::Complete { file, records } => log::info!(
            "{dataset:?}: complete, {records} records, {} bytes",
            file.size_bytes
        ),
        DatasetOutcome::Partial {
            records,
            failed_ids,
            ..
        } => log::warn!(
            "{dataset:?}: partial, {records} records, {} failed ids",
            failed_ids.len()
        ),
        DatasetOutcome::Failed { error } => log::error!("{dataset:?}: failed: {error}"),
    }
}
