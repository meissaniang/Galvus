"""Génère l'icône Galvus 1024x1024 (PNG RGBA) sans dépendance externe.

Rendu par champs de distance signés (SDF) pour un anticrénelage propre :
fond squircle sombre du design system + glyphe accent (carré arrondi + point),
repris du logo « GalvusSidebar.dc.html ».
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
ACCENT_DIM = (0x0E, 0x9F, 0x6E)  # dégradé du glyphe

R_OUTER = 0.2237 * S             # rayon squircle macOS
MARGIN = 0.0                     # l'icône occupe tout le carré

# Glyphe : carré arrondi (contour) + disque central, proportions du logo SVG
# d'origine (rect 11/16 avec rx 3, cercle r 2.1).
G_HALF = 0.2600 * S              # demi-côté du carré du glyphe
G_RADIUS = 0.0950 * S            # rayon des coins du glyphe
G_STROKE = 0.0470 * S            # épaisseur du contour
DOT_R = 0.0980 * S               # rayon du point central


def sdf_round_rect(px, py, half, radius):
    """Distance signée à un carré arrondi centré à l'origine."""
    qx = abs(px) - (half - radius)
    qy = abs(py) - (half - radius)
    ax, ay = max(qx, 0.0), max(qy, 0.0)
    return math.hypot(ax, ay) + min(max(qx, qy), 0.0) - radius


def coverage(d):
    """Couverture anticrénelée d'un pixel à partir de sa distance signée."""
    return min(max(0.5 - d, 0.0), 1.0)


def blend(dst, src, a):
    return tuple(int(round(dst[i] * (1 - a) + src[i] * a)) for i in range(3))


rows = bytearray()
cx = cy = S / 2.0

for y in range(S):
    rows.append(0)  # filtre PNG « None »
    py = y + 0.5 - cy
    t = y / (S - 1)
    bg = tuple(int(round(BG_TOP[i] + (BG_BOTTOM[i] - BG_TOP[i]) * t)) for i in range(3))
    glyph = tuple(int(round(ACCENT[i] + (ACCENT_DIM[i] - ACCENT[i]) * t * 0.55)) for i in range(3))

    for x in range(S):
        px = x + 0.5 - cx

        # Silhouette de l'icône (squircle) -> alpha du pixel.
        a_icon = coverage(sdf_round_rect(px, py, S / 2.0 - MARGIN, R_OUTER))
        if a_icon <= 0.0:
            rows.extend((0, 0, 0, 0))
            continue

        color = bg

        # Contour du carré arrondi : |sdf| - demi-épaisseur.
        d_ring = abs(sdf_round_rect(px, py, G_HALF, G_RADIUS)) - G_STROKE / 2.0
        a_ring = coverage(d_ring)

        # Disque central.
        d_dot = math.hypot(px, py) - DOT_R
        a_dot = coverage(d_dot)

        a_glyph = max(a_ring, a_dot)
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
