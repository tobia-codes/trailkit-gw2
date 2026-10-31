//! `cargo xtask build-icons [--check]` — renders the addons' SVG icons to the
//! committed PNGs that the DLLs embed (ROADMAP R55, R57, R58).
//!
//! Every `addons/<addon>/assets/icons/<name>.svg` produces `<name>.png` and a
//! lighter `<name>-hover.png` next to it. The hover variant is derived by
//! replacing the parchment fill token [`PARCHMENT_FILL`] with
//! [`PARCHMENT_HOVER_FILL`], so each icon has exactly one hand-written source.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use resvg::{tiny_skia, usvg};

use crate::workspace::Addon;

/// Edge length of a Nexus quick access icon in pixels.
const ICON_SIZE: u32 = 64;

/// Parchment fill every icon SVG must use for its base shape.
const PARCHMENT_FILL: &str = "#E9DCB8";

/// Lighter parchment used for the hover variant.
const PARCHMENT_HOVER_FILL: &str = "#FAF1DA";

/// Suffix appended to the file stem of the hover variant.
const HOVER_SUFFIX: &str = "-hover";

/// Whether the command writes the PNGs or only verifies them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Render and overwrite the PNG files.
    Write,
    /// Fail if any committed PNG differs from a fresh render (CI).
    Check,
}

/// One PNG that should exist on disk, with its freshly rendered contents.
struct RenderedIcon {
    /// Target path of the PNG.
    path: PathBuf,
    /// Encoded PNG bytes.
    png: Vec<u8>,
}

/// Runs the command for every addon.
///
/// # Errors
/// Returns an error if an SVG cannot be read or rendered, a PNG cannot be
/// written, or (in [`Mode::Check`]) a committed PNG is missing or outdated.
pub fn run(mode: Mode) -> Result<()> {
    let mut outdated = Vec::new();
    for addon in Addon::all()? {
        for svg_path in icon_sources(&addon.dir().join("assets").join("icons"))? {
            for icon in render_icon_variants(&svg_path)? {
                let current = std::fs::read(&icon.path).ok();
                if current.as_deref() == Some(icon.png.as_slice()) {
                    continue;
                }
                match mode {
                    Mode::Write => {
                        std::fs::write(&icon.path, &icon.png)
                            .with_context(|| format!("writing {}", icon.path.display()))?;
                        println!("wrote {}", icon.path.display());
                    }
                    Mode::Check => outdated.push(icon.path),
                }
            }
        }
    }
    if !outdated.is_empty() {
        let list: Vec<String> = outdated
            .iter()
            .map(|path| path.display().to_string())
            .collect();
        bail!(
            "icon PNGs are outdated — run `cargo xtask build-icons`:\n  {}",
            list.join("\n  ")
        );
    }
    Ok(())
}

/// Lists the `.svg` files in `icons_dir`, sorted; an absent directory yields none.
///
/// # Errors
/// Returns an error if the directory exists but cannot be read.
fn icon_sources(icons_dir: &Path) -> Result<Vec<PathBuf>> {
    if !icons_dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut sources = Vec::new();
    for entry in
        std::fs::read_dir(icons_dir).with_context(|| format!("reading {}", icons_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().is_some_and(|extension| extension == "svg") {
            sources.push(path);
        }
    }
    sources.sort();
    Ok(sources)
}

/// Renders the normal and the hover variant of one SVG icon.
///
/// # Errors
/// Returns an error if the file cannot be read, lacks the parchment fill token
/// or cannot be rendered.
fn render_icon_variants(svg_path: &Path) -> Result<Vec<RenderedIcon>> {
    let svg = std::fs::read_to_string(svg_path)
        .with_context(|| format!("reading {}", svg_path.display()))?;
    if !svg.contains(PARCHMENT_FILL) {
        bail!(
            "{} does not use the parchment fill {PARCHMENT_FILL} (needed for the hover variant, R58)",
            svg_path.display()
        );
    }
    let stem = svg_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .with_context(|| format!("invalid file name {}", svg_path.display()))?;
    let hover_svg = svg.replace(PARCHMENT_FILL, PARCHMENT_HOVER_FILL);
    Ok(vec![
        RenderedIcon {
            path: svg_path.with_file_name(format!("{stem}.png")),
            png: render_png(&svg).with_context(|| format!("rendering {}", svg_path.display()))?,
        },
        RenderedIcon {
            path: svg_path.with_file_name(format!("{stem}{HOVER_SUFFIX}.png")),
            png: render_png(&hover_svg)
                .with_context(|| format!("rendering hover of {}", svg_path.display()))?,
        },
    ])
}

/// Renders SVG source scaled to fit an [`ICON_SIZE`] square and encodes it as PNG.
///
/// # Errors
/// Returns an error if the SVG is invalid or PNG encoding fails.
fn render_png(svg: &str) -> Result<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &usvg::Options::default())?;
    let mut pixmap =
        tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE).context("allocating the icon pixmap")?;
    let size = tree.size();
    #[expect(
        clippy::cast_precision_loss,
        reason = "ICON_SIZE is 64, exactly representable as f32"
    )]
    let scale = ICON_SIZE as f32 / size.width().max(size.height());
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal icon using the parchment token.
    const SAMPLE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32"><rect width="32" height="32" fill="#E9DCB8"/></svg>"##;

    #[test]
    fn renders_a_64_pixel_png() -> Result<()> {
        let png = render_png(SAMPLE_SVG)?;
        let pixmap = tiny_skia::Pixmap::decode_png(&png)?;
        assert_eq!((pixmap.width(), pixmap.height()), (ICON_SIZE, ICON_SIZE));
        Ok(())
    }

    #[test]
    fn hover_variant_differs() -> Result<()> {
        let hover = SAMPLE_SVG.replace(PARCHMENT_FILL, PARCHMENT_HOVER_FILL);
        assert_ne!(render_png(SAMPLE_SVG)?, render_png(&hover)?);
        Ok(())
    }
}
