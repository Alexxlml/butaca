"""
kodi_discord_bridge.py — Puente Kodi (en red) -> Discord Rich Presence (local).

Corre en la PC donde está Discord abierto. Consulta cada pocos segundos el
JSON-RPC HTTP de Kodi (CoreELEC, Android, etc.) y publica "Viendo ..." en tu
perfil a través del IPC local de Discord.

Sin dependencias externas: solo la biblioteca estándar de Python 3.8+.

Protocolo IPC basado en service.discord.richpresence de Hiumee (MIT).
"""

import base64
import json
import os
import socket
import struct
import sys
import time
import urllib.parse
import urllib.request
import uuid

# --------------------------------------------------------------------------
# Configuración (también se puede sobrescribir con variables de entorno)
# --------------------------------------------------------------------------
KODI_HOST = os.environ.get("KODI_HOST", "192.168.1.50")   # IP del Xiaomi Box
KODI_PORT = int(os.environ.get("KODI_PORT", "8080"))
KODI_USER = os.environ.get("KODI_USER", "kodi")
KODI_PASS = os.environ.get("KODI_PASS", "")

# Application ID de https://discord.com/developers/applications
# El nombre de la aplicación es lo que aparece como "Viendo <nombre>".
DISCORD_CLIENT_ID = os.environ.get("DISCORD_CLIENT_ID", "1563628762875302008")

# Opcional: API key (v3) de TMDb, para obtener póster cuando Kodi solo tiene arte local.
TMDB_API_KEY = os.environ.get("TMDB_API_KEY", "")

# Nombre de un asset subido en tu app de Discord (Rich Presence > Art Assets)
# que se usa si no hay póster público. Déjalo vacío si no subiste ninguno.
FALLBACK_IMAGE = os.environ.get("FALLBACK_IMAGE", "kodi")

POLL_SECONDS = float(os.environ.get("POLL_SECONDS", "3"))

# Intenta mostrar "Viendo <serie>" en lugar de "Viendo <nombre de la app>".
# Se desactiva solo si Discord rechaza el campo.
USE_NAME_OVERRIDE = os.environ.get("USE_NAME_OVERRIDE", "1") == "1"


def log(msg):
    print(time.strftime("%H:%M:%S"), msg, flush=True)


# --------------------------------------------------------------------------
# Cliente IPC de Discord
# --------------------------------------------------------------------------
OP_HANDSHAKE, OP_FRAME, OP_CLOSE = 0, 1, 2


class DiscordIPC:
    def __init__(self, client_id):
        self.client_id = client_id
        self._f = None
        self._sock = None

    # --- transporte -------------------------------------------------------
    def connect(self):
        if sys.platform == "win32":
            for i in range(10):
                try:
                    self._f = open(r"\\?\pipe\discord-ipc-{}".format(i), "r+b", buffering=0)
                    break
                except OSError:
                    continue
            else:
                raise ConnectionError("No se encontró el pipe de Discord (¿está abierto?)")
        else:
            base = (os.environ.get("XDG_RUNTIME_DIR") or os.environ.get("TMPDIR")
                    or os.environ.get("TMP") or os.environ.get("TEMP") or "/tmp")
            for i in range(10):
                path = os.path.join(base, "discord-ipc-{}".format(i))
                if not os.path.exists(path):
                    continue
                try:
                    s = socket.socket(socket.AF_UNIX)
                    s.connect(path)
                    self._sock = s
                    break
                except OSError:
                    continue
            else:
                raise ConnectionError("No se encontró el socket de Discord (¿está abierto?)")

        op, data = self._send_recv({"v": 1, "client_id": self.client_id}, OP_HANDSHAKE)
        if op != OP_FRAME or data.get("evt") != "READY":
            self.close()
            raise ConnectionError("Handshake rechazado: {}".format(data))
        user = (data.get("data") or {}).get("user") or {}
        log("Conectado a Discord como {}".format(user.get("username", "?")))

    def _write(self, b):
        if self._f:
            self._f.write(b)
            self._f.flush()
        else:
            self._sock.sendall(b)

    def _read(self, n):
        buf = b""
        while len(buf) < n:
            chunk = self._f.read(n - len(buf)) if self._f else self._sock.recv(n - len(buf))
            if not chunk:
                raise ConnectionError("Discord cerró la conexión")
            buf += chunk
        return buf

    def _send(self, payload, op=OP_FRAME):
        body = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        self._write(struct.pack("<II", op, len(body)) + body)

    def _recv(self):
        op, length = struct.unpack("<II", self._read(8))
        return op, json.loads(self._read(length).decode("utf-8"))

    def _send_recv(self, payload, op=OP_FRAME):
        self._send(payload, op)
        nonce = payload.get("nonce")
        while True:
            rop, data = self._recv()
            if rop == OP_CLOSE:
                raise ConnectionError("Discord cerró la conexión: {}".format(data))
            if data.get("nonce") == nonce:
                return rop, data

    # --- API --------------------------------------------------------------
    def set_activity(self, activity):
        args = {"pid": os.getpid()}
        if activity is not None:
            args["activity"] = activity
        _, data = self._send_recv({"cmd": "SET_ACTIVITY", "args": args, "nonce": str(uuid.uuid4())})
        if data.get("evt") == "ERROR":
            log("Discord rechazó la actividad: {}".format(data.get("data")))
            return False
        return True

    def close(self):
        try:
            if self._f:
                self._f.close()
            if self._sock:
                self._sock.close()
        except OSError:
            pass
        self._f = self._sock = None


# --------------------------------------------------------------------------
# Cliente JSON-RPC de Kodi
# --------------------------------------------------------------------------
class Kodi:
    def __init__(self, host, port, user, password):
        self.url = "http://{}:{}/jsonrpc".format(host, port)
        self.headers = {"Content-Type": "application/json"}
        if user:
            token = base64.b64encode("{}:{}".format(user, password).encode()).decode()
            self.headers["Authorization"] = "Basic " + token

    def call(self, method, params=None):
        body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}})
        req = urllib.request.Request(self.url, body.encode(), self.headers)
        with urllib.request.urlopen(req, timeout=5) as r:
            data = json.loads(r.read().decode("utf-8"))
        if "error" in data:
            raise RuntimeError("{}: {}".format(method, data["error"]))
        return data["result"]

    def now_playing(self):
        """Devuelve (item, props) del video en reproducción, o (None, None)."""
        players = self.call("Player.GetActivePlayers")
        video = next((p for p in players if p.get("type") == "video"), None)
        if not video:
            return None, None
        pid = video["playerid"]
        item = self.call("Player.GetItem", {
            "playerid": pid,
            "properties": ["title", "showtitle", "season", "episode", "year", "genre", "director",
                           "art", "uniqueid", "tvshowid", "file"],
        })["item"]
        props = self.call("Player.GetProperties", {
            "playerid": pid, "properties": ["time", "totaltime", "speed"],
        })
        if item.get("type") == "episode" and item.get("tvshowid", -1) > 0:
            try:
                show = self.call("VideoLibrary.GetTVShowDetails", {
                    "tvshowid": item["tvshowid"], "properties": ["uniqueid", "art"],
                })["tvshowdetails"]
                item["_show_uniqueid"] = show.get("uniqueid", {})
                item["_show_art"] = show.get("art", {})
            except Exception as e:
                log("No se pudieron leer los datos de la serie: {}".format(e))
        return item, props


# --------------------------------------------------------------------------
# Imágenes
# --------------------------------------------------------------------------
def decode_kodi_art(url):
    """image://https%3a%2f%2fimage.tmdb.org%2f...jpg/  ->  https://image.tmdb.org/...jpg"""
    if not url:
        return None
    if url.startswith("image://"):
        url = urllib.parse.unquote(url[len("image://"):].rstrip("/"))
    return url


_tmdb_cache = {}


def tmdb_poster(kind, tmdb_id):
    if not (TMDB_API_KEY and tmdb_id):
        return None
    key = (kind, str(tmdb_id))
    if key in _tmdb_cache:
        return _tmdb_cache[key]
    url = "https://api.themoviedb.org/3/{}/{}?api_key={}".format(kind, tmdb_id, TMDB_API_KEY)
    poster = None
    try:
        with urllib.request.urlopen(url, timeout=5) as r:
            path = json.loads(r.read().decode("utf-8")).get("poster_path")
        if path:
            poster = "https://image.tmdb.org/t/p/w500" + path
    except Exception as e:
        log("TMDb falló para {} {}: {}".format(kind, tmdb_id, e))
    _tmdb_cache[key] = poster
    return poster


_tmdb_movie_cache = {}


def tmdb_movie_info(tmdb_id):
    """(año, [directores]) desde TMDb, con nombres en alfabeto latino. Cacheado por película."""
    if not (TMDB_API_KEY and tmdb_id):
        return None, None
    if tmdb_id in _tmdb_movie_cache:
        return _tmdb_movie_cache[tmdb_id]
    year, directors = None, None
    try:
        url = ("https://api.themoviedb.org/3/movie/{}?api_key={}&append_to_response=credits"
               .format(tmdb_id, TMDB_API_KEY))
        with urllib.request.urlopen(url, timeout=5) as r:
            data = json.loads(r.read().decode("utf-8"))
        if data.get("release_date"):
            year = int(data["release_date"][:4])
        crew = (data.get("credits") or {}).get("crew") or []
        directors = [c["name"] for c in crew if c.get("job") == "Director"] or None
    except Exception as e:
        log("TMDb (película {}) falló: {}".format(tmdb_id, e))
    _tmdb_movie_cache[tmdb_id] = (year, directors)
    return year, directors


def pick_poster(item):
    """Prefiere un póster público (http/https) del propio Kodi; si no, TMDb; si no, None."""
    is_ep = item.get("type") == "episode"
    art = item.get("art", {})
    show_art = item.get("_show_art", {})
    candidates = ([art.get("tvshow.poster"), show_art.get("poster"), art.get("season.poster")]
                  if is_ep else [art.get("poster"), art.get("thumb")])
    for c in candidates:
        u = decode_kodi_art(c)
        if u and u.startswith(("http://", "https://")) and len(u) <= 256:
            return u
    ids = item.get("_show_uniqueid", {}) if is_ep else item.get("uniqueid", {})
    return tmdb_poster("tv" if is_ep else "movie", ids.get("tmdb"))


# --------------------------------------------------------------------------
# Construcción de la actividad
# --------------------------------------------------------------------------
def to_seconds(t):
    return t["hours"] * 3600 + t["minutes"] * 60 + t["seconds"]


def fmt_time(s):
    h, rem = divmod(int(s), 3600)
    m, s = divmod(rem, 60)
    return "{}:{:02}:{:02}".format(h, m, s) if h else "{:02}:{:02}".format(m, s)


def clip(text, n=128):
    text = (text or "").strip()
    return text if len(text) <= n else text[: n - 1] + "…"


def build_activity(item, props):
    kind = item.get("type")
    title = item.get("title") or item.get("label") or os.path.basename(item.get("file", ""))

    if kind == "episode":
        details = item.get("showtitle") or title
        # Solo el título del episodio: Discord recorta el texto (no lo desplaza),
        # y la temporada/episodio ya aparecen en la tarjeta y en el texto del póster.
        state = title
        large_text = "Temporada {:02}, episodio {:02}".format(item.get("season", 0), item.get("episode", 0))
        header = details
    elif kind == "movie":
        # Encabezado = título; negritas = año · director (evita repetir el título)
        header = title
        t_year, t_dirs = tmdb_movie_info((item.get("uniqueid") or {}).get("tmdb"))
        year = t_year or item.get("year")
        directors = ", ".join((t_dirs or item.get("director") or [])[:2])
        details = " · ".join(str(x) for x in (year, directors) if x) or title
        state = ", ".join((item.get("genre") or [])[:3]) or None
        large_text = "{} ({})".format(title, year) if year else title
    else:
        header = details = title
        state, large_text = None, "Kodi"

    # type 3 = "Viendo"
    # name: reemplaza el encabezado "Viendo <app>" por "Viendo <serie/película>"
    # status_display_type 0: el texto bajo tu nombre usa "name" (el encabezado);
    # si Discord no acepta "name", se usa 2 (details) para no mostrar el nombre de la app.
    if USE_NAME_OVERRIDE:
        activity = {"type": 3, "details": clip(details), "name": clip(header), "status_display_type": 0}
    else:
        activity = {"type": 3, "details": clip(header if kind == "movie" else details), "status_display_type": 2}
    if state:
        activity["state"] = clip(state)

    assets = {"large_text": clip(large_text)}
    poster = pick_poster(item)
    if poster:
        assets["large_image"] = poster
    elif FALLBACK_IMAGE:
        assets["large_image"] = FALLBACK_IMAGE
    activity["assets"] = assets

    cur, total = to_seconds(props["time"]), to_seconds(props["totaltime"])
    if props.get("speed", 1) == 0:
        activity["state"] = clip("En pausa · {} / {}".format(fmt_time(cur), fmt_time(total)))
    elif total > 0:
        start = int(time.time() - cur)
        activity["timestamps"] = {"start": start, "end": start + total}

    return activity


def same_activity(a, b):
    """Compara ignorando pequeñas derivas de los timestamps (evita spam a Discord)."""
    if a is None or b is None:
        return a is b
    a2, b2 = dict(a), dict(b)
    ta, tb = a2.pop("timestamps", None), b2.pop("timestamps", None)
    if a2 != b2 or (ta is None) != (tb is None):
        return False
    return ta is None or abs(ta["start"] - tb["start"]) <= 3


# --------------------------------------------------------------------------
# Bucle principal
# --------------------------------------------------------------------------
def main():
    if not DISCORD_CLIENT_ID.isdigit():
        log("Configura DISCORD_CLIENT_ID con el Application ID de tu app de Discord.")
        sys.exit(1)

    kodi = Kodi(KODI_HOST, KODI_PORT, KODI_USER, KODI_PASS)
    discord = None
    last = "sin-estado"
    kodi_ok = None

    log("Puente Kodi {}:{} -> Discord iniciado".format(KODI_HOST, KODI_PORT))
    while True:
        try:
            if discord is None:
                discord = DiscordIPC(DISCORD_CLIENT_ID)
                discord.connect()
                last = "sin-estado"

            try:
                item, props = kodi.now_playing()
                if kodi_ok is not True:
                    log("Kodi accesible")
                kodi_ok = True
            except Exception as e:
                if kodi_ok is not False:
                    log("Kodi no responde ({}); se limpia el estado".format(e))
                kodi_ok = False
                item = props = None

            activity = build_activity(item, props) if item else None
            if last == "sin-estado" or not same_activity(activity, last):
                ok = discord.set_activity(activity)
                if not ok and activity and "name" in activity:
                    global USE_NAME_OVERRIDE
                    USE_NAME_OVERRIDE = False
                    log("Discord no acepta 'name' por IPC; se usa solo status_display_type")
                    activity = build_activity(item, props)
                    discord.set_activity(activity)
                log("Actividad: {}".format(
                    "{} | {}".format(activity.get("details"), activity.get("state", "")) if activity else "(vacía)"))
                last = activity

        except (ConnectionError, OSError, struct.error) as e:
            log("Sin conexión con Discord: {}. Reintento en 10 s".format(e))
            if discord:
                discord.close()
            discord = None
            time.sleep(10)
            continue
        except KeyboardInterrupt:
            break

        time.sleep(POLL_SECONDS)

    if discord:
        try:
            discord.set_activity(None)
        except Exception:
            pass
        discord.close()


if __name__ == "__main__":
    main()