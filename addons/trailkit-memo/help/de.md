# trailkit-memo

> Vorab-Version (Grundgerüst) — noch keine Notizen.

## Fenster öffnen

Klicke auf das **Notiz-Symbol** (Blatt mit Stift) in der Nexus-Schnellzugriffs-
leiste oder nutze die Tastenbelegung **„Notizen öffnen/schließen“**. Sie ist
standardmäßig nicht belegt: unter
*Nexus → Optionen → Tastenbelegungen → trailkit-memo* festlegen.

Während Ladebildschirmen, Zwischensequenzen und der Charakterauswahl wird das
Fenster ausgeblendet und erscheint danach wieder.

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
