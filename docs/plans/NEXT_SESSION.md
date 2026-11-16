# Next session — handoff

Last updated: 2026-11-16 (session 40; violet accent, rust/ move, token source, pre-push hook). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo, UX/keyboard improvements, help window and status cards through M186 and the crash protection (R104) are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**First, ask the user: can you test in game now?** Pending checks with where/how/expected: [`in-game-tests.md`](in-game-tests.md)
(user request 2026-11-13: keep it current until each item is confirmed; ask every session). A failed test becomes a `bugfix/` branch.

**On the phone** ("nehme next session auf dem handy auf"): no game, skip that question; grilling asks **one question per message**.

**Cloud sessions:** auto mode may refuse `git commit`: ask the user, never work around it. PRs via `gh api .../pulls`, auto-merge
via `PUT .../pulls/{n}/ccr/auto_merge` (`merge_method=merge`); branch deletion is blocked (the user deletes). Push does not set
the upstream (narrow fetch refspec): set `branch.<b>.remote/merge` by hand for the stop hook. `cargo xwin clippy` works.

**On the Mac (no game, no WSL):** `git fetch origin && git switch develop && git pull --ff-only`; deploy, `ui-test` and in-game
tests wait for the Windows PC; check `source ~/.cargo/env` and a first `cargo test` there; snapshots via the PR's Windows CI.

**Session 38 (2026-11-14, cloud):** PR #129 merged (German empty-trash fixture, test only). C-703/25 P checked via
secondary sources only (CURIA blocked in cloud): still pending, no hearing, DPF in force. The user chose memo first:
**favorites grilling in progress on `feature/memo-favorites`** — decided M188–M191 (Q399–Q406, Q409–Q413); open Q407,
Q408, Q414–Q419 (trailkit-memo.md "Open questions"; Q407/Q408 carry the user's follow-up questions). Then R87 mockup.
Core-mandatory plan R109 merged (PR #130); R110–R112 and the frontend topics went to develop on their own PR.

**Open, in this order:**
1. **In-game tests 1–3, 6–8, 10, 11, 18** ([`in-game-tests.md`](in-game-tests.md)) whenever the user is at the PC; deploy `develop` first.
   **Session 40 (2026-11-16):** tests 4, 5, 9, 12–17 OK; stuck rename fixed (#134). Q436 (menu divider) open. Q437–Q443 settled:
   violet accent R124, red fill R125, chips R126, calendar icons M193 on `feature/violet-accent` (deployed, test 18).
   **Session 39 (2026-11-15):** wishes of 2026-11-14 settled as ROADMAP R114–R123 (Q420–Q435; memo M192) on
   `chore/plan-wishes-2026-11-14`. **Next:** the frontend/backend round (R121: stack, SSR, deployment, accounts without
   email, sync, admin roles, forum, REST API, price history). Core wishes settled: layout editor R122,
   hint about new addons R123. **Then** continue the favorites grilling on `feature/memo-favorites` (Q407, Q408, Q416–Q419),
   mockup (R87), implementation. The `rust/` move (R116, #137) and the token source and generator (R118, #138) are merged. Review finding 2 done; **next: finding 3**.
2. **Remaining wishes of 2026-11-13:** memo struck-through checked items, then the second window. event:
   filters ("used in notes", a meta train order: research trains, timing, a source with licence/GDPR first), favorite events.
   All addons: more languages without growing the DLLs (BACKLOG BL15; reworks the `get` tables kept whole in Q392).
3. Last: chat translation addon feasibility (BL16, research first); then the queue below and the review items, one at a time.

**Decided, not scheduled:** Q340 backups with restore after settings and export/import (M28); Q329 `cargo bloat` at release prep.

**Lessons:** measure before fixing; remove probes by editing; mockups first (R87); snapshot checks need a (draft) PR.

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X; recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction gradients, offset shapes
as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code: grilling plus R87. ArenaNet's
Content Terms for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).
Next ids: **M194+, R129+, B26+, BL18+, questions from 463**. Deferred: GW2 build/Nexus version in `session_start` (C12).
## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures:** done (memo M180, 2026-11-10).
2. **Pre-push protection:** done (Q460–Q462, R48, 2026-11-16).
3. **WvW snapshot validity:** `rust/tools/trailkit-datagen/src/wvw.rs::write_reset_snapshot`
   saves current matches without checking the intended pre-reset time/match period.
   A delayed job can succeed with post-reset data. Check the actual Nov 6 EU run as well.
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
