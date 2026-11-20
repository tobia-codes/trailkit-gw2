//! The shared trailkit fonts while drawing (Windows only; memo plan M35, M36):
//! which `ImFont` each [`FontRole`] uses, and [`FontScope`] to draw with it.
//! The fonts are registered with the host by the host's bridge (for Nexus:
//! `trailkit-nexus-bridge`, Q496), which hands every delivered font to
//! [`install_font`]; `trailkit-ui` itself never calls the Nexus API (R136).

use std::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

use nexus::imgui::{Ui, sys};

use crate::text::fonts::FontRole;

/// The `ImFont` pointer the host last delivered for each [`FontRole`] (index =
/// position in [`FontRole::ALL`]); null until the font is ready.
static FONT_POINTERS: [AtomicPtr<sys::ImFont>; FontRole::ALL.len()] =
    [const { AtomicPtr::new(ptr::null_mut()) }; FontRole::ALL.len()];

/// Reads the size of an ImGui font, e.g. Nexus' `FontUI`; `None` if null.
///
/// # Safety
///
/// `font` must be null or point to a live `ImFont` of the current ImGui
/// context (true for the pointers in Nexus' link data while Nexus runs).
#[must_use]
pub unsafe fn font_size_of(font: *const sys::ImFont) -> Option<f32> {
    // SAFETY: guaranteed by the caller; only a plain `f32` field is read.
    unsafe { font.as_ref() }.map(|font| font.FontSize)
}

/// Uses `font` for `role`: called by the host's bridge whenever the host
/// delivers a font (`trailkit-nexus-bridge`), by the headless test harness
/// (`trailkit-ui-testing`, memo plan M65) or by any host that builds its own
/// font atlas. A null pointer makes [`FontScope`] fall back to the current
/// font.
///
/// # Safety
///
/// `font` must be null or point to an `ImFont` of the current ImGui context's
/// atlas, and stay valid until it is replaced (or the context is destroyed
/// after the last frame that used it).
pub unsafe fn install_font(role: FontRole, font: *mut sys::ImFont) {
    if let Some(index) = FontRole::ALL
        .iter()
        .position(|candidate| *candidate == role)
    {
        FONT_POINTERS[index].store(font, Ordering::Release);
    }
}

/// Pushes the font of a [`FontRole`] onto ImGui's font stack and pops it on
/// drop. If the font is not ready yet, nothing is pushed and the current font
/// is used.
///
/// The icon font is sized by the text it accompanies (R147): pushed while a
/// text font is current, its icons are that font's size × `font.size.icon`.
/// It is registered once at the largest size and scaled through ImGui's
/// per-font `Scale`, which every size query and draw call honors.
#[must_use = "the font is popped again when the scope is dropped"]
pub struct FontScope<'ui> {
    /// Whether a font was pushed and must be popped.
    pushed: bool,
    /// The icon font's `Scale` before this scope changed it, restored on
    /// drop; `None` if the scope did not change it.
    restore_scale: Option<(*mut sys::ImFont, f32)>,
    /// Ties the scope to the frame's `Ui`.
    frame: PhantomData<&'ui ()>,
}

/// How large icons are relative to the current text font (R147).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconSize {
    /// An icon that accompanies text (a button's icon, a menu entry's, a
    /// status line's): the text's size × `font.size.icon`.
    WithText,
    /// An icon that stands alone or is a control itself (the empty state's
    /// icon, a task checkbox; Q535): the text's full size.
    Standalone,
}

impl IconSize {
    /// The icon size as a factor of the current text font's size.
    fn factor(self) -> f32 {
        match self {
            Self::WithText => crate::theme::Theme::default().typography.icon,
            Self::Standalone => 1.0,
        }
    }
}

impl<'ui> FontScope<'ui> {
    /// Pushes the icon font sized for icons of `size` next to the current
    /// text font, for the lifetime of the returned scope.
    /// `FontScope::push(ui, FontRole::Icons)` is [`IconSize::WithText`].
    pub fn push_icons(ui: &'ui Ui, size: IconSize) -> Self {
        Self::push_with_icon_size(ui, FontRole::Icons, size)
    }

    /// Pushes the font of `role` for the lifetime of the returned scope.
    pub fn push(ui: &'ui Ui, role: FontRole) -> Self {
        Self::push_with_icon_size(ui, role, IconSize::WithText)
    }

    /// Pushes the font of `role`; the icon font is sized for `icon_size`.
    fn push_with_icon_size(_ui: &'ui Ui, role: FontRole, icon_size: IconSize) -> Self {
        let index = FontRole::ALL
            .iter()
            .position(|candidate| *candidate == role)
            .unwrap_or_default();
        let font = FONT_POINTERS[index].load(Ordering::Acquire);
        if font.is_null() {
            return Self {
                pushed: false,
                restore_scale: None,
                frame: PhantomData,
            };
        }
        let restore_scale = if role == FontRole::Icons {
            // SAFETY: during a frame ImGui always has a current font, and
            // `font` is a live font of the same atlas (see below).
            unsafe { scale_icons_to_current_text(font, icon_size.factor()) }
        } else {
            None
        };
        // SAFETY: `font` was installed by the host's bridge for the current atlas
        // and is replaced (or cleared) before the atlas changes (`install_font`); pushing is only done during a frame (`Ui` is borrowed).
        unsafe { sys::igPushFont(font) };
        Self {
            pushed: true,
            restore_scale,
            frame: PhantomData,
        }
    }
}

/// Sets the `Scale` of the icon font `icons` so its size is the current
/// text font's size × `factor` (R147) and returns the previous
/// scale to restore. Returns `None` and changes nothing while the icon font
/// itself is current (an icon scope inside another keeps its size).
///
/// # Safety
///
/// Must be called during a frame, with `icons` a live font of the current
/// atlas.
unsafe fn scale_icons_to_current_text(
    icons: *mut sys::ImFont,
    factor: f32,
) -> Option<(*mut sys::ImFont, f32)> {
    // SAFETY: guaranteed by the caller; ImGui returns the current font,
    // which is never null during a frame.
    let current = unsafe { sys::igGetFont() };
    if current.is_null() || ptr::eq(current, icons) {
        return None;
    }
    // SAFETY: both pointers are live fonts of the current atlas; only plain
    // `f32` fields are read and the icon font's `Scale` is written on the
    // render thread, the only thread that touches ImGui.
    unsafe {
        let text_size = (*current).FontSize * (*current).Scale;
        let previous = (*icons).Scale;
        if (*icons).FontSize > 0.0 {
            (*icons).Scale = text_size * factor / (*icons).FontSize;
        }
        Some((icons, previous))
    }
}

impl Drop for FontScope<'_> {
    /// Pops the font pushed by [`FontScope::push`], if any, and gives the
    /// icon font back its previous scale.
    fn drop(&mut self) {
        if self.pushed {
            // SAFETY: balances the `igPushFont` in `push` within the same frame.
            unsafe { sys::igPopFont() };
        }
        if let Some((icons, scale)) = self.restore_scale {
            // SAFETY: `icons` is the live icon font this scope scaled in the
            // same frame; restoring after the pop leaves outer scopes as
            // they were.
            unsafe { (*icons).Scale = scale };
        }
    }
}
