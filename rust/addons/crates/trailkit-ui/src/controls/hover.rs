//! Whether the mouse is over the last item, also when another child window
//! of the same window lies on top of it.
//!
//! ImGui 1.80's `IsItemHovered` only compares root windows (`imgui.cpp`
//! `IsItemHovered`, "testing RootWindow instead"), so an item under an
//! overlay child of its own window, such as memo's window menu, still counts
//! as hovered and shows its tooltip or hover fill. Clicks are not affected:
//! ImGui's buttons test the hovered window itself. Every hover check goes
//! through this module; clippy bans the plain ImGui calls (see `clippy.toml`).

use nexus::imgui::{ItemHoveredFlags, Ui, WindowHoveredFlags};

/// Whether the mouse is over the last item and no other window lies on top
/// of it at the mouse.
#[must_use]
pub fn is_item_hovered(ui: &Ui) -> bool {
    is_item_hovered_with_flags(ui, ItemHoveredFlags::empty())
}

/// [`is_item_hovered`] with ImGui's `flags`, e.g. `ALLOW_WHEN_DISABLED`.
/// `ALLOW_WHEN_OVERLAPPED` keeps ImGui's own answer.
#[must_use]
pub fn is_item_hovered_with_flags(ui: &Ui, flags: ItemHoveredFlags) -> bool {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one place that asks ImGui; it adds the window check"
    )]
    let item_hovered = ui.is_item_hovered_with_flags(flags);
    item_hovered
        && (flags.contains(ItemHoveredFlags::ALLOW_WHEN_OVERLAPPED)
            || is_current_window_under_mouse(ui))
}

/// Whether the window being drawn is the one under the mouse, not a child
/// window on top of it. Active items and popups are left to the item check.
fn is_current_window_under_mouse(ui: &Ui) -> bool {
    ui.is_window_hovered_with_flags(
        WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM
            | WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_POPUP,
    )
}
