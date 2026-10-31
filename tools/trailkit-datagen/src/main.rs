//! `trailkit-datagen` — captures the daily GW2 trading post price archive
//! (data-pipeline plan, phase 0b). Later it also generates the served data
//! files for trailkit-core.
//!
//! It only writes into a local directory and never talks to GitHub; the
//! scheduled workflow uploads the results (data-pipeline D34).

mod archive;
mod archive_file;
mod compact_json;
mod gem_exchange;
mod gw2_api;
mod manifest;
mod orderbook;
mod raw_json;
mod wvw;

use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};

/// Command line of `trailkit-datagen`.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// What to do.
    #[command(subcommand)]
    command: Command,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
enum Command {
    /// Capture today's archive (UTC) into a directory. Data sets that the
    /// day's manifest in that directory lists as complete are skipped, so the
    /// same command serves the daily run and the catch-up run.
    Archive {
        /// Directory for the archive files and `<date>.manifest.json`.
        #[arg(long)]
        out_dir: PathBuf,
        /// Exit with an error if any data set is still incomplete afterwards
        /// (used by the catch-up run so a gap triggers a notification).
        #[arg(long)]
        fail_if_incomplete: bool,
    },
    /// Capture a WvW match snapshot shortly before a weekly reset.
    WvwSnapshot {
        /// Region whose reset follows.
        #[arg(long, value_enum)]
        region: wvw::Region,
        /// Directory for `<date>.wvw-<region>-reset.ndjson.gz`.
        #[arg(long)]
        out_dir: PathBuf,
    },
}

/// Entry point: parses the command line, sets up logging and dispatches.
///
/// # Errors
/// Returns an error (exit code 1) if the command fails.
fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    match Cli::parse().command {
        Command::Archive {
            out_dir,
            fail_if_incomplete,
        } => {
            let manifest = archive::run(&out_dir)?;
            let incomplete = manifest.incomplete_datasets();
            if fail_if_incomplete && !incomplete.is_empty() {
                bail!(
                    "archive {} is still incomplete: {incomplete:?}",
                    manifest.date
                );
            }
        }
        Command::WvwSnapshot { region, out_dir } => {
            std::fs::create_dir_all(&out_dir)?;
            let file = wvw::write_reset_snapshot(&out_dir, archive::today_utc(), region)?;
            log::info!("wrote {} ({} bytes)", file.name, file.size_bytes);
        }
    }
    Ok(())
}
