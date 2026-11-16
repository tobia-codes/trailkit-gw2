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
| `editor` | `Editor` (title, body, preview cache, Ctrl+S/B/I), `draw_toolbar` → `ToolbarAction`, `draw` → `EditorOutput` (actions, new split, splitter released) (Windows only) |
| `i18n` | the slice's texts (`EditorText`, en/de) |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Edit
session, formatting and texts are tested on any host with
`cargo test -p trailkit-memo-editor`; the drawing is covered by the app's
window snapshot tests (`cargo xtask ui-test`).
