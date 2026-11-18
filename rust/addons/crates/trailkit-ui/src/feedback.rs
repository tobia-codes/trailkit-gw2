//! Feedback to the user: dialog cards, toasts, status cards and status lines
//! (ROADMAP R134).

#[cfg(windows)]
pub mod dialog_frame;
#[cfg(windows)]
pub mod status_card;
#[cfg(windows)]
pub mod status_line;
#[cfg(windows)]
pub mod toast;
