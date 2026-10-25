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
- [x] **Prueba de Marco**: cambiar de idioma y revisar cómo muestra Discord la línea de temporada/episodio en cada uno (24-oct-2026; en francés solo funciona `Saison 02, episode 01`)

## Fase 5 — Repo
- [x] Identidad de git verificada: `noreply` de GitHub, solo en este repo
- [x] README en inglés (principal) + `README.es.md`, `README.fr.md`, `README.de.md`, con badges, capturas de la tarjeta y sección Legal
- [x] README: recomendar contraseña en el control remoto de Kodi y activar el anuncio de servicios para la búsqueda
- [x] `LICENSE` (MIT), `.gitignore`, `CHANGELOG.md`
- [x] `scripts/third_party_licenses.py` genera `THIRD-PARTY-LICENSES.txt` (se adjunta a cada Release)
- [x] Workflow de Releases (`.github/workflows/release.yml`)
- [x] `CLAUDE.md` se queda local (en `.gitignore`)
- [x] Revisión de Marco de los README antes de publicar
- [x] Primer commit, crear `Alexxlml/butaca` (público), push y topics (`kodi`, `discord`, `rich-presence`, `discord-rpc`, `coreelec`, `rust`, `windows`)
- [x] Tag `v0.1.0` y comprobar el Release (publicado el 24-oct-2026: `.exe`, `.sha256` y `THIRD-PARTY-LICENSES.txt`; los tests pasaron en CI)

## WebSocket (entra en v0.1.0)
- [x] WebSocket 9090 (`tungstenite`, `src/kodi_ws.rs`): señal inmediata al worker en `Player.OnPlay/OnPause/OnResume/OnStop/OnSeek/OnSpeedChanged`, con debounce de 400 ms; con el WebSocket abierto el polling baja a 15 s como respaldo. Opciones en `config.json`: `use_websocket` (true) y `websocket_port` (9090)
- [x] **Prueba de Marco** del WebSocket contra su Kodi real

## Después de v0.1.0
- [x] "Buscar actualizaciones…" en la bandeja (abre `releases/latest`) y versión en "Acerca de"
- [x] README: tipos de medios compatibles (películas y episodios), actualizar y desinstalar
- [ ] Botón que descargue y reemplace el exe (consultar `api.github.com/repos/Alexxlml/butaca/releases/latest`, verificar el `.sha256`, script auxiliar para reemplazar el exe en uso)
- [x] `actions/checkout@v7` y ejecución manual del workflow (Actions → Release → Run workflow) para probar tests, build y empaquetado sin publicar; deja el resultado como artefacto de 3 días
