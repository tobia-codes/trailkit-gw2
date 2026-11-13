//! Crash protection (ROADMAP R104): every Nexus callback runs inside
//! [`guard`], so a panic never unwinds into Nexus and the game.
//!
//! The first caught panic stops memo for the rest of the session (Q359 A):
//! unsaved editor text becomes a draft (Q360), Nexus shows one alert (Q364)
//! and from then on the render callback only draws the shared help window,
//! opened once on "Report a problem" with a warning card (Q366, Q367). A
//! panic while memo already runs in that mode, or one in `load`, stops it
//! completely. A reload in the Nexus addon list or a game restart frees the
//! DLL and starts memo fresh, so nothing here is persisted (Q369).
//!
//! Like `catch_unwind` in Node's `try`/`catch` around a callback: the panic
//! hook of `trailkit-log` has already written the `panic` line when the
//! panic reaches [`guard`].

use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Mutex, MutexGuard, TryLockError,
        atomic::{AtomicBool, AtomicU8, Ordering},
    },
};

use nexus::alert::send_alert;

use trailkit_memo_shared::i18n::{Language, LocalizedText as _};

use super::{DELETE_KEY_ARMED, HELP, LANGUAGE, NOTES, NotesState, SEARCH_KEY_ARMED, VISIBLE};
use crate::help_text::HelpText;

/// How far memo still runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub(super) enum RunState {
    /// Normal operation.
    Running = 0,
    /// Stopped after an error; the render callback only draws the help
    /// window, keybind and quick access icon open it (Q366).
    HelpOnly = 1,
    /// Stopped completely: every callback returns at once.
    Stopped = 2,
}

impl RunState {
    /// The state stored as `value` in [`RUN_STATE`].
    const fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Running,
            1 => Self::HelpOnly,
            _ => Self::Stopped,
        }
    }

    /// The `mode` field of the `addon_stopped` log line.
    const fn log_name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::HelpOnly => "help_only",
            Self::Stopped => "stopped",
        }
    }
}

/// The Nexus callback a panic came from; the `callback` field of the
/// `addon_stopped` log line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Callback {
    /// `load`.
    Load,
    /// `unload`.
    Unload,
    /// The `RT_Render` callback (main and help window).
    Render,
    /// The `RT_OptionsRender` callback (memo's tab in the Nexus options).
    OptionsRender,
    /// The open/close keybind, also fired by the quick access icon.
    Keybind,
    /// The `WndProc` callback that catches memo's keys and mouse buttons.
    WindowMessages,
    /// The fault injection keybind (feature `fault-injection`, Q365).
    #[cfg(feature = "fault-injection")]
    FaultInjection,
}

impl Callback {
    /// The name in the log.
    const fn log_name(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Unload => "unload",
            Self::Render => "render",
            Self::OptionsRender => "options_render",
            Self::Keybind => "keybind",
            Self::WindowMessages => "window_messages",
            #[cfg(feature = "fault-injection")]
            Self::FaultInjection => "fault_injection",
        }
    }
}

/// The current [`RunState`] as `u8`; written once per panic, read by every
/// callback.
static RUN_STATE: AtomicU8 = AtomicU8::new(RunState::Running as u8);

/// Whether a draft was kept when memo stopped; picks the card's detail line.
static DRAFT_KEPT: AtomicBool = AtomicBool::new(false);

/// Set by the keybind (and the panic itself) while memo runs help-only: the
/// render callback opens the help window on "Report a problem" (Q366).
static HELP_REQUESTED: AtomicBool = AtomicBool::new(false);

/// The current run state.
pub(super) fn run_state() -> RunState {
    RunState::from_u8(RUN_STATE.load(Ordering::Acquire))
}

/// Whether a draft was kept when memo stopped.
pub(super) fn draft_kept() -> bool {
    DRAFT_KEPT.load(Ordering::Relaxed)
}

/// Asks the render callback to open the help window on "Report a problem";
/// only touches an atomic, so the keybind thread may call it.
pub(super) fn request_help() {
    HELP_REQUESTED.store(true, Ordering::Relaxed);
}

/// Takes a pending request from [`request_help`].
pub(super) fn take_help_request() -> bool {
    HELP_REQUESTED.swap(false, Ordering::Relaxed)
}

/// Runs `body` for the Nexus callback `callback` and returns its result, or
/// `fallback` if it panicked. A panic stops memo (see the module docs);
/// nothing unwinds past this function.
pub(super) fn guard<T>(callback: Callback, fallback: T, body: impl FnOnce() -> T) -> T {
    // `AssertUnwindSafe`: after a panic memo never uses the state `body`
    // touched as if nothing happened; it only rescues the draft and draws
    // the help window, both of which tolerate a half-updated state.
    catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|_| {
        // The stop itself must not panic into Nexus either.
        if catch_unwind(|| stop_after_panic(callback)).is_err() {
            RUN_STATE.store(RunState::Stopped as u8, Ordering::Release);
        }
        fallback
    })
}

/// Locks `mutex` without waiting, also when a panic poisoned it (the data
/// is still there; a half-updated state is acceptable after a panic).
/// Returns `None` while someone else holds it.
pub(super) fn try_lock_after_panic<T>(mutex: &Mutex<T>) -> Option<MutexGuard<'_, T>> {
    match mutex.try_lock() {
        Ok(guard) => Some(guard),
        Err(TryLockError::Poisoned(poisoned)) => Some(poisoned.into_inner()),
        Err(TryLockError::WouldBlock) => None,
    }
}

/// The state after a panic in `callback` while memo is in `current`: from
/// running to help-only, or to stopped for a panic in `load` or without a
/// help window to draw; from help-only to stopped. `None` if memo already
/// stopped completely.
fn state_after_panic(current: RunState, callback: Callback, help_exists: bool) -> Option<RunState> {
    match (current, callback) {
        (RunState::Stopped, _) => None,
        (RunState::Running, Callback::Load) | (RunState::HelpOnly, _) => Some(RunState::Stopped),
        (RunState::Running, _) if help_exists => Some(RunState::HelpOnly),
        (RunState::Running, _) => Some(RunState::Stopped),
    }
}

/// Moves memo one step down after a panic in `callback` (see
/// [`state_after_panic`]); on the first stop it rescues the draft, logs
/// `addon_stopped` and sends the Nexus alert.
fn stop_after_panic(callback: Callback) {
    let current = run_state();
    let Some(next) = state_after_panic(current, callback, help_exists()) else {
        return;
    };
    RUN_STATE.store(next as u8, Ordering::Release);
    // The game gets every key again (M174, M175).
    SEARCH_KEY_ARMED.store(false, Ordering::Relaxed);
    DELETE_KEY_ARMED.store(false, Ordering::Relaxed);
    VISIBLE.store(false, Ordering::Relaxed);
    if current != RunState::Running {
        log::error!(callback = callback.log_name(), mode = next.log_name(); "addon_stopped");
        return;
    }
    let draft = rescue_draft();
    DRAFT_KEPT.store(draft == DraftRescue::Kept, Ordering::Relaxed);
    log::error!(
        callback = callback.log_name(),
        mode = next.log_name(),
        draft = draft.log_name();
        "addon_stopped"
    );
    let alert = if next == RunState::HelpOnly {
        request_help();
        HelpText::StoppedAlert
    } else {
        HelpText::StoppedWhileLoadingAlert
    };
    send_alert(alert.get(Language::from_u8(LANGUAGE.load(Ordering::Relaxed))));
}

/// What happened to the unsaved editor text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DraftRescue {
    /// It became a draft.
    Kept,
    /// There was nothing to keep (no editor, no change, no notes).
    Nothing,
    /// The notes were locked elsewhere or keeping it panicked.
    Failed,
}

impl DraftRescue {
    /// The `draft` field of the `addon_stopped` log line.
    const fn log_name(self) -> &'static str {
        match self {
            Self::Kept => "kept",
            Self::Nothing => "none",
            Self::Failed => "failed",
        }
    }
}

/// Keeps the unsaved editor text as a draft like on unload (Q360). Never
/// waits for the lock, so it is safe on every Nexus thread.
fn rescue_draft() -> DraftRescue {
    catch_unwind(|| {
        let Some(mut notes) = try_lock_after_panic(&NOTES) else {
            return DraftRescue::Failed;
        };
        match &mut *notes {
            NotesState::Ready(window) => {
                if window.keep_draft_for_unload() {
                    DraftRescue::Kept
                } else {
                    DraftRescue::Nothing
                }
            }
            NotesState::Loading { .. } | NotesState::Unavailable => DraftRescue::Nothing,
        }
    })
    .unwrap_or(DraftRescue::Failed)
}

/// Whether the help window exists (`load` creates it), so the alert can
/// point to it.
fn help_exists() -> bool {
    try_lock_after_panic(&HELP).is_some_and(|help| help.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panic_moves_memo_one_step_down() {
        let help_only = Some(RunState::HelpOnly);
        let stopped = Some(RunState::Stopped);
        assert_eq!(
            state_after_panic(RunState::Running, Callback::Render, true),
            help_only
        );
        assert_eq!(
            state_after_panic(RunState::Running, Callback::Keybind, true),
            help_only
        );
        assert_eq!(
            state_after_panic(RunState::Running, Callback::Render, false),
            stopped
        );
        assert_eq!(
            state_after_panic(RunState::Running, Callback::Load, true),
            stopped
        );
        assert_eq!(
            state_after_panic(RunState::HelpOnly, Callback::Render, true),
            stopped
        );
        assert_eq!(
            state_after_panic(RunState::Stopped, Callback::Render, true),
            None
        );
    }

    #[test]
    fn guard_returns_the_result_or_the_fallback_and_never_unwinds() {
        assert_eq!(guard(Callback::Render, 0, || 7), 7);
        // Without Nexus the alert itself panics; the guard catches that too.
        let caught = guard(Callback::Render, 1, || -> i32 { panic!("test panic") });
        assert_eq!(caught, 1);
        assert_ne!(run_state(), RunState::Running);
        assert_eq!(guard(Callback::Render, 2, || -> i32 { panic!("again") }), 2);
        assert_eq!(run_state(), RunState::Stopped);
    }

    #[test]
    fn a_poisoned_lock_still_opens() {
        let mutex = Mutex::new(5);
        let poisoned = catch_unwind(|| {
            let _held = mutex.lock();
            panic!("poison");
        });
        assert!(poisoned.is_err() && mutex.is_poisoned());
        assert_eq!(try_lock_after_panic(&mutex).map(|value| *value), Some(5));
    }
}
