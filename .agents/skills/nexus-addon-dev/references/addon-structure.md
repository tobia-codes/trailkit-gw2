# Addon structure blueprint

How a trailkit addon is laid out and wired (ROADMAP R76, questions 22–58;
reference implementation: `addons/trailkit-memo`). Follow it for every new
addon so all addons look the same. The rules behind it are in the skill
(§0); this page is the practical template.

## 1. Directory layout

```
addons/trailkit-<addon>/
├── Cargo.toml            app crate: builds the DLL (crate-type cdylib + rlib)
├── README.md             what the addon is, how to build and use it, layout table
├── package.json          version placeholder for Changesets (R16)
├── help/{en,de}.md       in-game help page (AGENTS.md rule 6)
├── assets/icons/         quick-access icon source (.svg; .png generated)
├── src/
│   ├── lib.rs            crate root; `all_texts()` = app + every slice's texts
│   ├── addon.rs          Nexus export, load/unload, render (#[cfg(windows)])
│   ├── main_window.rs    window state, `new`, `draw`, close/unload handling
│   └── main_window/      one file per concern, e.g.
│       ├── actions.rs    `WindowAction` + `apply` dispatcher + per-area apply fns
│       ├── layout.rs     panes, drawer, empty state
│       ├── <slice>.rs    wiring of one slice (inputs, closures, action mapping)
│       ├── dialogs.rs    dialog card and window
│       └── tests.rs      snapshot and interaction tests of the window
├── tests/snapshots/      UI snapshot PNGs (`cargo xtask ui-test`)
├── shared/               crate `trailkit-<addon>-shared`
│   ├── Cargo.toml
│   ├── README.md
│   └── src/              data model, persisted documents, storage worker,
│                         i18n (`Language`, `LocalizedText`, shared texts),
│                         neutral types slices exchange via the app
└── slices/
    └── <slice>/          crate `trailkit-<addon>-<slice>`, one per feature
        ├── Cargo.toml
        ├── README.md     modules table + "how the app wires it"
        └── src/
            ├── lib.rs    `pub mod i18n;` + ImGui-free modules + `#[cfg(windows)] pub mod <ui>;`
            ├── i18n.rs   `<Slice>Text` enum implementing `LocalizedText`
            ├── <logic>.rs   ImGui-free logic, unit-tested on Linux
            └── <ui>.rs   drawing (Windows only), sub-modules in `<ui>/`
```

Register every new crate in the root `Cargo.toml` (`[workspace.dependencies]`
path entry; members already glob `addons/*/shared` and `addons/*/slices/*`).

## 2. What goes where

| Code | Place |
|------|-------|
| One user-facing feature (search, editor, note list, …) | its own slice |
| Needed by two slices of the same addon | `shared` |
| Needed by two addons | `crates/` (`trailkit-ui` for UI, `trailkit-model` for domain types, R75) |
| Wiring slices together, persisting, the window frame | the app |
| ImGui-free logic (parsing, geometry, search, schedules) | non-`cfg(windows)` modules, so `cargo test` runs on Linux |
| Drawing | `#[cfg(windows)]` modules; nexus/trailkit-ui under `[target.'cfg(windows)'.dependencies]` |

Extract only on real reuse (AGENTS.md rule 7): a helper moves to `shared`
or `crates/` when the second place needs it, and every copy is replaced in
the same change.

## 3. Slice contract

- **Never depends on another slice** (Cargo enforces it). When slice A
  needs something of slice B, the app hands it in as a neutral type from
  `shared`:
  - a trait in `shared` that B's type implements (e.g. `ListFilter`,
    implemented by the search results, read by the note list);
  - a plain data type (e.g. `RowBadge`, `SearchQuery` from trailkit-ui);
  - closures for things A draws on B's behalf (e.g. `NoteRowExtras`:
    badge, tooltip, context-menu items; `TitleTrailing` in note-view).
- **State**: a struct owned by the slice (`Default` when possible), held by
  the app as a field; small methods for what the app triggers
  (`start_rename`, `cancel_drag`, `request_focus`).
- **Drawing**: `draw(&mut self, ui, language, theme, input) -> Output`;
  `input` is a struct of borrows (`XyzInput<'a>`), the output an enum
  (`XyzAction`) or a small struct with `actions: Vec<XyzAction>` plus
  flags. The app maps actions to its `WindowAction` (`impl From<XyzAction>`).
- **Persistence**: never touches the storage worker; persisted values come
  in as `&mut` single fields, the slice reports changes, the app saves.
- **Texts**: one enum implementing `LocalizedText` (`ALL`, `get`), test
  `assert_all_translated::<T>()`, added to the app's `all_texts()`. Texts
  the app also needs (e.g. default titles) stay in `shared`.
- **Docs**: crate docs name the slice, its plan decisions and what it
  depends on; README with a modules table and how the app wires it; a
  section of the help page.

## 4. The app

- `main_window.rs` keeps the window state and `draw`; everything else lives
  in `main_window/<concern>.rs` as `impl MemoWindow` blocks.
- `apply` is a dispatcher: guards first (e.g. unsaved edits), then an
  exhaustive list of variants that hands each action to one
  `apply_<area>_action` — the compiler reports a new variant that is not
  routed.
- Window tests import the types they use explicitly; they do not rely on
  the window's own imports.

## 5. Size limits

Files and functions stay small enough to read in one go; see the file
length check (`cargo xtask check-file-length`, once settled) and split by
concern as in §1 when a file grows past it.

## 6. Checklist for a new addon

1. Create the app crate, `shared` and the first slice from the layout
   above; add the crates to the root `Cargo.toml`.
2. README in every crate, help page `help/{en,de}.md`, `package.json`.
3. Nexus identifiers as typed constants `trailkit.<addon>.<kind>.<name>`.
4. `all_texts()` collects every text enum; fonts are registered from it.
5. `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
   `cargo xwin clippy` (Windows), `cargo test`, `cargo xtask ui-test`,
   `cargo xtask check-names`.
