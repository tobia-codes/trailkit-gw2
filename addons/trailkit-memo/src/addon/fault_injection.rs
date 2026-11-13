//! Fault injection for the in-game test of the crash protection (ROADMAP
//! R104, Q365); only built with the Cargo feature `fault-injection`. An
//! unbound keybind asks for a panic, and the next frame that draws memo's
//! windows panics while it holds the notes, like a real bug would.

use std::sync::atomic::{AtomicBool, Ordering};

use nexus::keybind::{keybind_handler, register_keybind_with_string};

use super::{
    UNBOUND,
    guard::{Callback, guard},
};

/// Nexus keybind identifier; it shows untranslated in the Nexus keybind
/// list, because only test builds have it.
const KEYBIND_FAULT: &str = "trailkit.memo.keybind.fault_injection";

/// Set by the keybind; taken by the next drawn frame.
static FAULT_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Registers the unbound fault keybind; Nexus removes it on unload.
pub(super) fn register() {
    let on_fault = keybind_handler!(|_id, is_release| {
        guard(Callback::FaultInjection, (), || {
            if !is_release {
                FAULT_REQUESTED.store(true, Ordering::Relaxed);
            }
        });
    });
    register_keybind_with_string(KEYBIND_FAULT, on_fault, UNBOUND).revert_on_unload();
}

/// Panics once after the keybind was pressed.
///
/// # Panics
/// On purpose, once per key press.
pub(super) fn panic_if_requested() {
    assert!(
        !FAULT_REQUESTED.swap(false, Ordering::Relaxed),
        "fault injection: panic on request"
    );
}
