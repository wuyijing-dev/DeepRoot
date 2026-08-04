#!/usr/bin/env python3
"""gen_frames.py - compress Bad Apple to BA01 (1-bit xor+RLE) for realtime ASCII."""

from __future__ import annotations

import struct
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MP4 = ROOT / "badapple.mp4"
OUT = ROOT / "frames.ba01"

# Compact terminal size; player maps bits → ASCII at play time.
W, H, FPS = 48, 18, 10


def pack_bits(gray: bytes) -> bytes:
    out = bytearray((W * H + 7) // 8)
    for i, p in enumerate(gray):
        if p >= 128:
            out[i >> 3] |= 1 << (7 - (i & 7))
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
    print(f"ffmpeg → {W}x{H} @{FPS}fps …", flush=True)
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
    prev = bytes((W * H + 7) // 8)
    body = bytearray()
    for fi in range(n):
        fr = raw[fi * W * H : (fi + 1) * W * H]
        packed = pack_bits(fr)
        xor = bytes(a ^ b for a, b in zip(packed, prev))
        enc = rle(xor)
        body += struct.pack("<H", len(enc))
        body += enc
        prev = packed
    hdr = b"BA01" + bytes([W, H, FPS, 0]) + struct.pack("<I", n)
    OUT.write_bytes(hdr + body)
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes, {n} frames)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
