---
name: trailkit-logs
description: Analyse a trailkit addon log a user sends (the day files `trailkit/logs/<addon>/<addon>-<yyyymmdd>.log` or a log export) — find crashes, panics and failed saves, map `at=` to code and explain the cause. Use whenever a trailkit log file or pasted log lines appear, or a user reports an addon problem together with a log.
---

# Analysing trailkit logs

The binding format and the meaning of every event are in
[`crates/trailkit-log/README.md`](../../../crates/trailkit-log/README.md)
("Log format"). Read its **Line** and **Events** sections first; this skill
only adds the procedure. Decisions: ROADMAP R89, R92.

## Handle the file as user data

- A log is personal data of the user (when they played, their setup), even
  though it holds no note content. Do not commit it, paste it into issues,
  plans or commits, or send it to any external service. Quote only the lines
  needed to explain a finding.
- If a log contains user content or an absolute path, that is a bug: report
  it as a privacy finding (the logger must not write it).

## Procedure

1. **Identify the build.** Read every `session_start`: `addon`, `version`,
   `build` (`debug` builds come from developers), `commit`, `windows`,
   `language`. Check out or inspect that commit (`git show <commit>`) before
   reading code: `at=` line numbers refer to it. `commit=unknown` means a
   build without Git; then only module paths are reliable.
2. **Split into sessions.** A session runs from `session_start` to
   `session_end`. Several addons in one export each have their own sessions;
   compare times to see whether they failed together.
3. **Look for the hard failures first**, newest session first:
   - `previous_session_unclean`: the session before ended without
     `session_end` (crash or killed game). Its last lines are the best clue.
   - `panic`: message, `thread` and `at=file.rs:line`. A `worker_died` right
     after means the storage worker was the thread.
   - `log_full`: that day logged 5 MiB (five files); something logs in a loop
     (a bug). Find the repeated event just before it; normal lines after it
     are missing until midnight, only `panic`, `worker_died`,
     `save_gave_up`, `session_start` and `session_end` still arrive.
   - `save_gave_up`: a change is not on disk (all automatic writes or the
     last write at unload failed). Followed by `save_recovered` if a later
     write succeeded; without it the user lost that change. The
     `save_failed` lines before it (`attempt=1…`) hold `stage`, `os_error`.
4. **Group WARN and ERROR** by event, `at=` and `file=`. Read `reason`,
   `stage` and `error` (the whole cause chain). Typical causes:
   - `os error 5` access denied: read-only folder, permissions, controlled
     folder access of Windows Defender.
   - `os error 32` file in use: antivirus scan, cloud sync (OneDrive), a
     second program holding the file. Usually passes on retry.
   - `os error 112` disk full; `os error 3` path not found.
   - `load_recovered`: the file was unreadable and moved to `backups/` as
     `preserved_as`; the user's data is in that copy.
   - `load_read_only reason=newer_version`: an older addon version found a
     file from a newer one; the user downgraded.
5. **Map `at=` to code.** `trailkit_memo_shared::storage_worker:318` is crate
   `trailkit-memo-shared` (folder from the workspace `Cargo.toml`), module
   `storage_worker` (`src/storage_worker.rs`), line 318. `::` steps into
   subfolders below `src/`. Search with `rg -n '"<event>"'` to find every
   call of an event.
6. **Check durations.** `ms=` on `load_finished`/`save_failed` far above a
   few milliseconds points to a slow disk, a network drive or a scanner.
7. **Report** briefly: what failed, since when (version, session), the likely
   cause with the evidence lines, whether user data is at risk or where it
   lies (e.g. `backups/`), and the fix or the next question for the user.
   Name what the log cannot tell (no line before a crash, DEBUG lines missing
   in release builds, `Nexus.log` holds nexus-rs' panic backtrace if the user
   can send it).

## Limits

- A normal session is three lines: `session_start`, `load_finished`,
  `session_end`. Successful saves are not logged. `missing=notes.json` on
  `load_finished` outside a first start explains lost notes.
- Only the newest 14 days with a log are kept, none older than 30 days; a
  session can span two day files (`session_continues` starts the second). Ask for a fresh export after
  reproducing the problem.
- Times are the user's local time with offset; compare across users in UTC.
