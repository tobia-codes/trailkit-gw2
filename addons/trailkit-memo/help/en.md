# trailkit-memo

> Pre-release skeleton — no notes yet.

## Opening the window

Click the **notes icon** (sheet with pencil) in the Nexus quick access bar, or
use the keybind **"Open/close notes"**. The keybind is unbound by default: set
it under *Nexus → Options → Keybinds → trailkit-memo*.

The window hides during loading screens, cutscenes and character select and
reappears afterwards.

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
