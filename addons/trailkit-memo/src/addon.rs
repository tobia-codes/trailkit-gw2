//! Nexus glue: addon export, load/unload and render callback (Windows only).
//!
//! Threading (nexus-addon-dev skill §2): the keybind callback only flips an
//! atomic; the render callback returns after one relaxed atomic load while the
//! window is closed. All registrations use `revert_on_unload`, so hot reload
//! cleans up automatically.

use std::sync::{
    OnceLock,
    atomic::{AtomicBool, AtomicU8, Ordering},
};

use nexus::{
    AddonFlags, UpdateProvider,
    gui::{RenderType, register_render, render},
    imgui::{Condition, Window},
    keybind::{keybind_handler, register_keybind_with_string},
    localization::{set_translation, translate},
};

use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};

use crate::i18n::{Language, Text};

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

/// Whether the main window is visible. Written by keybind and render callbacks.
static VISIBLE: AtomicBool = AtomicBool::new(false);

/// Effective UI language, stored as the [`Language`] discriminant.
static LANGUAGE: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Main window title incl. a stable ImGui id (`###`), built once at load.
static WINDOW_TITLE: OnceLock<String> = OnceLock::new();

/// Called by Nexus when the addon is loaded: resolves the language and
/// registers the keybind, its localized name and the render callback.
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

    register_render(RenderType::Render, render!(|ui| render_main_window(ui))).revert_on_unload();
}

/// Called by Nexus before the addon is unloaded. Every registration was made
/// with `revert_on_unload`, so nothing else needs to be torn down yet.
fn unload() {
    log::info!("unloading");
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

/// Draws the main window. Returns immediately while the window is hidden.
fn render_main_window(ui: &nexus::imgui::Ui) {
    if !VISIBLE.load(Ordering::Relaxed) {
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
