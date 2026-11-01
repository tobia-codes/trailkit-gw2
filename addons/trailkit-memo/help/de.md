# trailkit-memo

> Vorab-Version — Notizen lassen sich anlegen, umbenennen und ansehen;
> den Inhalt bearbeiten kommt in einer späteren Version.

## Fenster öffnen

Klicke auf das **Notiz-Symbol** (Blatt mit Stift) in der Nexus-Schnellzugriffs-
leiste oder nutze die Tastenbelegung **„Notizen öffnen/schließen“**. Sie ist
standardmäßig nicht belegt: unter
*Nexus → Optionen → Tastenbelegungen → trailkit-memo* festlegen.

Während Ladebildschirmen, Zwischensequenzen und der Charakterauswahl wird das
Fenster ausgeblendet und erscheint danach wieder.

## Deine Notizen

Das Fenster zeigt links die Notizliste und rechts die ausgewählte Notiz. Ist
das Fenster schmal, siehst du immer nur eines davon; mit **Zurück** geht es
zur Liste. Ziehe die Linie zwischen beiden Seiten, um ihre Breite zu ändern.

- **Neue Notiz / neue Gruppe:** die beiden Buttons über der Liste. Der neue
  Eintrag landet in der ausgewählten Gruppe (bzw. der Gruppe der
  ausgewählten Notiz), ganz am Ende. Tippe gleich einen Namen ein und
  drücke Enter.
- **Umbenennen:** Doppelklick auf einen Eintrag oder Rechtsklick →
  *Umbenennen*. Enter oder ein Klick daneben übernimmt den Namen, Escape
  bricht ab.
- **Abzeichen:** **T** kennzeichnet eine tägliche Notiz (Reset um 00:00 UTC),
  **W** eine wöchentliche (Reset montags um 07:30 UTC). Fahre mit der Maus
  darüber für Details.
- **Checkboxen** in einer Notiz kannst du direkt abhaken.
- **Links** öffnen sich im Browser (nur `http://` und `https://`); fahre mit
  der Maus darüber, um das Ziel zu sehen.

Lange Namen werden mit „…“ gekürzt — fahre mit der Maus darüber, um den
ganzen Namen zu sehen.

## Wo deine Notizen gespeichert werden

Deine Notizen werden kurz nach jeder Änderung automatisch gespeichert, in
`addons/trailkit/memo/` im Guild-Wars-2-Ordner (`notes.json` und
`settings.json`).

- Ist die Notizdatei beschädigt, startet memo mit einer leeren Liste und
  weist dich darauf hin. Die beschädigte Datei bleibt im Ordner `backups`
  daneben erhalten, es geht also nichts verloren.
- Wurden deine Notizen von einer neueren memo-Version gespeichert, lässt
  memo die Datei unverändert und bittet dich, das Addon zu aktualisieren.

## Probleme?

Fehler landen im Nexus-Log (`addons/Nexus/Nexus.log` im Guild-Wars-2-Ordner).
Bitte hänge die passenden Zeilen an, wenn du ein Issue auf GitHub eröffnest.
