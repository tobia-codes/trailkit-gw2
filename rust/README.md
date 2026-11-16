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
| `services/` | [`trailkit-api`](services/trailkit-api/README.md) | REST API server, skeleton only (backend deferred) |
| `tools/` | [`trailkit-datagen`](tools/trailkit-datagen/README.md) | Daily data job: trading post price archive and WvW snapshots |
| `xtask/` | [`xtask`](xtask/README.md) | Build and check commands: `cargo xtask <command>` |

Workspace files: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`clippy.toml`, `deny.toml` and `.cargo/config.toml` (the `cargo xtask` alias).
Build output goes to `target/` in this folder.

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
