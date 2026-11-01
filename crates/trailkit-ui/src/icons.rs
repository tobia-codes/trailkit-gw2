//! Lucide icons used by trailkit addons (memo plan M36, `lay-icons`).
//!
//! Each icon is one glyph of the Lucide icon font (codepoints from
//! `lucide-static`'s `font/codepoints.json`). Nexus only puts glyphs into the
//! font atlas that appear in registered texts, so [`glyph_text`] lists every
//! icon below and is registered as a localization text (nexus-addon-dev
//! skill §1). Add new icons here, never inline.

/// One Lucide icon glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Icon(char);

impl Icon {
    /// `file-plus`: create a note.
    pub const FILE_PLUS: Self = Self('\u{e0c9}');
    /// `folder-plus`: create a group.
    pub const FOLDER_PLUS: Self = Self('\u{e0d9}');
    /// `menu`: show or hide a list (memo note list, M48).
    pub const MENU: Self = Self('\u{e115}');
    /// `square`: unchecked task.
    pub const SQUARE: Self = Self('\u{e167}');
    /// `square-check`: checked task (tick inside the box, M64).
    pub const SQUARE_CHECK: Self = Self('\u{e559}');
    /// `triangle-alert`: warning or error hint.
    pub const TRIANGLE_ALERT: Self = Self('\u{e193}');
    /// `folder-open`: open a folder in the file explorer.
    pub const FOLDER_OPEN: Self = Self('\u{e247}');
    /// `pencil`: start editing.
    pub const PENCIL: Self = Self('\u{e1f9}');
    /// `heading`: format as heading.
    pub const HEADING: Self = Self('\u{e384}');
    /// `bold`: format as bold.
    pub const BOLD: Self = Self('\u{e05d}');
    /// `italic`: format as italic.
    pub const ITALIC: Self = Self('\u{e0fb}');
    /// `list`: bulleted list.
    pub const LIST: Self = Self('\u{e106}');
    /// `list-todo`: checkbox (task) list item.
    pub const LIST_TODO: Self = Self('\u{e4c3}');
    /// `link`: insert a link.
    pub const LINK: Self = Self('\u{e102}');
    /// `calendar-clock`: insert an event reference.
    pub const CALENDAR_CLOCK: Self = Self('\u{e304}');
    /// `eye`: show or hide the preview (M60).
    pub const EYE: Self = Self('\u{e0ba}');

    /// Every icon above; keep in sync when adding one.
    pub const ALL: [Self; 16] = [
        Self::FILE_PLUS,
        Self::FOLDER_PLUS,
        Self::MENU,
        Self::SQUARE,
        Self::SQUARE_CHECK,
        Self::TRIANGLE_ALERT,
        Self::FOLDER_OPEN,
        Self::PENCIL,
        Self::HEADING,
        Self::BOLD,
        Self::ITALIC,
        Self::LIST,
        Self::LIST_TODO,
        Self::LINK,
        Self::CALENDAR_CLOCK,
        Self::EYE,
    ];

    /// The glyph as a character of the icon font.
    #[must_use]
    pub const fn glyph(self) -> char {
        self.0
    }

    /// The glyph as a string slice (for ImGui text calls), written into `buffer`.
    #[must_use]
    pub fn as_str(self, buffer: &mut [u8; 4]) -> &str {
        self.0.encode_utf8(buffer)
    }
}

/// All icon glyphs as one string, registered with Nexus so the atlas contains them.
#[must_use]
pub fn glyph_text() -> String {
    Icon::ALL.iter().map(|icon| icon.glyph()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_are_in_the_private_use_area_and_unique() {
        for (index, icon) in Icon::ALL.iter().enumerate() {
            assert!(('\u{e000}'..='\u{f8ff}').contains(&icon.glyph()));
            assert!(!Icon::ALL[index + 1..].contains(icon));
        }
        assert_eq!(glyph_text().chars().count(), Icon::ALL.len());
    }
}
