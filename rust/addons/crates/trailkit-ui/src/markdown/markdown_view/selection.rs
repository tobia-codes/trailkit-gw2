//! Text selection while the markdown view draws (memo plan M217, P1–P6 of
//! the approved mock): fills the selected parts of each line, finds the text
//! position under the mouse and moves the selection with it.
//!
//! The view calls [`SelectionPass`] for every piece of text, chip, link icon,
//! space and line end it lays out; the pass fills what is selected behind it
//! and remembers the closest place to the mouse. After the document,
//! [`SelectionPass::finish`] applies the mouse to the [`TextSelection`].

use std::ops::Range;

use nexus::imgui::{MouseButton, MouseCursor, Ui};

use super::link_menu::request_selection_menu;
use crate::{
    drawing::draw_list::window_draw_list,
    markdown::MarkdownDocument,
    markdown::text_selection::{SelectionGesture, TextPosition, TextSelection},
    theme::Theme,
};

/// The vertical extent of one laid-out line (screen y).
#[derive(Debug, Clone, Copy)]
pub(super) struct LineBox {
    /// Top of the line, its leading included.
    pub(super) top: f32,
    /// Bottom of the line: the top of the next one.
    pub(super) bottom: f32,
}

/// What kind of element a candidate place lies in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Element {
    /// Plain text: the mouse shows the text cursor over it (P3).
    Text,
    /// A link, chip or anything else with its own cursor and click.
    Interactive,
    /// Empty room at a line's end.
    Room,
}

/// The place closest to the mouse found so far.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    /// Vertical distance from the mouse to the element's line, 0 inside it.
    vertical: f32,
    /// Horizontal distance from the mouse to the element, 0 inside it.
    horizontal: f32,
    /// The place.
    position: TextPosition,
    /// What lies under the mouse there.
    element: Element,
    /// Whether the element under the mouse is (partly) selected.
    over_selection: bool,
}

impl Candidate {
    /// Whether the mouse is right on the element.
    fn is_under_mouse(&self) -> bool {
        self.vertical == 0.0 && self.horizontal == 0.0
    }
}

/// The selection state of one drawing of the document.
#[derive(Debug)]
pub(super) struct SelectionPass {
    /// The selected range as it stood when the frame started.
    range: Option<Range<TextPosition>>,
    /// The fill behind selected text.
    color: [f32; 4],
    /// The mouse, while its place in the text is needed.
    probe: Option<[f32; 2]>,
    /// The closest place to the mouse so far.
    best: Option<Candidate>,
    /// The block being drawn.
    pub(super) block: usize,
    /// The span being drawn.
    pub(super) span: usize,
}

impl SelectionPass {
    /// Starts a frame of `selection`; `track_mouse` asks for the place under
    /// the mouse (the mouse is over the text or a gesture runs).
    pub(super) fn new(
        ui: &Ui,
        theme: &Theme,
        selection: &TextSelection,
        track_mouse: bool,
    ) -> Self {
        Self {
            range: selection.range(),
            color: theme.text_selection().to_array(),
            probe: track_mouse.then(|| ui.io().mouse_pos),
            best: None,
            block: 0,
            span: 0,
        }
    }

    /// The position before byte `offset` of the current span.
    pub(super) const fn at(&self, offset: usize) -> TextPosition {
        TextPosition::new(self.block, self.span, offset)
    }

    /// The selected bytes of the current span between `offsets`, if any.
    pub(super) fn selected_in(&self, offsets: Range<usize>) -> Option<Range<usize>> {
        let range = self.range.as_ref()?;
        let start = range.start.max(self.at(offsets.start));
        let end = range.end.min(self.at(offsets.end));
        (start < end).then_some(start.offset..end.offset)
    }

    /// Whether the line end before `position` is selected: the selection
    /// goes on past it to the next line (P2).
    pub(super) fn crosses(&self, position: TextPosition) -> bool {
        self.range
            .as_ref()
            .is_some_and(|range| range.start <= position && position < range.end)
    }

    /// Fills `left..right` of `line` as selected.
    pub(super) fn fill(&self, ui: &Ui, left: f32, right: f32, line: LineBox) {
        if right > left {
            window_draw_list(ui)
                .add_rect([left, line.top], [right, line.bottom], self.color)
                .filled(true)
                .build();
        }
    }

    /// Fills the selected part of the text `piece`, drawn at `left`, that
    /// starts at byte `offset` of the current span, limited to the piece's
    /// bytes `within` (all of them, or a filled search hit to cover again,
    /// P6). Uses the current font.
    pub(super) fn fill_text(
        &self,
        ui: &Ui,
        (piece, offset): (&str, usize),
        within: Range<usize>,
        left: f32,
        line: LineBox,
    ) {
        if let Some(selected) = self.selected_in(offset + within.start..offset + within.end) {
            let start = left + ui.calc_text_size(&piece[..selected.start - offset])[0];
            let end = left + ui.calc_text_size(&piece[..selected.end - offset])[0];
            self.fill(ui, start, end, line);
        }
    }

    /// Notes the text `piece` drawn at `left` on `line`, starting at byte
    /// `offset` of the current span, as a place for the mouse; `interactive`
    /// for link text. Uses the current font.
    pub(super) fn probe_text(
        &mut self,
        ui: &Ui,
        piece: &str,
        offset: usize,
        left: f32,
        line: LineBox,
        interactive: bool,
    ) {
        let right = left + ui.calc_text_size(piece)[0];
        let Some((vertical, horizontal, mouse_x)) = self.distances(left, right, line) else {
            return;
        };
        // The character boundary nearest to the mouse.
        let mut position = offset;
        let mut previous_x = left;
        for (index, character) in piece.char_indices() {
            let next = index + character.len_utf8();
            let next_x = left + ui.calc_text_size(&piece[..next])[0];
            if mouse_x < f32::midpoint(previous_x, next_x) {
                break;
            }
            position = offset + next;
            previous_x = next_x;
        }
        let element = if interactive {
            Element::Interactive
        } else {
            Element::Text
        };
        let over_selection = self.selected_in(offset..offset + piece.len()).is_some();
        self.offer(
            vertical,
            horizontal,
            self.at(position),
            element,
            over_selection,
        );
    }

    /// Notes an element without inner places between `left` and `right` on
    /// `line`, such as a chip or spaces: the mouse lands `before` or `after`
    /// it, whichever half it is in; `interactive` for links and chips, else
    /// it counts as text.
    pub(super) fn probe_atom(
        &mut self,
        left: f32,
        right: f32,
        line: LineBox,
        [before, after]: [TextPosition; 2],
        interactive: bool,
    ) {
        let Some((vertical, horizontal, mouse_x)) = self.distances(left, right, line) else {
            return;
        };
        let position = if mouse_x < f32::midpoint(left, right) {
            before
        } else {
            after
        };
        let element = if interactive {
            Element::Interactive
        } else {
            Element::Text
        };
        let over_selection = self
            .range
            .as_ref()
            .is_some_and(|range| range.start < after && before < range.end);
        self.offer(vertical, horizontal, position, element, over_selection);
    }

    /// Notes the room after a line's last element, from `left` to `right`:
    /// the mouse lands at `position`, the line's end.
    pub(super) fn probe_line_end(
        &mut self,
        left: f32,
        right: f32,
        line: LineBox,
        position: TextPosition,
    ) {
        if let Some((vertical, horizontal, _)) = self.distances(left, right.max(left), line) {
            self.offer(vertical, horizontal, position, Element::Room, false);
        }
    }

    /// The mouse's distances to the box `left..right` of `line` and its x,
    /// or `None` without a mouse or when an earlier element is at least as
    /// close.
    fn distances(&self, left: f32, right: f32, line: LineBox) -> Option<(f32, f32, f32)> {
        let [x, y] = self.probe?;
        let vertical = (line.top - y).max(y - line.bottom).max(0.0);
        let horizontal = (left - x).max(x - right).max(0.0);
        let is_closer = self
            .best
            .is_none_or(|best| (vertical, horizontal) < (best.vertical, best.horizontal));
        is_closer.then_some((vertical, horizontal, x))
    }

    /// Keeps a place if it is closer to the mouse than the best so far.
    fn offer(
        &mut self,
        vertical: f32,
        horizontal: f32,
        position: TextPosition,
        element: Element,
        over_selection: bool,
    ) {
        self.best = Some(Candidate {
            vertical,
            horizontal,
            position,
            element,
            over_selection,
        });
    }

    /// Applies the mouse of this frame to `selection` of `document` after
    /// drawing it (Q551–Q555): a left press in the text (`pressed_here`)
    /// places both ends (Shift moves only the focus), a drag past ImGui's
    /// threshold moves the focus, the release ends the gesture; a right
    /// click on the selection (`right_clicked_here`) opens the menu with
    /// "Copy" (P4). Sets the text cursor over plain text and while dragging
    /// (P3).
    pub(super) fn finish(
        self,
        ui: &Ui,
        document: &MarkdownDocument,
        selection: &mut TextSelection,
        (pressed_here, right_clicked_here): (bool, bool),
    ) {
        let Some(best) = self.best else {
            if !ui.is_mouse_down(MouseButton::Left) {
                selection.gesture = SelectionGesture::Idle;
            }
            return;
        };
        if pressed_here {
            // Shift+click extends the selection from its anchor (Q555).
            if !ui.io().key_shift {
                selection.anchor = best.position;
            }
            selection.focus = best.position;
            selection.gesture = SelectionGesture::Pressed;
        } else if selection.gesture != SelectionGesture::Idle {
            if !ui.is_mouse_down(MouseButton::Left) {
                selection.gesture = SelectionGesture::Idle;
            } else if ui.is_mouse_dragging(MouseButton::Left) {
                selection.gesture = SelectionGesture::Dragging;
                selection.focus = best.position;
            }
        }
        let is_dragging = selection.gesture == SelectionGesture::Dragging;
        if is_dragging || (best.is_under_mouse() && best.element == Element::Text) {
            ui.set_mouse_cursor(Some(MouseCursor::TextInput));
        }
        if right_clicked_here
            && best.is_under_mouse()
            && best.over_selection
            && let Some(text) = selection.text(document)
        {
            request_selection_menu(text);
        }
    }
}
