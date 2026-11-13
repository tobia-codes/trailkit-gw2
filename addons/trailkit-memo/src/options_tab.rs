//! What memo draws into its tab of the Nexus options (ROADMAP R53, memo
//! Q307–Q309, mockup https://claude.ai/artifact/94EzMkYMDQHFmLw1b2fTdZ
//! variant A): on its own trailkit pane, the window title in Heading2,
//! `trailkit-memo <version>` in Secondary and the button "Open help". The
//! button "Open settings" joins once the settings window exists (Q307).
//!
//! Nexus draws the options window, its title and tabs itself; this module
//! only fills the tab and reports the click. Render thread only.

use nexus::imgui::Ui;

use trailkit_memo_shared::i18n::{Language, LocalizedText as _, Text};
use trailkit_ui::{
    button::Button,
    font_registry::FontScope,
    fonts::FontRole,
    icons::Icon,
    pane::draw_pane,
    text::{TextStyle, draw_text},
    theme::Theme,
};

use crate::help_text::HelpText;

/// The ImGui id of the pane, stable across languages.
const PANE_ID: &str = "trailkit.memo.options";

/// The addon name and version below the title (Q309): it helps when
/// reporting a problem.
pub const VERSION_LINE: &str = concat!("trailkit-memo ", env!("CARGO_PKG_VERSION"));

/// Draws memo's options tab with `version_line` below the title and
/// returns whether "Open help" was clicked. The caller pushes the theme
/// ([`trailkit_ui::style_scope::StyleScope`]).
pub fn draw_options_tab(ui: &Ui, theme: &Theme, language: Language, version_line: &str) -> bool {
    let _body = FontScope::push(ui, FontRole::Body);
    let height = pane_height(ui, theme);
    draw_pane(ui, theme, PANE_ID, [0.0, height], || {
        draw_text(
            ui,
            theme,
            TextStyle::Heading2,
            Text::MainWindowTitle.get(language),
        );
        draw_text(ui, theme, TextStyle::Secondary, version_line);
        Button::new(HelpText::OpenHelp.get(language))
            .icon(Icon::CIRCLE_HELP)
            .draw(ui, theme)
    })
    .unwrap_or(false)
}

/// The height of the pane: its padding around the title, the version
/// line and the button. ImGui 1.80 child windows cannot fit their content.
fn pane_height(ui: &Ui, theme: &Theme) -> f32 {
    let title = {
        let _heading = TextStyle::Heading2.push(ui, theme);
        ui.text_line_height()
    };
    let version = {
        let _secondary = TextStyle::Secondary.push(ui, theme);
        ui.text_line_height()
    };
    let spacing = ui.clone_style().item_spacing[1];
    2.0 * theme.pane.padding + title + version + ui.frame_height() + 2.0 * spacing
}

#[cfg(test)]
mod tests {
    //! Snapshots of the options tab (Q307–Q309) in Dark, Light and German,
    //! run with `cargo xtask ui-test`.

    use nexus::imgui::{Condition, Window};
    use trailkit_ui::{style_scope::StyleScope, theme::ThemeKind};
    use trailkit_ui_testing::{
        harness::{FrameInput, UiHarness},
        snapshot::assert_snapshot,
    };

    use super::*;

    /// About the width of the tab in Nexus' options window.
    const TAB: [f32; 2] = [400.0, 140.0];

    /// A fixed version, so the snapshots do not change with every release.
    const VERSION: &str = "trailkit-memo 0.3.0";

    #[test]
    fn options_tab_in_every_theme_and_language() {
        for (name, theme, language) in [
            ("dark", ThemeKind::Dark, Language::English),
            ("light", ThemeKind::Light, Language::English),
            ("de-dark", ThemeKind::Dark, Language::German),
        ] {
            let theme = theme.theme();
            let mut harness = UiHarness::new(TAB);
            let image = harness.settle(4, &FrameInput::default(), |ui| {
                // Stands in for Nexus' options window around the tab.
                Window::new("Options###nexus-options")
                    .position([0.0, 0.0], Condition::Always)
                    .size(TAB, Condition::Always)
                    .title_bar(false)
                    .build(ui, || {
                        let _style = StyleScope::push(ui, &theme);
                        draw_options_tab(ui, &theme, language, VERSION);
                    });
            });
            assert_snapshot(
                env!("CARGO_MANIFEST_DIR"),
                &format!("memo-options-tab-{name}"),
                &image,
            );
        }
    }
}
