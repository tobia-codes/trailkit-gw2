//! Window and pane layout: panes, columns with splitters, stacks,
//! separators, toolbars, page heads, the title bar and the empty state (ROADMAP R134).

#[cfg(windows)]
pub mod empty_state;
#[cfg(windows)]
pub mod page;
#[cfg(windows)]
pub mod pane;
#[cfg(windows)]
pub mod pane_columns;
#[cfg(windows)]
pub mod separator;
#[cfg(windows)]
pub mod splitter;
#[cfg(windows)]
pub mod stack;
#[cfg(windows)]
pub mod title_bar;
#[cfg(windows)]
pub mod toolbar;
