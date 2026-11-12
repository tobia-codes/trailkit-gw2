//! Snapshots of the title bar `?` that opens the help window (memo Q171,
//! ROADMAP R100): at rest, hovered and active, Dark and Light, German, at
//! the minimum width. The other snapshots draw the window without it.

use super::*;

/// Center of the `?` at the minimum width: left of the chevron and the `x`.
const HELP_BUTTON: [f32; 2] = [MINIMUM_WINDOW[0] - 100.0, 25.0];

/// Draws `memo` like the addon does, with the help `?` after its toggles.
fn draw_with_help(
    ui: &Ui,
    memo: &mut MemoWindow,
    theme: ThemeKind,
    language: Language,
    help_open: bool,
) {
    let theme = theme.theme();
    let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
    let mut title_bar = TitleBarState::default();
    let mut open = true;
    let [menu] = memo.title_bar_toggles(language);
    let toggles = [menu, help_title_bar_toggle(language, help_open)];
    let _clicked = draw_window(
        ui,
        &theme,
        Window::new("Notes###memo")
            .position([0.0, 0.0], Condition::Always)
            .size(MINIMUM_WINDOW, Condition::Always),
        title_bar_texts(language),
        &toggles,
        &mut title_bar,
        &mut open,
        || memo.draw(ui, language, &theme),
    );
}

#[test]
fn every_help_button_state() {
    let document = sample_document();
    let away = [MINIMUM_WINDOW[0] / 2.0, MINIMUM_WINDOW[1] - 4.0];
    for (name, theme, language, help_open, mouse) in [
        ("rest-dark", ThemeKind::Dark, Language::English, false, away),
        (
            "rest-light",
            ThemeKind::Light,
            Language::English,
            false,
            away,
        ),
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
            draw_with_help(ui, &mut memo, theme, language, help_open);
        });
        assert_snapshot(
            env!("CARGO_MANIFEST_DIR"),
            &format!("memo-title-bar-help-{name}"),
            &image,
        );
    }
}
