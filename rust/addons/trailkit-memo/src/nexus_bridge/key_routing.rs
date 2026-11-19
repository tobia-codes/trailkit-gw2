//! Decides what a key message means for memo (M79, M135, M174, M175, M198,
//! M199): which
//! keys belong to the window right now and which of them the game must not
//! get. Free of global state, so every rule can be tested; the window message
//! callback in `nexus_bridge` applies the result.

use std::time::Instant;

use windows::Win32::UI::{
    Input::KeyboardAndMouse::{
        VIRTUAL_KEY, VK_0, VK_CONTROL, VK_DELETE, VK_F, VK_F3, VK_NUMPAD0, VK_RETURN, VK_SHIFT,
        VK_Y, VK_Z,
    },
    WindowsAndMessaging::{WM_KEYDOWN, WM_KEYUP},
};

use super::double_shift::DoubleShift;

/// One key message of the game window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct KeyMessage {
    /// The window message, e.g. `WM_KEYDOWN`.
    pub(super) message: u32,
    /// The virtual-key code.
    pub(super) key: usize,
    /// Whether the key was already down (auto repeat).
    pub(super) is_repeat: bool,
}

/// Which keys belong to the window in this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ArmedKeys {
    /// Shift twice toggles the list search (M199).
    pub(super) search: bool,
    /// Ctrl+F toggles find in the note (M198).
    pub(super) find: bool,
    /// F3 steps through the find hits (M198).
    pub(super) find_step: bool,
    /// Delete moves the selected entry to the trash (M175).
    pub(super) delete: bool,
    /// Ctrl+Z undoes, Ctrl+Y and Ctrl+Shift+Z redo (M135).
    pub(super) undo: bool,
    /// Ctrl+0 resets the text size (M79).
    pub(super) text_size_reset: bool,
    /// Enter answers the open dialog (M56, M152, M180).
    pub(super) dialog_enter: bool,
}

/// What a key message asks the window to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum KeySignal {
    /// Toggle the list search.
    ToggleSearch,
    /// Toggle find in the note.
    ToggleFind,
    /// Go to the next find hit.
    FindNext,
    /// Go to the previous find hit.
    FindPrevious,
    /// Delete the selected entry.
    Delete,
    /// Take the latest step back.
    Undo,
    /// Do the latest undone step again.
    Redo,
    /// Reset the text size.
    ResetTextSize,
    /// Answer the open dialog as Enter does.
    ConfirmDialog,
}

impl KeySignal {
    /// Whether the game must not get the key. Shift stays with the game.
    pub(super) const fn consumes_key(self) -> bool {
        !matches!(self, Self::ToggleSearch)
    }
}

/// Follows `message` at `now` and returns what it asks for, if anything.
/// `is_down` tells whether a modifier is held; it is only asked for key
/// presses that no other rule took.
pub(super) fn route_key(
    message: KeyMessage,
    armed: ArmedKeys,
    now: Instant,
    double_shift: &mut DoubleShift,
    is_down: impl Fn(VIRTUAL_KEY) -> bool,
) -> Option<KeySignal> {
    let is_key = |virtual_key: VIRTUAL_KEY| message.key == usize::from(virtual_key.0);
    if message.message == WM_KEYUP && is_key(VK_SHIFT) {
        double_shift.shift_up(now);
        return None;
    }
    if message.message != WM_KEYDOWN {
        return None;
    }
    if is_key(VK_SHIFT) {
        let completed = double_shift.shift_down(now, message.is_repeat);
        return (completed && armed.search).then_some(KeySignal::ToggleSearch);
    }
    double_shift.other_key();
    if is_key(VK_DELETE) && armed.delete {
        Some(KeySignal::Delete)
    } else if is_key(VK_RETURN) && armed.dialog_enter {
        Some(KeySignal::ConfirmDialog)
    } else if is_key(VK_F3) && armed.find_step {
        Some(if is_down(VK_SHIFT) {
            KeySignal::FindPrevious
        } else {
            KeySignal::FindNext
        })
    } else if is_down(VK_CONTROL) {
        route_ctrl_shortcut(message.key, armed, is_down(VK_SHIFT))
    } else {
        None
    }
}

/// What `key` pressed with Ctrl asks for; `shift` tells whether Shift is
/// held too.
fn route_ctrl_shortcut(key: usize, armed: ArmedKeys, shift: bool) -> Option<KeySignal> {
    let is_key = |virtual_key: VIRTUAL_KEY| key == usize::from(virtual_key.0);
    if is_key(VK_F) && armed.find {
        Some(KeySignal::ToggleFind)
    } else if is_key(VK_Z) && armed.undo {
        Some(if shift {
            KeySignal::Redo
        } else {
            KeySignal::Undo
        })
    } else if is_key(VK_Y) && armed.undo {
        Some(KeySignal::Redo)
    } else if (is_key(VK_0) || is_key(VK_NUMPAD0)) && armed.text_size_reset {
        Some(KeySignal::ResetTextSize)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    /// Every key belongs to the window.
    const ALL_ARMED: ArmedKeys = ArmedKeys {
        search: true,
        find: true,
        find_step: true,
        delete: true,
        undo: true,
        text_size_reset: true,
        dialog_enter: true,
    };

    /// A key message without auto repeat.
    fn key_message(message: u32, key: VIRTUAL_KEY) -> KeyMessage {
        KeyMessage {
            message,
            key: usize::from(key.0),
            is_repeat: false,
        }
    }

    /// Routes a key press of `key` while the keys in `held` are down.
    fn press(key: VIRTUAL_KEY, armed: ArmedKeys, held: &[VIRTUAL_KEY]) -> Option<KeySignal> {
        route_key(
            key_message(WM_KEYDOWN, key),
            armed,
            Instant::now(),
            &mut DoubleShift::new(),
            |modifier| held.contains(&modifier),
        )
    }

    #[test]
    fn ctrl_f_toggles_find_only_with_ctrl_and_while_armed() {
        let signal = press(VK_F, ALL_ARMED, &[VK_CONTROL]);
        assert_eq!(signal, Some(KeySignal::ToggleFind));
        assert!(signal.is_some_and(KeySignal::consumes_key));
        assert_eq!(press(VK_F, ALL_ARMED, &[]), None, "a plain F is typing");
        assert_eq!(press(VK_F, ArmedKeys::default(), &[VK_CONTROL]), None);
    }

    #[test]
    fn f3_steps_and_shift_reverses_it() {
        assert_eq!(press(VK_F3, ALL_ARMED, &[]), Some(KeySignal::FindNext));
        assert_eq!(
            press(VK_F3, ALL_ARMED, &[VK_SHIFT]),
            Some(KeySignal::FindPrevious)
        );
        assert_eq!(press(VK_F3, ArmedKeys::default(), &[]), None);
    }

    #[test]
    fn delete_belongs_to_the_window_only_while_armed() {
        assert_eq!(press(VK_DELETE, ALL_ARMED, &[]), Some(KeySignal::Delete));
        assert_eq!(press(VK_DELETE, ArmedKeys::default(), &[]), None);
    }

    #[test]
    fn ctrl_z_undoes_and_with_shift_redoes_while_armed() {
        let signal = press(VK_Z, ALL_ARMED, &[VK_CONTROL]);
        assert_eq!(signal, Some(KeySignal::Undo));
        assert!(signal.is_some_and(KeySignal::consumes_key));
        assert_eq!(
            press(VK_Z, ALL_ARMED, &[VK_CONTROL, VK_SHIFT]),
            Some(KeySignal::Redo)
        );
        assert_eq!(press(VK_Z, ALL_ARMED, &[]), None, "a plain Z is typing");
        assert_eq!(press(VK_Z, ArmedKeys::default(), &[VK_CONTROL]), None);
    }

    #[test]
    fn ctrl_y_redoes_while_armed() {
        let signal = press(VK_Y, ALL_ARMED, &[VK_CONTROL]);
        assert_eq!(signal, Some(KeySignal::Redo));
        assert!(signal.is_some_and(KeySignal::consumes_key));
        assert_eq!(press(VK_Y, ALL_ARMED, &[]), None);
        assert_eq!(press(VK_Y, ArmedKeys::default(), &[VK_CONTROL]), None);
    }

    #[test]
    fn ctrl_0_resets_the_text_size_while_armed() {
        let signal = press(VK_0, ALL_ARMED, &[VK_CONTROL]);
        assert_eq!(signal, Some(KeySignal::ResetTextSize));
        assert!(signal.is_some_and(KeySignal::consumes_key));
        assert_eq!(
            press(VK_NUMPAD0, ALL_ARMED, &[VK_CONTROL]),
            Some(KeySignal::ResetTextSize),
            "the number pad's 0 too"
        );
        assert_eq!(press(VK_0, ALL_ARMED, &[]), None);
        assert_eq!(press(VK_0, ArmedKeys::default(), &[VK_CONTROL]), None);
    }

    #[test]
    fn enter_answers_the_dialog_only_while_armed() {
        let signal = press(VK_RETURN, ALL_ARMED, &[]);
        assert_eq!(signal, Some(KeySignal::ConfirmDialog));
        assert!(signal.is_some_and(KeySignal::consumes_key));
        assert_eq!(press(VK_RETURN, ArmedKeys::default(), &[]), None);
    }

    /// Taps Shift at `start + at_ms` and returns what its press asked for.
    fn tap_shift(
        double_shift: &mut DoubleShift,
        armed: ArmedKeys,
        start: Instant,
        at_ms: u64,
    ) -> Option<KeySignal> {
        let now = start + Duration::from_millis(at_ms);
        let signal = route_key(
            key_message(WM_KEYDOWN, VK_SHIFT),
            armed,
            now,
            double_shift,
            |_| false,
        );
        route_key(
            key_message(WM_KEYUP, VK_SHIFT),
            armed,
            now + Duration::from_millis(30),
            double_shift,
            |_| false,
        );
        signal
    }

    #[test]
    fn shift_twice_toggles_the_search_and_stays_with_the_game() {
        let (mut double_shift, start) = (DoubleShift::new(), Instant::now());
        assert_eq!(tap_shift(&mut double_shift, ALL_ARMED, start, 0), None);
        let signal = tap_shift(&mut double_shift, ALL_ARMED, start, 200);
        assert_eq!(signal, Some(KeySignal::ToggleSearch));
        assert!(signal.is_some_and(|signal| !signal.consumes_key()));
    }

    #[test]
    fn shift_twice_does_nothing_while_the_game_has_the_focus() {
        let (mut double_shift, start) = (DoubleShift::new(), Instant::now());
        let armed = ArmedKeys::default();
        assert_eq!(tap_shift(&mut double_shift, armed, start, 0), None);
        assert_eq!(tap_shift(&mut double_shift, armed, start, 200), None);
    }

    #[test]
    fn another_key_between_the_shift_taps_starts_over() {
        let (mut double_shift, start) = (DoubleShift::new(), Instant::now());
        tap_shift(&mut double_shift, ALL_ARMED, start, 0);
        route_key(
            key_message(WM_KEYDOWN, VK_F),
            ALL_ARMED,
            start + Duration::from_millis(100),
            &mut double_shift,
            |_| false,
        );
        assert_eq!(tap_shift(&mut double_shift, ALL_ARMED, start, 200), None);
    }
}
