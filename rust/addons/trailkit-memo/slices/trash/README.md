# trailkit-memo-trash

Trash slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M12, M133–M151): the trash column shown from the window menu (M166)
— header with "Empty", one two-line row per deleted note or group (newest
first, "N days left" in `danger` during the last week), the context menu
"Restore" / "Delete permanently …" and the empty state — plus the note
pane's toolbar for a trashed entry and the preview of a trashed group.
Depends only on [`trailkit-memo-shared`](../../shared/README.md), never on
another slice. The trash operations (`NotesDocument::move_to_trash`,
`restore_from_trash`, …) and the undo history (`undo`) live in shared,
because the app applies them to the note tree.

| Module | Content |
|--------|---------|
| `trash_view` | `draw_trash_header`, `draw_trash_entries`, `draw_preview_toolbar`, `draw_group_preview`, `TrashAction` (Windows only) |
| `labels` | formatted texts: days left, info line, dialog questions, toast message |
| `i18n` | the slice's texts (`TrashText`, en/de), including the delete dialogs and the undo toast |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). Texts and
labels are tested on any host with `cargo test -p trailkit-memo-trash`; the
views are covered by the app's window snapshot tests (`cargo xtask ui-test`).
