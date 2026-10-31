# trailkit-event

- **Status:** draft — requirements settled (E1–E15)
- **Crate path:** `addons/trailkit-event`
- **Nexus signature / ID:** TBD

## Goal

An event timeline/table showing when and where events happen (like the GW2
wiki event timer), with custom events organized in groups, multiple
configurable tables, and highlighting of events referenced in trailkit-memo.

## Requirements (from user)

- Event table/timeline: **when, where, which** event.
- Highlights events that the user referenced/selected in **trailkit-memo**
  (toggleable).
- **Custom events**: recurring daily or weekly, one or multiple times per
  day/week. Handling of time conflicts in the timeline is an open question.
- Custom events organized in **groups and subgroups** (e.g. `Guild 1 → Raid 1,
  Raid 2`), displayed as categories like the GW2 wiki timer.
- Toggle whether custom events are shown and whether memo events are
  highlighted.
- **Multiple event tables**, each configurable (e.g. table A: official
  events only; table B: custom events only).
- Project-wide: own settings window, help page, en/de, themes, performance.

## Technical design (proposal)

- Official schedules come from trailkit-core (curated dataset, see data-pipeline / backend
  plan). Without core: only custom events (to confirm).
- Table = filter config `{ sources, groups, categories, time window, layout }`
  rendered as horizontal timeline rows per category (wiki style) with a "now"
  marker; countdowns update once per second.
- Custom event model: `CustomEvent { id, group_id, name, duration, recurrence:
  Daily{times} | Weekly{weekday_times}, timezone (local vs. UTC), color,
  waypoint chat link? }`.
- Conflicts: overlapping events in one category row → stacked sub-rows
  (proposal).
- Export/import of custom events and tables (same mechanism as memo →
  shared crate).

## Open questions

- None currently (round 14 settled E8–E15).

## Decisions

| # | Decision | Date |
|---|----------|------|
| E1 | Each table is its own freely placeable window, managed in settings | 2026-10-31 |
| E2 | Overlapping events in one category are stacked as extra sub-rows | 2026-10-31 |
| E3 | trailkit-event **requires** trailkit-core; without core it shows a guided install hint instead of the timeline | 2026-10-31 |
| E4 | Custom event **data owned by trailkit-core** (single source of truth, also visible in memo); created/edited in trailkit-event via the core protocol | 2026-10-31 |
| E5 | Custom events store the creator's IANA time zone + weekday + time; displayed in each viewer's local time (DST-correct) | 2026-10-31 |
| E6 | Sharing custom events in v1 via export/import (file + copyable text code); backend sync in backlog | 2026-10-31 |
| E7 | Time zone is auto-detected from Windows (`iana-time-zone`), shown as a hint in the form ("Times in Europe/Berlin") with a small "change" link for exceptions | 2026-10-31 |
| E8 | Views per table: **timeline** (wiki style, rows per category, "now" line; default) or **list** ("next events" with countdown) | 2026-10-31 |
| E9 | Timeline window default −30 min … +2 h; zoom (mouse wheel) and horizontal scroll per table | 2026-10-31 |
| E10 | Reminders: bell per event → toast X min before (default 5, 1–30); optional sound (default off); optional auto-reminders for events referenced in memo (default off) | 2026-10-31 |
| E11 | Left click copies the waypoint chat link (toast "waypoint copied"); right-click menu: reminder on/off, find in memo notes, copy name, **open wiki page** (en or de wiki per addon language; needs wiki page names in the event dataset) | 2026-10-31 |
| E12 | Tables are managed in settings ("+ table": name, filter by source/categories/groups, view, lock position/size); show/hide per table via Nexus quick-access menu; one global keybind toggles all tables, **plus an optional keybind per table** (registered dynamically per table id); all keybinds unbound by default | 2026-10-31 |
| E13 | Tables hide automatically during loading screens, cutscenes and character select; "hide in combat" option (default off) | 2026-10-31 |
| E14 | Events referenced in memo get an accent-colored outline + star; toggle per table | 2026-10-31 |
| E15 | First start creates one table "Official events" with all categories in timeline view | 2026-10-31 |
