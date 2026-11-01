//! A headless ImGui context for tests (memo plan M65, Windows only).
//!
//! It loads the real trailkit fonts (all [`FontRole`]s at the fallback body
//! size), installs them for `FontScope`, maps keys like Nexus' Win32 backend
//! (virtual-key codes) and runs frames with simulated input. Every frame can
//! be rasterized into an [`Image`]. ImGui allows one context at a time, so a
//! harness holds a global lock: UI tests run one after another.

use std::sync::{Mutex, MutexGuard};

use nexus::imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Key, Ui, sys};
use trailkit_ui::{
    font_registry::install_font,
    fonts::{FALLBACK_BODY_SIZE, FontRole},
};

use crate::raster::{Image, Texture, Vertex};

/// Serializes all harnesses: ImGui has one global current context.
static CONTEXT_LOCK: Mutex<()> = Mutex::new(());

/// Background behind the windows in rendered images (a neutral dark grey,
/// roughly the brightness of a GW2 scene).
pub const BACKGROUND: [u8; 4] = [58, 62, 68, 255];

/// Glyph ranges of the text fonts: Basic Latin to Latin Extended-B plus
/// general punctuation and arrows (like Nexus' atlas plus our texts).
static TEXT_GLYPHS: [u16; 7] = [0x0020, 0x024F, 0x2000, 0x206F, 0x2190, 0x21FF, 0];

/// Glyph ranges of the icon font: the private use area.
static ICON_GLYPHS: [u16; 3] = [0xE000, 0xF8FF, 0];

/// Windows virtual-key codes for ImGui's named keys (as Nexus maps them).
const KEY_CODES: [(Key, u32); 22] = [
    (Key::Tab, 0x09),
    (Key::LeftArrow, 0x25),
    (Key::RightArrow, 0x27),
    (Key::UpArrow, 0x26),
    (Key::DownArrow, 0x28),
    (Key::PageUp, 0x21),
    (Key::PageDown, 0x22),
    (Key::Home, 0x24),
    (Key::End, 0x23),
    (Key::Insert, 0x2D),
    (Key::Delete, 0x2E),
    (Key::Backspace, 0x08),
    (Key::Space, 0x20),
    (Key::Enter, 0x0D),
    (Key::Escape, 0x1B),
    (Key::KeyPadEnter, 0x10D),
    (Key::A, 0x41),
    (Key::C, 0x43),
    (Key::V, 0x56),
    (Key::X, 0x58),
    (Key::Y, 0x59),
    (Key::Z, 0x5A),
];

/// The input of one simulated frame.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameInput {
    /// Mouse position in pixels.
    pub mouse_position: [f32; 2],
    /// Whether the left mouse button is held.
    pub left_button_down: bool,
    /// Characters typed in this frame.
    pub characters: String,
    /// Virtual-key codes pressed in this frame (e.g. `0x08` Backspace).
    pub keys: Vec<u32>,
    /// Ctrl held.
    pub control: bool,
    /// Shift held.
    pub shift: bool,
    /// Vertical mouse wheel movement in this frame (positive: away from the
    /// user).
    pub mouse_wheel: f32,
}

/// A headless ImGui context with the trailkit fonts.
pub struct UiHarness {
    /// The ImGui context (dropped before the lock is released).
    context: Context,
    /// The font atlas as straight RGBA.
    atlas: (u32, u32, Vec<u8>),
    /// Output size in pixels.
    size: [f32; 2],
    /// Keys held in the previous frame, released at the start of the next.
    held_keys: Vec<u32>,
    /// Keeps other harnesses out while this one exists.
    _lock: MutexGuard<'static, ()>,
}

impl UiHarness {
    /// A harness rendering `size` pixels.
    #[must_use]
    pub fn new(size: [f32; 2]) -> Self {
        let lock = CONTEXT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut context = Context::create();
        context.set_ini_filename(None);
        {
            let io = context.io_mut();
            io.display_size = size;
            io.delta_time = 1.0 / 60.0;
            for (key, code) in KEY_CODES {
                io.key_map[key as usize] = code;
            }
        }
        let atlas = {
            let mut fonts = context.fonts();
            for role in FontRole::ALL {
                let ranges: &'static [u16] = if role == FontRole::Icons {
                    &ICON_GLYPHS
                } else {
                    &TEXT_GLYPHS
                };
                let size_pixels = role.size_for(FALLBACK_BODY_SIZE);
                fonts.add_font(&[FontSource::TtfData {
                    data: role.data(),
                    size_pixels,
                    config: Some(FontConfig {
                        size_pixels,
                        glyph_ranges: FontGlyphRanges::from_slice(ranges),
                        ..FontConfig::default()
                    }),
                }]);
            }
            let texture = fonts.build_rgba32_texture();
            (texture.width, texture.height, texture.data.to_vec())
        };
        install_fonts();
        Self {
            context,
            atlas,
            size,
            held_keys: Vec::new(),
            _lock: lock,
        }
    }

    /// Runs one frame with `input`, calling `draw` with the frame's `Ui`,
    /// and returns the rendered image.
    pub fn frame(&mut self, input: &FrameInput, draw: impl FnOnce(&Ui)) -> Image {
        {
            let io = self.context.io_mut();
            for code in self.held_keys.drain(..) {
                if let Some(down) = io.keys_down.get_mut(code as usize) {
                    *down = false;
                }
            }
            io.mouse_pos = input.mouse_position;
            io.mouse_down[0] = input.left_button_down;
            io.key_ctrl = input.control;
            io.key_shift = input.shift;
            io.mouse_wheel = input.mouse_wheel;
            for code in &input.keys {
                if let Some(down) = io.keys_down.get_mut(*code as usize) {
                    *down = true;
                }
            }
            for character in input.characters.chars() {
                io.add_input_character(character);
            }
        }
        self.held_keys.clone_from(&input.keys);
        let ui = self.context.frame();
        draw(&ui);
        let draw_data = ui.render();
        rasterize(draw_data, self.size, &self.atlas)
    }

    /// Runs `frames` frames without input changes (e.g. to let layouts
    /// settle), returning the last image.
    pub fn settle(
        &mut self,
        frames: usize,
        input: &FrameInput,
        mut draw: impl FnMut(&Ui),
    ) -> Image {
        let mut image = Image::new(1, 1, BACKGROUND);
        for _ in 0..frames.max(1) {
            image = self.frame(input, &mut draw);
        }
        image
    }
}

/// Installs the atlas fonts (added in [`FontRole::ALL`] order) for `FontScope`.
fn install_fonts() {
    // SAFETY: a context exists (created just before) and its atlas was built;
    // `Fonts.Data` holds one `ImFont*` per added font, in insertion order, and
    // stays valid until the context is destroyed, which happens after the
    // harness' last frame.
    unsafe {
        let io = sys::igGetIO();
        let fonts = &(*(*io).Fonts).Fonts;
        for (index, role) in FontRole::ALL.into_iter().enumerate() {
            let Ok(index) = isize::try_from(index) else {
                continue;
            };
            if index < isize::try_from(fonts.Size).unwrap_or(0) {
                install_font(role, *fonts.Data.offset(index));
            }
        }
    }
}

/// Draws ImGui's draw data into an image of `size`.
fn rasterize(
    draw_data: &nexus::imgui::DrawData,
    size: [f32; 2],
    atlas: &(u32, u32, Vec<u8>),
) -> Image {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "test image sizes are small and positive"
    )]
    let mut image = Image::new(size[0] as u32, size[1] as u32, BACKGROUND);
    let texture = Texture {
        width: atlas.0,
        height: atlas.1,
        rgba: &atlas.2,
    };
    for list in draw_data.draw_lists() {
        let vertices = list.vtx_buffer();
        let indices = list.idx_buffer();
        for command in list.commands() {
            let nexus::imgui::DrawCmd::Elements { count, cmd_params } = command else {
                continue;
            };
            let start = cmd_params.idx_offset;
            let Some(triangle_indices) = indices.get(start..start + count) else {
                continue;
            };
            for triangle in triangle_indices.as_chunks::<3>().0 {
                let corner = |index: u16| {
                    let vertex = vertices[cmd_params.vtx_offset + usize::from(index)];
                    Vertex {
                        position: vertex.pos,
                        uv: vertex.uv,
                        color: vertex.col,
                    }
                };
                image.draw_triangle(
                    [
                        corner(triangle[0]),
                        corner(triangle[1]),
                        corner(triangle[2]),
                    ],
                    cmd_params.clip_rect,
                    &texture,
                );
            }
        }
    }
    image
}
