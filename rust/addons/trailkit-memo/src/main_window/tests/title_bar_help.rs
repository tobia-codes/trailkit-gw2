//! Snapshots of the title bar `?` that opens the help window (memo Q171,
//! ROADMAP R100): hovered and active, Dark and Light, German, at
//! the minimum width. The other snapshots show it at rest.

use super::*;

/// Center of the `?` at the minimum width: left of the chevron and the `x`.
const HELP_BUTTON: [f32; 2] = [MINIMUM_WINDOW[0] - 100.0, 25.0];

#[test]
fn help_button_states_match_snapshots() {
    let document = sample_document();
    let away = [MINIMUM_WINDOW[0] / 2.0, MINIMUM_WINDOW[1] - 4.0];
    for (name, theme, language, help_open, mouse) in [
        (
            "hover-dark",
            ThemeKind::Dark,
            Language::English,
            false,
            HELP_BUTTON,
        ),
        (
            "hover-de-light",
            ThemeKind::Light,
            Language::German,
            false,
            HELP_BUTTON,
        ),
        (
            "active-dark",
            ThemeKind::Dark,
            Language::English,
            true,
            away,
        ),
        (
            "active-light",
            ThemeKind::Light,
            Language::English,
            true,
            away,
        ),
    ] {
        let (mut memo, _worker, _directory) =
            memo_window(document.clone(), selected_first(&document));
        let mut harness = UiHarness::new(MINIMUM_WINDOW);
        let input = FrameInput {
            mouse_position: mouse,
            ..FrameInput::default()
        };
        let image = harness.settle(4, &input, |ui| {
            draw_like_the_addon(ui, &mut memo, MINIMUM_WINDOW, theme, language, help_open);
        });
        assert_snapshot(
            env!("CARGO_MANIFEST_DIR"),
            &format!("memo-title-bar-help-{name}"),
            &image,
        );
    }
}
