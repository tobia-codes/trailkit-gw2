# Agent skills

Shared skills for Claude Code and Codex. This directory is the single source of
truth; `.claude/skills` is a symlink to it.

| Skill | Source | License |
|-------|--------|---------|
| `grill-me` | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/productivity/grill-me) | MIT |
| `grilling` | [mattpocock/skills](https://github.com/mattpocock/skills/tree/main/skills/productivity/grilling) | MIT |
| `rust-best-practices` | [apollographql/skills](https://github.com/apollographql/skills/tree/main/skills/rust-best-practices) | MIT |
| `rust-skills` | [leonardomso/rust-skills](https://github.com/leonardomso/rust-skills) (SKILL.md + rules only) | MIT |

Vendored on 2026-10-31. To update, re-copy from upstream and keep the `LICENSE`
file in each skill directory.

## Project skills (written for this repo)

| Skill | Content |
|-------|---------|
| `nexus-addon-dev` | Nexus API facts, threading & performance research, inter-addon protocol, i18n, API key handling |
| `trailkit-ui` | Design system, themes, shared components, full UX rule set for ImGui overlays (`references/ux-rules.md`) |

## Inspiration (not vendored)

| Source | Used for | License |
|--------|----------|---------|
| [nextlevelbuilder/ui-ux-pro-max-skill](https://github.com/nextlevelbuilder/ui-ux-pro-max-skill) | Rules adapted into `trailkit-ui/references/ux-rules.md` (removed as a skill on 2026-10-31: web/mobile-centric, risk of agents applying non-overlay rules) | MIT |
