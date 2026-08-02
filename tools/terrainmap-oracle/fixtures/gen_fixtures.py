#!/usr/bin/env python3
"""Generate the terrainmap-oracle fixtures.

Every fixture is synthetic. No retail game content enters this repository.
Run from this directory:

    python3 gen_fixtures.py

Layout, shared by the C dumper (`../main.cpp`) and the Rust parity test
(`crates/mp/engine/qcommon/tests/terrainmap_parity.rs`):

* `heightmap.bin`   - 65 by 65 bytes, the landscape heightmap
                      (`GetRealWidth` by `GetRealHeight`).
* `bg.rgba`         - 64 by 64 RGBA, the `01_bg` background tile.
* `sym_start.rgba`  - 16 by 16 RGBA, one map symbol.
* `sym_end.rgba`    - 16 by 16 RGBA, one map symbol.
* `sym_objective.rgba` - 16 by 16 RGBA, one map symbol.
* `sym_bld.rgba`    - 16 by 16 RGBA, one map symbol.
"""

import math
import struct

HM_W = 65
HM_H = 65
BG_W = 64
BG_H = 64
SYM = 16


def heightmap():
    out = bytearray()
    for y in range(HM_H):
        for x in range(HM_W):
            # Two ridges plus a basin, so the five-tap average has real
            # gradients to average and the water blend has values under the
            # base water height.
            v = 128.0
            v += 60.0 * math.sin(x * 0.19) * math.cos(y * 0.13)
            v += 30.0 * math.sin((x + y) * 0.07)
            v -= 40.0 * math.exp(-((x - 20) ** 2 + (y - 44) ** 2) / 180.0)
            out.append(max(0, min(255, int(v))))
    return bytes(out)


def background():
    out = bytearray()
    for y in range(BG_H):
        for x in range(BG_W):
            out.append((x * 4 + y) & 0xFF)
            out.append((y * 4 + x) & 0xFF)
            out.append((x ^ y) * 3 & 0xFF)
            out.append(0xFF)
    return bytes(out)


def symbol(seed):
    """A soft disc with a hard rim, so the blit alpha mask has all of 0, 255,
    and the middle."""
    out = bytearray()
    cx = cy = (SYM - 1) / 2.0
    for y in range(SYM):
        for x in range(SYM):
            d = math.hypot(x - cx, y - cy)
            a = 0
            if d < 6.5:
                a = int(255 * max(0.0, min(1.0, (6.5 - d) / 2.5)))
            if 6.5 <= d < 7.2:
                a = 255
            out.append((x * 9 + seed) & 0xFF)
            out.append((y * 11 + seed * 3) & 0xFF)
            out.append((seed * 37 + x + y) & 0xFF)
            out.append(a)
    return bytes(out)


def write(name, data):
    with open(name, "wb") as f:
        f.write(data)
    print(f"{name}: {len(data)} bytes")


def main():
    write("heightmap.bin", heightmap())
    write("bg.rgba", background())
    write("sym_start.rgba", symbol(1))
    write("sym_end.rgba", symbol(2))
    write("sym_objective.rgba", symbol(3))
    write("sym_bld.rgba", symbol(4))
    # Silence the unused-import lint if struct is ever dropped.
    assert struct is not None


if __name__ == "__main__":
    main()
