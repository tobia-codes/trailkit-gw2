# trailkit-memo-search

Search slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo plan
M118–M129). Depends only on [`trailkit-memo-shared`](../../shared/README.md),
never on another slice; the app wires it into the note list, which reads
the results through the shared `ListFilter` trait.

| Module | Content |
|--------|---------|
| `note_search` | search index over titles and content, the rows a search shows, match counts per group and content hits per note (ImGui-free) |
| `no_search_results` | the empty search result with its clear button, drawn by the app instead of the list (M126, Windows only) |
| `search_field` | the search field above the note list: magnifier, hint, clear `x`, brass focus frame (Windows only) |
| `i18n` | the slice's texts (`SearchText`, en/de) |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). The search
logic and texts are tested on any host with
`cargo test -p trailkit-memo-search`; the field is covered by the app's
window snapshot tests (`cargo xtask ui-test`).
