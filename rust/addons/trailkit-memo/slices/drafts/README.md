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
| `i18n` | the slice's texts (`DraftText`, en/de); "Discard" stays in shared, the unsaved-changes question uses it too |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Texts are
tested on any host with `cargo test -p trailkit-memo-drafts`; the hint is
covered by the app's window snapshot tests (`cargo xtask ui-test`).
