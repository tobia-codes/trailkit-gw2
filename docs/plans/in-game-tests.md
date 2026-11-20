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

## Test 38 — no tooltips through the window menu (2026-11-20)

No design change; a fix. Before, a button under the open window menu still
showed its tooltip and hover fill (snapshot `memo-state-window-menu-hover-selected`
even captured the "New note" tooltip).

1. Open the window menu (the ☰ at the top left). Move the mouse over its
   entries, especially over *Notes* and *Search*, where the buttons above the
   note list lie underneath. **Expected:** only the menu entry under the mouse
   is highlighted; no tooltip such as "New note" appears.
2. Close the menu and hover the "New note" button. **Expected:** its tooltip
   shows as before.
3. Right-click a note › hover *Reset* so the submenu opens, then move onto a
   submenu entry. **Expected:** no row of the main menu behind the submenu
   lights up.
