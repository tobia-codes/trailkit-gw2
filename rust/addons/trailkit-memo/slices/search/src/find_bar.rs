//! The find bar of the open note (memo plan M198, mockup
//! https://claude.ai/artifact/S2rNsD4TnkoemgMULKerJc variant A): fixed below
//! the note toolbar, it holds the find field, "2 of 3" or "No results" in
//! `danger`, previous and next (Lucide `arrow-up`, `arrow-down`) and close,
//! with a divider below. A pane too narrow for a usable field gives the field
//! the whole width and moves the rest to a second line, right-aligned
//! (`lay-responsive`). Drawing only; the window owns the find state.

use nexus::imgui::Ui;
use trailkit_ui::{
    controls::button::{
        ButtonState, draw_icon_button, draw_icon_button_with_state, icon_button_width,
    },
    drawing::draw_list::window_draw_list,
    layout::separator::draw_compact_separator,
    text::TextStyle,
    text::icons::Icon,
    text::text_layout::body_characters_width,
    theme::Theme,
};

use crate::search_field::{SearchField, field_height};

/// Below this many average body characters the field takes the whole width
/// and the count and buttons move to a second line.
const MINIMUM_FIELD_CHARACTERS: f32 = 10.0;

/// The texts the bar shows, in the addon's language.
#[derive(Debug, Clone, Copy)]
pub struct FindBarTexts<'text> {
    /// Placeholder of the empty field ("Find in note …").
    pub hint: &'text str,
    /// The position among the hits, e.g. "2 of 3".
    pub count: &'text str,
    /// Shown instead of the count when nothing matches.
    pub no_results: &'text str,
    /// Tooltip of the previous button.
    pub previous: &'text str,
    /// Tooltip of the next button.
    pub next: &'text str,
    /// Tooltip of the close button.
    pub close: &'text str,
}

/// What the bar shows besides the field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindBarStatus {
    /// The field is empty: no count, the arrows are disabled.
    Empty,
    /// The field holds text that matches nothing.
    NoResults,
    /// At least one hit: the count text and working arrows.
    Hits,
}

/// What the user did with the bar in this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FindBarResponse {
    /// The field text changed.
    pub text_changed: bool,
    /// Go to the previous hit (button).
    pub previous: bool,
    /// Go to the next hit (button).
    pub next: bool,
    /// Close the bar (button).
    pub close: bool,
}

/// Draws the bar over the full available width at the cursor, with
/// `field` as its input, and leaves the cursor below its divider.
pub fn draw_find_bar(
    ui: &Ui,
    theme: &Theme,
    field: &mut SearchField,
    status: FindBarStatus,
    texts: FindBarTexts<'_>,
) -> FindBarResponse {
    let gap = theme.spacing.small;
    let label = match status {
        FindBarStatus::Empty => "",
        FindBarStatus::NoResults => texts.no_results,
        FindBarStatus::Hits => texts.count,
    };
    let label_width = {
        let _font = TextStyle::Secondary.push(ui, theme);
        ui.calc_text_size(label)[0]
    };
    let buttons_width = icon_button_width(ui, Icon::ARROW_UP)
        + icon_button_width(ui, Icon::ARROW_DOWN)
        + icon_button_width(ui, Icon::X)
        + 2.0 * gap;
    let trailing = buttons_width
        + if label.is_empty() {
            0.0
        } else {
            label_width + 2.0 * gap
        };
    let available = ui.content_region_avail()[0];
    let wraps = available - trailing < body_characters_width(ui, MINIMUM_FIELD_CHARACTERS);
    let field_width = if wraps {
        available
    } else {
        available - trailing
    };

    let start = ui.cursor_screen_pos();
    let field_response = field.draw_without_end_button(ui, theme, texts.hint, field_width);
    // The field's own height; the cursor below it also holds the item spacing.
    let field_height = field_height(ui, theme);
    let mut response = FindBarResponse {
        text_changed: field_response.text_changed,
        ..FindBarResponse::default()
    };

    // The count sits centered on the field's middle, or on a second line at
    // the right end; "No results" in `danger`.
    let (row_top, mut x) = if wraps {
        (
            start[1] + field_height + gap / 2.0,
            start[0] + (available - trailing + gap).max(0.0),
        )
    } else {
        (start[1], start[0] + field_width + gap)
    };
    let middle = row_top + field_height / 2.0;
    if !label.is_empty() {
        let _font = TextStyle::Secondary.push(ui, theme);
        let color = if status == FindBarStatus::NoResults {
            theme.status.danger
        } else {
            theme.text.secondary
        };
        let top = middle - ui.text_line_height() / 2.0;
        window_draw_list(ui).add_text([x, top], color.to_array(), label);
        x += label_width + gap;
    }

    let button_top = middle - ui.frame_height() / 2.0;
    // Disabled arrows say why (trailkit-ui principle 4).
    let arrows = match status {
        FindBarStatus::Hits => ButtonState::Ready,
        FindBarStatus::NoResults => ButtonState::Disabled {
            reason: texts.no_results,
        },
        FindBarStatus::Empty => ButtonState::Disabled { reason: texts.hint },
    };
    ui.set_cursor_screen_pos([x, button_top]);
    response.previous =
        draw_icon_button_with_state(ui, theme, Icon::ARROW_UP, texts.previous, arrows);
    ui.same_line_with_spacing(0.0, gap / 2.0);
    response.next = draw_icon_button_with_state(ui, theme, Icon::ARROW_DOWN, texts.next, arrows);
    ui.same_line_with_spacing(0.0, gap / 2.0);
    response.close = draw_icon_button(ui, theme, Icon::X, texts.close);

    let spacing = ui.clone_style().item_spacing[1];
    ui.set_cursor_screen_pos([start[0], row_top + field_height + spacing]);
    draw_compact_separator(ui, theme);
    response
}
