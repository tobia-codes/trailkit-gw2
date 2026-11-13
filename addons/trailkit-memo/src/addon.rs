//! Nexus glue: addon export, load/unload and registrations (Windows only);
//! the render callbacks are in `render`.
//!
//! Threading (nexus-addon-dev skill §2): the keybind callback only flips an
//! atomic; the render callback returns after one relaxed atomic load while the
//! window is closed. Notes are loaded and saved by the storage worker (memo
//! plan M30); only `settings.json` is read once in `load`, because the quick
//! access icon depends on it. All registrations use `revert_on_unload`, so hot
//! reload cleans up automatically; the cached `NexusLink` pointer is dropped
//! and the storage worker is shut down (writing pending saves) in `unload`.
//! Logging (`trailkit-log`, ROADMAP R92) starts first in `load` and stops last
//! in `unload`, so the storage worker's last lines still reach the file.

use std::{
    path::{Path, PathBuf},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering},
        mpsc::{Receiver, TryRecvError},
    },
    time::Instant,
};

use jiff::Timestamp;

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{GetKeyState, VK_CONTROL},
        WindowsAndMessaging::WM_KEYDOWN,
    },
};

use nexus::{
    AddonFlags, UpdateProvider,
    data_link::{NexusLink, get_nexus_link},
    gui::{
        RenderType, deregister_close_on_escape, register_close_on_escape, register_render, render,
    },
    keybind::{keybind_handler, register_keybind_with_string},
    localization::{set_translation, translate},
    paths::get_addon_dir,
    quick_access::add_quick_access,
    texture::load_texture_from_memory,
    wnd_proc::register_wnd_proc,
};

use trailkit_log::ErrorChain;
use trailkit_storage::{
    AddonName, BACKUPS_DIRECTORY_NAME, DocumentStore, LoadOutcome, ReadOnlyReason,
    TRAILKIT_DIRECTORY_NAME, TrailkitDirectory,
};
use trailkit_ui::{font_registry::register_fonts, theme::ThemeKind, title_bar::TitleBarState};

use crate::main_window::{MemoWindow, NotesProblem};
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::{
    i18n::{Language, Text},
    note_tree::NotesDocument,
    settings::{QuickAccessIcon, Settings},
    storage_worker::{
        LoadedDocuments, SaveConnection, SavingAllowed, SettingsSummary, StorageSaver,
        StorageWorker, WorkerTiming, log_load_outcome,
    },
};

mod help;
mod logging;
mod render;
mod storage;

use self::help::MemoHelp;

use self::storage::start_storage;

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

/// Memo's short name: folder `trailkit/memo/`, logs `trailkit/logs/memo/`
/// (M17, ROADMAP R92).
const ADDON: AddonName = AddonName::new("memo");

/// User settings, read from `settings.json` once at load (defaults if absent).
static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// The storage worker; only touched in `load` and `unload`.
static STORAGE_WORKER: Mutex<Option<StorageWorker>> = Mutex::new(None);

/// The notes as seen by the render thread. The render thread only uses
/// `try_lock`, so it never waits; `load` and `unload` are the only other users.
static NOTES: Mutex<NotesState> = Mutex::new(NotesState::Unavailable);

/// Localization key registering memo's UI characters with the font atlas.
const GLYPHS_KEY: &str = "trailkit.memo.localization.glyphs";

/// Where the render thread's notes currently come from.
enum NotesState {
    /// The storage worker is still loading.
    Loading {
        /// Delivers what the worker loaded, once.
        loaded: Receiver<LoadedDocuments>,
        /// Queues saves on the worker.
        saver: StorageSaver,
        /// The addon folder.
        directory: PathBuf,
    },
    /// The notes are loaded and shown in the main window.
    Ready(Box<MemoWindow>),
    /// Storage could not be started; details are in the Nexus log.
    Unavailable,
}

/// Shared Nexus link data (holds `is_gameplay` and the fonts), cached at load
/// and cleared at unload. Null if Nexus did not provide it.
static NEXUS_LINK: AtomicPtr<NexusLink> = AtomicPtr::new(std::ptr::null_mut());

/// Whether the main window is visible. Written by keybind and render callbacks.
static VISIBLE: AtomicBool = AtomicBool::new(false);

/// Whether the editor holds unsaved changes (M54). Written by the render
/// callback, so a keybind or quick access click that hid the window can be
/// turned into the unsaved changes dialog instead.
static UNSAVED_CHANGES: AtomicBool = AtomicBool::new(false);

/// Set when the window was hidden from outside while changes were unsaved;
/// the render callback shows it again and asks (M54).
static CLOSE_REQUESTED: AtomicBool = AtomicBool::new(false);

/// Whether Escape should clear the search instead of closing the window (the
/// focused search field holds text, M124). Written by the render callback.
static SEARCH_TAKES_ESCAPE: AtomicBool = AtomicBool::new(false);

/// Set when Nexus hid the window while the search took Escape; the render
/// callback shows it again and clears the search (M124).
static SEARCH_ESCAPED: AtomicBool = AtomicBool::new(false);

/// Whether Ctrl+F belongs to the window: it has the focus and no editor is open
/// (M174). Written by the render callback, false while it does not draw.
static SEARCH_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Ctrl+F was caught; the render callback shows the search (M174).
static SEARCH_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether the Delete key belongs to the note list (M175). Written by the render
/// callback, false while it does not draw.
static DELETE_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when the Delete key was caught; the render callback deletes (M175).
static DELETE_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Windows virtual-key code of `F`.
const VIRTUAL_KEY_F: usize = 0x46;
/// Windows virtual-key code of Delete.
const VIRTUAL_KEY_DELETE: usize = 0x2E;
/// What a `WndProc` callback returns for a message it consumed.
const MESSAGE_CONSUMED: u32 = 0;
/// What a `WndProc` callback returns for a message others should see.
const MESSAGE_PASSED: u32 = 1;

/// Effective UI language, stored as the [`Language`] discriminant.
static LANGUAGE: AtomicU8 = AtomicU8::new(Language::English as u8);

/// Memo's help window (R100); created in `load`, dropped in `unload`. The
/// render thread only uses `try_lock`.
static HELP: Mutex<Option<MemoHelp>> = Mutex::new(None);

/// Whether the help window is open, so the render callback also draws while
/// only the help window is shown. Written by the render callback, and set to
/// `false` by Nexus on Escape (close on Escape), which the render callback
/// then applies to the window.
static HELP_OPEN: AtomicBool = AtomicBool::new(false);

/// Collapsed state of the main window's title bar (R71); render thread only.
static TITLE_BAR: Mutex<TitleBarState> = Mutex::new(TitleBarState::NEW);

/// Main window title incl. a stable ImGui id (`###`), built once at load.
static WINDOW_TITLE: OnceLock<String> = OnceLock::new();

/// Called by Nexus when the addon is loaded: resolves the language, caches the
/// Nexus link and registers the keybind (with its localized name), the quick
/// access icon and the render callback.
fn load() {
    let language = Language::resolve(None, detect_nexus_language().as_deref());
    LANGUAGE.store(language as u8, Ordering::Relaxed);
    let trailkit_directory = match get_addon_dir(TRAILKIT_DIRECTORY_NAME) {
        Ok(path) => {
            let directory = TrailkitDirectory::new(path);
            logging::start_logging(directory.clone(), language);
            Some(directory)
        }
        Err(error) => {
            // No folder, no log file: only Nexus can tell.
            logging::forward_to_nexus(
                log::Level::Error,
                &format!("folder_unavailable error=\"{error}\""),
            );
            None
        }
    };
    let window_title = WINDOW_TITLE
        .get_or_init(|| format!("{}###trailkit-memo", Text::MainWindowTitle.get(language)));
    // Escape closes the window (`overlay-escape`). Nexus handles the key
    // before ImGui sees it and clears the flag; with unsaved changes the
    // render callback reopens the window and asks instead (M54).
    // SAFETY: `VISIBLE` is a static, so the pointer stays valid for the whole
    // time Nexus may use it; the registration is removed in `unload` before
    // the DLL goes away. `AtomicBool` has the same in-memory representation
    // as `bool`, and Nexus only stores `false` into it (a single-byte write).
    register_close_on_escape(window_title, unsafe { &mut *VISIBLE.as_ptr() });
    // SAFETY: as above, with the static `HELP_OPEN`.
    register_close_on_escape(help::WINDOW_ID, unsafe { &mut *HELP_OPEN.as_ptr() });

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
    register_wnd_proc(catch_window_keys).revert_on_unload();
    // UI strings plus the help pages, which the help window draws.
    register_fonts(
        GLYPHS_KEY,
        &(crate::all_texts() + include_str!("../help/en.md") + include_str!("../help/de.md")),
        &ThemeKind::default().theme().typography,
    );

    NEXUS_LINK.store(get_nexus_link().cast_mut(), Ordering::Release);

    if let Ok(mut help) = HELP.lock() {
        *help = Some(MemoHelp::new(
            trailkit_directory.as_ref().map(TrailkitDirectory::logs),
        ));
    }
    let settings = start_storage(trailkit_directory);

    if settings.quick_access_icon == QuickAccessIcon::Shown {
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

    register_render(
        RenderType::Render,
        render!(|ui| render::render_main_window(ui)),
    )
    .revert_on_unload();
    register_render(
        RenderType::OptionsRender,
        render!(|ui| render::render_options_tab(ui)),
    )
    .revert_on_unload();
}

/// Called by Nexus before the addon is unloaded. Every registration was made
/// with `revert_on_unload`; the cached Nexus link pointer is dropped
/// (nexus-addon-dev skill §1) and the storage worker is shut down, which
/// writes all pending saves before returning.
fn unload() {
    if let Some(window_title) = WINDOW_TITLE.get() {
        deregister_close_on_escape(window_title);
    }
    deregister_close_on_escape(help::WINDOW_ID);
    NEXUS_LINK.store(std::ptr::null_mut(), Ordering::Release);
    // Unsaved edits become a draft; the worker writes it while shutting down (M54).
    if let Ok(mut notes) = NOTES.lock()
        && let NotesState::Ready(window) = &mut *notes
    {
        window.keep_draft_for_unload();
    }
    let worker = STORAGE_WORKER
        .lock()
        .map_or(None, |mut worker| worker.take());
    drop(worker);
    if let Ok(mut notes) = NOTES.lock() {
        *notes = NotesState::Unavailable;
    }
    if let Ok(mut help) = HELP.lock() {
        *help = None;
    }
    HELP_OPEN.store(false, Ordering::Relaxed);
    // Last, so every line above still reaches the file; writes `session_end`.
    logging::stop_logging();
}

/// The active settings (defaults until `load` has read them).
fn settings() -> Settings {
    SETTINGS.get().copied().unwrap_or(Settings::DEFAULT)
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

/// Catches memo's keys that would not reach ImGui on the input thread (M174,
/// M175), while they belong to the window; otherwise the game gets them. Only
/// flips atomics (nexus-addon-dev skill §2).
extern "C-unwind" fn catch_window_keys(
    _window: HWND,
    message: u32,
    key: WPARAM,
    _details: LPARAM,
) -> u32 {
    if message != WM_KEYDOWN {
        return MESSAGE_PASSED;
    }
    if key.0 == VIRTUAL_KEY_DELETE && DELETE_KEY_ARMED.load(Ordering::Relaxed) {
        DELETE_KEY_PRESSED.store(true, Ordering::Relaxed);
        return MESSAGE_CONSUMED;
    }
    if key.0 == VIRTUAL_KEY_F && SEARCH_KEY_ARMED.load(Ordering::Relaxed) {
        // SAFETY: `GetKeyState` only reads the calling thread's key state, and this
        // thread is processing the window's messages. The high bit means "down".
        let control_is_down = unsafe { GetKeyState(i32::from(VK_CONTROL.0)) } < 0;
        if control_is_down {
            SEARCH_KEY_PRESSED.store(true, Ordering::Relaxed);
            return MESSAGE_CONSUMED;
        }
    }
    MESSAGE_PASSED
}
