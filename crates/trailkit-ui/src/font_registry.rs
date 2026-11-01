//! Registers the shared trailkit fonts with Nexus and gives access to them
//! while drawing (Windows only; memo plan M35, M36).
//!
//! Facts this relies on (verified in Nexus' `FontManager.cpp`, 2026-11):
//!
//! - Nexus copies the font data and config, so the embedded bytes may be
//!   passed directly.
//! - A second registration of an existing identifier only adds a callback, so
//!   all addons share one copy of each font.
//! - The atlas only contains Basic Latin, Latin-1, Latin Extended-A/B and every
//!   character of every registered localization text. Icons and typographic
//!   punctuation are therefore registered as a localization text *before* the
//!   fonts are added (adding a font rebuilds the atlas).
//! - Callbacks receive the new `ImFont` pointer after every atlas rebuild.

use std::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicPtr, AtomicU32, Ordering},
};

use nexus::{
    font::{add_font_from_memory, font_receive, resize_font},
    imgui::{Ui, sys},
    localization::set_translation,
};

use crate::{
    fonts::{FALLBACK_BODY_SIZE, FontRole},
    icons::glyph_text,
};

/// Localization key whose text makes Nexus put the shared glyphs into the atlas.
const SHARED_GLYPHS_KEY: &str = "trailkit.shared.localization.glyphs";

/// Typographic characters outside Latin-1 that trailkit texts use.
const TYPOGRAPHIC_GLYPHS: &str = "•…–—„“”‘’→";

/// Language codes the glyph texts are registered for. Nexus collects texts of
/// every language, one is enough; English is always present.
const GLYPH_LANGUAGE: &str = "en";

/// The `ImFont` pointer Nexus last delivered for each [`FontRole`] (index =
/// position in [`FontRole::ALL`]); null until the font is ready.
static FONT_POINTERS: [AtomicPtr<sys::ImFont>; FontRole::ALL.len()] =
    [const { AtomicPtr::new(ptr::null_mut()) }; FontRole::ALL.len()];

/// Body size (as `f32` bits) the fonts were last registered or resized for.
static REGISTERED_BODY_SIZE: AtomicU32 = AtomicU32::new(0);

/// Registers the glyph texts and all shared fonts with Nexus.
///
/// `addon_glyph_key` and `addon_glyphs` register the addon's own UI strings
/// (all languages concatenated) so their characters are in the atlas too,
/// e.g. `("trailkit.memo.localization.glyphs", all_memo_texts)`. Call once
/// from the addon's `load`; registrations are reverted on unload.
pub fn register_fonts(addon_glyph_key: &str, addon_glyphs: &str) {
    let mut shared_glyphs = glyph_text();
    shared_glyphs.push_str(TYPOGRAPHIC_GLYPHS);
    set_translation(SHARED_GLYPHS_KEY, GLYPH_LANGUAGE, &shared_glyphs);
    set_translation(addon_glyph_key, GLYPH_LANGUAGE, addon_glyphs);

    let receive = font_receive!(|identifier, font| store_font(identifier, font));
    REGISTERED_BODY_SIZE.store(FALLBACK_BODY_SIZE.to_bits(), Ordering::Relaxed);
    for role in FontRole::ALL {
        add_font_from_memory(
            role.identifier(),
            role.data(),
            role.size_for(FALLBACK_BODY_SIZE),
            None,
            receive,
        )
        .revert_on_unload();
    }
}

/// Resizes the shared fonts if `body_size` (Nexus' current `FontUI` size)
/// differs from the size they were built for. Cheap when nothing changed;
/// call it once per frame while a window is visible.
pub fn follow_body_size(body_size: f32) {
    if !(body_size.is_finite() && body_size >= 1.0) {
        return;
    }
    let previous = f32::from_bits(REGISTERED_BODY_SIZE.load(Ordering::Relaxed));
    if (previous - body_size).abs() < 0.5 {
        return;
    }
    REGISTERED_BODY_SIZE.store(body_size.to_bits(), Ordering::Relaxed);
    for role in FontRole::ALL {
        resize_font(role.identifier(), role.size_for(body_size));
    }
}

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

/// Uses `font` for `role` without Nexus: for the headless test harness
/// (`trailkit-ui-testing`, memo plan M65) or any host that builds its own
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

/// Stores the font pointer Nexus delivered for `identifier`.
fn store_font(identifier: &str, font: Option<&mut sys::ImFont>) {
    let Some(index) = FontRole::ALL
        .iter()
        .position(|role| role.identifier() == identifier)
    else {
        return;
    };
    let pointer = font.map_or(ptr::null_mut(), ptr::from_mut);
    FONT_POINTERS[index].store(pointer, Ordering::Release);
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
        // SAFETY: `font` was delivered by Nexus' font callback for the current
        // atlas and is replaced (or cleared) by that callback before the atlas
        // changes; pushing is only done during a frame (`Ui` is borrowed).
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
