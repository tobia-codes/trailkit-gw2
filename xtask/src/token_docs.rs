//! Generates the design token reference from `trailkit_ui::theme` (ROADMAP R84).
//!
//! The themes are the only source of the values, so the document cannot drift:
//! the test [`tests::the_committed_document_is_current`] fails in `cargo test`
//! (pre-push hook and CI) when it is out of date. Every token struct is
//! destructured without `..`, so a new token does not compile until it is
//! documented here.

use std::fs;

use anyhow::{Context, Result, bail};
use trailkit_ui::{
    color::Color,
    theme::{
        AccentColors, BorderTokens, ButtonMetrics, ChipCategoryColors, ChipColors, CoinColors,
        EventColors, ListMetrics, PaneMetrics, Radii, ResetColors, Spacing, StatusColors,
        SurfaceColors, TextColors, Theme, Typography,
    },
};

use crate::workspace;

/// The generated document, relative to the repository root.
const DOCUMENT_PATH: &str = ".agents/skills/trailkit-ui/references/token-values.md";

/// What `cargo xtask tokens` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Rewrite the document.
    Write,
    /// Fail if the document differs from the themes.
    Check,
}

/// The value of a token.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenValue {
    /// A color.
    Color(Color),
    /// A size in pixels.
    Pixels(f32),
    /// A multiple of the current font size.
    FontSizeFactor(f32),
}

/// One documented token.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Token {
    /// The token id used in the design documents, e.g. `bg.window`.
    id: &'static str,
    /// Its value in a theme.
    value: TokenValue,
    /// What it is for.
    purpose: &'static str,
}

/// A titled table of tokens.
#[derive(Debug, Clone, PartialEq)]
struct TokenGroup {
    /// The heading of the table.
    title: &'static str,
    /// The tokens, in reading order.
    tokens: Vec<Token>,
}

/// A color token.
const fn color(id: &'static str, value: Color, purpose: &'static str) -> Token {
    Token {
        id,
        value: TokenValue::Color(value),
        purpose,
    }
}

/// A size token in pixels.
const fn pixels(id: &'static str, value: f32, purpose: &'static str) -> Token {
    Token {
        id,
        value: TokenValue::Pixels(value),
        purpose,
    }
}

/// The surface colors.
fn surface_group(colors: SurfaceColors) -> TokenGroup {
    let SurfaceColors {
        window,
        panel,
        input,
        hover,
        active,
        pressed,
        popup,
        popup_hover,
        modal_overlay,
    } = colors;
    TokenGroup {
        title: "Surfaces",
        tokens: vec![
            color("bg.window", window, "Window frame, gaps between panes"),
            color("bg.panel", panel, "Panes and other content regions"),
            color("bg.input", input, "Input fields"),
            color(
                "bg.hover",
                hover,
                "Hovered rows and inputs, fill of regular buttons",
            ),
            color(
                "bg.active",
                active,
                "Selected and pressed list rows, hovered regular buttons",
            ),
            color(
                "bg.pressed",
                pressed,
                "Pressed regular buttons, dropdowns and menu rows",
            ),
            color("bg.popup", popup, "Popups, tooltips, menus"),
            color("bg.popup_hover", popup_hover, "Hovered rows inside popups"),
            color(
                "overlay.modal",
                modal_overlay,
                "Dimming behind dialogs and drawers",
            ),
        ],
    }
}

/// The text colors.
fn text_group(colors: TextColors) -> TokenGroup {
    let TextColors {
        primary,
        secondary,
        disabled,
        link,
        on_accent,
    } = colors;
    TokenGroup {
        title: "Text",
        tokens: vec![
            color("text.primary", primary, "Body text, headings"),
            color("text.secondary", secondary, "Counts, hints, placeholders"),
            color(
                "text.disabled",
                disabled,
                "Disabled controls, decorative marks",
            ),
            color("text.link", link, "Links"),
            color("text.on_accent", on_accent, "Text on an `accent` fill"),
        ],
    }
}

/// The accent colors.
fn accent_group(colors: AccentColors) -> TokenGroup {
    let AccentColors {
        base,
        hover,
        active,
        text,
        highlight,
    } = colors;
    TokenGroup {
        title: "Accent",
        tokens: vec![
            color("accent", base, "Fill of the primary button, selection bar"),
            color("accent.hover", hover, "Hovered primary button"),
            color("accent.active", active, "Pressed primary button"),
            color("accent.text", text, "Brass as text, icon or line"),
            color("accent.highlight", highlight, "Behind search hits"),
        ],
    }
}

/// The status colors.
fn status_group(colors: StatusColors) -> TokenGroup {
    let StatusColors {
        success,
        warning,
        danger,
        info,
    } = colors;
    TokenGroup {
        title: "Status",
        tokens: vec![
            color("success", success, "Success"),
            color("warning", warning, "Warnings, invalid chat links"),
            color("danger", danger, "Destructive actions"),
            color("info", info, "Information"),
        ],
    }
}

/// The border colors and width.
fn border_group(tokens: BorderTokens) -> TokenGroup {
    let BorderTokens {
        color: default_color,
        focus,
        subtle,
        width,
    } = tokens;
    TokenGroup {
        title: "Borders",
        tokens: vec![
            color("border", default_color, "Frame of inputs and dropdowns"),
            color("border.focus", focus, "Focused element"),
            color(
                "border.subtle",
                subtle,
                "Dividers, outline of windows, panes, popups, dialogs",
            ),
            pixels("border.width", width, "Border thickness"),
        ],
    }
}

/// The colors of the reset note kinds.
fn reset_group(colors: ResetColors) -> TokenGroup {
    let ResetColors { daily, weekly } = colors;
    TokenGroup {
        title: "Reset kinds",
        tokens: vec![
            color("reset.daily", daily, "Badge and dropdown of daily notes"),
            color("reset.weekly", weekly, "Badge and dropdown of weekly notes"),
        ],
    }
}

/// The two colors of one chip category.
fn chip_tokens(
    [text_id, background_id]: [&'static str; 2],
    colors: ChipCategoryColors,
    purpose: &'static str,
) -> [Token; 2] {
    let ChipCategoryColors { text, background } = colors;
    [
        color(text_id, text, purpose),
        color(background_id, background, purpose),
    ]
}

/// The colors of the chat-link chip categories.
fn chip_group(colors: ChipColors) -> TokenGroup {
    let ChipColors {
        place,
        character,
        item,
        progress,
    } = colors;
    let categories = [
        (
            ["chip.place.text", "chip.place.background"],
            place,
            "Map points, WvW objectives",
        ),
        (
            ["chip.character.text", "chip.character.background"],
            character,
            "Skills, traits, builds",
        ),
        (
            ["chip.item.text", "chip.item.background"],
            item,
            "Items, recipes, skins, outfits, coins",
        ),
        (
            ["chip.progress.text", "chip.progress.background"],
            progress,
            "Achievements, NPC texts, players",
        ),
    ];
    TokenGroup {
        title: "Chat-link chips",
        tokens: categories
            .into_iter()
            .flat_map(|(ids, category, purpose)| chip_tokens(ids, category, purpose))
            .collect(),
    }
}

/// The coin colors.
fn coin_group(colors: CoinColors) -> TokenGroup {
    let CoinColors {
        gold,
        silver,
        copper,
    } = colors;
    TokenGroup {
        title: "Coins",
        tokens: vec![
            color("coin.gold", gold, "Gold amounts in chips"),
            color("coin.silver", silver, "Silver amounts in chips"),
            color("coin.copper", copper, "Copper amounts in chips"),
        ],
    }
}

/// The event timeline colors.
fn event_group(colors: EventColors) -> TokenGroup {
    let EventColors {
        default,
        highlight,
        active,
        upcoming,
        custom,
    } = colors;
    TokenGroup {
        title: "Events",
        tokens: vec![
            color("event.default", default, "Regular events (later addons)"),
            color(
                "event.highlight",
                highlight,
                "Highlighted events (later addons)",
            ),
            color("event.active", active, "Running events (later addons)"),
            color("event.upcoming", upcoming, "Upcoming events (later addons)"),
            color("event.custom", custom, "User-defined events (later addons)"),
        ],
    }
}

/// The spacing scale.
fn spacing_group(scale: Spacing) -> TokenGroup {
    let Spacing {
        extra_small,
        small,
        medium,
        large,
        extra_large,
    } = scale;
    TokenGroup {
        title: "Spacing",
        tokens: vec![
            pixels("space.xs", extra_small, "Tightest gap"),
            pixels("space.sm", small, "Item gap, compact dividers"),
            pixels("space.md", medium, "Section dividers"),
            pixels("space.lg", large, "Dialog padding"),
            pixels("space.xl", extra_large, "Largest gap"),
        ],
    }
}

/// The metrics of lists, panes and buttons.
fn layout_group(list: ListMetrics, pane: PaneMetrics, button: ButtonMetrics) -> TokenGroup {
    let ListMetrics {
        item_spacing,
        indent,
        selection_bar_width,
        selection_bar_inset,
    } = list;
    let PaneMetrics { padding, gap } = pane;
    let ButtonMetrics { min_width } = button;
    TokenGroup {
        title: "Layout",
        tokens: vec![
            pixels("list.item_spacing", item_spacing, "Gap between rows"),
            pixels("list.indent", indent, "Indentation per tree level"),
            pixels(
                "list.selection_bar_width",
                selection_bar_width,
                "Width of the selection bar",
            ),
            pixels(
                "list.selection_bar_inset",
                selection_bar_inset,
                "Gap of the bar to its row's ends",
            ),
            pixels("pane.padding", padding, "Inner padding of a pane"),
            pixels("pane.gap", gap, "Gap between neighboring panes"),
            Token {
                id: "button.min_width",
                value: TokenValue::FontSizeFactor(min_width),
                purpose: "Minimum width of a text button",
            },
        ],
    }
}

/// The corner radii.
fn radius_group(radii: Radii) -> TokenGroup {
    let Radii {
        small,
        medium,
        large,
    } = radii;
    TokenGroup {
        title: "Corner radii",
        tokens: vec![
            pixels("radius.sm", small, "Chips, badges, scrollbar grabs"),
            pixels(
                "radius.md",
                medium,
                "Buttons, inputs, dropdowns, rows, menu items",
            ),
            pixels("radius.lg", large, "Windows, panes, popups, dialogs"),
        ],
    }
}

/// The font sizes.
fn typography_group(typography: Typography) -> TokenGroup {
    let Typography {
        small,
        body,
        heading,
        line_height,
    } = typography;
    TokenGroup {
        title: "Typography",
        tokens: vec![
            pixels("font.size.small", small, "Small text"),
            pixels("font.size.body", body, "Body text"),
            pixels("font.size.heading", heading, "Headings"),
            Token {
                id: "font.line_height",
                value: TokenValue::FontSizeFactor(line_height),
                purpose: "Line height of running text",
            },
        ],
    }
}

/// Every token of `theme`, grouped. Destructuring without `..` makes a new
/// group fail to compile until it is documented.
fn token_groups(theme: Theme) -> Vec<TokenGroup> {
    let Theme {
        surface,
        text,
        accent,
        status,
        border,
        event,
        coin,
        reset,
        chip,
        spacing,
        list,
        pane,
        button,
        radius,
        typography,
    } = theme;
    vec![
        surface_group(surface),
        text_group(text),
        accent_group(accent),
        status_group(status),
        border_group(border),
        reset_group(reset),
        chip_group(chip),
        coin_group(coin),
        event_group(event),
        spacing_group(spacing),
        layout_group(list, pane, button),
        radius_group(radius),
        typography_group(typography),
    ]
}

/// A channel from 0 to 1 as a byte.
fn channel_byte(channel: f32) -> u8 {
    // The clamp keeps the rounded value inside `u8`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        (channel.clamp(0.0, 1.0) * 255.0).round() as u8
    }
}

/// `value` as shown in a table: `#RRGGBB`, with its alpha if translucent,
/// `4 px` or `1.5 ×`.
fn format_value(value: TokenValue) -> String {
    match value {
        TokenValue::Color(color) => {
            let hex = format!(
                "#{:02X}{:02X}{:02X}",
                channel_byte(color.red),
                channel_byte(color.green),
                channel_byte(color.blue)
            );
            if color.alpha < 1.0 {
                format!("{hex} at {:.0} %", color.alpha * 100.0)
            } else {
                hex
            }
        }
        TokenValue::Pixels(size) => format!("{size} px"),
        TokenValue::FontSizeFactor(factor) => format!("{factor} × font size"),
    }
}

/// The whole document for the built-in themes.
fn render_document() -> String {
    let mut document = String::from(
        "# Token values\n\n\
         Generated by `cargo xtask tokens` from `crates/trailkit-ui/src/theme.rs`; do\n\
         not edit by hand. `cargo test` fails when this file is out of date. When to\n\
         use which token is binding in [`tokens.md`](tokens.md).\n",
    );
    let dark_groups = token_groups(Theme::DARK);
    let light_groups = token_groups(Theme::LIGHT);
    for (dark_group, light_group) in dark_groups.iter().zip(&light_groups) {
        document.push_str(&format!(
            "\n## {}\n\n| Token | Dark | Light | Purpose |\n| --- | --- | --- | --- |\n",
            dark_group.title
        ));
        for (dark, light) in dark_group.tokens.iter().zip(&light_group.tokens) {
            document.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                dark.id,
                format_value(dark.value),
                format_value(light.value),
                dark.purpose
            ));
        }
    }
    document
}

/// Whether `committed` is `generated`, whatever line endings the checkout gave
/// it: Git may check text files out with CRLF on Windows.
fn is_same_document(committed: &str, generated: &str) -> bool {
    committed.replace("\r\n", "\n") == generated
}

/// Writes the document, or in [`Mode::Check`] verifies it.
///
/// # Errors
/// Returns an error if the file cannot be read or written, or — in
/// [`Mode::Check`] — if it is out of date.
pub fn run(mode: Mode) -> Result<()> {
    let path = workspace::root().join(DOCUMENT_PATH);
    let document = render_document();
    match mode {
        Mode::Write => {
            fs::write(&path, document).with_context(|| format!("writing {}", path.display()))?;
            println!("wrote {DOCUMENT_PATH}");
        }
        Mode::Check => {
            let committed =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            if !is_same_document(&committed, &document) {
                bail!("{DOCUMENT_PATH} is out of date; run `cargo xtask tokens`");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_document_is_current() {
        if let Err(error) = run(Mode::Check) {
            panic!("{error:#}");
        }
    }

    #[test]
    fn line_endings_of_the_checkout_do_not_matter() {
        assert!(is_same_document("a\r\nb\r\n", "a\nb\n"));
        assert!(is_same_document("a\nb\n", "a\nb\n"));
        assert!(!is_same_document("a\r\nc\r\n", "a\nb\n"));
    }

    #[test]
    fn values_are_formatted_for_reading() {
        let opaque = Color::from_rgb_hex(0x0F1318);
        assert_eq!(format_value(TokenValue::Color(opaque)), "#0F1318");
        assert_eq!(
            format_value(TokenValue::Color(opaque.with_alpha(0.24))),
            "#0F1318 at 24 %"
        );
        assert_eq!(format_value(TokenValue::Pixels(4.0)), "4 px");
        assert_eq!(
            format_value(TokenValue::FontSizeFactor(1.5)),
            "1.5 × font size"
        );
    }

    #[test]
    fn both_themes_document_the_same_tokens() {
        let ids = |theme| -> Vec<&'static str> {
            token_groups(theme)
                .iter()
                .flat_map(|group| group.tokens.iter().map(|token| token.id))
                .collect()
        };
        assert_eq!(ids(Theme::DARK), ids(Theme::LIGHT));
    }
}
