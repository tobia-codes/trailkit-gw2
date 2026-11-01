# trailkit-storage

Versioned, crash-safe JSON persistence for all trailkit Nexus addons
(ROADMAP R22, memo plan M29–M33). Every file an addon writes goes through
this crate; addons never hand-roll file I/O.

## What it does

| Concern | Behavior |
|---------|----------|
| Format | Pretty-printed JSON, `schema_version` as the first key |
| Saving | Atomic: write `<file>.tmp`, flush, rename over the live file |
| Older version | Migrated step by step (`VersionedDocument::migrate_one_step`); the original is copied to `backups/<stem>.v<N>.bak.json` first |
| Newer version | `LoadOutcome::ReadOnly` — the file is never overwritten |
| Unreadable file | Moved to `backups/<stem>.corrupt-<time>.json`, caller starts fresh (`LoadOutcome::Recovered`) |
| Unknown fields | Ignored (new fields always bump `schema_version`) |
| Cleanup | Only the newest migration copy per file, deleted after 30 days; at most 3 copies of unreadable files |

The crate is platform-independent and does no threading: run it on the
addon's worker thread, never on the render thread (nexus-addon-dev skill §2).

## Usage

```rust
use serde::{Deserialize, Serialize};
use trailkit_storage::{DocumentStore, LoadOutcome, SchemaVersion, VersionedDocument};

#[derive(Default, Serialize, Deserialize)]
struct Notes {
    titles: Vec<String>,
}

impl VersionedDocument for Notes {
    const FILE_STEM: &'static str = "notes"; // → notes.json
    const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::FIRST;
}

fn example(addon_directory: std::path::PathBuf) -> Result<(), trailkit_storage::StorageError> {
    let store = DocumentStore::new(addon_directory);
    let outcome = store.load::<Notes>(jiff::Timestamp::now());
    let may_save = outcome.allows_saving();
    let notes = outcome.into_document().unwrap_or_default();
    if may_save {
        store.save(&notes)?;
    }
    Ok(())
}
```

When a format changes: bump `CURRENT_SCHEMA_VERSION` and implement
`migrate_one_step` for the previous version.

## Test

```sh
cargo test -p trailkit-storage
```
