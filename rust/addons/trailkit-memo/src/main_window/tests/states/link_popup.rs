//! The link popup (M208): its snapshots and what confirming it writes.

use super::*;

/// The link button of the editor toolbar.
const LINK_BUTTON: [f32; 2] = [320.0, 64.0];

/// The link button at the narrowest window, in the toolbar's second row.
const MINIMUM_LINK_BUTTON: [f32; 2] = [36.0, 96.0];

/// The narrowest window.
const MINIMUM: [f32; 2] = [320.0, 360.0];

/// Windows virtual-key code of Enter.
const ENTER: u32 = 0x0D;

/// Windows virtual-key code of `K`.
const KEY_K: u32 = 0x4B;

/// A click on `button` with the mouse moved away afterwards.
fn open_with(button: [f32; 2]) -> Vec<FrameInput> {
    let mut steps = click(button);
    steps.extend(hover(NOWHERE));
    steps
}

/// The focused editor with `text` of the body selected.
fn editing_selecting(text: &str, length: Option<usize>) -> Fixture {
    let mut fixture = selected_editing_focused();
    if let Some(editor) = fixture.0.editor.as_mut() {
        let start = editor.body_text().find(text).unwrap_or(0);
        editor.select_in_body(start..start + length.unwrap_or(text.len()));
    }
    fixture
}

/// The focused editor with the cursor inside the sample's link.
fn editing_in_link() -> Fixture {
    editing_selecting("wiki]", Some(0))
}

/// The focused editor inside a link whose address memo does not open.
fn editing_in_unsupported_link() -> Fixture {
    let mut document = sample_document();
    if let Some(Node::Note(first)) = document.root.first_mut() {
        first.body = "[wiki](ftp://example.com)".to_owned();
    }
    let ui_state = selected_first(&document);
    let mut fixture = memo_window(document, ui_state);
    fixture
        .0
        .apply(&WindowAction::StartEditing, Language::English);
    if let Some(editor) = fixture.0.editor.as_mut() {
        editor.select_in_body(2..2);
    }
    fixture
}

/// Runs `steps` on `fixture` and returns the edited body.
fn body_after(mut fixture: Fixture, steps: &[FrameInput]) -> String {
    let mut harness = UiHarness::new(WIDE);
    let mut frame =
        |ui: &Ui| draw_in_language(ui, &mut fixture.0, WIDE, ThemeKind::Dark, Language::English);
    for _ in 0..2 {
        harness.step(&FrameInput::default(), &mut frame);
    }
    for step in steps {
        harness.step(step, &mut frame);
    }
    fixture
        .0
        .editor
        .as_ref()
        .map_or_else(String::new, |editor| editor.body_text().to_owned())
}

/// Idle frames, so a field that asked for the focus has it.
fn wait() -> Vec<FrameInput> {
    hover(NOWHERE).into_iter().cycle().take(4).collect()
}

/// A frame that presses `key`.
fn pressing(key: u32) -> FrameInput {
    FrameInput {
        mouse_position: NOWHERE,
        keys: vec![key],
        ..FrameInput::default()
    }
}

/// A frame that types `characters`.
fn typing_text(characters: &str) -> FrameInput {
    FrameInput {
        mouse_position: NOWHERE,
        characters: characters.to_owned(),
        ..FrameInput::default()
    }
}

#[test]
fn link_popup_states() {
    assert_state(
        "link-popup",
        WIDE,
        selected_editing_focused,
        &open_with(LINK_BUTTON),
    );
    assert_state(
        "link-popup-selection",
        WIDE,
        || editing_selecting("first", None),
        &open_with(LINK_BUTTON),
    );
    assert_state(
        "link-popup-existing",
        WIDE,
        editing_in_link,
        &open_with(LINK_BUTTON),
    );
    let mut confirm = open_with(LINK_BUTTON);
    confirm.extend(wait());
    confirm.push(pressing(ENTER));
    confirm.extend(wait());
    assert_state(
        "link-popup-error",
        WIDE,
        editing_in_unsupported_link,
        &confirm,
    );
    assert_state_in(
        Language::German,
        "min-de-link-popup",
        MINIMUM,
        selected_editing_focused,
        &open_with(MINIMUM_LINK_BUTTON),
    );
    assert_state_in(
        Language::German,
        "min-de-link-popup-existing",
        MINIMUM,
        editing_in_link,
        &open_with(MINIMUM_LINK_BUTTON),
    );
}

#[test]
fn ctrl_k_opens_the_link_popup() {
    let mut steps = wait();
    steps.push(FrameInput {
        control: true,
        ..pressing(KEY_K)
    });
    steps.extend(wait());
    assert_state("link-popup-ctrl-k", WIDE, selected_editing_focused, &steps);
}

#[test]
fn confirming_writes_a_link_around_the_selection() {
    let mut steps = open_with(LINK_BUTTON);
    steps.extend(wait());
    steps.push(typing_text("example.com"));
    steps.extend(wait());
    steps.push(pressing(ENTER));
    steps.extend(wait());
    let body = body_after(editing_selecting("first", None), &steps);
    assert!(
        body.ends_with("[first](https://example.com)."),
        "unexpected body: {body}"
    );
}
