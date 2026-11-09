# trailkit-gw2

Open-source tools for Guild Wars 2: [Raidcore Nexus](https://raidcore.gg/Nexus)
addons written in Rust, the shared libraries behind them, and a data pipeline
that archives the trading post prices every day. A REST backend is planned.

> **Status:** the first addon, `trailkit-memo` (notes), is in development and
> not released yet. The daily price archive runs. Roadmap:
> [`docs/plans/ROADMAP.md`](docs/plans/ROADMAP.md).

## Contents

- [Find your way](#find-your-way)
- [I want to …](#i-want-to-)
- [Setup (WSL)](#setup-wsl)
- [Build, test and deploy](#build-test-and-deploy)
- [Contributing](#contributing)
- [License](#license)

## Find your way

Every project has its own README with purpose, build, usage and its rules.

| Area | Project | What it is |
|------|---------|------------|
| Addons | [`addons/trailkit-memo`](addons/trailkit-memo/README.md) | Notes addon; its README lists the slices it is built from |
| Shared crates | [`crates/trailkit-ui`](crates/trailkit-ui/README.md) | Design system: theme tokens, fonts, icons and shared ImGui components |
| | [`crates/trailkit-ui-testing`](crates/trailkit-ui-testing/README.md) | Headless UI tests and snapshot comparison (dev-dependency only) |
| | [`crates/trailkit-storage`](crates/trailkit-storage/README.md) | Versioned, crash-safe JSON files for everything an addon saves |
| | [`crates/trailkit-log`](crates/trailkit-log/README.md) | One local log file per addon and the binding log format |
| Services | [`services/trailkit-api`](services/trailkit-api/README.md) | REST API server, skeleton only (backend deferred) |
| Tools | [`tools/trailkit-datagen`](tools/trailkit-datagen/README.md) | Daily data job: trading post price archive and WvW snapshots |
| | [`tools/bruno-gw2-api`](tools/bruno-gw2-api/README.md) | Bruno collection for trying the GW2 API v2 by hand (public and account endpoints) |
| | [`xtask`](xtask/README.md) | Build and check commands: `cargo xtask <command>` |
| Data | [`data/events`](data/events/README.md) | Curated event schedules (source of truth) |
| Docs | [`docs/plans`](docs/plans/README.md) | Roadmap, plans and every decision with its id |
| | [`.agents/skills`](.agents/skills/README.md) | Detailed rules for addon code, UI and Rust (read by humans and coding agents) |
| | [`.changeset`](.changeset/README.md) | Versioning and changelogs with Changesets |

Rules for contributors: [`CONTRIBUTING.md`](CONTRIBUTING.md). Security reports:
[`SECURITY.md`](SECURITY.md). Coding agents: [`AGENTS.md`](AGENTS.md).

## I want to …

| … | Start here |
|---|-----------|
| report a bug or suggest an idea | [Open an issue](https://github.com/tobia-codes/trailkit-gw2/issues) |
| fix a bug in memo | [memo README](addons/trailkit-memo/README.md) → the slice's README → its tests |
| change something visible in an addon | [CONTRIBUTING § UI rules](CONTRIBUTING.md#6-ui-rules) (visual proposal first) → [`trailkit-ui` skill](.agents/skills/trailkit-ui/SKILL.md) |
| add or change a design token | [`tokens.md`](.agents/skills/trailkit-ui/references/tokens.md) → `cargo xtask tokens` |
| build a new addon | [addon blueprint](.agents/skills/nexus-addon-dev/references/addon-structure.md) → [Nexus rules](.agents/skills/nexus-addon-dev/SKILL.md) |
| fix or add event times | [`data/events`](data/events/README.md) |
| use the price archive | [`tools/trailkit-datagen`](tools/trailkit-datagen/README.md) (monthly GitHub releases `archive-YYYY-MM`) |
| understand why something is the way it is | [`docs/plans`](docs/plans/README.md), search the decision id (e.g. `R76`) |

## Setup (WSL)

Development happens in WSL (Linux); addons are cross-compiled to Windows.

1. Rust via [rustup](https://rustup.rs); the toolchain is pinned by `rust-toolchain.toml`.
2. `cargo install --locked cargo-xwin cargo-deny`
3. LLVM for cross-compiling: `sudo apt install clang lld llvm`
4. Node via [fnm](https://github.com/Schniz/fnm) (`.node-version`) and pnpm
   (`packageManager` in `package.json`), then `pnpm install`.
5. Point deploys at the game's addons folder
   ([xtask README](xtask/README.md#configuring-the-gw2-addons-folder)).
6. Enable the git hooks: `cargo xtask install-hooks`.

## Build, test and deploy

```sh
cargo build                        # everything for Linux (addons need the Windows target)
cargo test                         # unit tests, translations, generated docs, README links
cargo xtask ui-test                # UI snapshot tests (Windows binaries via WSL)
cargo xtask deploy trailkit-memo   # build the DLL and hot-reload it into the running game
```

The full list of checks before a pull request is in
[CONTRIBUTING § Tests and checks](CONTRIBUTING.md#8-tests-and-checks); every
`xtask` command is in [`xtask/README.md`](xtask/README.md).

## Contributing

Issues are welcome. For a pull request, agree on the change in an issue first.
[`CONTRIBUTING.md`](CONTRIBUTING.md) explains how the code is organised and the
rules it follows, and why.

## License

Code: [MIT](LICENSE). Event schedule data in `data/events/`:
[CC BY-NC-SA 2.5](data/events/LICENSE) (derived from the Guild Wars 2 Wiki).
