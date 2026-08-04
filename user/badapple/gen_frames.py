#!/usr/bin/env python3
"""gen_frames.py - BA02 4-bit xor+RLE with Bayer dither for finer ASCII."""

from __future__ import annotations

import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MP4 = ROOT / "badapple.mp4"
OUT = ROOT / "frames.ba01"

# Higher res + 16 gray levels; Bayer dither puts mid glyphs on edges.
W, H, FPS, BITS = 96, 32, 8, 4
BAYER4 = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
]


def pack4_bayer(gray: bytes) -> bytes:
    out = bytearray((W * H + 1) // 2)
    for y in range(H):
        for x in range(W):
            i = y * W + x
            v = gray[i] + (BAYER4[y & 3][x & 3] - 7.5) * 6.0
            if v < 0:
                v = 0.0
            elif v > 255:
                v = 255.0
            lvl = int(v * 16 / 256)
            if lvl > 15:
                lvl = 15
            if (i & 1) == 0:
                out[i // 2] = lvl << 4
            else:
                out[i // 2] |= lvl
    return bytes(out)


def rle(data: bytes) -> bytes:
    out = bytearray()
    i = 0
    while i < len(data):
        v = data[i]
        j = i + 1
        while j < len(data) and data[j] == v and (j - i) < 255:
            j += 1
        out.append(j - i)
        out.append(v)
        i = j
    return bytes(out)


def main() -> int:
    if not MP4.exists():
        print(f"missing {MP4}", file=sys.stderr)
        return 1
    print(f"ffmpeg → {W}x{H} @{FPS}fps {BITS}-bit Bayer …", flush=True)
    raw = subprocess.check_output(
        [
            "ffmpeg",
            "-v",
            "error",
            "-i",
            str(MP4),
            "-vf",
            f"fps={FPS},scale={W}:{H}:flags=area,format=gray",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "gray",
            "pipe:1",
        ]
    )
    n = len(raw) // (W * H)
    packed_len = (W * H + 1) // 2
    prev = bytes(packed_len)
    body = bytearray()
    for fi in range(n):
        fr = raw[fi * W * H : (fi + 1) * W * H]
        packed = pack4_bayer(fr)
        xor = bytes(a ^ b for a, b in zip(packed, prev))
        enc = rle(xor)
        if len(enc) > 65535:
            print("frame RLE overflow", file=sys.stderr)
            return 1
        body += struct.pack("<H", len(enc))
        body += enc
        prev = packed
    hdr = b"BA02" + bytes([W, H, FPS, BITS]) + struct.pack("<I", n)
    OUT.write_bytes(hdr + body)
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes, {n} frames)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
