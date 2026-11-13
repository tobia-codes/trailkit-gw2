# trailkit-memo

## Getting started

memo keeps your notes and checklists in a window inside the game.

### Opening the window

Click the **notes icon** (sheet with pencil) in the Nexus quick access bar, or use the keybind **"Open/close notes"**. The keybind is unbound by default: set it under *Nexus › Options › Keybinds › trailkit-memo*.

The window hides during loading screens, cutscenes and character select and reappears afterwards.

### The window

The window shows the note list on the left and the selected note on the right. Drag the gap between both sides to change their widths (a line appears when you point at it). The note list keeps its width when you resize the window; the note gets the extra room.

The title bar holds these buttons:

- **Menu button**, top left: opens the menu. Click the dimmed area or the button again to close it.
- **?** on the right: opens or closes the help window.
- **Arrow** on the right: collapses the window to its title bar (click it again to expand); **×** closes it.

A highlighted button means that what it switches is shown. Long names are shortened with "…" — hover them to see the full name. In a long note the title stays at the top while you scroll; in a long list the search field and the buttons stay at the top.

### The menu and views

The menu takes you to the different parts of memo:

- **Notes** always brings you back to the regular page: it leaves the trash, closes the search, ends the compact view and shows the note list again.
- **Search** shows the search field. **Trash** shows the trash.
- **Hide note list** gives the note the full width; **Show note list** brings the list back. memo remembers your choice.
- **Compact view** (arrows button, top left of the note): hides the toolbar with *Edit* and the kind box, so the note gets more room. The button behind the title ends it again. You can still choose other notes and tick checkboxes; editing a note (also by double-click) or creating a new one ends the compact view. memo remembers it. Together with a hidden note list only the note is left.
- **Small window:** the window cannot get narrower than the note list and the note need side by side, so the list never disappears by itself; it only gets narrower. Text at a larger size makes this minimum wider.
- **Text size:** hold **Ctrl** and turn the mouse wheel over the window to make all text larger or smaller; **Ctrl+0** goes back to the default. The current size shows next to the mouse in percent. memo remembers the size; it builds on the UI font size set in Nexus.

### Help and problems

The **?** in the title bar opens this help window; click it again to close it. **Open help** in memo's tab under *Nexus › Options* opens it too, or brings it to the front. **Escape** closes it.

The window has two tabs: **Help** with these topics, and **Report a problem**, which saves your log as a text file and opens the problem form on GitHub. Until the game restarts, the window opens again on the tab and topic you last looked at.

If memo runs into an error, it stops instead of crashing the game: your unsaved text is kept as a draft, Nexus shows a short message and this window opens on **Report a problem**. Until you reload memo in the Nexus addon list or restart the game, the keybind and the quick access icon open this window.

## Notes and groups

Your notes sit in a list that you can sort and organize in groups.

### Creating and renaming

- **New note / new group:** the two buttons above the list. The new entry goes into the selected group (or the group of the selected note), at the end. Type a name right away and press Enter.
- **Right-click** a group or an empty spot of the list for the same actions: on a group the new entry goes into it, on an empty spot at the end of the list.
- **Rename:** double-click an entry, or right-click › *Rename*. Enter or clicking elsewhere keeps the new name, Escape cancels.
- **Note menu:** the button with three dots next to *Edit* offers *Rename*, *Move to*, *Reset type* and *Delete* for the open note. *Rename* opens the editor with the title selected; finish with *Save*.

### Sorting and moving

- **Reorder:** drag a note or group to a new place. A line shows where it lands; on the lower half of an open group it becomes the group's first entry, below the last entry it goes to the end of the list. Drop on the middle of a closed group to put it at its end; rest there a moment and the group opens. A red "no entry" sign means it cannot go there (a group cannot go into itself). Near the top or bottom edge the list scrolls. Escape cancels.
- **Move to:** right-click › *Move to* › *Top level* or a group moves the entry to the end of it, without dragging.

### Groups

Groups have bold names and show how many notes they contain on the right. Click anywhere on a group's row to open or close it; the note you are reading stays on the right.

### Deleting

Right-click › *Delete*, or select the entry and press **Delete** while the list has the focus. The entry goes to the trash; a group with notes asks first. The note you are editing cannot be deleted until you save or cancel. How to get it back is under *Trash and undo*.

## Editing notes

You write notes in markdown, with buttons for the formatting.

### The editor

Click **Edit** at the top right of a note, or double-click the note. You edit the title and the text in markdown; the buttons in the bar above (in groups, like in Word) insert headings (H1, H2 — clicking the other level switches it, the same level removes it), **bold**, *italic*, lists, checkboxes and links (also **Ctrl+B** and **Ctrl+I**); clicking a button again removes the formatting (also links). Buttons of the formatting at the cursor light up. The event button needs trailkit-core.

- The **eye** button shows a preview of the formatted note next to the text; drag the gap between them to change the widths. memo remembers both.
- **While you edit** the note list makes way for the editor and comes back when you save or cancel. If you had hidden it yourself, it stays hidden. Click into the game to use the chat or your keys; click into the text to go on writing.

### Checkboxes and links

- **Checkboxes** in a note can be ticked directly, without the editor.
- **Links** open in your browser (only `http://` and `https://`), e-mail addresses in your mail program; hover a link to see where it goes.

### Chat links

Chat links from the game (`[&…]`, e.g. a waypoint or an item) show as chips with an icon, the kind and the id; coins show their amount. The chip color shows the group: copper for places, blue for skills, traits and builds, green for items, violet for achievements, NPC texts and players. Names (instead of ids) will come with trailkit-core.

- Click a chip to copy its code (a short message at the bottom of the window confirms it), then paste it into the game chat. Hover it to see the code.
- A chip with **!** has a broken code; the tooltip says why.
- In the editor, paste a code into the text and it becomes a chip. The cursor jumps over a chip and Backspace deletes it whole; to change it, delete it and paste the new code.

### Saving and drafts

- **Save** (or **Ctrl+Enter** / **Ctrl+S**) saves your changes and closes the editor once they are on disk, **Cancel** throws them away — memo asks first if you changed something.
- If you choose another note, create one or close the window while you have unsaved changes, memo asks whether to save or discard them. (Set `"unsaved_changes": "save_automatically"` in `settings.json` to save without asking; a switch follows in the settings window.)
- **Escape** closes the window; with unsaved changes memo asks first.
- If the game or the addon closes while you are editing, your changes are kept as a draft. Such notes are marked with a yellow warning sign in the list (a collapsed group shows it too), and a hint above the list says how many there are; **Go to note** takes you to them one by one, and **x** hides the hint until the game restarts. In the note, choose **Restore** to continue or **Discard** to drop the changes. Drafts are deleted automatically after 30 days.

## Daily and weekly notes

Daily and weekly notes untick their checkboxes at the game's reset.

### Choosing the kind

Choose the kind of a note in the box next to its title (*Normal*, *Daily reset*, *Weekly reset*), or right-click it in the list › *Reset type*. At the game's daily or weekly reset all checkboxes in such a note are unticked again — handy for dailies and weekly clears. A note you switch to daily or weekly keeps its ticks until the next reset.

### When notes reset

- **Badges** at the right end of a row: **D** marks a daily note (reset at 00:00 UTC), **W** a weekly note (reset on Mondays at 07:30 UTC). Hover a badge to see how long it is until the next reset.
- If the game was closed at reset time, memo resets the note the next time you open the window — once, however many resets you missed.
- While you edit a note (or it has a draft), its reset waits until you save or discard; your text is never changed under your hands.

## Searching

The search finds notes by the words in their title or text.

### Using the search

Show the **Search** field from the menu or press **Ctrl+F** (not while editing); it stays above the note list until you close it, also after a restart. The **×** in the field first clears what you typed; on the empty field it closes the search. **Ctrl+F** again closes it at once. Type into it: the list shows only notes that contain every word you typed — in their title or text, upper or lower case alike. Umlauts count as their own letters: "ä" finds "Ä", but not "a".

- **×** in the field or **Escape** (while you type in the field) clears the search; a second Escape closes the window. Hiding the field also clears the search. The search stays when you close and reopen the window, until the game restarts.
- You cannot drag entries while searching; clear the search first. Renaming, the right-click menu and *Move to* work as usual. A new note or group clears the search.

### What the results show

- Groups with hits open by themselves; their number shows how many notes match. A group whose name matches shows everything inside.
- Hits are marked in the titles and in the open note, which jumps to the first hit. Notes that match only through their text show the number of hits at the end of the row.
- Link texts and chat-link codes are searched, link addresses are not. A chip whose code matches gets a frame.
- The open note always stays in the list, even if it no longer matches after an edit.

## Trash and undo

Deleted notes stay in the trash for 30 days, and recent steps can be undone.

### Undo

After deleting, a message at the bottom of the window says *Moved to the trash* and offers *Undo* and *Open trash* for a few seconds (pointing at it keeps it open). **Ctrl+Z** undoes the last delete or move, **Ctrl+Y** (or Ctrl+Shift+Z) does it again — up to 20 steps, until the game restarts. While you type, Ctrl+Z undoes your typing instead.

### The trash

Choose **Trash** in the menu to see the trash in place of the note list; the **back arrow** next to *Trash*, or *Notes* in the menu, takes you back to your notes. Deleted notes and groups stay there for **30 days** and are then removed for good. Each entry shows how many days are left — red in the last week.

- Click an entry to read it on the right. It cannot be edited there.
- **Restore** (or right-click › *Restore*) puts it back where it was. If its group no longer exists, it goes to the end of the list. A group comes back with everything inside.
- The red **trash can** next to *Restore* (or right-click › *Delete permanently*) deletes it for good, **Empty** deletes everything in the trash. Both ask first and cannot be undone.
- A note's unsaved changes stay with it in the trash and come back when you restore it.

## Keyboard shortcuts

These are the keys memo uses, sorted by where they work.

### Window

- **Ctrl+F** – show or close the search (not while editing)
- **Escape** – clear the search while you type in it; otherwise close the window (with unsaved changes memo asks first)
- **Ctrl+Z** – undo the last delete or move
- **Ctrl+Y** or **Ctrl+Shift+Z** – do it again
- **Ctrl+mouse wheel** – make all text larger or smaller
- **Ctrl+0** – default text size
- **Escape** in the help window – close the help window

### Note list

- **Delete** – move the selected entry to the trash
- **Double-click** – rename an entry
- **Enter** – keep the new name
- **Escape** – cancel renaming or dragging

### Editor

- **Ctrl+B** – bold
- **Ctrl+I** – italic
- **Ctrl+Enter** or **Ctrl+S** – save
- **Ctrl+Z** – undo your typing
- **Backspace** – next to a chat link chip: delete the whole chip

## Where your notes are stored

memo saves your notes automatically in your Guild Wars 2 folder.

### Files

Your notes are saved a moment after every change, in `addons/trailkit/memo/` inside your Guild Wars 2 folder (`notes.json`, `settings.json`, and `drafts.json` while a draft exists).

- If the notes file is damaged, memo starts with an empty list and tells you. The damaged file is kept in the `backups` folder next to it, so nothing is lost. You can close this hint with **x**; it stays closed until the game restarts.
- If your notes were saved by a newer version of memo, memo leaves the file untouched and asks you to update the addon.

### When saving fails

- If saving fails (for example because another program holds the file or the disk is full), memo tries again three times, five seconds apart. If it still fails, a red *Saving failed* message stays at the bottom of the window until saving works again; *Try again* saves once more, and the **x** hides the message until the next change fails. Your notes stay in the window meanwhile, and memo saves once more when the game closes. *Changes saved* tells you when it worked.
- **Save** in the editor waits until your text is on disk (*Saving…*). If that fails, the editor stays open with your text. If you then leave the editor, memo asks: *Discard* deletes your changes, *Keep editing* stays, *Save again* tries once more.

## Privacy

What stays on your PC and what leaves it.
