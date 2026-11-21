//! The Nexus bridge (ROADMAP R132, R136): the only module of memo that calls
//! the Nexus API. Addon export, load/unload and registrations (Windows only);
//! the render callbacks are in `render`. Everything else in memo (main window,
//! slices) and `trailkit-ui` draws with ImGui only and knows nothing of Nexus.
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
//! Every callback runs inside `guard::guard` (crash protection, ROADMAP R104).

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
        Input::KeyboardAndMouse::{GetKeyState, VIRTUAL_KEY},
        WindowsAndMessaging::{WM_MBUTTONDOWN, WM_RBUTTONDOWN},
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
use trailkit_nexus_bridge::fonts::register_fonts;
use trailkit_storage::{
    AddonName, BACKUPS_DIRECTORY_NAME, DocumentStore, LoadOutcome, ReadOnlyReason,
    TRAILKIT_DIRECTORY_NAME, TrailkitDirectory,
};
use trailkit_ui::{
    layout::title_bar::{TitleBarState, record_mouse_button_press},
    theme::ThemeKind,
};

use crate::help_document::help_document;
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

mod double_shift;
#[cfg(feature = "fault-injection")]
mod fault_injection;
mod guard;
mod help;
mod key_routing;
mod logging;
mod render;
mod storage;

use self::double_shift::DoubleShift;
use self::guard::{Callback, RunState, guard, run_state};
use self::help::MemoHelp;
use self::key_routing::{ArmedKeys, KeyMessage, KeySignal, route_key};

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

/// Whether Escape belongs to the window instead of closing it (an open
/// popup or window menu, or the search, M124; see `MemoWindow::takes_escape`).
/// Written by the render callback.
static WINDOW_TAKES_ESCAPE: AtomicBool = AtomicBool::new(false);

/// Set when Nexus hid the window while it took Escape; the render callback
/// shows it again and lets the window handle the key.
static ESCAPE_CAUGHT: AtomicBool = AtomicBool::new(false);

/// Whether Shift pressed twice belongs to the window: it has the focus and no
/// editor is open (M199). Written by the render callback, false while it does
/// not draw.
static SEARCH_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Shift was pressed twice; the render callback toggles the list
/// search (M199).
static SEARCH_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Follows Shift taps for [`SEARCH_KEY_PRESSED`]; only the window message
/// callback locks it.
static DOUBLE_SHIFT: Mutex<DoubleShift> = Mutex::new(DoubleShift::new());

/// Whether Ctrl+F belongs to the window: it has the focus and a note is open
/// (M174, M198). Written by the render callback, false while it does not draw.
static FIND_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Ctrl+F was caught; the render callback opens find in the note.
static FIND_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether F3 belongs to the window: it has the focus and the find bar is
/// open (M198). Written by the render callback.
static FIND_STEP_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when F3 was caught: [`FIND_STEP_NEXT`] or [`FIND_STEP_PREVIOUS`]
/// (Shift+F3); the render callback steps through the hits.
static FIND_STEP: AtomicU8 = AtomicU8::new(FIND_STEP_NONE);
/// [`FIND_STEP`]: no step waiting.
const FIND_STEP_NONE: u8 = 0;
/// [`FIND_STEP`]: go to the next hit (F3).
const FIND_STEP_NEXT: u8 = 1;
/// [`FIND_STEP`]: go to the previous hit (Shift+F3).
const FIND_STEP_PREVIOUS: u8 = 2;

/// Whether the Delete key belongs to the note list (M175). Written by the render
/// callback, false while it does not draw.
static DELETE_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when the Delete key was caught; the render callback deletes (M175).
static DELETE_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether Ctrl+Z, Ctrl+Y and Ctrl+Shift+Z belong to the window (M135,
/// in-game test 25). Written by the render callback, false while it does not
/// draw.
static UNDO_KEYS_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when an undo key was caught: [`HISTORY_STEP_UNDO`] or
/// [`HISTORY_STEP_REDO`]; the render callback takes the step.
static HISTORY_STEP: AtomicU8 = AtomicU8::new(HISTORY_STEP_NONE);
/// [`HISTORY_STEP`]: no step waiting.
const HISTORY_STEP_NONE: u8 = 0;
/// [`HISTORY_STEP`]: undo (Ctrl+Z).
const HISTORY_STEP_UNDO: u8 = 1;
/// [`HISTORY_STEP`]: redo (Ctrl+Y or Ctrl+Shift+Z).
const HISTORY_STEP_REDO: u8 = 2;

/// Whether Ctrl+0 belongs to the window (M79, in-game test 34). Written by
/// the render callback, false while it does not draw.
static TEXT_SIZE_RESET_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Ctrl+0 was caught; the render callback resets the text size.
static TEXT_SIZE_RESET_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether Enter answers memo's open dialog (in-game test 35). Written by the
/// render callback, false while it does not draw.
static DIALOG_ENTER_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Enter was caught for the dialog; the render callback answers it.
static DIALOG_ENTER_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether Escape belongs to the main window: it or one of its panes has the
/// focus. Nexus closes a window on Escape only while the window itself has
/// the focus, not one of its child panes (list, note, window menu), so memo
/// catches the key there itself. Written by the render callback, false while
/// it does not draw.
static ESCAPE_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Escape was caught for the main window; the render callback closes
/// what is open in it, else the window (as its close button does).
static ESCAPE_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Ctrl+C and Ctrl+A caught since the last frame; the main window and the
/// help window each apply them only while they belong to it (M217).
#[derive(Debug, Clone, Copy, Default)]
struct SelectionKeys {
    /// Ctrl+C was caught.
    copy: bool,
    /// Ctrl+A was caught.
    select_all: bool,
}

/// Whether Ctrl+C belongs to the window: it has the focus and text of the
/// reading view is selected (M217). Written by the render callback.
static COPY_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Ctrl+C was caught; the render callback copies the selection.
static COPY_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Whether Ctrl+A belongs to the window: it has the focus and shows a note to
/// read (M217). Written by the render callback.
static SELECT_ALL_KEY_ARMED: AtomicBool = AtomicBool::new(false);

/// Set when Ctrl+A was caught; the render callback selects the whole note.
static SELECT_ALL_KEY_PRESSED: AtomicBool = AtomicBool::new(false);

/// Gives every key memo catches back to the game until the render callback
/// arms them again (M79, M135, M174, M175, M198, M199).
fn disarm_window_keys() {
    for armed in [
        &SEARCH_KEY_ARMED,
        &FIND_KEY_ARMED,
        &FIND_STEP_ARMED,
        &DELETE_KEY_ARMED,
        &UNDO_KEYS_ARMED,
        &TEXT_SIZE_RESET_ARMED,
        &DIALOG_ENTER_ARMED,
        &ESCAPE_KEY_ARMED,
        &COPY_KEY_ARMED,
        &SELECT_ALL_KEY_ARMED,
    ] {
        armed.store(false, Ordering::Relaxed);
    }
}

/// PROTOTYPE (R159): the Windows cursor the window message callback sets on
/// `WM_SETCURSOR`; `CURSOR_NONE` leaves the cursor to the game.
static REQUESTED_CURSOR: AtomicU8 = AtomicU8::new(CURSOR_NONE);
/// No cursor request: the game sets its own.
const CURSOR_NONE: u8 = 0;
/// The arrow.
const CURSOR_ARROW: u8 = 1;
/// The text cursor (I-beam).
const CURSOR_TEXT: u8 = 2;
/// The hand over links.
const CURSOR_HAND: u8 = 3;
/// Resizing up and down.
const CURSOR_RESIZE_NS: u8 = 4;
/// Resizing left and right.
const CURSOR_RESIZE_EW: u8 = 5;
/// Resizing along the bottom-left corner.
const CURSOR_RESIZE_NESW: u8 = 6;
/// Resizing along the bottom-right corner.
const CURSOR_RESIZE_NWSE: u8 = 7;
/// Moving in every direction.
const CURSOR_RESIZE_ALL: u8 = 8;
/// Not allowed.
const CURSOR_NOT_ALLOWED: u8 = 9;

/// PROTOTYPE (R159): sets the requested Windows cursor; returns whether one
/// was requested, so the game must not set its own.
fn set_requested_cursor() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        IDC_ARROW, IDC_HAND, IDC_IBEAM, IDC_NO, IDC_SIZEALL, IDC_SIZENESW, IDC_SIZENS,
        IDC_SIZENWSE, IDC_SIZEWE, LoadCursorW, SetCursor,
    };
    let id = match REQUESTED_CURSOR.load(Ordering::Relaxed) {
        CURSOR_NONE => return false,
        CURSOR_TEXT => IDC_IBEAM,
        CURSOR_HAND => IDC_HAND,
        CURSOR_RESIZE_NS => IDC_SIZENS,
        CURSOR_RESIZE_EW => IDC_SIZEWE,
        CURSOR_RESIZE_NESW => IDC_SIZENESW,
        CURSOR_RESIZE_NWSE => IDC_SIZENWSE,
        CURSOR_RESIZE_ALL => IDC_SIZEALL,
        CURSOR_NOT_ALLOWED => IDC_NO,
        _ => IDC_ARROW,
    };
    // SAFETY: loading a shared system cursor and setting it are plain calls
    // on the thread that processes the window's messages.
    if let Ok(cursor) = unsafe { LoadCursorW(None, id) } {
        // SAFETY: `cursor` is a shared system cursor that is never freed.
        unsafe { SetCursor(Some(cursor)) };
    }
    true
}

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

/// Called by Nexus when the addon is loaded; see [`load_addon`]. A panic
/// stops memo instead of reaching Nexus (R104).
fn load() {
    guard(Callback::Load, (), load_addon);
}

/// Resolves the language, caches the Nexus link and registers the keybind
/// (with its localized name), the quick access icon and the render callback.
fn load_addon() {
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
        guard(Callback::Keybind, (), || toggle_key(is_release));
    });
    register_keybind_with_string(KEYBIND_TOGGLE, on_toggle, UNBOUND).revert_on_unload();
    register_wnd_proc(catch_window_keys).revert_on_unload();
    #[cfg(feature = "fault-injection")]
    fault_injection::register();
    // UI strings plus the help with the privacy policy, which the help
    // window draws; this also builds both help texts off the render thread.
    register_fonts(
        GLYPHS_KEY,
        &(crate::all_texts() + help_document(Language::English) + help_document(Language::German)),
        &ThemeKind::default().theme().typography,
    );

    NEXUS_LINK.store(get_nexus_link().cast_mut(), Ordering::Release);

    if let Ok(mut help) = HELP.lock() {
        *help = Some(MemoHelp::new(
            trailkit_directory.as_ref().map(TrailkitDirectory::logs),
        ));
    }
    let settings = start_storage(trailkit_directory);
    // Size the fonts now, while no memo window is open: resizing them on the
    // first frame rebuilds Nexus' font atlas and the whole window blinks once
    // (Q371). If Nexus' `FontUI` is not ready yet (game start), the render
    // callback still sizes them on the first frame.
    render::follow_nexus_font_size(
        settings.text_scale,
        &ThemeKind::default().theme().typography,
    );

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
        render!(|ui| guard(Callback::Render, (), || render::render(ui))),
    )
    .revert_on_unload();
    register_render(
        RenderType::OptionsRender,
        render!(|ui| guard(Callback::OptionsRender, (), || {
            render::render_options_tab(ui);
        })),
    )
    .revert_on_unload();
}

/// The open/close keybind (also fired by the quick access icon): toggles the
/// main window, or opens the help window after an error (R104, Q366). Only
/// touches atomics (nexus-addon-dev skill §2).
fn toggle_key(is_release: bool) {
    if is_release {
        return;
    }
    match run_state() {
        RunState::Running => {
            VISIBLE.fetch_xor(true, Ordering::Relaxed);
        }
        RunState::HelpOnly => guard::request_help(),
        RunState::Stopped => {}
    }
}

/// Called by Nexus before the addon is unloaded; see [`unload_addon`]. Runs
/// in every [`RunState`]; a panic is only logged (R104).
fn unload() {
    guard(Callback::Unload, (), unload_addon);
}

/// Every registration was made with `revert_on_unload`; the cached Nexus
/// link pointer is dropped (nexus-addon-dev skill §1) and the storage worker
/// is shut down, which writes all pending saves before returning. Locks a
/// panic poisoned are used anyway, so the worker still shuts down.
fn unload_addon() {
    if let Some(window_title) = WINDOW_TITLE.get() {
        deregister_close_on_escape(window_title);
    }
    deregister_close_on_escape(help::WINDOW_ID);
    NEXUS_LINK.store(std::ptr::null_mut(), Ordering::Release);
    // Unsaved edits become a draft; the worker writes it while shutting down
    // (M54). After an error the guard already kept it (R104, Q360).
    if run_state() == RunState::Running
        && let NotesState::Ready(window) = &mut *lock_after_panic(&NOTES)
    {
        window.keep_draft_for_unload();
    }
    let worker = lock_after_panic(&STORAGE_WORKER).take();
    drop(worker);
    *lock_after_panic(&NOTES) = NotesState::Unavailable;
    *lock_after_panic(&HELP) = None;
    HELP_OPEN.store(false, Ordering::Relaxed);
    // Last, so every line above still reaches the file; writes `session_end`.
    logging::stop_logging();
}

/// Locks `mutex`, waiting for it, also when a panic poisoned it (R104).
/// Only for `unload`, never on the render thread.
fn lock_after_panic<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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

/// Catches memo's keys that would not reach ImGui on the input thread (M79,
/// M135, M174, M175), while they belong to the window, and counts right and middle mouse
/// button presses for the focus (M186); otherwise the game gets them. Only
/// touches atomics (nexus-addon-dev skill §2).
extern "C-unwind" fn catch_window_keys(
    _window: HWND,
    message: u32,
    key: WPARAM,
    details: LPARAM,
) -> u32 {
    guard(Callback::WindowMessages, MESSAGE_PASSED, || {
        if run_state() == RunState::Running {
            route_window_message(message, key, details)
        } else {
            // After an error the game gets every key and click (R104).
            MESSAGE_PASSED
        }
    })
}

/// The body of [`catch_window_keys`] while memo runs. `details` carries the
/// key's previous state (bit 30), which marks auto repeat.
fn route_window_message(message: u32, key: WPARAM, details: LPARAM) -> u32 {
    if message == windows::Win32::UI::WindowsAndMessaging::WM_SETCURSOR && set_requested_cursor() {
        return MESSAGE_CONSUMED;
    }
    if message == WM_RBUTTONDOWN || message == WM_MBUTTONDOWN {
        // Clicks into the game never reach ImGui; the window frame uses this
        // to drop the focus (M186). The game still gets the click.
        record_mouse_button_press();
        return MESSAGE_PASSED;
    }
    let armed = ArmedKeys {
        search: SEARCH_KEY_ARMED.load(Ordering::Relaxed),
        find: FIND_KEY_ARMED.load(Ordering::Relaxed),
        find_step: FIND_STEP_ARMED.load(Ordering::Relaxed),
        delete: DELETE_KEY_ARMED.load(Ordering::Relaxed),
        undo: UNDO_KEYS_ARMED.load(Ordering::Relaxed),
        text_size_reset: TEXT_SIZE_RESET_ARMED.load(Ordering::Relaxed),
        dialog_enter: DIALOG_ENTER_ARMED.load(Ordering::Relaxed),
        escape: ESCAPE_KEY_ARMED.load(Ordering::Relaxed),
        copy: COPY_KEY_ARMED.load(Ordering::Relaxed),
        select_all: SELECT_ALL_KEY_ARMED.load(Ordering::Relaxed),
    };
    let key_message = KeyMessage {
        message,
        key: key.0,
        is_repeat: details.0 & KEY_WAS_DOWN != 0,
    };
    let signal = route_key(
        key_message,
        armed,
        Instant::now(),
        &mut lock_double_shift(),
        is_down,
    );
    let Some(signal) = signal else {
        return MESSAGE_PASSED;
    };
    match signal {
        KeySignal::ToggleSearch => SEARCH_KEY_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::ToggleFind => FIND_KEY_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::FindNext => FIND_STEP.store(FIND_STEP_NEXT, Ordering::Relaxed),
        KeySignal::FindPrevious => FIND_STEP.store(FIND_STEP_PREVIOUS, Ordering::Relaxed),
        KeySignal::Delete => DELETE_KEY_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::Undo => HISTORY_STEP.store(HISTORY_STEP_UNDO, Ordering::Relaxed),
        KeySignal::Redo => HISTORY_STEP.store(HISTORY_STEP_REDO, Ordering::Relaxed),
        KeySignal::ResetTextSize => TEXT_SIZE_RESET_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::ConfirmDialog => DIALOG_ENTER_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::Escape => ESCAPE_KEY_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::Copy => COPY_KEY_PRESSED.store(true, Ordering::Relaxed),
        KeySignal::SelectAll => SELECT_ALL_KEY_PRESSED.store(true, Ordering::Relaxed),
    }
    if signal.consumes_key() {
        MESSAGE_CONSUMED
    } else {
        MESSAGE_PASSED
    }
}

/// Bit 30 of a key message's `lParam`: the key was already down (repeat).
const KEY_WAS_DOWN: isize = 1 << 30;

/// Whether `key` is held, as seen by the thread processing window messages.
fn is_down(key: VIRTUAL_KEY) -> bool {
    // SAFETY: `GetKeyState` only reads the calling thread's key state, and this
    // thread is processing the window's messages. The high bit means "down".
    unsafe { GetKeyState(i32::from(key.0)) < 0 }
}

/// Locks [`DOUBLE_SHIFT`]; a poisoned lock keeps its last state, which is
/// only timing.
fn lock_double_shift() -> std::sync::MutexGuard<'static, DoubleShift> {
    DOUBLE_SHIFT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
