# trailkit-memo-drafts

Drafts slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M54, M55): the hint above a note with a kept draft — when it was saved,
whether the note changed since, "Restore" / "Discard". Depends only on
[`trailkit-memo-shared`](../../shared/README.md), never on another slice.
The drafts document (`drafts.json`) stays in shared: the storage worker
writes it and the editor creates drafts.

| Module | Content |
|--------|---------|
| `draft_hint` | `DraftHint` (caches the formatted time) and `DraftHintAction` (Windows only) |
| `draft_overview` | the card above the panes about notes with unsaved changes: count, marked rows, "Go to note"; hidden until the next game start when closed (M185, Windows only) |
| `i18n` | the slice's texts (`DraftText`, en/de); "Discard" stays in shared, the unsaved-changes question uses it too |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-memo-shared`](../../shared/README.md) | own | Note tree, settings, drafts, undo, storage worker and texts shared by memo. | Uses memo's note types, texts and settings. |
| [`trailkit-ui`](../../../crates/trailkit-ui/README.md) | own, Windows only | Theme, fonts and shared ImGui components. | Uses the shared theme, text and controls. |
| `jiff` | external, Windows only | Date, time and time-zone types. | Shows when a draft was saved. |
| `nexus` | external, Windows only | Rust bindings for the Raidcore Nexus addon API and its ImGui. | Draws its interface with ImGui. |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Texts are
tested on any host with `cargo test -p trailkit-memo-drafts`; the hint is
covered by the app's window snapshot tests (`cargo xtask ui-test`).
