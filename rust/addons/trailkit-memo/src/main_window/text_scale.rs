//! The window's text size: Ctrl+mouse wheel and Ctrl+0 (M79), applied once
//! the wheel rests and shown as a percentage while it changes (M80).

use nexus::imgui::{Ui, WindowHoveredFlags};
use trailkit_ui::{controls::tooltip::draw_tooltip, theme::Theme};

use trailkit_memo_shared::{
    i18n::{Language, LocalizedText, Text, label_with_percent},
    settings::TextScale,
};

use super::MemoWindow;

/// How long the text size waits after the last Ctrl+mouse wheel step before
/// the fonts are rebuilt (M80): rebuilding on every step flickers.
const TEXT_SCALE_APPLY_DELAY_SECONDS: f64 = 0.4;

/// How long the text size percentage stays visible after a change (M80).
const TEXT_SCALE_INDICATOR_SECONDS: f64 = 1.5;

/// Windows virtual-key code of `0` (Nexus feeds ImGui with virtual-key
/// codes; Ctrl+0 resets the text size, M79).
const VIRTUAL_KEY_0: i32 = 0x30;

impl MemoWindow {
    /// Ctrl+mouse wheel over the window makes the text larger or smaller,
    /// Ctrl+0 resets it (M79); the change is saved and takes effect when the
    /// addon resizes the fonts next frame.
    pub(super) fn handle_text_scale_keys(&mut self, ui: &Ui) {
        let io = ui.io();
        if !io.key_ctrl {
            return;
        }
        let hovered = ui.is_window_hovered_with_flags(
            WindowHoveredFlags::ROOT_AND_CHILD_WINDOWS
                | WindowHoveredFlags::ALLOW_WHEN_BLOCKED_BY_ACTIVE_ITEM,
        );
        let scale = if hovered && io.mouse_wheel != 0.0 {
            self.settings.text_scale.stepped(io.mouse_wheel.signum())
        } else if self.window_focused && ui.is_key_index_pressed(VIRTUAL_KEY_0) {
            TextScale::DEFAULT
        } else {
            return;
        };
        self.change_text_scale(ui, scale);
    }

    /// Whether Ctrl+0 belongs to this window (M79): it has the focus and no
    /// text input is active, which would get the key from Nexus itself. The
    /// window message callback catches it only then, because the game would
    /// otherwise get it (in-game test 34).
    #[must_use]
    pub const fn takes_text_size_reset_key(&self) -> bool {
        self.window_focused && !self.text_input_active
    }

    /// Resets the text size after the window message callback caught Ctrl+0
    /// (M79).
    pub fn reset_text_size_by_key(&mut self, ui: &Ui) {
        if self.takes_text_size_reset_key() {
            self.change_text_scale(ui, TextScale::DEFAULT);
        }
    }

    /// Saves `scale` as the text size and shows the size tooltip, which the
    /// change applies to after a short rest (M80).
    fn change_text_scale(&mut self, ui: &Ui, scale: TextScale) {
        if scale != self.settings.text_scale {
            self.settings.text_scale = scale;
            self.saver.save_settings(self.settings);
        }
        // Show the size even when it is already at a limit.
        self.text_scale_changed_at = Some(ui.time());
    }

    /// Applies a changed text size once the wheel rests and shows its
    /// percentage next to the mouse in a `theme` tooltip while it changes
    /// (M80).
    pub(super) fn update_text_scale(&mut self, ui: &Ui, language: Language, theme: &Theme) {
        let Some(changed_at) = self.text_scale_changed_at else {
            return;
        };
        let elapsed = ui.time() - changed_at;
        if elapsed >= TEXT_SCALE_APPLY_DELAY_SECONDS
            && self.applied_text_scale != self.settings.text_scale
        {
            self.applied_text_scale = self.settings.text_scale;
        }
        if elapsed < TEXT_SCALE_INDICATOR_SECONDS {
            draw_tooltip(
                ui,
                theme,
                label_with_percent(
                    Text::TextSize.get(language),
                    self.settings.text_scale.percent_of_default(),
                    language,
                ),
            );
        } else {
            self.text_scale_changed_at = None;
        }
    }
}
