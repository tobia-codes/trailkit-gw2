# trailkit-log

One local log per trailkit Nexus addon, one file per day (ROADMAP R89, R92).
Every `log::` call of an addon and its crates becomes one line in
`addons/trailkit/logs/<addon>/<addon>-<yyyymmdd>.log`; WARN and ERROR also
reach `Nexus.log`.
This README is the **binding description of the log format**; the skill
[`trailkit-logs`](../../.agents/skills/trailkit-logs/SKILL.md) explains how to
analyse a log a user sends.

> **Status:** used by `trailkit-memo`. Plan: [ROADMAP R92](../../docs/plans/ROADMAP.md).

| Module | Content |
|--------|---------|
| `lib.rs` | `start`, `LogConfig`, `LogSession` (drop = `session_end`, join) |
| `logger` | the `log::Log` implementation: filter, event/fields, Nexus forwarding |
| `writer` | writer thread owning the files, day and part switch, size cap, `log_full`, reserve for critical events |
| `day_files` | file names, listing, cleanup at start, unclean-session detection, `log_files` |
| `line` | line format, quoting, path scrubber |
| `panic_hook` | `panic` line, chained before nexus-rs' hook |
| `error_chain` | `ErrorChain`: an error with all its causes |
| `events` | the events this crate writes itself |
| `system` | build kind, commit (`build.rs`), Windows version |

## Build and test

```sh
cargo test -p trailkit-log
```

`tests/event_catalog.rs` scans `addons/` and `crates/`: every `log::` call
must use a snake_case event listed below, no call may log `.display()`, and
every listed event must still occur in the code.

## Usage

```rust
// In `load`, before anything else logs (Windows-only addon code):
let session = trailkit_log::start(trailkit_log::LogConfig {
    directory: TrailkitDirectory::new(get_addon_dir(TRAILKIT_DIRECTORY_NAME)?),
    addon: AddonName::new("memo"),
    version: env!("CARGO_PKG_VERSION"),
    local_offset: local_utc_offset,
    forward: forward_to_nexus,
    session_fields: vec![("language", "de".to_owned())],
})?;
// Anywhere: the message is the event, details are key = value pairs.
log::error!(file:% = FileName::of(&path), error:% = ErrorChain(&error); "save_failed");
// In `unload`, last: writes `session_end` and joins the writer thread.
drop(session);
```

Use `key = value` for numbers and `&str`, `key:% = value` for anything that
implements `Display`. Choose an existing event before adding a new one; a new
event needs a row in the table below.

## Log format

### File

| Property | Value |
|----------|-------|
| Path | `addons/trailkit/logs/<addon>/<addon>-<yyyymmdd>.log`, e.g. `memo/memo-20261109.log`; one folder per addon, one file per local day; a full file continues in `memo-20261109_1.log`, `_2`, … |
| Encoding | UTF-8, one line per entry, `\n` line ends, appended unbuffered |
| Retention | At start every addon deletes whole files of **all** addons' folders: only the newest **14 days with a log** stay, none older than **30 days**. Nothing is read or rewritten; empty folders go too |
| Size cap | **1 MiB** per file, at most **5 files** per day. In the last file normal lines stop 8 KiB early: `log_full` is written once, then only critical events (`panic`, `worker_died`, `session_start`, `session_end`) until midnight |
| Memory | Nothing is held in memory: lines are appended, only the last 8 KiB of the newest file are read at start to see whether the previous session ended with `session_end` |
| Levels written | ERROR, WARN, INFO; DEBUG only in debug builds. Dependencies (targets not starting with `trailkit`): WARN and ERROR only |
| Nexus | WARN and ERROR also go to `Nexus.log` in the addon's channel (without the time) |

### Line

```text
2026-11-08T15:02:11+01:00 WARN addon=memo version=0.3.0 save_failed at=trailkit_memo_shared::storage_worker:318 file=notes.json ms=3 error="cannot write document: Access is denied. (os error 5)"
```

| Position | Field | Meaning |
|----------|-------|---------|
| 1 | time | Local time, ISO 8601 with seconds and UTC offset (from Windows) |
| 2 | level | `ERROR`, `WARN`, `INFO`, `DEBUG` |
| 3 | `addon=` | Short addon name (`memo`) |
| 4 | `version=` | Addon version |
| 5 | event | What happened, snake_case (table below) |
| 6 | `at=` | Code location: `crate::module:line` (Rust module path; `::` maps to folders below `src/`), for `panic` `file.rs:line` |
| 7… | `key=value` | Details. Values with spaces, `"`, `=` or `\` are quoted; `\"`, `\\`, `\n`, `\r`, `\t` are escaped |

Common fields: `file` (bare file name, never a path), `ms` (duration in
milliseconds), `error` (the whole cause chain, `a: b: c`), `reason` (a fixed
code).

### Privacy

No note text, titles or other user content, and no absolute paths (they
contain the Windows user name; GDPR, `CONTRIBUTING.md`). Paths are logged as
`trailkit_storage::FileName`; every value additionally passes a scrubber that
cuts absolute Windows paths down to the file name. User actions (clicks,
searches, opened windows) are not logged; only technical events and setting
changes. Nothing is sent anywhere: a user exports the log by hand (R89).

### Events

| Event | Level | Fields | Meaning |
|-------|-------|--------|---------|
| `session_start` | INFO | `build` (`release`/`debug`), `commit`, `windows`, `language`, `theme` | The addon was loaded; first line of a session |
| `session_continues` | INFO | the fields of `session_start` | First line of a new day's file while a session keeps running |
| `session_end` | INFO | – | The addon unloaded cleanly; last line of a session |
| `previous_session_unclean` | WARN | – | The previous session has no `session_end`: game or addon crashed, or the game was killed |
| `log_full` | WARN | `max_bytes` | The day's size cap (5 files) was reached; normal lines are missing until midnight |
| `log_unavailable` | WARN | `stage` (`prepare`/`write`), `error` | The log cannot be used; only in `Nexus.log`. After `prepare` nothing is logged this session; after `write` the line is lost, the next one tries again (reported once per session) |
| `log_maintenance_failed` | WARN | `stage` (`cleanup`/`session_check`), `error` | Old files could not be deleted, or the end of the previous session could not be read (it then counts as clean); logging goes on |
| `panic` | ERROR | `thread`, `message` | A Rust panic; `at=` is `file.rs:line` |
| `message` | any | `text` | A log message that is not an event (from a dependency) |
| `folder_unavailable` | ERROR | `error` | Nexus did not return the `trailkit/` folder; nothing is saved or logged; only in `Nexus.log` |
| `worker_start_failed` | ERROR | `error` | The storage worker thread could not start; notes are unavailable |
| `worker_died` | ERROR | – | The storage worker thread panicked (see the `panic` line before it) |
| `load_finished` | INFO | `file`, `result` (`missing`/`loaded`/`migrated`), `from_version`, `ms` | A document was loaded |
| `load_recovered` | ERROR | `file`, `error`, `preserved_as`, `ms` | A document was unreadable; it was moved to `backups/` and the addon starts without it |
| `load_read_only` | WARN/ERROR | `file`, `reason` (`newer_version`/`io`), `found`, `error`, `ms` | A document must not be overwritten: written by a newer version (WARN) or not accessible (ERROR) |
| `save_finished` | DEBUG | `file`, `ms` | A document was written |
| `save_failed` | ERROR | `file`, `ms`, `error` | Writing a document failed; the change is not on disk |
| `save_dropped` | WARN/ERROR | `reason` (`read_only`/`worker_stopped`) | A change was not saved: read-only document (WARN) or no storage worker (ERROR) |
| `setting_changed` | INFO | `key`, `value` | A setting changed (e.g. `text_scale`) |
| `backup_age_unreadable` | WARN | `file`, `error` | The age of a safety copy cannot be read; it is kept |
| `backup_delete_failed` | WARN | `file`, `error` | An expired safety copy cannot be deleted |
| `folder_unlistable` | WARN | `folder`, `error` | A folder cannot be listed during the safety-copy cleanup |
| `local_time_failed` | WARN | `error` | Windows could not convert to local time; times are shown in UTC |
| `folder_open_failed` | WARN | `error` | Explorer could not be started for the addon folder |
| `link_open_failed` | WARN | `error` | The browser could not be started for a link |
| `font_pixels_missing` | WARN | – | The font atlas pixels are not in memory; glyphs are centered by their quads |

Windows error codes often seen in `error`: `os error 5` access denied, `32`
file in use by another process (antivirus, cloud sync), `112` disk full,
`3` path not found.

## Rules

- Call `start` first in `load` and drop the `LogSession` last in `unload`.
- The logger only formats and queues; it never touches the file, so it is
  safe on the render thread. Still, never log per frame.
- Log file names, never paths; never user content. Add every new event to
  the table above (the catalog test fails otherwise).
- nexus-rs' `log` feature stays off: this crate is the DLL's logger.

## Links

- Decisions: [ROADMAP R89, R92](../../docs/plans/ROADMAP.md)
- Analysing a log: [`trailkit-logs` skill](../../.agents/skills/trailkit-logs/SKILL.md)
- Upstream: [`log` key-values](https://docs.rs/log/latest/log/kv/index.html)
