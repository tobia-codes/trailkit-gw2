//! Loading and saving one [`VersionedDocument`] per file in an addon folder.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use jiff::Timestamp;
use serde::Serialize;
use serde_json::Value;

use crate::{
    BACKUPS_DIRECTORY_NAME, DocumentFields, SchemaVersion, VersionedDocument, safety_copies,
    write_atomically,
};

/// JSON key of the format version in every document (ROADMAP R22).
const SCHEMA_VERSION_KEY: &str = "schema_version";

/// Reads and writes versioned documents in one directory (the addon folder)
/// and keeps their safety copies in its `backups/` subfolder.
#[derive(Debug, Clone)]
pub struct DocumentStore {
    /// Folder holding the live files, e.g. `addons/trailkit/memo`.
    directory: PathBuf,
}

/// Result of [`DocumentStore::load`].
#[derive(Debug)]
pub enum LoadOutcome<T> {
    /// No file exists yet; start with the default document.
    Missing,
    /// The document was read, migrated forward if `migrated_from` is set.
    Loaded {
        /// The document in the current format.
        document: T,
        /// The version the file had before migration, if it was older.
        migrated_from: Option<SchemaVersion>,
    },
    /// The file could not be understood. It was moved into the backups folder
    /// (`preserved_as`); start with the default document and tell the user.
    Recovered {
        /// Why the file was unreadable (for the log).
        reason: String,
        /// Where the original file now lives.
        preserved_as: PathBuf,
    },
    /// The file must not be overwritten; run without saving and tell the user.
    ReadOnly(ReadOnlyReason),
}

/// Why a document is loaded read-only.
#[derive(Debug)]
pub enum ReadOnlyReason {
    /// The file was written by a newer addon version (ROADMAP R22).
    NewerVersion {
        /// The version found in the file.
        found: SchemaVersion,
    },
    /// The file exists but could not be read, preserved or safety-copied.
    Io(io::Error),
}

/// Why a document could not be saved.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// The document could not be turned into a JSON object.
    #[error("cannot serialize document: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The document serialized to something other than a JSON object.
    #[error("document does not serialize to a JSON object")]
    NotAnObject,
    /// Writing the file failed.
    #[error("cannot write document: {0}")]
    Io(#[from] io::Error),
}

impl<T> LoadOutcome<T> {
    /// Whether the caller may save the document after this outcome.
    #[must_use]
    pub const fn allows_saving(&self) -> bool {
        !matches!(self, Self::ReadOnly(_))
    }

    /// The loaded document, if there is one.
    #[must_use]
    pub fn into_document(self) -> Option<T> {
        match self {
            Self::Loaded { document, .. } => Some(document),
            Self::Missing | Self::Recovered { .. } | Self::ReadOnly(_) => None,
        }
    }
}

/// Wrapper that writes `schema_version` as the first key of a document.
#[derive(Serialize)]
struct Envelope<'document, T> {
    /// The format version of `document`.
    schema_version: SchemaVersion,
    /// The document's own fields, written next to `schema_version`.
    #[serde(flatten)]
    document: &'document T,
}

impl DocumentStore {
    /// Creates a store for the live files in `directory`. Nothing is touched on
    /// disk until a document is loaded or saved.
    #[must_use]
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    /// The folder holding the live files.
    #[must_use]
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// The `backups/` subfolder holding all safety copies.
    #[must_use]
    pub fn backups_directory(&self) -> PathBuf {
        self.directory.join(BACKUPS_DIRECTORY_NAME)
    }

    /// Path of the live file of document type `T`.
    #[must_use]
    pub fn path_of<T: VersionedDocument>(&self) -> PathBuf {
        self.directory.join(format!("{}.json", T::FILE_STEM))
    }

    /// Loads the document of type `T`, migrating it forward if needed.
    ///
    /// Also prunes expired migration copies of `T` (memo plan M33). `now` is
    /// the current time, injected so tests are deterministic. Never fails:
    /// every problem is reported through the returned [`LoadOutcome`].
    pub fn load<T: VersionedDocument>(&self, now: Timestamp) -> LoadOutcome<T> {
        let backups_directory = self.backups_directory();
        safety_copies::prune_expired_migration_copies(&backups_directory, T::FILE_STEM, now);

        let path = self.path_of::<T>();
        let content = match fs::read(&path) {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return LoadOutcome::Missing,
            Err(error) => return LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)),
        };

        match parse::<T>(&content) {
            Ok(Parsed::Current(document)) => LoadOutcome::Loaded {
                document,
                migrated_from: None,
            },
            Ok(Parsed::Migrated { document, from }) => {
                // The original must be safe before anything is saved in the new format.
                match safety_copies::write_migration_copy(
                    &backups_directory,
                    T::FILE_STEM,
                    from,
                    &content,
                ) {
                    Ok(_) => LoadOutcome::Loaded {
                        document,
                        migrated_from: Some(from),
                    },
                    Err(error) => LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)),
                }
            }
            Ok(Parsed::Newer(found)) => {
                LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { found })
            }
            Err(reason) => {
                match safety_copies::preserve_unreadable_file(
                    &path,
                    &backups_directory,
                    T::FILE_STEM,
                    now,
                ) {
                    Ok(preserved_as) => LoadOutcome::Recovered {
                        reason,
                        preserved_as,
                    },
                    Err(error) => LoadOutcome::ReadOnly(ReadOnlyReason::Io(error)),
                }
            }
        }
    }

    /// Saves `document` atomically as pretty-printed JSON with the current
    /// `schema_version`, creating the folder if needed.
    ///
    /// Callers must not call this after a [`LoadOutcome`] whose
    /// [`allows_saving`](LoadOutcome::allows_saving) is `false`.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] if the document cannot be serialized or the
    /// file cannot be written; the previous file stays intact in that case.
    pub fn save<T: VersionedDocument>(&self, document: &T) -> Result<(), StorageError> {
        // A document with no fields of its own would not be a JSON object.
        if !serde_json::to_value(document)?.is_object() {
            return Err(StorageError::NotAnObject);
        }
        let envelope = Envelope {
            schema_version: T::CURRENT_SCHEMA_VERSION,
            document,
        };
        let mut content = serde_json::to_vec_pretty(&envelope)?;
        content.push(b'\n');
        fs::create_dir_all(&self.directory)?;
        write_atomically(&self.path_of::<T>(), &content)?;
        Ok(())
    }
}

impl DocumentStore {
    /// Deletes the file of document type `T`, e.g. when a document became
    /// empty and should not linger on disk. A missing file is not an error.
    ///
    /// # Errors
    ///
    /// Returns [`StorageError`] if the file exists but cannot be deleted.
    pub fn delete<T: VersionedDocument>(&self) -> Result<(), StorageError> {
        match fs::remove_file(self.path_of::<T>()) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error.into()),
            _ => Ok(()),
        }
    }
}

/// A successfully parsed file.
enum Parsed<T> {
    /// The file was in the current format.
    Current(T),
    /// The file was older and has been migrated in memory.
    Migrated {
        /// The document in the current format.
        document: T,
        /// The version the file had.
        from: SchemaVersion,
    },
    /// The file was written by a newer version and was not parsed further.
    Newer(SchemaVersion),
}

/// Parses `content` as a document of type `T`, migrating older versions.
///
/// # Errors
///
/// Returns a human-readable reason if the content is not valid JSON, has no
/// valid `schema_version`, cannot be migrated or does not match `T`.
fn parse<T: VersionedDocument>(content: &[u8]) -> Result<Parsed<T>, String> {
    let value: Value = serde_json::from_slice(content).map_err(|error| error.to_string())?;
    let Value::Object(mut fields) = value else {
        return Err("document is not a JSON object".to_owned());
    };
    let version = fields
        .remove(SCHEMA_VERSION_KEY)
        .and_then(|version| version.as_u64())
        .and_then(|version| u32::try_from(version).ok())
        .map(SchemaVersion)
        .filter(|version| *version >= SchemaVersion::FIRST)
        .ok_or_else(|| format!("missing or invalid `{SCHEMA_VERSION_KEY}`"))?;

    if version > T::CURRENT_SCHEMA_VERSION {
        return Ok(Parsed::Newer(version));
    }
    let fields = migrate_to_current::<T>(version, fields)?;
    let document =
        serde_json::from_value(Value::Object(fields)).map_err(|error| error.to_string())?;
    Ok(if version == T::CURRENT_SCHEMA_VERSION {
        Parsed::Current(document)
    } else {
        Parsed::Migrated {
            document,
            from: version,
        }
    })
}

/// Applies [`VersionedDocument::migrate_one_step`] until `fields` reach the
/// current version.
///
/// # Errors
///
/// Returns the failing migration step's error as text.
fn migrate_to_current<T: VersionedDocument>(
    mut version: SchemaVersion,
    mut fields: DocumentFields,
) -> Result<DocumentFields, String> {
    while version < T::CURRENT_SCHEMA_VERSION {
        fields = T::migrate_one_step(version, fields).map_err(|error| error.to_string())?;
        version = version.next();
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    use crate::MigrationError;

    /// Test document in format version 2; version 1 called `title` `name`.
    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    struct TestDocument {
        /// Some text.
        title: String,
    }

    impl VersionedDocument for TestDocument {
        const FILE_STEM: &'static str = "test";
        const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(2);

        fn migrate_one_step(
            from: SchemaVersion,
            mut fields: DocumentFields,
        ) -> Result<DocumentFields, MigrationError> {
            match from.0 {
                1 => {
                    let name =
                        fields
                            .remove("name")
                            .ok_or_else(|| MigrationError::InvalidDocument {
                                from,
                                reason: "missing `name`".to_owned(),
                            })?;
                    fields.insert("title".to_owned(), name);
                    Ok(fields)
                }
                _ => Err(MigrationError::NoMigrationPath { from }),
            }
        }
    }

    /// Creates a store in a fresh temporary directory.
    fn store() -> io::Result<(tempfile::TempDir, DocumentStore)> {
        let directory = tempfile::tempdir()?;
        let store = DocumentStore::new(directory.path());
        Ok((directory, store))
    }

    /// Writes raw file content for the test document.
    fn write_raw(store: &DocumentStore, content: &str) -> io::Result<()> {
        fs::write(store.path_of::<TestDocument>(), content)
    }

    #[test]
    fn delete_removes_the_file_and_tolerates_a_missing_one()
    -> Result<(), Box<dyn std::error::Error>> {
        let (_directory, store) = store()?;
        store.delete::<TestDocument>()?;
        store.save(&TestDocument::default())?;
        assert!(store.path_of::<TestDocument>().exists());
        store.delete::<TestDocument>()?;
        assert!(!store.path_of::<TestDocument>().exists());
        Ok(())
    }

    #[test]
    fn missing_file_is_reported() -> io::Result<()> {
        let (_directory, store) = store()?;
        let outcome = store.load::<TestDocument>(Timestamp::now());
        assert!(matches!(outcome, LoadOutcome::Missing));
        Ok(())
    }

    #[test]
    fn save_then_load_round_trips_with_version_first() -> Result<(), Box<dyn std::error::Error>> {
        let (_directory, store) = store()?;
        let document = TestDocument {
            title: "Dailies".to_owned(),
        };
        store.save(&document)?;

        let content = fs::read_to_string(store.path_of::<TestDocument>())?;
        assert!(
            content.starts_with("{\n  \"schema_version\": 2,"),
            "{content}"
        );
        let loaded = store.load::<TestDocument>(Timestamp::now()).into_document();
        assert_eq!(loaded, Some(document));
        Ok(())
    }

    #[test]
    fn unknown_fields_are_ignored() -> io::Result<()> {
        let (_directory, store) = store()?;
        write_raw(
            &store,
            r#"{"schema_version": 2, "title": "a", "future": true}"#,
        )?;
        let outcome = store.load::<TestDocument>(Timestamp::now());
        assert!(matches!(
            outcome,
            LoadOutcome::Loaded {
                migrated_from: None,
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn older_version_is_migrated_and_copied() -> io::Result<()> {
        let (_directory, store) = store()?;
        let original = r#"{"schema_version": 1, "name": "old"}"#;
        write_raw(&store, original)?;

        let outcome = store.load::<TestDocument>(Timestamp::now());
        let LoadOutcome::Loaded {
            document,
            migrated_from,
        } = outcome
        else {
            panic!("expected Loaded, got {outcome:?}");
        };
        assert_eq!(document.title, "old");
        assert_eq!(migrated_from, Some(SchemaVersion(1)));
        let copy = store.backups_directory().join("test.v1.bak.json");
        assert_eq!(fs::read_to_string(copy)?, original);
        Ok(())
    }

    #[test]
    fn newer_version_is_read_only_and_untouched() -> io::Result<()> {
        let (_directory, store) = store()?;
        let content = r#"{"schema_version": 3, "whatever": 1}"#;
        write_raw(&store, content)?;

        let outcome = store.load::<TestDocument>(Timestamp::now());
        assert!(matches!(
            outcome,
            LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion {
                found: SchemaVersion(3)
            })
        ));
        assert!(!outcome.allows_saving());
        assert_eq!(
            fs::read_to_string(store.path_of::<TestDocument>())?,
            content
        );
        Ok(())
    }

    #[test]
    fn unreadable_file_is_moved_to_backups() -> io::Result<()> {
        for content in [
            "{ not json",
            "[]",
            r#"{"title": "no version"}"#,
            r#"{"schema_version": 2, "title": 5}"#,
        ] {
            let (_directory, store) = store()?;
            write_raw(&store, content)?;

            let outcome = store.load::<TestDocument>(Timestamp::now());
            let LoadOutcome::Recovered { preserved_as, .. } = &outcome else {
                panic!("expected Recovered for {content}, got {outcome:?}");
            };
            assert!(outcome.allows_saving());
            assert!(!store.path_of::<TestDocument>().exists());
            assert_eq!(fs::read_to_string(preserved_as)?, content);
        }
        Ok(())
    }
}
