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
4. Theme: memo always draws **Dark** in game; the theme choice comes with the
   settings window (M28 step 8), so Light is checked only by the snapshot tests
   (corrected 2026-11-17). German: set the
   Nexus language to Deutsch; "narrow" means dragging the window's
   bottom-right corner as far left as it goes.
5. Reload memo: Nexus menu › Addons › trailkit-memo › unload,
   then load again.

## Tests

37. **Select and copy text in the reading view (M217, branch
    `feature/read-view-text-selection`, mock
    https://claude.ai/artifact/1FeMLueyes6L11SiUyFbgi).** Deploy the branch
    (setup step 1 on it). Open a note with a checkbox list, a link and a chip
    (e.g. paste `[&AvoJTQAA]` into a note). Before: dragging in the note
    moved the whole window and nothing could be selected.
    1. Drag with the left mouse button from the middle of a line over two
       more lines: a violet fill marks the text (as in the editor), line by
       line, with a narrow rest at each line end; the window stays put.
       Over text the mouse shows the text cursor, over links and chips the
       hand (check whether GW2 really shows the text cursor at all).
    2. Press **Ctrl+C** and paste into the game chat or a text editor: the
       text as shown, `[ ] ` before an unticked item whose start was
       selected, link texts only, the chip as its code. The "Copied" message
       pops up at the bottom, as after a chip click. The game must not react
       to Ctrl+C.
    3. Right-click into the selection: a menu with only **Copy**; on the
       chip inside the selection: Copy, a divider, then Copy chat link /
       Open in wiki. Choosing Copy shows "Copied" at the bottom too.
    4. Start a drag on the chip and on the link: they select and do not
       copy or open; a plain click still copies the chip (with "Copied") and
       opens the link; a plain click on the checkbox still ticks it.
    5. **Shift+click** further down extends the selection, **Ctrl+A**
       selects the whole note; a click into the note, another note, **Edit**
       or a new search removes it. A double-click still opens the editor.
    6. Dragging on the note's toolbar or empty space in the note pane does
       not move the window; the title bar still does.
    7. Repeat 1 and 3 with Nexus in Deutsch: the entry says **Kopieren**.
