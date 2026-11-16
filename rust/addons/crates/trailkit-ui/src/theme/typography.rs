//! Typography tokens: font sizes as factors of the Nexus body size and
//! the line height. They are the same in every built-in theme.

/// Font sizes as factors of the Nexus body size (R96), and the line height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Typography {
    /// `font.size.small`: small text.
    pub small: f32,
    /// `font.size.body`: body text.
    pub body: f32,
    /// `font.size.heading3`: level 3 heading and below.
    pub heading3: f32,
    /// `font.size.heading2`: level 2 heading.
    pub heading2: f32,
    /// `font.size.heading1`: level 1 heading.
    pub heading1: f32,
    /// `font.line_height`: line height of running text (notes, editor) as a
    /// multiple of the font size; roomy enough for chat-link chips (M77).
    pub line_height: f32,
}

impl Typography {
    /// The line height of running text in `font_size`, in whole pixels.
    #[must_use]
    pub fn line_height_for(&self, font_size: f32) -> f32 {
        (font_size * self.line_height).round()
    }

    /// The space above the text in such a line (half the extra height), so
    /// text sits in the middle of its line.
    #[must_use]
    pub fn leading_for(&self, font_size: f32) -> f32 {
        ((self.line_height_for(font_size) - font_size) / 2.0).floor()
    }
}

/// Font sizes shared by both built-in themes.
pub(super) const TYPOGRAPHY: Typography = Typography {
    small: 0.85,
    body: 1.0,
    heading3: 1.15,
    heading2: 1.3,
    heading1: 1.5,
    line_height: 1.5,
};
