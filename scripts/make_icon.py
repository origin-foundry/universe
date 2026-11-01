#!/usr/bin/env python3
"""Draw Universe's placeholder app icon: a white "U" on a dark blue rounded square.

Standard library only (writes a PNG by hand), so the icon has no third-party artwork.
Usage: python scripts/make_icon.py OUT.png   then   npx tauri icon OUT.png -o crates/uni-app/icons
"""
import math
import struct
import sys
import zlib

SIZE = 1024
BACKGROUND = (24, 54, 102, 255)
FOREGROUND = (255, 255, 255, 255)


def inside_rounded_square(x, y, margin=64, radius=180):
    lo, hi = margin, SIZE - margin
    cx = min(max(x, lo + radius), hi - radius)
    cy = min(max(y, lo + radius), hi - radius)
    return lo <= x < hi and lo <= y < hi and math.hypot(x - cx, y - cy) <= radius


def inside_u(x, y, left=300, right=724, top=260, bottom=764, stroke=120):
    centre_x, arc_y = (left + right) / 2, bottom - (right - left) / 2
    outer = (right - left) / 2
    if top <= y <= arc_y and (left <= x < left + stroke or right - stroke < x <= right):
        return True
    if y > arc_y:
        d = math.hypot(x - centre_x, y - arc_y)
        return outer - stroke <= d <= outer
    return False


def pixel(x, y):
    if inside_u(x, y):
        return FOREGROUND
    if inside_rounded_square(x, y):
        return BACKGROUND
    return (0, 0, 0, 0)


def write_png(path):
    rows = bytearray()
    for y in range(SIZE):
        rows.append(0)  # no filter
        for x in range(SIZE):
            rows.extend(pixel(x, y))

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)  # 8-bit RGBA
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
                + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b""))


if __name__ == "__main__":
    write_png(sys.argv[1] if len(sys.argv) > 1 else "icon-source.png")
