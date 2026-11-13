//! memo's Nexus render callbacks (render thread only, nexus-addon-dev skill
//! §2): the main window with its title bar toggles and the help window
//! (`RT_Render`), and memo's tab in the Nexus options (`RT_OptionsRender`,
//! M182). The state they read lives in the parent module's statics. After
//! memo stopped on an error only the help window is drawn (R104, Q366).

use std::sync::atomic::Ordering;

use nexus::imgui::{Condition, Window};

use trailkit_memo_shared::{
    i18n::{Language, LocalizedText as _, Text},
    settings::TextScale,
};
use trailkit_ui::{
    font_registry::{follow_body_size, font_size_of},
    style_scope::StyleScope,
    text::{TextStyle, draw_text, draw_text_wrapped},
    theme::{Theme, ThemeKind, Typography},
    title_bar::{TitleBarToggle, draw_window},
};

use super::{
    CLOSE_REQUESTED, DELETE_KEY_ARMED, DELETE_KEY_PRESSED, HELP, HELP_OPEN, LANGUAGE, NEXUS_LINK,
    NOTES, NotesState, SEARCH_ESCAPED, SEARCH_KEY_ARMED, SEARCH_KEY_PRESSED, SEARCH_TAKES_ESCAPE,
    TITLE_BAR, UNSAVED_CHANGES, VISIBLE, WINDOW_TITLE,
    guard::{RunState, draft_kept, run_state, take_help_request, try_lock_after_panic},
    help::MemoHelp,
    is_gameplay, settings,
    storage::{receive_loaded_notes, restart_storage_worker_if_asked},
};
use crate::{
    main_window::{minimum_window_width, title_bar_texts},
    options_tab::{VERSION_LINE, draw_options_tab},
};

/// Nexus' render callback: draws memo while it runs, only the help window
/// after it stopped on an error (R104), nothing once it stopped completely.
pub(super) fn render(ui: &nexus::imgui::Ui) {
    match run_state() {
        RunState::Running => render_main_window(ui),
        RunState::HelpOnly => render_help_only(ui),
        RunState::Stopped => {}
    }
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
    #[cfg(feature = "fault-injection")]
    super::fault_injection::panic_if_requested();
    if let Some(help) = help.as_deref_mut()
        && !HELP_OPEN.load(Ordering::Relaxed)
    {
        help.close();
    }
    if main_visible {
        draw_main_window(ui, &theme, language, &mut notes, help.as_deref_mut());
    }
    if let Some(help) = help {
        if let NotesState::Ready(window) = &*notes {
            help.restore_once(window.help_window_state());
        }
        help.draw(ui, &theme, language);
        HELP_OPEN.store(help.is_open(), Ordering::Relaxed);
        if let Some(state) = help.take_state_to_save()
            && let NotesState::Ready(window) = &mut *notes
        {
            window.save_help_window_state(state);
        }
    }
}

/// Draws only the help window after memo stopped on an error (R104, Q366):
/// opened on "Report a problem" with the warning card when the panic or the
/// keybind asked for it. Touches neither the notes nor memo's settings
/// window state, and stays visible outside gameplay, so the way to report
/// the problem is never hidden.
fn render_help_only(ui: &nexus::imgui::Ui) {
    // Never wait on the render thread; the lock may be poisoned by the panic.
    let Some(mut help) = try_lock_after_panic(&HELP) else {
        return;
    };
    let Some(help) = help.as_mut() else {
        return;
    };
    if take_help_request() {
        help.open_on_problem_tab();
        HELP_OPEN.store(true, Ordering::Relaxed);
    } else if !HELP_OPEN.load(Ordering::Relaxed) {
        // Nexus cleared the flag on Escape.
        help.close();
    }
    if !help.is_open() {
        return;
    }
    let theme = ThemeKind::default().theme();
    follow_nexus_font_size(settings().text_scale, &theme.typography);
    let language = Language::from_u8(LANGUAGE.load(Ordering::Relaxed));
    let _theme = StyleScope::push(ui, &theme);
    help.draw_stopped(ui, &theme, language, draft_kept());
    HELP_OPEN.store(help.is_open(), Ordering::Relaxed);
}

/// Nexus' options render callback: memo's tab in the Nexus options (R53,
/// Q307–Q309). "Open help" opens the help window or brings it to the front
/// (Q296); the main render callback draws it from the next frame on. Draws
/// nothing after memo stopped on an error (R104).
pub(super) fn render_options_tab(ui: &nexus::imgui::Ui) {
    if run_state() != RunState::Running {
        return;
    }
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
