//! Where glyphs really are inside a text line (Windows only; memo plan M80,
//! M85). ImGui centers the whole font line box, but the text font keeps more room
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

/// The ink box `[left, top, right, bottom]` of `character` in the current
/// font at the current size, relative to the pen position at the line top;
/// `None` if unknown. Centers single letters exactly (memo M105), where the
/// advance width would add uneven side bearings.
pub fn glyph_ink_box(character: char) -> Option<[f32; 4]> {
    let character = u16::try_from(u32::from(character)).ok()?;
    // SAFETY: as in `glyph_bounds`: render thread, valid current font,
    // `ImFont_FindGlyph` only reads it.
    unsafe {
        let font = nexus::imgui::sys::igGetFont();
        let glyph = nexus::imgui::sys::ImFont_FindGlyph(font, character).as_ref()?;
        let scale = nexus::imgui::sys::igGetFontSize() / (*font).FontSize;
        Some([
            glyph.X0 * scale,
            glyph.Y0 * scale,
            glyph.X1 * scale,
            glyph.Y1 * scale,
        ])
    }
}

/// The middle of the capital letters below the line top in the current font,
/// e.g. to center a button label; `None` if the font cannot tell.
#[must_use]
pub fn capital_middle() -> Option<f32> {
    glyph_bounds('H').map(|(top, bottom)| (top + bottom) / 2.0)
}

/// Atlas coverage from which a texel counts as ink: the 50 % line is where
/// the glyph outline runs, so fringes of anti-aliasing and oversampling do
/// not widen the box.
const INK_COVERAGE_THRESHOLD: u8 = 128;

/// The visible ink box `[left, top, right, bottom]` of `character` in the
/// current font at the current size, relative to the pen position at the
/// line top, measured from the font atlas pixels (memo R76 question 56).
///
/// Unlike [`glyph_ink_box`] (the glyph's quad, which ImGui's oversampling
/// makes a fraction of a pixel wider, not evenly on both sides) this is
/// where the letter really shows, so it can be centered to the pixel.
/// `None` if the glyph is unknown, empty, or the atlas pixels are no longer
/// in memory; callers fall back to [`glyph_ink_box`]. Scans only the
/// glyph's own texels (a few hundred bytes).
pub fn glyph_visible_ink(character: char) -> Option<[f32; 4]> {
    let character = u16::try_from(u32::from(character)).ok()?;
    // SAFETY: as in `glyph_bounds`: render thread, valid current font and
    // its atlas; `ImFont_FindGlyph` only reads. The atlas pixel buffers are
    // either null or `TexWidth × TexHeight` elements owned by the atlas for
    // as long as the font exists; only texels inside the glyph's UV
    // rectangle (clamped to the texture) are read.
    unsafe {
        let font = nexus::imgui::sys::igGetFont();
        let glyph = nexus::imgui::sys::ImFont_FindGlyph(font, character).as_ref()?;
        let atlas = (*font).ContainerAtlas.as_ref()?;
        let width = usize::try_from(atlas.TexWidth).ok()?;
        let height = usize::try_from(atlas.TexHeight).ok()?;
        let alpha8 = atlas.TexPixelsAlpha8;
        let rgba32 = atlas.TexPixelsRGBA32;
        let coverage_at = |x: usize, y: usize| -> u8 {
            let index = y * width + x;
            if !alpha8.is_null() {
                *alpha8.add(index)
            } else {
                // IM_COL32 layout: alpha in the top byte.
                (*rgba32.add(index) >> 24).to_le_bytes()[0]
            }
        };
        if (alpha8.is_null() && rgba32.is_null()) || width == 0 || height == 0 {
            log_missing_atlas_pixels();
            return None;
        }
        let texel = |uv: f32, size: usize| -> usize {
            // Texture coordinates are non-negative and at most 1.
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss,
                reason = "uv is clamped to 0..=1 and the texture is far below 2^24 texels"
            )]
            let texel = (uv.clamp(0.0, 1.0) * size as f32).round() as usize;
            texel.min(size)
        };
        let [left, right] = [texel(glyph.U0, width), texel(glyph.U1, width)];
        let [top, bottom] = [texel(glyph.V0, height), texel(glyph.V1, height)];
        let ink = ink_texels(left, top, right, bottom, |x, y| {
            coverage_at(x, y) >= INK_COVERAGE_THRESHOLD
        })?;
        let scale = nexus::imgui::sys::igGetFontSize() / (*font).FontSize;
        #[expect(
            clippy::cast_precision_loss,
            reason = "glyph rectangles are a few dozen texels"
        )]
        let to_quad = |texel: usize, start: usize, end: usize, quad_start: f32, quad_end: f32| {
            let fraction = (texel - start) as f32 / (end - start).max(1) as f32;
            (quad_start + fraction * (quad_end - quad_start)) * scale
        };
        Some([
            to_quad(ink[0], left, right, glyph.X0, glyph.X1),
            to_quad(ink[1], top, bottom, glyph.Y0, glyph.Y1),
            to_quad(ink[2], left, right, glyph.X0, glyph.X1),
            to_quad(ink[3], top, bottom, glyph.Y0, glyph.Y1),
        ])
    }
}

/// The texel box `[left, top, right, bottom]` (right and bottom exclusive)
/// of the inked texels in `left..right` × `top..bottom`; `None` if none is.
fn ink_texels(
    left: usize,
    top: usize,
    right: usize,
    bottom: usize,
    is_ink: impl Fn(usize, usize) -> bool,
) -> Option<[usize; 4]> {
    let mut found: Option<[usize; 4]> = None;
    for y in top..bottom {
        for x in left..right {
            if !is_ink(x, y) {
                continue;
            }
            found = Some(match found {
                None => [x, y, x + 1, y + 1],
                Some([x0, y0, x1, y1]) => [x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1)],
            });
        }
    }
    found
}

/// Logs once that the font atlas pixels are gone, so visible ink cannot be
/// measured and badges fall back to the glyph quad.
fn log_missing_atlas_pixels() {
    static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        log::warn!("font atlas pixels are not in memory; glyphs are centered by their quads");
    }
}
