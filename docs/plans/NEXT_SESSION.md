# Next session — handoff

Last updated: 2026-11-10 (session 23). Current state and
next steps only; decisions belong in plans, lasting facts in skills. Keep under 100 lines.

## Where we are

- Phase 0 + 0b complete; daily archives sometimes run hours late. Memo standalone: storage, tree, editor/preview, chips, resets, drag & drop, search,
  trash/undo and UX/keyboard improvements through M178 are implemented and tested in game.
- Decisions: `addons/trailkit-memo.md`, `ROADMAP.md`. Appearance: trailkit-ui `current-design.md`; review: tokens.md §10.

## ▶ Resume here

**Session 23 (2026-11-10) — B22 addon status grilled, recorded as B23 in `backend.md`** on branch
`chore/addon-status-design` (not pushed; needs the user's merge OK). Done before: PR #103 (M180, R93) merged;
PR #96 closed (Q210); C-703/25 issues #11/#20 closed; local `feature/memo-save-error-design` deleted (only the
superseded panel draft). Update notice mockup approved (Q227): https://claude.ai/artifact/EDHSjZGj6aLW3FXsPMHLDb

**Next — continue the B23 grilling:**
- **Q236 open.** The user asked how an addon can be offline when GW2 is online. Answer given: the request goes to
  our server, not to ArenaNet; it fails when `api.trailkit.de` is down/deploying, the player's DNS/firewall/
  ad blocker (e.g. Pi-hole) blocks it, or it times out. Question still to answer: does a cached
  maintenance/problem lock expire at its expected end or at the latest after 24 h without confirmation, so an
  unreachable server never locks an addon forever? (Recommendation: yes; a cached update lock stays.)
- **Mockups (R87)** for the maintenance and problem locks: default texts "vorübergehend gesperrt" /
  "vorübergehend in Wartungsarbeiten" (Q234), optional message and expected end; Dark/Light, min width, de.
- **Check myself:** in Nexus source, does a `Direct` update reload without a game restart? Adjust the hint text.
- Then: whether the B23 server work comes before R89, and record it in `ROADMAP.md`; R89 (help `?`, export,
  problem page; `PRIVACY.md` incl. `api.trailkit.de`, Q170) follows.

Next ids: **M181+, R94+, B24+, questions from 238.** Still deferred: GW2 build and Nexus version in
`session_start` (need MumbleLink → core, C12). If ever needed: limit repeated log events once core adds network paths.

- **Housekeeping (Q211):** local `chore/contributor-docs`, `chore/xtask-token-split-and-docs`,
  `chore/handoff-session-20` (old handoffs, unmerged commits) need the user's OK to delete.

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
- **C-703/25 P:** Nov 10 CURIA check: still pending (case in progress); issues #11/#20 closed. Order of June 4, 2026 (ECLI:EU:C:2026:465).
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
