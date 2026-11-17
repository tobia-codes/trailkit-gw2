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
#[must_use = "the font is popped again when the scope is dropped"]
pub struct FontScope<'ui> {
    /// Whether a font was pushed and must be popped.
    pushed: bool,
    /// Ties the scope to the frame's `Ui`.
    frame: PhantomData<&'ui ()>,
}

impl<'ui> FontScope<'ui> {
    /// Pushes the font of `role` for the lifetime of the returned scope.
    pub fn push(_ui: &'ui Ui, role: FontRole) -> Self {
        let index = FontRole::ALL
            .iter()
            .position(|candidate| *candidate == role)
            .unwrap_or_default();
        let font = FONT_POINTERS[index].load(Ordering::Acquire);
        if font.is_null() {
            return Self {
                pushed: false,
                frame: PhantomData,
            };
        }
        // SAFETY: `font` was installed by the host's bridge for the current atlas
        // and is replaced (or cleared) before the atlas changes (`install_font`); pushing is only done during a frame (`Ui` is borrowed).
        unsafe { sys::igPushFont(font) };
        Self {
            pushed: true,
            frame: PhantomData,
        }
    }
}

impl Drop for FontScope<'_> {
    /// Pops the font pushed by [`FontScope::push`], if any.
    fn drop(&mut self) {
        if self.pushed {
            // SAFETY: balances the `igPushFont` in `push` within the same frame.
            unsafe { sys::igPopFont() };
        }
    }
}
