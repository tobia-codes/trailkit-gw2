# trailpack-gw2

Monorepo for Guild Wars 2 tooling: [Raidcore Nexus](https://raidcore.gg/Nexus)
addons and a Rust data pipeline (static data files + daily price archive;
a Rust backend server is planned for later).

> Status: planning phase. See [`docs/plans/`](docs/plans/README.md).

## Layout

| Path | Content |
|------|---------|
| `docs/plans/` | Master roadmap and one plan per addon / component |
| `.agents/skills/` | Agent skills shared by Claude Code and Codex |
| `AGENTS.md` | Rules for coding agents (`CLAUDE.md` imports it) |

## Contributing rules (short)

- Every directory that is a project has a `README.md`.
- Every function is documented (rustdoc `///`).
- Comments and docs are written in English.
- Decisions are made via the `grill-me` skill and recorded in `docs/plans/`.

## License

Code: [MIT](LICENSE). Event schedule data in `data/events/`:
[CC BY-NC-SA 2.5](data/events/LICENSE) (derived from the Guild Wars 2 Wiki).
