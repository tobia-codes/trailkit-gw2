# Next session — handoff

Last updated: 2026-11-13 (session 33). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo, UX/keyboard improvements, help window and status cards through M186 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 33 (2026-11-13).** PR #116 (`feature/status-hints`, auto-merge armed after the user's OK) brings: status cards for memo's
problem and draft hints (M184, shared `trailkit_ui::status_card`, token `warning.subtle`), notes with unsaved changes marked in the list
plus a counting card with "Go to note" (M185, Q341–Q354), right/middle clicks into the game drop the focus (M186), the two button styles
and their order (R103), "Open folder" opens `backups/`, and two game-crash fixes (tooltip over a chat-link chip in the editor: second
`DrawListMut` since R102; focusing no window mid-frame: null navigation window in `ItemAdd`). Check that #116 merged with green CI.

**Open, in this order:**
1. **Crash protection branch (R104):** Q353 `catch_unwind` around every Nexus callback (own grilling round first: keep the editor
   text as a draft? show a hint?), Q355 clippy ban of `get_window_draw_list` and raw focus calls with a nested-safe replacement (33
   uses, no pixel may change), Q356 harness configured like Nexus (keyboard navigation) plus a test over every state with navigation
   planned and a chip tooltip hovered, Q358 two-minute in-game smoke test before every merge in `tokens.md` §10 and `CONTRIBUTING.md`.
2. **Button cleanup (R103, Q338, Q350):** `DangerFill` confirm button, the error toast's `bg.active` buttons and the empty state's brass
   button on the left onto the two styles and the order; R87 mockup first; keep the icon-label gap.
3. **File-length splits, own grilling round (R77):** `theme.rs` (421 lines), `rows.rs` (422) with `draw_note_row` (91),
   `main_window.rs` (426), plus the older hints. Propose a split by concern; never split unasked.
4. Then the queue below (theme character grilling, B23–B25) and the review items, one at a time.

**Decided, not scheduled:** Q340 – daily backups with restore (M11, M31's "Restore from backup") stay in M28's order after settings
and export/import. Q329 – `cargo bloat` breakdown at the next release preparation (release DLL 2.51 MiB on 2026-11-13).

**Lessons from session 33 (keep doing):** mockups before every visible change paid off again (card, X, icon position, button order were
corrected from drafts). Crash traps are now in nexus-addon-dev §2.9; a crash without a `panic` line is read from
`addons/arcdps/arcdps_lastcrash.log` with `llvm-symbolizer`. Never `pkill -f` a pattern that matches the calling shell (tokens.md §10).

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X; recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction gradients, offset shapes
as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code: grilling plus R87. ArenaNet's
Content Terms for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).

Next ids: **M187+, R105+, B26+, questions from 359**. Deferred: GW2 build/Nexus version in `session_start` (C12).

- **Housekeeping (Q211/Q240):** the user deletes the approved old local branches by hand (all `chore/`, `contributor-docs`, `handoff-*`,
  `addon-status-*`, `help-window-design`, merged PR branches).

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

- Approved splits (R77): Q91
  `note_list.rs`/`draw_group`, Q95 `draw_dropdown`, Q96 `editor/view.rs`, Q105 `markdown_view.rs`.
  Q92 `Text::get`: recommendation no, no objection recorded. Grill remaining hints for
  `Editor::draw`, `draw_lines` and the large main-window test file before splitting.
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
  +5.9 h) assets hold the **post-reset** matches; CI was green. Confirms review item 3. Also Nov 6 14:13 catch-up failed on GemExchange.
- **DPF cron:** Nov 1 skipped, issue #11 opened manually. If Dec 1 skips, grill shifting off :00.
- **Flipping volumes (BL9):** DataWars2 vs GW2BLTC differs 3–10×; Silver allowed free use/
  archiving Nov 1 and confirmed suspicious `calculate_sold` branch; user sent the fix.
  GW2BLTC reply unknown; 24 h comparison needs an always-on machine (BL5). `flip_out` unreviewed.
- Nexus hover cursor: drafted Discord message remains with the user.
- User to-dos: verify public GitHub email; TMview/DPMA name check; C-703/25 Google Alert.
  Optional: cargo env in bashrc; GitHub “Automatically delete head branches”.
