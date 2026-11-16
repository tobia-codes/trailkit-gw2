//! The per-day archive manifest `<date>.manifest.json` (data-pipeline D33,
//! D35): what was captured when, by which tool version, and whether each data
//! set is complete, partial or failed.

use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use jiff::{Timestamp, civil::Date};
use serde::{Deserialize, Serialize};

use crate::archive_file::ArchiveFile;

/// Version of the manifest format; bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 1;

/// A data set captured by the daily archive run.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, clap::ValueEnum,
)]
#[serde(rename_all = "snake_case")]
pub enum Dataset {
    /// `/v2/commerce/prices` for every tradeable item, raw.
    Prices,
    /// `/v2/commerce/listings`, condensed (D17).
    Orderbook,
    /// `/v2/wvw/matches?ids=all`, raw.
    Wvw,
    /// `/v2/commerce/exchange/{gems,coins}` at the in-game quantities (D31).
    GemExchange,
}

impl Dataset {
    /// Every data set, in capture order.
    pub const ALL: [Self; 4] = [Self::Prices, Self::Orderbook, Self::Wvw, Self::GemExchange];

    /// Archive file name of this data set for `date`.
    #[must_use]
    pub fn file_name(self, date: Date) -> String {
        let suffix = match self {
            Self::Prices => "prices.ndjson.gz",
            Self::Orderbook => "orderbook.ndjson.gz",
            Self::Wvw => "wvw.ndjson.gz",
            Self::GemExchange => "gem-exchange.json.gz",
        };
        format!("{date}.{suffix}")
    }
}

/// Result of capturing one data set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DatasetOutcome {
    /// Every record was captured.
    Complete {
        /// The written file.
        file: ArchiveFile,
        /// Number of records (lines) in the file.
        records: u64,
    },
    /// The file exists, but some ids could not be fetched.
    Partial {
        /// The written file.
        file: ArchiveFile,
        /// Number of records (lines) in the file.
        records: u64,
        /// Ids whose request failed after all retries.
        failed_ids: Vec<u32>,
    },
    /// Nothing could be captured; no file was written.
    Failed {
        /// Human-readable cause.
        error: String,
    },
}

/// Manifest entry of one data set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetEntry {
    /// When the capture of this data set started (it may come from a catch-up run).
    pub captured_at: Timestamp,
    /// What came out of it.
    #[serde(flatten)]
    pub outcome: DatasetOutcome,
}

impl DatasetEntry {
    /// Whether this data set needs no further run.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        matches!(self.outcome, DatasetOutcome::Complete { .. })
    }
}

/// One execution of `trailkit-datagen archive` for the day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Run {
    /// Start of the run.
    pub started_at: Timestamp,
    /// End of the run.
    pub finished_at: Timestamp,
    /// `trailkit-datagen` version.
    pub tool_version: String,
    /// Game build from `/v2/build`, if it could be fetched.
    pub gw2_build: Option<u32>,
    /// Data sets captured in this run.
    pub datasets: Vec<Dataset>,
}

/// The manifest of one archive day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// [`SCHEMA_VERSION`] at the time of writing.
    pub schema_version: u32,
    /// The archive day (UTC).
    pub date: Date,
    /// Every run that touched this day, oldest first.
    pub runs: Vec<Run>,
    /// Latest outcome per data set.
    pub datasets: BTreeMap<Dataset, DatasetEntry>,
}

impl Manifest {
    /// An empty manifest for `date`.
    #[must_use]
    pub const fn new(date: Date) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            date,
            runs: Vec::new(),
            datasets: BTreeMap::new(),
        }
    }

    /// File name of the manifest for `date`.
    #[must_use]
    pub fn file_name(date: Date) -> String {
        format!("{date}.manifest.json")
    }

    /// Loads the manifest for `date` from `directory`, or starts a new one if
    /// there is none (first run of the day).
    ///
    /// # Errors
    /// Returns an error if an existing manifest cannot be read or parsed.
    pub fn load_or_new(directory: &Path, date: Date) -> Result<Self> {
        let path = directory.join(Self::file_name(date));
        if !path.exists() {
            return Ok(Self::new(date));
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
    }

    /// Writes the manifest (pretty-printed) into `directory`.
    ///
    /// # Errors
    /// Returns an error if the file cannot be written.
    pub fn save(&self, directory: &Path) -> Result<()> {
        let path = directory.join(Self::file_name(self.date));
        let text = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, text + "\n").with_context(|| format!("writing {}", path.display()))
    }

    /// Data sets that are missing, partial or failed, in capture order.
    #[must_use]
    pub fn incomplete_datasets(&self) -> Vec<Dataset> {
        Dataset::ALL
            .into_iter()
            .filter(|dataset| {
                !self
                    .datasets
                    .get(dataset)
                    .is_some_and(DatasetEntry::is_complete)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fixed archive day.
    const DATE: Date = jiff::civil::date(2026, 11, 1);

    /// A sample file description.
    fn file(name: &str) -> ArchiveFile {
        ArchiveFile {
            name: name.to_owned(),
            size_bytes: 1,
            sha256: "00".repeat(32),
        }
    }

    #[test]
    fn file_names_follow_the_layout() {
        assert_eq!(
            Dataset::Prices.file_name(DATE),
            "2026-11-01.prices.ndjson.gz"
        );
        assert_eq!(
            Dataset::GemExchange.file_name(DATE),
            "2026-11-01.gem-exchange.json.gz"
        );
        assert_eq!(Manifest::file_name(DATE), "2026-11-01.manifest.json");
    }

    #[test]
    fn only_complete_data_sets_are_skipped() {
        let mut manifest = Manifest::new(DATE);
        assert_eq!(manifest.incomplete_datasets(), Dataset::ALL);
        manifest.datasets.insert(
            Dataset::Prices,
            DatasetEntry {
                captured_at: Timestamp::UNIX_EPOCH,
                outcome: DatasetOutcome::Complete {
                    file: file("p"),
                    records: 1,
                },
            },
        );
        manifest.datasets.insert(
            Dataset::Orderbook,
            DatasetEntry {
                captured_at: Timestamp::UNIX_EPOCH,
                outcome: DatasetOutcome::Partial {
                    file: file("o"),
                    records: 1,
                    failed_ids: vec![24],
                },
            },
        );
        assert_eq!(
            manifest.incomplete_datasets(),
            [Dataset::Orderbook, Dataset::Wvw, Dataset::GemExchange]
        );
    }

    #[test]
    fn json_shape_is_flat_with_status_tag() -> Result<()> {
        let entry = DatasetEntry {
            captured_at: Timestamp::UNIX_EPOCH,
            outcome: DatasetOutcome::Failed {
                error: "HTTP 503".to_owned(),
            },
        };
        let json = serde_json::to_value(&entry)?;
        assert_eq!(json["status"], "failed");
        assert_eq!(json["error"], "HTTP 503");
        assert_eq!(json["captured_at"], "1970-01-01T00:00:00Z");
        let round_trip: DatasetEntry = serde_json::from_value(json)?;
        assert_eq!(round_trip, entry);
        Ok(())
    }
}
