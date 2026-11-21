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

35. **Table of contents in the help (R155 c, branch `feature/help-table-of-contents`).**
    Deploy the branch instead of `develop` (setup step 1 on
    `feature/help-table-of-contents`). Open memo, click the `?` in the title
    bar, then the **Help** tab. Look at the topic list: before, nine plain
    topics; now the topics are bold with a chevron, and below the selected
    one its sections are listed one level indented with a thin guide line
    (mock: snapshots `help-window-help-*`, `help-window-section-clicked-*`).
    Click the section **The window**: it gets the selection (fill and violet
    bar), the topic loses it, and the right side scrolls so "The window"
    stands at the top. Click the topic **Getting started**: the topic is
    marked again and the text starts at its beginning. Click **Privacy**:
    "Getting started" folds up, Privacy shows its seven sections from the
    privacy policy, down to "Responsible person". Hover a shortened title:
    the tooltip shows it in full. Repeat once in Deutsch.
