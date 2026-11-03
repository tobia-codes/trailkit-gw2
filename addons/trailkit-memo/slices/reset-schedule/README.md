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
| `note_kind` | kind options, reset colors, the row badge (`RowBadge`) with its tooltip, the kind dropdown (Windows only) |
| `i18n` | the slice's texts (`ResetText`, en/de) and the reset countdown format |

## Build and test

Built as part of the addon (`cargo xtask deploy trailkit-memo`). The logic
and texts are tested on any host with
`cargo test -p trailkit-memo-reset-schedule`; badge and dropdown are covered
by the app's window snapshot tests (`cargo xtask ui-test`).
