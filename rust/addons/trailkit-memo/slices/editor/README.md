# trailkit-memo-editor

Editor slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M51–M54, M58, M60, M69, M72, M93, M94, M100, M102): the edit session that
holds an uncommitted copy of title and body, the markdown formatting
commands, and the editor that draws the formatting toolbar, title input,
word-wrapping body and the preview beside it. Depends only on
[`trailkit-memo-shared`](../../shared/README.md), never on another slice:
the app draws the toolbar's end (Edit / Cancel / Save) and the
unsaved-changes dialog, hands in the preview layout, search and chip state
(`EditorInput`) and turns the editor's actions into its own.

| Module | Content |
|--------|---------|
| `note_editing` | `EditSession` (start, from draft, changes, to draft), `FormatCommand`, `format_edit`, `ActiveFormats` — platform-independent |
| `link_edit` | `link_draft`, `normalized_address`, `link_edit`: what the link dialog starts with, accepts and writes (M208) — platform-independent |
| `link_dialog` | The link dialog with link text and address, a shared `Dialog` over the editor's window (Windows only, private) |
| `editor` | `Editor` (title, body, preview cache, link dialog, Ctrl+S/B/I/K), `draw_toolbar` → `ToolbarAction`, `draw` → `EditorOutput` (actions, new split, splitter released) (Windows only) |
| `i18n` | the slice's texts (`EditorText`, en/de) |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-memo-shared`](../../shared/README.md) | own | Note tree, settings, drafts, undo, storage worker and texts shared by memo. | Uses memo's note types, texts and settings. |
| [`trailkit-ui`](../../../crates/trailkit-ui/README.md) | own | Theme, fonts and shared ImGui components. | Provides the text editor, Markdown view and controls. |
| `jiff` | external | Date, time and time-zone types. | Stamps saved notes and drafts. |
| `nexus` | external, Windows only | Rust bindings for the Raidcore Nexus addon API and its ImGui. | Draws its interface with ImGui. |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Edit
session, formatting and texts are tested on any host with
`cargo test -p trailkit-memo-editor`; the drawing is covered by the app's
window snapshot tests (`cargo xtask ui-test`).
