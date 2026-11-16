//! The state of a button whose action runs (`ix-busy-buttons`, memo M180):
//! busy while the action runs, and at least [`MINIMUM_BUSY`] after the
//! click, so an action that ends at once (e.g. a write that fails in a few
//! milliseconds) is still seen to run. While busy the button shows a
//! spinner (`ButtonState::Busy`) and its action, and the actions
//! that depend on it, are locked. On Windows `BusyState::button_state`
//! (in `button`) turns it into the `ButtonState` a frame draws.

use std::time::{Duration, Instant};

/// How long a busy button shows its spinner at least.
pub const MINIMUM_BUSY: Duration = Duration::from_millis(600);

/// Whether a button's action runs; see the module docs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BusyState {
    /// When the action started; `None` if it never did.
    started_at: Option<Instant>,
    /// Whether the action has not reported its end yet.
    running: bool,
}

impl BusyState {
    /// A state whose action started `at` and runs.
    #[must_use]
    pub const fn running_since(at: Instant) -> Self {
        Self {
            started_at: Some(at),
            running: true,
        }
    }

    /// Starts the action now.
    pub fn start(&mut self) {
        *self = Self::running_since(Instant::now());
    }

    /// The action reported its end; the spinner stays for the rest of
    /// [`MINIMUM_BUSY`].
    pub const fn finish(&mut self) {
        self.running = false;
    }

    /// Whether the button is busy now.
    #[must_use]
    pub fn is_busy(&self) -> bool {
        self.is_busy_at(Instant::now())
    }

    /// Whether the button is busy at `now`.
    #[must_use]
    pub fn is_busy_at(&self, now: Instant) -> bool {
        self.running
            || self
                .started_at
                .is_some_and(|started| now < started + MINIMUM_BUSY)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_action_that_never_started_is_not_busy() {
        assert!(!BusyState::default().is_busy());
    }

    #[test]
    fn a_running_action_is_busy_however_long_it_takes() {
        let start = Instant::now();
        let busy = BusyState::running_since(start);
        assert!(busy.is_busy_at(start + MINIMUM_BUSY * 10));
    }

    #[test]
    fn an_action_that_ends_at_once_stays_busy_for_the_minimum() {
        let start = Instant::now();
        let mut busy = BusyState::running_since(start);
        busy.finish();
        assert!(busy.is_busy_at(start + MINIMUM_BUSY / 2));
        assert!(!busy.is_busy_at(start + MINIMUM_BUSY));
    }
}
