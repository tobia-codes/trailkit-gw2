//! Nexus glue: addon export, load/unload and render callback (Windows only).
//!
//! Threading (nexus-addon-dev skill §2): the keybind callback only flips an
//! atomic; the render callback returns after one relaxed atomic load while the
//! window is closed. Notes are loaded and saved by the storage worker (memo
//! plan M30); only `settings.json` is read once in `load`, because the quick
//! access icon depends on it. All registrations use `revert_on_unload`, so hot
//! reload cleans up automatically; the cached `NexusLink` pointer is dropped
//! and the storage worker is shut down (writing pending saves) in `unload`.

use std::{
    path::Path,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicBool, AtomicPtr, AtomicU8, Ordering},
        mpsc::{Receiver, TryRecvError},
    },
};

use jiff::Timestamp;

use nexus::{
    AddonFlags, UpdateProvider,
    data_link::{NexusLink, get_nexus_link},
    gui::{RenderType, register_render, render},
    imgui::{Condition, Window},
    keybind::{keybind_handler, register_keybind_with_string},
    localization::{set_translation, translate},
    paths::get_addon_dir,
    quick_access::add_quick_access,
    texture::load_texture_from_memory,
};

use trailkit_storage::{BACKUPS_DIRECTORY_NAME, DocumentStore, LoadOutcome, ReadOnlyReason};
use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};

use crate::{
    i18n::{Language, Text},
    note_tree::NotesDocument,
    settings::{QuickAccessIcon, Settings},
    storage_worker::{SAVE_DEBOUNCE, SavingAllowed, StorageWorker},
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

/// Folder below the Nexus `addons` folder holding memo's files (M17).
const ADDON_DIRECTORY_NAME: &str = "trailkit/memo";

/// User settings, read from `settings.json` once at load (defaults if absent).
static SETTINGS: OnceLock<Settings> = OnceLock::new();

/// The storage worker; only touched in `load` and `unload`.
static STORAGE_WORKER: Mutex<Option<StorageWorker>> = Mutex::new(None);

/// The notes as seen by the render thread. The render thread only uses
/// `try_lock`, so it never waits; `load` and `unload` are the only other users.
static NOTES: Mutex<NotesState> = Mutex::new(NotesState::Unavailable);

/// Where the render thread's notes currently come from.
enum NotesState {
    /// The storage worker is still loading `notes.json`.
    Loading(Receiver<LoadOutcome<NotesDocument>>),
    /// The notes are loaded (possibly empty after a problem, see `problem`).
    Ready {
        /// The note tree, owned by the render thread (M30).
        #[expect(dead_code, reason = "shown and edited from phase 1 step 2 on")]
        document: NotesDocument,
        /// A loading problem the user must be told about, if any.
        problem: Option<NotesProblem>,
    },
    /// Storage could not be started; details are in the Nexus log.
    Unavailable,
}

/// A problem with `notes.json` shown in the main window (M31).
enum NotesProblem {
    /// The file was unreadable and moved away; memo started empty. Holds the
    /// new location relative to the addon folder, ready for display.
    Recovered(String),
    /// The file is from a newer addon version; nothing is saved.
    NewerVersion,
    /// The file cannot be accessed; nothing is saved.
    Inaccessible,
}

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

    let settings = start_storage();

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
}

/// Called by Nexus before the addon is unloaded. Every registration was made
/// with `revert_on_unload`; the cached Nexus link pointer is dropped
/// (nexus-addon-dev skill §1) and the storage worker is shut down, which
/// writes all pending saves before returning.
fn unload() {
    log::info!("unloading");
    NEXUS_LINK.store(std::ptr::null_mut(), Ordering::Release);
    let worker = STORAGE_WORKER
        .lock()
        .map_or(None, |mut worker| worker.take());
    drop(worker);
    if let Ok(mut notes) = NOTES.lock() {
        *notes = NotesState::Unavailable;
    }
}

/// Reads `settings.json` and starts the storage worker, which loads the
/// notes in the background. Returns the settings to use (defaults if the file
/// is missing or unusable). Problems are logged; the addon keeps working.
fn start_storage() -> Settings {
    let directory = match get_addon_dir(ADDON_DIRECTORY_NAME) {
        Ok(directory) => directory,
        Err(error) => {
            log::error!("cannot resolve the addon folder: {error}; notes are unavailable");
            return *SETTINGS.get_or_init(|| Settings::DEFAULT);
        }
    };
    let store = DocumentStore::new(directory);

    let settings_outcome = store.load::<Settings>(Timestamp::now());
    let saving_allowed = SavingAllowed {
        settings: settings_outcome.allows_saving(),
    };
    match &settings_outcome {
        LoadOutcome::Missing | LoadOutcome::Loaded { .. } => {}
        LoadOutcome::Recovered {
            reason,
            preserved_as,
        } => log::error!(
            "settings.json was unreadable ({reason}); moved to {}",
            preserved_as.display()
        ),
        LoadOutcome::ReadOnly(reason) => log::warn!("settings.json is read-only: {reason:?}"),
    }
    let settings = settings_outcome.into_document().unwrap_or_default();
    let settings = *SETTINGS.get_or_init(|| settings);

    match StorageWorker::spawn(store, SAVE_DEBOUNCE, saving_allowed) {
        Ok((worker, loaded)) => {
            if let Ok(mut slot) = STORAGE_WORKER.lock() {
                *slot = Some(worker);
            }
            if let Ok(mut notes) = NOTES.lock() {
                *notes = NotesState::Loading(loaded);
            }
        }
        Err(error) => {
            log::error!("cannot start the storage worker: {error}; notes are unavailable")
        }
    }
    settings
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

/// Draws the main window. Returns immediately while the window is closed, and
/// skips drawing (keeping it open) outside gameplay unless the settings allow it.
fn render_main_window(ui: &nexus::imgui::Ui) {
    if !VISIBLE.load(Ordering::Relaxed) || !settings().allows_drawing(is_gameplay()) {
        return;
    }
    // Never wait on the render thread; if `load`/`unload` holds the lock, skip a frame.
    let Ok(mut notes) = NOTES.try_lock() else {
        return;
    };
    receive_loaded_notes(&mut notes);
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
        .build(ui, || draw_window_content(ui, &notes, language));
    if !open {
        VISIBLE.store(false, Ordering::Relaxed);
    }
}

/// Moves the notes from [`NotesState::Loading`] to [`NotesState::Ready`] once
/// the storage worker has delivered them. Never blocks.
fn receive_loaded_notes(notes: &mut NotesState) {
    let NotesState::Loading(loaded) = notes else {
        return;
    };
    *notes = match loaded.try_recv() {
        Ok(outcome) => ready_state(outcome),
        Err(TryRecvError::Empty) => return,
        Err(TryRecvError::Disconnected) => NotesState::Unavailable,
    };
}

/// Turns the worker's load result into the render thread's notes state.
fn ready_state(outcome: LoadOutcome<NotesDocument>) -> NotesState {
    let (document, problem) = match outcome {
        LoadOutcome::Missing => (NotesDocument::default(), None),
        LoadOutcome::Loaded { document, .. } => (document, None),
        LoadOutcome::Recovered { preserved_as, .. } => (
            NotesDocument::default(),
            Some(NotesProblem::Recovered(display_path_of_safety_copy(
                &preserved_as,
            ))),
        ),
        LoadOutcome::ReadOnly(ReadOnlyReason::NewerVersion { .. }) => {
            (NotesDocument::default(), Some(NotesProblem::NewerVersion))
        }
        LoadOutcome::ReadOnly(ReadOnlyReason::Io(_)) => {
            (NotesDocument::default(), Some(NotesProblem::Inaccessible))
        }
    };
    NotesState::Ready { document, problem }
}

/// Formats a safety copy's path relative to the addon folder
/// (`backups/notes.corrupt-….json`), computed once instead of every frame.
fn display_path_of_safety_copy(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |name| format!("{BACKUPS_DIRECTORY_NAME}/{}", name.to_string_lossy()),
    )
}

/// Draws the main window's content: a loading line, a problem hint or the
/// notes (the note list and editor follow in phase 1 steps 2–3).
fn draw_window_content(ui: &nexus::imgui::Ui, notes: &NotesState, language: Language) {
    match notes {
        NotesState::Loading(_) => ui.text(Text::Loading.get(language)),
        NotesState::Unavailable => ui.text_wrapped(Text::NotesInaccessible.get(language)),
        NotesState::Ready { problem, .. } => {
            match problem {
                Some(NotesProblem::Recovered(shown_path)) => {
                    ui.text_wrapped(Text::NotesRecovered.get(language));
                    ui.text_wrapped(shown_path);
                }
                Some(NotesProblem::NewerVersion) => {
                    ui.text_wrapped(Text::NotesNewerVersion.get(language));
                }
                Some(NotesProblem::Inaccessible) => {
                    ui.text_wrapped(Text::NotesInaccessible.get(language));
                }
                None => {}
            }
            ui.text(Text::Hello.get(language));
        }
    }
}
