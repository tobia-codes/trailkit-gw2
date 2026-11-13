//! Nexus glue: addon export, load/unload and render callback (Windows only).
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
    imgui::{Condition, Window},
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
use trailkit_ui::{
    font_registry::{follow_body_size, font_size_of, register_fonts},
    style_scope::StyleScope,
    text::{TextStyle, draw_text, draw_text_wrapped},
    theme::{Theme, ThemeKind, Typography},
    title_bar::{TitleBarState, TitleBarToggle, draw_window},
};

use crate::main_window::{MemoWindow, NotesProblem, minimum_window_width, title_bar_texts};
use crate::options_tab::{VERSION_LINE, draw_options_tab};
use trailkit_memo_shared::i18n::LocalizedText;
use trailkit_memo_shared::{
    i18n::{Language, Text},
    note_tree::NotesDocument,
    settings::{QuickAccessIcon, Settings, TextScale},
    storage_worker::{
        LoadedDocuments, SaveConnection, SavingAllowed, SettingsSummary, StorageSaver,
        StorageWorker, WorkerTiming, log_load_outcome,
    },
};

mod help;
mod logging;
mod storage;

use self::help::MemoHelp;

use self::storage::{receive_loaded_notes, restart_storage_worker_if_asked, start_storage};

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

    register_render(RenderType::Render, render!(|ui| render_main_window(ui))).revert_on_unload();
    register_render(
        RenderType::OptionsRender,
        render!(|ui| render_options_tab(ui)),
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

/// Draws the main window and the help window. Returns immediately while
/// both are closed, and skips drawing (keeping them open) outside gameplay
/// unless the settings allow it.
fn render_main_window(ui: &nexus::imgui::Ui) {
    // Stay false on every path that does not draw the window, so the game keeps
    // these keys while memo is hidden.
    SEARCH_KEY_ARMED.store(false, Ordering::Relaxed);
    DELETE_KEY_ARMED.store(false, Ordering::Relaxed);
    reopen_hidden_window_if_needed();
    let main_visible = VISIBLE.load(Ordering::Relaxed);
    if !(main_visible || HELP_OPEN.load(Ordering::Relaxed))
        || !settings().allows_drawing(is_gameplay())
    {
        return;
    }
    // Never wait on the render thread; if `load`/`unload` holds the lock, skip a frame.
    let Ok(mut notes) = NOTES.try_lock() else {
        return;
    };
    receive_loaded_notes(&mut notes);
    let text_scale = match &*notes {
        NotesState::Ready(window) => window.text_scale(),
        NotesState::Loading { .. } | NotesState::Unavailable => settings().text_scale,
    };
    // Theme selection comes with the settings window (step 8); until then the default theme.
    let theme = ThemeKind::default().theme();
    follow_nexus_font_size(text_scale, &theme.typography);
    let language = Language::from_u8(LANGUAGE.load(Ordering::Relaxed));
    let _theme = StyleScope::push(ui, &theme);
    let mut help = HELP.try_lock().ok();
    let mut help = help.as_mut().and_then(|help| help.as_mut());
    if let Some(help) = help.as_deref_mut()
        && !HELP_OPEN.load(Ordering::Relaxed)
    {
        help.close();
    }
    if main_visible {
        draw_main_window(ui, &theme, language, &mut notes, help.as_deref_mut());
    }
    if let Some(help) = help {
        help.draw(ui, &theme, language);
        HELP_OPEN.store(help.is_open(), Ordering::Relaxed);
    }
}

/// Nexus' options render callback: memo's tab in the Nexus options (R53,
/// Q307–Q309). "Open help" opens the help window or brings it to the front
/// (Q296); the main render callback draws it from the next frame on.
fn render_options_tab(ui: &nexus::imgui::Ui) {
    let theme = ThemeKind::default().theme();
    follow_nexus_font_size(settings().text_scale, &theme.typography);
    let language = Language::from_u8(LANGUAGE.load(Ordering::Relaxed));
    let _theme = StyleScope::push(ui, &theme);
    if !draw_options_tab(ui, &theme, language, VERSION_LINE) {
        return;
    }
    // Never wait on the render thread; a click during load/unload is lost.
    if let Ok(mut help) = HELP.try_lock()
        && let Some(help) = help.as_mut()
    {
        help.open();
        HELP_OPEN.store(true, Ordering::Relaxed);
    }
}

/// Draws the main window with its title bar toggles: memo's window menu
/// once the notes are loaded and the help `?` (Q171) while `help` exists.
fn draw_main_window(
    ui: &nexus::imgui::Ui,
    theme: &Theme,
    language: Language,
    notes: &mut NotesState,
    help: Option<&mut MemoHelp>,
) {
    let title = WINDOW_TITLE
        .get()
        .map_or("###trailkit-memo", String::as_str);
    let mut open = true;
    let mut title_bar = TITLE_BAR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // The memo toggle (window menu; R78) exists once notes are loaded.
    let menu_toggle = match &*notes {
        NotesState::Ready(window) => Some(window.title_bar_toggles(language)[0]),
        NotesState::Loading { .. } | NotesState::Unavailable => None,
    };
    let help_toggle = help.as_ref().map(|help| help.title_bar_toggle(language));
    let both;
    let toggles: &[TitleBarToggle<'_>] = match (&menu_toggle, &help_toggle) {
        (Some(menu), Some(help)) => {
            both = [*menu, *help];
            &both
        }
        (Some(only), None) | (None, Some(only)) => std::slice::from_ref(only),
        (None, None) => &[],
    };
    let help_index = usize::from(menu_toggle.is_some());
    let clicked_toggle = draw_window(
        ui,
        theme,
        Window::new(title)
            .size([640.0, 420.0], Condition::FirstUseEver)
            .size_constraints([minimum_window_width(ui), 0.0], [f32::MAX, f32::MAX]),
        title_bar_texts(language),
        toggles,
        &mut title_bar,
        &mut open,
        || match &mut *notes {
            NotesState::Loading { .. } => {
                draw_text(ui, theme, TextStyle::Body, Text::Loading.get(language));
            }
            NotesState::Unavailable => draw_text_wrapped(
                ui,
                theme,
                TextStyle::Body,
                Text::NotesInaccessible.get(language),
            ),
            NotesState::Ready(window) => {
                if SEARCH_ESCAPED.swap(false, Ordering::Relaxed) {
                    window.clear_search_by_escape();
                }
                if SEARCH_KEY_PRESSED.swap(false, Ordering::Relaxed) {
                    window.toggle_search_by_key();
                }
                if DELETE_KEY_PRESSED.swap(false, Ordering::Relaxed) {
                    window.delete_selected_by_key(language);
                }
                window.draw(ui, language, theme);
            }
        },
    );
    drop(title_bar);
    if let Some(index) = clicked_toggle {
        apply_title_bar_click(index, help_index, notes, help);
    }
    if let NotesState::Ready(window) = &mut *notes {
        window.draw_dialog(ui, language, theme);
        restart_storage_worker_if_asked(window);
    }
    apply_close_request(notes, open);
    if let NotesState::Ready(window) = &*notes {
        let visible = VISIBLE.load(Ordering::Relaxed);
        SEARCH_KEY_ARMED.store(window.takes_search_key() && visible, Ordering::Relaxed);
        DELETE_KEY_ARMED.store(window.takes_delete_key() && visible, Ordering::Relaxed);
    }
}

/// Applies a click on the title bar toggle at `index`: the help `?` at
/// `help_index` opens or closes the help window, the others belong to the
/// loaded notes window.
fn apply_title_bar_click(
    index: usize,
    help_index: usize,
    notes: &mut NotesState,
    help: Option<&mut MemoHelp>,
) {
    match help {
        Some(help) if index == help_index => help.toggle(),
        _ => {
            if let NotesState::Ready(window) = notes {
                window.apply_title_bar_toggle(index);
            }
        }
    }
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

/// Shows the hidden window again when Nexus hid it too early: Escape in the
/// focused, non-empty search field only clears the search (M124), and with
/// unsaved changes a keybind or quick access click asks first (M54).
fn reopen_hidden_window_if_needed() {
    if !VISIBLE.load(Ordering::Relaxed) && SEARCH_TAKES_ESCAPE.swap(false, Ordering::Relaxed) {
        VISIBLE.store(true, Ordering::Relaxed);
        SEARCH_ESCAPED.store(true, Ordering::Relaxed);
    }
    if !VISIBLE.load(Ordering::Relaxed) && UNSAVED_CHANGES.load(Ordering::Relaxed) {
        VISIBLE.store(true, Ordering::Relaxed);
        CLOSE_REQUESTED.store(true, Ordering::Relaxed);
    }
}

/// Handles a close request of this frame; `open` is false after a click on the
/// title bar's close button, which closes the window menu along with it.
fn apply_close_request(notes: &mut NotesState, open: bool) {
    if !open && let NotesState::Ready(window) = notes {
        window.close_window_menu();
    }
    let close_requested = !open || CLOSE_REQUESTED.swap(false, Ordering::Relaxed);
    let may_close = match notes {
        NotesState::Ready(window) => {
            let may_close =
                window.take_close_approval() || (close_requested && window.request_close());
            UNSAVED_CHANGES.store(window.has_unsaved_changes(), Ordering::Relaxed);
            SEARCH_TAKES_ESCAPE.store(window.search_takes_escape(), Ordering::Relaxed);
            may_close
        }
        _ => close_requested,
    };
    if may_close {
        UNSAVED_CHANGES.store(false, Ordering::Relaxed);
        SEARCH_TAKES_ESCAPE.store(false, Ordering::Relaxed);
        VISIBLE.store(false, Ordering::Relaxed);
    }
}

/// Resizes the shared fonts when Nexus' `FontUI` size or the memo text
/// scale changes (M35, M79): body size = `FontUI` size × `text_scale`, the
/// other sizes are `typography`'s factors of it (R96).
fn follow_nexus_font_size(text_scale: TextScale, typography: &Typography) {
    let link = NEXUS_LINK.load(Ordering::Acquire);
    if link.is_null() {
        return;
    }
    // SAFETY: `link` is non-null and points to Nexus' link data, which lives as
    // long as Nexus (cleared in `unload`). `font_ui` is either null or a font
    // of the current atlas; `font_size_of` only reads its size.
    let size = unsafe { font_size_of(std::ptr::addr_of!((*link).font_ui).read_volatile()) };
    if let Some(size) = size {
        follow_body_size(size * text_scale.factor(), typography);
    }
}
