//! Memo's connection to `trailkit-log` (ROADMAP R92): starts the log first in
//! `load`, stops it last in `unload` and sends texts to `Nexus.log`.

use std::sync::Mutex;

use nexus::log::{LogLevel, log as nexus_log};
use trailkit_log::{ErrorChain, LogConfig, LogSession};
use trailkit_memo_shared::i18n::{Language, local_utc_offset};
use trailkit_storage::TrailkitDirectory;
use trailkit_ui::theme::ThemeKind;

use super::ADDON;

/// Channel name of memo's lines in `Nexus.log` (the name in `nexus::export!`).
const NEXUS_LOG_CHANNEL: &str = "trailkit-memo";

/// The running log; only touched in `load` and `unload`.
static LOG_SESSION: Mutex<Option<LogSession>> = Mutex::new(None);

/// Starts memo's log file (ROADMAP R92). Without it the addon still works,
/// but nothing is logged; the failure itself is sent to Nexus directly.
pub(super) fn start_logging(directory: TrailkitDirectory, language: Language) {
    let config = LogConfig {
        directory,
        addon: ADDON,
        version: env!("CARGO_PKG_VERSION"),
        local_offset: local_utc_offset,
        forward: forward_to_nexus,
        session_fields: vec![
            ("language", language.code().to_owned()),
            ("theme", ThemeKind::default().code().to_owned()),
        ],
    };
    match trailkit_log::start(config) {
        Ok(session) => {
            if let Ok(mut slot) = LOG_SESSION.lock() {
                *slot = Some(session);
            }
        }
        Err(error) => forward_to_nexus(
            log::Level::Warn,
            &format!("log_unavailable error=\"{}\"", ErrorChain(&error)),
        ),
    }
}

/// Writes `text` to `Nexus.log` in memo's channel.
pub(super) fn forward_to_nexus(level: log::Level, text: &str) {
    let level = match level {
        log::Level::Error => LogLevel::Critical,
        log::Level::Warn => LogLevel::Warning,
        log::Level::Info => LogLevel::Info,
        log::Level::Debug => LogLevel::Debug,
        log::Level::Trace => LogLevel::Trace,
    };
    nexus_log(level, NEXUS_LOG_CHANNEL, text);
}

/// Writes `session_end` and stops the log. Call it last in `unload`, so every
/// earlier line still reaches the file.
pub(super) fn stop_logging() {
    let log_session = LOG_SESSION
        .lock()
        .map_or(None, |mut session| session.take());
    drop(log_session);
}
