"""Génère l'icône Galvus 1024x1024 (PNG RGBA) sans dépendance externe.

Rendu par champs de distance signés (SDF) pour un anticrénelage propre :
squircle sombre du design system + éclair accent (« Galvus » ← galvanique).

Usage : python3 design/make-icon.py
        pnpm tauri icon design/galvus-icon.png
"""

import math
import struct
import zlib

S = 1024
OUT = "design/galvus-icon.png"

# Palette du design system.
BG_TOP = (0x1C, 0x24, 0x2F)      # --g-s1
BG_BOTTOM = (0x0B, 0x10, 0x17)   # --g-app
ACCENT = (0x23, 0xC4, 0x8A)      # --g-accent
ACCENT_DIM = (0x0E, 0x9F, 0x6E)  # bas du dégradé du glyphe

R_OUTER = 0.2237 * S             # rayon squircle macOS

# Éclair : polygone en fractions du côté, centré sur l'origine.
BOLT = [
    (0.047, -0.315),
    (-0.210, 0.047),
    (-0.037, 0.047),
    (-0.079, 0.315),
    (0.210, -0.058),
    (0.031, -0.058),
]


def sd_round_rect(px, py, half, radius):
    """Distance signée à un carré arrondi centré à l'origine."""
    qx, qy = abs(px) - (half - radius), abs(py) - (half - radius)
    return math.hypot(max(qx, 0.0), max(qy, 0.0)) + min(max(qx, qy), 0.0) - radius


def sd_polygon(px, py, verts):
    """Distance signée à un polygone (négative à l'intérieur)."""
    n = len(verts)
    d = (px - verts[0][0]) ** 2 + (py - verts[0][1]) ** 2
    s = 1.0
    for i in range(n):
        j = (i - 1) % n
        ex, ey = verts[j][0] - verts[i][0], verts[j][1] - verts[i][1]
        wx, wy = px - verts[i][0], py - verts[i][1]
        denom = ex * ex + ey * ey
        t = 0.0 if denom == 0 else max(0.0, min(1.0, (wx * ex + wy * ey) / denom))
        bx, by = wx - ex * t, wy - ey * t
        d = min(d, bx * bx + by * by)
        c1, c2, c3 = py >= verts[i][1], py < verts[j][1], ex * wy > ey * wx
        if (c1 and c2 and c3) or (not c1 and not c2 and not c3):
            s = -s
    return s * math.sqrt(d)


def coverage(d):
    """Couverture anticrénelée d'un pixel à partir de sa distance signée."""
    return min(max(0.5 - d, 0.0), 1.0)


def blend(dst, src, a):
    return tuple(int(round(dst[i] * (1 - a) + src[i] * a)) for i in range(3))


bolt = [(vx * S, vy * S) for vx, vy in BOLT]
rows = bytearray()
cx = cy = S / 2.0

for y in range(S):
    rows.append(0)  # filtre PNG « None »
    py = y + 0.5 - cy
    t = y / (S - 1)
    bg = tuple(int(round(BG_TOP[i] + (BG_BOTTOM[i] - BG_TOP[i]) * t)) for i in range(3))
    glyph = tuple(
        int(round(ACCENT[i] + (ACCENT_DIM[i] - ACCENT[i]) * t * 0.55)) for i in range(3)
    )

    for x in range(S):
        px = x + 0.5 - cx

        a_icon = coverage(sd_round_rect(px, py, S / 2.0, R_OUTER))
        if a_icon <= 0.0:
            rows.extend((0, 0, 0, 0))
            continue

        color = bg
        a_glyph = coverage(sd_polygon(px, py, bolt))
        if a_glyph > 0.0:
            color = blend(color, glyph, a_glyph)

        rows.extend((color[0], color[1], color[2], int(round(a_icon * 255))))


def chunk(tag, data):
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


png = b"\x89PNG\r\n\x1a\n"
png += chunk(b"IHDR", struct.pack(">IIBBBBB", S, S, 8, 6, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(bytes(rows), 9))
png += chunk(b"IEND", b"")

with open(OUT, "wb") as f:
    f.write(png)

print("écrit:", OUT)
