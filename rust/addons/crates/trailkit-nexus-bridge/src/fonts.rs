//! Registers the shared trailkit fonts with Nexus and resizes them with
//! Nexus' `FontUI` (memo plan M35, M36; moved out of `trailkit-ui`, Q496).
//! The delivered fonts go to `trailkit-ui`'s font registry, which draws with
//! them through `FontScope`.
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
    ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use nexus::{
    font::{add_font_from_memory, font_receive, resize_font},
    imgui::sys,
    localization::set_translation,
};

use trailkit_ui::{
    text::font_registry::install_font,
    text::fonts::{FALLBACK_BODY_SIZE, FontRole},
    text::icons::glyph_text,
    theme::Typography,
};

/// Localization key whose text makes Nexus put the shared glyphs into the atlas.
const SHARED_GLYPHS_KEY: &str = "trailkit.shared.localization.glyphs";

/// Typographic characters outside Latin-1 that trailkit texts use.
const TYPOGRAPHIC_GLYPHS: &str = "•…–—„“”‘’›";

/// Language codes the glyph texts are registered for. Nexus collects texts of
/// every language, one is enough; English is always present.
const GLYPH_LANGUAGE: &str = "en";

/// Body size (as `f32` bits) the fonts were last registered or resized for.
static REGISTERED_BODY_SIZE: AtomicU32 = AtomicU32::new(0);

/// Registers the glyph texts and all shared fonts with Nexus.
///
/// `addon_glyph_key` and `addon_glyphs` register the addon's own UI strings
/// (all languages concatenated) so their characters are in the atlas too,
/// e.g. `("trailkit.memo.localization.glyphs", all_memo_texts)`. The sizes
/// are `typography`'s factors of the body size (R96), usually those of the
/// addon's active theme. Call once from the addon's `load`; registrations are
/// reverted on unload.
pub fn register_fonts(addon_glyph_key: &str, addon_glyphs: &str, typography: &Typography) {
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
            role.size_for(FALLBACK_BODY_SIZE, typography),
            None,
            receive,
        )
        .revert_on_unload();
    }
}

/// Resizes the shared fonts if `body_size` (Nexus' current `FontUI` size)
/// differs from the size they were built for, each role at its
/// `typography` factor. Cheap when nothing changed; call it once per frame
/// while a window is visible.
pub fn follow_body_size(body_size: f32, typography: &Typography) {
    if !(body_size.is_finite() && body_size >= 1.0) {
        return;
    }
    let previous = f32::from_bits(REGISTERED_BODY_SIZE.load(Ordering::Relaxed));
    if (previous - body_size).abs() < 0.5 {
        return;
    }
    REGISTERED_BODY_SIZE.store(body_size.to_bits(), Ordering::Relaxed);
    for role in FontRole::ALL {
        resize_font(role.identifier(), role.size_for(body_size, typography));
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
    // SAFETY: Nexus delivers a font of the current atlas, or null when it is
    // gone, and calls this again with the new pointer after every rebuild.
    unsafe { install_font(FontRole::ALL[index], pointer) };
}
