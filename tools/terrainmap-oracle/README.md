# terrainmap-oracle: differential harness for the automap raster port

Compiles the **unmodified** Raven TUs `codemp/qcommon/cm_draw.cpp` (the
`CDraw32` 32-bit raster) and `codemp/qcommon/cm_terrainmap.cpp` (the
`CTerrainMap` RMG automap) into one standalone dumper, runs it over committed
synthetic fixtures, and stores canonical golden dumps under `golden/`. The Rust
port (`crates/mp/engine/qcommon/src/cm/cdraw32.rs`,
`crates/mp/engine/qcommon/src/cm/cterrainmap.rs`) reproduces them byte for byte
in `crates/mp/engine/qcommon/tests/terrainmap_parity.rs`. Executes
porting-rules §F18. Precedent: `tools/gp2-oracle/`, `tools/rmg-oracle/`.

## Usage

```sh
sh build.sh           # build the dumper, diff current output against golden/
sh build.sh --regen   # rebuild golden/ (after changing a fixture or a scenario)
cargo test -p mp_engine_qcommon --test terrainmap_parity
```

`build.sh` mirrors the `codemp/` tree into `build/` and copies the two oracle
TUs plus the two real headers under test next to the **stub headers** in
`stubs/`, so the sources' relative `#include`s (`../qcommon/`, `../png/`)
resolve to the stubs; `oracle/` is never edited. Toolchain: Homebrew `g++-16`,
`-std=c++14 -w -O1 -DNDEBUG`. `-DNDEBUG` is load bearing: Raven's `assert` calls
vanish in a release build, and the Rust port omits them, so the harness must
too. Goldens are committed, so `cargo test` needs no C++ toolchain.

**One syntax patch on the build copy.** `cm_draw.cpp:553,585` write
`unsigned short (expr)`, a functional cast only MSVC accepts. `build.sh`
rewrites those two lines to the C cast `(unsigned short)(expr)` in `build/`,
and fails when the rewrite does not land. Same conversion, no behavior change.

## Stubbed environment

`stubs/qcommon/` declares only what the two TUs name: `byte`, `qboolean`,
`vec3_t`, the Win32 `POINT`, `Z_Malloc`/`Z_Free`, `va`, and
`RotatePointAroundVector`. `stubs/qcommon/cm_landscape.h` is a six-accessor
`CCMLandScape` stand-in (`GetHeightMap`, `GetRealWidth`, `GetRealHeight`,
`GetBaseWaterHeight`, `GetMins`, `GetSize`), which is the whole landscape
surface `cm_terrainmap.cpp` reads. `src/host_stubs.cpp` implements the zone
allocator over `malloc`, transcribes `q_math.c`'s `RotatePointAroundVector`, and
records the renderer and PNG calls: `R_LoadImage` serves the fixtures,
`R_CreateAutomapImage` captures the uploaded raster, and `PNG_Save` captures
`mImage`, which is the class's only read-back of that private member.

## Fixtures

Every fixture is synthetic and produced by `fixtures/gen_fixtures.py`. **No
retail game content is committed.** Regenerate with:

```sh
cd fixtures && python3 gen_fixtures.py
```

* `heightmap.bin` - 65 by 65 bytes, two ridges plus a basin, so the five-tap
  average has gradients and the water blend has values under the base water
  height.
* `bg.rgba` - 64 by 64 RGBA background tile.
* `sym_start.rgba`, `sym_end.rgba`, `sym_objective.rgba`, `sym_bld.rgba` -
  16 by 16 RGBA symbols with a soft disc and a hard rim, so the blit alpha mask
  covers 0, 255, and the middle.

The harness appends one zero pad byte to the heightmap. `ApplyHeightmap` starts
`xRel` at `width`, so Raven's index runs into the next row and, on the last row,
one byte past the buffer. The pad makes that read deterministic and equal to the
Rust port's defined `0` (porting-rules §F19).

## Goldens

### `golden/draw.txt`: the `CDraw32` primitives

A 32 by 24 buffer, reset to a deterministic gradient before each of 17
sub-scenarios, dumped as full RGBA hex per row plus an FNV-1a hash:
`clear`, `line_solid`, `line_alpha`, `line_ave`, `line_aa`, `rect`, `box`,
`circle`, `circle_ave`, `poly_tri`, `poly_concave`, `poly_arrow`,
`poly_degenerate`, `blit`, `blit_color`, `emboss`, `clipped`. Each set includes
fully clipped, partially clipped, reversed, and degenerate inputs. `clip` lines
record `GetClip` after each clip change.

### `golden/terrainmap.txt`: the automap

One `CTerrainMap` over the fixture landscape (`mins` -2048/-2048/-512, `size`
4096/4096/1024, base water height 40). Four 512 by 512 buffers, dumped as a
per-row FNV-1a hash plus a whole-buffer hash: `image_after_ctor`,
`image_after_symbols` (after every `Add*` including out-of-range ones),
`upload_with_player`, and `upload_no_player` (which shows the second `Upload`
composing over the first). Then eight `convert` lines from
`CM_TM_ConvertPosition`, and the recorded call stream.

## What the rig does not cover

`PNG_Save` itself. The encoder is the unported `codemp/png/` TU, so the Rust
`SaveImageToDisk` builds Raven's filename and stops there. The golden records
the call and its arguments, which is the whole observable the port keeps.
