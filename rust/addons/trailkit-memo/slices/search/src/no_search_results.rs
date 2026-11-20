//! The empty search result (M126), drawn by the app instead of the note
//! list when nothing matches (ROADMAP R76, question 49).

use nexus::imgui::Ui;
use trailkit_memo_shared::i18n::{Language, LocalizedText};
use trailkit_ui::{
    controls::button::{button_width, draw_button},
    controls::text_field::move_below_last_item,
    text::TextStyle,
    text::text_layout::{centered_text, indent_to_center},
    theme::Theme,
};

use crate::i18n::SearchText;

/// Draws what was searched for (`text`) and, `space.sm` below, a button
/// that clears the search; no space above, the caller's layout places it
/// (R148). Returns whether the button was clicked.
pub fn draw_no_search_results(ui: &Ui, language: Language, theme: &Theme, text: &str) -> bool {
    let width = ui.content_region_avail()[0];
    let message = SearchText::NoSearchResults
        .get(language)
        .replace("{query}", text.trim());
    {
        // Centering measures the text, so the style is pushed (Q281).
        let _secondary = TextStyle::Secondary.push(ui, theme);
        centered_text(ui, &message, width);
    }
    move_below_last_item(ui, theme.spacing.small);
    let label = SearchText::ClearSearch.get(language);
    indent_to_center(ui, button_width(ui, theme, label), width);
    draw_button(ui, theme, label)
}
