# Next session — handoff

Last updated: 2026-11-12 (session 31). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 31 (2026-11-12).** R98/B25: problem reports via the backend behind a flag, GitHub form as fallback (Q288–Q293 in
`backend.md`). Q286 (tooltip looks like a button) is a later design round. Before the first release: DPF listing check (R99).

**▶ Work branch: `feature/help-problem-page`** (local only on the user's WSL machine, **not pushed**, 16 commits ahead of
`origin/develop`; keep working on it, no new branch, no push without the user's OK). Done: `trailkit-log::export`, `help_topics`,
R96 components, the shared help window in `trailkit-ui`, and in session 31 the **memo wiring** (f5fe420, 5a6db13, 1af2e81):
`TogglePlacement::Trailing` in the title bar with memo's `?` before the chevron (Q171), `addon/help.rs` (`MemoHelp`: log check and
export on short-lived threads, Downloads via `SHGetKnownFolderPath`, Explorer `/select`, issue form via `problem_report_url`),
`help_text.rs` (en/de texts; the German status texts for saved/trimmed/no log were drafted by Claude, not in the mockup),
`log_export.rs`, events `log_check_failed`/`log_export_failed`, Escape closes the help window (`HELP_OPEN` registered with Nexus),
and the `?` in every memo snapshot (user OK; 327 changed only in the title row). Not deployed or tested in game yet.
**Open, in this order:**
1. **Nexus options tab "Open help" (user wants it, R53):** waiting for the user's answers and visual OK (R87) on
   https://claude.ai/artifact/94EzMkYMDQHFmLw1b2fTdZ — **Q307** buttons: A only "Open help" until a settings window exists
   (recommended) or B with a disabled "Open settings"; **Q308** look: trailkit style on its own surface (recommended) or Nexus' own
   style (C); **Q309** header: window title Heading2, `trailkit-memo <version>` Secondary (recommended). Then: record the answers in
   the memo plan, `RT_OptionsRender` → `HelpWindow::open` (never closes; Q296), snapshots.
2. **Help rewrite M181:** nine topics in `help/{en,de}.md` (`##` topic, one Secondary lead sentence, `###` structure), privacy topic from
   `PRIVACY.md`/`.de.md`, parity test with `subsection_counts`; mention the `?`, the help window and Escape (rule 6).
3. **Ask the user:** the markdown view spaces `###` headings and paragraphs wider than the mockup (shared renderer, changes note
   snapshots): keep or tighten? Draft warning and problem hint still hand-draw status colors (an icon needs R87).
4. **Validate and ship (R74):** fmt, host and `cargo xwin` clippy, `cargo test`, `cargo xtask ui-test` (about 4 min, background, no filter;
   one `UiHarness` at a time), `check-file-length`, `check-names`, `cargo xtask deploy memo`, in-game test (export, Explorer selection,
   ` (2)` naming, GitHub link, Escape with both windows open), then the user's merge OK: push, PR into `develop`, auto-merge.

**Deferred by the user (2026-11-12): one theme character for addons and the coming React website.** No code yet. Mockups:
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (latest: Papier A–D). Wanted: GW2-reminiscent, own character, dark, very subtle
(background about `#060709`–`#0d0e17`), crisp edges. Rejected: mountains, waypoint markers, header band/spine, web-template look. Closest:
Papercraft Q–T and Flecken U–X (posterized mottled paper); recommended Papier B Rissrand. ImGui draws flat fills, polygons, one-direction
gradients, offset shapes as shadow, no blur or noise: mottling needs a PNG from a `cargo xtask`. Brass never decorates. Before code:
grilling (name, variant, Light, tokens in `theme.rs`/`tokens.json`, one place in `draw_window`) plus R87. ArenaNet's Content Terms
for fan content are not read yet; never copy game or Blish HUD art, never look official.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).

Next ids: **M182+, R101+, B26+, questions from 310** (Q307–Q309 open). Deferred: GW2 build/Nexus version in `session_start` (C12).

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
