# legacylisten
`legacylisten` ist ein einfacher CLI Musikspieler, den ich geschrieben
habe, da kein existierender meinen Bedürfnissen entsprach.  Die
Hauptbesonderheit ist, dass man ändern kann, wie oft ein Lied
abgespielt wird (`legacylisten` ist immer auf Shuffle-all), aber es
gibt auch einige noch komischere Funktionen.

## Funktionsweise
`legacylisten` erzeugt eine Liste aller Lieder in
`~/.zvavybir/legacylisten/data`[^1] zusammen mit ihrer
"Spielwahrscheinlichkeit" und Lautstärke (die Standardwerte sind 10
und 10% respektive).  Dann wählt es ein zufälliges Lied mit der
Wahrscheinlichkeit proportional zu seiner Spielwahrscheinlichkeit aus
und spielt es bis der Nutzer etwas anderes sagt.

Die Lautstärke ist pro Lied anpassbar und wird gespeichert.  Obwohl es
einfach wäre (lächerlich trivial sogar) zu implementieren, gibt es
keine Möglichkeit die globale Lautstärke zu verändern, da ich finde,
dass man dies besser dem Betriebssystem überlässt.  Worin sich ein
Musikspieler hervortun kann, ist zu wissen, welches Lied gespielt wird
und darauf zu reagieren.  Die Idee hinter dieser Funktion ist die
Lautstärke von sehr leisen Liedern einmal zu erhöhen und sich dann
nicht mehr darum kümmern zu müssen.

Eine andere ziemlich komische Funktion ist das man nicht nur sofort
stoppen/beenden kann, sondern auch wenn das aktuelle Lied zu Ende ist.
Das Komischste ist aber, dass `legacylisten` alle Verbindungen zur
Festplatte trennt, wenn das *NIX Signal `SIGUSR1` gefangen wird und
nur nach `SIGUSR2` wieder Verbindungen aufnimmt.  `SIGUSR1`
unterbricht allerdings kein bereits spielendes Lied, da Lieder immer
gepuffert werden[^2].

## Befehle
Befehle sind der Weg auf dem `legacylisten` bedient wird und bestehen
immer aus einem einzigen Zeichen.  Ursprünglich waren sie immer der
erste Buchstabe des Befehlsnamen, da dies aber in sehr komische Namen
resultierte (z.b. `f` – "**f**ainter" *schwächer* – um die Lautstärke
zu verringern), habe ich mich entschieden sie einfach alphabetisch
durch zu nummerieren.

Um ein Befehl auszuführen, tippe einfach sein Buchstaben ein (aber
denke daran, dass Terminals üblicherweise Zeilen-gepuffert sind,
d.h. `legacylisten` sieht – und reagiert auf – die Eingabe nur nachdem
Enter gedrückt wurde).

Es gibt die folgenden Befehle:

* `?`: Zeigt eine Liste aller Befehle mit Erklärung (mehr oder weniger
  diese Liste).[^3]
* `a`: Erhöht die Spielwahrscheinlichkeit des aktuellen Lieds um 1.
* `b`: Verringert die Spielwahrscheinlichkeit des aktuellen Lieds um 1.
* `c`: Beendet `legacylisten` und speichert die
  Spielwahrscheinlichkeiten und Lautstärken in
  `~/.zvavybir/legacylisten/songs.csv`.
* `d`: Stoppt das Abspielen.
* `e`: Setzt das Abspielen wieder fort nachdem es mit `d` oder `l`
  angehalten wurde (überschreibt `SIGUSR1` allerdings nicht).
* `f`: Überspringt das Lied.
* `g`: Erhöht die Lautstärke des aktuellen Lieds permanent um 1%
  (allerdings nicht auf mehr als 100%).
* `h`: Verringert die Lautstärke des aktuellen Lieds permanent um 1%
  (allerdings nicht auf weniger als 0%).
* `i`: Zeigt an wie lange das Lied schon spielt und – wenn
  verfügbar[^4] – wie lange es insgesamt brauchen wird.
* `j`: Wechselt zwischen Abspielen und Stoppen.
* `k`: Beendet `legacylisten` sobald das aktuelle Lied fertig ist (`k`
  nimmt Vorrang zu `l`).
* `l`: Wie `k`, stoppt aber nur anstatt zu beenden.
* `m`: Zeigt die Metadaten im id3-Tag des Lieds (die Länge muss
  üblicherweise mit `i` nachgefragt werden, da sie nur selten im
  id3-Tag gespeichert wird).
* `n`: Öffnet das Cover in dem eingestellten Bildbetrachter (diese
  Funktion nutzt `mimeopen` was so viel ich weiß unter MS Windows
  nicht verfügbar ist).  Wenn das Lied kein Cover hat wird
  `~/.zvavybir/legacylisten/default.png` stattdessen geöffnet.  Für
  das Bild, das ich nutze (und gemacht habe, also ziemlich schlecht
  ist) siehe
  [hier](https://github.com/zvavybir/legacylisten/blob/master/imgs/default.png).
* `o`: Wiederholung des Liedes anhalten (wenn allerdings das Lied als
  Wiederholung gespielt wird, wird nicht sofort beendet; wenn das
  gewünscht ist, überspringe auch noch mit `f`).
* `p`: Wiederholt das aktuelle Lied einmal.
* `q`: Wiederholt das aktuelle Lied ewig.
* `r`: Springt zum Anfang des aktuellen Lieds oder – wenn es bereits
  am Anfang ist – zum vorherigen.  Man kann so viele Lieder
  zurückgehen wie man will (oder genauer gesagt so viele es gibt).
  Alle gespielten Lieder werden gespeichert (allerdings nur in einem
  Aufruf von `legacylisten`, wenn es neu gestartet wird ist die
  Geschichte verloren) und wenn man zurückgeht, wird man auf dieses
  Lied wieder das Gleiche folgen sehen wie davor.

## Low memory handler
Wie schon vorher kurz angeschnitten, hatten insbesondere ältere
Versionen von `leacylisten` einen schlimmen RAM-Fußabdruck, was mein
System ein paar Mal für ein paar Sekunden lahmgelegt hat.  Unter Linux
gibt es für so einen Fall eigentlich den OOM Killer (welcher bei
Bedarf den unwichtigsten, aber verschwenderischsten Prozess beendet),
es stellt sich allerdings heraus, dass der Chromium Webbrowser (die
Freie-Software Variante von Google Chrome), welchen ich manchmal
gezwungener Maßen einsetzte, sogar noch schlimmer als meine Verbrechen
ist.  Anstatt etwas Vernünftiges zu machen, habe ich stattdessen eine
Routine zu `legacylisten` hinzugefügt, die die Menge an freiem RAM
überwacht und sich selbst beendet, wenn es unter ein (einstellbares)
Limit fällt (ein GiB aktuell).

Es wird aktuell eine falsche Definition von "freiem RAM" genutzt
(Festplattencaching wird als verwendet gezählt, obwohl es nicht ist;
siehe [diese berühmte Seite](https://www.linuxatemyram.com/) für
mehr), weswegen es unnötigerweise anschlägt.  Obwohl das besser ist
als umgekehrt, ist aus diesem Grund es aktuell standardmäßig aus.

Dies nutzt die *NIX Funktion `sysconf(3)`, wird also auf veralteten
Plattformen nicht funktionieren.

## Konfigurationsdatei
`legacylisten` kann mittels `~/.zvavybir/legacylisten/conffile.csv`
konfiguriert werden.  Trotz der Dateiendung ist es *keine* richtige
CSV-Datei, nur sehr stark daran angelehnt.  Wenn eine Option nicht
geparst werden kann, wird sie still verworfen, pass also auf.  Jede
Option hat eine eigene Zeile (mit verpflichtendem Newline-Zeichen am
Ende, sogar für die letzte Zeile und unter MS Windows) und jeder Teil
ist mit Kommata getrennt (und jede Zeile muss mit einem Komma enden).
Als ein Beispiel, hier ist meine Konfigurationsdatei:
```
data_dir,/media/my_user_name/external_harddrive/legacylisten,
ignore_ram,false,
lang,german,
```

Es gibt aktuell vier mögliche Optionen:
* `data_dir`: Wenn deine Musiksammlung woanders ist (z.b. wie bei mir
  auf einer externen Festplatte oder in `~/Musik`), kann diese Option
  genutzt werden, um das Verzeichnis, dass `legacylisten` durchsucht,
  zu ändern.  Die `~/` Notation ist sogar unter *NIX Systemen nicht in
  der Konfigurationsdatei verwendbar.
* `minimum_ram`: Das Limit für den [low memory
  handler](#low-memory-handler) in Bytes.
* `ignore_ram`: Deaktiviert den low memory handler (mögliche Werte
  sind `true` *wahr* und `false` *falsch*).  Wenn diese Option gesetzt
  ist (aktuell Standard) wird `minimum_ram` ignoriert.
* `lang`: `legacylisten` unterstützt grundlegende
  Internationalisierung und dies ist die Option, um es zu aktivieren.
  Es sind aktuell drei Werte zugelassen:
  * `english`: Stellt die Sprache auf Englisch (das ist der Standard).
  * `german` oder `deutsch`: Stellt die Sprache auf Deutsch.
  * `custom`: Wenn man eine Übersetzungsdatei hat, sie aber nicht in
    dem offiziellen Quellcode aufgenommen ist (vielleicht weil sie
    noch im Entstehen begriffen ist, man nur kurz was ausprobieren
    will oder weil aus legalen Gründen nicht möglich ist sie unter
    `legacylisten`s [license](#lizenz) zu veröffentlichen),
    ermöglicht diese Option, sie trotzdem zu nutzen.  Diese Option
    braucht zwei weitere Werte, den Pfad zur Datei und die Sprach-ID.
    Als Beispiel, wäre Englisch nicht bereits unterstützt, könnte man
    es so umgehen:
	```
	lang,custom,/pfad/zur/übersetzung.fl,en-US,
	```
    Der Pfad hat keine Anforderungen über Dateiname oder Dateiendung,
    die Sprach-ID muss aber korrekt sein.

## Mithelfen
Wie jedes Programm auch `legacylisten` kann immer verbessert werden.
Obwohl ich auch alleine versuche, es nutzbar zu machen, habe ich nicht
unbegrenzt Zeit und insbesondere nicht immer die besten Ideen.  Wenn
du damit oder mit etwas anderes (wie ein Featurevorschlag, einer
weiteren Sprache oder Dokumentationsverbesserungen) helfen kannst,
**helfe bitte mit**!

Ich nehme an, dass solange nichts anderes angegeben ist, alle Beiträge
unter der notwendigen Lizenz stehen.

## Lizenz
Obwohl unüblich für ein Rust Programm, steht `legacylisten` unter der
GNU General Public License Version 3 oder (deiner Wahl nach) jeder
späteren.

Für mehr siehe
[LICENSE.md](https://github.com/zvavybir/legacylisten/blob/master/LICENSE.md).

[^1]: Obwohl es nicht beabsichtigt war ([sogar im
	Gegenteil](https://www.fefe.de/nowindows/)), sollte `legacylisten`
	mehr oder weniger portabel sein (`~/` steht für das
	Benutzerverzeichnis – in `legacylisten` sogar unter MS Windows).

[^2]: Das ist natürlich ziemlich schlecht für den
    Arbeitsspeicherfußabdruck, aber es ist das beste was ich bisher
    machen konnte (zu mindestens ist es eine ganze Größenordnung
    besser als die schlimmste Implementation, die ich hatte).  Wenn du
    eine bessere Idee hast, **bitte** [helfe mit](#mithelfen)!

[^3]: Dieser Befehl ist ein bisschen speziell, da er intern anderes
    verarbeitet wird.  Man kann das zum einen an dem speziellen Namen
    (einziger Befehl ohne Buchstabe) sehen, andererseits (während man
    `legacylisten` ausführt) daran, dass obwohl Befehl üblicherweise
    streng in der angegebenen Reihenfolge ausgeführt werden, dieser
    vor allen anderen auf der selben Zeile ausgeführt wird.

[^4]: `legacylisten` versucht es aus den Metadaten der Audiodatei zu
    lesen oder – wenn das scheitert (was oft passiert, da die
    verwendete Routine sich noch in der Entwicklung zu befinden
    scheint) – dekodiert das ganze Lied ein zweites Mal um die Länge
    nach einer kurzen Wartezeit auf einem einfachen, aber teuren Weg
    zu bekommen.  Bis das behoben ist (wenn du eine Idee hast,
    **bitte** [helfe mit](#mithelfen)), würde ich nicht empfehlen
    mehrere Lieder in kurzer Abfolge zu überspringen, da pro Lied ein
    Thread zum Dekodieren gestartet wird – sogar nachdem bereits
    bekannt ist, dass das Ergebnis nicht benötigt werden wird.
