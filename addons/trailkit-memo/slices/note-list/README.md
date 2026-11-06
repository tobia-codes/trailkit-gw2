# trailkit-memo-note-list

Note-list slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M43–M50, M96–M117). Depends only on
[`trailkit-memo-shared`](../../shared/README.md), never on another slice.

| Module | Content |
|--------|---------|
| `note_list` | `NoteList`: the "New note / New group" header (M128), tree rows with titles, counts, badges and search hits, inline rename (M44), context menus with "Move to…" (M112), drag & drop with drop line, auto-scroll and drag label (M106–M117) (Windows only) |
| `tree_drag` | drop zones: turns the mouse position over the recorded rows into an insertion point and its indicator (ImGui-free) |
| `i18n` | the slice's texts (`NoteListText`, en/de) |

## How the app wires it

- The app draws the menu button and the search field, then
  `NoteList::draw_header`, then `NoteList::draw` inside the scrolling child
  window, and calls `NoteList::finish_drag` once per frame after the whole
  window (the list may not be drawn while it is hidden).
- A search comes in as `ListSearch` (query, a `&dyn ListFilter` — the search
  slice's results — the groups collapsed for this search and its texts).
- Other slices' row extras come in as closures (`NoteRowExtras`): the reset
  badge, its tooltip and the "Reset type" submenu.
- The list reports `NoteListAction`s; the app maps them to its own actions
  and changes the document. `start_rename` and `cancel_drag` let the app
  rename a new entry and cancel a drag on Escape.

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). The drop
zones and texts are tested on any host with
`cargo test -p trailkit-memo-note-list`; the list is covered by the app's
window snapshot and interaction tests (`cargo xtask ui-test`).
