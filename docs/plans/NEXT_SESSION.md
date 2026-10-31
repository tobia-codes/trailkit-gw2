# Next session — handoff

Last updated: 2026-10-31 (session 3).

## Where we are

- Planning complete (ROADMAP R1–R39, data-pipeline D1–D26, core C1–C13,
  memo M1–M15, event E1–E15). **Phase 0 (foundation) in progress** on branch
  `chore/phase-0-foundation` (cut from `develop`; merge after user approval, R29).
- Phase 0 grill round settled → R28–R39 in `ROADMAP.md`.
- Never push without approval. The user converses in German; repo text English.

## Phase 0 status

Done:
- rustup + pinned toolchain 1.99.1, cargo-xwin, cargo-deny installed.
- Workspace (`Cargo.toml`, lints, `clippy.toml`, `deny.toml`), `xtask`
  (`deploy`, `sync-versions`), `addons/trailkit-memo` skeleton, Changesets
  structure (pnpm), CI workflow (`.github/workflows/ci.yml`, inactive until 0b).
- Host checks green: fmt, clippy, tests, `cargo deny check`.

Optional for the user:
- Add `. "$HOME/.cargo/env"` to `~/.bashrc` (rustup was installed with
  `--no-modify-path`).

Settled since: R47 trailkit-ui crate (theme + StyleScope, done), R48 git
hooks (done), R49 update provider `Direct`, R50 kebab-case DLL names,
R51 gold palette, R52 domain `gw2.trailkit.de`; naming conventions (R40–R46, AGENTS.md "Naming conventions", `cargo xtask check-names`).

Settled in session 3: grill rounds 5–6 → R53–R61 (options tab, rarity
tokens, quick access icons + `cargo xtask build-icons`, gameplay auto-hide).

Open phase 0 work:
1. ~~Windows build, deploy, in-game test, hot reload~~ — done 2026-10-31.
2. Memo quick access icon (R55/R57–R59) and gameplay auto-hide (R56/R61) —
   implemented and deployed 2026-10-31; **in-game test by the user pending**:
   icon appears in the quick access bar (hover lighter, tooltip "Notes"/
   "Notizen"), click toggles the window, window hides in loading screens and
   character select and reappears afterwards, hot reload leaves no duplicate
   icon.

## ▶ Resume here

After the in-game test passes, phase 0 is complete → ask the user to approve
merging `chore/phase-0-foundation` into `develop` (R29), then start phase 0b
(price archive; first push needs explicit approval, R11).

## User to-dos

- Verify "trailkit" in TMview + DPMAregister.
- Google Alert for C-703/25 (D18).

## Recurring checks (2026-10-31)

- GW2 wiki: still behind Cognito login → Wayback workaround stays.
- C-703/25 P: pending, no hearing date; DPF in force.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`).
- Node 24 via fnm, pnpm 11.17.0 already installed.
