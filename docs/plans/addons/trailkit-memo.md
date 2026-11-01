# trailkit-memo

- **Status:** draft — requirements settled (M1–M15)
- **Crate path:** `addons/trailkit-memo`
- **Nexus signature / ID:** TBD

## Goal

An in-game notes app: a tree of groups/subgroups with markdown notes, reset-
aware checklists (daily/weekly), and — with trailkit-core — live event
references with autocompletion and status.

## Requirements (from user)

### Notes & structure
- List of notes, edited as **markdown**.
- **Groups and nested subgroups**; notes live in groups.
- Notes, groups and subgroups have **stable unique IDs**.
- Notes and groups are **reorderable / movable by drag & drop**.
- Note **type**: normal (default), **daily reset**, **weekly reset**. On reset
  all checkboxes in that note are unchecked.
- Cancelling an edit with unsaved changes asks for confirmation.
- Keybind opens/closes the notes window.

### Events (requires trailkit-core)
- Typing `/e` in a note opens **autocompletion** of events.
- Event references show start time and whether the event is running now.
- Selected/referenced events are visible to trailkit-event (highlighting).
- Without core: event settings greyed out + hint "more features with
  trailkit-core"; memo works as a plain markdown notes list.

### Import / export / backup
- Export and import **lists** (notes/groups).
- Export and import **settings**.
- **Backup**: one file with settings + all notes.
- Import of notes that already exist (same ID) opens a dialog to choose which
  existing notes get updated.

### Project-wide
- Own settings window, help page, en/de, themes, performance (sleep when
  closed).

## Technical design (proposal)

- IDs: UUID v7 (time-ordered) newtypes `NoteId`, `GroupId`.
- Storage: see M16–M18.
- Editor: Dear ImGui has no rich-text editor. Proposal: **edit mode** = raw
  markdown in a multiline input (with `/e` autocomplete via input callback);
  **view mode** = rendered markdown (headings, lists, bold/italic, links,
  checkboxes clickable, event chips). Renderer: M19.
- Event reference syntax: see M7.
- Reset times: daily 00:00 UTC, weekly Monday 07:30 UTC (GW2 standard
  resets). Reset applied on next open / on timer while running, based on
  `last_reset_applied_at` per note (so it also works if the game was closed).
- Export format: versioned JSON with `schema_version`.

## Open questions

- None currently (session 4 settled M16–M46; details are settled per
  implementation step).

## Decisions

| # | Decision | Date |
|---|----------|------|
| M1 | Built first, works fully without core (phase 1) | 2026-10-31 |
| M2 | Notes and groups have a scope: **global (default)** or **character**; changeable at any time — *amended by M26: character scope requires core* | 2026-10-31 |
| M3 | Edit/view toggle per note; v1 markdown: headings, bold/italic, lists, checkboxes, links, rules; toolbar with heading/bold/list/checkbox | 2026-10-31 |
| M4 | Toolbar has an **event button** opening a searchable event dropdown (alternative to `/e`) | 2026-10-31 |
| M5 | Child scope inherits from parent and can never be broader; moving a group to a character asks whether children follow; char select screen shows global notes only; toggle "show notes of all characters" — *amended by M26* | 2026-10-31 |
| M6 | Checkbox state of global daily/weekly notes is shared across characters (without core all notes are global, M26) | 2026-10-31 |
| M7 | `/e` or the event button inserts `[[event:<id>|<fallback name>]]` (custom events: `[[event:custom:<id>|<name>]]`); view mode renders a chip (name, start time / "running now", countdown); without core: greyed chip with stored name + hint | 2026-10-31 |
| M8 | Export via right-click → "Export…": single note, group incl. subgroups, or everything; versioned JSON (`.tpmemo`) plus "copy as text code" (e.g. for Discord) | 2026-10-31 |
| M9 | Import conflicts: dialog lists notes whose IDs already exist; per entry overwrite / keep / import as copy (new ID); "overwrite all" / "keep all"; expandable before/after preview; non-conflicting notes import directly | 2026-10-31 |
| M10 | Import target: groups keep their structure; existing group (same ID) receives the notes; new groups appended at the end; optional target group selectable before import | 2026-10-31 |
| M11 | Automatic local backups: daily on first open, keep the last 7, in the addon folder; "restore from backup" in settings (separate from manual export) | 2026-10-31 |
| M12 | Deletion: trash bin, purged after 30 days; "Deleted — Undo" toast; deleting a non-empty group asks for confirmation showing the note count | 2026-10-31 |
| M13 | Reset times: daily 00:00 UTC, weekly Monday 07:30 UTC; applied on next open if the game was closed at reset; note shows type icon + time to next reset | 2026-10-31 |
| M14 | Opening: Nexus quick-access icon + keybind, keybind **unbound by default**; first-start hint where to set it | 2026-10-31 |
| M15 | Search field above the note list, searching titles and content (respecting scope); hits highlighted, groups with hits expanded | 2026-10-31 |
| M16 | Persistence format: pretty-printed JSON (`serde_json`), same family as the export format (M8) | 2026-11-01 |
| M17 | Two files under `Paths_GetAddonDirectory("trailkit/memo")`: `settings.json` and `notes.json` (incl. trash), each with `schema_version`, atomic writes (R22) | 2026-11-01 |
| M18 | Saving on the worker thread, debounced 2 s after the last change, plus immediately on unload | 2026-11-01 |
| M19 | Markdown: `pulldown-cmark` with the tasklist extension; the renderer lives in `crates/trailkit-ui` from the start (help pages of every addon reuse it) with hooks for clickable checkboxes and event chips | 2026-11-01 |
| M20 | Help content `help/{en,de}.md` is embedded with `include_str!` (never out of sync with the DLL) | 2026-11-01 |
| M21 | Main window: two panes (tree with search on the left, note with edit/view toggle on the right) with a draggable splitter; below a width threshold single-pane (tree → note → back); responsive per `lay-responsive` | 2026-11-01 |
| M22 | Drag & drop for notes **and groups**: drop on the upper/lower edge = before/after, on a group = into; plus "Move to…" in the context menu as non-drag alternative | 2026-11-01 |
| M23 | Notes and groups are allowed on every level, including the root | 2026-11-01 |
| M24 | `notes.json` stores the tree nested: a group has an ordered `children` list of notes and groups; order = list position (no parent ids / sort keys) | 2026-11-01 |
| M25 | Note fields: `id` (UUID v7), `title`, `body` (markdown), `kind` (`normal` / `daily_reset` / `weekly_reset`), `created_at`, `updated_at`, `last_reset_applied_at` (reset kinds only). Group: `id`, `title`, `children`, `expanded`. Trash: separate list of entries with `deleted_at` and the former position | 2026-11-01 |
| M26 | Character scope (M2, M5) exists only with trailkit-core + API key; a character is identified by name + `created` from the API (survives renames). Without core every note is global; the scope control is greyed out with a core hint. No manual character reassignment | 2026-11-01 |
| M27 | Phase 1 settings: language (auto/en/de), theme (dark/light), quick access icon, outside-gameplay behavior, reset time display (local/UTC), backups (restore, open folder), settings export/import | 2026-11-01 |
| M28 | Phase 1 order, one `feature/` branch + PR each: storage & data model → tree & note view (markdown renderer) → editing & toolbar → reset kinds (M13) → drag & drop → search → trash & undo → settings window & options tab → export/import → backups → help page | 2026-11-01 |
| M29 | Shared persistence crate `crates/trailkit-storage` (atomic write, `schema_version` check, step-wise migration, safety copies, read-only on newer versions — R22); memo keeps only its data model | 2026-11-01 |
| M30 | Threading: the render thread owns the note tree; every change sends a copy over a channel to the worker, which debounces (M18), serializes and writes; the worker loads at start, the window shows "Loading…" until the data arrives; the render thread never touches files | 2026-11-01 |
| M31 | Unreadable `notes.json`: never overwritten automatically; preserved as a safety copy (M33), memo starts with an empty list and shows a prominent hint with "Open folder" (plus "Restore from backup" from step 10 on) | 2026-11-01 |
| M32 | Timestamps are stored as RFC 3339 UTC (`jiff::Timestamp`); unknown fields within the same `schema_version` are ignored (new fields always bump the version, newer versions are read-only) | 2026-11-01 |
| M33 | All safety copies live in one `backups/` subfolder (shared with the daily backups of M11), each kind capped: migration copy `notes.v<N>.bak.json` written before the first migrated save, only the newest per file, deleted after 30 days; copies of unreadable files `notes.corrupt-<time>.json`, max. 3; daily backups max. 7. The addon folder itself only holds the live files | 2026-11-01 |
| M34 | Step 2 scope: tree display, expand/collapse, select + rendered view, "New note"/"New group", inline rename; body editing in step 3, delete in step 7 | 2026-11-01 |
| M35 | Bold/italic/headings: embedded **Noto Sans** family (regular, bold, italic, bold italic, OFL) in `trailkit-ui`, registered once via the Nexus font API for all addons; body size = Nexus `FontUI` size (follows the GW2 interface size), headings 1.5× / 1.3× / 1.15×; fonts are rebuilt only on load or a size change, never per frame | 2026-11-01 |
| M36 | Icons: Lucide as icon font for all addons; custom icons in Lucide style (`lay-icons`) | 2026-11-01 |
| M37 | Links open in the default browser on click, only for `http(s)://` (other schemes render as text); tooltip shows the full URL; no confirmation | 2026-11-01 |
| M38 | Checkboxes are clickable in view mode; a click toggles `[ ]`/`[x]` in the markdown source and triggers a save | 2026-11-01 |
| M39 | Window state (selected note, splitter position, scroll position) is stored in its own `ui_state.json`, not in `settings.json`; window position/size is kept by ImGui | 2026-11-01 |
| M40 | Single-pane below two minimum pane widths; minimum pane width ≈ 22 characters of the current font (≈ 440 px threshold at default size, scales with the font) | 2026-11-01 |
| M41 | Code location: markdown renderer, fonts and icons in `trailkit-ui`; tree, splitter and empty state start in memo (move on second use, rule 7) | 2026-11-01 |
| M42 | Empty state: short text ("No notes yet") with primary button "Create your first note" and secondary "New group" | 2026-11-01 |
| M43 | New notes/groups go into the selected group, else the root, appended at the end, titled "New note"/"New group" in rename mode with the text selected; a new note is selected | 2026-11-01 |
| M44 | Inline rename via double-click or context menu: Enter or clicking elsewhere commits, Escape cancels, an empty title restores the old one | 2026-11-01 |
| M45 | Tree icons: none for groups (expand arrow) and normal notes; reset notes get a letter badge in a fixed leading column (empty for normal notes, so titles stay aligned): en **D**/**W**, de **T**/**W**, rounded box in Lucide style with accent color, drawn with ImGui; tooltip names the kind and reset time | 2026-11-01 |
| M46 | The reset badge counts as an icon: Lucide style with round corners, although panels and buttons keep the theme's radius 0 | 2026-11-01 |
| M51 | Step 3 editing: the note toolbar shows a flat pencil button (right) while viewing, "Cancel" and the primary "Done" while editing; double-clicking the note also starts editing. In edit mode the title is a single-line input above the body | 2026-11-01 |
| M52 | Editing works on a copy (`EditSession`): "Done" or Ctrl+Enter writes title and body back (saved debounced, M18), "Cancel" discards and asks first if something changed | 2026-11-01 |
| M53 | Formatting row (edit mode, wraps per `lay-responsive`): heading, bold, italic, bulleted list, checkbox, link (line prefixes toggle; bold/italic wrap the selection), plus Ctrl+B / Ctrl+I; the event button (M4) is disabled with a "requires trailkit-core" tooltip. ImGui 1.80 cannot wrap lines in a multiline input — resolved by the own editor (M58) | 2026-11-01 |
| M54 | Unsaved changes: closing the window (X, keybind, quick access, Escape) or navigating (other note, new note/group) asks "Save / Discard / Keep editing" — only if something changed (`form-unsaved`; such a question is a decision, not navigation, `nav-modal-use`). On unload the edit becomes a draft in `drafts.json` instead of being saved; opening the note shows "Unsaved changes from … — Restore / Discard" (and whether the note changed since). Escape: registered with `GUI_RegisterCloseOnEscape` (Nexus consumes the key); with unsaved changes memo reopens the window and asks, while the dialog is shown Escape keeps editing; Escape inside a text input keeps the typed text | 2026-11-01 |
| M55 | Draft cleanup: one draft per note; deleted on restore, discard or a regular save; at start drafts of missing notes, drafts older than 30 days and all beyond 20 are removed; an empty draft list deletes `drafts.json`; `drafts.json` is versioned and written atomically | 2026-11-01 |
| M56 | The unsaved changes dialog appears over the memo window only (`overlay-dialog-scope`: a borderless window covering exactly the memo window dims it, title bar included, with a centered pane; the game and other windows stay usable; clicking memo brings the dialog back to the front); Enter saves, Escape or a click on the dimmed area keeps editing; the editor is read-only while it is shown | 2026-11-01 |
| M58 | Step 3b: own text editor replaces ImGui's multiline input (word wrap). Lives in `crates/trailkit-ui`: a platform-independent text model (text, cursor, selection, undo, wrapping, highlighting; unit-tested) plus an ImGui view (drawing, mouse, keyboard); the order is step 3 → 3b → 3c → reset kinds | 2026-11-01 |
| M59 | Editor v1 features: word wrap (overlong words break hard); mouse click/drag/double-click word/shift+click; arrows, Home/End, Ctrl+arrows (words), Ctrl+Home/End, PageUp/Down, all with Shift to select; typing incl. umlauts, Backspace/Delete (Ctrl = word), Enter continues lists (`- `, `- [ ] `, `1. `; Enter on an empty item ends the list); Ctrl+A/C/X/V via ImGui's clipboard; undo/redo Ctrl+Z/Ctrl+Y with typing grouped; scrolling keeps the cursor visible. Not in v1: IME, find/replace | 2026-11-01 |
| M60 | Editor display: markdown source with light highlighting (heading lines bold, markers like `#`, `**`, `- [ ]`, link brackets in `text.secondary`). A toggleable live preview (Lucide `eye`): side by side in a wide pane, ~~switching editor/preview in a narrow one~~ (superseded by M72: always side by side); off by default, remembered in `ui_state.json`; the editor/preview split is resizable with the same splitter as the panes (shared `trailkit_ui::splitter`, remembered); re-parsed only when the text changes; independent scrolling in v1 | 2026-11-01 |
| M61 | Step 3c: GW2 chat codes (`[&…]`) are decoded locally (type + id) and shown as chips in view and editor; a click copies the code for the chat; with trailkit-core the chip shows the real name (waypoint, item, …); names come from our own lookup, not from third-party addons such as Decoder Ring (user decision 2026-11-01) | 2026-11-01 |
| M57 | Dialog titles are headings (Heading3); setting `unsaved_changes` in `settings.json` (`ask` default / `save_automatically`: closing or navigating saves without asking, "Cancel" discards without asking; drafts on unload unchanged); `settings.json` schema version 2; the toggle comes with the settings window (step 8) | 2026-11-01 |
| M62 | Text font changed from Noto Sans to **Open Sans** (static Regular/Bold/Italic/Bold Italic TTFs from googlefonts/opensans, OFL 1.1, no Reserved Font Name; softer, rounder look requested by the user; covers all characters of the texts incl. „ “ … –). Amends M35. Subsetting (done after 3b): `cargo xtask subset-fonts [--check]` (allsorts, xtask only) writes subsets of the originals in `assets/fonts/<font>/source/` — text fonts keep U+0020–007E, U+00A0–024F, U+2000–206F, U+2190–21FF, the icon font only `Icon::ALL`; unit tests check that every icon, UI string and help page character is covered. Open Sans has no arrows, so texts use › instead of → | 2026-11-01 |
| M63 | Markdown views show a single line break as a line break (like Obsidian/GitHub comments/Discord); blank lines still separate paragraphs. Applies to all `trailkit-ui` markdown views; help pages keep one paragraph or list item per source line. List marker column fits the task box plus a gap | 2026-11-01 |
| M64 | Checked task boxes use Lucide `square-check` (tick inside the box) instead of `square-check-big` | 2026-11-01 |
| M65 | Visual and interaction tests for all ImGui code, as their own step right after 3b (before font subsetting and 3c): (1) more logic moves out of the ImGui views into plain, Linux-tested functions (key → command mapping, hit testing); (2) headless ImGui tests with simulated mouse/keyboard input; (3) snapshot tests: a small CPU rasterizer turns ImGui draw data (with the real fonts and themes) into PNGs compared against reference images with a tolerance, a diff image on failure, regenerated on purpose. First coverage: editor (empty, wrapped, selection, highlighting), memo main window (wide, narrow, drawer, empty state), unsaved changes dialog, each in dark and light. Tools live in `crates/trailkit-ui-testing` (dev-dependency only, never in an addon DLL); references under each crate's `tests/snapshots/`. Run on the Windows CI runner (and locally via WSL interop if possible). Nexus-specific behavior (keyboard routing, Escape, font atlas) still needs the in-game test | 2026-11-01 |
| M66 | Markdown views set consecutive list items tight (no paragraph gap between them); paragraphs, headings and whole lists keep the gap (found while reviewing the first snapshots) | 2026-11-01 |
| M67 | README screenshots are deferred to the release preparation (user decision): right before the first release everything is tested in full, screenshots are rendered and all docs are updated, so nothing is outdated. Proposal to decide then: `cargo xtask screenshots` renders scenes with the M65 harness at 2× scale onto a generated dusk gradient (no game imagery) into `addons/<addon>/docs/screenshots/` — main window dark/light, editor with preview, narrow window with drawer; English sample notes; not checked by CI | 2026-11-01 |
| M68 | Bold and italic toggle like the line prefixes: markers of the bold/italic span containing the cursor/selection (parsed with pulldown-cmark, also with the cursor in the middle of the text), around it or included in it are removed on a second click; italic never removes half of a bold marker (`***x***` handled) | 2026-11-01 |
| M69 | The formatting toolbar shows what applies at the cursor, like a word processor: heading, bold, italic, list, checkbox and link buttons are drawn in the accent color when active (same style as the preview toggle); computed from the parsed note, cached per text revision and selection | 2026-11-01 |
| M70 | Dialog layout: card on `bg.panel` with a 1 px `border` frame and `space.lg` padding, more space between heading, text and buttons; min width ≈ 28 characters, max 90 % of the window, text wraps; "Discard" left (apart), "Keep editing" and the primary "Save" right-aligned; when they do not fit, buttons stack full width (Save, Keep editing, Discard); `overlay.modal` stronger (dark 70 %, light 50 %) | 2026-11-01 |
| M71 | The link button also toggles: inside `[label](url)` it removes the link syntax and keeps the label. The menu button (≡) is drawn in the accent color while the note list is visible (column shown or drawer open), like the other active toggles | 2026-11-01 |
| M72 | Editor and preview always stay side by side, however narrow the window (supersedes the narrow-window switching of M60, user decision 2026-11-01): in edit mode both must stay visible; each keeps only ~3 characters so the splitter stays usable | 2026-11-01 |
| M73 | Note list width (user decision 2026-11-01): the splitter may make the list narrower than the regular pane minimum (down to ~10 characters); the default width and the drawer threshold (M40) stay unchanged. While editing with the preview visible, the note pane needs two pane minimums, so the list turns into the drawer earlier | 2026-11-01 |
| M74 | The note list has a fixed width instead of a share of the window (user decision 2026-11-01): resizing the window gives the extra room to the note; the list is squeezed only while the window lacks space and returns to its width afterwards. Stored in `ui_state.json` as `note_list_characters` (average characters, follows the font size; default 22 = regular pane minimum); the old `splitter_fraction` is ignored | 2026-11-01 |
| M75 | Step 3c details (user decisions 2026-11-01, refines M61): a chat code is only `[&` + base64 characters + `]` (all GW2 link types share it; the first decoded byte is the type, the payload differs per type); **all** link types get a chip (coins show their value, e.g. "2 g 50 s"; build templates "Build (profession)"; achievements included); without a name the chip shows a localized "type #id" label (items with quantity ×n); the raw code stays in the markdown, names are never stored; in the editor a chip is an atomic cell (cursor skips it, Backspace deletes it whole); click copies the code with a short "Copied" feedback, tooltip shows type, id and code; codes arrive only by pasting (no toolbar button in 3c); malformed codes or unknown types show a warning chip "!" with the reason in the tooltip (click copies the raw text), "does not exist" needs core data; chips only in note bodies (not titles, not the tree); the chip component lives in `crates/trailkit-ui` (pill on theme tokens, rarity colors with core); "Open in wiki" is postponed (BL10) | 2026-11-01 |
| M76 | With trailkit-core: slash commands `/w` (waypoint), `/e` (event, M7) and later other types open a name search and insert the chat code (user decision 2026-11-01) | 2026-11-01 |
| M77 | 3c implementation details: waypoints, points of interest and vistas share one link type, so without data the chip says "Map point #id" ("Kartenpunkt"); coins show amounts with gold/silver/copper dots (theme tokens `coin.*`, no unit letters to translate); in the **editor** a click only places the cursor (copying there would overwrite the clipboard while editing) and the tooltip shows kind, id and code; copying is a click in the view and the preview; player links show "Player" without decoding the name; icons are centered by their visible glyph bounds; the skill chip uses Lucide `swords` (user decision 2026-11-01) | 2026-11-01 |
| M78 | Running text (note view, preview, editor) uses line height 1.5 × font size (theme token `font.line_height`, user decision 2026-11-01: all lines, not only lines with chips), text centered in its line; the whole memo window uses the shared body font, so the note list and the note have the same size (the window's default font differed) | 2026-11-01 |
| M79 | Text size (user decisions 2026-11-01): all text in the memo window scales with `settings.json` `text_scale` relative to Nexus' `FontUI` size, default **1.125** (larger than before), range 0.75–2.0 in steps of 0.125; changed with **Ctrl+mouse wheel** over the window, **Ctrl+0** resets; the settings window (step 8) adds a slider. Chips are calmer: icons at 80 % of the font size (about capital-letter height), slimmer and more subtle pill | 2026-11-01 |
| M80 | Polish after in-game test (user feedback 2026-11-01): chips get a small corner radius (3 px, not a pill; the theme radius is 0); chip icons are drawn at the text size and centered on the capital letters (supersedes the 80 % of M79, which looked too small); a text-size change is applied to the fonts only 0.4 s after the last wheel step (rebuilding the fonts on every step flickered), and the size shows as a percentage next to the mouse for 1.5 s | 2026-11-01 |
| M81 | The note view keeps title and separator fixed at the top; only the content scrolls, so its scrollbar starts below the separator (user decision 2026-11-01). Chip icons at 90 % of the text size (100 % looked taller than the capitals, 80 % too small) | 2026-11-01 |
| M82 | Chips a bit smaller (user decision 2026-11-01): icons at 85 % of the text size, chip reaches only a sixth of the line leading above and below the text (was a third) | 2026-11-01 |
| M83 | Bug fix: in a scrolled editor (long notes) the invisible keyboard input sat at the content top, out of view, so ImGui did not activate it and Nexus sent the keys to the game; it is now drawn at the visible top (regression test `keyboard_stays_captured_when_the_editor_is_scrolled`) | 2026-11-01 |
| M50 | Groups are recognizable by a **bold** title (bold text font) and the number of notes they contain (nested groups included), right-aligned in `text.secondary`; no own background (`lay-hierarchy`). Drag & drop stays in step 5 (M28 order unchanged) | 2026-11-01 |
| M49 | Replaces the badge column of M45/M47: note titles start flush left (where a group's arrow starts), reset badges and any later row icons sit right-aligned at the row's right edge; the title is shortened with "…" before them. Groups keep the expand arrow on the left, so group titles start one arrow width further right | 2026-11-01 |
| M48 | Replaces the single-pane "tree → note → back" of M21/M40: the note pane has its own toolbar row with a flat menu button (Lucide `menu`, tooltip) on the left (step 3 adds the edit/view toggle on the right). Wide window: the button hides/shows the note list column (stored in `ui_state.json`). Narrow window: the note fills the window and the button opens the note list as a drawer (`lay-drawer`) over the dimmed note — instantly, no slide animation (`overlay-motion`); width = minimum pane width, at most 85 % of the window; closes on choosing a note, a click on the dimmed note or the menu button (repeated in the drawer's toolbar); stays open for groups, expand/collapse and rename; open automatically while no note is selected; Escape still closes the window. Drawer code starts in memo (rule 7) | 2026-11-01 |
| M47 | Amends M45: the badge column of a note row sits in the slot of the expand arrow (notes never have one), so note and group titles on one level align and the tree is not indented twice; tree rows are one frame tall; rename follows `lay-inline-edit` | 2026-11-01 |
