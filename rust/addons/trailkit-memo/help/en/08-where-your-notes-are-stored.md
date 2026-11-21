## Where your notes are stored

memo saves your notes automatically in your Guild Wars 2 folder.

### Files

Your notes are saved a moment after every change, in `addons/trailkit/memo/` inside your Guild Wars 2 folder (`notes.json`, `settings.json`, and `drafts.json` while a draft exists).

- If the notes file is damaged, memo starts with an empty list and tells you. The damaged file is kept in the `backups` folder next to it, so nothing is lost. You can close this hint with **x**; it stays closed until the game restarts.
- If your notes were saved by a newer version of memo, memo leaves the file untouched and asks you to update the addon.

### When saving fails

- If saving fails (for example because another program holds the file or the disk is full), memo tries again three times, five seconds apart. If it still fails, a red *Saving failed* message stays at the bottom of the window until saving works again; *Try again* saves once more, and the **x** hides the message until the next change fails. Your notes stay in the window meanwhile, and memo saves once more when the game closes. *Changes saved* tells you when it worked.
- **Save** in the editor waits until your text is on disk (*Saving…*). If that fails, the editor stays open with your text. If you then leave the editor, memo asks: *Discard* deletes your changes, *Keep editing* stays, *Save again* tries once more.
