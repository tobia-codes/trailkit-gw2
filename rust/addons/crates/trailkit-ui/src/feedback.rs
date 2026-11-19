//! Feedback to the user: dialogs and their backdrop, toasts, status cards
//! and status lines (ROADMAP R134).

#[cfg(windows)]
pub mod backdrop;
#[cfg(windows)]
pub mod dialog;
#[cfg(windows)]
pub mod status_card;
#[cfg(windows)]
pub mod status_line;
#[cfg(windows)]
pub mod toast;
