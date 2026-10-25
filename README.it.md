# Butaca

[![Ultima versione](https://img.shields.io/github/v/release/Alexxlml/butaca?include_prereleases&label=versione)](https://github.com/Alexxlml/butaca/releases/latest)
[![Download](https://img.shields.io/github/downloads/Alexxlml/butaca/total)](https://github.com/Alexxlml/butaca/releases)
[![Download (ultima)](https://img.shields.io/github/downloads/Alexxlml/butaca/latest/total)](https://github.com/Alexxlml/butaca/releases/latest)
[![Licenza: MIT](https://img.shields.io/github/license/Alexxlml/butaca)](LICENSE)
![Piattaforma: Windows](https://img.shields.io/badge/piattaforma-Windows%2010%20%7C%2011-0078D4)
![Realizzato con Rust](https://img.shields.io/badge/realizzato%20con-Rust-CE422B)

[English](README.md) | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | **Italiano**

**Discord Rich Presence per Kodi (non ufficiale).** Butaca mostra sul tuo profilo Discord ciò che stai guardando su *qualsiasi* Kodi della tua rete locale (un box CoreELEC, un Android TV, un Raspberry Pi…) sotto forma di scheda «Sta guardando …».

<p>
  <img src="docs/img/card-movie.png" alt="Scheda Discord durante un film" width="300">
  &nbsp;
  <img src="docs/img/card-episode.png" alt="Scheda Discord durante un episodio" width="300">
  &nbsp;
  <img src="docs/img/card-paused.png" alt="Scheda Discord in pausa" width="300">
</p>

*Schede di un film, di un episodio di una serie e di un episodio in pausa. Discord mostra il titolo, la locandina, una barra di avanzamento e, per gli episodi, la stagione e il numero dell'episodio.*

## Perché gira sul PC e non su Kodi?

Discord espone la sua interfaccia Rich Presence solo sul computer in cui è in esecuzione. Butaca quindi gira sul tuo PC Windows, chiede a Kodi in rete cosa sta riproducendo e lo pubblica nel Discord locale. Kodi stesso non viene mai modificato.

È un unico piccolo `.exe` (senza installer, senza .NET né altri runtime) che resta nell'area di notifica.

## Funzionalità

- Funziona con qualsiasi Kodi raggiungibile dal tuo PC; non serve nessun add-on di Kodi.
- Schede per film, episodi e «altri video», con locandina, barra di avanzamento e stato di pausa.
- Pulsanti **Cerca Kodi nella rete** (mDNS/Zeroconf) e **Testa connessione**.
- **Aggiornamento istantaneo:** Butaca ascolta le notifiche di Kodi (WebSocket, porta 9090), così riproduzione, pausa e spostamenti compaiono subito su Discord. Se la porta non è disponibile, controlla ogni pochi secondi.
- Chiave [TMDb](https://www.themoviedb.org/) facoltativa per locandina, anno e registi quando Kodi non li ha.
- Interfaccia e testo della presenza in 5 lingue (inglese, spagnolo, francese, tedesco, italiano).
- Avvio con Windows (facoltativo), istanza unica, log con rotazione, impostazioni applicate senza riavviare.
- La password di Kodi è salvata in **Gestione credenziali di Windows**, non in un file.
- Nessuna telemetria, nessun account, nessuna pubblicità.

## Installazione

Butaca richiede **Windows 10 o 11 (x64)** e l'**app desktop di Discord** (non la versione web).

1. Scarica `Butaca-vX.Y.Z-x64.exe` dalla [pagina delle Release](https://github.com/Alexxlml/butaca/releases/latest).
2. *(Facoltativo)* Verifica il download con il file `.sha256` pubblicato accanto:
   ```powershell
   Get-FileHash .\Butaca-vX.Y.Z-x64.exe -Algorithm SHA256
   ```
3. Metti l'`.exe` dove preferisci e aprilo. La prima volta la finestra delle impostazioni si apre da sola.

> **Windows SmartScreen:** l'eseguibile non è firmato digitalmente, quindi Windows può mostrare l'avviso «Il PC è stato protetto da Windows». Scegli *Ulteriori informazioni → Esegui comunque* se ti fidi del download (confronta lo SHA-256 qui sopra, oppure compilalo tu stesso, vedi più avanti).

## Configurare Kodi

In Kodi apri **Impostazioni → Servizi → Controllo** e attiva:

- **Consenti il controllo remoto tramite HTTP** (porta `8080` predefinita).
- **Imposta un nome utente e una password.** Kodi te lo consiglia quando attivi il controllo remoto; seguilo. Altrimenti chiunque nella tua rete potrebbe controllare il tuo Kodi. Butaca salva la password in Gestione credenziali di Windows.
- **Consenti il controllo remoto da applicazioni su altri sistemi.**
- *(Solo per il pulsante «Cerca Kodi nella rete»)* **Annuncia questi servizi ad altri sistemi.**

Poi, nelle impostazioni di Butaca, scrivi l'indirizzo IP del tuo Kodi (oppure premi **Cerca Kodi nella rete**), il nome utente e la password, e premi **Testa connessione**. Salva e hai finito.

Butaca non modifica mai le impostazioni di Kodi. È stato provato con Kodi 22 su CoreELEC.

L'aggiornamento istantaneo usa il WebSocket di Kodi sulla porta `9090`, che si attiva insieme al controllo remoto da altri sistemi. **Testa connessione** indica se è disponibile; in caso contrario, Butaca continua a funzionare con il polling. Per disattivarlo o cambiare la porta, modifica `use_websocket` e `websocket_port` in `config.json`.

## Uso

- Butaca gira nell'**area di notifica**. Fai doppio clic sull'icona per aprire le impostazioni.
- Clic destro sull'icona per il menu: stato, Impostazioni…, Metti in pausa / Riprendi la presenza, Apri la cartella dei log, Cerca aggiornamenti…, Informazioni (mostra la versione installata), Esci.
- All'uscita il tuo stato Discord viene cancellato.
- Se apri Butaca una seconda volta, si apre soltanto la finestra delle impostazioni di quella già in esecuzione.

**Impostazioni**

| Opzione | Cosa fa |
|---|---|
| Indirizzo, porta, utente e password di Kodi | Come raggiungere Kodi. L'indirizzo resta vuoto finché non lo scrivi. |
| Chiave API di TMDb (facoltativa) | Locandine migliori, anno e registi. [Ottieni una chiave gratuita](https://www.themoviedb.org/settings/api). |
| Avvia con Windows | Aggiunge Butaca alle app di avvio del tuo utente. |
| Mostra lo stato quando non c'è nulla in riproduzione | Mostra una scheda neutra mentre Kodi è inattivo. Disattivata di default (lo stato viene cancellato). |
| Lingua | English, Español, Français, Deutsch o Italiano. Cambia l'interfaccia **e** il testo che Butaca pubblica su Discord (per esempio «In pausa» o «Stagione 01, episodio 05»). Predefinita: inglese. |

I file si trovano in `%APPDATA%\Butaca\`: `config.json` (senza la password) e `butaca.log` (con rotazione, 1 MB × 3).

## Aggiornare e disinstallare

**Aggiornare:** menu dell'area di notifica → *Cerca aggiornamenti…* apre la pagina dell'ultima versione. Scarica il nuovo `.exe`, chiudi Butaca (menu → Esci) e sostituisci il vecchio file. Le tue impostazioni vengono conservate. Confronta la versione mostrata in *Informazioni* con quella della pagina delle Releases.

**Disinstallare:** deseleziona *Avvia con Windows* nelle impostazioni, chiudi Butaca ed elimina l'`.exe`. Per non lasciare tracce, elimina anche la cartella `%APPDATA%\Butaca\` e, se hai salvato una password di Kodi, la voce `kodi-password.Butaca` in *Gestione credenziali* di Windows.

## Come viene scelta la locandina

1. L'immagine che Kodi ha già, se è un URL `https` pubblico.
2. [TMDb](https://www.themoviedb.org/), se imposti una chiave API.
3. L'icona di Butaca.

Discord non può mostrare percorsi locali né immagini `smb://`, quindi vengono saltati. Gli URL delle locandine li scarica Discord, non Butaca.

## Privacy e sicurezza

- Butaca comunica con: il tuo Kodi (HTTP e WebSocket sulla rete locale), Discord (IPC locale sul tuo PC) e, **solo se imposti una chiave**, TMDb.
- Ciò che guardi viene pubblicato sul tuo profilo Discord, come prevede Rich Presence. Usa *Metti in pausa la presenza* nel menu dell'area di notifica per fermarlo.
- La password di Kodi è in Gestione credenziali di Windows. La chiave di TMDb è salvata in `config.json` in chiaro; è una chiave gratuita legata al tuo account.
- Il login HTTP di Kodi viaggia non cifrato sulla tua rete. Tieni il controllo remoto di Kodi nella rete domestica e non esporlo mai su Internet.
- Il binario non contiene nessuna chiave API; l'unico valore incorporato è l'ID pubblico dell'applicazione Discord.

## Risoluzione dei problemi

| Sintomo | Cosa controllare |
|---|---|
| «Discord non è aperto» | Apri l'app **desktop** di Discord con l'accesso effettuato. Butaca e Discord devono girare con lo stesso utente Windows e lo stesso livello di privilegi (entrambi normali, non «come amministratore»). |
| «In attesa di Kodi» | Kodi è spento, in standby o l'indirizzo è sbagliato. Usa **Testa connessione**. |
| «Nome utente o password errati» | Usa le credenziali della pagina *Servizi → Controllo* di Kodi. |
| «Cerca Kodi nella rete» non trova nulla | Attiva *Annuncia questi servizi ad altri sistemi* in Kodi, oppure scrivi l'IP a mano. |
| Non compare nulla su Discord | In Discord, *Impostazioni → Privacy dell'attività* → *Condividi la mia attività* deve essere attivo. |

Il log (menu dell'area di notifica → *Apri la cartella dei log*) registra ogni cambio di stato.

## Compilare dal codice sorgente

Servono [Rust](https://rustup.rs/) (stable, toolchain MSVC) e il Windows SDK.

```powershell
git clone https://github.com/Alexxlml/butaca.git
cd butaca
cargo build --release
```

L'eseguibile è `target\release\butaca.exe`. Con `cargo test` si eseguono i test unitari. Per aggiungere una lingua basta creare `locales/xx.json` con le stesse chiavi di `en.json` e registrarlo in `src/i18n.rs`.

## Prossimamente

- Scaricare e installare gli aggiornamenti dall'app stessa.

## Note legali

Kodi® is a registered trademark of the XBMC Foundation. Discord is a trademark of Discord Inc. This project is not affiliated with, endorsed by, or sponsored by either.

This product uses the TMDB API but is not endorsed or certified by TMDB.

Butaca è distribuito con [licenza MIT](LICENSE). Le licenze di terze parti sono elencate in `THIRD-PARTY-LICENSES.txt`, pubblicato con ogni versione.

## Crediti

- Il protocollo IPC di Discord segue [service.discord.richpresence](https://github.com/Hiumee/service.discord.richpresence) di Hiumee (MIT).
- L'esperienza di app nell'area di notifica è ispirata a [AMWin-RP](https://github.com/PKBeam/AMWin-RP).
