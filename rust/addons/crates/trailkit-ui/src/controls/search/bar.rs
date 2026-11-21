//! The search field with its count and arrows (BL30, Q581, mock v9; find bar
//! of memo M198). "2 of 3", or "No results" in `danger`, and previous and
//! next (Lucide `arrow-up`, `arrow-down`) sit either in a row below the field
//! (a list's search) or beside it (the find bar of an open page). Beside the
//! field they move to the row below when the field would be narrower than
//! [`MINIMUM_FIELD_CHARACTERS`] (`lay-responsive`). The row below looks the
//! same in both: the count at the left, the arrows at the right end.
//! Drawing only; the app owns the hits.

use nexus::imgui::Ui;

use crate::{
    controls::button::{ButtonState, draw_icon_button_with_state, icon_button_width},
    controls::search::field::{SearchField, SearchFieldResponse, SearchFieldTexts, Submit},
    drawing::draw_list::window_draw_list,
    text::TextStyle,
    text::icons::Icon,
    text::text_layout::body_characters_width,
    theme::Theme,
};

/// Below this many average body characters the field beside the count takes
/// the whole width and the count and arrows move to a row below (mock v9).
pub const MINIMUM_FIELD_CHARACTERS: f32 = 12.0;

/// Where the count and the arrows sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountPlacement {
    /// In a row below the field, shown only while there are hits: the
    /// search above a list, whose empty result says "no results" itself.
    Below,
    /// Beside the field, wrapping below it when the field gets too narrow:
    /// the find bar of an open page (Ctrl+F).
    Beside,
}

/// What the search has found, for the count and the arrows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStatus {
    /// The field is empty: no count; the arrows say why they wait.
    Empty,
    /// The field holds text that matches nothing.
    NoResults,
    /// At least one hit: the count text and working arrows.
    Hits,
}

/// The texts the search shows, in the addon's language.
#[derive(Debug, Clone, Copy)]
pub struct SearchBarTexts<'text> {
    /// The field's placeholder, clear and close tooltips.
    pub field: SearchFieldTexts<'text>,
    /// The position among the hits, e.g. "2 of 3".
    pub count: &'text str,
    /// Shown instead of the count when nothing matches.
    pub no_results: &'text str,
    /// Tooltip of the previous button.
    pub previous: &'text str,
    /// Tooltip of the next button.
    pub next: &'text str,
}

/// What the user did with the search in this frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SearchBarResponse {
    /// The field text changed.
    pub text_changed: bool,
    /// The `x` of the empty closable field was clicked: close the search.
    pub close_clicked: bool,
    /// Go to the previous hit (button or Shift+Enter).
    pub previous: bool,
    /// Go to the next hit (button or Enter).
    pub next: bool,
}

impl From<SearchFieldResponse> for SearchBarResponse {
    /// The field's part of the response: typing, the `x` and Enter.
    fn from(field: SearchFieldResponse) -> Self {
        Self {
            text_changed: field.text_changed,
            close_clicked: field.close_clicked,
            previous: field.submitted == Some(Submit::Backward),
            next: field.submitted == Some(Submit::Forward),
        }
    }
}

/// Draws the search over the full available width at the cursor, with
/// `field` as its input, and leaves the cursor below it; the caller's stack
/// puts the divider below (R148).
pub fn draw_search_bar(
    ui: &Ui,
    theme: &Theme,
    field: &mut SearchField,
    status: SearchStatus,
    placement: CountPlacement,
    texts: SearchBarTexts<'_>,
) -> SearchBarResponse {
    let gap = theme.spacing.small;
    let label = match status {
        SearchStatus::Empty => "",
        SearchStatus::NoResults => texts.no_results,
        SearchStatus::Hits => texts.count,
    };
    let label_width = {
        let _font = TextStyle::Secondary.push(ui, theme);
        ui.calc_text_size(label)[0]
    };
    let arrows_width =
        icon_button_width(ui, Icon::ARROW_UP) + icon_button_width(ui, Icon::ARROW_DOWN) + gap / 2.0;
    let beside_width = arrows_width
        + if label.is_empty() {
            gap
        } else {
            label_width + 2.0 * gap
        };
    let available = ui.content_region_avail()[0];
    let row = match placement {
        CountPlacement::Below => (status == SearchStatus::Hits).then_some(Row::Below),
        CountPlacement::Beside => Some(
            if available - beside_width < body_characters_width(ui, MINIMUM_FIELD_CHARACTERS) {
                Row::Below
            } else {
                Row::Beside
            },
        ),
    };
    let field_width = if row == Some(Row::Beside) {
        available - beside_width
    } else {
        available
    };

    let start = ui.cursor_screen_pos();
    let mut response: SearchBarResponse =
        field.draw_sized(ui, theme, texts.field, field_width).into();
    let field_end = ui.item_rect_max()[1];
    let Some(row) = row else {
        return response;
    };

    // Beside: the count centered on the field's middle after the field.
    // Below: a row a small gap under the field, the count at its left and
    // the arrows at its right end, like the mock's counter row.
    let button_height = ui.frame_height();
    let (middle, label_x, arrows_x) = match row {
        Row::Beside => {
            let middle = (start[1] + field_end) / 2.0;
            let label_x = start[0] + field_width + gap;
            let arrows_x = start[0] + available - arrows_width;
            (middle, label_x, arrows_x)
        }
        Row::Below => {
            let middle = field_end + gap / 2.0 + button_height / 2.0;
            (middle, start[0], start[0] + available - arrows_width)
        }
    };
    if !label.is_empty() {
        let _font = TextStyle::Secondary.push(ui, theme);
        let color = if status == SearchStatus::NoResults {
            theme.status.danger
        } else {
            theme.text.secondary
        };
        let top = middle - ui.text_line_height() / 2.0;
        window_draw_list(ui).add_text([label_x, top], color.to_array(), label);
    }

    // Disabled arrows say why (trailkit-ui principle 4).
    let arrows = match status {
        SearchStatus::Hits => ButtonState::Ready,
        SearchStatus::NoResults => ButtonState::Disabled {
            reason: texts.no_results,
        },
        SearchStatus::Empty => ButtonState::Disabled {
            reason: texts.field.hint,
        },
    };
    ui.set_cursor_screen_pos([arrows_x, middle - button_height / 2.0]);
    response.previous |=
        draw_icon_button_with_state(ui, theme, Icon::ARROW_UP, texts.previous, arrows);
    ui.same_line_with_spacing(0.0, gap / 2.0);
    response.next |= draw_icon_button_with_state(ui, theme, Icon::ARROW_DOWN, texts.next, arrows);
    // Leave the cursor below the lower of field and row, as one item would.
    let bottom = match row {
        Row::Beside => field_end,
        Row::Below => middle + button_height / 2.0,
    };
    ui.set_cursor_screen_pos([start[0], bottom]);
    ui.dummy([0.0, 0.0]);
    response
}

/// Where the count row ended up in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Row {
    /// Beside the field.
    Beside,
    /// Below the field.
    Below,
}
