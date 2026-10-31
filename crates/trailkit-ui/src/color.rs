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

    /// Returns the same color with a different opacity.
    #[must_use]
    pub const fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
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

#[cfg(test)]
mod tests {
    use super::*;

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
