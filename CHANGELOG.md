# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-10-24

First public version.

### Added
- Instant updates through Kodi's WebSocket notifications (port 9090, on by default), with polling as a fallback; "Test connection" reports whether it is available.
- "Check for updates…" in the tray menu opens the latest GitHub Release; "About" shows the installed version.
- `config.json` is refreshed automatically when a new version adds options.
- Discord Rich Presence for any Kodi on the local network, with cards for movies, TV episodes and other videos, poster, progress bar and pause state.
- System tray app: status tooltip, settings, pause/resume presence, open log folder, about, exit.
- Settings window: Kodi address/port/user/password, "Find Kodi on the network" (mDNS), "Test connection", optional TMDb API key, start with Windows, idle status.
- Kodi password stored in the Windows Credential Manager (migrated automatically from older `config.json` files).
- Interface and Discord text in English (default), Spanish, French, German and Italian, selectable in the settings (`locales/*.json`, embedded in the exe).
- Single instance, rotating log (1 MB × 3), settings applied without restarting.
