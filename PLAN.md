# Butaca — Plan de fases

Detente al final de cada fase para que Marco pruebe. Actualiza las casillas al terminar cada paso.

## Fase 1 — Núcleo sin UI
- [x] `Cargo.toml` con perfil release (`opt-level = "z"`, `lto`, `codegen-units = 1`, `strip`, `panic = "abort"`)
- [x] Módulo `config`: lee/crea `%APPDATA%\Butaca\config.json` con valores de ejemplo (host vacío)
- [x] Módulo `kodi`: cliente JSON-RPC (GetActivePlayers, GetItem, GetProperties, GetTVShowDetails)
- [x] Módulo `discord_ipc`: named pipe, handshake, SET_ACTIVITY con lectura de respuesta por nonce
- [x] Módulo `tmdb`: póster, año y directores, con caché en memoria
- [x] Módulo `presence`: construcción de la actividad y comparación con tolerancia de ±3 s
- [x] Bucle principal: polling de 3 s, reconexión a Discord cada 10 s, respaldo de `name`, limpieza al salir
- [x] Tests unitarios de `presence` y decodificación del arte de Kodi
- [x] `cargo build --release` y guía de prueba
- [x] **Prueba de Marco contra su Kodi real**

## Fase 2 — Bandeja del sistema
- [x] Sin consola (`windows_subsystem = "windows"`), event loop y hilo de trabajo con canales
- [x] Ícono en bandeja con tooltip de estado
- [x] Menú: estado, Configuración…, Pausar/Reanudar, Abrir carpeta de logs, Acerca de, Salir
- [x] Limpiar la actividad al salir
- [x] **Prueba de Marco**

## Fase 3 — Ventana de configuración
- [x] Ventana nativa con los campos de Kodi, TMDb y casillas
- [x] "Buscar Kodi en la red" (mDNS `_xbmc-jsonrpc-h._tcp.local.`)
- [x] "Probar conexión" (`JSONRPC.Ping` + `Application.GetProperties`)
- [x] Contraseña en Credential Manager (`keyring`); migra la que hubiera en `config.json` (decisión de Marco: sin contraseñas en texto plano)
- [x] "Iniciar con Windows" (`HKCU\...\Run`)
- [x] Guardar aplica los cambios sin reiniciar; primera ejecución abre la configuración
- [x] **Prueba de Marco**

## Fase 4 — Pulido
- [x] Instancia única (mutex con nombre)
- [x] Log con rotación (1 MB × 3)
- [x] Ícono embebido en el exe (`build.rs`)
- [ ] WebSocket 9090 opcional con polling de respaldo (aplazado: el polling funciona bien; se retoma si hace falta)
- [x] Manejo de errores claro
- [x] **Prueba de Marco**

## Fase 4b — Idiomas (en/es/fr/de/it)
- [x] `src/i18n.rs` + `locales/*.json` embebidos; inglés por defecto con respaldo al inglés
- [x] Bandeja, ventana, errores y texto de la presencia traducidos; logs en inglés
- [x] Selector de idioma en la ventana (vista previa; se aplica al guardar sin reiniciar)
- [x] Test que exige las mismas claves y marcadores en todos los idiomas
- [x] README en los 5 idiomas (`README.it.md` nuevo)
- [ ] **Prueba de Marco**: cambiar de idioma y revisar cómo muestra Discord la línea de temporada/episodio en cada uno

## Fase 5 — Repo
- [x] Identidad de git verificada: `noreply` de GitHub, solo en este repo
- [x] README en inglés (principal) + `README.es.md`, `README.fr.md`, `README.de.md`, con badges, capturas de la tarjeta y sección Legal
- [x] README: recomendar contraseña en el control remoto de Kodi y activar el anuncio de servicios para la búsqueda
- [x] `LICENSE` (MIT), `.gitignore`, `CHANGELOG.md`
- [x] `scripts/third_party_licenses.py` genera `THIRD-PARTY-LICENSES.txt` (se adjunta a cada Release)
- [x] Workflow de Releases (`.github/workflows/release.yml`)
- [x] `CLAUDE.md` se queda local (en `.gitignore`)
- [ ] Revisión de Marco de los README antes de publicar
- [ ] Primer commit, crear `Alexxlml/butaca` (público), push y topics (`kodi`, `discord`, `rich-presence`, `discord-rpc`, `coreelec`, `rust`, `windows`)
- [ ] Tag `v0.1.0` y comprobar el Release

## Pendiente para v0.2.0
- [ ] WebSocket 9090 (`tungstenite`): señal inmediata al worker en `Player.OnPlay/OnPause/OnResume/OnStop/OnSeek`, con debounce y el polling como respaldo
