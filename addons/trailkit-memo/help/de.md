# trailkit-memo

> Vorab-Version — Notizen lassen sich anlegen, umbenennen, ansehen und bearbeiten.

## Fenster öffnen

Klicke auf das **Notiz-Symbol** (Blatt mit Stift) in der Nexus-Schnellzugriffsleiste oder nutze die Tastenbelegung **„Notizen öffnen/schließen“**. Sie ist standardmäßig nicht belegt: unter *Nexus › Optionen › Tastenbelegungen › trailkit-memo* festlegen.

Während Ladebildschirmen, Zwischensequenzen und der Charakterauswahl wird das Fenster ausgeblendet und erscheint danach wieder.

## Deine Notizen

Das Fenster zeigt links die Notizliste und rechts die ausgewählte Notiz. Ziehe den Spalt zwischen beiden Seiten, um ihre Breite zu ändern (beim Darüberfahren erscheint eine Linie). Die Notizliste behält ihre Breite, wenn du das Fenster größer oder kleiner ziehst; der zusätzliche Platz geht an die Notiz.

- **Menü-Button** (drei Striche, oben links in der Notiz): blendet die Notizliste aus, damit die Notiz die ganze Breite bekommt; ein weiterer Klick blendet sie wieder ein. memo merkt sich deine Wahl.
- **Schmales Fenster:** Die Notiz füllt das Fenster, und der Menü-Button öffnet die Notizliste darüber. Wähle eine Notiz, klicke auf die abgedunkelte Notiz oder erneut auf den Menü-Button, um die Liste zu schließen. Beim Bearbeiten mit offener Vorschau schaltet memo schon früher um, damit Text und Vorschau Platz behalten.

- **Neue Notiz / neue Gruppe:** die beiden Buttons über der Liste. Der neue Eintrag landet in der ausgewählten Gruppe (bzw. der Gruppe der ausgewählten Notiz), ganz am Ende. Tippe gleich einen Namen ein und drücke Enter.
- **Umbenennen:** Doppelklick auf einen Eintrag oder Rechtsklick › *Umbenennen*. Enter oder ein Klick daneben übernimmt den Namen, Escape bricht ab.
- **Gruppen** haben fett gedruckte Namen und zeigen rechts, wie viele Notizen sie enthalten.
- **Abzeichen** am rechten Ende einer Zeile: **T** kennzeichnet eine tägliche Notiz (Reset um 00:00 UTC), **W** eine wöchentliche (Reset montags um 07:30 UTC). Fahre mit der Maus darüber für Details.
- **Checkboxen** in einer Notiz kannst du direkt abhaken.
- **Links** öffnen sich im Browser (nur `http://` und `https://`); fahre mit der Maus darüber, um das Ziel zu sehen.
- **Chat-Links** aus dem Spiel (`[&…]`, z. B. eine Wegmarke oder ein Gegenstand) erscheinen als Chips mit Symbol, Art und ID; Münzen zeigen ihren Betrag. Klicke auf einen Chip, um den Code zu kopieren, und füge ihn dann im Spiel-Chat ein. Fahre mit der Maus darüber, um den Code zu sehen. Ein Chip mit **!** hat einen beschädigten Code; der Tooltip sagt, warum. Namen (statt IDs) kommen mit trailkit-core.

Lange Namen werden mit „…“ gekürzt — fahre mit der Maus darüber, um den ganzen Namen zu sehen. In einer langen Notiz bleibt der Titel oben stehen, während du scrollst.

**Textgröße:** Halte **Strg** und drehe das Mausrad über dem Fenster, um den ganzen Text größer oder kleiner zu machen; **Strg+0** stellt die Standardgröße wieder her. Die aktuelle Größe erscheint in Prozent neben der Maus. memo merkt sich die Größe; sie baut auf der UI-Schriftgröße auf, die du in Nexus eingestellt hast.

## Eine Notiz bearbeiten

Klicke auf den **Stift** oben rechts in einer Notiz oder doppelklicke die Notiz. Du bearbeitest Titel und Text in Markdown; die Buttons über dem Text fügen Überschriften, **Fett**, *Kursiv*, Aufzählungen, Checkboxen und Links ein (auch **Strg+B** und **Strg+I**); ein erneuter Klick entfernt die Formatierung wieder (auch Links). Die Buttons der Formatierung an der Cursorposition leuchten auf. Der Event-Button benötigt trailkit-core.

- **Chat-Links:** Füge einen Code (`[&…]`) in den Text ein, und er wird zum Chip. Der Cursor springt über einen Chip, und die Rücktaste löscht ihn ganz; um ihn zu ändern, lösche ihn und füge den neuen Code ein.
- Der **Augen**-Button zeigt eine Vorschau der formatierten Notiz neben dem Text; ziehe den Spalt dazwischen, um die Breiten zu ändern. memo merkt sich beides.
- **Fertig** (oder **Strg+Enter**) übernimmt deine Änderungen, **Abbrechen** verwirft sie — memo fragt vorher nach, wenn du etwas geändert hast.
- Wählst du eine andere Notiz, legst eine neue an oder schließt das Fenster, während Änderungen offen sind, fragt memo, ob sie übernommen oder verworfen werden sollen. (Mit `"unsaved_changes": "save_automatically"` in `settings.json` wird ohne Nachfrage übernommen; ein Schalter folgt im Einstellungsfenster.)
- **Escape** schließt das Fenster; bei ungespeicherten Änderungen fragt memo vorher nach.
- Wird das Spiel oder das Addon beim Bearbeiten beendet, bleiben deine Änderungen als Entwurf erhalten. Beim nächsten Öffnen der Notiz wählst du **Wiederherstellen** zum Weitermachen oder **Verwerfen**. Entwürfe werden nach 30 Tagen automatisch gelöscht.

## Wo deine Notizen gespeichert werden

Deine Notizen werden kurz nach jeder Änderung automatisch gespeichert, in `addons/trailkit/memo/` im Guild-Wars-2-Ordner (`notes.json`, `settings.json` und `drafts.json`, solange ein Entwurf existiert).

- Ist die Notizdatei beschädigt, startet memo mit einer leeren Liste und weist dich darauf hin. Die beschädigte Datei bleibt im Ordner `backups` daneben erhalten, es geht also nichts verloren.
- Wurden deine Notizen von einer neueren memo-Version gespeichert, lässt memo die Datei unverändert und bittet dich, das Addon zu aktualisieren.

## Probleme?

Fehler landen im Nexus-Log (`addons/Nexus/Nexus.log` im Guild-Wars-2-Ordner). Bitte hänge die passenden Zeilen an, wenn du ein Issue auf GitHub eröffnest.
