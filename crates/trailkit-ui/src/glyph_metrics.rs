//! Where glyphs really are inside a text line (Windows only; memo plan M80,
//! M85). ImGui centers the whole font line box, but Open Sans keeps more room
//! above capitals (for accents) than below, so text centered by ImGui sits a
//! little low. These measurements let components center by the letters.

/// The vertical bounds `(top, bottom)` of `character` in the current font at
/// the current size, measured from the line top; `None` if unknown.
pub fn glyph_bounds(character: char) -> Option<(f32, f32)> {
    let character = u16::try_from(u32::from(character)).ok()?;
    // SAFETY: called on the render thread inside a frame, so the current
    // font is valid; `ImFont_FindGlyph` only reads it and returns a pointer
    // into the font (the fallback glyph if the character is missing), or
    // null.
    unsafe {
        let font = nexus::imgui::sys::igGetFont();
        let glyph = nexus::imgui::sys::ImFont_FindGlyph(font, character).as_ref()?;
        let scale = nexus::imgui::sys::igGetFontSize() / (*font).FontSize;
        Some((glyph.Y0 * scale, glyph.Y1 * scale))
    }
}

/// The middle of the capital letters below the line top in the current font,
/// e.g. to center a button label; `None` if the font cannot tell.
#[must_use]
pub fn capital_middle() -> Option<f32> {
    glyph_bounds('H').map(|(top, bottom)| (top + bottom) / 2.0)
}
