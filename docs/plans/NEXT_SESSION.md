# Next session — handoff

Last updated: 2026-11-18 (session 43; find in note, data-safety tests, README packages). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, both searches,
  trash/undo, UX/keyboard improvements, help window, status cards and crash protection (R104) are implemented and tested in game (through M199).
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**First, if [`in-game-tests.md`](in-game-tests.md) lists tests, ask the user: can you test in game now?** Remove each item once
the user confirms it (user requests 2026-11-13, 2026-11-16); an empty list means no question. A failed test becomes a `bugfix/` branch.

**On the Mac or phone** ("…auf dem mac/handy auf"): no game, so skip that question and add every needed game check to
[`in-game-tests.md`](in-game-tests.md) in the same commit (2026-11-16). Phone: grilling asks **one question per message**.
**Cloud sessions:** auto mode may refuse `git commit`: ask the user, never work around it. PRs via `gh api .../pulls`, auto-merge
via `PUT .../pulls/{n}/ccr/auto_merge` (`merge_method=merge`); branch deletion is blocked (the user deletes). Push does not set
the upstream (narrow fetch refspec): set `branch.<b>.remote/merge` by hand for the stop hook. `cargo xwin clippy` works.

**On the Mac (no game, no WSL):** `git fetch origin && git switch develop && git pull --ff-only`; deploy, `ui-test` and in-game
tests wait for the Windows PC; check `source ~/.cargo/env` and a first `cargo test` there; snapshots via the PR's Windows CI.

**Session 43 (2026-11-18, PC), all on `develop`:** #144 M194–M197; #145 find in note, Shift twice (M198, M199); #146 stopped-worker fix, 22 data-safety
tests; #147 README **Packages** (R138: update with every dependency change). All in-game tests passed; the list is empty.
**Next: data pipeline** (`rust/tools/trailkit-datagen`): the failing GemExchange job below (review item 3 done, D42), then tests for `archive.rs`, `wvw.rs`,
`gem_exchange.rs`, `gw2_api.rs`. Coverage 2026-11-18 before #146: addons 89 % of lines, memo app 70 %, datagen 52 %.
**Then test gaps:** extract and test the Escape/close logic of `nexus_bridge/render.rs` like `key_routing.rs`; window actions (new group, format,
preview, compact view, open folder, clear search); Ctrl+Y/Ctrl+Shift+Z keys. `xtask` last. Prove new tests with injected faults.
**Coverage on demand, not in CI** (`llvm-tools` installed): `RUSTFLAGS="-C instrument-coverage" CARGO_TARGET_DIR=target/coverage cargo xwin test --no-run`, run
each binary from its crate with `LLVM_PROFILE_FILE`/`TRAILKIT_SNAPSHOT_ROOT` via `WSLENV` (`/p`), `llvm-profdata merge`, `llvm-cov export`; then delete `rust/*.profraw` and the 2.8 GB build.

**Open, in this order:**
1. **In-game tests:** none open. After that pipeline work: Q501 toast visibility needs an R87 mockup; BL18 text size in core settings needs grilling.
   Propose splits via grilling for length hints: `shared/i18n.rs`, `get`, `slices/editor/editor.rs`, `draw_find_bar`, `draw_main_window`, `actions.rs`.
   Q436 (menu divider) stays open. The user kept the current find marks (current filled, others framed; section 5 of the mockup was declined).
   **Favorites** on `feature/memo-favorites` (M188–M191 decided): open Q407, Q408, Q416–Q419, then mockup (R87) and implementation.
2. **Remaining wishes of 2026-11-13:** memo struck-through checked items, then the second window. event:
   filters ("used in notes", a meta train order: research trains, timing, a source with licence/GDPR first), favorite events.
   All addons: more languages without growing the DLLs (BACKLOG BL15; reworks the `get` tables kept whole in Q392).
3. Last: chat translation addon feasibility (BL16, research first); then the queue below and the review items, one at a time.

**Memo data model (M200–M203, 2026-11-18, phone), not implemented:** keep `notes.json`; `expanded` to `ui_state.json`; forward-compatible
fields (unknown fields/values kept, M202); `kind` → `reset_schedule` (`never`/`daily`/`weekly`) with a migration step. Before the first release.
**Decided, not scheduled:** Q340 backups with restore after settings and export/import (M28); Q329 `cargo bloat` at release prep.

**Lessons:** measure before fixing; remove probes by editing; mockups first (R87); snapshot checks need a (draft) PR.

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X; recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction gradients, offset shapes
as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code: grilling plus R87. ArenaNet's
Content Terms for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).
Next ids: **M204+, R139+, B46+, BL20+, questions from 515**. Deferred: GW2 build/Nexus version in `session_start` (C12).
## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures:** done (memo M180, 2026-11-10).
2. **Pre-push protection:** done (Q460–Q462, R48, 2026-11-16).
3. **WvW snapshot validity:** done (D42, 2026-11-18): no file and a failed run once the region's matches have ended.
4. **Search cache:** `rust/addons/trailkit-memo/src/main_window/search.rs::update_search` calls `SearchQuery::parse` before checking the
   cache, allocating for unchanged queries every frame. Check unchanged inputs first; no benchmark was run.
5. **Docs/skills accuracy:** READMEs done (R91). Still wrong: `nexus-addon-dev/references/addon-structure.md`
   says Cargo forbids sibling-slice dependencies (it only rejects cycles; the rule is enforced by review);
   `rust-best-practices/references/chapter_09.md` marks `&mut T` as not Send (it is Send when T is Send).
6. **Performance contract:** reconcile the Nexus skill's blanket render-thread parsing ban with the change-triggered Markdown parsing;
   clarify allowed work and caching before any rewrite; keep the performance/security guarantees.
7. **Decision/documentation workflow:** broad token-loading fixes are done (R88).
   Relaxing grilling for routine implementation choices or rustdoc for trivial getters
   was only suggested, not approved. Preserve current rules until explicitly settled.

## Earlier implementation queue — after the review work

- Q134: split `actions.rs` into tree/editor actions at its next rebuild, not beforehand.
- Continue M28: settings/options → export/import → backups → help → release preparation
  (full tests, M67 screenshots, docs, manually measure release DLL size, R72).
- Separate grill round: hide Markdown outside the active editor line. Optional once:
  Light `reset.daily` on pressed tinted dropdown is 4.16:1 (R85 scope).

## Recurring checks and other open items

Triggers/consequences: [agent reference](../agent-reference.md#recurring-checks).
- **Dependencies (R130):** checked and updated 2026-11-17 (26 patch updates on `chore/dependency-patch-updates`, in-game test 19; jiff 1.0 open; nexus-rs current; `cargo audit`: no vulnerabilities, unmaintained `instant` and `paste` via nexus-rs). Next due 2026-11-24.
- **Wiki:** Nov 6 review got HTTP 403; public recovery/login status not confirmed; keep Wayback.
- **C-703/25 P:** Nov 15 check on CURIA: still pending, no hearing, no AG opinion, DPF in force. Order of June 4, 2026 (ECLI:EU:C:2026:465). Recheck on CURIA when triggered:
  https://juris.curia.europa.eu/juris/liste.jsf?language=en&num=C-703%2F25
- **WvW reset snapshots (checked 2026-11-07):** both Nov 6 EU (ran 22:12 UTC, +4.8 h) and Nov 7 NA (07:16 UTC,
  +5.9 h) assets hold the **post-reset** matches; CI was green. Confirms review item 3. GemExchange fails daily since ≤ Nov 10 (HTTP 400, guess: gem > 1 gold).
- **DPF cron:** Nov 1 skipped, issue #11 opened manually. If Dec 1 skips, grill shifting off :00.
- **Flipping volumes (BL9):** DataWars2 vs GW2BLTC differs 3–10×; Silver allowed free use/
  archiving Nov 1 and confirmed suspicious `calculate_sold` branch; user sent the fix.
  GW2BLTC reply unknown; 24 h comparison needs an always-on machine (BL5). `flip_out` unreviewed.
- Nexus hover cursor: drafted Discord message remains with the user.
- User to-dos: verify public GitHub email; TMview/DPMA name check; C-703/25 Google Alert.
  Optional: cargo env in bashrc; GitHub “Automatically delete head branches”.
