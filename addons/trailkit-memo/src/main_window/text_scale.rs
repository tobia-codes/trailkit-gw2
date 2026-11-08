//! The window's text size: Ctrl+mouse wheel and Ctrl+0 (M79), applied once
//! the wheel rests and shown as a percentage while it changes (M80).

use nexus::imgui::{Ui, WindowHoveredFlags};
use trailkit_ui::tooltip::draw_tooltip;

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
        if scale != self.settings.text_scale {
            self.settings.text_scale = scale;
            self.saver.save_settings(self.settings);
        }
        // Show the size even when it is already at a limit.
        self.text_scale_changed_at = Some(ui.time());
    }

    /// Applies a changed text size once the wheel rests and shows its
    /// percentage next to the mouse while it changes (M80).
    pub(super) fn update_text_scale(&mut self, ui: &Ui, language: Language) {
        let Some(changed_at) = self.text_scale_changed_at else {
            return;
        };
        let elapsed = ui.time() - changed_at;
        if elapsed >= TEXT_SCALE_APPLY_DELAY_SECONDS
            && self.applied_text_scale != self.settings.text_scale
        {
            self.applied_text_scale = self.settings.text_scale;
            // Once per resting wheel, not per step (ROADMAP R92).
            log::info!(
                key = "text_scale",
                value = f64::from(self.settings.text_scale.factor());
                "setting_changed"
            );
        }
        if elapsed < TEXT_SCALE_INDICATOR_SECONDS {
            draw_tooltip(
                ui,
                label_with_percent(
                    Text::TextSize.get(language),
                    self.settings.text_scale.factor() * 100.0,
                    language,
                ),
            );
        } else {
            self.text_scale_changed_at = None;
        }
    }
}
