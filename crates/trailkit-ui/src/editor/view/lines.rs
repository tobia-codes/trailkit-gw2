//! Drawing of the editor's visible lines (memo plan M58–M60): the cached
//! layout, selection, formatted text, chat-link chips and the cursor.

use std::ops::Range;

use nexus::imgui::Ui;

use super::{BLINK_HALF_PERIOD, TextEditor, is_editor_hovered};
use crate::{
    chat_link::{ChatLinkChip, ChipText},
    chat_link_chip::{ChipContext, chip_tooltip, chip_width, draw_chip},
    draw_list::{WindowDrawList, window_draw_list},
    editor::{
        highlight::SpanStyle,
        layout::{Layout, VisualLine},
    },
    font_registry::FontScope,
    fonts::FontRole,
    theme::Theme,
};

/// The width of `text` in the current font, exactly as ImGui draws it.
/// `Ui::calc_text_size` rounds every result up to a whole pixel, which adds
/// up when characters are measured one by one and moves the cursor away
/// from the text.
fn unrounded_width(text: &str) -> f32 {
    let mut size = nexus::imgui::sys::ImVec2 { x: 0.0, y: 0.0 };
    let begin = text.as_ptr().cast::<std::ffi::c_char>();
    // SAFETY: called on the render thread inside a frame, so the current
    // font is valid; `begin..begin + len` is the UTF-8 text, which ImGui
    // only reads; the remaining-text output pointer may be null.
    unsafe {
        nexus::imgui::sys::ImFont_CalcTextSizeA(
            &raw mut size,
            nexus::imgui::sys::igGetFont(),
            nexus::imgui::sys::igGetFontSize(),
            f32::MAX,
            0.0,
            begin,
            begin.add(text.len()),
            std::ptr::null_mut(),
        );
    }
    size.x
}

/// The layout of a specific text revision, width and font size.
#[derive(Debug)]
pub(super) struct CachedLayout {
    /// Text revision the layout was built for.
    revision: u64,
    /// Wrap width.
    width: f32,
    /// Font size.
    font_size: f32,
    /// Identifies the chip texts (their language) the chips were measured
    /// with: the address of one of them.
    chip_texts: usize,
    /// The layout.
    pub(super) layout: Layout,
}

impl TextEditor {
    /// Rebuilds the layout if the text, width, font size or the language of
    /// the chip texts changed.
    pub(super) fn refresh_layout(
        &mut self,
        ui: &Ui,
        theme: &Theme,
        width: f32,
        font_size: f32,
        chips: ChipContext<'_>,
    ) {
        let chip_texts = (chips.text)(ChipText::ClickToCopy).as_ptr().addr();
        let is_current = self.layout.as_ref().is_some_and(|cached| {
            cached.revision == self.revision
                && (cached.width - width).abs() < 0.5
                && (cached.font_size - font_size).abs() < f32::EPSILON
                && cached.chip_texts == chip_texts
        });
        if is_current {
            return;
        }
        let layout = Layout::build(self.buffer.text(), width, |piece, style| match style {
            SpanStyle::Strong => {
                let _bold = FontScope::push(ui, FontRole::BodyBold);
                unrounded_width(piece)
            }
            SpanStyle::ChatLink => chip_width(ui, &ChatLinkChip::new(piece), theme, chips),
            SpanStyle::Text | SpanStyle::Marker => unrounded_width(piece),
        });
        self.layout = Some(CachedLayout {
            revision: self.revision,
            width,
            font_size,
            chip_texts,
            layout,
        });
    }

    /// Draws the visible lines: selection, highlighted text and cursor.
    #[expect(clippy::too_many_arguments, reason = "drawing state of one frame")]
    pub(super) fn draw_lines(
        &self,
        ui: &Ui,
        theme: &Theme,
        layout: &Layout,
        origin: [f32; 2],
        line_height: f32,
        scroll: f32,
        visible_height: f32,
        chips: ChipContext<'_>,
    ) {
        let draw_list = window_draw_list(ui);
        // Text sits in the middle of its taller line.
        let leading = theme.typography.leading_for(ui.current_font_size());
        let selection = self.buffer.selection();
        let cursor = self.buffer.cursor();
        let cursor_line = layout.line_index_of(cursor);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "scroll offsets are small and non-negative"
        )]
        let first_visible = (scroll / line_height).floor().max(0.0) as usize;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "line counts are small"
        )]
        let visible_lines = (visible_height / line_height).ceil() as usize + 1;
        let line_context = LineContext {
            theme,
            layout,
            text: self.buffer.text(),
            selection,
            line_height,
            editor_hovered: is_editor_hovered(ui),
            chips,
        };
        #[expect(clippy::cast_possible_truncation, reason = "only the parity matters")]
        let blink_on = ((ui.time() - self.blink_start) / BLINK_HALF_PERIOD) as i64 % 2 == 0;
        for (index, line) in layout
            .lines()
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(visible_lines)
        {
            #[expect(clippy::cast_precision_loss, reason = "line counts are small")]
            let top = origin[1] + index as f32 * line_height;
            draw_line_selection(ui, draw_list, &line_context, line, [origin[0], top]);
            draw_line_text(
                ui,
                draw_list,
                &line_context,
                line,
                [origin[0], top + leading],
            );
            if self.focused && index == cursor_line && blink_on {
                let x = origin[0] + line.x_of(cursor);
                draw_list
                    .add_line(
                        [x, top + leading],
                        [x, top + leading + ui.text_line_height()],
                        theme.text.primary.to_array(),
                    )
                    .thickness(1.0)
                    .build();
            }
        }
    }
}

/// What every visible line of one frame is drawn with.
struct LineContext<'frame> {
    /// Colors, fonts and radii.
    theme: &'frame Theme,
    /// The wrapped, highlighted text.
    layout: &'frame Layout,
    /// The editor text.
    text: &'frame str,
    /// The selected range (start ≤ end).
    selection: Range<usize>,
    /// Distance between two lines.
    line_height: f32,
    /// Whether the mouse is over the editor.
    editor_hovered: bool,
    /// Texts of chat-link chips.
    chips: ChipContext<'frame>,
}

/// Draws the selection highlight of `line` from `line_origin` (screen
/// position of the line's top left), if the selection touches it; a selected
/// line break shows as a small extra width.
fn draw_line_selection(
    ui: &Ui,
    draw_list: WindowDrawList<'_>,
    context: &LineContext<'_>,
    line: &VisualLine,
    line_origin: [f32; 2],
) {
    let selection = &context.selection;
    if selection.is_empty() || selection.start > line.range.end || selection.end < line.range.start
    {
        return;
    }
    let start_x = line.x_of(selection.start.max(line.range.start));
    let mut end_x = line.x_of(selection.end.min(line.range.end));
    if selection.end > line.range.end && line.ends_logical_line {
        end_x += ui.calc_text_size(" ")[0];
    }
    draw_list
        .add_rect(
            [line_origin[0] + start_x, line_origin[1]],
            [line_origin[0] + end_x, line_origin[1] + context.line_height],
            context.theme.accent.base.with_alpha(0.35).to_array(),
        )
        .filled(true)
        .rounding(context.theme.radius.small)
        .build();
}

/// Draws the formatted text of `line` from `text_origin` (screen position of
/// the line's text top): bold, dimmed markdown markers and chat-link chips,
/// a chip tinted while selected and with its tooltip on hover.
fn draw_line_text(
    ui: &Ui,
    draw_list: WindowDrawList<'_>,
    context: &LineContext<'_>,
    line: &VisualLine,
    text_origin: [f32; 2],
) {
    for span in context.layout.spans_in(line.range.clone()) {
        let position = [text_origin[0] + line.x_of(span.range.start), text_origin[1]];
        let piece = &context.text[span.range.clone()];
        match span.style {
            SpanStyle::Strong => {
                let _bold = FontScope::push(ui, FontRole::BodyBold);
                draw_list.add_text(position, context.theme.text.primary.to_array(), piece);
            }
            SpanStyle::Marker => {
                draw_list.add_text(position, context.theme.text.secondary.to_array(), piece);
            }
            SpanStyle::Text => {
                draw_list.add_text(position, context.theme.text.primary.to_array(), piece);
            }
            SpanStyle::ChatLink => {
                let Some(chip) = context.layout.chip_at(span.range.start) else {
                    draw_list.add_text(position, context.theme.text.primary.to_array(), piece);
                    continue;
                };
                let [min, max] =
                    draw_chip(ui, draw_list, chip, position, context.theme, context.chips);
                // The chip covers the selection drawn below it.
                if context.selection.start < span.range.end
                    && span.range.start < context.selection.end
                {
                    draw_list
                        .add_rect(
                            min,
                            max,
                            context.theme.accent.base.with_alpha(0.35).to_array(),
                        )
                        .filled(true)
                        .rounding(context.theme.radius.small)
                        .build();
                }
                if context.editor_hovered && ui.is_mouse_hovering_rect(min, max) {
                    chip_tooltip(ui, chip, context.theme, context.chips, false);
                }
            }
        }
    }
}
