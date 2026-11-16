//! Loading the worker's documents at start and the lines that log it
//! (ROADMAP R92): `load_finished` once per session, unusual loads as
//! `load_migrated`, `load_recovered` or `load_read_only`.

use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

use jiff::Timestamp;
use trailkit_log::ErrorChain;
use trailkit_storage::{DocumentStore, FileName, LoadOutcome, ReadOnlyReason, VersionedDocument};

use super::{LoadedDocuments, Permissions, SaveReport, SavingAllowed};
use crate::{
    drafts::DraftsDocument,
    note_tree::{NotesDocument, TreeNode},
    settings::{Settings, TextScale},
    ui_state::UiState,
};

/// What the caller learned when it read `settings.json`, for the
/// `load_finished` line.
#[derive(Debug, Clone, Copy, Default)]
pub struct SettingsSummary {
    /// Whether the file did not exist.
    pub missing: bool,
    /// The text size the addon starts with.
    pub text_scale: TextScale,
    /// How long reading it took.
    pub elapsed: Duration,
}

/// Loads the worker's documents and logs the `load_finished` summary.
/// `reports` is handed to the UI with the documents.
pub(super) fn load_documents(
    store: &DocumentStore,
    saving_allowed: SavingAllowed,
    settings_summary: SettingsSummary,
    reports: Receiver<SaveReport>,
) -> (LoadedDocuments, Permissions) {
    let started = Instant::now();
    let now = Timestamp::now();
    let notes = load_logged::<NotesDocument>(store, now);
    let ui_state_outcome = load_logged::<UiState>(store, now);
    let drafts_outcome = load_logged::<DraftsDocument>(store, now);
    let permissions = Permissions {
        notes: notes.allows_saving(),
        settings: saving_allowed.settings,
        ui_state: ui_state_outcome.allows_saving(),
        drafts: drafts_outcome.allows_saving(),
    };
    let mut missing = Vec::new();
    if settings_summary.missing {
        missing.push(document_file_name::<Settings>());
    }
    missing.extend(missing_file(&notes));
    missing.extend(missing_file(&ui_state_outcome));
    let ui_state = ui_state_outcome.into_document().unwrap_or_default();
    let drafts = drafts_outcome.into_document().unwrap_or_default();
    log_load_finished(
        &LoadCounts {
            notes: note_count(&notes),
            drafts: drafts.drafts.len(),
        },
        &missing,
        settings_summary,
        started.elapsed() + settings_summary.elapsed,
    );
    let documents = LoadedDocuments {
        notes,
        ui_state,
        drafts,
        reports,
    };
    (documents, permissions)
}

/// Loads `D` from `store` and logs the outcome with its duration.
fn load_logged<D: VersionedDocument>(store: &DocumentStore, now: Timestamp) -> LoadOutcome<D> {
    let started = Instant::now();
    let outcome = store.load::<D>(now);
    log_load_outcome(&outcome, started.elapsed());
    outcome
}

/// The file name of document `D`, e.g. `notes.json`.
pub(super) fn document_file_name<D: VersionedDocument>() -> String {
    format!("{}.json", D::FILE_STEM)
}

/// The file name of `D` if `outcome` found no file.
fn missing_file<D: VersionedDocument>(outcome: &LoadOutcome<D>) -> Option<String> {
    matches!(outcome, LoadOutcome::Missing).then(document_file_name::<D>)
}

/// How many notes `outcome` holds (0 without a document).
fn note_count(outcome: &LoadOutcome<NotesDocument>) -> usize {
    match outcome {
        LoadOutcome::Loaded { document, .. } => document
            .root
            .iter()
            .map(|node| match node {
                TreeNode::Note(_) => 1,
                TreeNode::Group(group) => group.note_count(),
            })
            .sum(),
        _ => 0,
    }
}

/// What the documents held when they were loaded.
struct LoadCounts {
    /// Notes in the tree, without the trash.
    notes: usize,
    /// Saved drafts.
    drafts: usize,
}

/// Logs the one `load_finished` line of a session (ROADMAP R92): that loading
/// ended, and how much there was. `missing` names the files that do not exist
/// (drafts are normally absent and not listed); outside a first start a
/// missing `notes.json` explains "my notes are gone".
fn log_load_finished(
    counts: &LoadCounts,
    missing: &[String],
    settings: SettingsSummary,
    elapsed: Duration,
) {
    let LoadCounts { notes, drafts } = *counts;
    let text_scale = f64::from(settings.text_scale.factor());
    let ms = elapsed.as_millis();
    if missing.is_empty() {
        log::info!(notes, drafts, text_scale, ms; "load_finished");
    } else {
        let missing = missing.join(",");
        log::info!(notes, drafts, missing = missing.as_str(), text_scale, ms; "load_finished");
    }
}

/// Logs a load that was not routine (ROADMAP R92): `load_migrated` (INFO),
/// `load_recovered` (ERROR, the unreadable file was moved to `backups/`) or
/// `load_read_only` (newer version: WARN, I/O error: ERROR), each with the
/// file name and the duration. A found or missing file logs nothing here;
/// [`log_load_finished`] sums those up.
pub fn log_load_outcome<D: VersionedDocument>(outcome: &LoadOutcome<D>, elapsed: Duration) {
    let file = document_file_name::<D>();
    let file = file.as_str();
    let ms = elapsed.as_millis();
    match outcome {
        LoadOutcome::Missing
        | LoadOutcome::Loaded {
            migrated_from: None,
            ..
        } => {}
        LoadOutcome::Loaded {
            migrated_from: Some(version),
            ..
        } => log::info!(file, from_version = version.0, ms; "load_migrated"),
        LoadOutcome::Recovered {
            reason,
            preserved_as,
        } => log::error!(
            file,
            error = reason.as_str(),
            preserved_as:% = FileName::of(preserved_as),
            ms;
            "load_recovered"
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { found }) => {
            log::warn!(file, reason = "newer_version", found = found.0, ms; "load_read_only");
        }
        LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)) => {
            log::error!(file, reason = "io", error:% = ErrorChain(error), ms; "load_read_only");
        }
    }
}
