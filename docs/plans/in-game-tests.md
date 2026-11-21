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

38. **Trashed groups as a tree (M218, branch `feature/trash-group-tree`,
    mock https://claude.ai/artifact/3F7P7A1uxLhVhMGjwZ6b54).** Deploy the
    branch. Make a group with three notes, delete one note on its own, then
    the group. Open the trash. Before: a group looked like a note (bold
    title and count only) and could only come back whole.
    1. The group has an arrow; the separately deleted note says
       "from “<group>” · 30 days left" below its title.
    2. Click the group's row: it is shown on the right and opens at once;
       its notes follow indented with the guide line, without days left.
       Click again: it closes. Leave the trash and open it again: closed.
    3. Click a note inside: it shows on the right with "In the trashed
       group “<group>” · …"; text can be selected and copied.
    4. **Restore** that note: the group is back in the note list at its old
       place with only this note; the trash keeps the rest. Restore the
       separately deleted note: it joins the same group. Restore the rest:
       everything is in its old order, no second group.
    5. Delete a note inside a trashed group for good (red button): it asks
       first, then only that note is gone.
    6. The note list looks and behaves as before (open, close, drag a group:
       it and its notes fade, search).
    7. Repeat 1–3 with Nexus in Deutsch at the narrowest window.
39. **One search everywhere and "Search notes" (BL30, BL31, M219, branch
    `feature/shared-search`, built on `feature/trash-group-tree`: test 38
    first; mocks https://claude.ai/artifact/DQRhxQtb4zyUxKB1hBtEVy v9 and
    https://claude.ai/artifact/6ijSByZw8m9PS2hkPXcm5i).** Deploy the
    branch. Test data: two notes that hold "ore" twice and once, one note
    without it, one note in a group with "ore", and a deleted group with a
    note holding "mithril". Before: the list search had no count or arrows,
    the find bar had its own close `x`, the trash and the help had no search,
    and the menu said "Search".
    1. **Menu:** ☰ shows **Search notes** (Deutsch: **Notizen durchsuchen**,
       the menu gets a bit wider at the narrowest window, nothing overlaps).
       Open the trash, ☰ › Search notes: back to the notes with the search.
    2. **Note list, empty field:** only the field, no count row; the `x`
       closes the search. A divider sits under the field, above the two
       buttons (none under the buttons while the search shows).
    3. Type "ore": "1 of 4" with ▲▼ below the field; the open note marks its
       current hit filled, the others framed. ▼/Enter: 2 of 4 in the same
       note, then the next note opens at its hit (groups collapsed in the
       search open). After 4 of 4 ▼ goes to 1 of 4; ▲ from 1 goes to 4 (the
       last hit of the last note). Shift+Enter goes back.
    4. A note with exactly one hit, a note matching only by title (one stop,
       nothing filled in the text), "zzz" (no count row, the list says
       "No notes for …"), Escape in the field clears it, a second Escape
       closes the window.
    5. **Find bar (Ctrl+F):** no separate close `x`; the field's `x` clears
       text, on the empty field it closes the bar. Count and arrows beside
       the field; drag the window narrow: they move below the field like the
       list's count row. Ctrl+F again and Escape still close it.
    6. **Trash:** the field "Search trash …" sits under "Trash · Empty",
       always shown, no `x` while empty. "mithril" shows the deleted group
       open with its matching note, counts at the row ends, hits marked in
       the titles; ▼ selects the note inside the group and the preview marks
       the hit. Close the group's arrow during the search, step again: it
       opens. Ctrl+F finds in the preview. Leave and reopen the trash: the
       search is empty. Shift twice in the trash opens the note search.
    7. **Help window (?):** "Search help …" above the topics with a divider
       below. "trash": only matching topics, opened to their sections with
       hits, the topic's count at the row end, hits marked in the titles;
       click a topic: it opens at its first hit; click a section: at that
       section's first hit. ▼ steps through all hits and topics, wrapping.
       Search a word from the privacy topic: it is found. "zzz": "No
       results". Ctrl+F (focus in the help) opens the find bar below the
       topic's head; F3 steps. Shift twice in the help brings the memo
       window forward with the note search focused.
    8. Repeat 2, 3, 6 and 7 with Nexus in Deutsch at the narrowest window.
    9. Known gap to judge: Escape in the help window still closes the whole
       window, also while its search field has text (the main window clears
       the field first).
