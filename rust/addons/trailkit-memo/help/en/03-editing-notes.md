## Editing notes

You write notes in markdown, with buttons for the formatting.

### The editor

Click **Edit** at the top right of a note, or double-click the note. You edit the title and the text in markdown; the buttons in the bar above (in groups, like in Word) insert headings (H1, H2 — clicking the other level switches it, the same level removes it), **bold**, *italic*, bulleted and numbered lists (a numbered line continues the number above) and checkboxes (also **Ctrl+B** and **Ctrl+I**); clicking a button again removes the formatting. The link button or **Ctrl+K** opens a small dialog for the link's text and address. Buttons of the formatting at the cursor light up. The event button needs trailkit-core. **Ctrl+C** copies the selected text; a short message at the bottom confirms it, as everywhere in memo.

- The **eye** button shows a preview of the formatted note next to the text; drag the gap between them to change the widths. memo remembers both.
- **While you edit** the note list makes way for the editor and comes back when you save or cancel. If you had hidden it yourself, it stays hidden. Click into the game to use the chat or your keys; click into the text to go on writing.

### Checkboxes and links

- **Checkboxes** in a note can be ticked directly, without the editor.
- **Links** open in your browser (only `http://` and `https://`), e-mail addresses in your mail program; the icon before a link shows which; hover a link to see where it goes, unless it already shows its address.
- **Add a link:** select text if you like, click the link button or press **Ctrl+K**, enter the address and press Enter. The text is optional, `https://` is added if it is missing, and an address from the clipboard is filled in. With the cursor in a link you change it there, or remove the link and keep its text.
- **Right-click a link** to open it or copy its address.
- **Paste an address:** a plain `https://…` or e-mail address in your text becomes a clickable link by itself.

### Chat links

Chat links from the game (`[&…]`, e.g. a waypoint or an item) show as chips with an icon, the kind and the id; coins show their amount. The chip color shows the group: copper for places, blue for skills, traits and builds, green for items, violet for achievements, NPC texts and players. Names (instead of ids) will come with trailkit-core.

- Click a chip to copy its code (a short message at the bottom of the window confirms it), then paste it into the game chat. Hover it to see the code.
- Right-click a chip to copy its chat link or, where the wiki can have a page, open it in the wiki.
- A chip with **!** has a broken code; the tooltip says why.
- In the editor, paste a code into the text and it becomes a chip. The cursor jumps over a chip and Backspace deletes it whole; to change it, delete it and paste the new code.

### Saving and drafts

- **Save** (or **Ctrl+S**) saves your changes and closes the editor once they are on disk, **Cancel** throws them away — memo asks first if you changed something.
- If you choose another note, create one or close the window while you have unsaved changes, memo asks whether to save or discard them. (Set `"unsaved_changes": "save_automatically"` in `settings.json` to save without asking; a switch follows in the settings window.)
- **Escape** closes the window; with unsaved changes memo asks first.
- If the game or the addon closes while you are editing, your changes are kept as a draft. Such notes are marked with a yellow warning sign in the list (a collapsed group shows it too), and a hint above the list says how many there are; **Go to note** takes you to them one by one, and **x** hides the hint until the game restarts. In the note, choose **Restore** to continue or **Discard** to drop the changes. Drafts are deleted automatically after 30 days.
