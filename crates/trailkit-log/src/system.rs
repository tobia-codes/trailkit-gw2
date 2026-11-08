//! Build and system facts for the `session_start` line (ROADMAP R92).

/// `release` or `debug`.
pub(crate) const BUILD_KIND: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "release"
};

/// Short Git commit of the build (`unknown` without Git), set by `build.rs`.
pub(crate) const COMMIT: &str = env!("TRAILKIT_COMMIT");

/// The Windows version as `major.minor.build`, e.g. `10.0.22631`.
///
/// Uses `RtlGetVersion`, which reports the real version (unlike
/// `GetVersionExW`, which lies to programs without a manifest). Returns
/// `unknown` if the call fails.
#[cfg(windows)]
pub(crate) fn windows_version() -> String {
    use windows::{
        Wdk::System::SystemServices::RtlGetVersion,
        Win32::System::SystemInformation::OSVERSIONINFOW,
    };

    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: u32::try_from(size_of::<OSVERSIONINFOW>()).unwrap_or(u32::MAX),
        ..Default::default()
    };
    // SAFETY: `info` is a live, writable `OSVERSIONINFOW` whose size field is
    // set as the function requires; it is only used during the call.
    let status = unsafe { RtlGetVersion(&raw mut info) };
    if status.is_ok() {
        format!(
            "{}.{}.{}",
            info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
        )
    } else {
        "unknown".to_owned()
    }
}

/// Without Windows (unit tests on Linux) there is no Windows version.
#[cfg(not(windows))]
pub(crate) fn windows_version() -> String {
    "none".to_owned()
}
