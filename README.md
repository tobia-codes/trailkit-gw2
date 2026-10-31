# trailkit-gw2

Monorepo for Guild Wars 2 tooling: [Raidcore Nexus](https://raidcore.gg/Nexus)
addons and a Rust data pipeline (static data files + daily price archive;
a Rust backend server is planned for later).

> Status: phase 0 (foundation). See [`docs/plans/`](docs/plans/README.md).

## Layout

| Path | Content |
|------|---------|
| `addons/` | Nexus addons (`trailkit-memo`: phase 0 skeleton) |
| `crates/trailkit-ui/` | Shared design system: theme tokens, ImGui style scope |
| `.githooks/` | Versioned git hooks (`cargo xtask install-hooks`) |
| `xtask/` | Task runner: `cargo xtask deploy`, `cargo xtask sync-versions` |
| `data/events/` | Curated event schedules |
| `.changeset/` | Changesets (versioning, ROADMAP R16) |
| `docs/plans/` | Master roadmap and one plan per addon / component |
| `.agents/skills/` | Agent skills shared by Claude Code and Codex |
| `AGENTS.md` | Rules for coding agents (`CLAUDE.md` imports it) |

## Setup (WSL)

1. Rust: [rustup](https://rustup.rs) — the toolchain is pinned by `rust-toolchain.toml`.
2. `cargo install --locked cargo-xwin cargo-deny`
3. LLVM for cross-compiling: `sudo apt install clang lld llvm`
4. Node via fnm (`.node-version`) and pnpm (`packageManager` in `package.json`), then `pnpm install`.
5. Configure the GW2 addons folder for deploys (see [`xtask/README.md`](xtask/README.md)).
6. Enable the git hooks: `cargo xtask install-hooks`.

Checks before a PR: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
`cargo xwin clippy --target x86_64-pc-windows-msvc -- -D warnings`, `cargo test`, `cargo deny check`.

## Contributing rules (short)

- Naming: readable names over abbreviations; Rust per the API guidelines
  (`snake_case` files/functions, `UpperCamelCase` types); all other files and
  directories `kebab-case`; data keys `snake_case`; Nexus identifiers
  `trailkit.<addon>.<kind>.<name>`. Full table:
  [AGENTS.md → Naming conventions](AGENTS.md#naming-conventions)
  (checked by `cargo xtask check-names`).
- Every directory that is a project has a `README.md`.
- Every function is documented (rustdoc `///`).
- Comments and docs are written in English.
- Decisions are made via the `grill-me` skill and recorded in `docs/plans/`.

## License

Code: [MIT](LICENSE). Event schedule data in `data/events/`:
[CC BY-NC-SA 2.5](data/events/LICENSE) (derived from the Guild Wars 2 Wiki).
