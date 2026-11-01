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

- None currently (session 4 settled M16–M28; details are settled per
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
