#!/usr/bin/env python3
"""Generate a simple solid-color PNG icon (1024x1024) for EasyMusic.

Pure stdlib (zlib + struct) — no Pillow needed. Writes a rounded-ish look by
drawing a filled indigo square with a lighter diagonal stripe, good enough as
the source for `npx tauri icon`.
"""
import struct
import zlib

W = H = 1024
BG = (99, 102, 241)      # indigo #6366F1
STRIPE = (129, 140, 248) # lighter indigo #818CF8
CLEAR = (255, 255, 255)

def chunk(tag: bytes, data: bytes) -> bytes:
    return (struct.pack(">I", len(data)) + tag + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

def px(x: int, y: int) -> tuple:
    # diagonal stripe from bottom-left to top-right
    if abs((x - y)) < 90:
        return STRIPE
    return BG

rows = b""
for y in range(H):
    row = b"\x00"  # filter type 0
    for x in range(W):
        r, g, b = px(x, y)
        row += bytes((r, g, b))
    rows += row

png = b"\x89PNG\r\n\x1a\n"
png += chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0))
png += chunk(b"IDAT", zlib.compress(rows, 9))
png += chunk(b"IEND", b"")

with open("/opt/data/workspace/Code/EasyMusic/app-icon.png", "wb") as f:
    f.write(png)
print("wrote app-icon.png", len(png), "bytes")
