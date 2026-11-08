//! The `log` logger of one addon DLL: formats each record into a line,
//! hands it to the writer thread and forwards WARN and ERROR to `Nexus.log`.

use std::{
    borrow::Cow,
    sync::{Arc, mpsc::Sender},
};

use jiff::Timestamp;
use log::kv::{self, VisitSource};

use crate::{
    events::EVENT_MESSAGE,
    line::{Field, LineContent, TEXT_FIELD, format_body, format_line, is_event_name},
    writer::{Command, Shared},
};

/// Prefix of the module paths of our own crates; records from other crates
/// (dependencies) are kept only from WARN up.
const OWN_TARGET_PREFIX: &str = "trailkit";

/// Most detailed level written: DEBUG in debug builds, INFO in release
/// builds (ROADMAP R92).
pub(crate) const MAX_LEVEL: log::LevelFilter = if cfg!(debug_assertions) {
    log::LevelFilter::Debug
} else {
    log::LevelFilter::Info
};

/// The installed logger.
pub(crate) struct TrailkitLogger {
    /// Identity, Nexus forwarding and the file.
    pub(crate) shared: Arc<Shared>,
    /// Queue of the writer thread.
    pub(crate) commands: Sender<Command>,
}

impl log::Log for TrailkitLogger {
    /// Our crates log up to [`MAX_LEVEL`], dependencies only WARN and ERROR.
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        let limit = if metadata.target().starts_with(OWN_TARGET_PREFIX) {
            MAX_LEVEL
        } else {
            log::LevelFilter::Warn
        };
        metadata.level() <= limit
    }

    /// Formats `record`, queues it for the file and forwards WARN and ERROR
    /// to `Nexus.log`. Runs on whatever thread logged (often the render
    /// thread), so it only formats and sends; it never touches the file.
    fn log(&self, record: &log::Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let message = record
            .args()
            .as_str()
            .map_or_else(|| Cow::Owned(record.args().to_string()), Cow::Borrowed);
        let mut fields = FieldCollector::default();
        // The collector never fails.
        let _ = record.key_values().visit(&mut fields);
        let mut fields = fields.0;
        let event = if is_event_name(&message) {
            message.as_ref()
        } else {
            fields.push((Cow::Borrowed(TEXT_FIELD), Cow::Owned(message.to_string())));
            EVENT_MESSAGE
        };
        let at = format!(
            "{}:{}",
            record.module_path().unwrap_or_else(|| record.target()),
            record.line().unwrap_or(0)
        );
        let content = LineContent {
            level: record.level(),
            event,
            at: &at,
            fields: &fields,
        };
        let line = format_line(&self.shared.identity, Timestamp::now(), &content);
        // After shutdown the writer is gone; the line is then only forwarded.
        let _ = self.commands.send(Command::Line(line));
        if record.level() <= log::Level::Warn {
            (self.shared.forward)(
                record.level(),
                &format_body(&self.shared.identity, &content),
            );
        }
    }

    /// Nothing is buffered: the writer writes every line unbuffered.
    fn flush(&self) {}
}

/// Collects the `key = value` pairs of a record as strings.
#[derive(Default)]
struct FieldCollector(Vec<Field<'static>>);

impl<'kvs> VisitSource<'kvs> for FieldCollector {
    /// Stores one pair.
    fn visit_pair(&mut self, key: kv::Key<'kvs>, value: kv::Value<'kvs>) -> Result<(), kv::Error> {
        self.0.push((
            Cow::Owned(key.as_str().to_owned()),
            Cow::Owned(value.to_string()),
        ));
        Ok(())
    }
}
