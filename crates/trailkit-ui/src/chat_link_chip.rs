//! Draws a chat-link chip (Windows only; memo plan M61, M75): a pill with
//! the type icon and a label such as "Item #19721 ×250", coins as amounts
//! with gold, silver and copper dots, and a warning chip for codes that
//! cannot be decoded. Used by the markdown view and the editor.

use nexus::imgui::{DrawListMut, Ui};

use crate::{
    chat_link::{ChatLink, ChatLinkChip, ChatLinkError, ChatLinkKind, ChipText},
    color::Color,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    theme::Theme,
};

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
        ChatLinkKind::Skill => Icon::ZAP,
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
fn piece_width(ui: &Ui, piece: &Piece<'_>, theme: &Theme) -> f32 {
    match piece {
        Piece::Icon(icon, _) => {
            let _icons = FontScope::push(ui, FontRole::Icons);
            let mut buffer = [0; 4];
            ui.calc_text_size(icon.as_str(&mut buffer))[0] + theme.spacing.extra_small
        }
        Piece::Text(text, _) => ui.calc_text_size(text)[0],
        // A little air before the dot, more after it (before the next amount).
        Piece::Dot(_) => dot_radius(ui) * 2.0 + theme.spacing.extra_small * 1.5,
    }
}

/// Radius of a coin dot: a third of the font size.
fn dot_radius(ui: &Ui) -> f32 {
    ui.current_font_size() / 3.0
}

/// Horizontal padding inside the pill.
fn padding(theme: &Theme) -> f32 {
    theme.spacing.extra_small * 1.5
}

/// The width `chip` takes in the current font, padding included.
#[must_use]
pub fn chip_width(ui: &Ui, chip: &ChatLinkChip, theme: &Theme, context: ChipContext<'_>) -> f32 {
    let mut width = 2.0 * padding(theme);
    for_each_piece(chip, theme, context, |piece| {
        width += piece_width(ui, &piece, theme);
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
    let max = [position[0] + width, position[1] + height];
    draw_list
        .add_rect(position, max, theme.surface.hover.to_array())
        .filled(true)
        .rounding(height / 2.0)
        .build();
    if chip.link.is_err() {
        draw_list
            .add_rect(position, max, theme.status.warning.to_array())
            .rounding(height / 2.0)
            .thickness(theme.border.width.max(1.0))
            .build();
    }
    let mut x = position[0] + padding(theme);
    for_each_piece(chip, theme, context, |piece| {
        let advance = piece_width(ui, &piece, theme);
        match piece {
            Piece::Icon(icon, color) => {
                let _icons = FontScope::push(ui, FontRole::Icons);
                let mut buffer = [0; 4];
                draw_list.add_text([x, position[1]], color.to_array(), icon.as_str(&mut buffer));
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
    [position, max]
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
