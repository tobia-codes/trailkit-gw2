# trailkit-memo

> Pre-release — notes can be created, renamed, viewed and edited, and reset daily or weekly.

## Opening the window

Click the **notes icon** (sheet with pencil) in the Nexus quick access bar, or use the keybind **"Open/close notes"**. The keybind is unbound by default: set it under *Nexus › Options › Keybinds › trailkit-memo*.

The window hides during loading screens, cutscenes and character select and reappears afterwards.

## Your notes

The window shows the note list on the left and the selected note on the right. The arrow left of the title collapses the window to its title bar (click it again to expand), the **×** on the right closes it. Drag the gap between both sides to change their widths (a line appears when you point at it). The note list keeps its width when you resize the window; the note gets the extra room.

- **Sidebar button** (panel icon, top left of the note): hides the note list so the note gets the full width; click it again to show the list. memo remembers your choice.
- **Narrow window:** the note fills the window and the sidebar button opens the note list over it. Choose a note, click the dimmed note or the sidebar button again to close the list. While you edit with the preview open, memo switches earlier, so text and preview keep their room.

- **New note / new group:** the two buttons above the list. The new entry goes into the selected group (or the group of the selected note), at the end. Type a name right away and press Enter.
- **Right-click** a group or an empty spot of the list for the same actions: on a group the new entry goes into it, on an empty spot at the end of the list.
- **Rename:** double-click an entry, or right-click › *Rename*. Enter or clicking elsewhere keeps the new name, Escape cancels.
- **Reorder:** drag a note or group to a new place. A line shows where it lands; on the lower half of an open group it becomes the group's first entry, below the last entry it goes to the end of the list. Drop on the middle of a closed group to put it at its end; rest there a moment and the group opens. A red "no entry" sign means it cannot go there (a group cannot go into itself). Near the top or bottom edge the list scrolls. Escape cancels.
- **Move to:** right-click › *Move to* › *Top level* or a group moves the entry to the end of it, without dragging.
- **Groups** have bold names and show how many notes they contain on the right.
- **Badges** at the right end of a row: **D** marks a daily note (reset at 00:00 UTC), **W** a weekly note (reset on Mondays at 07:30 UTC). Hover a badge to see how long it is until the next reset.

## Searching

Type into the **Search** field at the top of the note list (or press **Ctrl+F**). The list shows only notes that contain every word you typed — in their title or text, upper or lower case alike. Umlauts count as their own letters: "ä" finds "Ä", but not "a".

- Groups with hits open by themselves; their number shows how many notes match. A group whose name matches shows everything inside.
- Hits are marked in the titles and in the open note, which jumps to the first hit. Notes that match only through their text show the number of hits at the end of the row.
- Link texts and chat-link codes are searched, link addresses are not. A chip whose code matches gets a frame.
- The open note always stays in the list, even if it no longer matches after an edit.
- You cannot drag entries while searching; clear the search first. Renaming, the right-click menu and *Move to* work as usual. A new note or group clears the search.
- **×** in the field or **Escape** (while you type in the field) clears the search; a second Escape closes the window. The search stays when you close and reopen the window, until the game restarts.

## Daily and weekly notes

Choose the kind of a note in the box next to its title (*Normal*, *Daily reset*, *Weekly reset*), or right-click it in the list › *Reset type*. At the game's daily or weekly reset all checkboxes in such a note are unticked again — handy for dailies and weekly clears. Hover the badge in the list to see how long it is until the next reset.

- A note you switch to daily or weekly keeps its ticks until the next reset.
- If the game was closed at reset time, memo resets the note the next time you open the window — once, however many resets you missed.
- While you edit a note (or it has a draft), its reset waits until you save or discard; your text is never changed under your hands.
- **Checkboxes** in a note can be ticked directly.
- **Links** open in your browser (only `http://` and `https://`); hover a link to see where it goes.
- **Chat links** from the game (`[&…]`, e.g. a waypoint or an item) show as chips with an icon, the kind and the id; coins show their amount. The chip color shows the group: copper for places, blue for skills, traits and builds, green for items, violet for achievements, NPC texts and players. Click a chip to copy its code, then paste it into the game chat. Hover it to see the code. A chip with **!** has a broken code; the tooltip says why. Names (instead of ids) will come with trailkit-core.

Long names are shortened with "…" — hover them to see the full name. In a long note the title stays at the top while you scroll.

**Text size:** hold **Ctrl** and turn the mouse wheel over the window to make all text larger or smaller; **Ctrl+0** goes back to the default. The current size shows next to the mouse in percent. memo remembers the size; it builds on the UI font size set in Nexus.

## Editing a note

Click **Edit** at the top right of a note, or double-click the note. You edit the title and the text in markdown; the buttons in the bar above (in groups, like in Word) insert headings (H1, H2 — clicking the other level switches it, the same level removes it), **bold**, *italic*, lists, checkboxes and links (also **Ctrl+B** and **Ctrl+I**); clicking a button again removes the formatting (also links). Buttons of the formatting at the cursor light up. The event button needs trailkit-core.

- **Chat links:** paste a code (`[&…]`) into the text and it becomes a chip. The cursor jumps over a chip and Backspace deletes it whole; to change it, delete it and paste the new code.
- The **eye** button shows a preview of the formatted note next to the text; drag the gap between them to change the widths. memo remembers both.
- **Save** (or **Ctrl+Enter** / **Ctrl+S**) saves your changes, **Cancel** throws them away — memo asks first if you changed something.
- If you choose another note, create one or close the window while you have unsaved changes, memo asks whether to save or discard them. (Set `"unsaved_changes": "save_automatically"` in `settings.json` to save without asking; a switch follows in the settings window.)
- **Escape** closes the window; with unsaved changes memo asks first.
- If the game or the addon closes while you are editing, your changes are kept as a draft. Next time you open the note, choose **Restore** to continue or **Discard** to drop them. Drafts are deleted automatically after 30 days.

## Where your notes are stored

Your notes are saved automatically a moment after every change, in `addons/trailkit/memo/` inside your Guild Wars 2 folder (`notes.json`, `settings.json`, and `drafts.json` while a draft exists).

- If the notes file is damaged, memo starts with an empty list and tells you. The damaged file is kept in the `backups` folder next to it, so nothing is lost.
- If your notes were saved by a newer version of memo, memo leaves the file untouched and asks you to update the addon.

## Problems?

Errors are written to the Nexus log (`addons/Nexus/Nexus.log` in your Guild Wars 2 folder). Please attach the relevant lines when you open an issue on GitHub.
