# In-game tests

Pending checks the maintainer runs in Guild Wars 2, one at a time
(user request 2026-11-13). **Agents:** at the start of every session ask
whether the user can test in game now; keep this list current with every
change that needs an in-game check; remove an item only after the user
confirms it, and note failures as bugs in the relevant plan.

## Setup

1. Check out the branch named in the section and deploy a normal build
   (this also replaces the `fault-injection` debug DLL of session 34):
   `cargo xtask deploy trailkit-memo`
2. In game, open memo. Test in Dark and Light (Nexus theme) where a test
   says so; for German set the Nexus language to Deutsch.

## Branch `feature/button-cleanup` (R105, R106, M187)

| # | Where | How | Expected |
|---|-------|-----|----------|
| 1 | Trash | Delete a note (`⋯` → Delete), then window menu `≡` → Trash. | "Empty" at the top of the trash list is a red filled button; hover makes it slightly lighter. Clicking it asks first; Cancel keeps everything. |
| 2 | Trash, entry selected | Click an entry in the trash. | The trash icon left of the brass "Restore" is a red filled square with the icon in the window color; hover lightens it; its tooltip says "Delete permanently". Clicking asks first. |
| 3 | Delete questions | (a) Trash → "Empty"; (b) right-click a group with notes → Delete. | Red "Delete permanently" / "Delete" on the right, "Cancel" left of it. Enter confirms only (b), Escape cancels both. |
| 4 | Note menu | Select a note, click `⋯` next to Edit. | "Delete" is the last entry, label and icon red; hover gives the normal row highlight. |
| 5 | List right-click menu | Right-click a note, then a group. | "Delete" red as in 4. While editing a note, right-click its group: "Delete" is grey with the reason tooltip. |
| 6 | Unsaved changes dialog | Edit a note, type something, click another note in the list. | "Discard" red, set apart on the left; "Keep editing" and brass "Save" on the right. Make the window narrow (German): buttons stack, brass first, red "Discard" last. |
| 7 | Draft hint | Edit a note, type something, then close the game (or reload memo in Nexus) without saving; reopen. | Yellow card above the note: red "Discard" left of brass "Restore". Narrow window: "Restore" on top, "Discard" below. |
| 8 | Save error toast and save-failed dialog | Make `addons/trailkit/memo/notes.json` read-only (file properties), change a note, wait about 15 s. Then edit a note, change it, Save, and leave the editor. | Toast at the bottom: "Saving failed" with a regular (not darker-filled) "Try again" and the `x`. Leaving the editor shows "Saving failed" with red "Discard" left and brass "Save again" right. Remove read-only afterwards. |
| 9 | Empty state | Only with a spare notes folder: rename `notes.json` while the game is closed, start, look, close, rename it back. | "No notes yet" with "New group" left and brass "Create your first note" right. |

## Branch `feature/crash-protection` (R104, merged)

| # | Where | How | Expected |
|---|-------|-----|----------|
| 10 | Draft rescue after a panic | Deploy with `cargo xtask deploy trailkit-memo --features fault-injection`, bind the key `trailkit.memo.keybind.fault_injection` in Nexus, edit a note, type, press the key. Afterwards deploy a normal build again. | memo stops and opens the help on "Report a problem"; after reloading memo the typed text is back as a draft (test 7). The log `trailkit/logs/memo/` contains `draft=kept`. |
