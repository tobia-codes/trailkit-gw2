# In-game tests

Pending checks the maintainer runs in Guild Wars 2, one at a time
(user request 2026-11-13). **Agents:** at the start of every session,
if this list has tests, ask whether the user can test in game now; keep this list current with every
change that needs an in-game check; remove an item as soon as the user
confirms it (and only then), and note failures as bugs in the relevant plan. Every item
says how to get there click by click, what to look at, how it looked
before, how it must look now and links the approved design (R87 mockup).
In a session on the Mac or the phone (no game there, user request
2026-11-16), add the entry in the same commit as the change, so the test
is ready when the user is back at the PC.

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
**Since R124/R125 (2026-11-16):** brass became violet and the Dark red fill is
the darker `#B83A33` with a light label; read "brass" as violet and judge the
red against the [red fill mockup](https://claude.ai/artifact/E6E3L6RTBxPsPBuCNW75VK) (A1).

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

### 8 · Save error: "Saving failed" dialog (toast part OK 2026-11-17)

- **Design:** [mockup, section `error-toast`](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#error-toast) and [discard dialogs](https://claude.ai/artifact/3v1UcDmCNSJinm8iWZb5wr#discard-dialogs)
- **Get there:** in Windows Explorer open `<GW2>/addons/trailkit/memo/`,
  `notes.json` › Properties › tick **Read-only**. In game: select a note ›
  Edit › type a letter › **Save** › click another note.
  If nothing fails, read-only is not enough: report it.
- **Look at:** the dialog "Saving failed" (it appears when you leave the
  note while the save fails), button **"Discard"**.
- **Before:** grey button.
- **Now:** **red filled** "Discard" left, brass "Save again" right.
- **Clean up:** untick Read-only, click "Save again".

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

## Violet accent and calendar icons, branch `feature/violet-accent` (R124–R126, M193)

### 18 · New colors and reset icons

- **Design:** [mockup v4](https://claude.ai/artifact/GgYDB2fh1Lg1w3sd5jEaM5) (V3 Amethyst, red A1, Lucide A, chips).
- **Get there:** open memo in Dark, then in Light (Setup 4). Have one daily
  and one weekly note (right-click a note › Reset type).
- **Look at:** (a) **Edit** button, selection bar, check boxes, focus frame of
  the search field and a rename, search hits; (b) the list rows of the daily
  and weekly notes and the dropdown next to their title; (c) a chat-link chip
  of a waypoint, a skill and an achievement; (d) a delete question and a red
  "Discard" (test 6), hover and hold them.
- **Before:** brass accent; letters D/T and W in gold/violet boxes; tinted
  dropdown; copper waypoints, blue skills, violet achievements; salmon red.
- **Now:** (a) violet with white label, darker on hover and hold; (b) grey
  calendar icons (`1` daily, dots weekly) without box, tooltip with kind and
  countdown, plain dropdown; (c) waypoint blue, skill copper, achievement
  rosé; (d) deep red with light label, darker on hover and hold.

## Dependency patch updates, branch `chore/dependency-patch-updates` (R130, Q484)

### 19 · Memo still works after the updates

- **Design:** none (no visible change).
- **Get there:** deploy the branch (`cargo xtask deploy trailkit-memo`), start the game, open memo (Setup 2).
- **Look at:** open a note, edit and save it, use the search, delete and restore a note from the trash,
  set a daily reset on a note (dates and countdown use the updated `jiff`), then reload memo (Setup 5).
- **Before:** everything worked.
- **Now:** everything works the same; the note, the reset countdown and the trash entry survive the reload.

## Generated theme folder, branch `chore/generated-theme-folder` (R133)

### 20 · Theme looks unchanged

- **Design:** none (no visible change; the values were compared one to one).
- **Get there:** deploy the branch (`cargo xtask deploy trailkit-memo`), open memo in Dark, then in Light (Setup 4).
- **Look at:** window and pane backgrounds, borders of windows and popups, a tooltip and its shadow, the violet
  **Edit** button, a red "Discard" button and chat-link chips.

## trailkit-ui module groups, branch `chore/trailkit-ui-module-groups` (R134)

### 21 · Memo unchanged after the move

- **Design:** none (files moved, no code changed).
- **Get there:** deploy the branch (`cargo xtask deploy trailkit-memo`), open memo (Setup 2).
- **Look at:** fonts and icons load (title bar, list, buttons), a tooltip, the `⋯` menu, a toast (e.g. delete
  a note), the help window (menu › Help) with its topics, a chat-link chip and the Markdown preview of a note.
- **Also (fonts moved to `trailkit-nexus-bridge`, Q496):** with memo open, change the GW2 interface size
  (GW2 options › Graphics › Interface Size) and the memo text size (Ctrl+mouse wheel over memo): memo's text
  and icons grow and shrink with it, no `?` boxes appear. (Corrected 2026-11-17: memo follows Nexus' `FontUI`,
  which follows the GW2 interface size, not the Nexus font size slider.)
- **Before / now:** identical to `develop`; nothing may differ.
