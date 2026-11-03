# trailkit-memo-shared

What the slices of [trailkit-memo](../README.md) share (ROADMAP R76). Slices
depend on this crate, never on each other.

| Module | Content |
|--------|---------|
| `note_tree` | note tree data model (notes, groups, trash), persisted as `notes.json` |
| `reset_schedule` | reset periods: daily (00:00 UTC) and weekly (Monday 07:30 UTC) reset times (M13, M90) |
| `drafts` | unsaved edits kept for restoring (`drafts.json`) |
| `settings` | typed user settings + visibility rules, persisted as `settings.json` |
| `ui_state` | restorable window state, persisted as `ui_state.json` (M39) |
| `storage_worker` | worker thread that loads and saves all documents debounced (M18, M30) |
| `i18n` | languages + typed UI strings (en/de), local time formatting |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Its logic is
platform independent and tested on any host with
`cargo test -p trailkit-memo-shared`; local time uses the Windows API only on
Windows (UTC elsewhere).
