//! The labelled text field: its states and the spacing rules of `tokens.md`
//! §5.1, which a snapshot alone would not protect.

use std::cell::Cell;

use nexus::imgui::Ui;
use trailkit_ui::{
    controls::text_field::{TextField, TextFieldTexts, move_below_last_item},
    theme::Theme,
};
use trailkit_ui_testing::harness::UiHarness;

use super::{THEMES, assert_state, in_window, mouse};

/// Size of the test window.
const SIZE: [f32; 2] = [280.0, 110.0];

/// Width of the field.
const WIDTH: f32 = 240.0;

/// A point on the field.
const ON_FIELD: [f32; 2] = [60.0, 50.0];

/// A point beside it.
const AWAY: [f32; 2] = [270.0, 105.0];

/// The shared texts of the tests.
const TEXTS: TextFieldTexts<'static> = TextFieldTexts {
    optional: "(optional)",
};

/// An address wider than the field.
const LONG: &str = "https://wiki.guildwars2.com/wiki/Guild_Wars_2_Wiki:Requests_for_comment";

/// What a state changes on the field.
#[derive(Debug, Clone, Copy, Default)]
struct Variant {
    /// The text in the field.
    value: &'static str,
    /// Whether the label says "(optional)".
    is_optional: bool,
    /// The error below the field.
    error: Option<&'static str>,
    /// Whether the field takes the focus.
    has_focus: bool,
}

/// Draws a field in `variant`.
fn draw_field(ui: &Ui, theme: &Theme, variant: Variant) {
    let mut value = variant.value.to_owned();
    TextField::new("field", "Address", &mut value, WIDTH)
        .placeholder("https://")
        .optional(variant.is_optional)
        .error(variant.error)
        .focus(variant.has_focus)
        .draw(ui, theme, TEXTS);
}

#[test]
fn text_field_states_match_snapshots() {
    let filled = Variant {
        value: "https://a.de",
        ..Variant::default()
    };
    let states = [
        ("rest", filled, AWAY),
        ("empty", Variant::default(), AWAY),
        ("hover", filled, ON_FIELD),
        (
            "optional",
            Variant {
                is_optional: true,
                ..filled
            },
            AWAY,
        ),
        (
            "focused",
            Variant {
                has_focus: true,
                ..filled
            },
            AWAY,
        ),
        (
            "long",
            Variant {
                value: LONG,
                ..Variant::default()
            },
            AWAY,
        ),
        (
            "error",
            Variant {
                error: Some("Enter a valid address, e.g. https://example.com."),
                ..filled
            },
            AWAY,
        ),
    ];
    for (kind, theme_name) in THEMES {
        let theme = kind.theme();
        for (state, variant, position) in states {
            let mut harness = UiHarness::new(SIZE);
            let mut draw =
                |ui: &Ui| in_window(ui, &theme, SIZE, || draw_field(ui, &theme, variant));
            assert_state(
                &mut harness,
                ("text-field", state, theme_name),
                &mouse(position, false),
                &mut draw,
            );
        }
    }
}

#[test]
fn long_text_keeps_its_distance_from_the_right_edge() {
    let theme = THEMES[0].0.theme();
    let bottom = Cell::new(0.0_f32);
    let left = Cell::new(0.0_f32);
    let height = Cell::new(0.0_f32);
    let mut harness = UiHarness::new(SIZE);
    let image = harness.settle(3, &mouse(AWAY, false), &mut |ui: &Ui| {
        in_window(ui, &theme, SIZE, || {
            left.set(ui.cursor_screen_pos()[0]);
            draw_field(
                ui,
                &theme,
                Variant {
                    value: LONG,
                    ..Variant::default()
                },
            );
            bottom.set(ui.item_rect_max()[1]);
            height.set(ui.frame_height() + theme.spacing.small);
        });
    });
    let (left, bottom, height) = (left.get(), bottom.get(), height.get());
    let right = left + WIDTH;
    let inset = theme.spacing.small;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "small positive pixels"
    )]
    let pixel = |x: f32, y: f32| image.pixel(x as u32, y as u32);
    let mut y = bottom - height + 4.0;
    while y < bottom - 4.0 {
        let fill = pixel(left + 3.0, y);
        let mut x = right - inset + 1.0;
        while x < right - 2.0 {
            assert_eq!(pixel(x, y), fill, "text in the right padding at {x}, {y}");
            x += 1.0;
        }
        y += 1.0;
    }
}

#[test]
fn form_gaps_are_measured_from_the_last_item() {
    let theme = THEMES[0].0.theme();
    let measured = Cell::new(None);
    let mut harness = UiHarness::new(SIZE);
    harness.settle(2, &mouse(AWAY, false), &mut |ui: &Ui| {
        in_window(ui, &theme, SIZE, || {
            draw_field(ui, &theme, Variant::default());
            let bottom = ui.item_rect_max()[1];
            move_below_last_item(ui, theme.spacing.medium);
            measured.set(Some(ui.cursor_screen_pos()[1] - bottom));
        });
    });
    assert_eq!(measured.get(), Some(theme.spacing.medium));
}
