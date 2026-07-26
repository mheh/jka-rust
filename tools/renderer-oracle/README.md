# renderer-oracle — differential harness for shader parsing

Compiles the **unmodified** Raven `oracle/codemp/renderer/tr_shader.cpp`
(`R_InitShaders` → `ScanAndLoadShaderFiles` → `R_FindShader` → `ParseShader` →
`FinishShader`) into a standalone dumper, runs it over `fixtures/*.shader`,
and stores the canonical dumps under `golden/`. This is the R4a 2D-first
backbone slice of the renderer port's R3 stage (`docs/plans/2026-07-24-client-port/renderer-plan.md`),
following the §18 oracle-differential pattern (`docs/porting-rules.md` §F18)
DEC-37 ruling 15 puts on the renderer track. The eventual Rust frontend port
(`crates/mp/renderer`, not touched by this harness) must reproduce these
goldens byte-for-byte.

`oracle/` is never edited. The goldens are committed, so `cargo test` (once
the Rust side exists) needs no C++ toolchain — `run.sh` is only needed to
regenerate or spot-check.

## Usage

```sh
sh run.sh           # build the dumper, diff current output against golden/
sh run.sh --regen   # rebuild golden/ (after adding/changing fixtures)
```

Clean-rebuild reproducibility: `rm -rf build && sh run.sh` reproduces
`golden/` byte-for-byte (verified twice during bring-up).

## Build strategy

`run.sh` copies `tr_shader.cpp` and its full sibling-header closure
(`codemp/renderer/*.h`, `codemp/game/*.h` + `q_shared.c`/`q_math.c`,
`codemp/qcommon/*.h`, `codemp/ghoul2/*.h`, `codemp/cgame/*.h`,
`codemp/RMG/*.h`) into `build/`, mirroring `tools/ui-oracle`'s "full header
closure" pattern rather than `tools/gp2-oracle`'s from-scratch stub headers —
like `ui_shared.c`, `tr_shader.cpp` is too large and too entangled with
`tr_local.h`'s renderer-wide type closure for hand-stubbing every header to
be less work. Unlike `ui-oracle`, everything here is **already C++** (no
`.c`/`.cpp` language split to bridge with a `pc_bridge.cpp`-style shim):
`tr_shader.cpp`, `q_shared.c`, and `q_math.c` all compile with `c++ -x c++`,
and every stub in `main.cpp` links against them directly by C++-mangled name.

`main.cpp` (this directory, not oracle) supplies:
- `trGlobals_t tr;` / `glconfig_t glConfig;` — the globals some *other*
  `tr_*.cpp` normally defines (`tr_init.cpp`), zero-initialized like retail's
  own static storage.
- Every `qgl*` ARB/NV entry point `tr_shader.cpp` references, as null
  function pointers (see "GL surface" below).
- Every `ri`-less direct-call engine import `tr_shader.cpp` needs
  (`Com_Error`/`Com_Printf`/`FS_*`/`Hunk_Alloc`/...) — JKA's MP renderer
  calls these as plain globals, not through a `refimport_t` vtable (there is
  no `ri.` indirection anywhere in this TU or `tr_local.h`).
- `R_FindImageFile`/`CIN_PlayCinematic` — the two image/media registration
  seams `tr_shader.cpp` reaches across into `tr_image.cpp`/`cl_cin.cpp` for.
- The canonical dump (field-by-field `shader_t`/`shaderStage_t`, source
  declaration order, `%.6f` floats, symbolic names alongside every enum's
  raw integer value).

### GL surface

`stubs/qgl.h` **replaces** the copied real `oracle/codemp/renderer/qgl.h`
(dropped into `build/codemp/renderer/qgl.h`, overwriting the real copy —
`glext.h`/`qgl_console.h`/`glext_console.h` are deleted from the build tree
entirely once it does, since nothing else references them). The real
`qgl.h` platform-selects a base GL header (`windows.h`+`gl/gl.h`,
`macosx_glimp.h`, `GL/gl.h`+`GL/glx.h`, ...) before declaring ~200
`(APIENTRY *)`-style entry points; this harness never renders, and
`R_InitShaders`/`R_FindShader`/`ParseShader`'s only GL touch-point is
`CreateInternalShaders`' glow-shader setup, which probes exactly 13 ARB/NV
entry points (grep `qgl[A-Za-z]*` over `tr_shader.cpp`) and *every* call site
is a `if (qglFoo)` null-check. `stubs/qgl.h` declares only those 13 plus the
handful of `GL_*`/`ARB_*`/`NV_*` enum values `CreateInternalShaders`
references (values copied verbatim from the real header's own unconditional
`#define` block, never linked against it) — `main.cpp` zero-initializes all
13, so every guard deterministically takes the "extension unavailable"
branch. `CollapseMultitexture` (`tr_shader.cpp:2612`) also gates on
`qglActiveTextureARB` before doing anything, so **GL multitexture
stage-collapsing never fires** in this harness — a two-stage shader that
would collapse into one hardware-multitexture pass on real GL1.3+ hardware
stays as two separate `stages[]` entries in the dump. This is a deliberate,
documented divergence from *some* retail configurations (multitexture was
near-universal by JKA's ship date, but the collapse is backend/hardware
policy, not shader-grammar content) — the R3 Rust port's differential test
must either also disable the equivalent collapse path or accept this as an
intentionally out-of-scope backend concern (§F18 "the goldens gate shader
PARSING, not the whole frontend").

## Stub inventory

Every symbol `main.cpp` defines that isn't in the compiled oracle sources
(`tr_shader.cpp`/`q_shared.c`/`q_math.c`), with its deterministic behavior —
the R3 Rust differential test must reproduce each one identically for the
golden compare to be meaningful:

| Symbol | Behavior |
| --- | --- |
| `tr` / `glConfig` | Zero-initialized `trGlobals_t`/`glconfig_t`, matching retail's own zero-initialized static globals at process start. `main()` additionally populates `tr.defaultImage`, `tr.whiteImage`, `tr.numLightmaps = 1` + `tr.lightmaps[0]`, and all 16 `tr.scratchImage[]` slots before calling `R_InitShaders` — see "lightmap availability" and "videoMap" below. |
| `qgl*` (13 ARB/NV entries) | Null function pointers — see "GL surface" above. |
| `r_detailTextures` / `r_vertexLight` / `r_uiFullScreen` | Each a `cvar_t*` to a zeroed `cvar_t` (`integer = 0`, i.e. "off"). No fixture depends on detail-texture stage culling, vertex-light single-pass collapsing, or the ui-fullscreen two-pass-lightmap suppression `FinishShader` gates on these — "always off" is the simplest deterministic default. |
| `Hunk_Alloc(size, pref)` | `malloc` + zero-fill (retail `Hunk_Alloc` hands back zeroed memory), **never freed** — single-shot process, leaking for the process lifetime is simplest and deterministic. |
| `Com_Memcpy` / `Com_Memset` | Direct `memcpy`/`memset` wrappers — Raven's own real-build implementations reduce to the same thing (see `oracle/codemp/qcommon/common.cpp`), not pulled in to avoid its much larger TU closure. |
| `Com_Printf` / `Com_DPrintf` | `vfprintf` to **stderr** (never stdout, so golden diffs never see them) — visible during `--regen`/spot-checks. |
| `Com_Error(code, fmt, ...)` | Formats the message and **throws** a `ComErrorAbort{code, message}` C++ exception instead of retail's longjmp-to-per-frame-safe-point. `main()`'s driver loop catches one `ComErrorAbort` per `R_FindShader()` call (and once around `R_InitShaders` itself), printing an `ERROR: Com_Error(code=.., "..")` line and moving on to the next fixture name — same "abort this shader, keep going" observable behavior as retail, but stack-unwind-safe (RAII, not a raw `longjmp`) through `ParseTexMod`/`ParseShader`'s C++ frames. Four `Com_Error` call sites exist in this TU; only one is fixture-reachable: `tr_shader.cpp:579` (`ParseTexMod`'s tcMod-overflow guard, `ERR_DROP` — reachable, exercised by `edge_tcmod_overflow`, see `edge_cases.shader`). The other three are unreachable here: `:3019` (`FinishShader`'s lightstyle-without-lightmap guard, `ERR_DROP` — the driver always passes `stylesDefault = {LS_NORMAL, LS_LSNONE, LS_LSNONE, LS_LSNONE}`, and since `LS_LSNONE = 0xff >= LS_UNUSED = 0xfe`, `numStyles` computes to 0 and the guard's `if (numStyles > 0)` never opens); `:3911` (`ScanAndLoadShaderFiles` finds no shader files, `ERR_FATAL` — `run_one` always stages at least one `.shader` file); `:3928` (`ScanAndLoadShaderFiles` couldn't load a staged file, `ERR_DROP` — same reason). An aborted shader's already-parsed `map` lines still consume `R_FindImageFile` counter ticks before the abort fires — see the `R_FindImageFile` row. |
| `FS_ListFiles` / `FS_ReadFile` | Backed by a **real directory** (`argv[1]`, the per-fixture `build/shaders_<fixture>/` staging dir `run.sh` populates) via `opendir`/`fopen` — `FS_ListFiles` returns `.shader`-suffixed basenames in **alphabetical order** (retail's own order depends on pak mount order, which this harness has no equivalent of; alphabetical is the simplest deterministic choice). |
| `FS_FreeFileList` / `FS_FreeFile` | No-ops — single-shot process, never freed. |
| `R_FindImageFile(name, mipmap, allowPicmip, allowTC, glWrapClampMode)` | **Always succeeds** (unlike retail, which fails for a texture missing on disk) — allocates a fresh `image_t` with `width=height=64`, a monotonically increasing `texnum` counter (base 1000, one call = one increment, mirroring `ui-oracle`'s `registerShaderNoMip`-style counters), and `imgName`/`mipmap`/`allowPicmip`/`wrapClampMode` set from the call arguments (`allowTC` is retail-only texture-compression policy with no observable field on `image_t`, so it's accepted and dropped). **No dedup by name** — two stages referencing the same path get two distinct `image_t`s with two distinct `texnum`s (retail's image cache would return the SAME pointer). The golden dump only observes per-stage image *content*, never pointer identity across stages, so dedup would not change any dumped field. The counter's prologue is order-coupled and must be mirrored exactly: `main()` consumes 1000 (`defaultImage`), 1001 (`whiteImage`), 1002 (`lightmaps[0]`), 1003-1018 (16 `scratchImage[]` slots), then `R_InitShaders`' internal/external shader creation consumes 1019-1020, so fixture parsing starts at `texnum = 1021` — any inserted fixture or extra prologue allocation shifts every later `texnum`. A shader whose parse later aborts via `Com_Error` still burns the ticks its already-parsed `map` lines consumed before the abort — `edge_unknown_stage_keyword` is `texnum=1021`, then the aborted `edge_tcmod_overflow` burns `1022`, then `edge_truncated_stage` is `texnum=1023` (see `golden/edge_cases.txt`); the R3 Rust double must burn those ticks too, not skip them. |
| `CIN_PlayCinematic` | `videoMap`'s cinematic handle: a monotonically increasing counter mod `NUM_SCRATCH_IMAGES` (16). `main()` pre-populates every `tr.scratchImage[]` slot with a distinct placeholder image before calling `R_InitShaders`, so the handle is always valid to dereference at `tr_shader.cpp:1460`. |
| `R_InitSkyTexCoords` | No-op — writes to a module-private sky tex-coord table this harness never reads. |
| `R_SyncRenderThread` / `Cmd_Argc` | No-ops / `0` — reached only from `R_MergeShaders` (RMG terrain-blend path) and `R_ShaderList_f` (a console command), neither of which any fixture exercises; present only to satisfy the link. |

### Lightmap availability

The driver (`main()`'s per-name loop) calls `R_FindShader(name, lightmapIndex,
stylesDefault, qtrue)` with `lightmapIndex = {lm0, LIGHTMAP_NONE,
LIGHTMAP_NONE, LIGHTMAP_NONE}`, where `lm0` is `LIGHTMAP_NONE` (`-1`) by
default — matching `CreateExternalShaders`' own calling convention — unless
the names-file line is suffixed `:<lm0>` (e.g. `foo:0`). Since `tr.numLightmaps
= 1`, a `:0` override makes `map $lightmap` take the "lightmap available"
branch (`stage->bundle[0].image = tr.lightmaps[shader.lightmapIndex[0]]`);
every other fixture takes the "no lightmap available" branch (falls back to
`tr.whiteImage`, with a `Com_Printf` warning). See
`stage_keywords.shader`'s `stage_map_lightmap` (not-available) vs.
`stage_map_lightmap_available` (available, driven via
`stage_keywords.names`' `:0` suffix).

## Quirks and findings (genuine, not harness artifacts)

- **`animMap`/`clampanimMap`/`oneshotanimMap` repurpose `textureBundle_t::image`
  as an `image_t **` (§19).** `ParseStage`'s animMap branch
  (`tr_shader.cpp:1400-1443`) collects up to `MAX_IMAGE_ANIMATIONS` (32)
  `image_t*`s into a local array, then does
  `stage->bundle[0].image = (image_t*)Hunk_Alloc(numImageAnimations *
  sizeof(image_t*), h_low); memcpy(stage->bundle[0].image, images, ...)` —
  i.e. it stores an **array of pointers** through a field declared as a
  single `image_t *`. Every other stage keyword (`map`/`clampmap`/
  `videoMap`/`$lightmap`/`$whiteimage`) leaves `numImageAnimations` at 0 and
  `image` as an ordinary single pointer. The dumper's `DumpBundle` detects
  `numImageAnimations > 0` and reinterprets `image` as `image_t **` before
  printing each frame — dereferencing it as a scalar `image_t*` (the
  straightforward reading of the field's declared type) reads garbage bytes
  from the pointer array as if they were a `char imgName[64]`, which is
  exactly what happened during bring-up (caught by a non-ASCII-byte sweep
  over the golden output). The R3 Rust port's `textureBundle` equivalent
  should almost certainly model this as an explicit sum type
  (`Single(ImageId)` vs. `Animated(Vec<ImageId>)`) rather than reproducing
  the raw reinterpret-cast.
- **Exceeding `MAX_SHADER_DEFORMS` desyncs the rest of the shader parse
  (real behavior, not a harness bug).** `ParseDeform`'s overflow guard
  (`tr_shader.cpp:1947-1950`, `MAX_SHADER_DEFORMS` is 3) fires **after**
  consuming only the deform *subtype* token (e.g. `"bulge"`) and returns
  immediately — it never consumes that subtype's own parameter tokens (e.g.
  `bulge`'s width/height/speed triple). Those tokens are left in the token
  stream and get replayed against `ParseShader`'s *outer* general-keyword
  dispatch loop, which doesn't recognize a bare number as a keyword and
  returns `qfalse` — silently turning a 4th `deformVertexes` line into a
  fully-failed (`defaultShader = 1`) parse of the *entire* shader, not just
  a dropped deform. See `general_keywords.shader`'s `gen_deform_overflow`
  and its golden entry (`defaultShader = 1`, `explicitlyDefined = 0`,
  `numUnfoggedPasses = 0` — the three deforms that did parse are still
  dumped as `numDeforms = 3`, since `FinishShader`/`GeneratePermanentShader`
  run even on the failed shader). This is the same class of "non-brace-counted
  recovery desyncs unrelated later content" bug `ui-oracle`'s README
  documents for `PC_Script_Parse` — Raven's shader/menu parsers share the
  "keep reading tokens from a stream, deal with errors by returning `qfalse`
  rather than resynchronizing" design, and both harnesses independently
  rediscovered an instance of it.
- **`ClearGlobalShader` seeds every shader with `CONTENTS_SOLID |
  CONTENTS_OPAQUE`** (`tr_shader.cpp:245`) regardless of any `surfaceParm`/
  `surfaceparm` line — visible in every golden entry's
  `contentFlags = 0x00008001` baseline (`CONTENTS_SOLID = 0x00000001`,
  `CONTENTS_OPAQUE = 0x00008000`) before any explicit content flag is
  OR'd/AND'd in by `ParseSurfaceParm`.

## Fixtures and keyword coverage

Hand-written, not copied from any shipped `.shader` asset (per the work
order). One shader (or stage) block per keyword group for easy review; each
fixture's `.shader` has a matching `.names` file listing which shader
identifiers to `R_FindShader()` + dump, in file order (see "Lightmap
availability" above for the `:<lm0>` suffix syntax).

### `general_keywords.shader` — `ParseShader`'s dispatch table (22/22 keyword groups)

`cull` (6 named synonyms + invalid + default) · `sort` (all 14 named values +
numeric fallback) · `portal` · `skyparms` (full outerbox + `-`/cloudHeight-0
default) · `deformVertexes`/`deform` alias (`wave` incl. div-by-zero guard,
`normal`, `move`, `bulge`, `projectionShadow`, `autosprite`, `autosprite2`,
`text3`, out-of-range `text9` clamp, unknown subtype, `MAX_SHADER_DEFORMS`
overflow) · `nomipmaps` · `nopicmip` · `noglfog` · `polygonOffset` · `noTC` ·
`entityMergable` · `light` · `clampTime` · `fogParms` · `material` /
`q3map_material` alias · `sun` · `surfacelight` · `lightColor` (skip) ·
`tesssize` (skip) · `q3map_*` catch-all (skip) · `qer_*` catch-all (skip) ·
`surfaceParm` (8 representative `infoParms[]` entries + unknown) · zero-stage
sky and fog shaders (the `s==0` guard's two legal exceptions).

`q3map_sun` and `q3map_surfacelight` are declared aliases of `sun` and
`surfacelight` respectively (same dispatch arm, `Q_stricmp(token, "sun") ||
Q_stricmp(token, "q3map_sun")` style) — not separately fixtured, since they
share their group's handler and exercise no distinct code path.

### `stage_keywords.shader` — `ParseStage`'s dispatch table (16/16 reachable keyword groups)

`map` (plain / `$whiteimage` / `$lightmap`, both branches) · `clampmap` ·
`animMap` / `clampanimMap` / `oneshotanimMap` · `videoMap` · `alphaFunc` (all
4 `NameToAFunc` entries + invalid) · `depthFunc` (all 3 + invalid) · `detail`
· `blendFunc` (3 named shortcuts + explicit src/dst pair + invalid) ·
`rgbGen` (all 11 `NameToGenFunc`-independent variants + invalid) · `alphaGen`
(all 11 variants incl. `portal` with and without its range parameter +
invalid) · `tcGen`/`texgen` alias (all 4 + invalid) · `tcMod` (all 7
`ParseTexMod` subtypes + invalid + stacked/multiple-per-stage) · `depthwrite`
· `glow` (JKA addition) · `surfaceSprites` (all 3 types + invalid) · all 13
`ss*` optional parameters + invalid · a two-active-stage composite shader
(exercises `FinishShader`'s lightmap-merge bookkeeping across >1 stage with
GL multitexture-collapse disabled, per "GL surface" above).

Two keyword groups are declared in `ParseStage` but **not compiled** in this
profile and so are absent from the fixtures: `specularmap` (behind
`#ifdef VV_LIGHTING`, never defined in a retail MP build) and `bumpmap`
(behind `#ifdef _XBOX`).

### `edge_cases.shader` — parse-error / recovery paths

Unknown general-shader keyword (→ `defaultShader` fallback) · unknown stage
keyword (→ same fallback) · a legal empty `{ }` stage · `tcMod` overflow
past `TR_MAX_TEXMODS` (4) → `Com_Error(ERR_DROP, ...)`, exercising the
harness's exception-based abort path · a shader truncated mid-file (missing
closing braces at EOF) · (via `edge_cases.names` only, not present in the
`.shader` file at all) a shader name absent from the corpus entirely,
exercising `R_FindShader`'s "not found in shader text" fallback that
constructs a default `CGEN_LIGHTING_DIFFUSE` single-stage shader straight
from a (harness-stubbed, always-succeeding) `R_FindImageFile` call.

## Re-run commands

```sh
cd tools/renderer-oracle
sh run.sh                       # diff against golden/
sh run.sh --regen               # after editing/adding fixtures
rm -rf build && sh run.sh       # clean-rebuild reproducibility check
```
