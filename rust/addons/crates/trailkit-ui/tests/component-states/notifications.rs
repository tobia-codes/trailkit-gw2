//! Snapshots of the notifications: toasts in every tone, width and action
//! layout, and status cards (R82).

use trailkit_ui::{
    status_card::{CardDetail, CardStatus, StatusCard, draw_status_card},
    toast::{Toast, ToastAction, ToastExtras, ToastTone, draw_toast, draw_toast_with_extras},
};

use super::*;

/// One state of an error or success toast (memo M180).
struct ToneState {
    /// The snapshot state name.
    name: &'static str,
    /// The image size; narrow sizes move the action below the message.
    size: [f32; 2],
    /// The toast's tone and message.
    tone: ToastTone,
    /// The message.
    message: &'static str,
    /// The action, if any, and whether it runs.
    action: Option<(&'static str, bool)>,
    /// Counter, spinner and close button.
    extras: ToastExtras<'static>,
    /// Where the mouse is.
    mouse: [f32; 2],
}

#[test]
fn toast_states_match_snapshots() {
    let size = [360.0, 120.0];
    let over_action = [247.0, 71.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut toast = Toast::new("Deleted “Shopping list”".to_owned());
        let mut harness = UiHarness::new(size);
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_toast(
                    ui,
                    &theme,
                    &mut toast,
                    &[ToastAction {
                        label: "Undo",
                        tooltip: None,
                        state: ButtonState::Ready,
                    }],
                );
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
fn toast_with_two_actions_in_every_width() {
    let actions = [
        ToastAction {
            label: "Undo",
            tooltip: None,
            state: ButtonState::Ready,
        },
        ToastAction {
            label: "Open trash",
            tooltip: None,
            state: ButtonState::Ready,
        },
    ];
    // Wide enough for one row, for the buttons in a row below the message
    // and only for one button per row (M159).
    for (state, size) in [
        ("two-actions", [420.0, 140.0]),
        ("two-actions-below", [250.0, 140.0]),
        ("two-actions-stacked", [150.0, 180.0]),
    ] {
        for (kind, theme_name) in THEMES {
            let theme = kind.theme();
            let mut toast = Toast::new("Moved to the trash".to_owned());
            let mut harness = UiHarness::new(size);
            let mut draw = |ui: &Ui| {
                in_window(ui, &theme, size, || {
                    draw_toast(ui, &theme, &mut toast, &actions);
                });
            };
            assert_state(
                &mut harness,
                ("toast", state, theme_name),
                &mouse([4.0, 4.0], false),
                &mut draw,
            );
        }
    }
}

#[test]
fn error_and_success_toast_states_match_snapshots() {
    let close = Some("Close");
    let wide = [420.0, 120.0];
    let states = [
        ToneState {
            name: "error-retrying",
            size: wide,
            tone: ToastTone::Error,
            message: "Saving failed",
            action: None,
            extras: ToastExtras {
                counter: Some("2/3"),
                spinner: true,
                close_tooltip: close,
            },
            mouse: [4.0, 4.0],
        },
        ToneState {
            name: "error-gave-up",
            size: wide,
            tone: ToastTone::Error,
            message: "Saving failed",
            action: Some(("Try again", false)),
            extras: ToastExtras {
                close_tooltip: close,
                ..ToastExtras::default()
            },
            mouse: [4.0, 4.0],
        },
        ToneState {
            name: "error-close-hover",
            size: wide,
            tone: ToastTone::Error,
            message: "Saving failed",
            action: Some(("Try again", false)),
            extras: ToastExtras {
                close_tooltip: close,
                ..ToastExtras::default()
            },
            mouse: [301.0, 71.0],
        },
        ToneState {
            name: "error-saving",
            size: wide,
            tone: ToastTone::Error,
            message: "Saving failed",
            action: Some(("Saving…", true)),
            extras: ToastExtras {
                close_tooltip: close,
                ..ToastExtras::default()
            },
            mouse: [4.0, 4.0],
        },
        ToneState {
            name: "error-narrow",
            size: [320.0, 140.0],
            tone: ToastTone::Error,
            message: "Speichern fehlgeschlagen",
            action: Some(("Erneut versuchen", false)),
            extras: ToastExtras {
                close_tooltip: Some("Schließen"),
                ..ToastExtras::default()
            },
            mouse: [4.0, 4.0],
        },
        ToneState {
            name: "success",
            size: wide,
            tone: ToastTone::Success,
            message: "Changes saved",
            action: None,
            extras: ToastExtras::default(),
            mouse: [4.0, 4.0],
        },
    ];
    for state in states {
        for (kind, theme_name) in THEMES {
            let theme = kind.theme();
            let mut toast = Toast::with_tone(state.message.to_owned(), state.tone);
            let actions: Vec<ToastAction<'_>> = state
                .action
                .map(|(label, busy)| ToastAction {
                    label,
                    tooltip: None,
                    state: if busy {
                        ButtonState::Busy
                    } else {
                        ButtonState::Ready
                    },
                })
                .into_iter()
                .collect();
            let mut harness = UiHarness::new(state.size);
            let mut draw = |ui: &Ui| {
                in_window(ui, &theme, state.size, || {
                    draw_toast_with_extras(ui, &theme, &mut toast, &actions, state.extras);
                });
            };
            assert_state(
                &mut harness,
                ("toast", state.name, theme_name),
                &mouse(state.mouse, false),
                &mut draw,
            );
        }
    }
}

/// The status card in both statuses (M184): `danger` with a detail line,
/// a button and the close button, at rest and with the close button
/// hovered; `warning` with a secondary detail and two buttons that wrap into
/// two rows in a narrow window.
#[test]
fn status_card_states_match_snapshots() {
    // Tall enough for no scrollbar; the hover sits on the close button.
    let size = [260.0, 300.0];
    let close_hover = [232.0, 28.0];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        let mut harness = UiHarness::new(size);
        let buttons = [Button::new("Open folder")];
        let wide_buttons = [
            Button::new("Restore unsaved changes").kind(ButtonKind::Primary),
            Button::new("Discard"),
        ];
        let mut draw = |ui: &Ui| {
            in_window(ui, &theme, size, || {
                draw_status_card(
                    ui,
                    &theme,
                    &StatusCard {
                        status: CardStatus::Danger,
                        text: "Your notes could not be loaded. The damaged file was kept as:",
                        detail: Some(CardDetail {
                            text: "backups/notes.corrupt.json",
                            style: TextStyle::Body,
                        }),
                        buttons: &buttons,
                        close_tooltip: Some("Close"),
                    },
                );
                draw_status_card(
                    ui,
                    &theme,
                    &StatusCard {
                        status: CardStatus::Warning,
                        text: "Unsaved changes from 11/01/2026, 08:00",
                        detail: Some(CardDetail {
                            text: "The note was changed since.",
                            style: TextStyle::Secondary,
                        }),
                        buttons: &wide_buttons,
                        close_tooltip: None,
                    },
                );
            });
        };
        for (state, input) in [
            ("rest", mouse([250.0, 295.0], false)),
            ("close-hovered", mouse(close_hover, false)),
        ] {
            assert_state(
                &mut harness,
                ("status-card", state, theme_name),
                &input,
                &mut draw,
            );
        }
    }
}

/// The card's close button works while another `x` icon button of the same
/// window was drawn first, as the title bar's is: both take the icon as
/// ImGui id, so the card needs its own id scope (M185, the `x` did nothing in
/// game).
#[test]
fn the_status_card_closes_next_to_another_close_button() {
    let theme = ThemeKind::Dark.theme();
    let size = [260.0, 300.0];
    let close_center = [232.0, 28.0];
    let closed = Cell::new(false);
    let buttons = [Button::new("Open folder")];
    let mut harness = UiHarness::new(size);
    for down in [false, true, false, false] {
        let _image = harness.frame(&mouse(close_center, down), |ui: &Ui| {
            in_window(ui, &theme, size, || {
                let start = ui.cursor_screen_pos();
                // Another `x` first, out of the way at the bottom.
                ui.set_cursor_screen_pos([start[0], size[1] - 40.0]);
                let _title_close = draw_icon_button(ui, &theme, Icon::X, "Close");
                ui.set_cursor_screen_pos(start);
                let outcome = draw_status_card(
                    ui,
                    &theme,
                    &StatusCard {
                        status: CardStatus::Danger,
                        text: "Your notes could not be loaded. The damaged file was kept as:",
                        detail: Some(CardDetail {
                            text: "backups/notes.corrupt.json",
                            style: TextStyle::Body,
                        }),
                        buttons: &buttons,
                        close_tooltip: Some("Close"),
                    },
                );
                if outcome.closed {
                    closed.set(true);
                }
            });
        });
    }
    assert!(closed.get(), "a click on the card's x closes it");
}
