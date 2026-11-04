# Next session — handoff

Last updated: 2026-11-04 (session 15: PR #83 merged; new window layout M136–M139 on branch `feature/memo-sidebar-layout`, deployed, awaiting in-game OK; trash & undo half grilled).

## Where we are

- **Phase 0 + 0b complete.** Daily archive runs (sometimes hours late).
- **Phase 1 (trailkit-memo standalone):** steps 1 (storage), 2 (tree & note
  view), 3 (editing), 3b (own editor with preview), 3c (chat-link chips),
  reset kinds and **drag & drop (M106–M117)** are merged into `develop` and
  tested in game; **search (M118–M129)** is merged and deployed, not yet
  confirmed in game. Decisions in `docs/plans/addons/trailkit-memo.md`.
- Order from here (M28 as amended by M58, M62, M65):
  ~~visual & interaction tests~~ → ~~font subsetting~~ → ~~3c chips~~ →
  ~~reset kinds~~ → ~~drag & drop~~ → ~~search~~ → **trash & undo
  (delete, M12, next)** → settings window → export/import → backups → help page →
  **release preparation** (full test, screenshots M67, docs, measure the
  release DLL size by hand, R72).
- **Session 9 (PRs #65–#69): new "Trailkit" theme, ROADMAP R71** (supersedes
  R51), all deployed; the user called the result "richtig gut":
  - Dark/Light (named just that): ink-tinted dark and parchment-warm light
    surfaces; **brass** accent `#E2A740` spent only on the primary action,
    focus and selection; `accent.text` for brass text, icons and lines.
  - Selection = neutral `bg.active` + short inset 3 px brass bar
    (`trailkit_ui::selection_bar`); tree expand arrows are Lucide chevrons
    (`tree_arrow`); no folder/note icons (M49 stays).
  - Chat-link chips in four categories (`ChatLinkKind::category()`,
    `theme.chip.*`): place copper, character blue, item jade, progress
    violet; id in `text.secondary`; invalid links stay warnings.
  - Radii 4/6/10 (`radius.sm/md/lg`); buttons filled with `bg.hover`
    (never darker than the pane); separators in `border.subtle`, drawn on a
    whole pixel; no pane highlight edge, no shadows.
  - **One dropdown** (`draw_dropdown(ui, theme, tint, …)`): button height,
    `bg.hover` fill, `border` frame, optional tint; the chip variant is gone.
  - **Own title bar** (`title_bar::draw_window`): no band, Lucide chevron
    collapses, bold title, Lucide `x` closes; memo's Edit is a brass button
    with a pencil (`draw_primary_button_with_icon`); `draw_icon_button`
    moved into trailkit-ui.
  - Inter was tried and **dropped** (≈130 KB bigger, wider, less soft) —
    Open Sans stays (M62).
  - **Binding token guideline** for Claude and Codex:
    `.agents/skills/trailkit-ui/references/tokens.md` (linked from the
    trailkit-ui skill).
  - Visual reference in the user's Claude account: design system
    "Trailkit" (https://claude.ai/artifact/G7sNVZe1JYTgxbUE5mhS33) and the
    mockup canvas (https://claude.ai/artifact/Dp8BGiKczKw1ysiBmA8acj). Both
    are **out of date** in places (still mention Inter, the pane highlight
    and a selection bar at a fixed x); `theme.rs` is the source of truth.
- Session 8 (PRs #63–#64): drag & drop and "Move to…" for notes and groups
  (M106–M115), gold drag-label border (M116), text size steps of 6.25 %
  (M117).
- Session 7 (PRs #52–#61), all deployed and checked in game by the user:
  - Reset kinds: `reset_schedule` (daily 00:00 UTC, weekly Mon 07:30 UTC),
    due resets untick all tasks on open/while open, wait for edited notes
    and drafts (M88–M90); kind via chip dropdown right behind the note
    title (view mode only, M102) or right-click › Reset type; countdown
    only in the badge tooltip (M95); colors `reset.daily` gold /
    `reset.weekly` violet for badge and chip (M104); bold, filled,
    ink-centered badges (M105).
  - Editor: one toolbar `[◧] | H1 H2 | B I | lists | link event | 👁 …
    Cancel Save` with group separators (M94, M100, M102); Save/Speichern
    instead of Done (M100); Ctrl+S = Save (M93).
  - Note list toggle = sidebar icon `panel-left-close/open` (M101).
  - Right-click: groups and empty list area offer New note/New group
    (M96, M97); submenus open flush and use Lucide check/chevron
    (`trailkit_ui::menu`, M98, M99).
  - All addons: radius 4 px and `border.subtle` outline (6 % white dark /
    10 % black light) for windows, panes, popups, dialogs (M103).
  - trailkit-ui gained `dropdown` (`Field`, `Chip { tint }`), `menu`,
    `glyph_ink_box`; harness `right_button_down`.
- Late session 5 (PRs #26–#30): bold/italic/link toggle off from anywhere
  inside the span (M68, M71, parsed with `markdown::inline_ranges`), toolbar
  buttons light up for the formatting at the cursor (M69), ≡ is gold while the
  note list is visible (M71), roomier unsaved-changes dialog with a border,
  right-aligned primary and stacked buttons in narrow windows (M70).
  Snapshot tests are strict now (no differing pixels; CI renders identically).
- Session 6 (PRs #33–#50), all tested in game by the user:
  - Layout: drawer scrim really dims (#33), editor and preview always side
    by side (M72), narrower note list + earlier drawer while previewing
    (M73), fixed note-list width (M74), note title stays at the top while
    the content scrolls (M81).
  - Dev loop (R70): auto-merge with required checks on `develop` (user set
    branch protection + `allow_auto_merge`), docs-only PRs skip the
    Windows job (verified, ~40 s), CI runs on push only for `main`,
    pre-push also runs `cargo xwin clippy` for the Windows target.
  - **3c chat-link chips** (M75–M77, M80–M82): `trailkit_ui::chat_link`
    decodes every link type locally, `chat_link_chip` draws them (icon at
    85 % of the text size, centered on the capitals; 3 px radius; slim);
    click copies in view/preview, atomic in the editor; warning chip for
    broken codes. Names later via core (C15); slash commands `/w`, `/e`
    with core (M76); never touch the game chat (C16); "open in wiki" → BL10.
  - Text: line height 1.5 × (token `font.line_height`, M78); whole memo
    window uses the body font (list and note had different sizes); text
    size setting `text_scale` (default 112.5 %, Ctrl+mouse wheel, Ctrl+0,
    percentage tooltip, fonts rebuilt only after the wheel rests; M79/M80).
  - Fixes: keyboard sink stays in view in scrolled editors (keys went to
    the game in long notes, M83); tooltips start right of GW2's large
    cursor (`cursor_extent`, M84); text-button labels centered by their
    capitals (`draw_button`, M85).
- Session 5 also: Open Sans replaces Noto Sans (M62), panes with padding
  (`lay-pane-padding`), drawer + menu button (M48), titles flush left with
  right-aligned badges (M49), bold groups with note count (M50), in-window
  dialogs (`overlay-dialog-scope`), Escape via Nexus registration.

## ▶ Resume here

0. **Open branch first:** `feature/memo-sidebar-layout` (local, **unpushed**,
   2 commits, **deployed** into the game). The user has **not** yet tested
   it in game nor said OK — ask for the in-game test, then "darf ich
   mergen?"; after the OK: push, PR into `develop`, auto-merge (R74). The
   trash & undo implementation continues **on the same branch** unless the
   user wants the layout merged first.
1. **What the branch contains** (session 15, questions 63–88):
   - Title bar (R78): addon toggles left — **⋮ tools** (`ellipsis-vertical`)
     and **list** (note list) — bold title, then collapse chevron + `x` on
     the right; first/last button flush with the panes below.
   - **Tool rail** (M136, `trailkit_ui::tool_rail`): hidden by default,
     holds Search now; Trash comes with the trash step, Settings later at
     the bottom.
   - Note list toggle moved to the title bar (M137); `panel-left` in the
     note toolbar and the drawer close button are gone.
   - **Search field only while shown** (M138): magnifier in the rail or
     Ctrl+F; stays (also across restarts) until hidden; hiding clears it.
   - **Compact note view** (M139): `maximize-2` in the note toolbar hides
     toolbar + kind dropdown, `minimize-2` at the title row end; editing or
     creating ends it.
   - **R79:** toggles that keep a state are neutral (`bg.active` +
     `text.primary`), never brass (`draw_icon_toggle`, tokens.md §3).
   - Mockups of every variant and state: Claude Design canvas
     https://claude.ai/artifact/5cVQAjdFNuuWHfP5Nz36Q5 (the chosen layout is
     the row "Beide Schalter im Fensterkopf").
2. **Trash & undo — settled** (M133–M135, M140–M142): delete via right-click
   and Delete key; empty groups without asking, non-empty with note count;
   undo for delete + move, 20 steps in memory, Ctrl+Z / Ctrl+Y; restore to
   the former place; the edited note cannot be deleted; toast in trailkit-ui.
3. **Trash & undo — still open, ask first (questions 74–80, recommended
   "yes" each):** Q74 trash view (rail icon switches the list; newest first,
   "27 d left", read-only preview with Restore / Delete forever), Q75
   actions (right-click, "Empty trash" button replaces New note/group,
   confirm, not undoable, stay in the view), Q76 deleted groups restore only
   as a whole, Q77 toast (bottom centre in the window, 5 s, hover pauses,
   one at a time), Q78 selection after delete (below, else above, else
   empty), Q79 purge after 30 days at start + hourly, Q80 drafts stay with
   a trashed note. The user may want mockups for Q74/Q75 first. Next ids:
   **M145+, R80+, questions from 91**. Also on the branch: M143 (rounded
   row fills, round selection bar ends) and M144 (full-width rows).
4. **Open questions raised by the implementation:**
   - R79 vs the editor toolbar: the formatting buttons still light up in
     brass for the formatting at the cursor (M69) — apply R79 there too?
   - R77 hint: `shared/src/i18n.rs` `Text::get` has 86 code lines (a flat
     string table) — propose: no split (lookup table), or split per area.
5. Earlier R77 candidates (`--all`, untouched): `editor/view.rs`,
   `markdown_view.rs`, long `draw_*` functions (see session 14 notes).
6. Optional, offer once: update the Claude Design system "Trailkit"
   (https://claude.ai/artifact/G7sNVZe1JYTgxbUE5mhS33) to R71–R79.
7. Working rules that proved useful:
   - UI tests: `cargo xtask ui-test [--update]`; look at PNGs only when
     they change. Tests with fixed list coordinates use
     `selected_first_with_search` (search field shown, M138).
   - Characters in help pages must exist in the subset fonts (`⋮` does not;
     the test `every_character_is_in_the_subset_fonts` catches it).
   - Lucide codepoints by name: parse the source TTF `cmap` + `post`
     (a stdlib Python script did it in session 15).
   - Test fonts are 16 px, in game 18 px: verify in-game screenshots.
   - Deploy after every UI change (`cargo xtask deploy trailkit-memo`);
     check `Nexus.log` for "Loaded addon".
   - In this WSL shell plain `grep` may fail; use `command grep`. Run
     `source ~/.cargo/env` before cargo.
   - Never leave an `eprintln!` debug line before `cargo fmt`.

## Session 15 summary (2026-11-04, PR #83 + branch)

- PR #83 (addon blueprint R76, length hints R77) merged.
- Trash & undo grill round 1 (Q63–Q70) settled; the trash entry point led
  to a whole layout round (Q68–Q88) with ~30 mockups on a Claude Design
  canvas: tool rail like VS Code, both layout toggles in the title bar,
  hideable search, compact note view, neutral toggles (user: "perfekt").
- Implemented and deployed on `feature/memo-sidebar-layout`.

## Session 14 summary (2026-11-04, PRs #81–#82)

- R76 questions 48–58: note-list slice (`ListFilter` trait, closures for
  reset extras, empty search result in the search slice), `note_tree`
  split, `trailkit_ui::text_layout`, `main_window` split with an `apply`
  dispatcher.
- **M132** row badge letters pixel-centred: visible ink measured from the
  font atlas (`glyph_metrics::glyph_visible_ink`), letter drawn at the
  sub-pixel centre (`draw_glyph_at`), all badges one width, height by
  ink-centre parity (rule in the trailkit-ui skill). Took three in-game
  rounds — measure in-game screenshots, the test renderer is 16 px and
  samples textures nearest-neighbour.
- **R77** advisory length hints (`cargo xtask check-file-length`).
- Addon structure blueprint (`references/addon-structure.md`).
- Nexus hot reload stopped once in a game session (see below); a game
  restart fixed it. If a deploy shows no "Loaded addon" line in
  `Nexus.log`, ask the user to reload or restart.
- The user is learning Rust (asked about `self.0`, derives, tuple structs,
  test placement, closures): explain with TypeScript comparisons.

## Session 11 summary (2026-11-03, PRs #77–#80)

- **M130/M131** (#77, #78): primary toolbar buttons carry a Lucide icon
  (Save = floppy `save`, Edit = pencil); text buttons are at least
  `button.min_width` = 4 × font size wide, icon buttons stay square.
- **R74** (#79): merge into `develop` only after the user's in-game OK.
- **R75** (#79): three type layers — `trailkit-gw2-api-types` (boundary
  only), `trailkit-model` (our refined domain types, shared by addons,
  datagen, backend), `trailkit-protocol` (`#[repr(C)]`).
- **#80**: `main_window.rs` split into `main_window/{note_list,note_pane,
  dialogs,layout,tests}.rs`; check-names allows snake_case module dirs
  below `src/`.
- **R76** (local branch): addons are built from slice crates; step 1 done.

## Session 10 summary (2026-11-03, PRs #71–#75)

- **R72 DLL size:** jiff without default features (no bundled tzdb),
  local time via `SystemTimeToTzSpecificLocalTime` (Wine maps it under
  Linux/Proton), clippy bans `jiff::tz::TimeZone::system`. Memo release
  DLL 2 315 264 → 1 964 032 bytes. User: **performance beats size** —
  `opt-level` 3 and `panic = "unwind"` stay; measured `s` −11 %, `z`
  −14 %, abort −14 %. Compared with installed addons memo is mid-range
  (Killproofme 14.8 MB, Event Timers 3.7 MB, DRF 2.4 MB, ArcDPS 1.4 MB).
- **Search M118–M127** (`note_search`, `search_field`,
  `trailkit_ui::text_search`, `markdown::plain_text`,
  `markdown_view::SearchHighlight`), token **`accent.highlight` R73**
  (brass budget now has four uses, tokens.md §3), Lucide `search` icon.
- **M128** list header (equal full-width buttons that stack, taller
  field, clear `x` mirrors the magnifier), **M129** fixed list header.
- **#74** `xtask subset-fonts` fixes Lucide glyph boxes (see facts).

## Facts learned in session 10

- The Lucide source font stores `xMin`/`yMin` = 0 for every glyph, so
  ImGui glyph quads were far larger than the ink and icons could not be
  centered by `glyph_ink_box`. `cargo xtask subset-fonts` now rewrites the
  boxes from the outline points (`fix_glyph_bounds`); drawing positions do
  not change.
- Test fonts are 16 px, in game 18 px: anything sized from an icon
  button's natural width can overflow in game while tests look fine —
  size inner controls from the ink or a fixed square.

## Facts learned in session 9

- ImGui's `AddLine` already adds half a pixel; add none yourself or a 1 px
  line straddles two rows and shows as a dim 2 px band in game (the test
  renderer does not show it).
- ImGui draws a tree node's triangle in the node's text color: push a
  transparent `Text` color while pushing the node, then draw chevron and
  title yourself.
- Windows with `NO_TITLE_BAR` can be moved from any empty area even when
  `ConfigWindowsMoveFromTitleBarOnly` is set.
- Lucide glyph codepoints: parse the source TTF's `cmap` + `post` tables in
  Python (e.g. `x` = U+E1B2); no fontTools available.

## Facts learned in session 7

- ImGui submenus overlap their parent by `ItemInnerSpacing.x` (read in
  `BeginMenu`); `trailkit_ui::menu::begin_submenu` sets it to 0.
- A `Selectable`/button click fires on release over the item: UI tests
  must release on the item before moving the mouse away.
- Python heredoc edits: never `open(p,'w')` and read the same file in one
  expression (it truncated `main_window.rs` once; restored from git).

## Facts learned in session 6 (also in the skills)

- The keyboard sink must stay **in view**: ImGui does not activate an
  input scrolled out of its window, and Nexus then sends every key to the
  game (M83, nexus-addon-dev skill).
- Open Sans sits low in ImGui's line-box centering; center by the capitals
  (`glyph_metrics::capital_middle`) — text buttons via `draw_button` (M85).
  Lucide icons fill the whole line box: a 16 px "H" is 9 px tall, the icon
  square 15 px.
- `DrawListMut` is a singleton in imgui-rs: shared components draw through
  the raw `ImDrawList_*` calls when a caller may hold one (dialog card).
- Tooltip position = mouse + 24 px × `style.MouseCursorScale`; `StyleScope`
  sets the scale from the measured cursor (`cursor_extent`).
- Nexus font resizes (`resize_font`) flicker; debounce them (text size).
- The test harness fonts are 16 px; in game the body is FontUI × 1.125
  (18 px) — measure in-game screenshots when sizes matter
  (`png_measure`-style pixel checks worked well).
- Auto mode blocks: changing branch protection (user ran it) and
  `git commit --amend` (use a follow-up commit).

## Nexus / ImGui facts learned in session 5 (also in the nexus-addon-dev skill)

- Nexus consumes **Escape** before ImGui → always `GUI_RegisterCloseOnEscape`.
- Nexus routes **key presses** (arrows, Backspace, Enter) to ImGui only while
  a real ImGui text input is active (`WantTextInput`); custom widgets keep an
  invisible input active ("keyboard sink" in `trailkit_ui::editor::view`).
- `IsWindowHovered` is false while any item is active → use
  `ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM`.
- ImGui 1.80: alpha 0 windows skip all items; `CalcTextSize` rounds up per
  call (measure glyphs with `ImFont_CalcTextSizeA`); no `BeginDisabled`;
  dragging where no item is moves the window (`movable(false)` on children);
  child backgrounds render into the parent's draw list, so a `ChildBg` scrim
  sits below earlier siblings — draw the dimming rectangle in the scrim's own
  draw list (PR #33, trailkit-ui skill §4).
- Nexus keeps fonts per identifier until the game restarts (font changes
  need a restart, not a hot reload).
- The game's hover cursor (gear, speech bubble) leaks through addon windows —
  Nexus issue; Discord message drafted for the user.

## Open with the user

- Sales volumes for a future flipping addon: DataWars2 (api.datawars2.ie,
  Hetzner DE, polls every 4 min) vs GW2BLTC differ by factor 3–10; one hour
  of own 1-minute collection matched Mithril roughly, not ecto/mystic coin.
  Silver (DataWars2) answered 2026-11-01: free use and archiving allowed,
  attribution up to us, nginx may log IPs (see BACKLOG BL9). He confirmed the
  `calculate_sold` branch looks wrong (untested `-total > cutoff` case
  overstates `listed`); the user sent him the fix (use the `else` formula). GW2BLTC e-mail
  drafted, no answer known.
  A 24 h own collection needs an always-on machine (backend, BL5).
  PieOrCake also has a `flip_out` addon (not looked at).
- Nexus hover-cursor issue: user may post the drafted Discord message.

## Nexus hot reload stopped once (session 14, 2026-11-04)

- In the game session started 13:40 no deploy was hot-reloaded; manual
  unload/load in the Nexus addons menu works. Tested: WSL rename-over,
  WSL move-aside + copy, and a native PowerShell rename + copy — Nexus logged
  nothing for any of them, so its directory watcher did not fire; not a
  deploy-script bug. Nexus 2026.10.25.60 (installed 2026-10-25, hot reload
  still worked on 2026-11-03). After a game restart the watcher fired again
  on the next deploy ("Processing changes" in the log): a Nexus hiccup in
  that game session. If it recurs: restart the game; if it persists, ask
  in the Raidcore Discord.

## Recurring checks

- GW2 wiki (2026-10-31): still behind Cognito login → Wayback workaround stays.
- C-703/25 P (checked 2026-11-04, sessions 14/15 — still pending, DPF in force; check again next session): still
  pending, no hearing date or judgment reported; DPF in force. Microsoft intervened for the
  Commission; noyb asked for repeal (2026-06-30) but has not sued yet.
- DPF reminder workflow: the 2026-11-01 08:00 cron was silently skipped by
  GitHub; triggered manually (issue #11). If it skips again on 2026-12-01,
  grill round on moving the cron off the full hour.
- Daily archive: WvW EU snapshot Fri 2026-11-06 17:23 UTC is the real test of
  the cron delay margin (D29).

## User to-dos

- Add + verify `tobiacodes@gmail.com` on GitHub (Settings → Emails).
- Verify "trailkit" in TMview + DPMAregister.
- Google Alert for C-703/25 (D18).
- Optional: add `. "$HOME/.cargo/env"` to `~/.bashrc`.

## Environment notes

- GW2 addons folder: `C:\Guild Wars 2\addons` (WSL: `/mnt/c/Guild Wars 2/addons`);
  Nexus log: `addons/Nexus/Nexus.log` (readable from WSL for diagnostics).
- `cargo xtask deploy trailkit-memo` hot-reloads the addon in a running game.
- Node 24 via fnm, pnpm 11.17.0 already installed.
- `gh` is logged in as `tobia-codes` (ssh).
