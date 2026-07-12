# trmodel-oracle — differential golden harness for the tr_model loader + cache

Verifies the `mp_renderer` `RenderModels` / `CachedEndianedModelBinary` port and
the `mp_engine_ghoul2` `matcomp` codec (`docs/subsystems/tr-model.md`, FROZEN)
against the **unmodified** Raven `codemp/renderer/tr_model.cpp` + `matcomp.c`,
exactly like `tools/gp2-oracle` / `icarus-oracle` / `stringed-oracle`
(porting-rules §18). The oracle `.cpp`/`.h` are copied into `build/` and compiled
standalone against stub headers; canonical dumps are stored under `goldens/` and
committed, so the Rust parity tests need **no** C++ toolchain — only `build.sh`
does, to (re)generate or check.

`oracle/` is never edited.

## Usage

```sh
sh build.sh          # build + run all modes, diff dumps against goldens/
sh build.sh --regen  # regenerate fixtures/* and goldens/*
```

Toolchain: Homebrew `g++-16` (override with `CXX=`). Flags mirror the sibling
harnesses (`-fsigned-char -ffp-contract=off -fno-fast-math`) plus the WinDed
DEDICATED Release macro set the port models: **`-DDEDICATED`** (headless loader —
`GetRefAPI` exports only `RE_Shutdown`), **`-DNDEBUG`** (asserts no-op), and
**`-D_M_IX86`** (the shipped x86 target: every `#ifndef _M_IX86` big-endian swap
block compiles OUT, so `LittleLong/Short/Float` are identity on this LE host —
`TRM-D3`). `-fpermissive` downgrades the one LP64-ism (see Normalizations).

## Goldens (each pins a § Verification strategy unit)

| Golden | Pins | Doc unit |
| --- | --- | --- |
| `goldens/load.txt` | `ServerLoadMDXM`/`ServerLoadMDXA` header parse + in-place write-backs (`ident`/`version`/counts/offsets, `surf->ident = SF_MDX`), the glm→gla `animIndex` recursion, `mod->type`/`dataSize`/`numLods`; the `model_mdxm`/`model_mdxa` **NULL-parity** (SET where `model_t.mdxm`/`.mdxa` is non-NULL, else NULL); version-reject + unknown-ident **fail** paths (return literal `0`, entry stays hashed under its nonzero index → re-register returns that index — the MOD_BAD retention asymmetry, ruling 53); `R_GetModelByHandle` out-of-range → `models[0]` (MOD_BAD). | "Header-parse + endian", "Seam", "Handle/pool" |
| `goldens/cache_hitmiss.txt` | Disk **miss** (2 FS reads: glm + its gla) vs cache **hit** (after `R_HunkClearCrap`+`R_ModelInit` the disk images survive in `CachedModels`; re-register does 0 FS reads — `pqbAlreadyFound` flips true, the `RE_RegisterServerModels_Malloc` repeat branch). | "Cache goldens" (hit/miss) |
| `goldens/cache_evict.txt` | Level-keyed eviction: `RE_RegisterModels_LevelLoadEnd(qfalse)` with `r_modelpoolmegs=0` dumps entries whose `iLastLevelUsedOn` is stale; survivors printed in `std::map`/BTreeMap **sorted** order via `RE_RegisterModels_Info_f`. | "Cache goldens" (eviction) |
| `goldens/cache_dumpnonpure.txt` | `RE_RegisterModels_DumpNonPure` (via `RE_RegisterMedia_LevelLoadBegin` with `sv_pure=1`) evicts entries whose `FS_FileIsInPAK` checksum no longer matches (the 1/-1 convention, ruling 59) — **never** `*default.gla` (the 294-byte `FakeGLAFile`). | "Cache goldens" (DumpNonPure) |
| `goldens/matcomp.txt` | `MC_UnCompressQuat` quantized-quaternion → 3×4 matrix over a spread of packed inputs (the sole live matcomp path, `UnCompressBone`) + `MC_Compress`/`MC_UnCompress` round-trips. Floats dumped as raw IEEE-754 bits for bit-exact parity. | "MC_UnCompressQuat goldens" |

The Rust parity tests (in `mp_renderer` for the cache/pool/pipeline, in
`mp_engine_ghoul2` for matcomp) read `fixtures/*` + `goldens/*` from here and must
reproduce every golden exactly, injecting cvar/FS/pak values via the fixture-backed
`MockHost` (`crates/mp/host-interface/src/mock.rs`).

## Fixtures — `modelgen.cpp` (ruling 14: hand-authored, no retail data)

`modelgen` emits minimal-but-valid `mdxm`/`mdxa` byte images the stubbed FS then
serves to the unmodified load path. Every header field, ident/version, and offset
is spelled out with its `mdx_format.h` cite. Byte-layout choices:

- **`.gla` = a 100-byte `mdxaHeader_t`** (`mdx_format.h:351-371`): `ident=2LGA`,
  `version=6`, `numFrames=1` (the `<1` reject guard), `ofsEnd=100`. The
  `#ifndef _M_IX86` frame/skel swap walk compiles out, so no body is needed.
- **`.glm` = 360 bytes**: a 164-byte `mdxmHeader_t` + `mdxmHierarchyOffsets_t` +
  one `mdxmSurfHierarchy_t` (stride 144) + one LOD (`mdxmLOD_t` +
  `mdxmLODSurfOffset_t`) + one 40-byte `mdxmSurface_t`. The intel-live middle
  section (`:880-991` — surf-hierarchy walk, LOD/surface field swaps,
  `SHADER_MAX_VERTEXES/INDEXES` bounds, `ident=SF_MDX`, `StoreShaderRequest`) runs
  and is exercised; the `#ifndef _M_IX86` triangle/vertex/bone-ref walks compile
  out, so no vertex/triangle/bone-ref bodies are needed. `animName="skeletons/test"`
  drives the glm→gla `animIndex` cross-reference (`:867`).
- **`ofsEnd == exact file length`** on every image, so the morph'd disk buffer's
  zone size equals `iAllocSize` and `Z_MemSize(GLM/GLA)` == the local
  `alloc_size` sum (`GetModelDataAllocSize` parity, `TRM-D3`).
- **`badident.glm`** (ident `0xDEADBEEF` → `switch` default → fail) and
  **`badversion.glm`** (`MDXM_IDENT`, `version=99` → `ServerLoadMDXM` reject).
- **`modelb.glm` + `skeletons/test2.gla`** — a second distinct pair so the
  eviction / DumpNonPure survivors form a clean split.

## Deterministic host (`host.cpp`)

Implements the qcommon/q_shared seam with fully deterministic behaviour: an FS
that serves `fixtures/<qpath>` (lowercased, as the loader lowercases before
`FS_ReadFile`) and a PAK-checksum map for `FS_FileIsInPAK` (`Some`→`1`, else
`-1`, ruling 59); a zone allocator that tracks **per-tag byte sums** so
`Z_MemSize` == the sum of live `iAllocSize` (the `GetModelDataAllocSize`
derivation, `TRM-D3`); a cvar registry (`sv_pure`/`r_modelpoolmegs`/
`r_noServerGhoul2`); and captured console output (`Com_Printf`→stdout = the
golden; `Com_DPrintf` silent, matching non-developer). No raw pointer/address is
ever dumped — only parsed values, offsets, keys, handles, and NULL/SET — so dumps
are run-twice byte-identical.

## Normalizations (documented; NONE edit oracle source)

The oracle `.cpp` is compiled **unmodified**; the only accommodations are build
flags and the stub-header environment (Raven's MSVC precompiled header supplied
these ambiently):

- **`-D_M_IX86` / `-DDEDICATED` / `-DNDEBUG`** — WinDed DEDICATED Release config
  the port models (§ Raven ground truth), not source edits.
- **`-fpermissive`** downgrades one LP64-ism to a warning: the surf-hierarchy
  stride `(int)(&((mdxmSurfHierarchy_t*)0)->childIndexes[n])` casts a pointer to
  `int` — exact on the 32-bit ship (pointer==int width, ruling 44); the computed
  offset is small so the value is identical here.
- **`stubs/compat/memory.h`** (build include path only) redirects `matcomp.c`'s
  `#include <memory.h>` (an SVR4/MSVC header) to `<cstring>`.
- **Stub headers** (`stubs/`) declare exactly the shared/renderer surface the live
  server pipeline touches; the §20 client MD3/tag/bounds code must *compile* (it
  shares tr_model's TU) but is never linked-live, so its md3/orientation types are
  present with any layout (never dumped) and its externs (`R_LoadMDXA/M`,
  `KillTheShaderHashTable`, `VectorNormalize`) are host link stubs.

## Deviations from the doc's plan (with reason)

1. **hit/miss uses `R_HunkClearCrap`+`R_ModelInit` between registers.** The doc
   says "register a model twice (`pqbAlreadyFound` flips)". A plain second
   `RE_RegisterServerModel` short-circuits on the `mhHashTable` hit and never
   re-enters `RE_RegisterServerModels_Malloc`, so the flip is unobservable that
   way. Dropping the model pool + hash (`R_HunkClearCrap`) while keeping
   `CachedModels`, then re-initing the null model, makes the re-register reach the
   cached disk image — the genuine `pqbAlreadyFound=true` repeat branch — with
   zero FS reads and no UB. This is the faithful way to exercise the flip.
2. **`Info_f` entries run together with no per-entry newline.** Faithful: the
   trailing `\n` in `RE_RegisterModels_Info_f` sits inside `#ifdef _DEBUG`
   (`:485`), compiled out under `-DNDEBUG`. The Rust port must reproduce this exact
   NDEBUG format.

## On-disk size

Committed artifacts: 6 fixtures (`.glm`/`.gla`, ~1.2 KB) + 5 goldens (~4.7 KB)
= 11 files, ~5.9 KB total. `build/` is git-ignored.
