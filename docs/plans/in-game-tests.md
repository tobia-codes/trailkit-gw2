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

No visible design change; checks that existing notes survive the
`notes.json` version 2 migration. Before deploying, note which groups are
collapsed and which notes are daily/weekly reset notes.

1. Deploy (Setup step 1), open memo. **Expected:** all notes, groups and the
   trash are there. **Once only:** every group is expanded (the old expand
   state is not carried over, M201).
2. Hover a reset note's calendar badge, and open the dropdown behind its
   title. **Expected:** daily/weekly notes keep their kind; a note without
   a reset shows **No reset** (Deutsch: **Kein Reset**, M204). Check a box in a daily note: it stays checked
   until the next daily reset, as before.
3. Collapse a group (click its arrow), reload memo (Setup step 5).
   **Expected:** the group is still collapsed. Before: same behavior, but
   it was stored in `notes.json`; now `ui_state.json` holds
   `collapsed_groups`.
4. With that group collapsed: create a note in it (right-click the group,
   new note in its context menu), drag another note into it, then press
   Ctrl+Z.
   **Expected:** each time the group opens to show the entry, as before.
5. Delete a note inside the collapsed group, open the trash, restore it.
   **Expected:** the group opens and shows the restored note.
6. In the memo folder (`addons/trailkit/memo/`), check
   `backups/notes.v1.bak.json` exists (copy of the old file) and
   `notes.json` shows `"schema_version": 2` and `"reset_schedule"` instead of
   `"kind"`.

## Test 26 — "No reset" label (M204, 2026-11-18)

Approved design: https://claude.ai/artifact/HKKD3uQMPHy3cSyXxLgUZz

1. Deploy (Setup step 1), open memo and select a note that has no reset.
   **Before:** the box next to its title said "Normal". **Now:** it says
   **No reset**.
2. Open that box. **Expected:** No reset (checked), Daily reset, Weekly
   reset.
3. Right-click the note in the list › **Reset type**. **Expected:** the same
   three entries, No reset checked.
4. Set the Nexus language to Deutsch, drag the window as narrow as it goes.
   **Expected:** the box says **Kein Reset**, the list shows Kein Reset,
   Täglicher Reset and Wöchentlicher Reset, nothing is cut off.

## Test 27 — dropping onto groups (M205, 2026-11-18)

Approved design: https://claude.ai/artifact/MsEL3kqnjuRrTA3eSehcWb

Needs a group with at least two notes and a note outside it. Ctrl+Z undoes
each move.

1. Drag the outside note onto the **middle** of the open group's row.
   **Before:** the upper half placed it above the group. **Now:** the whole
   row gets a frame; on release the note is the group's last entry.
2. Drag it to the group row's **top edge**. **Expected:** a line above the
   group; the note lands above the group.
3. Drag it to the row's **bottom edge**. **Expected:** an indented line below
   the title; the note becomes the group's first entry.
4. Drag it between two entries of the group. **Expected:** an indented line
   there, and the note lands exactly there.
5. Collapse the group and repeat on its row. **Expected:** top edge = above,
   middle = frame and into the group (it opens), bottom edge = below it.
6. Rest on the middle of the collapsed group for about a second, without
   releasing. **Expected:** it opens, the frame stays; on release the note is
   its last entry.

## Test 28 — Backspace removes a list marker (M206, 2026-11-18)

No visible design change. Open a note and click **Edit**.

1. Type `- [ ] ` on a new line and press Backspace once. **Before:** one
   character went away. **Now:** the whole marker is gone.
2. Type `- [ ] Text`, put the cursor directly before `Text`, press Backspace.
   **Expected:** only `Text` stays; Ctrl+Z brings the marker back.
3. Repeat with `- Text`, `1. Text` and an indented `  - [ ] Text`.
   **Expected:** only `Text` stays, without indentation.
4. Put the cursor inside or at the end of `Text`, press Backspace.
   **Expected:** one character goes away, as before.

## Test 29 — deleting a link at once (M207, 2026-11-18)

No visible design change. Open a note, click **Edit** and type
`Vorher [Wiki](https://wiki.guildwars2.com) nachher`.

1. Put the cursor directly behind `)` and press Backspace. **Before:** only
   `)` went away. **Now:** the whole link is gone; Ctrl+Z brings it back.
2. Put the cursor directly before `[` and press Delete. **Expected:** the whole
   link is gone; Ctrl+Z brings it back.
3. Put the cursor between `]` and `(`, press Backspace. **Expected:** the whole
   link is gone; Ctrl+Z brings it back.
4. Put the cursor inside `Wiki`, then inside the address, and press Backspace.
   **Expected:** one character goes away each time, the rest stays a link.
5. Type a bare `https://wiki.guildwars2.com`, press Backspace at its end.
   **Expected:** one character goes away.

## Test 30 — link dialog and pasted addresses (M208, M209, 2026-11-18)

Approved design: https://claude.ai/artifact/SgWHJ1mMKSTArQMEVaqACm

Open a note and click **Edit**.

1. With nothing selected click the **link button**. **Before:** `[](https://)`
   appeared in the text. **Now:** a dialog **Insert link**, centered over the dimmed window, with **Text (optional)** and
   **Address** opens, the cursor is in Text, Insert is active. Press Enter: a red message says an
   address is needed.
2. Press Tab, type `wiki.guildwars2.com`, press Enter. **Expected:** the dialog
   closes and `https://wiki.guildwars2.com` is in the text.
3. Select a word, press **Ctrl+K**. **Expected:** the word is in Text and the
   cursor in Address. Type an address, press Enter: the word is now a link.
4. Copy an `https://` address, press Ctrl+K elsewhere. **Expected:** it is
   already in Address. Escape closes the dialog, the text stays unchanged.
5. Put the cursor in a link, press Ctrl+K. **Expected:** both values are there,
   with Save and an icon at the left. Change the text and save; then open it
   again and click the icon: the link is gone, its text stays.
6. Enter `sdsdsd`, press Enter, then try `ftp://example.com` and click into Text. **Expected:** a red message
   with an icon below Address; the button stays active and does not insert.
7. Save the note. **Expected:** the pasted `https://…` address from step 2 is
   a clickable link in the reading view, without a tooltip. A link with its
   own text still shows its address as a tooltip.
8. Set Nexus to Deutsch and drag the window narrow. **Expected:** the dialog
   fits and shows **Text (optional)**, **Adresse**, **Abbrechen**, **Einfügen**.
9. Open the dialog and click the dimmed area beside it, then open it again
   and click the **x** at its top right. **Expected:** it closes each time
   and the text stays unchanged. The delete and unsaved-changes dialogs have
   the same **x**.

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
