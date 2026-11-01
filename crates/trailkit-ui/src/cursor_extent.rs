//! How far the mouse cursor reaches right of and below its hot spot
//! (Windows only; memo plan M84). GW2 draws a large cursor of its own;
//! tooltips must start beyond it, otherwise the cursor covers their text.
//!
//! The cursor image is read with `GetCursorInfo` / `GetIconInfo` /
//! `GetDIBits` only when the cursor handle changes, so a frame normally costs
//! one `GetCursorInfo` call.

use std::cell::Cell;

use windows::Win32::{
    Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
        GetDIBits, GetObjectW, HBITMAP, HGDIOBJ, ReleaseDC,
    },
    UI::WindowsAndMessaging::{CURSORINFO, GetCursorInfo, GetIconInfo, HICON, ICONINFO},
};

/// Width ImGui keeps free right of the mouse before a tooltip, in units of
/// `style.MouseCursorScale` (imgui.cpp, `FindBestWindowPosForPopup`).
const IMGUI_TOOLTIP_AVOID_WIDTH: f32 = 24.0;

/// Gap between the cursor and a tooltip in pixels.
const TOOLTIP_GAP: f32 = 4.0;

/// The `style.MouseCursorScale` that places tooltips just right of a cursor
/// reaching `extent` (`[right, down]`) from its hot spot; never below
/// ImGui's default of 1.
#[must_use]
pub fn tooltip_cursor_scale(extent: Option<[f32; 2]>) -> f32 {
    extent.map_or(1.0, |[right, _]| {
        ((right + TOOLTIP_GAP) / IMGUI_TOOLTIP_AVOID_WIDTH).max(1.0)
    })
}

thread_local! {
    /// The cursor handle measured last and its extent.
    static CACHE: Cell<Option<(usize, Option<[f32; 2]>)>> = const { Cell::new(None) };
}

/// The visible extent `[right, down]` of the current cursor in pixels,
/// measured from its hot spot (the tip); `None` if Windows cannot tell.
#[must_use]
pub fn cursor_extent() -> Option<[f32; 2]> {
    let mut info = CURSORINFO {
        cbSize: u32::try_from(size_of::<CURSORINFO>()).ok()?,
        ..CURSORINFO::default()
    };
    // SAFETY: `info` is a valid `CURSORINFO` with `cbSize` set, as required.
    unsafe { GetCursorInfo(&raw mut info) }.ok()?;
    let handle = info.hCursor.0 as usize;
    if handle == 0 {
        return None;
    }
    if let Some((cached, extent)) = CACHE.get()
        && cached == handle
    {
        return extent;
    }
    let extent = measure(HICON(info.hCursor.0));
    CACHE.set(Some((handle, extent)));
    extent
}

/// Measures the opaque area of `icon` relative to its hot spot.
fn measure(icon: HICON) -> Option<[f32; 2]> {
    let mut icon_info = ICONINFO::default();
    // SAFETY: `icon` is the current cursor handle and `icon_info` a valid
    // output struct; the bitmaps it returns are deleted below.
    unsafe { GetIconInfo(icon, &raw mut icon_info) }.ok()?;
    let mask = icon_info.hbmMask;
    let color = icon_info.hbmColor;
    let bounds = opaque_bounds(mask, color);
    for bitmap in [mask, color] {
        if !bitmap.is_invalid() {
            // SAFETY: `GetIconInfo` created these bitmaps for the caller,
            // who must delete them; each is deleted exactly once.
            let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        }
    }
    let [right, bottom] = bounds?;
    let hot_spot = [icon_info.xHotspot, icon_info.yHotspot];
    #[expect(clippy::cast_precision_loss, reason = "cursor sizes are tiny")]
    Some([
        right.saturating_sub(hot_spot[0]) as f32,
        bottom.saturating_sub(hot_spot[1]) as f32,
    ])
}

/// The size of `bitmap` in pixels.
fn bitmap_size(bitmap: HBITMAP) -> Option<(u32, u32)> {
    let mut header = BITMAP::default();
    let size = i32::try_from(size_of::<BITMAP>()).ok()?;
    // SAFETY: `header` is a `BITMAP` of `size` bytes, as `GetObjectW` expects
    // for a bitmap handle.
    let written = unsafe { GetObjectW(HGDIOBJ(bitmap.0), size, Some((&raw mut header).cast())) };
    if written == 0 {
        return None;
    }
    Some((
        u32::try_from(header.bmWidth).ok()?,
        u32::try_from(header.bmHeight).ok()?,
    ))
}

/// Reads `bitmap` as top-down 32-bit pixels (`0xAARRGGBB`).
fn read_pixels(bitmap: HBITMAP, width: u32, height: u32) -> Option<Vec<u32>> {
    let mut pixels = vec![0_u32; usize::try_from(width * height).ok()?];
    let mut info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).ok()?,
            biWidth: i32::try_from(width).ok()?,
            // Negative: rows top to bottom.
            biHeight: -i32::try_from(height).ok()?,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..BITMAPINFOHEADER::default()
        },
        ..BITMAPINFO::default()
    };
    // SAFETY: the screen DC is released below; `pixels` holds `width *
    // height` 32-bit values, exactly what `info` describes.
    unsafe {
        let screen = GetDC(None);
        let lines = GetDIBits(
            screen,
            bitmap,
            0,
            height,
            Some(pixels.as_mut_ptr().cast()),
            &raw mut info,
            DIB_RGB_COLORS,
        );
        ReleaseDC(None, screen);
        (lines > 0).then_some(pixels)
    }
}

/// The exclusive right and bottom edge of the visible pixels. A color cursor
/// shows pixels with alpha (or, without any alpha, where its mask is 0); a
/// monochrome cursor stacks an AND mask over an XOR mask and shows pixels
/// where AND is 0 or XOR is 1.
fn opaque_bounds(mask: HBITMAP, color: HBITMAP) -> Option<[u32; 2]> {
    let (mask_width, mask_height) = bitmap_size(mask)?;
    let mask_pixels = read_pixels(mask, mask_width, mask_height)?;
    let (width, height, visible): (u32, u32, Box<dyn Fn(usize) -> bool>) = if color.is_invalid() {
        let height = mask_height / 2;
        let offset = usize::try_from(mask_width * height).ok()?;
        (
            mask_width,
            height,
            Box::new(move |index| {
                mask_pixels[index] & 0xFF_FFFF == 0 || mask_pixels[offset + index] & 0xFF_FFFF != 0
            }),
        )
    } else {
        let (width, height) = bitmap_size(color)?;
        let color_pixels = read_pixels(color, width, height)?;
        let has_alpha = color_pixels.iter().any(|pixel| pixel >> 24 != 0);
        (
            width,
            height,
            Box::new(move |index| {
                if has_alpha {
                    color_pixels[index] >> 24 != 0
                } else {
                    mask_pixels
                        .get(index)
                        .is_some_and(|pixel| pixel & 0xFF_FFFF == 0)
                }
            }),
        )
    };
    let mut bounds: Option<[u32; 2]> = None;
    for y in 0..height {
        for x in 0..width {
            if visible(usize::try_from(y * width + x).ok()?) {
                let [right, bottom] = bounds.get_or_insert([0, 0]);
                *right = (*right).max(x + 1);
                *bottom = (*bottom).max(y + 1);
            }
        }
    }
    bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tooltips_start_right_of_the_cursor() {
        // A 44 px wide game cursor: 44 + 4 px gap = 48 px = 2 × 24.
        assert!((tooltip_cursor_scale(Some([44.0, 60.0])) - 2.0).abs() < f32::EPSILON);
        // Small cursors and unknown ones keep ImGui's default.
        assert!((tooltip_cursor_scale(Some([10.0, 16.0])) - 1.0).abs() < f32::EPSILON);
        assert!((tooltip_cursor_scale(None) - 1.0).abs() < f32::EPSILON);
    }
}
