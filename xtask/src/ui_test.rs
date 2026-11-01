//! `cargo xtask ui-test [--update]`: builds the Windows test binaries of the
//! UI crates with cargo-xwin and runs them directly (memo plan M65).
//!
//! The ImGui harness (`trailkit-ui-testing`) only compiles for Windows. On WSL
//! the test executables run through Windows interop; cargo itself would try
//! to start them with wine. Each binary gets `TRAILKIT_SNAPSHOT_ROOT` (its
//! crate directory, converted to a Windows path through `WSLENV`), so the
//! snapshot references are found. `--update` sets
//! `TRAILKIT_UPDATE_SNAPSHOTS=1` to write new reference images.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

use crate::workspace;

/// Target triple of the addons.
const TARGET: &str = "x86_64-pc-windows-msvc";

/// Crates whose tests use the ImGui harness.
const PACKAGES: [&str; 3] = ["trailkit-ui", "trailkit-ui-testing", "trailkit-memo"];

/// Whether reference images are compared or rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Compare with the references (fail on differences).
    Compare,
    /// Write the rendered images as new references.
    Update,
}

/// A built test binary and the crate it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TestBinary {
    /// The `.exe` path.
    executable: PathBuf,
    /// The crate directory (where `tests/snapshots/` lives).
    crate_directory: PathBuf,
}

/// Builds and runs the UI test binaries.
///
/// # Errors
/// Returns an error if the build fails, a binary cannot be started, or any
/// test fails.
pub fn run(mode: Mode) -> Result<()> {
    let binaries = build()?;
    let mut failed = Vec::new();
    for binary in &binaries {
        println!("running {}", binary.executable.display());
        let mut command = Command::new(&binary.executable);
        command
            .current_dir(&binary.crate_directory)
            .env("TRAILKIT_SNAPSHOT_ROOT", &binary.crate_directory)
            .env(
                "WSLENV",
                "TRAILKIT_SNAPSHOT_ROOT/p:TRAILKIT_UPDATE_SNAPSHOTS",
            );
        if mode == Mode::Update {
            command.env("TRAILKIT_UPDATE_SNAPSHOTS", "1");
        }
        let status = command
            .status()
            .with_context(|| format!("failed to start {}", binary.executable.display()))?;
        if !status.success() {
            failed.push(binary.executable.display().to_string());
        }
    }
    if !failed.is_empty() {
        bail!("UI tests failed in: {}", failed.join(", "));
    }
    Ok(())
}

/// Runs `cargo xwin test --no-run` and collects the test executables.
///
/// # Errors
/// Returns an error if cargo cannot be started or the build fails.
fn build() -> Result<Vec<TestBinary>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command.current_dir(workspace::root()).args([
        "xwin",
        "test",
        "--no-run",
        "--message-format=json",
        "--target",
        TARGET,
    ]);
    for package in PACKAGES {
        command.args(["--package", package]);
    }
    let output = command
        .output()
        .context("failed to run `cargo xwin` (install with `cargo install --locked cargo-xwin`)")?;
    if !output.status.success() {
        bail!(
            "building the UI tests failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(parse_test_binaries(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

/// Extracts test executables and their crate directories from cargo's JSON
/// messages (one JSON object per line).
fn parse_test_binaries(messages: &str) -> Vec<TestBinary> {
    messages
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|message| message["reason"] == "compiler-artifact")
        .filter_map(|message| {
            let executable = message["executable"].as_str()?;
            let manifest = message["manifest_path"].as_str()?;
            Some(TestBinary {
                executable: PathBuf::from(executable),
                crate_directory: Path::new(manifest).parent()?.to_path_buf(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_test_executables_with_their_crates() {
        let messages = concat!(
            r#"{"reason":"compiler-artifact","manifest_path":"/repo/crates/ui/Cargo.toml","executable":"/repo/target/ui-1.exe"}"#,
            "\n",
            r#"{"reason":"compiler-artifact","manifest_path":"/repo/crates/ui/Cargo.toml","executable":null}"#,
            "\n",
            r#"{"reason":"build-finished","success":true}"#,
        );
        assert_eq!(
            parse_test_binaries(messages),
            vec![TestBinary {
                executable: PathBuf::from("/repo/target/ui-1.exe"),
                crate_directory: PathBuf::from("/repo/crates/ui"),
            }]
        );
    }
}
