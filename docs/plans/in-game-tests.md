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

## Test 25 — migrated notes and collapsed groups (M201–M203, 2026-11-18)

Steps 1–3, 5 and 6 passed (2026-11-18/19). **Step 4 failed (2026-11-19):** Ctrl+Z
never reaches memo (the game opened Contacts); retest after the `bugfix/`
branch for memo's shortcut keys:

4. Collapse a group, drag another note into it, click a list entry (no text
   cursor), press Ctrl+Z. **Expected:** the note goes back to its old place,
   its group opens and it is selected; the game opens nothing.

## Test 31 — right-click menu of links and chips (M210, 2026-11-18)

Approved design: https://claude.ai/artifact/T75AGbpSsUuT4K4yTwpSTs

Use a note with a web link, a `mailto:` link, an item or map point chip and a
coin chip, in the reading view.

1. Right-click the web link. **Expected:** Open link, Copy link address.
   Copy it and paste it somewhere: the address arrives, "Copied" appears.
2. Right-click the e-mail link. **Expected:** Write email, Copy email address;
   the copied text has no `mailto:`.
3. Right-click the item or map point chip. **Expected:** Copy chat link, Open
   in wiki. The wiki opens in the browser with a search for the code.
4. Right-click the coin chip. **Expected:** only Copy chat link.
5. With Nexus in German, **Im Wiki öffnen** opens the German wiki.
6. Open a menu and press Escape. **Expected:** only the menu closes.

## Test 32 — even popup spacing and e-mail links (R144, M211, 2026-11-18)

1. Open any right-click menu and the reset type dropdown, hover the first and
   last row. **Before:** 7 px at the sides, 5 px above and below. **Now:** the
   highlight keeps the same 8 px (`space.sm`) from all four edges.
2. Type `bogus@example.com` in a note and save it. **Expected:** it is a link
   with the mail icon and no tooltip. A click opens the mail program; a right
   click offers Write email and Copy email address.
3. Type `a@b` and `` `bogus@example.com` ``. **Expected:** both stay plain text.

## Test 33 — shared dialog and backdrop (R145, 2026-11-19)

No visible design change; every dialog and the window menu now use the
shared `Dialog` and `Backdrop` of trailkit-ui.

1. Edit a note, then click another note: the **unsaved changes** dialog
   appears. **Expected:** it looks as before (title, close icon top right,
   question, buttons) and only the memo window is dimmed. Close it once each
   with the close icon, Escape and a click on the dimmed area: every time you
   keep editing. Enter saves.
2. In the editor, press Ctrl+K. **Expected:** the **Insert link** dialog
   looks as before; the close icon, Escape and a click on the dimmed area
   cancel it, Insert still inserts.
3. Delete a note from the trash to get the delete question. **Expected:** as
   before; clicking the dimmed area cancels.
4. Open the window menu (title bar menu button). **Expected:** the dimmed
   backdrop over the content looks as before; a click on a row runs it, a
   click on the dimmed strip beside the menu closes it.

## Test 34 — text size percent counts from the default (M213, 2026-11-19)

No design change besides the number in the tooltip. **Ctrl+0 failed (2026-11-19):**
it does nothing in game (same cause as test 25 step 4); retest after that bugfix.

1. Hover memo, press **Ctrl+0**. **Expected:** the tooltip says
   **Textgröße: 100 %** (before: 112,5 %); the text is the default size.
2. Turn the mouse wheel one step up, then two steps down with **Ctrl** held.
   **Expected:** 106 %, then 94 %.
3. Press **Ctrl+0** again. **Expected:** 100 %.

## Test 35 — one underline per link (R149, 2026-11-19)

Approved design: https://claude.ai/artifact/BYrmZuZwPSeTaiw1idKVop

1. Put `Vorher im [das ist ein tester](https://wiki.guildwars2.com) nachsehen.`
   into a note and save. **Before:** each word had its own underline with gaps.
   **Now:** one line runs under `das ist ein tester`, through the spaces; the
   icon in front stays without line, `nachsehen.` has none.
2. Drag the window narrow until the link wraps. **Expected:** the line ends at
   the line end and starts again on the next line.
3. A link with a **bold** part, e.g. `[ein **fetter** Link](https://a.de)`.
   **Expected:** one line under all of it.
