# trailkit-memo-reset-schedule

Reset-schedule slice of [trailkit-memo](../../README.md) (ROADMAP R76, memo
plan M13, M86–M105). Depends only on
[`trailkit-memo-shared`](../../shared/README.md), never on another slice.
The period type `ResetPeriod` belongs to the shared note model; everything
else about resets lives here.

| Module | Content |
|--------|---------|
| `schedule` | reset times: daily 00:00 UTC, weekly Monday 07:30 UTC (`ResetTimes`), next check, minutes to the next reset |
| `due_resets` | `apply_due_resets`: unticks the tasks of reset notes whose reset passed; edited notes wait (M88–M90) |
| `note_kind` | kind options, the row badge (`RowBadge`, a calendar icon, M193) with its tooltip, the kind dropdown (Windows only) |
| `i18n` | the slice's texts (`ResetText`, en/de) and the reset countdown format |

## Packages

The packages this project uses directly.

| Package | Type | What it does | Why it is used here |
|---------|------|--------------|---------------------|
| [`trailkit-memo-shared`](../../shared/README.md) | own | Note tree, settings, drafts, undo, storage worker and texts shared by memo. | Uses memo's note types, texts and settings. |
| [`trailkit-ui`](../../../crates/trailkit-ui/README.md) | own | Theme, fonts and shared ImGui components. | Uses the shared theme, text and controls. |
| `jiff` | external | Date, time and time-zone types. | Calculates daily and weekly reset times. |
| `nexus` | external, Windows only | Rust bindings for the Raidcore Nexus addon API and its ImGui. | Draws its interface with ImGui. |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). The logic
and texts are tested on any host with
`cargo test -p trailkit-memo-reset-schedule`; badge and dropdown are covered
by the app's window snapshot tests (`cargo xtask ui-test`).
