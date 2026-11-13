# trailkit-memo

## Erste Schritte

memo bewahrt deine Notizen und Checklisten in einem Fenster im Spiel auf.

### Fenster öffnen

Klicke auf das **Notiz-Symbol** (Blatt mit Stift) in der Nexus-Schnellzugriffsleiste oder nutze die Tastenbelegung **„Notizen öffnen/schließen“**. Sie ist standardmäßig nicht belegt: unter *Nexus › Optionen › Tastenbelegungen › trailkit-memo* festlegen.

Während Ladebildschirmen, Zwischensequenzen und der Charakterauswahl wird das Fenster ausgeblendet und erscheint danach wieder.

### Das Fenster

Das Fenster zeigt links die Notizliste und rechts die ausgewählte Notiz. Ziehe den Spalt zwischen beiden Seiten, um ihre Breite zu ändern (beim Darüberfahren erscheint eine Linie). Die Notizliste behält ihre Breite, wenn du das Fenster größer oder kleiner ziehst; der zusätzliche Platz geht an die Notiz.

In der Titelleiste sitzen diese Buttons:

- **Menü-Button**, oben links: öffnet das Menü. Ein Klick auf die abgedunkelte Fläche oder erneut auf den Button schließt es.
- **?** rechts: öffnet oder schließt das Hilfefenster.
- **Pfeil** rechts: klappt das Fenster auf seine Titelleiste ein (erneut klicken zum Ausklappen); **×** schließt es.

Ein hinterlegter Button bedeutet: Was er schaltet, ist gerade eingeblendet. Lange Namen werden mit „…“ gekürzt — fahre mit der Maus darüber, um den ganzen Namen zu sehen. In einer langen Notiz bleibt der Titel oben stehen, während du scrollst; in einer langen Liste bleiben Suchfeld und Buttons oben.

### Menü und Ansichten

Das Menü führt dich zu den Bereichen von memo:

- **Notizen** bringt dich immer zurück zur normalen Seite: Es verlässt den Papierkorb, schließt die Suche, beendet die kompakte Ansicht und blendet die Notizliste wieder ein.
- **Suchen** zeigt das Suchfeld. **Papierkorb** zeigt den Papierkorb.
- **Notizliste ausblenden** gibt der Notiz die ganze Breite; **Notizliste einblenden** holt die Liste zurück. memo merkt sich deine Wahl.
- **Kompakte Ansicht** (Pfeile-Button, oben links in der Notiz): blendet die Leiste mit *Bearbeiten* und das Typ-Feld aus, damit die Notiz mehr Platz bekommt. Der Button hinter dem Titel beendet sie wieder. Andere Notizen auswählen und Kästchen abhaken geht weiter; eine Notiz bearbeiten (auch per Doppelklick) oder eine neue anlegen beendet die kompakte Ansicht. memo merkt sie sich. Zusammen mit ausgeblendeter Notizliste bleibt nur die Notiz übrig.
- **Kleines Fenster:** Das Fenster wird nicht schmaler, als Notizliste und Notiz nebeneinander brauchen, die Liste verschwindet also nie von selbst, sie wird nur schmaler. Größerer Text macht diese Mindestbreite größer.
- **Textgröße:** Halte **Strg** und drehe das Mausrad über dem Fenster, um den ganzen Text größer oder kleiner zu machen; **Strg+0** stellt die Standardgröße wieder her. Die aktuelle Größe erscheint in Prozent neben der Maus. memo merkt sich die Größe; sie baut auf der UI-Schriftgröße auf, die du in Nexus eingestellt hast.

### Hilfe und Probleme

Das **?** in der Titelleiste öffnet dieses Hilfefenster; ein weiterer Klick schließt es. **Hilfe öffnen** in memos Tab unter *Nexus › Optionen* öffnet es ebenfalls oder holt es nach vorn. **Escape** schließt es.

Das Fenster hat zwei Tabs: **Hilfe** mit diesen Themen und **Problem melden**, das dein Log als Textdatei speichert und das Problemformular auf GitHub öffnet. Bis zum Neustart des Spiels öffnet sich das Fenster wieder auf dem Tab und Thema, das du zuletzt angesehen hast.

Stößt memo auf einen Fehler, hält es an, statt das Spiel abstürzen zu lassen: Dein ungespeicherter Text bleibt als Entwurf erhalten, Nexus zeigt eine kurze Meldung und dieses Fenster öffnet sich auf **Problem melden**. Bis du memo in der Addon-Liste von Nexus neu lädst oder das Spiel neu startest, öffnen Keybind und Schnellzugriffssymbol dieses Fenster.

## Notizen und Gruppen

Deine Notizen stehen in einer Liste, die du sortieren und in Gruppen ordnen kannst.

### Anlegen und umbenennen

- **Neue Notiz / neue Gruppe:** die beiden Buttons über der Liste. Der neue Eintrag landet in der ausgewählten Gruppe (bzw. der Gruppe der ausgewählten Notiz), ganz am Ende. Tippe gleich einen Namen ein und drücke Enter.
- **Rechtsklick** auf eine Gruppe oder eine leere Stelle der Liste bietet dasselbe: auf einer Gruppe kommt der neue Eintrag in sie hinein, auf einer leeren Stelle ans Ende der Liste.
- **Umbenennen:** Doppelklick auf einen Eintrag oder Rechtsklick › *Umbenennen*. Enter oder ein Klick daneben übernimmt den Namen, Escape bricht ab.
- **Notizmenü:** Der Button mit den drei Punkten neben *Bearbeiten* bietet *Umbenennen*, *Verschieben nach*, *Reset-Art* und *Löschen* für die offene Notiz. *Umbenennen* öffnet den Editor mit markiertem Titel; beende ihn mit *Speichern*.

### Sortieren und verschieben

- **Sortieren:** Ziehe eine Notiz oder Gruppe an eine neue Stelle. Eine Linie zeigt, wo sie landet; auf der unteren Hälfte einer offenen Gruppe wird sie deren erster Eintrag, unter dem letzten Eintrag kommt sie ans Ende der Liste. Auf der Mitte einer geschlossenen Gruppe landet sie an deren Ende; bleibst du kurz darauf, öffnet sich die Gruppe. Ein rotes Verbotszeichen heißt: Dort geht es nicht hin (eine Gruppe kann nicht in sich selbst). Am oberen oder unteren Rand scrollt die Liste. Escape bricht ab.
- **Verschieben nach:** Rechtsklick › *Verschieben nach* › *Oberste Ebene* oder eine Gruppe verschiebt den Eintrag an deren Ende, ohne zu ziehen.

### Gruppen

Gruppen haben fett gedruckte Namen und zeigen rechts, wie viele Notizen sie enthalten. Ein Klick irgendwo auf die Zeile klappt die Gruppe auf oder zu; die Notiz, die du gerade liest, bleibt rechts stehen.

### Löschen

Rechtsklick › *Löschen*, oder den Eintrag auswählen und **Entf** drücken, während die Liste den Fokus hat. Der Eintrag kommt in den Papierkorb; eine Gruppe mit Notizen fragt vorher nach. Die Notiz, die du gerade bearbeitest, lässt sich erst löschen, wenn du speicherst oder abbrichst. Wie du ihn zurückholst, steht unter *Papierkorb und Rückgängig*.

## Notizen bearbeiten

Du schreibst Notizen in Markdown, mit Buttons für die Formatierung.

### Der Editor

Klicke oben rechts in einer Notiz auf **Bearbeiten** oder doppelklicke die Notiz. Du bearbeitest Titel und Text in Markdown; die Buttons in der Leiste darüber (in Gruppen wie in Word) fügen Überschriften (H1, H2 – ein Klick auf die andere Ebene wechselt sie, auf dieselbe entfernt sie), **Fett**, *Kursiv*, Aufzählungen, Checkboxen und Links ein (auch **Strg+B** und **Strg+I**); ein erneuter Klick entfernt die Formatierung wieder (auch Links). Die Buttons der Formatierung an der Cursorposition leuchten auf. Der Event-Button benötigt trailkit-core.

- Der **Augen**-Button zeigt eine Vorschau der formatierten Notiz neben dem Text; ziehe den Spalt dazwischen, um die Breiten zu ändern. memo merkt sich beides.
- **Beim Bearbeiten** macht die Notizliste dem Editor Platz und kommt zurück, sobald du speicherst oder abbrichst. Hattest du sie selbst ausgeblendet, bleibt sie ausgeblendet. Klicke ins Spiel, um den Chat oder deine Tasten zu nutzen; ein Klick in den Text und du schreibst weiter.

### Checkboxen und Links

- **Checkboxen** in einer Notiz kannst du direkt abhaken, ohne den Editor.
- **Links** öffnen sich im Browser (nur `http://` und `https://`), E-Mail-Adressen in deinem Mailprogramm; fahre mit der Maus darüber, um das Ziel zu sehen.

### Chat-Links

Chat-Links aus dem Spiel (`[&…]`, z. B. eine Wegmarke oder ein Gegenstand) erscheinen als Chips mit Symbol, Art und ID; Münzen zeigen ihren Betrag. Die Farbe zeigt die Gruppe: Kupfer für Orte, Blau für Fertigkeiten, Eigenschaften und Builds, Grün für Gegenstände, Violett für Erfolge, NPC-Texte und Spieler. Namen (statt IDs) kommen mit trailkit-core.

- Klicke auf einen Chip, um den Code zu kopieren (eine kurze Meldung unten im Fenster bestätigt es), und füge ihn dann im Spiel-Chat ein. Fahre mit der Maus darüber, um den Code zu sehen.
- Ein Chip mit **!** hat einen beschädigten Code; der Tooltip sagt, warum.
- Füge im Editor einen Code in den Text ein, und er wird zum Chip. Der Cursor springt über einen Chip, und die Rücktaste löscht ihn ganz; um ihn zu ändern, lösche ihn und füge den neuen Code ein.

### Speichern und Entwürfe

- **Speichern** (oder **Strg+Enter** / **Strg+S**) übernimmt deine Änderungen und schließt den Editor, sobald sie gespeichert sind, **Abbrechen** verwirft sie — memo fragt vorher nach, wenn du etwas geändert hast.
- Wählst du eine andere Notiz, legst eine neue an oder schließt das Fenster, während Änderungen offen sind, fragt memo, ob sie übernommen oder verworfen werden sollen. (Mit `"unsaved_changes": "save_automatically"` in `settings.json` wird ohne Nachfrage übernommen; ein Schalter folgt im Einstellungsfenster.)
- **Escape** schließt das Fenster; bei ungespeicherten Änderungen fragt memo vorher nach.
- Wird das Spiel oder das Addon beim Bearbeiten beendet, bleiben deine Änderungen als Entwurf erhalten. Solche Notizen sind in der Liste mit einem gelben Warnzeichen markiert (eine eingeklappte Gruppe zeigt es auch), und ein Hinweis über der Liste sagt, wie viele es sind; **Zur Notiz springen** führt dich nacheinander hin, **x** blendet den Hinweis bis zum nächsten Spielstart aus. In der Notiz wählst du **Wiederherstellen** zum Weitermachen oder **Verwerfen**. Entwürfe werden nach 30 Tagen automatisch gelöscht.

## Tägliche und wöchentliche Notizen

Tägliche und wöchentliche Notizen entfernen ihre Häkchen beim Reset des Spiels.

### Die Art wählen

Wähle die Art einer Notiz im Feld neben ihrem Titel (*Normal*, *Täglicher Reset*, *Wöchentlicher Reset*) oder per Rechtsklick in der Liste › *Reset-Art*. Beim täglichen bzw. wöchentlichen Reset des Spiels werden alle Häkchen in so einer Notiz entfernt – praktisch für Dailies und wöchentliche Raids. Stellst du eine Notiz auf täglich oder wöchentlich um, bleiben ihre Häkchen bis zum nächsten Reset.

### Wann Notizen zurückgesetzt werden

- **Abzeichen** am rechten Ende einer Zeile: **T** kennzeichnet eine tägliche Notiz (Reset um 00:00 UTC), **W** eine wöchentliche (Reset montags um 07:30 UTC). Fahre mit der Maus darüber, um zu sehen, wie lange es bis zum nächsten Reset dauert.
- War das Spiel zum Reset geschlossen, setzt memo die Notiz beim nächsten Öffnen des Fensters zurück – einmal, egal wie viele Resets du verpasst hast.
- Während du eine Notiz bearbeitest (oder es einen Entwurf gibt), wartet ihr Reset, bis du speicherst oder verwirfst; dein Text wird nie unter deinen Händen geändert.

## Suchen

Die Suche findet Notizen über die Wörter in ihrem Titel oder Text.

### Die Suche benutzen

Blende das Feld **Suchen** über das Menü ein oder drücke **Strg+F** (nicht beim Bearbeiten); es bleibt über der Notizliste, bis du es schließt, auch nach einem Neustart. Das **×** im Feld leert zuerst deine Eingabe; beim leeren Feld schließt es die Suche. Erneutes **Strg+F** schließt sie sofort. Tippe hinein: Die Liste zeigt dann nur Notizen, die jedes eingegebene Wort enthalten — im Titel oder im Text, egal ob groß oder klein geschrieben. Umlaute zählen als eigene Buchstaben: „ä“ findet „Ä“, aber nicht „a“.

- **×** im Feld oder **Escape** (während du im Feld tippst) leert die Suche; ein zweites Escape schließt das Fenster. Auch das Ausblenden des Felds leert die Suche. Die Suche bleibt beim Schließen und Öffnen des Fensters erhalten, bis das Spiel neu startet.
- Während einer Suche kannst du nichts verschieben; leere zuerst die Suche. Umbenennen, das Rechtsklick-Menü und *Verschieben nach* gehen wie gewohnt. Eine neue Notiz oder Gruppe leert die Suche.

### Was die Treffer zeigen

- Gruppen mit Treffern klappen von selbst auf; ihre Zahl zeigt, wie viele Notizen passen. Passt der Name einer Gruppe, erscheint alles darin.
- Treffer sind in den Titeln und in der geöffneten Notiz markiert; die Notiz springt zum ersten Treffer. Notizen, die nur über ihren Text passen, zeigen am Ende der Zeile die Zahl der Treffer.
- Linktexte und Chat-Codes werden durchsucht, Linkadressen nicht. Ein Chip, dessen Code passt, bekommt einen Rahmen.
- Die geöffnete Notiz bleibt immer in der Liste, auch wenn sie nach einer Änderung nicht mehr passt.

## Papierkorb und Rückgängig

Gelöschte Notizen bleiben 30 Tage im Papierkorb, und die letzten Schritte lassen sich rückgängig machen.

### Rückgängig

Nach dem Löschen bietet die Meldung *In den Papierkorb verschoben* unten im Fenster ein paar Sekunden lang *Rückgängig* und *Papierkorb öffnen* an (solange die Maus darauf zeigt, bleibt sie stehen). **Strg+Z** macht das letzte Löschen oder Verschieben rückgängig, **Strg+Y** (oder Strg+Umschalt+Z) stellt es wieder her – bis zu 20 Schritte, bis zum Neustart des Spiels. Während du tippst, macht Strg+Z stattdessen dein Tippen rückgängig.

### Der Papierkorb

Wähle im Menü **Papierkorb**, um statt der Notizliste den Papierkorb zu sehen; der **Zurück-Pfeil** neben *Papierkorb* oder *Notizen* im Menü bringt dich zurück zu deinen Notizen. Gelöschte Notizen und Gruppen bleiben dort **30 Tage** und werden dann endgültig entfernt. Jeder Eintrag zeigt, wie viele Tage noch bleiben – in der letzten Woche rot.

- Klicke einen Eintrag an, um ihn rechts zu lesen. Bearbeiten lässt er sich dort nicht.
- **Wiederherstellen** (oder Rechtsklick › *Wiederherstellen*) legt ihn dorthin zurück, wo er war. Gibt es seine Gruppe nicht mehr, landet er am Ende der Liste. Eine Gruppe kommt mit allem darin zurück.
- Der rote **Papierkorb** neben *Wiederherstellen* (oder Rechtsklick › *Endgültig löschen*) löscht ihn endgültig, **Leeren** löscht alles im Papierkorb. Beides fragt vorher nach und lässt sich nicht rückgängig machen.
- Ungespeicherte Änderungen einer Notiz bleiben im Papierkorb bei ihr und kommen beim Wiederherstellen mit zurück.

## Tastenkürzel

Das sind die Tasten, die memo nutzt, sortiert danach, wo sie wirken.

### Fenster

- **Strg+F** – Suche einblenden oder schließen (nicht beim Bearbeiten)
- **Escape** – Suche leeren, während du im Feld tippst; sonst das Fenster schließen (bei ungespeicherten Änderungen fragt memo vorher nach)
- **Strg+Z** – letztes Löschen oder Verschieben rückgängig machen
- **Strg+Y** oder **Strg+Umschalt+Z** – wiederherstellen
- **Strg+Mausrad** – den ganzen Text größer oder kleiner machen
- **Strg+0** – Standard-Textgröße
- **Escape** im Hilfefenster – das Hilfefenster schließen

### Notizliste

- **Entf** – ausgewählten Eintrag in den Papierkorb verschieben
- **Doppelklick** – Eintrag umbenennen
- **Enter** – neuen Namen übernehmen
- **Escape** – Umbenennen oder Ziehen abbrechen

### Editor

- **Strg+B** – fett
- **Strg+I** – kursiv
- **Strg+Enter** oder **Strg+S** – speichern
- **Strg+Z** – dein Tippen rückgängig machen
- **Rücktaste** – neben einem Chat-Link-Chip: den ganzen Chip löschen

## Wo deine Notizen gespeichert werden

memo speichert deine Notizen automatisch im Guild-Wars-2-Ordner.

### Dateien

Deine Notizen werden kurz nach jeder Änderung gespeichert, in `addons/trailkit/memo/` im Guild-Wars-2-Ordner (`notes.json`, `settings.json` und `drafts.json`, solange ein Entwurf existiert).

- Ist die Notizdatei beschädigt, startet memo mit einer leeren Liste und weist dich darauf hin. Die beschädigte Datei bleibt im Ordner `backups` daneben erhalten, es geht also nichts verloren. Den Hinweis kannst du mit **x** schließen; er bleibt bis zum nächsten Spielstart zu.
- Wurden deine Notizen von einer neueren memo-Version gespeichert, lässt memo die Datei unverändert und bittet dich, das Addon zu aktualisieren.

### Wenn das Speichern fehlschlägt

- Schlägt das Speichern fehl (etwa weil ein anderes Programm die Datei sperrt oder die Festplatte voll ist), versucht memo es dreimal im Abstand von fünf Sekunden erneut. Klappt es dann immer noch nicht, bleibt unten im Fenster die rote Meldung *Speichern fehlgeschlagen* stehen, bis das Speichern wieder funktioniert; *Erneut versuchen* speichert noch einmal, und das **x** blendet die Meldung aus, bis die nächste Änderung scheitert. Deine Notizen bleiben so lange im Fenster, und beim Beenden des Spiels speichert memo noch einmal. *Änderungen gespeichert* zeigt dir, wenn es geklappt hat.
- **Speichern** im Editor wartet, bis dein Text auf der Festplatte ist (*Speichert…*). Scheitert das, bleibt der Editor mit deinem Text offen. Verlässt du ihn dann, fragt memo nach: *Verwerfen* löscht deine Änderungen, *Weiter bearbeiten* bleibt im Editor, *Erneut speichern* versucht es noch einmal.

## Datenschutz

Was auf deinem PC bleibt und was ihn verlässt.
