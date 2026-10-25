# Butaca

[![Neueste Version](https://img.shields.io/github/v/release/Alexxlml/butaca?include_prereleases&label=version)](https://github.com/Alexxlml/butaca/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/Alexxlml/butaca/total)](https://github.com/Alexxlml/butaca/releases)
[![Downloads (neueste)](https://img.shields.io/github/downloads/Alexxlml/butaca/latest/total)](https://github.com/Alexxlml/butaca/releases/latest)
[![Lizenz: MIT](https://img.shields.io/github/license/Alexxlml/butaca)](LICENSE)
![Plattform: Windows](https://img.shields.io/badge/plattform-Windows%2010%20%7C%2011-0078D4)
![Gebaut mit Rust](https://img.shields.io/badge/gebaut%20mit-Rust-CE422B)

[English](README.md) | [Español](README.es.md) | [Français](README.fr.md) | **Deutsch** | [Italiano](README.it.md)

**Discord Rich Presence für Kodi (inoffiziell).** Butaca zeigt in deinem Discord-Profil, was du auf *irgendeinem* Kodi in deinem lokalen Netzwerk ansiehst (CoreELEC-Box, Android TV, Raspberry Pi …), als „Schaut …"-Karte.

<p>
  <img src="docs/img/card-movie.png" alt="Discord-Karte bei einem Film" width="300">
  &nbsp;
  <img src="docs/img/card-episode.png" alt="Discord-Karte bei einer Serienfolge" width="300">
  &nbsp;
  <img src="docs/img/card-paused.png" alt="Discord-Karte im Pausenzustand" width="300">
</p>

*Karten für einen Film, eine Serienfolge und eine pausierte Folge. Discord zeigt Titel, Poster, einen Fortschrittsbalken und bei Folgen Staffel und Folgennummer.*

## Warum läuft es auf dem PC und nicht auf Kodi?

Discord stellt seine Rich-Presence-Schnittstelle nur auf dem Computer bereit, auf dem es läuft. Butaca läuft deshalb auf deinem Windows-PC, fragt Kodi über das Netzwerk, was gerade abgespielt wird, und veröffentlicht es im lokalen Discord. Kodi selbst wird nie verändert.

Es ist eine einzige kleine `.exe` (ohne Installer, ohne .NET oder andere Laufzeitumgebung), die im Infobereich der Taskleiste läuft.

## Funktionen

- Funktioniert mit jedem Kodi, das dein PC erreicht; ein Kodi-Add-on ist nicht nötig.
- Karten für Filme, Serienfolgen und „andere Videos" mit Poster, Fortschrittsbalken und Pausenstatus.
- Schaltflächen **Kodi im Netzwerk suchen** (mDNS/Zeroconf) und **Verbindung testen**.
- **Sofortige Aktualisierung:** Butaca hört auf die Benachrichtigungen von Kodi (WebSocket, Port 9090), sodass Wiedergabe, Pause und Spulen sofort in Discord erscheinen. Ist der Port nicht erreichbar, fragt Butaca stattdessen alle paar Sekunden ab.
- Optionaler [TMDb](https://www.themoviedb.org/)-Schlüssel für Poster, Erscheinungsjahr und Regie, wenn Kodi sie nicht hat.
- Oberfläche und Discord-Text in 5 Sprachen: English, Español, Français, Deutsch und Italiano.
- Start mit Windows (optional), nur eine Instanz, rotierendes Protokoll, Einstellungen werden ohne Neustart übernommen.
- Das Kodi-Passwort liegt in der **Windows-Anmeldeinformationsverwaltung**, nicht in einer Datei.
- Keine Telemetrie, keine Konten, keine Werbung.

## Unterstützte Medientypen

| Medientyp | Was Discord anzeigt |
|---|---|
| Filme | Titel, Jahr und Regisseur, Genres, Poster, Fortschrittsbalken |
| Serienepisoden | Serie, Episodentitel, Staffel und Episodennummer, Poster, Fortschrittsbalken |

Andere Videodateien zeigen eine einfache Karte mit dem Titel.

## Installation

Butaca benötigt **Windows 10 oder 11 (x64)** und die **Discord-Desktop-App** (nicht die Web-Version).

1. Lade `Butaca-vX.Y.Z-x64.exe` von der [Releases-Seite](https://github.com/Alexxlml/butaca/releases/latest) herunter.
2. *(Optional)* Prüfe den Download mit der daneben veröffentlichten `.sha256`-Datei:
   ```powershell
   Get-FileHash .\Butaca-vX.Y.Z-x64.exe -Algorithm SHA256
   ```
3. Lege die `.exe` an einen beliebigen Ort und starte sie. Beim ersten Start öffnet sich das Einstellungsfenster von selbst.

> **Windows SmartScreen:** Die ausführbare Datei ist nicht digital signiert, daher kann Windows die Warnung „Der Computer wurde durch Windows geschützt" anzeigen. Wähle *Weitere Informationen → Trotzdem ausführen*, wenn du dem Download vertraust (vergleiche die SHA-256 oben oder baue das Programm selbst, siehe unten).

## Kodi einrichten

Öffne in Kodi **Einstellungen → Dienste → Steuerung** und aktiviere:

- **Fernsteuerung per HTTP erlauben** (Standardport `8080`).
- **Lege Benutzername und Passwort fest.** Kodi weist beim Aktivieren der Fernsteuerung darauf hin; befolge das. Sonst könnte jeder in deinem Netzwerk dein Kodi steuern. Butaca speichert das Passwort in der Windows-Anmeldeinformationsverwaltung.
- **Fernsteuerung durch Anwendungen auf anderen Systemen erlauben.**
- *(Nur für die Schaltfläche „Kodi im Netzwerk suchen")* **Diese Dienste anderen Systemen bekanntgeben.**

Trage dann in den Butaca-Einstellungen die IP-Adresse deines Kodi ein (oder klicke auf **Kodi im Netzwerk suchen**), dazu Benutzername und Passwort, und klicke auf **Verbindung testen**. Speichern, fertig.

Butaca ändert nie die Einstellungen von Kodi. Getestet wurde mit Kodi 22 auf CoreELEC.

Die sofortige Aktualisierung nutzt den WebSocket von Kodi auf Port `9090`, der zusammen mit der Fernsteuerung durch andere Systeme aktiviert wird. **Verbindung testen** zeigt, ob er erreichbar ist; falls nicht, arbeitet Butaca per Abfrage weiter. Zum Abschalten oder um den Port zu ändern, bearbeite `use_websocket` und `websocket_port` in `config.json`.

## Benutzung

- Butaca läuft im **Infobereich der Taskleiste**. Doppelklick auf das Symbol öffnet die Einstellungen.
- Rechtsklick auf das Symbol öffnet das Menü: Status, Einstellungen…, Präsenz pausieren/fortsetzen, Protokollordner öffnen, Nach Updates suchen…, Info (zeigt die installierte Version), Beenden.
- Beim Beenden wird dein Discord-Status gelöscht.
- Startest du Butaca ein zweites Mal, öffnet sich nur das Einstellungsfenster der laufenden Instanz.

**Einstellungen**

| Option | Wirkung |
|---|---|
| Kodi-Adresse, Port, Benutzer, Passwort | So wird Kodi erreicht. Die Adresse bleibt leer, bis du sie einträgst. |
| TMDb-API-Schlüssel (optional) | Bessere Poster, Jahr und Regie. [Kostenlosen Schlüssel holen](https://www.themoviedb.org/settings/api). |
| Mit Windows starten | Fügt Butaca den Autostart-Apps deines Benutzers hinzu. |
| Status anzeigen, wenn nichts abgespielt wird | Zeigt eine neutrale Karte, solange Kodi im Leerlauf ist. Standardmäßig aus (der Status wird gelöscht). |
| Sprache | English, Español, Français, Deutsch oder Italiano. Sie ändert die Oberfläche **und** den Text, den Butaca auf Discord veröffentlicht (z. B. „Pausiert“ oder „Staffel 01, Folge 05“). Standard: Englisch. |

Die Dateien liegen in `%APPDATA%\Butaca\`: `config.json` (ohne Passwort) und `butaca.log` (rotierend, 1 MB × 3).

## Aktualisieren und deinstallieren

**Aktualisieren:** Menü im Infobereich → *Nach Updates suchen…* öffnet die Seite der neuesten Version. Lade die neue `.exe` herunter, beende Butaca (Menü → Beenden) und ersetze die alte Datei. Deine Einstellungen bleiben erhalten. Vergleiche die in *Info* angezeigte Version mit der auf der Releases-Seite.

**Deinstallieren:** Entferne in den Einstellungen das Häkchen bei *Mit Windows starten*, beende Butaca und lösche die `.exe`. Um alle Spuren zu entfernen, lösche außerdem den Ordner `%APPDATA%\Butaca\` und, falls du ein Kodi-Passwort gespeichert hast, den Eintrag `kodi-password.Butaca` in der Windows-*Anmeldeinformationsverwaltung*.

## Wie das Poster ausgewählt wird

1. Die Grafik, die Kodi bereits hat, wenn es eine öffentliche `https`-URL ist.
2. [TMDb](https://www.themoviedb.org/), wenn du einen API-Schlüssel einträgst.
3. Das eigene Symbol von Butaca.

Discord kann lokale Pfade und `smb://`-Grafiken nicht anzeigen; sie werden übersprungen. Die Poster-URLs lädt Discord, nicht Butaca.

## Datenschutz und Sicherheit

- Butaca kommuniziert mit: deinem Kodi (HTTP und WebSocket im lokalen Netzwerk), Discord (lokale IPC auf deinem PC) und, **nur wenn du einen Schlüssel einträgst**, mit TMDb.
- Was du ansiehst, wird in deinem Discord-Profil veröffentlicht – so funktioniert Rich Presence. Mit *Präsenz pausieren* im Taskleistenmenü stoppst du das.
- Das Kodi-Passwort liegt in der Windows-Anmeldeinformationsverwaltung. Der TMDb-Schlüssel steht im Klartext in `config.json`; es ist ein kostenloser Schlüssel, der an dein eigenes Konto gebunden ist.
- Die HTTP-Anmeldung von Kodi läuft unverschlüsselt durch dein Netzwerk. Halte die Fernsteuerung von Kodi im Heimnetz und mache sie niemals im Internet erreichbar.
- Die Binärdatei enthält keine API-Schlüssel; der einzige eingebaute Wert ist die öffentliche Anwendungs-ID von Discord.

## Fehlerbehebung

| Symptom | Was zu prüfen ist |
|---|---|
| „Discord ist nicht geöffnet“ | Öffne die **Desktop**-App von Discord und melde dich an. Butaca und Discord müssen mit demselben Windows-Benutzer und derselben Berechtigungsstufe laufen (beide normal, nicht „als Administrator"). |
| „Warten auf Kodi“ | Kodi ist aus, im Standby oder die Adresse stimmt nicht. Nutze **Verbindung testen**. |
| „Benutzername oder Passwort falsch“ | Verwende die Zugangsdaten von *Dienste → Steuerung* in Kodi. |
| „Kodi im Netzwerk suchen“ findet nichts | Aktiviere in Kodi *Diese Dienste anderen Systemen bekanntgeben* oder trage die IP von Hand ein. |
| In Discord erscheint nichts | In Discord muss unter *Einstellungen → Aktivitätsdatenschutz* die Option *Meine Aktivität teilen* aktiv sein. |

Das Protokoll (Taskleistenmenü → *Protokollordner öffnen*) hält jede Zustandsänderung fest.

## Aus dem Quellcode bauen

Du brauchst [Rust](https://rustup.rs/) (stable, MSVC-Toolchain) und das Windows SDK.

```powershell
git clone https://github.com/Alexxlml/butaca.git
cd butaca
cargo build --release
```

Die ausführbare Datei ist `target\release\butaca.exe`. Mit `cargo test` laufen die Unit-Tests.

## Ausblick

- Updates direkt in der App herunterladen und installieren.

## Rechtliches

Kodi® is a registered trademark of the XBMC Foundation. Discord is a trademark of Discord Inc. This project is not affiliated with, endorsed by, or sponsored by either.

This product uses the TMDB API but is not endorsed or certified by TMDB.

Butaca steht unter der [MIT-Lizenz](LICENSE). Die Lizenzen von Drittanbietern stehen in `THIRD-PARTY-LICENSES.txt`, das mit jeder Version veröffentlicht wird.

## Danksagung

- Das Discord-IPC-Protokoll folgt [service.discord.richpresence](https://github.com/Hiumee/service.discord.richpresence) von Hiumee (MIT).
- Das Konzept der Taskleisten-App ist von [AMWin-RP](https://github.com/PKBeam/AMWin-RP) inspiriert.
