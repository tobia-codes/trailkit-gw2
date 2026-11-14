//! Snapshots of the title bar: its buttons and toggles in every state (R82).

use super::*;

#[test]
fn title_bar_states_match_snapshots() {
    let size = [320.0, 120.0];
    let toggles = [
        TitleBarToggle {
            icon: Icon::SEARCH,
            tooltip: "Tools",
            active: false,
            placement: TogglePlacement::Leading,
        },
        TitleBarToggle {
            icon: Icon::X,
            tooltip: "Note list",
            active: true,
            placement: TogglePlacement::Leading,
        },
    ];
    let texts = TitleBarTexts {
        title: "Notes",
        collapse: "Collapse",
        expand: "Expand",
        close: "Close",
    };
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut state = TitleBarState::default();
        let mut open = true;
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
            let _clicked = draw_title_bar(
                ui,
                &theme,
                Window::new("Notes###title")
                    .position([0.0, 0.0], Condition::Always)
                    .size(size, Condition::Always),
                texts,
                &toggles,
                &mut state,
                &mut open,
                || {},
            );
        };
        let names = |state| ("title-bar", state, theme_name);
        let collapse = [size[0] - 64.0, 25.0];
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([160.0, 100.0], false),
            &mut draw,
        );
        for (state_name, position) in [
            ("hover-toggle-off", [28.0, 25.0]),
            ("hover-toggle-on", [68.0, 25.0]),
            ("hover-collapse", collapse),
            ("hover-close", [size[0] - 28.0, 25.0]),
        ] {
            assert_state(
                &mut harness,
                names(state_name),
                &mouse(position, false),
                &mut draw,
            );
        }
        harness.settle(1, &mouse(collapse, false), &mut draw);
        harness.settle(1, &mouse(collapse, true), &mut draw);
        harness.settle(1, &mouse(collapse, false), &mut draw);
        assert_state(
            &mut harness,
            names("collapsed"),
            &mouse([160.0, 100.0], false),
            &mut draw,
        );
    }
}

/// The help `?` at the right end of the title row (memo Q171), next to a
/// leading toggle, at rest, hovered, and active while the help is open.
#[test]
fn trailing_toggle_states_match_snapshots() {
    let size = [320.0, 120.0];
    let texts = TitleBarTexts {
        title: "Notes",
        collapse: "Collapse",
        expand: "Expand",
        close: "Close",
    };
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (state_name, help_open, position) in [
            ("rest", false, [160.0, 100.0]),
            ("hover", false, [size[0] - 100.0, 25.0]),
            ("active", true, [160.0, 100.0]),
        ] {
            let toggles = [
                TitleBarToggle {
                    icon: Icon::MENU,
                    tooltip: "Menu",
                    active: false,
                    placement: TogglePlacement::Leading,
                },
                TitleBarToggle {
                    icon: Icon::CIRCLE_HELP,
                    tooltip: "Help",
                    active: help_open,
                    placement: TogglePlacement::Trailing,
                },
            ];
            let mut state = TitleBarState::default();
            let mut open = true;
            let mut harness = UiHarness::new(size);
            let mut draw = |ui: &Ui| {
                let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
                let _clicked = draw_title_bar(
                    ui,
                    &theme,
                    Window::new("Notes###title")
                        .position([0.0, 0.0], Condition::Always)
                        .size(size, Condition::Always),
                    texts,
                    &toggles,
                    &mut state,
                    &mut open,
                    || {},
                );
            };
            assert_state(
                &mut harness,
                ("title-bar-help", state_name, theme_name),
                &mouse(position, false),
                &mut draw,
            );
        }
    }
}
