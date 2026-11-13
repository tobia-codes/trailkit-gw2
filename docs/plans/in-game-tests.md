# In-game tests

Pending checks the maintainer runs in Guild Wars 2, one at a time
(user request 2026-11-13). **Agents:** at the start of every session ask
whether the user can test in game now; keep this list current with every
change that needs an in-game check; remove an item only after the user
confirms it, and note failures as bugs in the relevant plan. Every item
says how to get there click by click, what to look at, how it looked
before, how it must look now and links the approved design (R87 mockup).

## Setup (once per test session)

1. In WSL, on an up-to-date `develop`: `cargo xtask deploy trailkit-memo`
   (a normal build; it also replaces the `fault-injection` debug DLL).
2. In game, open memo: the **notes icon** (sheet with pencil) in the Nexus
   quick access bar.
3. Test data: make sure the list has a group with at least one note in it
   and one more note outside it. The two buttons **above the list** create a
   note (sheet with plus) and a group (folder with plus); type a name, Enter.
4. Theme: switch Dark/Light in Nexus where a test says so. German: set the
   Nexus language to Deutsch; "narrow" means dragging the window's
   bottom-right corner as far left as it goes.
5. Reload memo (tests 7, 10): Nexus menu › Addons › trailkit-memo › unload,
   then load again.

## Button cleanup, PR #124 (R105, R106, R107, M187, merged before the in-game test at the user's request)

"Red filled" means a red button with the label in the window color, like
the brass "Save" button but red. Approved design for all tests below:
[button cleanup mockup](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr) (each test links its section).

### 1 · "Empty" in the trash

- **Design:** [mockup, section `trash-buttons`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#trash-buttons)
- **Get there:** select the extra note › `⋯` (right of "Edit", top right
  of the note) › Delete. Then the **menu button** (top left of the title
  bar) › **Trash**.
- **Look at:** the **"Empty"** button at the top of the trash list; hover it.
- **Before:** grey button with red text.
- **Now:** **red filled** button; hover makes it clearly darker (test 10). Click
  it: it still asks first; click **Cancel**.

### 2 · "Delete permanently" icon in the trash

- **Design:** [mockup, section `trash-buttons`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#trash-buttons)
- **Get there:** in the trash (test 1), click the entry in the list.
- **Look at:** the **trash icon** top right, left of brass "Restore"; hover it.
- **Before:** red icon without background.
- **Now:** **red filled** square with the icon in the window color; hover
  makes it clearly darker; the tooltip "Delete permanently" is readable; clicking still asks first.

### 3 · Confirm button of the delete questions (unchanged)

- **Design:** [mockup, section `delete-dialog`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#delete-dialog)
- **Get there:** (a) in the trash click "Empty"; (b) back arrow to leave the
  trash, right-click the group that holds a note › Delete.
- **Look at:** the confirm button on the right of the question.
- **Before and now (unchanged):** **red filled** "Delete permanently" /
  "Delete", "Cancel" left of it. Escape cancels.

### 4 · "Delete" in the note menu

- **Design:** [mockup, section `menu-delete`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#menu-delete)
- **Get there:** select a note › `⋯` right of "Edit".
- **Look at:** the last entry **"Delete"**; hover it.
- **Before:** white text and icon like the other entries.
- **Now:** **red** text and trash icon; hover gives the normal grey row
  highlight.

### 5 · "Delete" in the right-click menu

- **Design:** [mockup, section `menu-delete`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#menu-delete)
- **Get there:** right-click a note in the list; then right-click the group.
  Then select the group's note › **Edit** › right-click the group.
- **Look at:** the entry **"Delete"**.
- **Before:** white text and icon.
- **Now:** **red** text and icon. While editing the group's note it is
  **grey** with a reason tooltip (as before). Leave the editor with Cancel.

### 6 · "Discard" in "Unsaved changes"

- **Design:** [mockup, section `discard-dialogs`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#discard-dialogs)
- **Get there:** select a note › **Edit** › type a letter in the text ›
  click **another note** in the list.
- **Look at:** the **"Discard"** button on the left of the dialog.
- **Before:** grey button.
- **Now:** **red filled**; "Keep editing" and brass "Save" stay on the
  right. In German and narrow: buttons stack, brass on top, red
  "Verwerfen" last. Click "Keep editing", then Cancel.

### 7 · "Discard" on the draft card

- **Design:** [mockup, section `draft-card`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#draft-card)
- **Get there:** select a note › **Edit** › type a letter › reload memo
  (Setup 5) without saving › open memo › select that note.
- **Look at:** the yellow card above the note, button **"Discard"**.
- **Before:** grey button left of brass "Restore".
- **Now:** **red filled** button left of brass "Restore". Narrow:
  "Restore" on top, red "Discard" below. Click "Discard" to clean up.

### 8 · Save error: "Try again" toast and "Saving failed" dialog

- **Design:** [mockup, section `error-toast`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#error-toast) and [discard dialogs](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#discard-dialogs)
- **Get there:** in Windows Explorer open `<GW2>/addons/trailkit/memo/`,
  `notes.json` › Properties › tick **Read-only**. In game: select a note ›
  Edit › type a letter › **Save** › click another note.
  If nothing fails, read-only is not enough: report it.
- **Look at:** (a) the red toast at the bottom, button **"Try again"**;
  (b) the dialog "Saving failed", button **"Discard"**.
- **Before:** (a) darker filled button without frame; (b) grey button.
- **Now:** (a) **normal grey button with frame**, like "Cancel";
  (b) **red filled** "Discard" left, brass "Save again" right.
- **Clean up:** untick Read-only, click "Save again".

### 9 · Buttons of the empty memo

- **Design:** [mockup, section `empty-state`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#empty-state)
- **Get there:** close the game, rename `<GW2>/addons/trailkit/memo/notes.json`
  to `notes.json.bak`, start the game, open memo. Afterwards close the game
  and rename it back.
- **Look at:** the two buttons under "No notes yet".
- **Before:** brass "Create your first note" **left**, "New group" right.
- **Now:** "New group" **left**, brass "Create your first note" **right**.

### 10 · Hover and pressed colors of brass and red buttons (R107)

- **Design:** [mockup, section `proposal`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#proposal)
- **Get there:** brass: select a note, hover **Edit** (top right), then hold
  the mouse button down on it and drag away before releasing. Red: Trash ›
  hover **Empty**, hold it down, drag away. In Dark and in Light.
- **Look at:** the fill of the button while hovered and while held.
- **Before:** brass got **lighter** on hover in Dark; red got duller
  (transparent) and in Light hard to read.
- **Now:** both get **clearly darker** on hover and a bit darker again while
  held, in Dark and Light; the label stays easy to read. The brass line of
  the splitter between list and note also turns darker brass on hover.

## Branch `feature/crash-protection` (R104, merged)

### 11 · Draft rescue after a panic

- **Get there:** `cargo xtask deploy trailkit-memo --features fault-injection`;
  in Nexus › Options › Keybinds bind `trailkit.memo.keybind.fault_injection`;
  select a note › Edit › type a letter › press the key.
- **Look at:** the help window, then after reloading memo (Setup 5) the note,
  then the newest log in `<GW2>/addons/trailkit/logs/memo/`.
- **Before:** a panic could crash the game.
- **Now:** memo stops and opens the help on "Report a problem"; after the
  reload the typed text is back as a **draft** (yellow card, test 7); the
  log contains **`draft=kept`**. Then deploy a normal build again (Setup 1).
