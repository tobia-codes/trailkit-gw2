## Wo deine Notizen gespeichert werden

memo speichert deine Notizen automatisch im Guild-Wars-2-Ordner.

### Dateien

Deine Notizen werden kurz nach jeder Änderung gespeichert, in `addons/trailkit/memo/` im Guild-Wars-2-Ordner (`notes.json`, `settings.json` und `drafts.json`, solange ein Entwurf existiert).

- Ist die Notizdatei beschädigt, startet memo mit einer leeren Liste und weist dich darauf hin. Die beschädigte Datei bleibt im Ordner `backups` daneben erhalten, es geht also nichts verloren. Den Hinweis kannst du mit **x** schließen; er bleibt bis zum nächsten Spielstart zu.
- Wurden deine Notizen von einer neueren memo-Version gespeichert, lässt memo die Datei unverändert und bittet dich, das Addon zu aktualisieren.

### Wenn das Speichern fehlschlägt

- Schlägt das Speichern fehl (etwa weil ein anderes Programm die Datei sperrt oder die Festplatte voll ist), versucht memo es dreimal im Abstand von fünf Sekunden erneut. Klappt es dann immer noch nicht, bleibt unten im Fenster die rote Meldung *Speichern fehlgeschlagen* stehen, bis das Speichern wieder funktioniert; *Erneut versuchen* speichert noch einmal, und das **x** blendet die Meldung aus, bis die nächste Änderung scheitert. Deine Notizen bleiben so lange im Fenster, und beim Beenden des Spiels speichert memo noch einmal. *Änderungen gespeichert* zeigt dir, wenn es geklappt hat.
- **Speichern** im Editor wartet, bis dein Text auf der Festplatte ist (*Speichert…*). Scheitert das, bleibt der Editor mit deinem Text offen. Verlässt du ihn dann, fragt memo nach: *Verwerfen* löscht deine Änderungen, *Weiter bearbeiten* bleibt im Editor, *Erneut speichern* versucht es noch einmal.
