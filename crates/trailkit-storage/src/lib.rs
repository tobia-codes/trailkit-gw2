//! Versioned, crash-safe JSON persistence for trailkit Nexus addons.
//!
//! Every file an addon writes goes through this crate (ROADMAP R22, memo plan
//! M29, nexus-addon-dev skill "Persisted user data"):
//!
//! - Documents are pretty-printed JSON with a top-level `schema_version`
//!   ([`VersionedDocument`]).
//! - Saves are atomic: the content is written to a temporary file, flushed to
//!   disk and renamed over the live file ([`write_atomically`]), so a crash
//!   never leaves a half-written document behind.
//! - Older versions are migrated forward step by step; a copy of the original
//!   file is kept before anything is saved in the new format.
//! - A file with a *newer* `schema_version` is never overwritten: the caller
//!   gets [`LoadOutcome::ReadOnly`] and must not save.
//! - An unreadable file is never overwritten either: it is moved into the
//!   `backups/` folder and the caller starts fresh ([`LoadOutcome::Recovered`]).
//! - All safety copies live in one `backups/` subfolder with a fixed cap per
//!   kind, so the addon folder does not fill up with forgotten copies (M33).
//!
//! This crate does no threading of its own: callers run it on their worker
//! thread (nexus-addon-dev skill §2) — never on the render thread.

mod atomic_write;
mod document_store;
mod safety_copies;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use atomic_write::write_atomically;
pub use document_store::{DocumentStore, LoadOutcome, ReadOnlyReason, StorageError};

/// Name of the subfolder (next to the live files) that holds every safety copy.
pub const BACKUPS_DIRECTORY_NAME: &str = "backups";

/// Version of a persisted document format, stored as `schema_version`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    /// The first version of every document format.
    pub const FIRST: Self = Self(1);

    /// The version directly after this one.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl std::fmt::Display for SchemaVersion {
    /// Formats the version as its plain number.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// The fields of a document as untyped JSON — only used while migrating.
pub type DocumentFields = serde_json::Map<String, serde_json::Value>;

/// A document type that is persisted as one JSON file.
///
/// The `schema_version` field is added and removed by [`DocumentStore`]; the
/// type itself must not declare it. Unknown fields are ignored when reading
/// (memo plan M32), so types must not use `#[serde(deny_unknown_fields)]`.
pub trait VersionedDocument: Serialize + DeserializeOwned {
    /// File name without extension, e.g. `"notes"` for `notes.json`.
    const FILE_STEM: &'static str;

    /// The version this build reads and writes.
    const CURRENT_SCHEMA_VERSION: SchemaVersion;

    /// Migrates `fields` from version `from` to `from.next()`.
    ///
    /// Called repeatedly until the document reaches
    /// [`Self::CURRENT_SCHEMA_VERSION`]. The default has no migrations, which is
    /// correct while a format is still at [`SchemaVersion::FIRST`].
    ///
    /// # Errors
    ///
    /// Returns [`MigrationError`] if the document cannot be converted; the file
    /// is then treated as unreadable and preserved as a safety copy.
    fn migrate_one_step(
        from: SchemaVersion,
        fields: DocumentFields,
    ) -> Result<DocumentFields, MigrationError> {
        let _ = fields;
        Err(MigrationError::NoMigrationPath { from })
    }
}

/// Why a document could not be migrated to the current version.
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    /// This build has no migration starting at `from`.
    #[error("no migration from schema version {from}")]
    NoMigrationPath {
        /// The version the migration would have started from.
        from: SchemaVersion,
    },
    /// The document content did not have the shape the migration expected.
    #[error("invalid document for schema version {from}: {reason}")]
    InvalidDocument {
        /// The version the migration started from.
        from: SchemaVersion,
        /// Human-readable description of the problem (for the log).
        reason: String,
    },
}
