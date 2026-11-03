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
    /// `panel-left-close`: hide the list on the left (memo note list, M101).
    pub const PANEL_LEFT_CLOSE: Self = Self('\u{e21c}');
    /// `panel-left-open`: show the list on the left (memo note list, M101).
    pub const PANEL_LEFT_OPEN: Self = Self('\u{e21d}');
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
    /// `heading-1`: format as heading level 1 (memo M100).
    pub const HEADING_1: Self = Self('\u{e385}');
    /// `heading-2`: format as heading level 2 (memo M100).
    pub const HEADING_2: Self = Self('\u{e386}');
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

    /// `coins`: coins chat link (M75).
    pub const COINS: Self = Self('\u{e097}');
    /// `package`: item chat link (M75).
    pub const PACKAGE: Self = Self('\u{e129}');
    /// `message-square`: NPC text chat link (M75).
    pub const MESSAGE_SQUARE: Self = Self('\u{e117}');
    /// `map-pin`: waypoint / point of interest / vista chat link (M75).
    pub const MAP_PIN: Self = Self('\u{e111}');
    /// `swords`: skill chat link (M75).
    pub const SWORDS: Self = Self('\u{e2b4}');
    /// `sparkles`: trait chat link (M75).
    pub const SPARKLES: Self = Self('\u{e412}');
    /// `user`: player chat link (M75).
    pub const USER: Self = Self('\u{e19f}');
    /// `scroll-text`: recipe chat link (M75).
    pub const SCROLL_TEXT: Self = Self('\u{e45f}');
    /// `shirt`: skin chat link (M75).
    pub const SHIRT: Self = Self('\u{e1ca}');
    /// `venetian-mask`: outfit chat link (M75).
    pub const VENETIAN_MASK: Self = Self('\u{e2aa}');
    /// `castle`: WvW objective chat link (M75).
    pub const CASTLE: Self = Self('\u{e3e0}');
    /// `hammer`: build template chat link (M75).
    pub const HAMMER: Self = Self('\u{e0ec}');
    /// `trophy`: achievement chat link (M75).
    pub const TROPHY: Self = Self('\u{e373}');
    /// `chevron-down`: opens a dropdown (memo M87).
    pub const CHEVRON_DOWN: Self = Self('\u{e06d}');
    /// `check`: marks the chosen option of a dropdown or menu (memo M87, M99).
    pub const CHECK: Self = Self('\u{e06c}');
    /// `chevron-right`: a menu entry that opens a submenu (memo M99).
    pub const CHEVRON_RIGHT: Self = Self('\u{e06f}');
    /// `ban`: a drag over a place it cannot be dropped (memo M109).
    pub const BAN: Self = Self('\u{e051}');
    /// `x`: closes a window from its title bar (R71).
    pub const X: Self = Self('\u{e1b2}');
    /// Every icon above; keep in sync when adding one.
    pub const ALL: [Self; 36] = [
        Self::X,
        Self::FILE_PLUS,
        Self::FOLDER_PLUS,
        Self::PANEL_LEFT_CLOSE,
        Self::PANEL_LEFT_OPEN,
        Self::SQUARE,
        Self::SQUARE_CHECK,
        Self::TRIANGLE_ALERT,
        Self::FOLDER_OPEN,
        Self::PENCIL,
        Self::HEADING_1,
        Self::HEADING_2,
        Self::BOLD,
        Self::ITALIC,
        Self::LIST,
        Self::LIST_TODO,
        Self::LINK,
        Self::CALENDAR_CLOCK,
        Self::EYE,
        Self::COINS,
        Self::PACKAGE,
        Self::MESSAGE_SQUARE,
        Self::MAP_PIN,
        Self::SWORDS,
        Self::SPARKLES,
        Self::USER,
        Self::SCROLL_TEXT,
        Self::SHIRT,
        Self::VENETIAN_MASK,
        Self::CASTLE,
        Self::HAMMER,
        Self::TROPHY,
        Self::CHEVRON_DOWN,
        Self::CHECK,
        Self::CHEVRON_RIGHT,
        Self::BAN,
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
