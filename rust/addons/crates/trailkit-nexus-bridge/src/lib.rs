//! trailkit-nexus-bridge — the Nexus API helpers that every trailkit addon
//! shares (ROADMAP R132, R136, Q496).
//!
//! Each addon calls the Nexus API only from its own `nexus_bridge` module;
//! what several addons need from Nexus lives here. `trailkit-ui` and the
//! addons' windows and slices draw with ImGui only and know nothing of Nexus,
//! so a port to another host replaces only the bridges. Windows only: on
//! other targets this crate is empty.

#[cfg(windows)]
pub mod fonts;
