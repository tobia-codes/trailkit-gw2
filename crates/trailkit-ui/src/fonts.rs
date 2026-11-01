//! The shared trailkit fonts: Open Sans text faces and the Lucide icon font
//! (memo plan M35, M36, M62).
//!
//! Every addon registers the same font identifiers with Nexus; Nexus loads a
//! font once and shares it between all addons that ask for the same id. The
//! body size follows Nexus' `FontUI` (which follows the GW2 interface size);
//! headings are derived from it. This module only describes the fonts and is
//! platform-independent; registration lives in `font_registry` (Windows).

/// Open Sans Regular (OFL, see `assets/fonts/open-sans/LICENSE`).
pub static OPEN_SANS_REGULAR: &[u8] =
    include_bytes!("../assets/fonts/open-sans/open-sans-regular.ttf");

/// Open Sans Bold.
pub static OPEN_SANS_BOLD: &[u8] = include_bytes!("../assets/fonts/open-sans/open-sans-bold.ttf");

/// Open Sans Italic.
pub static OPEN_SANS_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/open-sans/open-sans-italic.ttf");

/// Open Sans Bold Italic.
pub static OPEN_SANS_BOLD_ITALIC: &[u8] =
    include_bytes!("../assets/fonts/open-sans/open-sans-bold-italic.ttf");

/// Lucide icon font (ISC, see `assets/fonts/lucide/LICENSE`).
pub static LUCIDE: &[u8] = include_bytes!("../assets/fonts/lucide/lucide.ttf");

/// Body size used until Nexus reports its `FontUI` size (ImGui default).
pub const FALLBACK_BODY_SIZE: f32 = 16.0;

/// Every font trailkit draws with: a face at a size relative to the body text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontRole {
    /// Body text.
    Body,
    /// Bold body text.
    BodyBold,
    /// Italic body text.
    BodyItalic,
    /// Bold italic body text.
    BodyBoldItalic,
    /// Level 1 heading (bold, 1.5×).
    Heading1,
    /// Level 2 heading (bold, 1.3×).
    Heading2,
    /// Level 3 heading and below (bold, 1.15×).
    Heading3,
    /// Lucide icons at body size.
    Icons,
}

impl FontRole {
    /// Every role, in registration order.
    pub const ALL: [Self; 8] = [
        Self::Body,
        Self::BodyBold,
        Self::BodyItalic,
        Self::BodyBoldItalic,
        Self::Heading1,
        Self::Heading2,
        Self::Heading3,
        Self::Icons,
    ];

    /// The Nexus font identifier, shared by all trailkit addons
    /// (`trailkit.shared.font.<name>`, AGENTS.md naming conventions).
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Body => "trailkit.shared.font.body",
            Self::BodyBold => "trailkit.shared.font.body_bold",
            Self::BodyItalic => "trailkit.shared.font.body_italic",
            Self::BodyBoldItalic => "trailkit.shared.font.body_bold_italic",
            Self::Heading1 => "trailkit.shared.font.heading_1",
            Self::Heading2 => "trailkit.shared.font.heading_2",
            Self::Heading3 => "trailkit.shared.font.heading_3",
            Self::Icons => "trailkit.shared.font.icons",
        }
    }

    /// The font file of this role.
    #[must_use]
    pub fn data(self) -> &'static [u8] {
        match self {
            Self::Body => OPEN_SANS_REGULAR,
            Self::BodyBold | Self::Heading1 | Self::Heading2 | Self::Heading3 => OPEN_SANS_BOLD,
            Self::BodyItalic => OPEN_SANS_ITALIC,
            Self::BodyBoldItalic => OPEN_SANS_BOLD_ITALIC,
            Self::Icons => LUCIDE,
        }
    }

    /// Size relative to the body text (M35).
    #[must_use]
    pub const fn scale(self) -> f32 {
        match self {
            Self::Heading1 => 1.5,
            Self::Heading2 => 1.3,
            Self::Heading3 => 1.15,
            Self::Body | Self::BodyBold | Self::BodyItalic | Self::BodyBoldItalic | Self::Icons => {
                1.0
            }
        }
    }

    /// The pixel size of this role for a body size of `body_size`.
    #[must_use]
    pub fn size_for(self, body_size: f32) -> f32 {
        (body_size * self.scale()).round()
    }

    /// The body face for a combination of bold and italic.
    #[must_use]
    pub const fn body(bold: bool, italic: bool) -> Self {
        match (bold, italic) {
            (false, false) => Self::Body,
            (true, false) => Self::BodyBold,
            (false, true) => Self::BodyItalic,
            (true, true) => Self::BodyBoldItalic,
        }
    }

    /// The heading face for a markdown heading level (1 = largest).
    #[must_use]
    pub const fn heading(level: u8) -> Self {
        match level {
            0 | 1 => Self::Heading1,
            2 => Self::Heading2,
            _ => Self::Heading3,
        }
    }
}

/// Whether the TrueType font `data` has a glyph for `character`, read from
/// its Unicode `cmap` (formats 4 and 12). Used by tests that guard the font
/// subsets (memo plan M62): every UI string and icon must stay covered.
#[must_use]
pub fn covers(data: &[u8], character: char) -> bool {
    cmap_subtable(data)
        .is_some_and(|subtable| subtable_covers(data, subtable, u32::from(character)))
}

/// Reads a big-endian `u16` at `offset`.
fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        data.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

/// Reads a big-endian `u32` at `offset`.
fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        data.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

/// The offset of the best Unicode `cmap` subtable (format 12 preferred).
fn cmap_subtable(data: &[u8]) -> Option<usize> {
    let tables = usize::from(read_u16(data, 4)?);
    let cmap = (0..tables).find_map(|index| {
        let record = 12 + 16 * index;
        (data.get(record..record + 4)? == b"cmap").then(|| read_u32(data, record + 8))?
    })?;
    let cmap = usize::try_from(cmap).ok()?;
    let count = usize::from(read_u16(data, cmap + 2)?);
    let mut best: Option<(u16, usize)> = None;
    for index in 0..count {
        let record = cmap + 4 + 8 * index;
        let platform = read_u16(data, record)?;
        let encoding = read_u16(data, record + 2)?;
        let offset = cmap + usize::try_from(read_u32(data, record + 4)?).ok()?;
        let format = read_u16(data, offset)?;
        let is_unicode = platform == 0 || (platform == 3 && (encoding == 1 || encoding == 10));
        if is_unicode && (format == 4 || format == 12) && best.is_none_or(|(best, _)| format > best)
        {
            best = Some((format, offset));
        }
    }
    best.map(|(_, offset)| offset)
}

/// Whether the `cmap` subtable at `offset` maps `code` to a real glyph.
fn subtable_covers(data: &[u8], offset: usize, code: u32) -> bool {
    match read_u16(data, offset) {
        Some(12) => {
            let groups = read_u32(data, offset + 12).unwrap_or(0);
            (0..groups).any(|group| {
                let record = offset + 16 + 12 * usize::try_from(group).unwrap_or(usize::MAX / 24);
                matches!(
                    (read_u32(data, record), read_u32(data, record + 4)),
                    (Some(start), Some(end)) if (start..=end).contains(&code)
                )
            })
        }
        Some(4) => {
            let segments = usize::from(read_u16(data, offset + 6).unwrap_or(0) / 2);
            let ends = offset + 14;
            let starts = ends + 2 * segments + 2;
            (0..segments).any(|segment| {
                matches!(
                    (read_u16(data, starts + 2 * segment), read_u16(data, ends + 2 * segment)),
                    (Some(start), Some(end))
                        if (u32::from(start)..=u32::from(end)).contains(&code) && start != 0xFFFF
                )
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icons::Icon;

    #[test]
    fn subset_fonts_keep_icons_latin_and_typography() {
        for icon in Icon::ALL {
            assert!(
                covers(LUCIDE, icon.glyph()),
                "icon {:?} missing",
                icon.glyph()
            );
        }
        let text_fonts = [
            OPEN_SANS_REGULAR,
            OPEN_SANS_BOLD,
            OPEN_SANS_ITALIC,
            OPEN_SANS_BOLD_ITALIC,
        ];
        for font in text_fonts {
            for character in "Aaäöüß ÄÖÜ ŐőŁł •…–—„“”‘’› 0123456789".chars()
            {
                assert!(covers(font, character), "{character} missing");
            }
            assert!(!covers(font, '\u{4E2D}'), "subset: no CJK");
        }
    }

    #[test]
    fn identifiers_are_unique_and_shared() {
        for (index, role) in FontRole::ALL.iter().enumerate() {
            assert!(role.identifier().starts_with("trailkit.shared.font."));
            for other in &FontRole::ALL[index + 1..] {
                assert_ne!(role.identifier(), other.identifier());
            }
        }
    }

    #[test]
    fn heading_sizes_derive_from_body() {
        assert!((FontRole::Heading1.size_for(16.0) - 24.0).abs() < f32::EPSILON);
        assert!((FontRole::Body.size_for(16.0) - 16.0).abs() < f32::EPSILON);
        assert_eq!(FontRole::heading(5), FontRole::Heading3);
    }

    #[test]
    fn embedded_fonts_are_truetype() {
        for role in FontRole::ALL {
            assert_eq!(&role.data()[..4], &[0, 1, 0, 0], "{role:?}");
        }
    }
}
