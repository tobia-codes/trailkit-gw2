//! `cargo xtask subset-fonts [--check]`: cuts the embedded fonts down to the
//! characters trailkit can show (memo plan M62).
//!
//! Nexus only puts Basic Latin to Latin Extended-B and the characters of
//! registered texts into the shared font atlas, so every other glyph in an
//! embedded font is dead weight in the DLL. The originals stay in
//! `crates/trailkit-ui/assets/fonts/<font>/source/`; this command writes the
//! subset next to them (the files `trailkit-ui` embeds). Text fonts keep Latin
//! up to Extended-B plus general punctuation and arrows; the icon font keeps
//! only the icons in `trailkit_ui::icons::Icon::ALL`. `--check` only verifies
//! that the committed subsets match a fresh run.

use std::path::{Path, PathBuf};

use allsorts::{
    binary::read::ReadScope,
    font::read_cmap_subtable,
    font_data::FontData,
    subset::{CmapTarget, SubsetProfile, subset},
    tables::{FontTableProvider, cmap::Cmap},
    tag,
};
use anyhow::{Context, Result, anyhow, bail};
use trailkit_ui::icons::Icon;

use crate::workspace;

/// Unicode ranges kept in the text fonts (inclusive).
const TEXT_RANGES: [(u32, u32); 4] = [
    (0x0020, 0x007E),
    (0x00A0, 0x024F),
    (0x2000, 0x206F),
    (0x2190, 0x21FF),
];

/// Whether subsets are written or only verified.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Write the subset fonts.
    Write,
    /// Fail if a committed subset differs from a fresh one.
    Check,
}

/// Which characters a font keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharacterSet {
    /// [`TEXT_RANGES`].
    Text,
    /// The icons trailkit uses.
    Icons,
}

/// One font to subset, relative to the fonts directory.
struct FontJob {
    /// `<font>/source/<file>.ttf`.
    source: &'static str,
    /// `<font>/<file>.ttf`, the file `trailkit-ui` embeds.
    output: &'static str,
    /// Characters to keep.
    characters: CharacterSet,
}

/// Every embedded font.
const JOBS: [FontJob; 5] = [
    FontJob {
        source: "open-sans/source/open-sans-regular.ttf",
        output: "open-sans/open-sans-regular.ttf",
        characters: CharacterSet::Text,
    },
    FontJob {
        source: "open-sans/source/open-sans-bold.ttf",
        output: "open-sans/open-sans-bold.ttf",
        characters: CharacterSet::Text,
    },
    FontJob {
        source: "open-sans/source/open-sans-italic.ttf",
        output: "open-sans/open-sans-italic.ttf",
        characters: CharacterSet::Text,
    },
    FontJob {
        source: "open-sans/source/open-sans-bold-italic.ttf",
        output: "open-sans/open-sans-bold-italic.ttf",
        characters: CharacterSet::Text,
    },
    FontJob {
        source: "lucide/source/lucide.ttf",
        output: "lucide/lucide.ttf",
        characters: CharacterSet::Icons,
    },
];

/// Subsets (or checks) every embedded font.
///
/// # Errors
/// Returns an error if a font cannot be read, parsed, subset or written, or
/// (in check mode) if a committed subset is out of date.
pub fn run(mode: Mode) -> Result<()> {
    let fonts = workspace::root().join("crates/trailkit-ui/assets/fonts");
    let mut outdated = Vec::new();
    for job in &JOBS {
        let source = fonts.join(job.source);
        let output = fonts.join(job.output);
        let data =
            std::fs::read(&source).with_context(|| format!("reading {}", source.display()))?;
        let mut subset = subset_font(&data, &characters(job.characters))
            .with_context(|| format!("subsetting {}", source.display()))?;
        if job.characters == CharacterSet::Icons {
            fix_glyph_bounds(&mut subset)
                .with_context(|| format!("fixing glyph bounds of {}", source.display()))?;
        }
        match mode {
            Mode::Write => {
                std::fs::write(&output, &subset)
                    .with_context(|| format!("writing {}", output.display()))?;
                println!("{}: {} → {} bytes", job.output, data.len(), subset.len());
            }
            Mode::Check => {
                if std::fs::read(&output).ok().as_deref() != Some(subset.as_slice()) {
                    outdated.push(relative(&output));
                }
            }
        }
    }
    if !outdated.is_empty() {
        bail!(
            "subset fonts are out of date (run `cargo xtask subset-fonts`): {}",
            outdated.join(", ")
        );
    }
    Ok(())
}

/// The characters of a [`CharacterSet`].
fn characters(set: CharacterSet) -> Vec<char> {
    match set {
        CharacterSet::Text => TEXT_RANGES
            .iter()
            .flat_map(|(first, last)| (*first..=*last).filter_map(char::from_u32))
            .collect(),
        CharacterSet::Icons => Icon::ALL.iter().map(|icon| icon.glyph()).collect(),
    }
}

/// Returns `data` (a TrueType font) reduced to the glyphs of `characters`
/// (missing ones are skipped), keeping a Unicode `cmap`.
///
/// # Errors
/// Returns an error if the font cannot be parsed or subset.
fn subset_font(data: &[u8], characters: &[char]) -> Result<Vec<u8>> {
    let font_file = ReadScope::new(data)
        .read::<FontData<'_>>()
        .map_err(|error| anyhow!("parsing the font: {error}"))?;
    let provider = font_file
        .table_provider(0)
        .map_err(|error| anyhow!("reading the font tables: {error}"))?;
    let cmap_data = provider
        .read_table_data(tag::CMAP)
        .map_err(|error| anyhow!("reading cmap: {error}"))?;
    let cmap = ReadScope::new(&cmap_data)
        .read::<Cmap<'_>>()
        .map_err(|error| anyhow!("parsing cmap: {error}"))?;
    let (_, subtable) = read_cmap_subtable(&cmap)
        .map_err(|error| anyhow!("reading the cmap subtable: {error}"))?
        .ok_or_else(|| anyhow!("the font has no Unicode cmap"))?;
    // Glyph 0 (.notdef) must come first; no duplicates.
    let mut glyph_ids = vec![0_u16];
    for character in characters {
        if let Ok(Some(glyph)) = subtable.map_glyph(u32::from(*character))
            && !glyph_ids.contains(&glyph)
        {
            glyph_ids.push(glyph);
        }
    }
    subset(
        &provider,
        &glyph_ids,
        &SubsetProfile::Minimal,
        CmapTarget::Unicode,
    )
    .map_err(|error| anyhow!("subsetting: {error}"))
}

/// Big-endian `u16` at `offset`.
fn read_u16(font: &[u8], offset: usize) -> Result<u16> {
    let bytes = font
        .get(offset..offset + 2)
        .ok_or_else(|| anyhow!("font data ends at {offset}"))?;
    Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
}

/// Big-endian `u32` at `offset`.
fn read_u32(font: &[u8], offset: usize) -> Result<u32> {
    let bytes = font
        .get(offset..offset + 4)
        .ok_or_else(|| anyhow!("font data ends at {offset}"))?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// One entry of a font's table directory.
#[derive(Debug, Clone, Copy)]
struct TableRecord {
    /// Table tag, e.g. `glyf`.
    tag: [u8; 4],
    /// Byte offset of the directory record itself (for its checksum).
    record: usize,
    /// Byte offset of the table.
    offset: usize,
    /// Table length in bytes.
    length: usize,
}

/// Every table record of `font`.
fn table_directory(font: &[u8]) -> Result<Vec<TableRecord>> {
    let count = usize::from(read_u16(font, 4)?);
    (0..count)
        .map(|index| {
            let record = 12 + 16 * index;
            let tag = font
                .get(record..record + 4)
                .and_then(|tag| <[u8; 4]>::try_from(tag).ok())
                .ok_or_else(|| anyhow!("table record {index} is cut off"))?;
            let offset = usize::try_from(read_u32(font, record + 8)?)?;
            let length = usize::try_from(read_u32(font, record + 12)?)?;
            Ok(TableRecord {
                tag,
                record,
                offset,
                length,
            })
        })
        .collect()
}

/// The OpenType checksum of `data` (sum of big-endian `u32`s, zero padded).
fn checksum(data: &[u8]) -> u32 {
    data.chunks(4).fold(0_u32, |sum, chunk| {
        let mut word = [0_u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

/// Rewrites every simple glyph's bounding box from its outline points.
///
/// The Lucide font stores `xMin`/`yMin` as 0 for every glyph although the
/// outlines start further in (e.g. `x`: header 0–791, points 209–791).
/// ImGui sizes glyph quads by these boxes, so icons could not be centered by
/// their ink (memo M128). Drawing positions do not change: the outline stays
/// where it is, only its box becomes tight. Composite glyphs are left alone.
///
/// # Errors
/// Returns an error if a required table is missing or a glyph is malformed.
fn fix_glyph_bounds(font: &mut [u8]) -> Result<()> {
    let tables = table_directory(font)?;
    let find = |wanted: &[u8; 4]| {
        tables
            .iter()
            .find(|table| &table.tag == wanted)
            .copied()
            .ok_or_else(|| anyhow!("no {} table", String::from_utf8_lossy(wanted)))
    };
    let head = find(b"head")?.offset;
    let maxp = find(b"maxp")?.offset;
    let loca = find(b"loca")?.offset;
    let TableRecord {
        record: glyf_record,
        offset: glyf,
        length: glyf_length,
        ..
    } = find(b"glyf")?;
    let long_offsets = read_u16(font, head + 50)? == 1;
    let glyph_count = usize::from(read_u16(font, maxp + 4)?);
    let glyph_offset = |font: &[u8], glyph: usize| -> Result<usize> {
        Ok(if long_offsets {
            usize::try_from(read_u32(font, loca + 4 * glyph)?)?
        } else {
            usize::from(read_u16(font, loca + 2 * glyph)?) * 2
        })
    };
    for glyph in 0..glyph_count {
        let start = glyf + glyph_offset(font, glyph)?;
        let end = glyf + glyph_offset(font, glyph + 1)?;
        if end <= start {
            continue;
        }
        let Some(bounds) = outline_bounds(font, start)? else {
            continue;
        };
        for (index, value) in bounds.iter().enumerate() {
            let at = start + 2 + 2 * index;
            font[at..at + 2].copy_from_slice(&value.to_be_bytes());
        }
    }
    // Keep the checksums valid: the glyf table changed, so did the font.
    let glyf_checksum = checksum(&font[glyf..glyf + glyf_length]);
    font[glyf_record + 4..glyf_record + 8].copy_from_slice(&glyf_checksum.to_be_bytes());
    font[head + 8..head + 12].copy_from_slice(&[0; 4]);
    let adjustment = 0xB1B0_AFBA_u32.wrapping_sub(checksum(font));
    font[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    Ok(())
}

/// `[xMin, yMin, xMax, yMax]` over the points of the simple glyph at
/// `start`, or `None` for a composite glyph or one without points.
///
/// # Errors
/// Returns an error if the glyph data is cut off.
fn outline_bounds(font: &[u8], start: usize) -> Result<Option<[i16; 4]>> {
    let contours = read_u16(font, start)?;
    if contours == 0 || contours & 0x8000 != 0 {
        return Ok(None);
    }
    let contours = usize::from(contours);
    let point_count = usize::from(read_u16(font, start + 10 + 2 * (contours - 1))?) + 1;
    let instructions = usize::from(read_u16(font, start + 10 + 2 * contours)?);
    let mut cursor = start + 12 + 2 * contours + instructions;
    let byte_at = |at: usize| {
        font.get(at)
            .copied()
            .ok_or_else(|| anyhow!("glyph data ends at {at}"))
    };
    let mut flags = Vec::with_capacity(point_count);
    while flags.len() < point_count {
        let flag = byte_at(cursor)?;
        cursor += 1;
        flags.push(flag);
        if flag & 0x08 != 0 {
            let repeat = byte_at(cursor)?;
            cursor += 1;
            flags.extend(std::iter::repeat_n(flag, usize::from(repeat)));
        }
    }
    flags.truncate(point_count);
    // x then y: (short-vector bit, same-or-positive bit).
    let mut coordinates = [
        Vec::with_capacity(point_count),
        Vec::with_capacity(point_count),
    ];
    for (axis, (short, same)) in [(0x02_u8, 0x10_u8), (0x04, 0x20)].into_iter().enumerate() {
        let mut value = 0_i32;
        for &flag in &flags {
            if flag & short != 0 {
                let delta = i32::from(byte_at(cursor)?);
                cursor += 1;
                value += if flag & same != 0 { delta } else { -delta };
            } else if flag & same == 0 {
                value += i32::from(read_u16(font, cursor)?.cast_signed());
                cursor += 2;
            }
            coordinates[axis].push(value);
        }
    }
    let bound = |values: &[i32], pick: fn(i32, i32) -> i32| -> Result<i16> {
        let value = values
            .iter()
            .copied()
            .reduce(pick)
            .ok_or_else(|| anyhow!("glyph without points"))?;
        Ok(i16::try_from(value)?)
    };
    Ok(Some([
        bound(&coordinates[0], i32::min)?,
        bound(&coordinates[1], i32::min)?,
        bound(&coordinates[0], i32::max)?,
        bound(&coordinates[1], i32::max)?,
    ]))
}

/// `path` relative to the repository root, for messages.
fn relative(path: &Path) -> String {
    path.strip_prefix(workspace::root())
        .map_or_else(|_| path.to_path_buf(), PathBuf::from)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_glyph_boxes_are_tight() {
        let fonts = workspace::root().join("crates/trailkit-ui/assets/fonts");
        let source = std::fs::read(fonts.join("lucide/source/lucide.ttf"))
            .unwrap_or_else(|error| panic!("reading the Lucide source: {error}"));
        let mut font = subset_font(&source, &[Icon::X.glyph()])
            .unwrap_or_else(|error| panic!("subsetting: {error}"));
        fix_glyph_bounds(&mut font).unwrap_or_else(|error| panic!("fixing: {error}"));
        // Glyph 1 is the `x`; its header now matches its points.
        let tables = table_directory(&font).unwrap_or_else(|error| panic!("{error}"));
        let offset_of = |wanted: &[u8; 4]| {
            tables
                .iter()
                .find(|table| &table.tag == wanted)
                .map(|table| table.offset)
                .unwrap_or_else(|| panic!("table missing"))
        };
        let (head, loca, glyf) = (offset_of(b"head"), offset_of(b"loca"), offset_of(b"glyf"));
        let long = read_u16(&font, head + 50).unwrap_or_default() == 1;
        let start = glyf
            + if long {
                usize::try_from(read_u32(&font, loca + 4).unwrap_or_default()).unwrap_or_default()
            } else {
                usize::from(read_u16(&font, loca + 2).unwrap_or_default()) * 2
            };
        let header: Vec<u16> = (0..4)
            .map(|index| read_u16(&font, start + 2 + 2 * index).unwrap_or_default())
            .collect();
        let points = outline_bounds(&font, start)
            .unwrap_or_else(|error| panic!("{error}"))
            .unwrap_or_else(|| panic!("x is a simple glyph"));
        assert_eq!(header, points.map(i16::cast_unsigned));
        assert!(
            points[0] > 100 && points[1] > 100,
            "box starts inside: {points:?}"
        );
    }

    #[test]
    fn text_set_covers_umlauts_and_typography() {
        let set = characters(CharacterSet::Text);
        for character in ['A', 'ä', 'ß', 'Ő', '…', '„', '–', '→'] {
            assert!(set.contains(&character), "{character} missing");
        }
        assert!(!set.contains(&'\u{4E2D}'), "no CJK");
    }

    #[test]
    fn icon_set_is_the_used_icons() {
        assert_eq!(characters(CharacterSet::Icons).len(), Icon::ALL.len());
    }
}
