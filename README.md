# Butaca

[![Latest release](https://img.shields.io/github/v/release/Alexxlml/butaca?include_prereleases&label=release)](https://github.com/Alexxlml/butaca/releases/latest)
[![Downloads](https://img.shields.io/github/downloads/Alexxlml/butaca/total)](https://github.com/Alexxlml/butaca/releases)
[![Downloads (latest)](https://img.shields.io/github/downloads/Alexxlml/butaca/latest/total)](https://github.com/Alexxlml/butaca/releases/latest)
[![License: MIT](https://img.shields.io/github/license/Alexxlml/butaca)](LICENSE)
![Platform: Windows](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D4)
![Built with Rust](https://img.shields.io/badge/built%20with-Rust-CE422B)

**English** | [Español](README.es.md) | [Français](README.fr.md) | [Deutsch](README.de.md) | [Italiano](README.it.md)

**Discord Rich Presence for Kodi (unofficial).** Butaca shows what you are watching on *any* Kodi on your local network — a CoreELEC box, an Android TV, a Raspberry Pi — as a "Watching …" card on your Discord profile.

<p>
  <img src="docs/img/card-movie.png" alt="Discord card while watching a movie" width="300">
  &nbsp;
  <img src="docs/img/card-episode.png" alt="Discord card while watching a TV episode" width="300">
  &nbsp;
  <img src="docs/img/card-paused.png" alt="Discord card while paused" width="300">
</p>

*Cards for a movie, a TV episode and a paused episode. Discord shows the title, the poster, a progress bar and, for episodes, the season and episode number.*

## Why does it run on the PC and not on Kodi?

Discord only exposes its Rich Presence interface on the computer where Discord is running. Butaca therefore runs on your Windows PC, asks Kodi over the network what is playing, and publishes it to the local Discord. Kodi itself is not modified in any way.

It is a single small `.exe` (no installer, no .NET or other runtime) that lives in the system tray.

## Features

- Works with any Kodi reachable from your PC; no Kodi add-on needed.
- Movie, TV episode and "other video" cards, with poster, progress bar and pause state.
- **Find Kodi on the network** (mDNS/Zeroconf) and **Test connection** buttons.
- Optional [TMDb](https://www.themoviedb.org/) key for posters, release year and directors when Kodi does not have them.
- Interface and Discord text in 5 languages: English, Español, Français, Deutsch and Italiano.
- Starts with Windows (optional), single instance, rotating log, applies settings without restarting.
- Your Kodi password is stored in the **Windows Credential Manager**, not in a file.
- No telemetry, no accounts, no ads.

## Installation

Butaca needs **Windows 10 or 11 (x64)** and the **Discord desktop app** (not the web version).

1. Download `Butaca-vX.Y.Z-x64.exe` from the [Releases page](https://github.com/Alexxlml/butaca/releases/latest).
2. *(Optional)* Check the download against the `.sha256` file published next to it:
   ```powershell
   Get-FileHash .\Butaca-vX.Y.Z-x64.exe -Algorithm SHA256
   ```
3. Put the `.exe` anywhere you like and open it. The settings window opens by itself the first time.

> **Windows SmartScreen:** the executable is not code-signed, so Windows may show a "protected your PC" warning. Choose *More info → Run anyway* if you trust the download (compare the SHA-256 above, or build it yourself, see below).

## Setting up Kodi

In Kodi, go to **Settings → Services → Control** and enable:

- **Allow remote control via HTTP** (default port `8080`).
- **Set a username and a password.** Kodi warns you about this when you enable remote control; please follow that advice. Anyone on your network could otherwise control your Kodi. Butaca stores the password in the Windows Credential Manager.
- **Allow remote control from applications on other systems.**
- *(Only for the "Find Kodi on the network" button)* **Announce these services to other systems.**

Then, in Butaca's settings, type your Kodi's IP address (or press **Find Kodi on the network**), enter the username and password, and press **Test connection**. Save, and you are done.

Butaca never changes Kodi's settings. It was tested with Kodi 22 on CoreELEC.

## Using Butaca

- Butaca runs in the **system tray**. Double-click the icon to open the settings.
- Right-click the icon for the menu: status, Settings…, Pause/Resume presence, Open log folder, About, Exit.
- When you exit, your Discord status is cleared.
- Opening Butaca a second time just brings up the settings of the running one.

**Settings**

| Option | What it does |
|---|---|
| Kodi address, port, user, password | How to reach Kodi. The address is empty until you set it. |
| TMDb API key (optional) | Better posters, year and directors. [Get a free key](https://www.themoviedb.org/settings/api). |
| Start with Windows | Adds Butaca to your user's startup apps. |
| Show status when nothing is playing | Shows a neutral card while Kodi is idle. Off by default (the status is cleared). |
| Language | English, Español, Français, Deutsch or Italiano. It changes the interface **and** the text Butaca publishes on Discord (for example "Paused" or "Season 01, episode 05"). Default: English. |

Files live in `%APPDATA%\Butaca\`: `config.json` (no password inside) and `butaca.log` (rotated, 1 MB × 3).

## How the poster is chosen

1. The artwork Kodi already has, when it is a public `https` URL.
2. [TMDb](https://www.themoviedb.org/), when you set an API key.
3. Butaca's own icon.

Local paths and `smb://` artwork cannot be shown by Discord, so they are skipped. Poster URLs are fetched by Discord, not by Butaca.

## Privacy and security

- Butaca talks to: your Kodi (HTTP on your local network), Discord (local IPC on your PC) and, **only if you set a key**, TMDb.
- What you are watching is published to your Discord profile, as Rich Presence does by design. Use *Pause presence* in the tray menu to stop it.
- The Kodi password is stored in the Windows Credential Manager. The TMDb key is saved in `config.json` as plain text; it is a free key tied to your own account.
- Kodi's HTTP login travels unencrypted on your network. Keep Kodi's remote control inside your home network and never expose it to the internet.
- The binary contains no API keys; the only built-in value is the public Discord application ID.

## Troubleshooting

| Symptom | What to check |
|---|---|
| "Discord is not open" | Open the Discord **desktop** app, signed in. Butaca and Discord must run as the same Windows user and the same privilege level (both normal, not "as administrator"). |
| "Waiting for Kodi" | Kodi is off, asleep, or the address is wrong. Use **Test connection**. |
| "Wrong username or password" | Use the credentials from Kodi's *Services → Control* page. |
| "Find Kodi" finds nothing | Enable *Announce these services to other systems* in Kodi, or type the IP by hand. |
| No presence in Discord | In Discord, *Settings → Activity Privacy* → *Share my activity* must be on. |

The log (tray menu → *Open log folder*) records every state change.

## Build from source

You need [Rust](https://rustup.rs/) (stable, MSVC toolchain) and the Windows SDK.

```powershell
git clone https://github.com/Alexxlml/butaca.git
cd butaca
cargo build --release
```

The executable is `target\release\butaca.exe`. Run `cargo test` for the unit tests. To add a language, create `locales/xx.json` with the same keys as `en.json` and register it in `src/i18n.rs`.

## Roadmap

- Instant updates through Kodi's WebSocket notifications (port 9090), keeping polling as a fallback.

## Legal

Kodi® is a registered trademark of the XBMC Foundation. Discord is a trademark of Discord Inc. This project is not affiliated with, endorsed by, or sponsored by either.

This product uses the TMDB API but is not endorsed or certified by TMDB.

Butaca is released under the [MIT License](LICENSE). Third-party licenses are listed in `THIRD-PARTY-LICENSES.txt`, published with every release.

## Credits

- The Discord IPC protocol follows [service.discord.richpresence](https://github.com/Hiumee/service.discord.richpresence) by Hiumee (MIT).
- The tray-app experience is inspired by [AMWin-RP](https://github.com/PKBeam/AMWin-RP).
