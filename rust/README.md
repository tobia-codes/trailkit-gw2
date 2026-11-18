# rust

The Cargo workspace of trailkit-gw2: the Guild Wars 2 addons, their shared
crates, the services, the data tools and the `xtask` task runner.

## Status

Layout per [ROADMAP R116](../docs/plans/ROADMAP.md); every Rust project lives
below this folder, the repository root keeps only language-neutral files.

## Contents

| Folder | Project | What it is |
|--------|---------|------------|
| `addons/` | [`trailkit-memo`](addons/trailkit-memo/README.md) | Notes addon; its README lists the slices it is built from |
| `addons/crates/` | [`trailkit-ui`](addons/crates/trailkit-ui/README.md) | Design system: theme tokens, fonts, icons and shared ImGui components |
| | [`trailkit-ui-testing`](addons/crates/trailkit-ui-testing/README.md) | Headless UI tests and snapshot comparison (dev-dependency only) |
| | [`trailkit-storage`](addons/crates/trailkit-storage/README.md) | Versioned, crash-safe JSON files for everything an addon saves |
| | [`trailkit-log`](addons/crates/trailkit-log/README.md) | One local log file per addon and the binding log format |
| | [`trailkit-nexus-bridge`](addons/crates/trailkit-nexus-bridge/README.md) | The Nexus API helpers all addons share (fonts); the only shared crate that calls Nexus |
| `services/` | [`trailkit-api`](services/trailkit-api/README.md) | REST API server, skeleton only (backend deferred) |
| `tools/` | [`trailkit-datagen`](tools/trailkit-datagen/README.md) | Daily data job: trading post price archive and WvW snapshots |
| `xtask/` | [`xtask`](xtask/README.md) | Build and check commands: `cargo xtask <command>` |

**Folder rule (R137):** a folder holds at most 10 loose `.rs` files (files
without a folder of the same name; `lib.rs`, `main.rs` and `tests.rs` do not
count); with more they are grouped into folders by concern. Generated Rust
code lives in a `generated` module (R135). `cargo xtask check-file-length`
hints at folders over the limit. Binding details:
[naming conventions](../docs/agent-reference.md#naming-conventions).

Workspace files: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`clippy.toml`, `deny.toml` and `.cargo/config.toml` (the `cargo xtask` alias).
Build output goes to `target/` in this folder.

## Packages

Every package of this workspace. Each project README lists the own and external
packages that project uses directly.

| Package | What it does | Why it exists |
|---------|--------------|---------------|
| [`trailkit-api`](services/trailkit-api/README.md) | Skeleton of the REST API server. | Reserves the place for the deferred backend. |
| [`trailkit-datagen`](tools/trailkit-datagen/README.md) | Daily GW2 data job. | Builds the public GW2 data archive. |
| [`trailkit-log`](addons/crates/trailkit-log/README.md) | Local log files and the log export of the addons. | Gives every addon the same local diagnostics. |
| [`trailkit-memo`](addons/trailkit-memo/README.md) | Notes addon for Guild Wars 2. | The notes addon users install. |
| [`trailkit-memo-drafts`](addons/trailkit-memo/slices/drafts/README.md) | Draft card and draft overview. | Keeps unsaved text recoverable. |
| [`trailkit-memo-editor`](addons/trailkit-memo/slices/editor/README.md) | Edit session, Markdown formatting and editor view. | Separates editing from the rest of memo. |
| [`trailkit-memo-note-list`](addons/trailkit-memo/slices/note-list/README.md) | Note tree rows, rename, menus and drag and drop. | Separates list behavior from the rest of memo. |
| [`trailkit-memo-note-view`](addons/trailkit-memo/slices/note-view/README.md) | Reading view of the selected note. | Separates reading from the rest of memo. |
| [`trailkit-memo-reset-schedule`](addons/trailkit-memo/slices/reset-schedule/README.md) | Daily and weekly reset rules and their controls. | Separates reset rules from the rest of memo. |
| [`trailkit-memo-search`](addons/trailkit-memo/slices/search/README.md) | Note search, search field and find bar. | Separates search from the rest of memo. |
| [`trailkit-memo-shared`](addons/trailkit-memo/shared/README.md) | Note tree, settings, drafts, undo, storage worker and texts shared by memo. | Lets memo slices share data without depending on each other. |
| [`trailkit-memo-trash`](addons/trailkit-memo/slices/trash/README.md) | Trash list, preview and labels. | Separates deletion and recovery from the rest of memo. |
| [`trailkit-nexus-bridge`](addons/crates/trailkit-nexus-bridge/README.md) | Nexus helpers shared by the addons, such as font registration. | Keeps shared Nexus code out of each addon. |
| [`trailkit-storage`](addons/crates/trailkit-storage/README.md) | Versioned, crash-safe JSON files. | Protects addon files from corruption. |
| [`trailkit-ui`](addons/crates/trailkit-ui/README.md) | Theme, fonts and shared ImGui components. | Keeps every addon visually consistent. |
| [`trailkit-ui-testing`](addons/crates/trailkit-ui-testing/README.md) | Headless ImGui, simulated input and snapshot comparison. | Tests addon UI without starting the game. |
| [`xtask`](xtask/README.md) | Build, check and generator commands. | Keeps repository tasks in versioned Rust code. |

## Build and test

Run every `cargo` command from this folder:

```sh
cd rust
cargo test                         # unit tests, translations, generated docs, README links
cargo xtask ui-test                # UI snapshot tests (Windows binaries via WSL)
cargo xtask deploy trailkit-memo   # build the DLL and hot-reload it into the running game
```

`xtask.local.toml` stays in the repository root (git-ignored).

## Rules

A crate lives where it is used (R116): used by one addon → inside that addon;
by several addons → `addons/crates/`; by several services → `services/crates/`;
by addons and services or tools → `crates/`, moved in the same change as the
second use. General rules: [`CONTRIBUTING.md`](../CONTRIBUTING.md).
