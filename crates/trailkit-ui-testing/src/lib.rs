//! trailkit-ui-testing — test tools for trailkit ImGui code (memo plan M65).
//!
//! A dev-dependency only; it never ends up in an addon DLL.
//!
//! - [`raster`]: a CPU rasterizer for ImGui draw data (any platform);
//! - [`snapshot`]: comparison with reference PNGs (any platform);
//! - `harness` (Windows only): a headless ImGui context with the real fonts,
//!   simulated input and rendering to images.
//!
//! Windows test binaries are run from WSL with `cargo xtask ui-test`
//! (`--update` writes new reference images).

pub mod raster;
pub mod snapshot;

#[cfg(windows)]
pub mod harness;
