//! Shift pressed twice opens the list search (memo plan M199, Q511): two taps
//! of Shift alone within [`DOUBLE_SHIFT_WINDOW`], like JetBrains' "Search
//! Everywhere". Any other key in between, or Shift held with another key
//! (capitals), starts over. Fed from the window message callback.

use std::time::{Duration, Instant};

/// The longest time between the first tap's release and the second press.
pub const DOUBLE_SHIFT_WINDOW: Duration = Duration::from_millis(400);

/// Follows Shift taps across key messages.
#[derive(Debug, Default)]
pub struct DoubleShift {
    /// Shift is down and no other key was pressed since.
    pressed_alone: bool,
    /// When the last clean tap (press and release alone) ended.
    last_tap: Option<Instant>,
}

impl DoubleShift {
    /// No tap seen yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pressed_alone: false,
            last_tap: None,
        }
    }

    /// Shift went down at `now`; `is_repeat` marks the keyboard's auto
    /// repeat while it is held. Returns whether this completes a double tap.
    pub fn shift_down(&mut self, now: Instant, is_repeat: bool) -> bool {
        if is_repeat {
            return false;
        }
        let is_second = self
            .last_tap
            .take()
            .is_some_and(|tap| now.duration_since(tap) <= DOUBLE_SHIFT_WINDOW);
        self.pressed_alone = !is_second;
        is_second
    }

    /// Shift went up at `now`: a tap alone counts as the first of two.
    pub fn shift_up(&mut self, now: Instant) {
        if std::mem::take(&mut self.pressed_alone) {
            self.last_tap = Some(now);
        }
    }

    /// Another key went down: the taps start over.
    pub fn other_key(&mut self) {
        self.pressed_alone = false;
        self.last_tap = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Taps Shift once at `start + at_ms`, held for 50 ms; returns whether the
    /// press completed a double tap.
    fn tap(shift: &mut DoubleShift, start: Instant, at_ms: u64) -> bool {
        let down = start + Duration::from_millis(at_ms);
        let completed = shift.shift_down(down, false);
        shift.shift_up(down + Duration::from_millis(50));
        completed
    }

    #[test]
    fn two_quick_taps_count() {
        let (mut shift, start) = (DoubleShift::default(), Instant::now());
        assert!(!tap(&mut shift, start, 0));
        assert!(tap(&mut shift, start, 300));
        assert!(!tap(&mut shift, start, 600), "a third tap starts over");
    }

    #[test]
    fn slow_taps_do_not_count() {
        let (mut shift, start) = (DoubleShift::default(), Instant::now());
        assert!(!tap(&mut shift, start, 0));
        assert!(!tap(&mut shift, start, 500));
    }

    #[test]
    fn another_key_in_between_starts_over() {
        let (mut shift, start) = (DoubleShift::default(), Instant::now());
        assert!(!tap(&mut shift, start, 0));
        shift.other_key();
        assert!(!tap(&mut shift, start, 200));
    }

    #[test]
    fn capitals_do_not_count() {
        let (mut shift, start) = (DoubleShift::default(), Instant::now());
        // Shift+A, then Shift+B: Shift is never tapped alone.
        for at in [0, 200] {
            let down = start + Duration::from_millis(at);
            assert!(!shift.shift_down(down, false));
            shift.other_key();
            shift.shift_up(down + Duration::from_millis(80));
        }
    }

    #[test]
    fn held_shift_repeats_are_ignored() {
        let (mut shift, start) = (DoubleShift::default(), Instant::now());
        assert!(!shift.shift_down(start, false));
        assert!(!shift.shift_down(start + Duration::from_millis(30), true));
        shift.shift_up(start + Duration::from_millis(60));
        assert!(shift.shift_down(start + Duration::from_millis(200), false));
    }
}
