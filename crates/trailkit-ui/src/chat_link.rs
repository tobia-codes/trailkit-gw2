//! GW2 chat links (`[&…]`): finding and decoding them locally (memo plan
//! M61, M75).
//!
//! Every chat link is `[&` + base64 + `]`; the first decoded byte is the link
//! type, the rest is a type-specific payload (little-endian numbers). Names
//! are not part of a link: without trailkit-core a chip shows the type and
//! the id. Platform independent; `chat_link_chip` (Windows only) draws it.

use std::ops::Range;

/// The kind of thing a chat link points to (for icons and labels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChatLinkKind {
    /// An amount of coins.
    Coins,
    /// An item stack.
    Item,
    /// An NPC dialogue text.
    NpcText,
    /// A waypoint, point of interest or vista (one type in the game).
    MapPoint,
    /// A skill.
    Skill,
    /// A trait.
    Trait,
    /// A player.
    Player,
    /// A crafting recipe.
    Recipe,
    /// A wardrobe skin.
    Skin,
    /// An outfit.
    Outfit,
    /// A World vs. World objective.
    WvwObjective,
    /// A build template.
    BuildTemplate,
    /// An achievement.
    Achievement,
}

impl ChatLinkKind {
    /// Every kind.
    pub const ALL: [Self; 13] = [
        Self::Coins,
        Self::Item,
        Self::NpcText,
        Self::MapPoint,
        Self::Skill,
        Self::Trait,
        Self::Player,
        Self::Recipe,
        Self::Skin,
        Self::Outfit,
        Self::WvwObjective,
        Self::BuildTemplate,
        Self::Achievement,
    ];
}

/// A profession, as stored in build template links.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profession {
    /// Guardian (code 1).
    Guardian,
    /// Warrior (code 2).
    Warrior,
    /// Engineer (code 3).
    Engineer,
    /// Ranger (code 4).
    Ranger,
    /// Thief (code 5).
    Thief,
    /// Elementalist (code 6).
    Elementalist,
    /// Mesmer (code 7).
    Mesmer,
    /// Necromancer (code 8).
    Necromancer,
    /// Revenant (code 9).
    Revenant,
}

impl Profession {
    /// Every profession, in code order.
    pub const ALL: [Self; 9] = [
        Self::Guardian,
        Self::Warrior,
        Self::Engineer,
        Self::Ranger,
        Self::Thief,
        Self::Elementalist,
        Self::Mesmer,
        Self::Necromancer,
        Self::Revenant,
    ];

    /// The profession with build template code `code` (1–9).
    #[must_use]
    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.get(usize::from(code.checked_sub(1)?)).copied()
    }
}

/// A decoded chat link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatLink {
    /// Coins, in copper.
    Coins {
        /// The amount in copper (1 gold = 100 silver = 10 000 copper).
        copper: u32,
    },
    /// An item stack.
    Item {
        /// The item id.
        id: u32,
        /// The stack size.
        quantity: u8,
    },
    /// A thing identified by a single id (NPC text, map point, skill, trait,
    /// recipe, skin, outfit, achievement).
    Id {
        /// What the id refers to.
        kind: ChatLinkKind,
        /// The id.
        id: u32,
    },
    /// A player.
    Player,
    /// A World vs. World objective; the API id is `"{map_id}-{objective_id}"`.
    WvwObjective {
        /// The objective id within its map.
        objective_id: u32,
        /// The WvW map id.
        map_id: u32,
    },
    /// A build template.
    BuildTemplate {
        /// The profession of the build.
        profession: Profession,
    },
}

impl ChatLink {
    /// What the link points to.
    #[must_use]
    pub const fn kind(&self) -> ChatLinkKind {
        match self {
            Self::Coins { .. } => ChatLinkKind::Coins,
            Self::Item { .. } => ChatLinkKind::Item,
            Self::Id { kind, .. } => *kind,
            Self::Player => ChatLinkKind::Player,
            Self::WvwObjective { .. } => ChatLinkKind::WvwObjective,
            Self::BuildTemplate { .. } => ChatLinkKind::BuildTemplate,
        }
    }
}

/// Why a chat code could not be decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatLinkError {
    /// The base64 or the payload is broken (wrong length, bad characters).
    Damaged,
    /// The type byte is not a known link type.
    UnknownType,
}

/// Characters a chip shows besides its translated texts (ids, quantities,
/// coin amounts); addons register them for the font atlas.
pub const CHIP_DETAIL_CHARACTERS: &str = " #×-()0123456789";

/// A translatable text of a chat-link chip; the addon maps it to its
/// language (trailkit-ui has no strings of its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipText {
    /// The name of a link kind, e.g. "Item".
    Kind(ChatLinkKind),
    /// The name of a profession.
    Profession(Profession),
    /// Label of a chip whose code cannot be decoded.
    InvalidChatLink,
    /// Tooltip reason: unknown link type.
    UnknownLinkType,
    /// Tooltip reason: damaged code.
    DamagedCode,
    /// Tooltip hint: a click copies the code.
    ClickToCopy,
    /// Tooltip after a click: the code was copied.
    Copied,
}

impl ChipText {
    /// Every chip text.
    pub fn all() -> impl Iterator<Item = Self> {
        ChatLinkKind::ALL
            .into_iter()
            .map(Self::Kind)
            .chain(Profession::ALL.into_iter().map(Self::Profession))
            .chain([
                Self::InvalidChatLink,
                Self::UnknownLinkType,
                Self::DamagedCode,
                Self::ClickToCopy,
                Self::Copied,
            ])
    }
}

/// A chat code found in a text, decoded once and ready to draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLinkChip {
    /// The raw code, e.g. `[&AgH1WQAA]`.
    pub code: String,
    /// The decoded link, or why it failed.
    pub link: Result<ChatLink, ChatLinkError>,
    /// Language-independent text after the kind name, e.g. `" #19721 ×250"`
    /// (empty for coins and invalid codes).
    pub detail: String,
}

impl ChatLinkChip {
    /// Decodes `code` (a whole `[&…]` code).
    #[must_use]
    pub fn new(code: &str) -> Self {
        let link = decode(code);
        let detail = match link {
            Ok(ChatLink::Item { id, quantity }) if quantity > 1 => format!(" #{id} ×{quantity}"),
            Ok(ChatLink::Item { id, .. } | ChatLink::Id { id, .. }) => format!(" #{id}"),
            Ok(ChatLink::WvwObjective {
                objective_id,
                map_id,
            }) => format!(" #{map_id}-{objective_id}"),
            Ok(ChatLink::Coins { .. } | ChatLink::Player | ChatLink::BuildTemplate { .. })
            | Err(_) => String::new(),
        };
        Self {
            code: code.to_owned(),
            link,
            detail,
        }
    }
}

/// The byte ranges of every chat code candidate in `text`: `[&`, one or more
/// base64 characters, `]`. Anything else (e.g. `[&hello world]`) is text.
pub fn find_chat_codes(text: &str) -> impl Iterator<Item = Range<usize>> + '_ {
    let bytes = text.as_bytes();
    let mut search_from = 0;
    std::iter::from_fn(move || {
        while let Some(found) = text.get(search_from..)?.find("[&") {
            let start = search_from + found;
            let body_start = start + 2;
            let body_length = bytes[body_start..]
                .iter()
                .take_while(|byte| is_base64_byte(**byte))
                .count();
            let end = body_start + body_length;
            if body_length > 0 && bytes.get(end) == Some(&b']') {
                search_from = end + 1;
                return Some(start..end + 1);
            }
            search_from = body_start;
        }
        None
    })
}

/// The chat code range that strictly contains `position` (the cursor may sit
/// before or after a code, never inside it; memo plan M75).
#[must_use]
pub fn chat_code_around(text: &str, position: usize) -> Option<Range<usize>> {
    find_chat_codes(text)
        .take_while(|range| range.start < position)
        .find(|range| position < range.end)
}

/// Decodes a whole `[&…]` code.
///
/// # Errors
///
/// [`ChatLinkError::Damaged`] if the code is not valid base64 or the payload
/// is too short for its type, [`ChatLinkError::UnknownType`] for an unknown
/// type byte.
pub fn decode(code: &str) -> Result<ChatLink, ChatLinkError> {
    let body = code
        .strip_prefix("[&")
        .and_then(|rest| rest.strip_suffix(']'))
        .ok_or(ChatLinkError::Damaged)?;
    let bytes = decode_base64(body).ok_or(ChatLinkError::Damaged)?;
    let (&link_type, payload) = bytes.split_first().ok_or(ChatLinkError::Damaged)?;
    let id = |kind| {
        read_u32(payload, 0)
            .map(|id| ChatLink::Id { kind, id })
            .ok_or(ChatLinkError::Damaged)
    };
    match link_type {
        0x01 => read_u32(payload, 0)
            .map(|copper| ChatLink::Coins { copper })
            .ok_or(ChatLinkError::Damaged),
        0x02 => decode_item(payload),
        0x03 => id(ChatLinkKind::NpcText),
        0x04 => id(ChatLinkKind::MapPoint),
        0x06 => id(ChatLinkKind::Skill),
        0x07 => id(ChatLinkKind::Trait),
        0x08 if !payload.is_empty() => Ok(ChatLink::Player),
        0x09 => id(ChatLinkKind::Recipe),
        0x0A => id(ChatLinkKind::Skin),
        0x0B => id(ChatLinkKind::Outfit),
        0x0C => match (read_u32(payload, 0), read_u32(payload, 4)) {
            (Some(objective_id), Some(map_id)) => Ok(ChatLink::WvwObjective {
                objective_id,
                map_id,
            }),
            _ => Err(ChatLinkError::Damaged),
        },
        0x0D => payload
            .first()
            .copied()
            .and_then(Profession::from_code)
            .map(|profession| ChatLink::BuildTemplate { profession })
            .ok_or(ChatLinkError::Damaged),
        0x0E => id(ChatLinkKind::Achievement),
        0x08 => Err(ChatLinkError::Damaged),
        _ => Err(ChatLinkError::UnknownType),
    }
}

/// Decodes an item payload: quantity, 3-byte id, flags, then 4 bytes for
/// each of skin (`0x80`), first (`0x40`) and second upgrade (`0x20`).
fn decode_item(payload: &[u8]) -> Result<ChatLink, ChatLinkError> {
    let (&quantity, rest) = payload.split_first().ok_or(ChatLinkError::Damaged)?;
    let [low, middle, high, flags] = *rest.first_chunk::<4>().ok_or(ChatLinkError::Damaged)?;
    let extras = [0x80, 0x40, 0x20]
        .iter()
        .filter(|flag| flags & **flag != 0)
        .count();
    if rest.len() < 4 + 4 * extras {
        return Err(ChatLinkError::Damaged);
    }
    Ok(ChatLink::Item {
        id: u32::from_le_bytes([low, middle, high, 0]),
        quantity,
    })
}

/// The little-endian `u32` at `offset`, if the payload is long enough.
fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset + 4)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

/// Whether `byte` may appear in a base64 code body (`=` only as padding).
const fn is_base64_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')
}

/// The 6-bit value of a base64 character.
const fn base64_value(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// Decodes standard base64 (padding optional); `None` if it is malformed.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let data = text.trim_end_matches('=');
    if text.len() - data.len() > 2 || data.len() % 4 == 1 {
        return None;
    }
    let mut bytes = Vec::with_capacity(data.len() * 3 / 4);
    let mut bits: u32 = 0;
    let mut bit_count = 0;
    for byte in data.bytes() {
        bits = (bits << 6) | u32::from(base64_value(byte)?);
        bit_count += 6;
        if bit_count >= 8 {
            bit_count -= 8;
            bytes.push(u8::try_from((bits >> bit_count) & 0xFF).ok()?);
        }
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_codes_and_skips_look_alikes() {
        let text = "Buy [&AgH1WQAA] at [&BDgAAAA=], not [&hello world] or [&] or [&abc";
        let found: Vec<&str> = find_chat_codes(text).map(|range| &text[range]).collect();
        assert_eq!(found, ["[&AgH1WQAA]", "[&BDgAAAA=]"]);
    }

    #[test]
    fn decodes_items_coins_and_map_points() {
        assert_eq!(
            decode("[&AgH1WQAA]"),
            Ok(ChatLink::Item {
                id: 23029,
                quantity: 1
            })
        );
        assert_eq!(decode("[&AdsnAAA=]"), Ok(ChatLink::Coins { copper: 10203 }));
        assert_eq!(
            decode("[&BDgAAAA=]"),
            Ok(ChatLink::Id {
                kind: ChatLinkKind::MapPoint,
                id: 56
            })
        );
    }

    #[test]
    fn decodes_wvw_objectives_and_build_templates() {
        // 0x0C, objective 6, map 38.
        assert_eq!(
            decode("[&DAYAAAAmAAAA]"),
            Ok(ChatLink::WvwObjective {
                objective_id: 6,
                map_id: 38
            })
        );
        // 0x0D, profession 7 (Mesmer), rest of the template omitted.
        assert_eq!(
            decode("[&DQc=]"),
            Ok(ChatLink::BuildTemplate {
                profession: Profession::Mesmer
            })
        );
    }

    #[test]
    fn rejects_damaged_codes_and_unknown_types() {
        assert_eq!(decode("[&AgH1]"), Err(ChatLinkError::Damaged));
        assert_eq!(decode("[&A]"), Err(ChatLinkError::Damaged));
        // 0x2A is no link type.
        assert_eq!(decode("[&KgAAAAA=]"), Err(ChatLinkError::UnknownType));
        // An item that claims a skin but has none.
        assert_eq!(decode("[&AgH1WQCA]"), Err(ChatLinkError::Damaged));
    }

    #[test]
    fn chip_detail_shows_id_and_quantity() {
        // 0x02, quantity 250, id 19721.
        let chip = ChatLinkChip::new("[&AvoJTQAA]");
        assert_eq!(chip.detail, " #19721 ×250");
        assert_eq!(ChatLinkChip::new("[&BDgAAAA=]").detail, " #56");
        assert_eq!(ChatLinkChip::new("[&DAYAAAAmAAAA]").detail, " #38-6");
    }

    #[test]
    fn positions_inside_a_code_are_found() {
        let text = "a [&BDgAAAA=] b";
        assert_eq!(chat_code_around(text, 2), None);
        assert_eq!(chat_code_around(text, 3), Some(2..13));
        assert_eq!(chat_code_around(text, 12), Some(2..13));
        assert_eq!(chat_code_around(text, 13), None);
    }
}
