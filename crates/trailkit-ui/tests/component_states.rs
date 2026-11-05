//! Snapshots of every shared component in every interaction state and both
//! themes (ROADMAP R82): the reference images are reviewed against the design
//! system (`tokens.md` §10).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui). `cargo xtask ui-test --update` rewrites the
//! reference images in `tests/snapshots/`.
#![cfg(windows)]

use std::cell::Cell;

use nexus::imgui::{Condition, Ui, Window, WindowFlags};
use trailkit_ui::{
    button::{
        draw_button, draw_danger_button, draw_danger_icon_button, draw_icon_button,
        draw_icon_toggle, draw_primary_button, draw_primary_button_with_icon,
    },
    dropdown::draw_dropdown,
    icons::Icon,
    menu::{MenuItemTone, begin_submenu, draw_menu_item, draw_menu_item_with_icon, draw_popup},
    pane::draw_pane,
    separator::draw_compact_separator,
    splitter::draw_vertical_splitter,
    style_scope::StyleScope,
    theme::{Theme, ThemeKind},
    title_bar::{TitleBarState, TitleBarTexts, TitleBarToggle, draw_window as draw_title_bar},
    toast::{Toast, draw_toast},
    tool_rail::{ToolRailItem, draw_tool_rail},
};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

/// Both themes with the suffix of their snapshot names.
const THEMES: [(ThemeKind, &str); 2] = [(ThemeKind::Dark, "dark"), (ThemeKind::Light, "light")];

/// The frames a state is held so that hover and layout settle.
const SETTLE_FRAMES: usize = 3;

/// Input with the mouse at `position`, optionally holding the left button.
fn mouse(position: [f32; 2], down: bool) -> FrameInput {
    FrameInput {
        mouse_position: position,
        left_button_down: down,
        ..FrameInput::default()
    }
}

/// Draws `content` in a borderless window of `size` at the image origin, with
/// the style of `theme`.
fn in_window(ui: &Ui, theme: &Theme, size: [f32; 2], content: impl FnOnce()) {
    let _style = StyleScope::push_with_cursor_extent(ui, theme, None);
    Window::new("component")
        .position([0.0, 0.0], Condition::Always)
        .size(size, Condition::Always)
        .flags(WindowFlags::NO_TITLE_BAR | WindowFlags::NO_RESIZE)
        .build(ui, content);
}

/// Holds `input` for a few frames and compares the result with the reference
/// `<component>-<state>-<theme>`.
fn assert_state(
    harness: &mut UiHarness,
    (component, state, theme_name): (&str, &str, &str),
    input: &FrameInput,
    draw: &mut impl FnMut(&Ui),
) {
    let image = harness.settle(SETTLE_FRAMES, input, &mut *draw);
    assert_snapshot(
        env!("CARGO_MANIFEST_DIR"),
        &format!("ui-{component}-{state}-{theme_name}"),
        &image,
    );
}

/// A component drawn by the button test.
type DrawButton = fn(&Ui, &Theme);

/// Every button style with its snapshot name.
const BUTTONS: [(&str, DrawButton); 8] = [
    ("button-regular", |ui, theme| {
        draw_button(ui, theme, "Cancel");
    }),
    ("button-primary", |ui, theme| {
        draw_primary_button(ui, theme, "Save");
    }),
    ("button-primary-icon", |ui, theme| {
        draw_primary_button_with_icon(ui, theme, Icon::TRASH_2, "Restore");
    }),
    ("button-danger", |ui, theme| {
        draw_danger_button(ui, theme, "Empty");
    }),
    ("button-danger-icon", |ui, theme| {
        draw_danger_icon_button(ui, theme, Icon::TRASH_2, "Delete");
    }),
    ("button-icon", |ui, theme| {
        draw_icon_button(ui, theme, Icon::X, "Close");
    }),
    ("button-toggle-off", |ui, theme| {
        draw_icon_toggle(ui, theme, Icon::SEARCH, "Search", false);
    }),
    ("button-toggle-on", |ui, theme| {
        draw_icon_toggle(ui, theme, Icon::SEARCH, "Search", true);
    }),
];

#[test]
fn every_button_at_rest_hovered_and_pressed() {
    let size = [220.0, 90.0];
    let over_button = [20.0, 22.0];
    let mut harness = UiHarness::new(size);
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (name, draw_one) in BUTTONS {
            let mut draw = |ui: &Ui| in_window(ui, &theme, size, || draw_one(ui, &theme));
            let names = |state| (name, state, theme_name);
            assert_state(
                &mut harness,
                names("rest"),
                &mouse([200.0, 80.0], false),
                &mut draw,
            );
            assert_state(
                &mut harness,
                names("hover"),
                &mouse(over_button, false),
                &mut draw,
            );
            assert_state(
                &mut harness,
                names("pressed"),
                &mouse(over_button, true),
                &mut draw,
            );
            // Release away from the button: nothing is clicked.
            harness.settle(2, &mouse([200.0, 80.0], false), &mut draw);
        }
    }
}

/// Draws a menu with every kind of entry; the popup opens in the first frame.
fn draw_sample_menu(ui: &Ui, theme: &Theme, opened: &Cell<bool>) {
    if !opened.replace(true) {
        ui.open_popup("menu");
    }
    draw_popup(ui, "menu", || {
        draw_menu_item(ui, theme, "Rename", false);
        draw_menu_item(ui, theme, "Checked", true);
        if let Some(_submenu) = begin_submenu(ui, theme, "Move to") {
            draw_menu_item(ui, theme, "Normal", true);
            draw_menu_item(ui, theme, "Daily reset", false);
        }
        draw_compact_separator(ui, theme);
        draw_menu_item_with_icon(
            ui,
            theme,
            Icon::ROTATE_CCW,
            "Restore",
            MenuItemTone::Regular,
            None,
        );
        draw_menu_item_with_icon(
            ui,
            theme,
            Icon::TRASH_2,
            "Delete",
            MenuItemTone::Danger,
            None,
        );
        draw_menu_item_with_icon(
            ui,
            theme,
            Icon::TRASH_2,
            "Blocked",
            MenuItemTone::Regular,
            Some("Finish editing first"),
        );
    });
}

#[test]
fn every_menu_entry_at_rest_and_hovered() {
    let size = [340.0, 240.0];
    let opening = [12.0, 12.0];
    let row = |offset: f32| [opening[0] + 40.0, opening[1] + offset];
    let states = [
        ("hover-plain", row(20.0)),
        ("hover-checked", row(44.0)),
        ("hover-icon", row(100.0)),
        ("hover-danger", row(124.0)),
        ("hover-disabled", row(148.0)),
        ("submenu-open", row(68.0)),
    ];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let opened = Cell::new(false);
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || draw_sample_menu(ui, &theme, &opened));
        };
        let names = |state| ("menu", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse(opening, false),
            &mut draw,
        );
        for (state, position) in states {
            assert_state(
                &mut harness,
                names(state),
                &mouse(position, false),
                &mut draw,
            );
        }
        // Pressing a row last: releasing it would close the menu.
        harness.settle(2, &mouse(row(20.0), false), &mut draw);
        assert_state(
            &mut harness,
            names("pressed"),
            &mouse(row(20.0), true),
            &mut draw,
        );
    }
}

#[test]
fn every_dropdown_state() {
    let size = [260.0, 170.0];
    let field = [60.0, 26.0];
    let away = [240.0, 160.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (name, tint) in [
            ("dropdown", None),
            ("dropdown-tinted", Some(theme.reset.daily)),
        ] {
            let options = ["Normal", "Daily reset", "Weekly reset"];
            let mut harness = UiHarness::new(size);
            let mut draw = |ui: &Ui| {
                in_window(ui, &theme, size, || {
                    draw_dropdown(ui, &theme, tint, "kind", "Reset type", &options, 0);
                });
            };
            let names = |state| (name, state, theme_name);
            assert_state(&mut harness, names("rest"), &mouse(away, false), &mut draw);
            assert_state(
                &mut harness,
                names("hover"),
                &mouse(field, false),
                &mut draw,
            );
            assert_state(
                &mut harness,
                names("pressed"),
                &mouse(field, true),
                &mut draw,
            );
            // Releasing on the field clicks it, which opens the list.
            harness.settle(1, &mouse(field, false), &mut draw);
            assert_state(&mut harness, names("open"), &mouse(away, false), &mut draw);
            for (state, offset) in [("hover-selected", 32.0), ("hover-option", 56.0)] {
                let position = [field[0], field[1] + offset];
                assert_state(
                    &mut harness,
                    names(state),
                    &mouse(position, false),
                    &mut draw,
                );
            }
        }
    }
}

#[test]
fn every_toast_state() {
    let size = [360.0, 120.0];
    let over_action = [247.0, 71.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut toast = Toast::new("Deleted “Shopping list”".to_owned());
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_toast(ui, &theme, &mut toast, Some("Undo"));
            });
        };
        let names = |state| ("toast", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([4.0, 4.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("hover"),
            &mouse(over_action, false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("pressed"),
            &mouse(over_action, true),
            &mut draw,
        );
    }
}

#[test]
fn every_tool_rail_state() {
    let size = [180.0, 180.0];
    let top = [
        ToolRailItem {
            icon: Icon::SEARCH,
            tooltip: "Search",
            active: true,
        },
        ToolRailItem {
            icon: Icon::TRASH_2,
            tooltip: "Trash",
            active: false,
        },
    ];
    let bottom = [ToolRailItem {
        icon: Icon::X,
        tooltip: "Bottom tool",
        active: false,
    }];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_tool_rail(ui, &theme, &top, &bottom);
            });
        };
        let names = |state| ("tool-rail", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([160.0, 170.0], false),
            &mut draw,
        );
        for (state, position) in [
            ("hover-on", [24.0, 24.0]),
            ("hover-off", [24.0, 60.0]),
            ("hover-bottom", [24.0, 154.0]),
        ] {
            assert_state(
                &mut harness,
                names(state),
                &mouse(position, false),
                &mut draw,
            );
        }
    }
}

#[test]
fn every_title_bar_state() {
    let size = [320.0, 120.0];
    let toggles = [
        TitleBarToggle {
            icon: Icon::SEARCH,
            tooltip: "Tools",
            active: false,
        },
        TitleBarToggle {
            icon: Icon::X,
            tooltip: "Note list",
            active: true,
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

#[test]
fn every_splitter_state() {
    let size = [260.0, 120.0];
    let over_splitter = [116.0, 60.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_pane(ui, &theme, "left", [100.0, 0.0], || {});
                ui.same_line_with_spacing(0.0, 0.0);
                let height = ui.content_region_avail()[1];
                draw_vertical_splitter(ui, &theme, "split", 100.0, (50.0, 150.0), height);
                ui.same_line_with_spacing(0.0, 0.0);
                draw_pane(ui, &theme, "right", [0.0, 0.0], || {});
            });
        };
        let names = |state| ("splitter", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([4.0, 4.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("hover"),
            &mouse(over_splitter, false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("pressed"),
            &mouse(over_splitter, true),
            &mut draw,
        );
    }
}
