#!/usr/bin/env python3
"""cin-oracle fixture generator.

Writes the synthetic RoQ streams under fixtures/. Every byte comes from integer
arithmetic, so a regenerated fixture is byte-identical on any host. No retail
game content enters the repo.

Container layout, from oracle/codemp/client/cl_cin.cpp:1069-1077,1026-1030:

  file header, 8 bytes:  u16 id (0x1084), u32 size, u16 framerate
  chunk header, 8 bytes: u16 id, u32 size, u8 arg0, u8 arg1

Raven reads only three of the four size bytes, and it reads the two argument
bytes as `roq_flags = arg0 + arg1*256`, `roqF0 = (char)arg1`, `roqF1 =
(char)arg0`. Every stream ends with an all-zero terminator chunk, which the
driver uses as its stop mark.

  python3 gen_fixtures.py
"""

import os
import struct

ROQ_ID = 0x1084
ROQ_QUAD_INFO = 0x1001
ROQ_CODEBOOK = 0x1002
ROQ_QUAD_VQ = 0x1011
ZA_SOUND_MONO = 0x1020
ZA_SOUND_STEREO = 0x1021

OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fixtures")


def file_header(fps):
    return struct.pack("<HIH", ROQ_ID, 0xFFFFFFFF, fps)


def chunk(chunk_id, payload, arg0=0, arg1=0):
    return struct.pack("<HIBB", chunk_id, len(payload), arg0, arg1) + payload


def terminator():
    return b"\x00" * 8


def quad_info(xsize, ysize, maxsize=8, minsize=4):
    return chunk(ROQ_QUAD_INFO, struct.pack("<HHHH", xsize, ysize, maxsize, minsize))


def codebook(flags):
    """A ROQ_CODEBOOK payload sized exactly the way decodeCodeBook reads it.

    Source: oracle/codemp/client/cl_cin.cpp:655-692
    """
    if flags == 0:
        two = four = 256
    else:
        two = flags >> 8
        if two == 0:
            two = 256
        four = flags & 0xFF
    four *= 2

    body = bytearray()
    for i in range(two):
        body.append((i * 7) & 0xFF)
        body.append((i * 11 + 3) & 0xFF)
        body.append((i * 13 + 7) & 0xFF)
        body.append((i * 17 + 11) & 0xFF)
        body.append((i * 5 + 31) & 0xFF)
        body.append((i * 3 + 97) & 0xFF)
    for i in range(four):
        body.append((i * 23) & 0xFF)
        body.append((i * 29 + 5) & 0xFF)
    return chunk(ROQ_CODEBOOK, bytes(body), arg0=flags & 0xFF, arg1=(flags >> 8) & 0xFF)


# --- the VQ cel stream ------------------------------------------------------

# blitVQQuad32fs reads one flat sequence of 2-bit codes. Every eighth code
# reloads a 16-bit little-endian word, most significant pair first, and each
# code's argument bytes follow it in the byte stream.
# Source: oracle/codemp/client/cl_cin.cpp:456-532
CODE_VQ = 0x8000
CODE_DROP = 0xC000
CODE_MOTION = 0x4000
CODE_SKIP = 0x0000


def pack_cels(items):
    """Turns a list of (code, arg_bytes) into the RoQ_QUAD_VQ payload."""
    data = bytearray()
    for i, (_code, args) in enumerate(items):
        if i % 8 == 0:
            word = 0
            for k in range(8):
                if i + k < len(items):
                    word |= items[i + k][0] >> (2 * k)
            data += struct.pack("<H", word)
        data += args
    return bytes(data)


def block_positions(xsize, ysize):
    """The 8x8 block each top-level code addresses, in setupQuad's order.

    recurseQuad walks 16x16 macroblocks in raster order and splits each into
    four 8x8 quads in the order (x,y), (x+8,y), (x,y+8), (x+8,y+8).
    Source: oracle/codemp/client/cl_cin.cpp:728-731,768-770
    """
    out = []
    for my in range(0, ysize, 16):
        for mx in range(0, xsize, 16):
            for dx, dy in ((0, 0), (8, 0), (0, 8), (8, 8)):
                out.append((mx + dx, my + dy))
    return out


def motion_byte(bx, by, size, xoff, yoff, want_dx, want_dy, xsize, ysize, scale):
    """The mcomp index byte that moves a block by (want_dx, want_dy) texels.

    RoQPrepMcomp stores mcomp[(x*16)+y] = normalBuffer0 - ((y+yoff-8)*i +
    (x+xoff-8)*j), so a code byte of (x<<4)|y displaces the source by
    -(x+xoff-8) texels and -(y+yoff-8) rows, both times `scale` on the
    xsize == ysize*4 branch. The displacement is clamped twice: to what the
    0..15 index range can express, and to what keeps the source rectangle
    inside the frame.
    Source: oracle/codemp/client/cl_cin.cpp:837-856
    """

    def pick(pos, want, limit, off):
        # `d` is the displacement in mcomp units, so the texel step is d*scale.
        lo = max(-(pos // scale), -7 - off)
        hi = min((limit - size - pos) // scale, 8 - off)
        assert lo <= hi, (pos, limit, off, lo, hi)
        d = max(lo, min(want // scale, hi))
        return d, 8 - off - d

    dxu, mx = pick(bx, want_dx, xsize, xoff)
    dyu, my = pick(by, want_dy, ysize, yoff)
    assert 0 <= mx <= 15, (bx, by, xoff, want_dx, mx)
    assert 0 <= my <= 15, (bx, by, yoff, want_dy, my)
    assert 0 <= bx + dxu * scale and bx + dxu * scale + size <= xsize
    assert 0 <= by + dyu * scale and by + dyu * scale + size <= ysize
    return (mx << 4) | my


def vq_frame(xsize, ysize, xoff, yoff, seed, scale=1):
    """A ROQ_QUAD_VQ payload that drives every arm of blitVQQuad32fs.

    The four top-level arms rotate block by block: the 8x8 vq code, the 0xc000
    drop (whose four sub-quads take the 4x4 vq, the 2x2 vq, the motion and the
    skip arms in turn), the 8x8 motion code, and the skip. The rotation also
    steps once per macroblock, so no arm lands on the same sub-quad slot twice
    in a row and every motion code reads a painted neighbourhood.
    """
    blocks = block_positions(xsize, ysize)
    items = []

    for i, (bx, by) in enumerate(blocks):
        arm = (i + i // 4) % 4
        if arm == 0:
            items.append((CODE_VQ, bytes([(i * 13 + seed) & 0xFF])))
        elif arm == 1:
            items.append((CODE_DROP, b""))
            for sub, (sx, sy) in enumerate(((0, 0), (4, 0), (0, 4), (4, 4))):
                qx, qy = bx + sx, by + sy
                if sub == 0:
                    items.append((CODE_VQ, bytes([(i * 7 + sub + seed) & 0xFF])))
                elif sub == 1:
                    args = bytes([(i * 5 + k + seed) & 0xFF for k in range(4)])
                    items.append((CODE_DROP, args))
                elif sub == 2:
                    mb = motion_byte(qx, qy, 4, xoff, yoff, 4, -4, xsize, ysize, scale)
                    items.append((CODE_MOTION, bytes([mb])))
                else:
                    items.append((CODE_SKIP, b""))
        elif arm == 2:
            mb = motion_byte(bx, by, 8, xoff, yoff, -8, 8, xsize, ysize, scale)
            items.append((CODE_MOTION, bytes([mb])))
        else:
            items.append((CODE_SKIP, b""))

    return chunk(ROQ_QUAD_VQ, pack_cels(items), arg0=yoff & 0xFF, arg1=xoff & 0xFF)


def sound_payload(nbytes):
    return bytes((i & 0xFF) for i in range(nbytes))


# --- the fixtures -----------------------------------------------------------


def build():
    fixtures = {}

    # 1. readQuadInfo + setupQuad + recurseQuad at an aligned size.
    fixtures["quadinfo"] = file_header(30) + quad_info(64, 64) + terminator()

    # 2. recurseQuad's bounds rejection: neither edge is a multiple of 16, so the
    #    trailing macroblocks record fewer than 20 cels each.
    fixtures["quadinfo_ragged"] = file_header(30) + quad_info(40, 24) + terminator()

    # 3. A full 256-entry codebook, roq_flags == 0.
    fixtures["codebook"] = (
        file_header(30) + quad_info(64, 64) + codebook(0x0000) + terminator()
    )

    # 4. Both non-zero roq_flags branches: an explicit `two`, then the `two == 0`
    #    fallback to 256.
    fixtures["codebook_partial"] = (
        file_header(30)
        + quad_info(64, 64)
        + codebook(0x8040)
        + codebook(0x0040)
        + terminator()
    )

    # 5. Three VQ frames over a full codebook. Frame 0 runs on bank 0 with a zero
    #    motion offset, frame 1 on bank 1 with a negative roqF0, frame 2 back on
    #    bank 0 with a negative roqF1.
    fixtures["vq_frames"] = (
        file_header(30)
        + quad_info(64, 64)
        + codebook(0x0000)
        + vq_frame(64, 64, 0, 0, seed=0)
        + vq_frame(64, 64, -4, 3, seed=37)
        + vq_frame(64, 64, 5, -6, seed=91)
        + terminator()
    )

    # 6. xsize == ysize*4, which doubles both RoQPrepMcomp strides.
    fixtures["vq_nonsquare"] = (
        file_header(30)
        + quad_info(64, 16)
        + codebook(0x0000)
        + vq_frame(64, 16, 0, 0, seed=11, scale=2)
        + vq_frame(64, 16, -2, 2, seed=53, scale=2)
        + terminator()
    )

    # 7. RllDecodeMonoToStereo over two full 0..255 delta sweeps per chunk.
    mono = file_header(30)
    for flags in (0x0000, 0x8000, 0x1234):
        mono += chunk(
            ZA_SOUND_MONO, sound_payload(512), arg0=flags & 0xFF, arg1=(flags >> 8) & 0xFF
        )
    fixtures["sound_mono"] = mono + terminator()

    # 8. RllDecodeStereoToStereo, whose flag splits into two channel predictors.
    stereo = file_header(30)
    for flags in (0x0000, 0x8000, 0x1234):
        stereo += chunk(
            ZA_SOUND_STEREO, sound_payload(512), arg0=flags & 0xFF, arg1=(flags >> 8) & 0xFF
        )
    fixtures["sound_stereo"] = stereo + terminator()

    return fixtures


def main():
    os.makedirs(OUT_DIR, exist_ok=True)
    for name, data in sorted(build().items()):
        path = os.path.join(OUT_DIR, name + ".roq")
        with open(path, "wb") as f:
            f.write(data)
        print("wrote %s (%d bytes)" % (path, len(data)))


if __name__ == "__main__":
    main()
