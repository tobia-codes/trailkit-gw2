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
        let subset = subset_font(&data, &characters(job.characters))
            .with_context(|| format!("subsetting {}", source.display()))?;
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
