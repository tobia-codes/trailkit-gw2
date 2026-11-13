//! The events this crate writes itself; every other event comes from a
//! `log::` call in an addon or a shared crate. All events are listed in the
//! README's event table (checked by `tests/event_catalog.rs`).

/// First line of a session: build, commit, Windows version and the fields
/// the addon adds (language, theme).
pub(crate) const EVENT_SESSION_START: &str = "session_start";

/// First line of a new day's file while a session keeps running: the same
/// fields as `session_start`, so one day's file can be read on its own.
pub(crate) const EVENT_SESSION_CONTINUES: &str = "session_continues";

/// Last line of a session, written when the addon unloads cleanly.
pub(crate) const EVENT_SESSION_END: &str = "session_end";

/// The previous session has no `session_end`: the game or the addon crashed.
pub(crate) const EVENT_PREVIOUS_SESSION_UNCLEAN: &str = "previous_session_unclean";

/// The day's size cap was reached; nothing more is written until midnight
/// except the critical events.
pub(crate) const EVENT_LOG_FULL: &str = "log_full";

/// The log files cannot be prepared or written; this line only reaches
/// `Nexus.log`. After `prepare` nothing is logged in this session, after
/// `write` the next line tries again.
pub(crate) const EVENT_LOG_UNAVAILABLE: &str = "log_unavailable";

/// A housekeeping step at start failed (deleting old files, reading how the
/// previous session ended); logging goes on.
pub(crate) const EVENT_LOG_MAINTENANCE_FAILED: &str = "log_maintenance_failed";

/// A Rust panic, written by the panic hook.
pub(crate) const EVENT_PANIC: &str = "panic";

/// A log message that is not an event name (e.g. from a dependency); the
/// message is in the `text` field.
pub(crate) const EVENT_MESSAGE: &str = "message";

/// Every event above, for the catalog test.
pub const BUILT_IN_EVENTS: [&str; 9] = [
    EVENT_SESSION_START,
    EVENT_SESSION_CONTINUES,
    EVENT_SESSION_END,
    EVENT_PREVIOUS_SESSION_UNCLEAN,
    EVENT_LOG_FULL,
    EVENT_LOG_UNAVAILABLE,
    EVENT_LOG_MAINTENANCE_FAILED,
    EVENT_PANIC,
    EVENT_MESSAGE,
];

/// Events that may use the reserve at the end of a day's last file, so a
/// crash and the limits of a session are logged even when the day is full
/// (ROADMAP R92). `worker_died` and `save_gave_up` (a change that is not on
/// disk, memo M180) are written by the addons' storage worker,
/// `addon_stopped` by an addon's crash protection (R104).
const CRITICAL_EVENTS: [&str; 7] = [
    EVENT_PANIC,
    EVENT_SESSION_START,
    EVENT_SESSION_END,
    EVENT_LOG_FULL,
    "worker_died",
    "save_gave_up",
    "addon_stopped",
];

/// Whether `event` may use the reserve of a full day.
pub(crate) fn is_critical(event: &str) -> bool {
    CRITICAL_EVENTS.contains(&event)
}
