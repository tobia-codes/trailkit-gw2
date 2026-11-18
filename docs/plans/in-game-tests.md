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
