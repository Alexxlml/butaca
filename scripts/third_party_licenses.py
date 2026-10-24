"""Genera THIRD-PARTY-LICENSES.txt con el texto de licencia de cada dependencia del binario.

Uso:  python scripts/third_party_licenses.py [salida]
Requiere que las dependencias ya estén descargadas (cargo build / cargo fetch).
"""
import json
import pathlib
import subprocess
import sys

TARGET = "x86_64-pc-windows-msvc"
NAMES = ("LICENSE", "LICENCE", "COPYING", "NOTICE", "UNLICENSE")


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "THIRD-PARTY-LICENSES.txt")
    meta = json.loads(subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", TARGET],
        check=True, capture_output=True, text=True).stdout)
    in_tree = {n["id"] for n in meta["resolve"]["nodes"]}
    root = meta["resolve"]["root"]

    parts = ["Third-party software included in Butaca\n"
             "======================================\n\n"
             "Butaca itself is MIT-licensed (see LICENSE). It includes the following components,\n"
             "each under its own license, reproduced below.\n"]
    pkgs = sorted((p for p in meta["packages"] if p["id"] in in_tree and p["id"] != root),
                  key=lambda p: (p["name"], p["version"]))
    for p in pkgs:
        folder = pathlib.Path(p["manifest_path"]).parent
        files = sorted(f for f in folder.iterdir()
                       if f.is_file() and f.name.upper().startswith(NAMES))
        parts.append("\n" + "-" * 78 + "\n")
        parts.append(f"{p['name']} {p['version']}  —  {p['license'] or 'see below'}\n")
        if p.get("repository"):
            parts.append(f"{p['repository']}\n")
        parts.append("-" * 78 + "\n")
        if not files:
            parts.append("(This package declares the license above but ships no separate license file.)\n")
        for f in files:
            parts.append(f"\n[{f.name}]\n")
            parts.append(f.read_text(encoding="utf-8", errors="replace").strip() + "\n")
    out.write_text("".join(parts), encoding="utf-8")
    print(f"{len(pkgs)} paquetes -> {out}")


if __name__ == "__main__":
    main()
