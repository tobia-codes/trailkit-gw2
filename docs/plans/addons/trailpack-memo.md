# trailpack-memo

- **Status:** draft — requirements settled (M1–M15)
- **Crate path:** `addons/trailpack-memo`
- **Nexus signature / ID:** TBD

## Goal

An in-game notes app: a tree of groups/subgroups with markdown notes, reset-
aware checklists (daily/weekly), and — with trailpack-core — live event
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

### Events (requires trailpack-core)
- Typing `/e` in a note opens **autocompletion** of events.
- Event references show start time and whether the event is running now.
- Selected/referenced events are visible to trailpack-event (highlighting).
- Without core: event settings greyed out + hint "more features with
  trailpack-core"; memo works as a plain markdown notes list.

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
- Storage: one JSON (or RON) document in the addon dir, written atomically
  (write temp + rename) by the worker thread, debounced.
- Editor: Dear ImGui has no rich-text editor. Proposal: **edit mode** = raw
  markdown in a multiline input (with `/e` autocomplete via input callback);
  **view mode** = rendered markdown (headings, lists, bold/italic, links,
  checkboxes clickable, event chips). Renderer uses `pulldown-cmark` → custom
  ImGui drawing (candidate for `trailpack-ui` markdown component).
- Event reference syntax: see M7.
- Reset times: daily 00:00 UTC, weekly Monday 07:30 UTC (GW2 standard
  resets). Reset applied on next open / on timer while running, based on
  `last_reset_applied_at` per note (so it also works if the game was closed).
- Export format: versioned JSON with `schema_version`.

## Open questions

- None currently (round 12 settled M7–M15).

## Decisions

| # | Decision | Date |
|---|----------|------|
| M1 | Built first, works fully without core (phase 1) | 2026-10-31 |
| M2 | Notes and groups have a scope: **global (default)** or **character**; changeable at any time | 2026-10-31 |
| M3 | Edit/view toggle per note; v1 markdown: headings, bold/italic, lists, checkboxes, links, rules; toolbar with heading/bold/list/checkbox | 2026-10-31 |
| M4 | Toolbar has an **event button** opening a searchable event dropdown (alternative to `/e`) | 2026-10-31 |
| M5 | Child scope inherits from parent and can never be broader; moving a group to a character asks whether children follow; char select screen shows global notes only; toggle "show notes of all characters" | 2026-10-31 |
| M6 | Checkbox state of global daily/weekly notes is shared across characters | 2026-10-31 |
| M7 | `/e` or the event button inserts `[[event:<id>|<fallback name>]]` (custom events: `[[event:custom:<id>|<name>]]`); view mode renders a chip (name, start time / "running now", countdown); without core: greyed chip with stored name + hint | 2026-10-31 |
| M8 | Export via right-click → "Export…": single note, group incl. subgroups, or everything; versioned JSON (`.tpmemo`) plus "copy as text code" (e.g. for Discord) | 2026-10-31 |
| M9 | Import conflicts: dialog lists notes whose IDs already exist; per entry overwrite / keep / import as copy (new ID); "overwrite all" / "keep all"; expandable before/after preview; non-conflicting notes import directly | 2026-10-31 |
| M10 | Import target: groups keep their structure; existing group (same ID) receives the notes; new groups appended at the end; optional target group selectable before import | 2026-10-31 |
| M11 | Automatic local backups: daily on first open, keep the last 7, in the addon folder; "restore from backup" in settings (separate from manual export) | 2026-10-31 |
| M12 | Deletion: trash bin, purged after 30 days; "Deleted — Undo" toast; deleting a non-empty group asks for confirmation showing the note count | 2026-10-31 |
| M13 | Reset times: daily 00:00 UTC, weekly Monday 07:30 UTC; applied on next open if the game was closed at reset; note shows type icon + time to next reset | 2026-10-31 |
| M14 | Opening: Nexus quick-access icon + keybind, keybind **unbound by default**; first-start hint where to set it | 2026-10-31 |
| M15 | Search field above the note list, searching titles and content (respecting scope); hits highlighted, groups with hits expanded | 2026-10-31 |
