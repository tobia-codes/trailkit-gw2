//! Tests of the built-in themes: contrast minimums, the darkening rule and
//! the radii (trailkit-ui skill §1.5, R107).

use super::*;
use crate::color::Color;

/// Minimum WCAG contrast for text (trailkit-ui skill §1.5).
const MINIMUM_TEXT_CONTRAST: f32 = 4.5;

/// Asserts `foreground` on `background` meets the text contrast minimum.
fn assert_readable(kind: ThemeKind, name: &str, foreground: Color, background: Color) {
    let ratio = foreground.contrast_ratio(background);
    assert!(
        ratio >= MINIMUM_TEXT_CONTRAST,
        "{kind:?}: {name} has contrast {ratio:.2}, needs {MINIMUM_TEXT_CONTRAST}"
    );
}

#[test]
fn text_meets_contrast_minimum_in_every_theme() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        let backgrounds = [
            ("bg.window", theme.surface.window),
            ("bg.panel", theme.surface.panel),
            ("bg.input", theme.surface.input),
            ("bg.popup", theme.surface.popup),
            ("bg.popup_hover", theme.surface.popup_hover),
            ("bg.hover", theme.surface.hover),
            ("bg.active", theme.surface.active),
        ];
        let texts = [
            ("text.primary", theme.text.primary),
            ("text.secondary", theme.text.secondary),
            ("text.link", theme.text.link),
            ("accent.text", theme.accent.text),
            ("success", theme.status.success),
            ("warning", theme.status.warning),
            ("danger", theme.status.danger),
            ("info", theme.status.info),
        ];
        for (background_name, background) in backgrounds {
            for (text_name, text) in texts {
                assert_readable(
                    kind,
                    &format!("{text_name} on {background_name}"),
                    text,
                    background,
                );
            }
        }
        for (accent_name, accent) in [
            ("accent", theme.accent.base),
            ("accent.hover", theme.accent.hover),
            ("accent.active", theme.accent.active),
        ] {
            assert_readable(
                kind,
                &format!("text.on_accent on {accent_name}"),
                theme.text.on_accent,
                accent,
            );
        }
        // A delete button's label is `text.on_danger` (R125).
        for (danger_name, danger) in [
            ("danger.fill", theme.status.danger_fill),
            ("danger.fill_hover", theme.status.danger_fill_hover),
            ("danger.fill_active", theme.status.danger_fill_active),
        ] {
            assert_readable(
                kind,
                &format!("text.on_danger on {danger_name}"),
                theme.text.on_danger,
                danger,
            );
        }
    }
}

/// Hover and pressed fills of colored buttons follow one rule (R107):
/// the rest fill darkened by a fixed OKLab lightness step. The theme
/// stores hex values; they may differ from the formula by rounding only.
#[test]
fn button_state_fills_follow_the_darkening_rule() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        for (name, base, stored, step) in [
            (
                "accent.hover",
                theme.accent.base,
                theme.accent.hover,
                HOVER_DARKENING,
            ),
            (
                "accent.active",
                theme.accent.base,
                theme.accent.active,
                PRESSED_DARKENING,
            ),
            (
                "danger.fill_hover",
                theme.status.danger_fill,
                theme.status.danger_fill_hover,
                HOVER_DARKENING,
            ),
            (
                "danger.fill_active",
                theme.status.danger_fill,
                theme.status.danger_fill_active,
                PRESSED_DARKENING,
            ),
        ] {
            let expected = base.darkened(step).to_array();
            for (channel, (stored, expected)) in
                stored.to_array().iter().zip(expected).take(3).enumerate()
            {
                assert!(
                    (stored - expected).abs() <= 1.0 / 255.0,
                    "{kind:?}: {name} channel {channel} is {stored:.4}, rule gives {expected:.4}"
                );
            }
        }
    }
}

/// The error toast's message, counter and icon sit on `danger.subtle`;
/// its buttons use `bg.active` (memo M180). A status card puts the
/// status color, primary and secondary text on its tinted fill (M184).
#[test]
fn tinted_status_fills_meet_contrast_minimum() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        for (fill_name, fill, status) in [
            (
                "danger.subtle",
                theme.status.danger_subtle,
                theme.status.danger,
            ),
            (
                "warning.subtle",
                theme.status.warning_subtle,
                theme.status.warning,
            ),
        ] {
            for (name, text) in [
                ("text.primary", theme.text.primary),
                ("text.secondary", theme.text.secondary),
                ("status", status),
            ] {
                assert_readable(kind, &format!("{name} on {fill_name}"), text, fill);
            }
        }
    }
}

#[test]
fn text_stays_readable_on_search_highlights() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        for (surface_name, surface) in [
            ("bg.panel", theme.surface.panel),
            ("bg.hover", theme.surface.hover),
            ("bg.active", theme.surface.active),
        ] {
            let highlight = theme.accent.highlight.over(surface);
            // Titles (text.primary) sit on list rows; links only occur
            // in the note view on bg.panel.
            let link = (surface_name == "bg.panel").then_some(("text.link", theme.text.link));
            for (text_name, text) in [("text.primary", theme.text.primary)]
                .into_iter()
                .chain(link)
            {
                assert_readable(
                    kind,
                    &format!("{text_name} on accent.highlight over {surface_name}"),
                    text,
                    highlight,
                );
            }
        }
    }
}

/// `bg.pressed` is shown only while a control is held and carries only
/// button and menu labels, so only those need the minimum on it (R85).
#[test]
fn labels_meet_contrast_minimum_on_the_pressed_fill() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        for (name, label) in [
            ("text.primary", theme.text.primary),
            ("danger", theme.status.danger),
        ] {
            let contrast = label.contrast_ratio(theme.surface.pressed);
            assert!(
                contrast >= 4.5,
                "{kind:?}: {name} has contrast {contrast:.2}"
            );
        }
    }
}

#[test]
fn chip_text_meets_contrast_minimum_on_its_fill() {
    for kind in ThemeKind::ALL {
        let theme = kind.theme();
        let chip = theme.chip;
        for (name, colors) in [
            ("chip.map_point", chip.map_point),
            ("chip.wvw_objective", chip.wvw_objective),
            ("chip.character", chip.character),
            ("chip.item", chip.item),
            ("chip.progress", chip.progress),
        ] {
            assert_readable(kind, name, colors.text, colors.background);
            assert_readable(
                kind,
                &format!("text.secondary on {name}"),
                theme.text.secondary,
                colors.background,
            );
            assert_readable(
                kind,
                &format!("text.primary on {name}"),
                theme.text.primary,
                colors.background,
            );
        }
    }
}

#[test]
fn radii_grow_from_chips_to_windows() {
    for kind in ThemeKind::ALL {
        let radius = kind.theme().radius;
        assert_eq!(
            (radius.small, radius.medium, radius.large),
            (4.0, 6.0, 10.0)
        );
    }
}

#[test]
fn dark_is_default() {
    assert_eq!(ThemeKind::default(), ThemeKind::Dark);
    assert_eq!(Theme::default(), Theme::DARK);
}
