//! The card above the panes that tells about notes with unsaved changes
//! (M185): how many there are, that their rows are marked, and "Go to note".
//! Its close button hides it until the next game start; it disappears by
//! itself when no draft is left.

use nexus::imgui::Ui;
use trailkit_memo_shared::i18n::{Language, LocalizedText, Text};
use trailkit_ui::{
    button::Button,
    status_card::{CardStatus, StatusCard, draw_status_card},
    theme::Theme,
};

use crate::i18n::DraftText;

/// What the user chose on the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftOverviewAction {
    /// Show the next note with unsaved changes.
    GoToNote,
}

/// State of the card that outlives a frame.
#[derive(Debug, Default)]
pub struct DraftOverview {
    /// The sentence for (count, language), formatted only when either
    /// changes.
    label: Option<(usize, Language, String)>,
    /// Closed with its `x` in this session (M185).
    closed: bool,
}

impl DraftOverview {
    /// Draws the card for `count` notes with unsaved changes, unless it was
    /// closed or `count` is zero. Returns the user's choice.
    pub fn draw(
        &mut self,
        ui: &Ui,
        language: Language,
        theme: &Theme,
        count: usize,
    ) -> Option<DraftOverviewAction> {
        if self.closed || count == 0 {
            return None;
        }
        let label = match &mut self.label {
            Some((cached_count, cached_language, label))
                if *cached_count == count && *cached_language == language =>
            {
                label
            }
            label => {
                let text = if count == 1 {
                    DraftText::OverviewOne.get(language).to_owned()
                } else {
                    DraftText::OverviewMany
                        .get(language)
                        .replace("{count}", &count.to_string())
                };
                &mut label.insert((count, language, text)).2
            }
        };
        let buttons = [Button::new(DraftText::GoToNote.get(language))];
        let outcome = draw_status_card(
            ui,
            theme,
            &StatusCard {
                status: CardStatus::Warning,
                text: label,
                detail: None,
                buttons: &buttons,
                close_tooltip: Some(Text::CloseWindow.get(language)),
            },
        );
        if outcome.closed {
            self.closed = true;
        }
        outcome
            .clicked_button
            .map(|_| DraftOverviewAction::GoToNote)
    }
}
