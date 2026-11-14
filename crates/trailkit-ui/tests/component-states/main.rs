//! Snapshots of every shared component in every interaction state and both
//! themes (ROADMAP R82): the reference images are reviewed against the design
//! system (`tokens.md` §10).
//!
//! Run from WSL with `cargo xtask ui-test` (Windows only; the ImGui harness
//! needs nexus-rs' ImGui). `cargo xtask ui-test --update` rewrites the
//! reference images in `tests/snapshots/`.
#![cfg(windows)]

use std::cell::Cell;
use trailkit_ui::draw_list::window_draw_list;

use nexus::imgui::{Condition, Ui, Window, WindowFlags};
use trailkit_ui::{
    button::{
        Button, ButtonKind, ButtonState, draw_button, draw_danger_button, draw_danger_icon_button,
        draw_icon_button, draw_icon_button_with_state, draw_icon_toggle, draw_primary_button,
        draw_primary_button_with_icon,
    },
    dropdown::draw_dropdown,
    empty_state::{EmptyState, EmptyStateButtons, draw_empty_state},
    icons::Icon,
    list_row::{ListRow, ListRowDetail, draw_list_row},
    menu::{MenuItemTone, begin_submenu, draw_menu_item, draw_menu_item_with_icon, draw_popup},
    pane::draw_pane,
    separator::draw_compact_separator,
    splitter::draw_vertical_splitter,
    status_line::{Status, draw_status_line},
    style_scope::StyleScope,
    text::{TextStyle, draw_icon, draw_text, draw_text_wrapped, draw_truncated_text},
    theme::{Theme, ThemeKind},
    title_bar::{
        TitleBarState, TitleBarTexts, TitleBarToggle, TogglePlacement,
        draw_window as draw_title_bar,
    },
    tooltip::draw_tooltip,
};
use trailkit_ui_testing::{
    harness::{FrameInput, UiHarness},
    snapshot::assert_snapshot,
};

mod notifications;
mod title_bar;

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
fn button_kinds_match_snapshots() {
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

/// The states of the one button with a kind and a state (R93).
const BUTTON_STATES: [(&str, DrawButton); 4] = [
    ("button-disabled", |ui, theme| {
        Button::new("Discard")
            .state(ButtonState::Disabled {
                reason: "Available once saving has finished",
            })
            .draw(ui, theme);
    }),
    ("button-primary-busy", |ui, theme| {
        Button::new("Saving…")
            .kind(ButtonKind::Primary)
            .state(ButtonState::Busy)
            .draw(ui, theme);
    }),
    ("button-regular-busy", |ui, theme| {
        Button::new("Saving…")
            .state(ButtonState::Busy)
            .draw(ui, theme);
    }),
    ("button-icon-disabled", |ui, theme| {
        draw_icon_button_with_state(
            ui,
            theme,
            Icon::CALENDAR_CLOCK,
            "",
            ButtonState::Disabled {
                reason: "Insert event — requires trailkit-core",
            },
        );
    }),
];

#[test]
fn button_states_match_snapshots() {
    // Wide enough for the reason tooltip of a disabled button.
    let size = [360.0, 90.0];
    let over_button = [20.0, 22.0];
    let mut harness = UiHarness::new(size);
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (name, draw_one) in BUTTON_STATES {
            let mut draw = |ui: &Ui| in_window(ui, &theme, size, || draw_one(ui, &theme));
            let names = |state| (name, state, theme_name);
            assert_state(
                &mut harness,
                names("rest"),
                &mouse([340.0, 80.0], false),
                &mut draw,
            );
            assert_state(
                &mut harness,
                names("hover"),
                &mouse(over_button, false),
                &mut draw,
            );
            // Busy and disabled buttons are no ImGui buttons: pressing them
            // changes nothing (R93).
            assert_state(
                &mut harness,
                names("pressed"),
                &mouse(over_button, true),
                &mut draw,
            );
            harness.settle(2, &mouse([340.0, 80.0], false), &mut draw);
        }
    }
}

/// Draws a menu with every kind of entry; the popup opens in the first frame.
fn draw_sample_menu(ui: &Ui, theme: &Theme, opened: &Cell<bool>) {
    if !opened.replace(true) {
        ui.open_popup("menu");
    }
    draw_popup(ui, theme, "menu", || {
        draw_menu_item(ui, theme, "Rename", false);
        draw_menu_item(ui, theme, "Checked", true);
        if let Some(_submenu) = begin_submenu(ui, theme, Icon::FOLDER_INPUT, "Move to") {
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
fn menu_entry_states_match_snapshots() {
    let size = [340.0, 240.0];
    let opening = [12.0, 12.0];
    let row = |offset: f32| [opening[0] + 40.0, opening[1] + offset];
    let states = [
        ("hover-plain", row(20.0)),
        ("hover-checked", row(44.0)),
        ("hover-icon", row(100.0)),
        ("hover-danger", row(124.0)),
        ("hover-disabled", row(182.0)),
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
fn dropdown_states_match_snapshots() {
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
fn splitter_states_match_snapshots() {
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

/// Every status in the Body style, and a long text wrapped in the Small
/// style with its lines hanging under the text (R96).
#[test]
fn status_line_states_match_snapshots() {
    let long = "The log could not be saved because the Downloads folder is not writable.";
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let size = [300.0, 130.0];
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_status_line(ui, &theme, Status::Success, "Saved", TextStyle::Body);
                draw_status_line(ui, &theme, Status::Warning, "Check it", TextStyle::Body);
                draw_status_line(ui, &theme, Status::Danger, "Failed", TextStyle::Body);
            });
        };
        assert_state(
            &mut harness,
            ("status-line", "body", theme_name),
            &mouse([290.0, 120.0], false),
            &mut draw,
        );
        // Only one harness may exist at a time.
        drop(harness);
        let size = [220.0, 110.0];
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_status_line(ui, &theme, Status::Danger, long, TextStyle::Small);
            });
        };
        assert_state(
            &mut harness,
            ("status-line", "small-wrapped", theme_name),
            &mouse([210.0, 100.0], false),
            &mut draw,
        );
    }
}

/// The list row: a plain row, a selected one, a group with count and detail
/// line, a shortened title and an urgent detail, at rest and hovered.
#[test]
fn list_row_states_match_snapshots() {
    let size = [240.0, 190.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                let row = |id, selected, title, style, trailing, detail| ListRow {
                    id,
                    selected,
                    title,
                    title_style: style,
                    trailing,
                    detail,
                };
                let days = |text, color| Some(ListRowDetail { text, color });
                for list_row in [
                    row("##a", false, "Daily chores", TextStyle::Body, None, None),
                    row("##b", true, "Farming route", TextStyle::Body, None, None),
                    row(
                        "##c",
                        false,
                        "Guild notes",
                        TextStyle::BodyStrong,
                        Some("12"),
                        days("29 days left", theme.text.secondary),
                    ),
                    row(
                        "##d",
                        false,
                        "A very long title that cannot fit the row",
                        TextStyle::Body,
                        None,
                        days("3 days left", theme.status.danger),
                    ),
                ] {
                    draw_list_row(ui, &theme, &list_row);
                }
            });
        };
        assert_state(
            &mut harness,
            ("list-row", "rest", theme_name),
            &mouse([230.0, 180.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            ("list-row", "hover", theme_name),
            &mouse([60.0, 14.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            ("list-row", "hover-truncated", theme_name),
            &mouse([60.0, 150.0], false),
            &mut draw,
        );
    }
}

/// A sentence in every text style: heading sizes, body, emphasis, secondary
/// and small text (R96).
#[test]
fn text_styles_match_snapshots() {
    let size = [280.0, 250.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                for style in TextStyle::ALL {
                    draw_text(ui, &theme, style, &format!("{style:?} text"));
                }
                draw_text_wrapped(
                    ui,
                    &theme,
                    TextStyle::Small,
                    "Small text wraps at the window edge when a sentence is too long.",
                );
            });
        };
        assert_state(
            &mut harness,
            ("text", "styles", theme_name),
            &mouse([270.0, 240.0], false),
            &mut draw,
        );
    }
}

/// Text shortened with "…": the tooltip with the full text only while a
/// shortened text is hovered, none for text that fits (R96, Q282).
#[test]
fn truncated_text_with_and_without_tooltip() {
    let size = [260.0, 130.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_truncated_text(
                    ui,
                    &theme,
                    TextStyle::BodyStrong,
                    "Papierkorb mit langem Namen",
                    90.0,
                );
                ui.dummy([0.0, 30.0]);
                draw_truncated_text(ui, &theme, TextStyle::BodyStrong, "Kurz", 90.0);
            });
        };
        let names = |state| ("truncated-text", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([250.0, 120.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("hover-truncated"),
            &mouse([30.0, 20.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("hover-fits"),
            &mouse([20.0, 70.0], false),
            &mut draw,
        );
    }
}

/// An icon in every color a view draws it in.
#[test]
fn icons_in_text_colors() {
    let size = [200.0, 60.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                for color in [
                    theme.text.primary,
                    theme.text.secondary,
                    theme.text.disabled,
                    theme.status.warning,
                    theme.status.danger,
                ] {
                    draw_icon(ui, Icon::TRIANGLE_ALERT, color);
                    ui.same_line();
                }
            });
        };
        assert_state(
            &mut harness,
            ("icon", "colors", theme_name),
            &mouse([190.0, 50.0], false),
            &mut draw,
        );
    }
}

/// A tooltip shown while the caller holds the window's draw list, as the
/// editor does over a chat-link chip: imgui-rs allows one `DrawListMut` at a
/// time, so a tooltip must not take a second one (it crashed the game).
#[test]
fn tooltip_while_the_window_draw_list_is_held() {
    let theme = ThemeKind::Dark.theme();
    let size = [200.0, 80.0];
    let mut harness = UiHarness::new(size);
    let _image = harness.frame(&mouse([40.0, 30.0], false), |ui: &Ui| {
        in_window(ui, &theme, size, || {
            let draw_list = window_draw_list(ui);
            draw_tooltip(ui, &theme, "Tooltip");
            draw_list.add_line([0.0, 0.0], [1.0, 1.0], [1.0; 4]).build();
        });
    });
}

/// A right click outside every window takes the focus from a trailkit
/// window, as a left click does in ImGui (memo M186); a right click inside
/// keeps it.
#[test]
fn a_right_click_outside_takes_the_focus() {
    let theme = ThemeKind::Dark.theme();
    let size = [320.0, 240.0];
    let texts = TitleBarTexts {
        title: "Notes",
        collapse: "Collapse",
        expand: "Expand",
        close: "Close",
    };
    let mut state = TitleBarState::default();
    let mut open = true;
    let focused = Cell::new(false);
    let mut harness = UiHarness::new(size);
    let mut frame = |harness: &mut UiHarness, input: FrameInput| {
        let _image = harness.frame(&input, |ui: &Ui| {
            let _style = StyleScope::push_with_cursor_extent(ui, &theme, None);
            let _clicked = draw_title_bar(
                ui,
                &theme,
                Window::new("Notes###focus")
                    .position([0.0, 0.0], Condition::Always)
                    .size([200.0, 140.0], Condition::Always),
                texts,
                &[],
                &mut state,
                &mut open,
                || focused.set(ui.is_window_focused()),
            );
        });
    };
    let right = |position: [f32; 2], down: bool| FrameInput {
        mouse_position: position,
        right_button_down: down,
        ..FrameInput::default()
    };
    let inside = [100.0, 100.0];
    let outside = [280.0, 200.0];
    for input in [
        mouse(inside, true),
        mouse(inside, false),
        mouse(inside, false),
    ] {
        frame(&mut harness, input);
    }
    assert!(focused.get(), "a left click inside focuses the window");
    for input in [
        right(inside, true),
        right(inside, false),
        right(inside, false),
    ] {
        frame(&mut harness, input);
    }
    assert!(focused.get(), "a right click inside keeps the focus");
    for input in [
        right(outside, true),
        right(outside, false),
        right(outside, false),
    ] {
        frame(&mut harness, input);
    }
    assert!(!focused.get(), "a right click outside takes the focus");
}

/// The empty state in every variant: title only with two buttons, icon with
/// title and hint, one button, and the buttons stacked in a narrow pane (Q283: one layout).
#[test]
fn empty_states_match_snapshots() {
    type Variant = (&'static str, [f32; 2], fn() -> EmptyState<'static>);
    let variants: [Variant; 4] = [
        ("buttons", [320.0, 200.0], || EmptyState {
            icon: None,
            title: "No notes yet",
            hint: None,
            buttons: Some(EmptyStateButtons {
                primary: "Create first note",
                secondary: Some("New group"),
            }),
        }),
        ("buttons-stacked", [200.0, 200.0], || EmptyState {
            icon: None,
            title: "No notes yet",
            hint: None,
            buttons: Some(EmptyStateButtons {
                primary: "Create first note",
                secondary: Some("New group"),
            }),
        }),
        ("icon-and-hint", [260.0, 200.0], || EmptyState {
            icon: Some(Icon::TRASH_2),
            title: "The trash is empty.",
            hint: Some("Entries stay for 30 days."),
            buttons: None,
        }),
        ("one-button", [260.0, 200.0], || EmptyState {
            icon: Some(Icon::FILE_PLUS),
            title: "Nothing here yet",
            hint: Some("Notes you create show up here."),
            buttons: Some(EmptyStateButtons {
                primary: "New note",
                secondary: None,
            }),
        }),
    ];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (name, size, state) in variants {
            let mut harness = UiHarness::new(size);
            let mut draw = |ui: &Ui| {
                in_window(ui, &theme, size, || {
                    draw_empty_state(ui, &theme, &state());
                });
            };
            assert_state(
                &mut harness,
                ("empty-state", name, theme_name),
                &mouse([size[0] - 4.0, size[1] - 4.0], false),
                &mut draw,
            );
        }
    }
}

/// A borderless input: its frame stays invisible at rest and hovered.
#[test]
fn transparent_frame_at_rest_and_hovered() {
    let size = [220.0, 70.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let mut text = String::from("Typed text");
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                let _frame = StyleScope::transparent_frame(ui);
                ui.input_text("##input", &mut text).build();
            });
        };
        let names = |state| ("transparent-frame", state, theme_name);
        assert_state(
            &mut harness,
            names("rest"),
            &mouse([210.0, 60.0], false),
            &mut draw,
        );
        assert_state(
            &mut harness,
            names("hover"),
            &mouse([40.0, 22.0], false),
            &mut draw,
        );
    }
}
