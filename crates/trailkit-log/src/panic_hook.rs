//! Writes a `panic` line before a thread dies (ROADMAP R92), then runs the
//! hook that was installed before (nexus-rs' hook, which logs to `Nexus.log`
//! and may show a message box).

use std::{
    borrow::Cow,
    panic::{self, PanicHookInfo},
    path::Path,
    sync::{Arc, TryLockError, mpsc::Sender},
    thread,
};

use jiff::Timestamp;

use crate::{
    events::EVENT_PANIC,
    line::{LineContent, format_line},
    writer::{Command, Shared},
};

/// Installs the hook in front of the current one.
pub(crate) fn install(shared: Arc<Shared>, commands: Sender<Command>) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let line = panic_line(&shared, info);
        // A panic in a DLL callback usually ends the process right after the
        // hook, so the line is written here and not left in the queue. If the
        // file is busy (the panic happened while writing), the queue is the
        // only option left.
        let written = match shared.file.try_lock() {
            Ok(mut file) if file.is_open() => {
                file.write_line(&line, &shared.identity, shared.forward);
                true
            }
            Err(TryLockError::Poisoned(poisoned)) if poisoned.get_ref().is_open() => {
                poisoned
                    .into_inner()
                    .write_line(&line, &shared.identity, shared.forward);
                true
            }
            _ => false,
        };
        if !written {
            let _ = commands.send(Command::Line(line));
        }
        previous(info);
    }));
}

/// The `panic` line: message, thread and location (file name only, because a
/// dependency's location holds the build machine's folders).
fn panic_line(shared: &Shared, info: &PanicHookInfo<'_>) -> String {
    let message = info
        .payload()
        .downcast_ref::<&str>()
        .map(|text| Cow::Borrowed(*text))
        .or_else(|| {
            info.payload()
                .downcast_ref::<String>()
                .map(|text| Cow::Borrowed(text.as_str()))
        })
        .unwrap_or(Cow::Borrowed("(no message)"));
    let at = info.location().map_or_else(
        || "unknown".to_owned(),
        |location| {
            let file = Path::new(location.file())
                .file_name()
                .map_or_else(|| location.file().into(), |name| name.to_string_lossy());
            format!("{file}:{}", location.line())
        },
    );
    let thread_name = thread::current().name().unwrap_or("unnamed").to_owned();
    let fields = [
        (Cow::Borrowed("thread"), Cow::Owned(thread_name)),
        (Cow::Borrowed("message"), message),
    ];
    format_line(
        &shared.identity,
        Timestamp::now(),
        &LineContent {
            level: log::Level::Error,
            event: EVENT_PANIC,
            at: &at,
            fields: &fields,
        },
    )
}
