//! Draws a chat-link chip (Windows only; memo plan M61, M75): a pill with
//! the type icon and a label such as "Item #19721 ×250", coins as amounts
//! with gold, silver and copper dots, and a warning chip for codes that
//! cannot be decoded. Used by the markdown view and the editor.

use nexus::imgui::{DrawListMut, ImColor32, Ui};

use crate::{
    chat_link::{ChatLink, ChatLinkChip, ChatLinkError, ChatLinkKind, ChipText},
    color::Color,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    theme::Theme,
};

/// Corner radius of a chip relative to `spacing.extra_small`: slightly
/// rounded, not a pill (memo plan M80).
const ROUNDING_FACTOR: f32 = 0.75;

/// Icon size relative to the text size (memo plan M81): at 100 % icons look
/// taller than the capitals, at 80 % too small next to the letters.
const ICON_SCALE: f32 = 0.9;

/// Size and vertical position of chip icons in the current fonts: icons are
/// drawn at [`ICON_SCALE`] of the text size and centered on the capital
/// letters (memo plan M80, M81).
#[derive(Debug, Clone, Copy)]
struct IconMetrics {
    /// Factor from the icon font's own size to the drawn size.
    scale: f32,
    /// Distance from the line top to the top of the drawn icon.
    offset: f32,
}

/// The vertical bounds `(top, bottom)` of `character` in the current font at
/// the current size, measured from the line top; `None` if unknown.
fn glyph_bounds(character: char) -> Option<(f32, f32)> {
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

/// Measures the icon size and offset for the current text font. Lucide draws
/// all icons on one grid, so the full square stands for every icon (centering
/// each glyph by its own shape would shift flat icons off the line).
/// Measured: a 16 px "H" is 9 px tall, the icon square 15 px.
fn icon_metrics(ui: &Ui) -> IconMetrics {
    let capital = glyph_bounds('H');
    let square = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        glyph_bounds(Icon::SQUARE.glyph())
    };
    match (capital, square) {
        (Some((capital_top, capital_bottom)), Some((square_top, square_bottom))) => {
            let scale = ICON_SCALE;
            let offset =
                (capital_top + capital_bottom) / 2.0 - (square_top + square_bottom) / 2.0 * scale;
            IconMetrics { scale, offset }
        }
        _ => IconMetrics {
            scale: 1.0,
            offset: 0.0,
        },
    }
}

/// What a chip needs from the addon: its texts and the code copied last.
#[derive(Clone, Copy)]
pub struct ChipContext<'a> {
    /// Translates a chip text into the addon language.
    pub text: &'a dyn Fn(ChipText) -> &'static str,
    /// The code copied by the last chip click, while "Copied" is shown.
    pub copied: Option<&'a str>,
}

/// One drawable part of a chip, left to right.
enum Piece<'a> {
    /// An icon glyph.
    Icon(Icon, Color),
    /// A run of text.
    Text(&'a str, Color),
    /// A small filled circle (coin), in this color.
    Dot(Color),
}

/// The icon of a link kind.
const fn kind_icon(kind: ChatLinkKind) -> Icon {
    match kind {
        ChatLinkKind::Coins => Icon::COINS,
        ChatLinkKind::Item => Icon::PACKAGE,
        ChatLinkKind::NpcText => Icon::MESSAGE_SQUARE,
        ChatLinkKind::MapPoint => Icon::MAP_PIN,
        ChatLinkKind::Skill => Icon::SWORDS,
        ChatLinkKind::Trait => Icon::SPARKLES,
        ChatLinkKind::Player => Icon::USER,
        ChatLinkKind::Recipe => Icon::SCROLL_TEXT,
        ChatLinkKind::Skin => Icon::SHIRT,
        ChatLinkKind::Outfit => Icon::VENETIAN_MASK,
        ChatLinkKind::WvwObjective => Icon::CASTLE,
        ChatLinkKind::BuildTemplate => Icon::HAMMER,
        ChatLinkKind::Achievement => Icon::TROPHY,
    }
}

/// Writes `value` in decimal into `buffer` and returns it as text.
fn format_number(value: u32, buffer: &mut [u8; 10]) -> &str {
    let mut start = buffer.len();
    let mut rest = value;
    loop {
        start -= 1;
        // `rest % 10` is a single digit.
        buffer[start] = b'0' + u8::try_from(rest % 10).unwrap_or(0);
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    std::str::from_utf8(&buffer[start..]).unwrap_or("")
}

/// Calls `visit` with every piece of `chip`, left to right. Allocation free:
/// numbers are formatted into stack buffers.
fn for_each_piece(
    chip: &ChatLinkChip,
    theme: &Theme,
    context: ChipContext<'_>,
    mut visit: impl FnMut(Piece<'_>),
) {
    let accent = theme.accent.base;
    let primary = theme.text.primary;
    match chip.link {
        Ok(ChatLink::Coins { copper }) => {
            visit(Piece::Icon(Icon::COINS, accent));
            let parts = [
                (copper / 10_000, theme.coin.gold),
                (copper / 100 % 100, theme.coin.silver),
                (copper % 100, theme.coin.copper),
            ];
            let mut shown = false;
            for (index, (amount, color)) in parts.into_iter().enumerate() {
                let is_last = index == parts.len() - 1;
                if amount == 0 && !(is_last && !shown) {
                    continue;
                }
                let mut buffer = [0; 10];
                visit(Piece::Text(format_number(amount, &mut buffer), primary));
                visit(Piece::Dot(color));
                shown = true;
            }
        }
        Ok(link) => {
            let kind = link.kind();
            visit(Piece::Icon(kind_icon(kind), accent));
            visit(Piece::Text((context.text)(ChipText::Kind(kind)), primary));
            if let ChatLink::BuildTemplate { profession } = link {
                visit(Piece::Text(" (", primary));
                visit(Piece::Text(
                    (context.text)(ChipText::Profession(profession)),
                    primary,
                ));
                visit(Piece::Text(")", primary));
            }
            if !chip.detail.is_empty() {
                visit(Piece::Text(&chip.detail, theme.text.secondary));
            }
        }
        Err(_) => {
            let warning = theme.status.warning;
            visit(Piece::Icon(Icon::TRIANGLE_ALERT, warning));
            visit(Piece::Text(
                (context.text)(ChipText::InvalidChatLink),
                warning,
            ));
        }
    }
}

/// The width of a piece in the current font.
fn piece_width(ui: &Ui, piece: &Piece<'_>, theme: &Theme, icons: IconMetrics) -> f32 {
    match piece {
        Piece::Icon(icon, _) => {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.calc_text_size(icon.as_str(&mut buffer))[0] * icons.scale + theme.spacing.extra_small
        }
        Piece::Text(text, _) => ui.calc_text_size(text)[0],
        // A little air before the dot, more after it (before the next amount).
        Piece::Dot(_) => dot_radius(ui) * 2.0 + theme.spacing.extra_small * 1.5,
    }
}

/// How far the pill reaches above and below the text: a third of the
/// leading of running text, so it stays slim and chips on adjacent lines
/// keep a clear gap; at least one pixel.
fn vertical_padding(ui: &Ui, theme: &Theme) -> f32 {
    (theme.typography.leading_for(ui.current_font_size()) / 3.0).max(1.0)
}

/// Radius of a coin dot: about the size of a small letter.
fn dot_radius(ui: &Ui) -> f32 {
    ui.current_font_size() * 0.28
}

/// Draws `icon` at `position` in `size` pixels with the current (icon) font
/// into the current window's draw list; ImGui's draw-list API in Rust only
/// draws text at the font's own size.
fn draw_scaled_icon(icon: Icon, position: [f32; 2], size: f32, color: Color) {
    let mut buffer = [0; 4];
    let text = icon.as_str(&mut buffer);
    let begin = text.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, so the current
    // window's draw list and the current font are valid; `begin..begin +
    // len` is the UTF-8 glyph, which ImGui only reads; no clip rectangle.
    unsafe {
        nexus::imgui::sys::ImDrawList_AddText_FontPtr(
            nexus::imgui::sys::igGetWindowDrawList(),
            nexus::imgui::sys::igGetFont(),
            size,
            nexus::imgui::sys::ImVec2 {
                x: position[0],
                y: position[1],
            },
            ImColor32::from(color.to_array()).to_bits(),
            begin,
            begin.add(text.len()),
            0.0,
            std::ptr::null(),
        );
    }
}

/// Horizontal padding inside the pill.
fn padding(theme: &Theme) -> f32 {
    theme.spacing.extra_small * 1.5
}

/// The width `chip` takes in the current font, padding included.
#[must_use]
pub fn chip_width(ui: &Ui, chip: &ChatLinkChip, theme: &Theme, context: ChipContext<'_>) -> f32 {
    let icons = icon_metrics(ui);
    let mut width = 2.0 * padding(theme);
    for_each_piece(chip, theme, context, |piece| {
        width += piece_width(ui, &piece, theme, icons);
    });
    width
}

/// Draws `chip` with its top-left corner at `position` (screen
/// coordinates, one text line high) into `draw_list` and returns its
/// rectangle `[min, max]`.
pub fn draw_chip(
    ui: &Ui,
    draw_list: &DrawListMut<'_>,
    chip: &ChatLinkChip,
    position: [f32; 2],
    theme: &Theme,
    context: ChipContext<'_>,
) -> [[f32; 2]; 2] {
    let width = chip_width(ui, chip, theme, context);
    let height = ui.text_line_height();
    // The pill reaches a little into the line spacing above and below.
    let vertical_padding = vertical_padding(ui, theme);
    let min = [position[0], position[1] - vertical_padding];
    let max = [position[0] + width, position[1] + height + vertical_padding];
    let rounding = theme.spacing.extra_small * ROUNDING_FACTOR;
    draw_list
        .add_rect(min, max, theme.surface.hover.with_alpha(0.6).to_array())
        .filled(true)
        .rounding(rounding)
        .build();
    if chip.link.is_err() {
        draw_list
            .add_rect(min, max, theme.status.warning.to_array())
            .rounding(rounding)
            .thickness(theme.border.width.max(1.0))
            .build();
    }
    let icons = icon_metrics(ui);
    let icon_size = {
        let _icons = FontScope::push(ui, FontRole::Icons);
        ui.current_font_size() * icons.scale
    };
    let mut x = position[0] + padding(theme);
    for_each_piece(chip, theme, context, |piece| {
        let advance = piece_width(ui, &piece, theme, icons);
        match piece {
            Piece::Icon(icon, color) => {
                let _icons = FontScope::push(ui, FontRole::Icons);
                draw_scaled_icon(icon, [x, position[1] + icons.offset], icon_size, color);
            }
            Piece::Text(text, color) => {
                draw_list.add_text([x, position[1]], color.to_array(), text);
            }
            Piece::Dot(color) => {
                let radius = dot_radius(ui);
                draw_list
                    .add_circle(
                        [
                            x + theme.spacing.extra_small / 2.0 + radius,
                            position[1] + height / 2.0,
                        ],
                        radius,
                        color.to_array(),
                    )
                    .filled(true)
                    .build();
            }
        }
        x += advance;
    });
    [min, max]
}

/// Shows the tooltip of `chip`: its label (or why it is invalid), the raw
/// code and, if `copyable`, the click hint or "Copied".
pub fn chip_tooltip(
    ui: &Ui,
    chip: &ChatLinkChip,
    theme: &Theme,
    context: ChipContext<'_>,
    copyable: bool,
) {
    ui.tooltip(|| {
        match chip.link {
            Ok(ChatLink::Coins { .. }) => {}
            Ok(link) => {
                ui.text((context.text)(ChipText::Kind(link.kind())));
                if let ChatLink::BuildTemplate { profession } = link {
                    ui.same_line_with_spacing(0.0, 0.0);
                    ui.text(" (");
                    ui.same_line_with_spacing(0.0, 0.0);
                    ui.text((context.text)(ChipText::Profession(profession)));
                    ui.same_line_with_spacing(0.0, 0.0);
                    ui.text(")");
                }
                if !chip.detail.is_empty() {
                    ui.same_line_with_spacing(0.0, 0.0);
                    ui.text(&chip.detail);
                }
            }
            Err(error) => {
                let reason = match error {
                    ChatLinkError::Damaged => ChipText::DamagedCode,
                    ChatLinkError::UnknownType => ChipText::UnknownLinkType,
                };
                ui.text_colored(
                    theme.status.warning.to_array(),
                    (context.text)(ChipText::InvalidChatLink),
                );
                ui.text((context.text)(reason));
            }
        }
        ui.text_colored(theme.text.secondary.to_array(), &chip.code);
        if !copyable {
            return;
        }
        let hint = if context.copied == Some(chip.code.as_str()) {
            ChipText::Copied
        } else {
            ChipText::ClickToCopy
        };
        ui.text_colored(theme.text.secondary.to_array(), (context.text)(hint));
    });
}
