# In-game tests

Pending checks the maintainer runs in Guild Wars 2, one at a time
(user request 2026-11-13). **Agents:** at the start of every session ask
whether the user can test in game now; keep this list current with every
change that needs an in-game check; remove an item only after the user
confirms it, and note failures as bugs in the relevant plan. Every item
names what to look at, how it looked before and how it must look now.

## Setup

1. Check out the branch named in the section and deploy a normal build
   (this also replaces the `fault-injection` debug DLL of session 34):
   `cargo xtask deploy trailkit-memo`
2. In game, open memo. Test in Dark and Light (Nexus theme) where a test
   says so; for German set the Nexus language to Deutsch.

## Branch `feature/button-cleanup` (R105, R106, M187)

Look at the button colors. Red means the `danger` fill: a red button with
the label in the window color, like the brass "Save" button but red.

| # | Where and how | Look at | Before | Now (expected) |
|---|---------------|---------|--------|----------------|
| 1 | Delete a note (`⋯` → Delete), then window menu `≡` → Trash. Hover the button, click it, then Cancel. | **"Empty"** button at the top of the trash list | Grey button with red text | **Red filled** button; hover makes it slightly lighter; clicking still asks first |
| 2 | In the trash, click an entry. Hover the icon left of "Restore". | **Trash icon** button ("Delete permanently") | Red icon on no background | **Red filled** square with the icon in the window color; hover lightens it; clicking still asks first |
| 3 | (a) Trash → "Empty"; (b) right-click a group with notes → Delete. | **Confirm button** of the question | Red filled "Delete" on the right | **Unchanged:** red filled on the right, "Cancel" left of it; Enter confirms only (b) |
| 4 | Select a note, click `⋯` next to Edit. | **"Delete"** menu entry | White text and icon, like the other entries | **Red** text and icon; hover gives the normal row highlight |
| 5 | Right-click a note, then a group. Then edit a note and right-click its group. | **"Delete"** menu entry | White text and icon | **Red** text and icon; while editing it is **grey** with a reason tooltip (as before) |
| 6 | Edit a note, type something, click another note in the list. Then make the window narrow (German). | **"Discard"** button of "Unsaved changes" | Grey button on the left | **Red filled** button on the left; "Keep editing" and brass "Save" stay right. Narrow: brass on top, red "Discard" last |
| 7 | Edit a note, type something, then reload memo in Nexus (or close the game) without saving; reopen the note. | **"Discard"** on the yellow draft card above the note | Grey button left of brass "Restore" | **Red filled** button left of brass "Restore". Narrow: "Restore" on top, red "Discard" below |
| 8 | Make `addons/trailkit/memo/notes.json` read-only (file properties), change a note, wait about 15 s. Then edit a note, change it, Save, and leave the editor. Remove read-only afterwards. | (a) **"Try again"** on the red toast at the bottom; (b) **"Discard"** in the "Saving failed" dialog | (a) Darker filled button without frame; (b) grey button on the left | (a) **Normal grey button with frame**, like "Cancel"; (b) **red filled** button on the left, brass "Save again" right. If read-only does not make saving fail, report it |
| 9 | Close the game, rename `notes.json`, start, open memo, look, close, rename it back. | **Two buttons** under "No notes yet" | Brass "Create your first note" **left**, "New group" right | "New group" **left**, brass "Create your first note" **right** |

## Branch `feature/crash-protection` (R104, merged)

| # | Where and how | Look at | Before | Now (expected) |
|---|---------------|---------|--------|----------------|
| 10 | Deploy with `cargo xtask deploy trailkit-memo --features fault-injection`, bind the key `trailkit.memo.keybind.fault_injection` in Nexus, edit a note, type, press the key. Reload memo. Afterwards deploy a normal build again. | Help window, the note, the log in `trailkit/logs/memo/` | A panic could crash the game | memo stops and opens the help on "Report a problem"; after reloading the typed text is back as a **draft** (yellow card, test 7); the log contains **`draft=kept`** |
