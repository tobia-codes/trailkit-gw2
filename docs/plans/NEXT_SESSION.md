# Next session — handoff

Last updated: 2026-11-13 (session 35, PR #124 open). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo, UX/keyboard improvements, help window and status cards through M186 and the crash protection (R104) are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**First, ask the user: can you test in game now?** Pending checks with where/how/expected: [`in-game-tests.md`](in-game-tests.md)
(user request 2026-11-13: keep it current until each item is confirmed; ask every session).

**Session 35 (2026-11-13).** PR [#124](https://github.com/tobia-codes/trailkit-gw2/pull/124) `feature/button-cleanup` → `develop` is open
(user OK to push and open it; auto-merge NOT armed). R105 (delete always red, menu entries too), R106 (irreversible discards red,
UX rule `lay-danger-discard`), R107 (button states: colored fills 8 %/12 % darker), M187, R103 cleanup; mockup https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr (v6, section anchors).
The cloud clone fetches only `develop`; to work on the branch: `git fetch origin feature/button-cleanup:feature/button-cleanup && git switch feature/button-cleanup`.

**Open, in this order:**
1. **PR #124:** 125 snapshots reviewed (§10) and committed; the review found an unreadable tooltip on the red icon button (fixed in
   `c9d2d22`). Still to render via CI (R97) and review: `memo-state-trash-hover-delete-*`, `ui-button-danger-icon-{hover,pressed}-*`
   (tooltip must be readable). Then all checks green, in-game tests 1–9, merge OK, auto-merge. Side finding: `min-de-dialog-*`
   fixtures build the confirmation text in English (test only); fix only if the user agrees.
2. **File-length splits (R77):** approved Q91 (`note_list.rs`/`draw_group`), Q95 `draw_dropdown`, Q96 `editor/view.rs`, Q105
   `markdown_view.rs`. Grill the rest by concern, never split unasked: `theme.rs` (421), `rows.rs` (422, `draw_note_row` 91),
   `main_window.rs` (428), `actions.rs` (451, Q134), `help_text.rs::get` (104, a table; Q92 said no for `Text::get`),
   `component_states.rs` (1127), `main_window/tests.rs` (1010), `editor.rs::draw` (101), `token_groups.rs` (466).
3. **Wishes of 2026-11-13 (grilling; plan requirements):** event: filters ("used in notes", a meta train order: research trains,
   timing, a source with licence/GDPR first), favorite events. memo: struck-through checked items, a simple daily/weekly window,
   favorites. All addons: more languages without growing the DLLs (BACKLOG BL15).
4. **Last, once the rest is done:** a grilling round on whether the chat translation addon (BACKLOG BL16) is feasible, research first.
5. Then the queue below (theme character grilling, B23–B25) and the review items, one at a time.

**Decided, not scheduled:** Q340 – daily backups with restore (M11, M31's "Restore from backup") stay in M28's order after settings
and export/import. Q329 – `cargo bloat` breakdown at the next release preparation (release DLL 2.51 MiB on 2026-11-13).

**Lessons from session 34 (keep doing):** measure before fixing (a one-off log probe proved the flicker in one round). Nexus draws
alerts as yellow text without background at the top centre; an addon window covers them. `git checkout <file>` to drop a probe also
drops every other edit in that file: remove probes by editing. Mockups first (R87) paid off again.

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X; recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction gradients, offset shapes
as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code: grilling plus R87. ArenaNet's
Content Terms for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).
Next ids: **M188+, R108+, B26+, questions from 379**. Deferred: GW2 build/Nexus version in `session_start` (C12).

- **Housekeeping (Q211/Q240):** the user deletes approved old local branches by hand (`chore/*`, merged PR branches).

## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures:** done (memo M180, 2026-11-10).
2. **Pre-push protection:** `.githooks/pre-push` checks protected destination refs only for deletions; normal pushes check the current
   branch via `xtask/src/check_branch.rs`. Validate every destination ref; server-side protection was not inspected.
3. **WvW snapshot validity:** `tools/trailkit-datagen/src/wvw.rs::write_reset_snapshot`
   saves current matches without checking the intended pre-reset time/match period.
   A delayed job can succeed with post-reset data. Check the actual Nov 6 EU run as well.
4. **Search cache:** `addons/trailkit-memo/src/main_window/search.rs::update_search` calls `SearchQuery::parse` before checking the
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
- **C-703/25 P:** Nov 11 CURIA check: still pending (case in progress); issues #11/#20 closed. Order of June 4, 2026 (ECLI:EU:C:2026:465).
  Previous “sources only from 2025” note was stale. Recheck when triggered; source:
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
