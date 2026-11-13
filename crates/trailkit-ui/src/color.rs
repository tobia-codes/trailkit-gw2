//! RGBA colors and WCAG 2.x contrast calculation (trailkit-ui skill §1.5:
//! text contrast ≥ 4.5:1 in both themes).

/// An sRGB color with straight (non-premultiplied) alpha, components in `0.0..=1.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component.
    pub red: f32,
    /// Green component.
    pub green: f32,
    /// Blue component.
    pub blue: f32,
    /// Opacity (`1.0` = opaque).
    pub alpha: f32,
}

impl Color {
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self::from_rgba_hex(0x0000_0000);

    /// Creates an opaque color from a `0xRRGGBB` value.
    #[must_use]
    pub const fn from_rgb_hex(rgb: u32) -> Self {
        Self::from_rgba_hex((rgb << 8) | 0xFF)
    }

    /// Creates a color from a `0xRRGGBBAA` value.
    #[must_use]
    pub const fn from_rgba_hex(rgba: u32) -> Self {
        Self {
            red: channel_to_unit((rgba >> 24) & 0xFF),
            green: channel_to_unit((rgba >> 16) & 0xFF),
            blue: channel_to_unit((rgba >> 8) & 0xFF),
            alpha: channel_to_unit(rgba & 0xFF),
        }
    }

    /// Creates a color from `[red, green, blue, alpha]`, the layout ImGui
    /// uses (e.g. a style color read back from ImGui).
    #[must_use]
    pub const fn from_array([red, green, blue, alpha]: [f32; 4]) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// Returns the same color with a different opacity.
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }

    /// This color painted over the opaque `background` (alpha blending in
    /// sRGB, as ImGui does); the result is opaque.
    #[must_use]
    pub fn over(self, background: Self) -> Self {
        let blend = |top: f32, bottom: f32| top * self.alpha + bottom * (1.0 - self.alpha);
        Self {
            red: blend(self.red, background.red),
            green: blend(self.green, background.green),
            blue: blend(self.blue, background.blue),
            alpha: 1.0,
        }
    }

    /// Returns the components as `[red, green, blue, alpha]`, the layout ImGui expects.
    #[must_use]
    pub const fn to_array(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }

    /// Relative luminance per WCAG 2.x (alpha is ignored).
    #[must_use]
    pub fn relative_luminance(self) -> f32 {
        0.2126 * linearize(self.red)
            + 0.7152 * linearize(self.green)
            + 0.0722 * linearize(self.blue)
    }

    /// WCAG contrast ratio between two opaque colors, from `1.0` to `21.0`.
    #[must_use]
    pub fn contrast_ratio(self, other: Self) -> f32 {
        let (lighter, darker) = {
            let first = self.relative_luminance();
            let second = other.relative_luminance();
            if first >= second {
                (first, second)
            } else {
                (second, first)
            }
        };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// This color with its OKLab lightness lowered by `step` (`0.08` is 8 %),
    /// hue and chroma kept; channels outside sRGB are clamped. The rule
    /// behind the hover and pressed fills of colored buttons (R107): the
    /// theme stores the results as hex values, and a test checks them
    /// against this function. Alpha is kept.
    #[must_use]
    pub fn darkened(self, step: f32) -> Self {
        let [lightness, green_red, blue_yellow] = to_oklab([
            linearize(self.red),
            linearize(self.green),
            linearize(self.blue),
        ]);
        let [red, green, blue] = from_oklab([lightness - step, green_red, blue_yellow]);
        Self {
            red: delinearize(red),
            green: delinearize(green),
            blue: delinearize(blue),
            alpha: self.alpha,
        }
    }
}

/// Converts an 8-bit channel value (`0..=255`, exactly representable as `f32`)
/// to `0.0..=1.0`.
const fn channel_to_unit(channel: u32) -> f32 {
    channel as f32 / 255.0
}

/// Converts a gamma-encoded sRGB component to linear light (WCAG formula).
fn linearize(component: f32) -> f32 {
    if component <= 0.040_45 {
        component / 12.92
    } else {
        ((component + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts linear light back to a gamma-encoded sRGB component, clamped
/// to `0.0..=1.0`.
fn delinearize(component: f32) -> f32 {
    let encoded = if component <= 0.003_130_8 {
        12.92 * component
    } else {
        1.055 * component.powf(1.0 / 2.4) - 0.055
    };
    encoded.clamp(0.0, 1.0)
}

/// Converts linear sRGB to OKLab `[L, a, b]` (Björn Ottosson's matrices).
fn to_oklab([red, green, blue]: [f32; 3]) -> [f32; 3] {
    let long = (0.412_221_46 * red + 0.536_332_55 * green + 0.051_445_995 * blue).cbrt();
    let medium = (0.211_903_5 * red + 0.680_699_5 * green + 0.107_396_96 * blue).cbrt();
    let short = (0.088_302_46 * red + 0.281_718_85 * green + 0.629_978_7 * blue).cbrt();
    [
        0.210_454_26 * long + 0.793_617_8 * medium - 0.004_072_047 * short,
        1.977_998_5 * long - 2.428_592_2 * medium + 0.450_593_7 * short,
        0.025_904_037 * long + 0.782_771_77 * medium - 0.808_675_77 * short,
    ]
}

/// Converts OKLab `[L, a, b]` back to linear sRGB (may leave `0.0..=1.0`).
fn from_oklab([lightness, green_red, blue_yellow]: [f32; 3]) -> [f32; 3] {
    let long = (lightness + 0.396_337_78 * green_red + 0.215_803_76 * blue_yellow).powi(3);
    let medium = (lightness - 0.105_561_346 * green_red - 0.063_854_17 * blue_yellow).powi(3);
    let short = (lightness - 0.089_484_18 * green_red - 1.291_485_5 * blue_yellow).powi(3);
    [
        4.076_741_7 * long - 3.307_711_6 * medium + 0.230_969_94 * short,
        -1.268_438 * long + 2.609_757_4 * medium - 0.341_319_4 * short,
        -0.004_196_086_3 * long - 0.703_418_6 * medium + 1.707_614_7 * short,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn darkening_by_nothing_keeps_the_color() {
        let brass = Color::from_rgb_hex(0xE2A740);
        let same = brass.darkened(0.0);
        for (before, after) in brass.to_array().iter().zip(same.to_array()) {
            assert!((before - after).abs() < 0.5 / 255.0);
        }
    }

    #[test]
    fn parses_hex() {
        let color = Color::from_rgba_hex(0xFF00_8000);
        assert_eq!(color.to_array(), [1.0, 0.0, 128.0 / 255.0, 0.0]);
        assert_eq!(Color::from_rgb_hex(0x00FF00).alpha, 1.0);
    }

    #[test]
    fn black_on_white_has_maximum_contrast() {
        let black = Color::from_rgb_hex(0x000000);
        let white = Color::from_rgb_hex(0xFFFFFF);
        assert!((black.contrast_ratio(white) - 21.0).abs() < 0.01);
        assert!((white.contrast_ratio(black) - 21.0).abs() < 0.01);
    }
}
