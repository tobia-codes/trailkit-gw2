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

## Pending (branch `feature/insert-after-selection`, deploy it first)

Deploy: `git switch feature/insert-after-selection`, then `cargo xtask deploy trailkit-memo` in `rust/`.

1. **New entries next to the selection (M215).** Select a note at the top
   level (e.g. "Daily"), click **New group** above the list. Before: the
   group went to the end of the list. Now: it appears right below "Daily",
   in rename mode. Select a group, click **New note**: the note is the
   group's last entry. Select a note inside a group, click **New note** or
   **New group**: it appears right below that note, in the same group.
2. **Empty spot clears the selection (M216).** With a note selected, click
   the empty space below the last row. Now no row is highlighted (no fill,
   no purple bar); the note stays on the right. Then **New note** goes to
   the end of the list and the list scrolls to it.
3. **Fills 4 px from the guide lines (R156).** Open a group with notes and
   hover a note in it, then select one. Before: the fill started about
   10 px right of the group's thin line. Now: 4 px right of it; the line is
   never covered. Approved look:
   https://claude.ai/artifact/2wvMyxTH8fUumYQS8dqTNh
