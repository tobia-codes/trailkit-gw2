# trailkit-memo-shared

What the slices of [trailkit-memo](../README.md) share (ROADMAP R76). Slices
depend on this crate, never on each other.

| Module | Content |
|--------|---------|
| `note_tree` | note tree data model (notes, groups, trash, reset periods), persisted as `notes.json` |
| `list_filter` | `ListFilter` trait: what the note list reads from an active filter (the search results), so the two slices stay apart |
| `undo` | undo and redo of deletes and moves, 20 steps, in memory only (M135) |
| `row_badge` | neutral row badge (letter + color) that a slice describes and the note list draws |
| `drafts` | unsaved edits kept for restoring (`drafts.json`) |
| `settings` | typed user settings + visibility rules, persisted as `settings.json` |
| `ui_state` | restorable window state, persisted as `ui_state.json` (M39) |
| `save_text` | `SaveText`: the en/de strings of save errors and the attempt counter (M180) |
| `storage_worker` | worker thread that loads and saves all documents debounced (M18, M30), retries failed writes (3 times, 5 s apart) and reports to the UI (M180); `storage_worker/loading.rs` loads the documents and logs it, `storage_worker/pending_save.rs` the retry cycle; the only code that touches the files |
| `i18n` | languages, the texts no slice owns yet (en/de), local time and percentage formatting |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-log`](../../crates/trailkit-log/README.md) | own | Local log files and the log export of the addons. | Writes error chains into storage log events. |
| [`trailkit-storage`](../../crates/trailkit-storage/README.md) | own | Versioned, crash-safe JSON files. | Loads and saves memo's documents. |
| [`trailkit-ui`](../../crates/trailkit-ui/README.md) | own | Theme, fonts and shared ImGui components. | Provides icons and chat-link text types. |
| `jiff` | external | Date, time and time-zone types. | Stores note, draft and trash times. |
| `log` | external | Common logging interface for Rust crates. | Writes storage and load events. |
| `serde` | external | Framework for serializing and deserializing Rust types. | Defines memo's stored documents. |
| `serde_json` | external | JSON support for `serde`. | Migrates `notes.json` between schema versions; checks stored JSON in tests. |
| `thiserror` | external | Derives typed error enums. | Gives storage worker failures typed errors. |
| `uuid` | external | Generates and stores UUIDs. | Creates stable note and group ids. |
| `windows` | external, Windows only | Official Rust bindings for Windows APIs. | Converts UTC times to the local time shown to users. |
| `tempfile` | external, tests only | Creates temporary files and folders. | Gives storage tests isolated folders. |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Its logic is
platform independent and tested on any host with
`cargo test -p trailkit-memo-shared`; local time uses the Windows API only on
Windows (UTC elsewhere).

## Rules

- Something moves here only when a second slice needs it (rule 7); a type
  only one slice uses stays in that slice.
- No drawing that belongs to one feature: shared holds data, documents,
  traits and texts no slice owns.
