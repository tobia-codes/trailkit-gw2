## Notizen bearbeiten

Du schreibst Notizen in Markdown, mit Buttons für die Formatierung.

### Der Editor

Klicke oben rechts in einer Notiz auf **Bearbeiten** oder doppelklicke die Notiz. Du bearbeitest Titel und Text in Markdown; die Buttons in der Leiste darüber (in Gruppen wie in Word) fügen Überschriften (H1, H2 – ein Klick auf die andere Ebene wechselt sie, auf dieselbe entfernt sie), **Fett**, *Kursiv*, Aufzählungen, nummerierte Listen (eine nummerierte Zeile setzt die Nummer darüber fort) und Checkboxen ein (auch **Strg+B** und **Strg+I**); ein erneuter Klick entfernt die Formatierung wieder. Der Link-Button oder **Strg+K** öffnet einen kleinen Dialog für Text und Adresse des Links. Die Buttons der Formatierung an der Cursorposition leuchten auf. Der Event-Button benötigt trailkit-core. **Strg+C** kopiert den markierten Text; eine kurze Meldung unten bestätigt es, wie überall in memo.

- Der **Augen**-Button zeigt eine Vorschau der formatierten Notiz neben dem Text; ziehe den Spalt dazwischen, um die Breiten zu ändern. memo merkt sich beides.
- **Beim Bearbeiten** macht die Notizliste dem Editor Platz und kommt zurück, sobald du speicherst oder abbrichst. Hattest du sie selbst ausgeblendet, bleibt sie ausgeblendet. Klicke ins Spiel, um den Chat oder deine Tasten zu nutzen; ein Klick in den Text und du schreibst weiter.

### Checkboxen und Links

- **Checkboxen** in einer Notiz kannst du direkt abhaken, ohne den Editor.
- **Links** öffnen sich im Browser (nur `http://` und `https://`), E-Mail-Adressen in deinem Mailprogramm; das Icon vor dem Link zeigt, was sich öffnet; fahre mit der Maus darüber, um das Ziel zu sehen, sofern der Link es nicht schon anzeigt.
- **Link einfügen:** Markiere bei Bedarf Text, klicke auf den Link-Button oder drücke **Strg+K**, gib die Adresse ein und drücke Enter. Der Text ist optional, `https://` wird ergänzt, wenn es fehlt, und eine Adresse aus der Zwischenablage ist schon eingetragen. Steht der Cursor in einem Link, änderst du ihn dort oder entfernst den Link und behältst seinen Text.
- **Rechtsklick auf einen Link** öffnet ihn oder kopiert seine Adresse.
- **Adresse einfügen:** Ein einfaches `https://…` oder eine E-Mail-Adresse in deinem Text wird von selbst zum anklickbaren Link.

### Chat-Links

Chat-Links aus dem Spiel (`[&…]`, z. B. eine Wegmarke oder ein Gegenstand) erscheinen als Chips mit Symbol, Art und ID; Münzen zeigen ihren Betrag. Die Farbe zeigt die Gruppe: Kupfer für Orte, Blau für Fertigkeiten, Eigenschaften und Builds, Grün für Gegenstände, Violett für Erfolge, NPC-Texte und Spieler. Namen (statt IDs) kommen mit trailkit-core.

- Klicke auf einen Chip, um den Code zu kopieren (eine kurze Meldung unten im Fenster bestätigt es), und füge ihn dann im Spiel-Chat ein. Fahre mit der Maus darüber, um den Code zu sehen.
- Ein Rechtsklick auf einen Chip kopiert seinen Chat-Link oder öffnet ihn im Wiki, wenn es dort eine Seite geben kann.
- Ein Chip mit **!** hat einen beschädigten Code; der Tooltip sagt, warum.
- Füge im Editor einen Code in den Text ein, und er wird zum Chip. Der Cursor springt über einen Chip, und die Rücktaste löscht ihn ganz; um ihn zu ändern, lösche ihn und füge den neuen Code ein.

### Speichern und Entwürfe

- **Speichern** (oder **Strg+S**) übernimmt deine Änderungen und schließt den Editor, sobald sie gespeichert sind, **Abbrechen** verwirft sie — memo fragt vorher nach, wenn du etwas geändert hast.
- Wählst du eine andere Notiz, legst eine neue an oder schließt das Fenster, während Änderungen offen sind, fragt memo, ob sie übernommen oder verworfen werden sollen. (Mit `"unsaved_changes": "save_automatically"` in `settings.json` wird ohne Nachfrage übernommen; ein Schalter folgt im Einstellungsfenster.)
- **Escape** schließt das Fenster; bei ungespeicherten Änderungen fragt memo vorher nach.
- Wird das Spiel oder das Addon beim Bearbeiten beendet, bleiben deine Änderungen als Entwurf erhalten. Solche Notizen sind in der Liste mit einem gelben Warnzeichen markiert (eine eingeklappte Gruppe zeigt es auch), und ein Hinweis über der Liste sagt, wie viele es sind; **Zur Notiz springen** führt dich nacheinander hin, **x** blendet den Hinweis bis zum nächsten Spielstart aus. In der Notiz wählst du **Wiederherstellen** zum Weitermachen oder **Verwerfen**. Entwürfe werden nach 30 Tagen automatisch gelöscht.
