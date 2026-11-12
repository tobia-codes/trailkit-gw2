# Next session — handoff

Last updated: 2026-11-12 (session 30). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 30 (2026-11-12).** On `develop`: PR #109 (R96 shared text styles/components), #110 (privacy texts, issue form, `trailkit-logs`
skill), #112 (component snapshots). R98/B25: in-game problem reports through the backend behind a feature flag, R95 GitHub form
as fallback; open points Q288–Q293 in `backend.md`. Q286 (tooltip looks like a button) stays a later design round. Before the first
release: confirm GitHub's DPF listing on `dataprivacyframework.gov` (R99 check).

**▶ Work branch: `feature/help-problem-page`** (local only, **not pushed**, 10 commits ahead of `origin/develop`; keep working on it,
do not start a new branch). Done and committed: `trailkit-log::export` (`build_log_export`, `export_file_name`, `has_logs`; newest
3 days, 5 MiB, header with trims, tests, README), `trailkit-ui::help_topics` (`parse_help_topics`, `subsection_counts`), and the
docs: **R100** (help window re-grilled, Q294–Q304, mockup https://claude.ai/artifact/82dwWQcEtFeTfxpQ2ex1cy approved) and the rule
`lay-button-icon-position` (`ux-rules.md`). The old mockup v3 of R95 is unreadable in this account; build from R100 plus the unchanged
parts of R95. Existing icon buttons all have the icon in front already; no code change was needed for the rule.
The user may still send wording tweaks for the mockup texts. Remaining steps, in order:
1. ~~`draw_status_line` and `draw_list_row`~~ done (5acafb9; Q305/Q306: `draw_row_state`, note list keeps its tree). Draft warning and
   problem hint still draw status colors by hand (icon change needs R87 OK); `Status::Info` comes with the `info` icon (step 4).
2. ~~Help window in `trailkit-ui`~~ done (a067b7f, snapshots in `tests/snapshots/help-window-*`; `Button::opens_link`, `Status::Info`, four icons
   incl. `circle-help` already added). Open: the markdown view spaces `###` headings and paragraphs more widely than the mockup (shared
   renderer, would change note snapshots: needs a decision). Window code is generic: `HelpWindow::draw` takes `HelpWindowContent`.
3. memo: title bar `?` toggles (Q171, Q296), Nexus "Open help", export action on a worker thread (read logs, write to Downloads with
   `export_file_name`, open Explorer with the file selected), GitHub issue link with addon and version, `has_logs` check; button
   variant with a trailing icon for the GitHub button (`lay-button-icon-position`).
4. Help rewrite M181 (nine topics in `help/{en,de}.md`, privacy topic embedded from `PRIVACY.md`/`.de.md`, parity test with
   `subsection_counts`); icons are done.
5. Deploy, in-game test, then the user's merge OK (R74); push the branch, PR into `develop`, arm auto-merge. Pushing before that
   (e.g. to continue on another device) also needs the user's OK.

**Open design round (user, 2026-11-12):** a window background in the GW2 style that keeps contrast (also for the homepage). Mockup
https://claude.ai/artifact/SFFrXawiqYD6HouhdihGdv (A–H; F frame and H dust nearest). Own procedural textures only; ArenaNet's Content
Terms of Use not read yet (Blish HUD ships game-style art). Brass must not decorate. Needs grilling + R87 before any code.

**Queue after that:** B23/B24/B25: tech-stack grilling, Q288–Q293, `trailkit-i18n` for shared lock texts, icon `wrench`; server stays local (Q223).

Next ids: **M182+, R101+, B26+, questions from 307.** Deferred: GW2 build/Nexus version in `session_start` (C12).

- **Housekeeping (Q211/Q240):** the user deletes the approved old local branches by hand (`contributor-docs`,
  `xtask-token-split-and-docs`, `handoff-session-20`, `addon-status-design`, `addon-status-lock-design`,
  `handoff-session-26`, `handoff-branch-hint`, `help-window-design`, all `chore/`; merged PR branches).

## Review improvements — one at a time, agreed order, solutions still proposals

1. **Storage failures:** done (memo M180, 2026-11-10).
2. **Pre-push protection:** `.githooks/pre-push` checks protected destination refs only
   for deletions; normal pushes check the current branch via `xtask/src/check_branch.rs`.
   Validate every destination ref. Server-side protection was not inspected in the review.
3. **WvW snapshot validity:** `tools/trailkit-datagen/src/wvw.rs::write_reset_snapshot`
   saves current matches without checking the intended pre-reset time/match period.
   A delayed job can succeed with post-reset data. Check the actual Nov 6 EU run as well.
4. **Search cache:** `addons/trailkit-memo/src/main_window/search.rs::update_search`
   calls `SearchQuery::parse` before checking the cache, allocating for unchanged queries
   every frame. Check unchanged inputs first. No performance benchmark was run.
5. **Docs/skills accuracy:** READMEs done (R91). Still wrong: `nexus-addon-dev/references/addon-structure.md`
   says Cargo forbids sibling-slice dependencies (it only rejects cycles; the rule is enforced by review);
   `rust-best-practices/references/chapter_09.md` marks `&mut T` as not Send (it is Send when T is Send).
6. **Performance contract:** reconcile the Nexus skill's blanket render-thread parsing
   ban with existing change-triggered Markdown parsing. Clarify allowed work and caching
   before proposing an architectural rewrite; keep actual performance/security guarantees.
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
