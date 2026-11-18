# trailkit-memo-note-view

Note-view slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M41, M49, M81, M122): the selected note's title, a separator and its
rendered markdown in a scroll region below, so the title stays in place.
Search hits are marked; a new search or a newly opened note jumps to the
first one. Depends only on [`trailkit-memo-shared`](../../shared/README.md),
never on another slice: the app hands in the kind dropdown as a neutral
`TitleTrailing` of `trailkit-ui`'s page head (size + draw closure) and draws the draft hint above.

| Module | Content |
|--------|---------|
| `note_view` | `NoteView` (markdown cache, scroll restore, jump to first hit), `NoteViewInput`, `NoteViewOutput` (actions + "scroll changed"), `NoteViewAction` (Windows only) |
| `i18n` | the slice's texts (`NoteViewText`, en/de) |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-memo-shared`](../../shared/README.md) | own | Note tree, settings, drafts, undo, storage worker and texts shared by memo. | Uses memo's note types, texts and settings. |
| [`trailkit-ui`](../../../crates/trailkit-ui/README.md) | own, Windows only | Theme, fonts and shared ImGui components. | Provides the Markdown view and page layout. |
| `nexus` | external, Windows only | Rust bindings for the Raidcore Nexus addon API and its ImGui. | Draws its interface with ImGui. |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Texts are
tested on any host with `cargo test -p trailkit-memo-note-view`; the view is
covered by the app's window snapshot tests (`cargo xtask ui-test`).
