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

## Test 36 — dialogs have a maximum width (R153, 2026-11-19)

Approved design: https://claude.ai/artifact/VXdbvqSVZhTnbjrMPKRbyN (round 2,
20 font sizes).

1. Make the memo window wide (about 1000 px or more). Right-click a group
   with notes › *Delete*. **Expected:** the card is about 400 px wide and the
   question wraps into three short lines (before: two long lines, about
   480 px).
2. Open a note in the editor, type a letter, click another note.
   **Expected:** "Unsaved changes" keeps its three buttons in one row.
3. Ctrl+K in the editor. **Expected:** the link dialog's fields are as wide
   as the delete question.
4. Drag the window as narrow as it goes and open the question again.
   **Expected:** unchanged: the card fits the window, buttons stack when they
   do not fit.
