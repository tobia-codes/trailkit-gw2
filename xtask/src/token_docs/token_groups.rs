//! The tables of the token reference, one function per token group. Every token
//! struct is destructured without `..`, so a new token does not compile until it
//! is documented here (ROADMAP R84).

use trailkit_ui::{
    color::Color,
    theme::{
        AccentColors, BorderTokens, ButtonLayout, ChipCategoryColors, ChipColors, CoinColors,
        EventColors, ListLayout, MenuLayout, PaneLayout, Radii, ResetColors, ShadowTokens, Spacing,
        StatusColors, SurfaceColors, TextColors, Theme, Typography,
    },
};

/// The value of a token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TokenValue {
    /// A color.
    Color(Color),
    /// A size in pixels.
    Pixels(f32),
    /// A multiple of the current font size.
    FontSizeFactor(f32),
    /// A multiple of the body size (Nexus' `FontUI` × text scale), shown
    /// with two decimals (R96).
    BodySizeFactor(f32),
}

/// One documented token.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Token {
    /// The token id used in the design documents, e.g. `bg.window`.
    pub(super) id: &'static str,
    /// Its value in a theme.
    pub(super) value: TokenValue,
    /// What it is for.
    pub(super) purpose: &'static str,
}

/// A titled table of tokens.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct TokenGroup {
    /// The heading of the table.
    pub(super) title: &'static str,
    /// The tokens, in reading order.
    pub(super) tokens: Vec<Token>,
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

/// A token sized as a multiple of the body size.
const fn body_factor(id: &'static str, value: f32, purpose: &'static str) -> Token {
    Token {
        id,
        value: TokenValue::BodySizeFactor(value),
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
            color(
                "accent.hover",
                hover,
                "Hovered primary button (8 % darker, R107)",
            ),
            color(
                "accent.active",
                active,
                "Pressed primary button (12 % darker, R107)",
            ),
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
        danger_hover,
        danger_active,
        danger_subtle,
        warning_subtle,
        info,
    } = colors;
    TokenGroup {
        title: "Status",
        tokens: vec![
            color("success", success, "Success"),
            color("warning", warning, "Warnings, invalid chat links"),
            color(
                "danger",
                danger,
                "Destructive actions, fill of delete buttons",
            ),
            color(
                "danger.hover",
                danger_hover,
                "Hovered delete button (8 % darker, R107)",
            ),
            color(
                "danger.active",
                danger_active,
                "Pressed delete button (12 % darker, R107)",
            ),
            color(
                "danger.subtle",
                danger_subtle,
                "Fill of an error toast or a danger status card",
            ),
            color(
                "warning.subtle",
                warning_subtle,
                "Fill of a warning status card",
            ),
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

/// The tooltip shadow (Q327), the one shadow of the design.
fn shadow_group(tokens: ShadowTokens) -> TokenGroup {
    let ShadowTokens {
        popup,
        popup_offset,
    } = tokens;
    TokenGroup {
        title: "Shadows",
        tokens: vec![
            color("shadow.popup", popup, "Shadow below tooltips"),
            pixels(
                "shadow.popup.offset",
                popup_offset,
                "How far it reaches below",
            ),
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

/// The layout of lists, panes, buttons and menus.
fn layout_group(
    list: ListLayout,
    pane: PaneLayout,
    button: ButtonLayout,
    menu: MenuLayout,
) -> TokenGroup {
    let ListLayout {
        item_spacing,
        indent,
        selection_bar_width,
        selection_bar_inset,
    } = list;
    let PaneLayout { padding, gap } = pane;
    let ButtonLayout { min_width } = button;
    let MenuLayout {
        padding: menu_padding,
        row_padding_x,
        row_padding_y,
        row_gap,
        min_width: menu_min_width,
    } = menu;
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
            pixels(
                "menu.padding",
                menu_padding,
                "Gap between a menu's edge and its rows",
            ),
            pixels(
                "menu.row_padding_x",
                row_padding_x,
                "Gap between a menu row's edge and its content",
            ),
            pixels(
                "menu.row_padding_y",
                row_padding_y,
                "Gap between a menu row's edge and its text line",
            ),
            pixels("menu.row_gap", row_gap, "Gap between two menu rows"),
            Token {
                id: "menu.min_width",
                value: TokenValue::FontSizeFactor(menu_min_width),
                purpose: "Minimum width of a command menu",
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
            pixels(
                "radius.sm",
                small,
                "Chips, badges, scrollbar grabs, tooltips",
            ),
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
        heading3,
        heading2,
        heading1,
        line_height,
    } = typography;
    TokenGroup {
        title: "Typography",
        tokens: vec![
            body_factor("font.size.small", small, "Small text"),
            body_factor("font.size.body", body, "Body text"),
            body_factor("font.size.heading3", heading3, "Level 3 heading and below"),
            body_factor("font.size.heading2", heading2, "Level 2 heading"),
            body_factor("font.size.heading1", heading1, "Level 1 heading"),
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
pub(super) fn token_groups(theme: Theme) -> Vec<TokenGroup> {
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
        menu,
        radius,
        shadow,
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
        layout_group(list, pane, button, menu),
        radius_group(radius),
        shadow_group(shadow),
        typography_group(typography),
    ]
}
