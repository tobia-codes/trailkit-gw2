//! Nexus glue: addon export, load/unload and render callback (Windows only).
//!
//! Threading (nexus-addon-dev skill §2): the keybind callback only flips an
//! atomic; the render callback returns after one relaxed atomic load while the
//! window is closed. All registrations use `revert_on_unload`, so hot reload
//! cleans up automatically; the cached `NexusLink` pointer is dropped in
//! `unload`.

use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering},
};

use nexus::{
    AddonFlags, UpdateProvider,
    data_link::{NexusLink, get_nexus_link},
    gui::{RenderType, register_render, render},
    imgui::{Condition, Window},
    keybind::{keybind_handler, register_keybind_with_string},
    localization::{set_translation, translate},
    quick_access::add_quick_access,
    texture::load_texture_from_memory,
};

use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};

use crate::{
    i18n::{Language, Text},
    settings::{QuickAccessIcon, Settings},
};

nexus::export! {
    name: "trailkit-memo",
    // Random negative signature until Raidcore assigns an id (skill §1).
    signature: -2119926544,
    load,
    unload,
    flags: AddonFlags::None,
    // Update provider for the monorepo is still open (ROADMAP "Open").
    provider: UpdateProvider::None,
}

/// Nexus keybind identifier for opening/closing the main window.
const KEYBIND_TOGGLE: &str = "trailkit.memo.keybind.toggle";

/// Nexus localization key used to read the active Nexus language (skill §4).
const LANGUAGE_PROBE_KEY: &str = "trailkit.shared.localization.language";

/// Keybind string Nexus treats as "unbound" (memo plan M14: unbound by default).
const UNBOUND: &str = "(null)";

/// Nexus quick access shortcut identifier (R55).
const SHORTCUT_QUICK_ACCESS: &str = "trailkit.memo.shortcut.quick_access";

/// Nexus texture identifier of the quick access icon.
const TEXTURE_QUICK_ACCESS: &str = "trailkit.memo.texture.quick_access";

/// Nexus texture identifier of the quick access icon's hover variant.
const TEXTURE_QUICK_ACCESS_HOVER: &str = "trailkit.memo.texture.quick_access_hover";

/// Quick access icon PNG, rendered by `cargo xtask build-icons` (R57).
static QUICK_ACCESS_PNG: &[u8] = include_bytes!("../assets/icons/quick-access.png");

/// Hover variant of [`QUICK_ACCESS_PNG`] (R58).
static QUICK_ACCESS_HOVER_PNG: &[u8] = include_bytes!("../assets/icons/quick-access-hover.png");

/// User settings. Phase 0 uses the defaults; persistence follows in phase 1 (R60).
const SETTINGS: Settings = Settings::DEFAULT;

/// Shared Nexus link data (holds `is_gameplay`), cached at load and cleared at
/// unload. Null if Nexus did not provide it.
static NEXUS_LINK: AtomicPtr<NexusLink> = AtomicPtr::new(std::ptr::null_mut());

/// Whether the main window is visible. Written by keybind and render callbacks.
static VISIBLE: AtomicBool = AtomicBool::new(false);

/// Effective UI language, stored as the [`Language`] discriminant.
static LANGUAGE: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Main window title incl. a stable ImGui id (`###`), built once at load.
static WINDOW_TITLE: OnceLock<String> = OnceLock::new();

/// Called by Nexus when the addon is loaded: resolves the language, caches the
/// Nexus link and registers the keybind (with its localized name), the quick
/// access icon and the render callback.
fn load() {
    log::info!("loading");

    let language = Language::resolve(None, detect_nexus_language().as_deref());
    LANGUAGE.store(language as u8, Ordering::Relaxed);
    WINDOW_TITLE.get_or_init(|| format!("{}###trailkit-memo", Text::MainWindowTitle.get(language)));

    for language in Language::ALL {
        set_translation(
            KEYBIND_TOGGLE,
            language.code(),
            Text::KeybindToggle.get(language),
        );
    }

    let on_toggle = keybind_handler!(|_id, is_release| {
        if !is_release {
            VISIBLE.fetch_xor(true, Ordering::Relaxed);
        }
    });
    register_keybind_with_string(KEYBIND_TOGGLE, on_toggle, UNBOUND).revert_on_unload();

    NEXUS_LINK.store(get_nexus_link().cast_mut(), Ordering::Release);

    if SETTINGS.quick_access_icon == QuickAccessIcon::Shown {
        load_texture_from_memory(TEXTURE_QUICK_ACCESS, QUICK_ACCESS_PNG, None);
        load_texture_from_memory(TEXTURE_QUICK_ACCESS_HOVER, QUICK_ACCESS_HOVER_PNG, None);
        add_quick_access(
            SHORTCUT_QUICK_ACCESS,
            TEXTURE_QUICK_ACCESS,
            TEXTURE_QUICK_ACCESS_HOVER,
            KEYBIND_TOGGLE,
            Text::QuickAccessTooltip.get(language),
        )
        .revert_on_unload();
    }

    register_render(RenderType::Render, render!(|ui| render_main_window(ui))).revert_on_unload();
}

/// Called by Nexus before the addon is unloaded. Every registration was made
/// with `revert_on_unload`; only the cached Nexus link pointer is dropped
/// (nexus-addon-dev skill §1).
fn unload() {
    log::info!("unloading");
    NEXUS_LINK.store(std::ptr::null_mut(), Ordering::Release);
}

/// Whether the game is in gameplay (not in a loading screen, cutscene or
/// character select). Treated as `true` if the Nexus link is unavailable, so
/// the window never becomes unreachable.
fn is_gameplay() -> bool {
    let link = NEXUS_LINK.load(Ordering::Acquire);
    if link.is_null() {
        return true;
    }
    // SAFETY: `link` is non-null and points to the `DL_NEXUS_LINK` block, which
    // Nexus owns and keeps alive for its whole lifetime; the pointer is cleared
    // in `unload` before the DLL goes away. Nexus may update the flag from its
    // own thread, so it is read volatile (a single byte, never torn) without
    // creating a reference to the shared struct.
    unsafe { std::ptr::addr_of!((*link).is_gameplay).read_volatile() }
}

/// Reads the active Nexus language code via the localization trick from the
/// nexus-addon-dev skill §4. Returns `None` if Nexus answers with the bare key
/// (i.e. a language without a registered translation).
fn detect_nexus_language() -> Option<String> {
    for language in Language::ALL {
        set_translation(LANGUAGE_PROBE_KEY, language.code(), language.code());
    }
    translate(LANGUAGE_PROBE_KEY).filter(|code| code != LANGUAGE_PROBE_KEY)
}

/// Draws the main window. Returns immediately while the window is closed, and
/// skips drawing (keeping it open) outside gameplay unless the settings allow it.
fn render_main_window(ui: &nexus::imgui::Ui) {
    if !VISIBLE.load(Ordering::Relaxed) || !SETTINGS.allows_drawing(is_gameplay()) {
        return;
    }
    let language = Language::from_u8(LANGUAGE.load(Ordering::Relaxed));
    let title = WINDOW_TITLE
        .get()
        .map_or("###trailkit-memo", String::as_str);

    // Theme selection comes with the settings window (phase 1); until then the default theme.
    let _theme = StyleScope::push(ui, &ThemeKind::default().theme());
    let mut open = true;
    Window::new(title)
        .size([320.0, 120.0], Condition::FirstUseEver)
        .opened(&mut open)
        .build(ui, || ui.text(Text::Hello.get(language)));
    if !open {
        VISIBLE.store(false, Ordering::Relaxed);
    }
}
