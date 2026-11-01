# trailkit-memo

> Pre-release — notes can be created, renamed and viewed; editing the
> content comes in a later version.

## Opening the window

Click the **notes icon** (sheet with pencil) in the Nexus quick access bar, or
use the keybind **"Open/close notes"**. The keybind is unbound by default: set
it under *Nexus → Options → Keybinds → trailkit-memo*.

The window hides during loading screens, cutscenes and character select and
reappears afterwards.

## Your notes

The window shows the note list on the left and the selected note on the
right. Drag the gap between both sides to change their widths (a line
appears when you point at it).

- **Menu button** (three lines, top left of the note): hides the note list
  so the note gets the full width; click it again to show the list. memo
  remembers your choice.
- **Narrow window:** the note fills the window and the menu button opens the
  note list over it. Choose a note, click the dimmed note or the menu button
  again to close the list.

- **New note / new group:** the two buttons above the list. The new entry
  goes into the selected group (or the group of the selected note), at the
  end. Type a name right away and press Enter.
- **Rename:** double-click an entry, or right-click → *Rename*. Enter or
  clicking elsewhere keeps the new name, Escape cancels.
- **Groups** have bold names and show how many notes they contain on the
  right.
- **Badges** at the right end of a row: **D** marks a daily note (reset at 00:00 UTC), **W** a weekly
  note (reset on Mondays at 07:30 UTC). Hover a badge for details.
- **Checkboxes** in a note can be ticked directly.
- **Links** open in your browser (only `http://` and `https://`); hover a
  link to see where it goes.

Long names are shortened with "…" — hover them to see the full name.

## Where your notes are stored

Your notes are saved automatically a moment after every change, in
`addons/trailkit/memo/` inside your Guild Wars 2 folder (`notes.json` and
`settings.json`).

- If the notes file is damaged, memo starts with an empty list and tells
  you. The damaged file is kept in the `backups` folder next to it, so
  nothing is lost.
- If your notes were saved by a newer version of memo, memo leaves the file
  untouched and asks you to update the addon.

## Problems?

Errors are written to the Nexus log (`addons/Nexus/Nexus.log` in your Guild
Wars 2 folder). Please attach the relevant lines when you open an issue on
GitHub.
