//! Snapshot comparison: rendered images against reference PNGs (memo plan
//! M65).
//!
//! References live in `<crate>/tests/snapshots/<name>.png`. A test fails when
//! more than [`MAXIMUM_DIFFERENT_SHARE`] of the pixels differ by more than
//! [`CHANNEL_TOLERANCE`] in any channel; it then writes the rendered image and
//! a diff image (differences in red) to `tests/snapshots/failures/`.
//! Running with `TRAILKIT_UPDATE_SNAPSHOTS=1` writes the references instead.
//! `TRAILKIT_SNAPSHOT_ROOT` overrides the crate directory (needed when a
//! Windows test binary runs from WSL and only knows Linux paths at compile
//! time; `cargo xtask ui-test` sets it).

use std::{
    fs,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::raster::Image;

/// Largest per-channel difference that still counts as equal.
pub const CHANNEL_TOLERANCE: u8 = 3;

/// Largest share of different pixels that still passes (0.05 %).
pub const MAXIMUM_DIFFERENT_SHARE: f64 = 0.0005;

/// Environment variable that switches to writing references.
const UPDATE_VARIABLE: &str = "TRAILKIT_UPDATE_SNAPSHOTS";

/// Environment variable that overrides the crate directory.
const ROOT_VARIABLE: &str = "TRAILKIT_SNAPSHOT_ROOT";

/// The outcome of comparing an image with its reference.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotOutcome {
    /// The image matches the reference.
    Matches,
    /// The reference was (re)written because updating was requested.
    Updated,
    /// The image differs; the paths of the written actual and diff images.
    Differs {
        /// Number of pixels outside the tolerance.
        different_pixels: usize,
        /// Where the rendered image was written.
        actual: PathBuf,
        /// Where the diff image was written.
        diff: PathBuf,
    },
    /// There is no reference yet; the rendered image was written here.
    MissingReference(PathBuf),
}

/// Compares `image` with the reference `name` of the crate at
/// `crate_directory` (pass `env!("CARGO_MANIFEST_DIR")`) and panics with a
/// helpful message unless it matches (or updating was requested).
///
/// # Panics
///
/// Panics if the image differs, the reference is missing or files cannot be
/// read or written.
pub fn assert_snapshot(crate_directory: &str, name: &str, image: &Image) {
    let root = std::env::var_os(ROOT_VARIABLE)
        .map_or_else(|| PathBuf::from(crate_directory), PathBuf::from);
    let update = std::env::var_os(UPDATE_VARIABLE).is_some_and(|value| value == "1");
    match compare_snapshot(&root, name, image, update) {
        Ok(SnapshotOutcome::Matches | SnapshotOutcome::Updated) => {}
        Ok(SnapshotOutcome::Differs {
            different_pixels,
            actual,
            diff,
        }) => panic!(
            "snapshot `{name}` differs in {different_pixels} pixels; see {} and {} \
             (run `cargo xtask ui-test --update` if the change is intended)",
            actual.display(),
            diff.display()
        ),
        Ok(SnapshotOutcome::MissingReference(path)) => panic!(
            "snapshot `{name}` has no reference; the rendered image is at {} \
             (run `cargo xtask ui-test --update` to accept it)",
            path.display()
        ),
        Err(error) => panic!("snapshot `{name}`: {error}"),
    }
}

/// Compares `image` with `<root>/tests/snapshots/<name>.png`, or writes it
/// there when `update` is set.
///
/// # Errors
///
/// Returns an error message if a file cannot be read, decoded or written.
pub fn compare_snapshot(
    root: &Path,
    name: &str,
    image: &Image,
    update: bool,
) -> Result<SnapshotOutcome, String> {
    let directory = root.join("tests").join("snapshots");
    let reference_path = directory.join(format!("{name}.png"));
    if update {
        write_png(&reference_path, image)?;
        return Ok(SnapshotOutcome::Updated);
    }
    let failures = directory.join("failures");
    if !reference_path.exists() {
        let path = failures.join(format!("{name}.new.png"));
        write_png(&path, image)?;
        return Ok(SnapshotOutcome::MissingReference(path));
    }
    let reference = read_png(&reference_path)?;
    let (different_pixels, diff) = difference(&reference, image);
    let total = image.pixels.len().max(1);
    #[expect(clippy::cast_precision_loss, reason = "pixel counts are small")]
    let share = different_pixels as f64 / total as f64;
    let same_size = reference.width == image.width && reference.height == image.height;
    if same_size && share <= MAXIMUM_DIFFERENT_SHARE {
        return Ok(SnapshotOutcome::Matches);
    }
    let actual = failures.join(format!("{name}.actual.png"));
    let diff_path = failures.join(format!("{name}.diff.png"));
    write_png(&actual, image)?;
    write_png(&diff_path, &diff)?;
    Ok(SnapshotOutcome::Differs {
        different_pixels: if same_size { different_pixels } else { total },
        actual,
        diff: diff_path,
    })
}

/// Counts pixels outside the tolerance and builds a diff image: the actual
/// image dimmed, with differing pixels in red.
fn difference(reference: &Image, actual: &Image) -> (usize, Image) {
    let mut diff = Image::new(actual.width, actual.height, [0, 0, 0, 255]);
    let mut count = 0;
    for (index, pixel) in actual.pixels.iter().enumerate() {
        let expected = reference.pixels.get(index).copied();
        let differs = expected.is_none_or(|expected| {
            expected
                .iter()
                .zip(pixel)
                .any(|(left, right)| left.abs_diff(*right) > CHANNEL_TOLERANCE)
        });
        if let Some(out) = diff.pixels.get_mut(index) {
            *out = if differs {
                [255, 0, 0, 255]
            } else {
                [pixel[0] / 4, pixel[1] / 4, pixel[2] / 4, 255]
            };
        }
        count += usize::from(differs);
    }
    (count, diff)
}

/// Writes `image` as an RGBA PNG, creating the directory.
fn write_png(path: &Path, image: &Image) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))?;
    }
    let file = fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let bytes: Vec<u8> = image.pixels.iter().flatten().copied().collect();
    writer
        .write_image_data(&bytes)
        .map_err(|error| format!("{}: {error}", path.display()))
}

/// Reads an 8-bit RGBA PNG.
fn read_png(path: &Path) -> Result<Image, String> {
    let file = fs::File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder
        .read_info()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| format!("{}: image too large", path.display()))?;
    let mut buffer = vec![0; size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if info.color_type != png::ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
        return Err(format!("{}: not an 8-bit RGBA PNG", path.display()));
    }
    let pixels = buffer[..info.buffer_size()].as_chunks::<4>().0.to_vec();
    Ok(Image {
        width: info.width,
        height: info.height,
        pixels,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_then_compare_matches_and_detects_changes() -> Result<(), String> {
        let directory =
            std::env::temp_dir().join(format!("trailkit-snapshot-{}", std::process::id()));
        let image = Image::new(8, 8, [10, 20, 30, 255]);
        assert_eq!(
            compare_snapshot(&directory, "square", &image, true)?,
            SnapshotOutcome::Updated
        );
        assert_eq!(
            compare_snapshot(&directory, "square", &image, false)?,
            SnapshotOutcome::Matches
        );
        let mut changed = image.clone();
        changed.pixels[0] = [200, 20, 30, 255];
        // One of 64 pixels is far above the 0.05 % allowance.
        assert!(matches!(
            compare_snapshot(&directory, "square", &changed, false)?,
            SnapshotOutcome::Differs {
                different_pixels: 1,
                ..
            }
        ));
        let mut slightly = image.clone();
        slightly.pixels[0] = [12, 20, 30, 255];
        assert_eq!(
            compare_snapshot(&directory, "square", &slightly, false)?,
            SnapshotOutcome::Matches,
            "within the channel tolerance"
        );
        assert!(matches!(
            compare_snapshot(&directory, "missing", &image, false)?,
            SnapshotOutcome::MissingReference(_)
        ));
        let _ = fs::remove_dir_all(directory);
        Ok(())
    }
}
