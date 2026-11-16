//! The note kind as the user sees it (M86, M95, M102, M193): the kind
//! options, the row badge with its tooltip and the kind dropdown behind the
//! note title.

use jiff::Timestamp;
use trailkit_memo_shared::{
    i18n::{Language, LocalizedText},
    note_tree::{NoteKind, ResetPeriod},
    row_badge::RowBadge,
};
use trailkit_ui::icons::Icon;
#[cfg(windows)]
use trailkit_ui::theme::Theme;

use crate::{
    i18n::{ResetText, reset_countdown},
    schedule::minutes_until_next_reset,
};

/// The note kinds in menu and dropdown order with their labels (M86).
pub const KIND_OPTIONS: [(Option<ResetPeriod>, ResetText); 3] = [
    (None, ResetText::KindNormal),
    (Some(ResetPeriod::Daily), ResetText::KindDaily),
    (Some(ResetPeriod::Weekly), ResetText::KindWeekly),
];

/// The row badge of a note (M45, M193): a calendar icon for reset notes,
/// `None` for normal notes.
#[must_use]
pub fn reset_badge(kind: NoteKind) -> Option<RowBadge> {
    let icon = match kind.period()? {
        ResetPeriod::Daily => Icon::CALENDAR_1,
        ResetPeriod::Weekly => Icon::CALENDAR_DAYS,
    };
    Some(RowBadge { icon })
}

/// The badge tooltip (M95): the kind and the countdown to its next reset.
/// Call only while the badge is hovered, so the countdown is not formatted
/// every frame.
#[must_use]
pub fn reset_tooltip_lines(period: ResetPeriod, language: Language, now: Timestamp) -> [String; 2] {
    let kind = match period {
        ResetPeriod::Daily => ResetText::KindDaily,
        ResetPeriod::Weekly => ResetText::KindWeekly,
    };
    [
        kind.get(language).to_owned(),
        reset_countdown(minutes_until_next_reset(period, now), language),
    ]
}

/// The kind dropdown's labels in [`KIND_OPTIONS`] order.
#[must_use]
pub fn kind_labels(language: Language) -> [&'static str; 3] {
    KIND_OPTIONS.map(|(_, text)| text.get(language))
}

/// Width and height of the kind dropdown, for placing it before drawing.
#[cfg(windows)]
#[must_use]
pub fn kind_dropdown_size(ui: &nexus::imgui::Ui, language: Language) -> [f32; 2] {
    [
        trailkit_ui::dropdown::dropdown_width(ui, &kind_labels(language)),
        ui.frame_height(),
    ]
}

/// Draws the kind dropdown at the cursor (M102, the one shared dropdown
/// since R71; untinted since M193). Returns the chosen period
/// (`Some(None)` = normal note) when the user picked a kind.
#[cfg(windows)]
pub fn draw_kind_dropdown(
    ui: &nexus::imgui::Ui,
    theme: &Theme,
    language: Language,
    kind: NoteKind,
) -> Option<Option<ResetPeriod>> {
    let selected = KIND_OPTIONS
        .iter()
        .position(|(period, _)| *period == kind.period())
        .unwrap_or(0);
    let index = trailkit_ui::dropdown::draw_dropdown(
        ui,
        theme,
        "kind",
        ResetText::ResetType.get(language),
        &kind_labels(language),
        selected,
    )?;
    KIND_OPTIONS.get(index).map(|(period, _)| *period)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_reset_notes_have_a_badge() {
        assert_eq!(reset_badge(NoteKind::Normal), None);
        let weekly = NoteKind::with_period(Some(ResetPeriod::Weekly), Timestamp::UNIX_EPOCH);
        assert_eq!(
            reset_badge(weekly),
            Some(RowBadge {
                icon: Icon::CALENDAR_DAYS
            })
        );
    }
}
