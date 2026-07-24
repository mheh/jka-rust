# tr_model loader + model cache — MP engine (§F idiomatic reimplementation) Design

> **Amended by DEC-35 (2026-07-24, `docs/plans/2026-07-24-client-port/` era):**
> `model_mdxm`/`model_mdxa` no longer return `*mut c_void` — they return
> `Option<MdxmRef<'static>>`/`Option<MdxaRef<'static>>` (typed views + the
> parsed-once sidecar built at ingest, stored beside `disk_image`), with the
> mdx view module hoisted to `mp_host_interface`. The `*mut c_void`
> statements below are the pre-amendment record; G2SV-D5's substance (no
> ghoul2→renderer edge, no second parse path) is unchanged.
Status: FROZEN (delegated sign-off 2026-07-10, user default-decision grant)     Supersedes: none
Decision prefix: TRM     Ledger deps: DEC-01 (renderer deferral), DEC-04 (per-mode),
DEC-08 (Com_Error recovery), DEC-09 (verification); engine-fork-discovery rulings
**59** (2026-07-10, delegated: `EngineHost` = **20 methods**, commit `4c303bd1` —
the 20th `fs_file_is_in_pak` PAK-membership method closes `TRM-Q5`; also
§20-drops the client-only `RE_RegisterMedia_LevelLoadEnd` — `TRM-D5`),
**58** (2026-07-10: the `AlignedBytes` disk-image buffer type + sound seam casts;
closes `TRM-Q4`, confirms `TRM-Q3` wave-20-resolved — `TRM-D4`),
**56** (2026-07-09: closeout — matcomp home + `EngineHostView` self-borrow + the
blanket §F Cargo-edge authorization; closes `TRM-Q1`/`TRM-Q2` — `TRM-D1`),
**55** (2026-07-09: `EngineHost` = **19 methods**, commit `b2855df2`; the
StringEd-forced cvar surface — `TRM-D2`), and the standing rulings this pass
carries forward (`TRM-D3`): **52** (disk-image ownership + faithful level
eviction + four-point producer contract; `AlignedBytes` spelling per ruling 58),
**53** (`mp_renderer` `Engine` field +
`Vec` pool), **54** (§20 the dead, keep the live records), **40** (`_s`-struct
naming), **44** (Win32 `long` in binary formats), plus **43** (the
`EngineHostView` split-borrow accessor `Engine::<x>_call`) and **51** (this doc
queued, no deferrals). Cross-doc: `ghoul2-server.md` `G2SV-D2` (the punt boundary
this doc fills), `G2SV-D5` (the `EngineHost` model-memory seam, no crate edge);
rulings 11/36/55 (the BUILT `EngineHost` trait whose `model_mdxm`/`model_mdxa`
backing this doc owns).

This is a C++-track (`porting-rules.md` §F) design doc. It carries the
machine-readable `files:` roster and `divergences:` list (doc-standards rule 6)
that `.claude/workflows/port-cpp-subsystem.js` consumes via `designPath`; both
live in the `## Files roster` YAML block below.

## Standing context

Links only — never restated here:
- `docs/workspace-architecture.md` — crate graph. This subsystem lands in
  `mp_renderer` (`crates/mp/renderer`), the crate that **already owns**
  `model_s`/`model_t` (`src/tr_local/model_s.rs`), `trGlobals_t`
  (`src/tr_local/tr_globals_t.rs`), and the `mdxm*`/`mdxa*` headers
  (`src/mdx_format/`) — **except** `matcomp`, which lands in `mp_engine_ghoul2`
  beside its sole live consumer (`TRM-D1`(a), ruling 56a). `mp_renderer`'s deps
  today are `mp_qshared`, `mp_engine_qcommon`, `native_platform` (`Cargo.toml`);
  this doc's `## Seam definition` adds one more — `mp_host_interface` — because
  its registry methods name `EngineHost` (`TRM-D1`(c), the ruling-56c blanket
  §F-consumer authorization; acyclic since `mp_host_interface` deps only
  `mp_qshared`). Crucially there is **no edge from `mp_engine_ghoul2` to
  `mp_renderer`** (`G2SV-D5`), so the frozen bone subsystem reaches this doc's
  model memory only across the `EngineHost` service seam, never by naming a
  `mp_renderer` type — and matcomp lives on the ghoul2 side of that seam
  (`TRM-D1`(a)).
- `docs/porting-rules.md` — §F (C++ track: §F17 shape-first, §F18 differential
  goldens, §F19 UB divergence, §F20 drop-dead-surface, §F21 one-class-per-file),
  §C10 (fold compiled-out arms), §E13 (slice-driven), the comment/source-ref
  rules.
- `docs/subsystems/ghoul2-server.md` — the **FROZEN** sibling. Its `G2SV-D2`
  (`:118-125`, `:791-799`) fixes the ghoul2 renderer subset at exactly
  `tr_ghoul2.cpp`'s bone extent and **punts these loader TUs to "a separate
  `tr_model` subsystem doc"** — this doc. Its `G2SV-D5` freezes the
  `EngineHost::model_mdxm`/`model_mdxa` seam (raw `*mut c_void` into the parsed
  block, never an `mp_renderer` struct); **this doc owns the memory those
  accessors point into**. The `EngineHost` trait is quoted verbatim in
  `ghoul2-server.md` `## Seam definition` and re-quoted verbatim below at its
  ruling-59 20-method surface (`TRM-D2`, `TRM-D5`).
- `docs/plans/2026-07-08-mp-engine-build-out.md` — the WinDed DEDICATED link set
  (§570 macro set; §103 "`GetRefAPI` builds `refexport_t` — only 1 assign
  survives under `DEDICATED`", the signal that the dedicated renderer surface is
  a headless model/shader loader, not a draw engine).
- `docs/GOAL-engine.md` — the M3 renderer waves this doc gates (ruling 51).
- `docs/decisions.md` — DEC-01 (renderer port deferred; the client draw/GL
  surface this doc §20-drops belongs to that deferred work), DEC-04 (strict
  per-mode; MP/`jamp` dedicated slice only), DEC-08 (`Com_Error` → panic +
  `catch_unwind`), DEC-09 (TU harnesses + live peers).
- `docs/handoffs/engine-fork-discovery.md` — rulings 40/43/51/52/53/54/55/56.
- `crates/mp/host-interface/` — the BUILT, frozen `mp_host_interface` crate
  (`EngineHost` = 20 methods, commit `4c303bd1`, rulings 55 + 59a — the 20th is
  `fs_file_is_in_pak`); `src/engine_host.rs` holds `model_mdxm`/`model_mdxa`
  (ruling 36) and `fs_file_is_in_pak` (`:209`, ruling 59a). Reading it is
  required; this doc implements the model backing and re-quotes the trait verbatim.
- Already type-ported (layout frozen, reused never re-declared, `TRM-D3`/ruling
  40): `model_t` (`crates/mp/renderer/src/tr_local/model_s.rs`, `size 136`,
  `mdxm`@112/`mdxa`@120), `trGlobals_t` (`src/tr_local/tr_globals_t.rs`,
  `models`@3840 `[*mut model_t; 1024]`, `numModels`@12032), `modtype_t`
  (`MOD_BAD`/`MOD_MDXM`/`MOD_MDXA`, `tr_local.h:1104-1111`), and every `mdxm*`/
  `mdxa*`/`md3*` header the loaders parse (`src/mdx_format/`).

## Scope & non-goals

**In scope.** The idiomatic reimplementation of the **7 WinDed vcproj renderer
TUs** `G2SV-D2` punted here plus the never-linked `null_renderer.cpp` stub (99
fns total):
- `tr_model.cpp` (1,838 LOC) — the model cache and the **sole live
  dedicated-server model-load pipeline**: the `CachedEndianedModelBinary_s`
  cache + `CachedModels_t` map (`:48-68`), the `tr.models[]` pool + `mhHashTable`
  hash (`:36`, `:611-667`), and `RE_RegisterServerModel`/`ServerLoadMDXM`/
  `ServerLoadMDXA` (`:1003`, `:799`, `:683`) — the entry ghoul2 calls, gated at
  `G2_API.cpp:589,2619,2710`.
- `matcomp.c` (293 LOC) — `MC_Compress`/`MC_UnCompress`/`MC_UnCompressQuat`
  (`:50`, `:163`, `:219`). `MC_UnCompressQuat` is consumed by the **frozen**
  ghoul2 bone subset (`UnCompressBone`, `tr_ghoul2.cpp:1158-1162`), so per
  `TRM-D1`(a) (ruling 56a) matcomp's crate **home is `mp_engine_ghoul2`**, beside
  that consumer — the codec is part of the mdxa format the bone subset decodes.
  (`TRM-Q1` is closed; see `## Decisions`.)
- The **DEDICATED-dead** surface of `tr_shader.cpp`, `tr_image.cpp`,
  `tr_init.cpp`, `tr_main.cpp`, `tr_mesh.cpp`, `null_renderer.cpp` (~73 fns),
  classified §20/§C10 per-TU below (`TRM-D3`, ruling 54).

**This doc owns the memory behind the frozen `EngineHost::model_mdxm`/
`model_mdxa` pointers** (`G2SV-D5`; the parsed `.glm`/`.gla` block a
`CachedEndianedModelBinary` owns, `TRM-D3`/ruling 52).

**Non-goals (punted, with pointers).**
- The frozen bone subset of `tr_ghoul2.cpp` — `docs/subsystems/ghoul2-server.md`
  (FROZEN). This doc supplies the model memory it reads, nothing it kept.
- The client draw / GL / shader-compile / image-upload surface. It is
  DEDICATED-dead here (§20/§C10, `TRM-D3`) **and** belongs to the deferred client
  renderer (DEC-01); it is not re-added by this dedicated slice.
- `tr_backend.cpp` — also in the WinDed vcproj renderer set, but the GL backend,
  **not in this doc's settled 8-TU inventory**; deferred client renderer
  (DEC-01). (Noted so a reader doesn't read its absence as an oversight.)
- SP (`jasp`) loader (`oracle/code/renderer/`) — a future DEC-04 diff (duplicate,
  don't unify).
- The `EngineHost` trait itself — BUILT and frozen in `mp_host_interface`
  (rulings 55/59a, 20 methods); this doc **implements** the `model_mdxm`/
  `model_mdxa` backing and **consumes** `fs_file_is_in_pak` (the PAK-checksum
  seam, `TRM-D5`), and does not re-declare the trait (it re-quotes it verbatim,
  `TRM-D2`).

## Raven ground truth

**Build config.** WinDed DEDICATED Release: `-DNDEBUG -DDEDICATED -DBOTLIB`,
`FINAL_BUILD` undefined (plan `:570`). `_M_IX86` is **defined** (x86 target), so
every `#ifndef _M_IX86` big-endian block compiles **out** (`TRM-D3`); the port is
still LE-correct because `LittleLong`/`LittleShort`/`LittleFloat` are identity on
LE (the `LL()` macro `tr_model.cpp:20`).

**The dedicated init + call chain (the live surface).**
- `sv_init.cpp:578` (the `com_dedicated->integer` arm) calls `R_SVModelInit()`
  (`tr_model.cpp:1655`) → `R_ModelInit()` (`:1665`): `new CachedModels_t`,
  `tr.numModels = 0`, `memset(mhHashTable)`, and one `R_AllocModel()` reserved as
  the NULL model (`MOD_BAD`, `:1678-1679`). (The direct `R_ModelInit()` at
  `sv_init.cpp:571` is inside a commented-out block; `R_SVModelInit` is the live
  entry.)
- `sv_init.cpp:481` calls `RE_RegisterMedia_LevelLoadBegin(server, eForceReload)`
  (`tr_model.cpp:522`): when `eForceReload` is neither `eForceReload_MODELS` nor
  `eForceReload_ALL` (the `bDeleteModels` else-arm, `:526,533`) it runs
  `RE_RegisterModels_DumpNonPure()` on `sv_pure->integer` (`:535-538`) — the
  `bDeleteModels` arm itself calls `RE_RegisterModels_DeleteAll` (`:531`); it bumps
  `giRegisterMedia_CurrentLevel` only when the map name changed (the `sPrevMapName`
  file-static, `:560-565`). Its `R_Images_DeleteLightMaps()` tail is `#ifndef
  DEDICATED` (`:543-551`, dead).
- `z_memman_pc.cpp:226` calls `RE_RegisterModels_LevelLoadEnd(qtrue)`
  (`tr_model.cpp:337`) from the `Z_Malloc`-fail recovery path: it evicts cached
  models whose `iLastLevelUsedOn` is stale, gated by
  `r_modelpoolmegs->integer * 1024*1024` vs `GetModelDataAllocSize()` (`:351-352`,
  the `TAG_MODEL_MD3`+`GLM`+`GLA` `Z_MemSize` sum, `:326-331`); returns `qtrue` if
  it freed at least one (the recovery signal). **The Rust port derives
  `GetModelDataAllocSize` by summing `cached[*].alloc_size` locally over
  `RenderModels.cached` — no Zone-allocator seam** (`TRM-D3`, ruling 54
  consequence): `Z_MemSize(tag)` is `TheZone.Stats.iSizesPerTag[tag]`, a running
  sum of each live allocation's requested `iSize` (`z_memman_pc.cpp:292,338,446`),
  and every `TAG_MODEL_*` producer other than the server pipeline is
  §20-dropped/frozen-dead on the dedicated build — `TAG_MODEL_MD3` comes only
  from `R_LoadMD3` (`tr_model.cpp:1467`, §20 client loader) and the client
  `TAG_MODEL_GLM`/`GLA` from `R_LoadMDXM`/`R_LoadMDXA` (`tr_ghoul2.cpp:4859,5311`,
  the frozen doc's dead-here TU). So on the dedicated build the only live
  `TAG_MODEL_*` bytes are the server `disk_image` buffers, each recorded as
  `iAllocSize` (`RE_RegisterServerModels_Malloc` `:718,841`), and
  `Z_MemSize(MD3)+Z_MemSize(GLM)+Z_MemSize(GLA)` equals the local sum
  **byte-exactly**. (The real Zone allocator is type-only today; this derivation
  needs neither its function bodies nor a new `EngineHost` query.)
- `G2_API.cpp:589,2619,2710` call `RE_RegisterServerModel(name)`
  (`tr_model.cpp:1003`) whenever `G2_ShouldRegisterServer()` (or
  `com_dedicated->integer`). This is the **only** model registration the dedicated
  server reaches at runtime — the client `RE_RegisterModel` is the `else` arm at
  those three sites, dead when the server registers.
- `z_memman_pc.cpp:768` calls `R_HunkClearCrap()` (`tr_model.cpp:1683`):
  `KillTheShaderHashTable()` + zero `tr.numModels`/`mhHashTable`/`numShaders`/
  `numSkins` on a hunk reset.

**`RE_RegisterServerModel` (`:1003`).** Lazily **registers** `r_noServerGhoul2`
(`:1020-1023`, `r_noServerGhoul2 = Cvar_Get( "r_noserverghoul2", "0", 0)` —
"keep it from choking … Registering all r_ cvars for the server would be a Bad
Thing"), name-length-guards (`MAX_QPATH`), then hash-looks-up
`mhHashTable[generateHashValue(name)]` returning the cached handle on a
case-insensitive (`Q_stricmp`) name hit (`:1033-1042`). On a miss it
`R_AllocModel()`s, copies the name, and loops LODs (`iLODStart = MD3_MAX_LODS-1`
for `.md3`, else 0, `:1056-1067`) — for each LOD it `RE_RegisterModels_GetDiskFile`s
the (LOD-mangled) filename, reads the `ident` (LE-swapped iff not cached), and
dispatches: `MDXA_IDENT → ServerLoadMDXA`, `MDXM_IDENT → ServerLoadMDXM`,
**default → `goto fail`** ("out of luck" — no MD3/shader load on the server,
`:1100-1111`). `FS_FreeFile(buf)` iff **not** already cached (`:1113-1115`). On
success it dup-fills higher LOD slots (`:1132-1135`), `RE_InsertModelIntoHash`es,
and returns `mod->index` (`:1142`). On failure (the `fail:` label) it keeps the
`model_t` as `MOD_BAD`, **still inserts it into the hash** so the name isn't
rescanned, and **returns a literal `0` — NOT `mod->index`** (`:1148-1153`; the
`return 0;` is at `:1153`). The failed entry therefore sits hashed under its
nonzero `R_AllocModel`'d index (allocated before the fail branch) while every
`G2_API.cpp` caller gets back `0`, which is exactly how those callers detect a
failed registration. This zero-on-fail asymmetry is the `MOD_BAD` failed-entry
retention (`TRM-D3`/ruling 53). The lazy `Cvar_Get`
becomes `EngineHost::cvar_register("r_noserverghoul2", "0", 0)`; the read is
`cvar_integer` (`TRM-D2`, ruling 55 — `Cvar_Get`-then-read).

**`ServerLoadMDXA` (`:683`) / `ServerLoadMDXM` (`:799`).** Read `version`/`ofsEnd`
raw, `LittleLong` them **only if `!bAlreadyCached`** (`:703-707`, `:826-830`);
reject on `version != MDXA_VERSION`/`MDXM_VERSION`. Set `mod->type`
(`MOD_MDXA`/`MOD_MDXM`), add `size` to `mod->dataSize`, and set
`mod->mdxa`/`mod->mdxm = RE_RegisterServerModels_Malloc(size, buffer, mod_name,
&bAlreadyFound, TAG_MODEL_GLA/GLM)` (`:717-718`, `:840-841`). The
`assert(bAlreadyCached == bAlreadyFound)` (`:720`, `:843`) ties the two flags. On
`!bAlreadyFound` (a just-morphed disk buffer): set `bAlreadyCached = qtrue`
back-through the reference (so the caller skips `FS_FreeFile`, "we've hijacked
that memory block", `:722-731`, `:845-854`) and `LL()`-swap the header fields
in place (`:734-739`, `:857-863`). Reject `numFrames < 1`; if `bAlreadyFound`,
`return qtrue` **before** any further swap (`:742-749`, `:875-878`). `ServerLoadMDXM`
additionally recurses `mdxm->animIndex = RE_RegisterServerModel("%s.gla",
mdxm->animName)` (`:867`) and sets `mod->numLods = mdxm->numLODs-1` (`:873`).
The remaining per-surface/per-LOD work — the surface-hierarchy walk
(`:880-902`), the LOD/surface field swaps + `SHADER_MAX_VERTEXES`/`INDEXES`
bounds checks + `ident = SF_MDX` (`:905-991`) — **runs on intel too** (the
comment `:904` "we need to do the middle part of this even for intel, because of
shader reg and err-check"). Inside it, `RE_RegisterModels_StoreShaderRequest(
mod_name, &surfInfo->shader[0], &surfInfo->shaderIndex)` (`:898`) records the
shader-poke offsets (`surfInfo->shaderIndex = 0`, "We will not be using shaders
on the server", `:892-896`). The `#ifndef _M_IX86` bone-ref/triangle/vertex
swaps (`:938-983`, `:751-790`) compile out (`TRM-D3`).

**The cache — `CachedEndianedModelBinary_s` + `CachedModels_t` (`:48-68`).** A
`map<sstring_t, CachedEndianedModelBinary_t> *CachedModels` (`:67-68`, heap
singleton, `new`d in `R_ModelInit` `:1671`, `delete`d in `R_ModelFree` `:1696`).
Each entry owns `void *pModelDiskImage` (the parsed model block, `Z_Malloc`/
`Z_MorphMallocTag`-tagged, `Z_Free`d on eviction), `int iAllocSize`, a
`ShaderRegisterData_t` (`vector<pair<int,int>>` of name-offset/poke-offset,
`:46-52`), `int iLastLevelUsedOn` (`-1` init), `int iPAKFileCheckSum` (`-1` if
not from a PAK). The map key is the **lowercased** model name (`Q_strlwr`,
`:77`, `:132`, `:186`, `:260`). Its consumers:
- `RE_RegisterModels_GetDiskFile` (`:125`) — returns the cached block (setting
  `*pqbAlreadyCached = qtrue`) or, on a cache miss, `FS_ReadFile`s from disk.
  **Special case:** `sDEFAULT_GLA_NAME ".gla"` returns a `Z_Malloc`'d copy of the
  294-byte `FakeGLAFile[]` blob (`:95-116`, `:143-152`) — a program-internal
  default skeleton, never disk-loaded.
- `RE_RegisterServerModels_Malloc` (`:253`) — the **server** variant. On a fresh
  entry it morphs the disk buffer's alloc tag (or `Z_Malloc`s if `NULL`, the
  "limb hierarchy creation" case), stores it as `pModelDiskImage`, records
  `iAllocSize` + `iPAKFileCheckSum` (`FS_FileIsInPAK`), and sets `*pqbAlreadyFound
  = qfalse`. On a repeat entry it sets `*pqbAlreadyFound = qtrue` — **the shader
  re-register replay is commented out** ("No. Bad.", `:293-316`). Always stamps
  `iLastLevelUsedOn = RE_RegisterMedia_GetLevel()`.
- `RE_RegisterModels_Malloc` (`:179`) — the **client** variant, byte-identical
  except the repeat-entry branch **replays** the shader pokes under `#ifndef
  DEDICATED` (`:221-242`: for each recorded `(nameOffset, pokeOffset)`,
  `R_FindShader` and poke `sh->index`). Reached only by `R_LoadMD3` (`:1467`),
  i.e. the client loader path → not live on the dedicated server (`TRM-D3`); the
  `#ifndef DEDICATED` replay is the client-dead arm ruling 54 names.
- `RE_RegisterModels_StoreShaderRequest` (`:70`) — pushes an offset pair onto the
  entry's `ShaderRegisterData`. **Live server-side** (called by `ServerLoadMDXM`
  `:898`, unguarded); the recorded vector **stays on the cache struct** even
  though the server never replays it (`TRM-D3`, ruling 54).
- Eviction: `RE_RegisterModels_LevelLoadEnd` (`:337`, level-keyed +
  pool-megs-gated), `RE_RegisterModels_DumpNonPure` (`:418`, dumps entries whose
  `FS_FileIsInPAK` checksum no longer matches — but **never** `*default.gla`,
  `:438`), `RE_RegisterModels_DeleteAll` (`:496`). All iterate `CachedModels` in
  `std::map` key order and `erase()` in place; `RE_RegisterModels_Info_f`
  (`:467`) prints the same ordered walk. `gbInsideRegisterModel` (`:1406`, set
  around `RE_RegisterModel_Actual` `:1409-1414`) guards `LevelLoadEnd` from
  re-entrant eviction during a load (`:345-348`).

**The model pool + hash (`:36`, `:593-667`).** `tr.models[MAX_MOD_KNOWN=1024]`
(`tr_local.h:1138,1396`) is a Hunk-allocated `model_t*` array with `tr.numModels`
the high-water mark. `R_AllocModel` (`:611`) `Hunk_Alloc`s a `model_t`, sets
`->index = tr.numModels`, appends, and returns `NULL` at the `MAX_MOD_KNOWN` cap.
`R_GetModelByHandle` (`:593`) returns `tr.models[0]` (the default) for
out-of-range handles, else `tr.models[index]`. `mhHashTable[FILE_HASH_SIZE=1024]`
(`:35-36`) is a `modelHash_t*` intrusive-chain hash keyed by `generateHashValue`
(`:635`: sum of `tolower(letter)*(i+119)`, stopping at `.`, `\` → `/`, masked to
`size-1`); `RE_InsertModelIntoHash` (`:653`) `Hunk_Alloc`s a node and prepends it.
Lookups compare the **full name** case-insensitively (`Q_stricmp`), so each name
resolves to one handle regardless of bucket collisions.

**matcomp (`matcomp.c`).** Pure bit-packing math over `float mat[3][4]`, no
globals, no I/O. `MC_Compress` (`:50`) and `MC_UnCompress` (`:163`) pack/unpack
the 24-byte (`MC_COMP_BYTES`, `matcomp.h`) matrix; `MC_UnCompressQuat` (`:219`)
reads a 16-bit-quantized quaternion+translation into a rotation matrix. In the
WinDed build **only `MC_UnCompressQuat` has a live caller** (`UnCompressBone`,
`tr_ghoul2.cpp:1158-1162`, inside the frozen bone-eval chain); `MC_UnCompress`
appears only in commented-out `tr_ghoul2.cpp` lines (`:1705-1746`) and
`MC_Compress` has **no caller in `codemp/`**. All three still compile (the TU is
linked); per `TRM-D1`(a) (ruling 56a) matcomp lands in `mp_engine_ghoul2`, and
`MC_Compress`/`MC_UnCompress` take §20 zero-caller notes **iff** their callers
prove dead at port (the verify-at-port condition, `## Decisions`). Ruling 44
governs any Win32-`long` width in the formats — here the packing is `unsigned
int`/`unsigned short` (`matcomp.c:64` etc.), i32/u16-width.

**Loader cvars (host reads via register-then-read).** `sv_pure` (`:178`),
`r_modelpoolmegs` (`:332`), `r_noServerGhoul2` (`:1020`) are `extern cvar_t*`
read by the live cache/pipeline. Their client `Cvar_Get` **definitions** sit in
`R_Register` (`tr_init.cpp:985,1159,1180`), reached only through the `#ifndef
DEDICATED` `R_Init`→`RE_BeginRegistration` client path (`R_Init` `:1214`, called
from `RE_BeginRegistration` `tr_model.cpp:1631`, itself `#ifndef DEDICATED`), so
`R_Register` is part of the dead `tr_init` surface. Per `TRM-D2` (ruling 55) the
live path uses **`EngineHost::cvar_register` at init + `cvar_integer` per read**,
matching Raven's `Cvar_Get`-then-read: the lazy `Cvar_Get("r_noserverghoul2",
"0", 0)` at `:1020-1023` becomes a `cvar_register` call (establishes the default
once; not a no-op collapse), and each `->integer` read is `cvar_integer` (missing
→ 0, matching `Cvar_VariableIntegerValue`). `sv_pure` is server-owned
(`sv_main.cpp`); the goldens inject each value via `MockHost` (`## Verification
strategy`).

**The DEDICATED-dead surface (~73 fns, per-TU §20/§C10, ruling 54).** The
`refexport_t` the dedicated build exports is the ground-truth boundary:
`GetRefAPI` (`tr_init.cpp:1459`) sets **only** `re.Shutdown = RE_Shutdown`; every
other assignment (`RegisterModel`, `BeginRegistration`, `LerpTag`, `ModelBounds`,
draw/scene/font/light entries) is inside `#ifndef DEDICATED` (`:1473-1529`). So
the dedicated renderer ABI surface is one teardown call; the model loader is
reached **internally** (direct C++ calls from ghoul2/server), not through
`refexport_t`. Classification:

| TU | fns | verdict | evidence / cites |
|---|---|---|---|
| `tr_model.cpp` client path | `RE_RegisterModel`/`_Actual`, `R_LoadMD3`, `R_LerpTag`, `R_ModelBounds`, `R_GetTag`, `RE_BeginRegistration` | §20 (client-only; no live dedicated caller — not in the DEDICATED `refexport_t`, `tr_init.cpp:1472-1529`; the `G2_API.cpp:589,2619,2710` `else` arm is dead when the server registers) | `tr_model.cpp:1169,1407,1427,1768,1744,1629` |
| `tr_model.cpp` client `_Malloc` | `RE_RegisterModels_Malloc` + its `#ifndef DEDICATED` shader-poke replay | §20 fn / §C10 arm (client loader only, `:1467`; the poke replay `:221-242` is the client-dead arm) | `tr_model.cpp:179,221-242` |
| `tr_shader.cpp` | ~all (shader parse/compile/`R_FindShader`/lightmap/tables) | §20 (client draw; DEDICATED refexport has no shader entry) — **live cross-ref:** `KillTheShaderHashTable` (called by `R_HunkClearCrap`, `tr_model.cpp:1685`) | `tr_shader.cpp` (whole TU); `KillTheShaderHashTable` cited from `tr_model.cpp:1682` |
| `tr_image.cpp` | ~all (GL texture upload/scale/`R_CreateImage`) | §20 (client draw); 6 `#ifndef DEDICATED` blocks | `tr_image.cpp` (whole TU) |
| `tr_init.cpp` | `R_Register`, `R_Init`, `GetRefAPI` body, GL/glow init | §20 except: `RE_Shutdown` (`:1333`, the sole DEDICATED refexport entry, only `Cmd_RemoveCommand` calls survive `:1337-1348`, touches no model state) is §C10-folded; the loader-cvar `Cvar_Get`s (`:1159,1180`) map to `cvar_register`/`cvar_integer` (`TRM-D2`) | `tr_init.cpp:985,1214,1459,1333` |
| `tr_main.cpp` (~28 fns) | scene/cull/project/light | §20 (client draw); 4 `#ifndef DEDICATED` blocks | `tr_main.cpp` (whole TU) |
| `tr_mesh.cpp` (~7 fns) | `R_AddMD3Surfaces`/`R_CullModel`/`R_ComputeLOD`/… | §20 (client draw; `R_LoadMDXA`/`R_LoadMDXM` live-loader siblings are in `tr_ghoul2.cpp`, the frozen doc's TU, dead there too) | `tr_mesh.cpp:58,173,281` |
| `null_renderer.cpp` (21 LOC) | the null-renderer stub | §20 (never linked in the WinDed set — the real renderer TUs are) | `null/null_renderer.cpp` |

Each **live** or §C10-folded function is named individually above; the uniformly
§20 bulk is classified by TU because every one is the client GL/draw surface the
DEDICATED `refexport_t` excludes.

## State ownership

Every global the survey found. Per `TRM-D3`/ruling 53 the cache map + model pool
become fields of one **`mp_renderer` state struct `RenderModels`**, a direct
field of `Engine`. Because `RenderModels`'s pub methods take `(&mut self, host:
&mut impl EngineHost)`, they are reached through the ruling-43 split-borrow
accessor **`Engine::render_models_call(&mut self) -> (EngineHostView<'_>, &mut
RenderModels)`** — never a bare `&mut Engine.render_models` (an `impl EngineHost
for Engine` would alias the very field being mutated). The accessor's
`EngineHostView` borrows the host-service fields those methods need
(`common`/`sv`/loader — `fs_read_file`/`print`/`cvar_register`/`cvar_integer`)
and sets its `render_models: Option<&mut RenderModels>` field to **`None`**
(`TRM-D1`(b), ruling 56b); the `&mut RenderModels` is handed out separately as
the exclusive borrow. This is sound because **no `RenderModels` method calls
`model_mdxm`/`model_mdxa`** — `get_model` reads its own pool directly (`&self`),
and the `## Slice hooks` host-service map lists exactly the host methods each
`RenderModels` method binds (`model_*` is not among them). The `model_mdxm`/
`model_mdxa` seam is **ghoul2's**, answered by the `EngineHostView` from
`ghoul2_call` — where `render_models` **is** borrowed (`Some`), because there it
is not otherwise held. (`ghoul2_call` is ruling-56b-named
(`engine-fork-discovery.md:594-595`) and follows the ruling-43 `Engine::<x>_call`
split-borrow pattern this doc's `render_models_call` instantiates; no doc yet
records its formal Seam signature — `TRM-Q3`. This doc's soundness argument needs
only the settled name plus the `Some`-filling, never the accessor's second tuple
element type, which names ghoul2's own state struct and is `ghoul2-server.md`'s
territory.) One view type; a `model_*` call on a `None`
`render_models` is a contract violation → panic, fatal-bug class per fork 1
(`TRM-D1`(b); the invariant is also recorded in `state-ownership.md`). `matcomp`
has no globals. `sv_pure`/`r_modelpoolmegs`/`r_noServerGhoul2` are host
register-then-reads (no stored owner), mirroring the
`broadsword`/`cg_g2MarksAllModels` rows in `ghoul2-server.md`.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `CachedModels` (`map<sstring_t, CachedEndianedModelBinary_t>*`) | `tr_model.cpp:67-68` | `mp_renderer::RenderModels.cached: BTreeMap<String, CachedEndianedModelBinary>` — sorted-map iteration order kept for eviction/`Info_f` parity (same precedent as `stringed.md` ruling 50 / `ghoul2-server.md` `GoreRecords`); key = lowercased name (`TRM-D3`) | lazily on first `R_ModelInit` (Raven's `new CachedModels_t`) | `render_models_call` split-borrow (State ownership intro) |
| `CachedEndianedModelBinary_s::pModelDiskImage` (+`iAllocSize`, `ShaderRegisterData`, `iLastLevelUsedOn`, `iPAKFileCheckSum`) | `tr_model.cpp:48-64` | `CachedEndianedModelBinary { disk_image: Option<AlignedBytes>, alloc_size, shader_register_data: Vec<(i32,i32)>, last_level_used_on, pak_file_checksum }` — `AlignedBytes` (16-byte-aligned `alloc::alloc`, `TRM-D4`/ruling 58) heap-pinned/address-stable, in-place mutable for the endian swap, drop deallocs the same `Layout` (ruling 52 ownership + ruling 58 spelling); `None` mirrors `pModelDiskImage == NULL` | `RE_RegisterServerModels_Malloc` (morph/`Z_Malloc`) | field of the `cached` entry |
| `tr.models[MAX_MOD_KNOWN]`, `tr.numModels` | `tr_local.h:1396-1397`; `tr_model.cpp:611-624` | `RenderModels.models: Vec<Box<ModelData>>` (qhandle_t = index, `MAX_MOD_KNOWN=1024` cap → `R_AllocModel` returns `None`), `num_models: i32`. `ModelData` = the already-ported `model_t` (`TRM-D3`/ruling 40 reuse; whether a thin wrapper or the `model_t` directly is §D12 porter latitude), `Box`-pinned so a registered `model_t*` stays address-stable (`G2_API.cpp:2716` caches `currentModel`, `TRM-D3`(c)) | `R_AllocModel` | `render_models_call` (intro) |
| `mhHashTable[FILE_HASH_SIZE]` (+`modelHash_t` chains) | `tr_model.cpp:35-36,653-667` | `RenderModels.hash: name→qhandle_t map` (lookup-only, no ordered iteration → container choice is §D12 latitude, `HashMap` suffices) — replaces the intrusive chains (`TRM-D3`/ruling 53); key = case-insensitive name (`Q_stricmp`) | `RE_InsertModelIntoHash` | `render_models_call` (intro) |
| `giRegisterMedia_CurrentLevel` | `tr_model.cpp:521` | `RenderModels.current_level: i32` (three-kind persistent, ruling 2/3) | `RenderModels::default` (0) | `render_models_call` (intro) |
| `sPrevMapName` (`LevelLoadBegin` static) | `tr_model.cpp:560` | `RenderModels.prev_map_name: String` (fn-static → owned field, ruling 3 persistent) | `RenderModels::default` (empty) | `render_models_call` (intro) |
| `gbInsideRegisterModel` | `tr_model.cpp:1406` | `RenderModels.inside_register_model: bool` (re-entrancy guard; the `RE_RegisterModel_Actual` wrapper `:1409-1414`) | `RenderModels::default` (false) | `render_models_call` (intro) |
| `tr.numBSPModels` | `tr_local.h`; `tr_model.cpp:541,1231` | `RenderModels.num_bsp_models: i32` (the server increments it; the `#ifndef DEDICATED` `RE_LoadWorldMap_Actual` body is dead, `:1232-1234`) | `RenderModels::default` (0) | `render_models_call` (intro) |
| `FakeGLAFile[]` (const default-skeleton blob) | `tr_model.cpp:95-116` | `const` item (three-kind const-table) | — | — |
| `sv_pure`, `r_modelpoolmegs`, `r_noServerGhoul2` (`extern cvar_t*`) | `tr_model.cpp:178,332,1020`; defined `tr_init.cpp:1159,1180`, `sv_main.cpp` | **no stored owner** — `EngineHost::cvar_register(name, default, flags)` at the live init site (the lazy `Cvar_Get` `:1020-1023`), then `cvar_integer(name)` per read (missing → 0) — `TRM-D2`, ruling 55; same treatment as `ghoul2-server.md`'s `broadsword`/`cg_g2MarksAllModels` rows | — (not stored) | `&mut impl EngineHost` at each register/read site |
| `matcomp` file-scope | — (none; pure math) | — | — | — |

## Seam definition

Per doc-standards rule 5 the pub signatures freeze here; porters transcribe into
them without change.

**The `EngineHost` trait, re-quoted verbatim** from
`crates/mp/host-interface/src/engine_host.rs` (`TRM-D2`/`TRM-D5`, rulings 55 +
59a, **20 methods**, commit `4c303bd1`; the trait is BUILT and frozen — this doc
**implements the `model_mdxm`/`model_mdxa` backing** and **consumes**
`fs_read_file`/`fs_free_file`/`print`/`cvar_register`/`cvar_integer`/
`fs_file_is_in_pak`, and does **not** re-declare the trait):

```rust
// crates/mp/host-interface/src/engine_host.rs — verbatim (rulings 55 + 59a, 20 methods)
pub trait EngineHost {
    #[allow(clippy::too_many_arguments)]
    fn trace(
        &mut self,
        results: &mut trace_t,
        start: &vec3_t,
        mins: &vec3_t,
        maxs: &vec3_t,
        end: &vec3_t,
        pass_entity_num: i32,
        contentmask: i32,
        capsule: bool,
        trace_flags: i32,
        use_lod: i32,
    );
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;
    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}
    fn print(&mut self, msg: &str);
    fn error(&mut self, code: errorParm_t, msg: &str) -> !;
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;
    fn shared_memory(&mut self) -> *mut c_char;
    fn flrand(&mut self, min: f32, max: f32) -> f32;
    fn irand(&mut self, min: i32, max: i32) -> i32;
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
    fn cvar_integer(&mut self, name: &str) -> i32;
    fn sv_time(&mut self) -> i32;
    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;
    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;      // BACKED by this doc
    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;      // BACKED by this doc
    fn cvar_register(&mut self, name: &str, default: &str, flags: i32);
    fn cvar_string(&mut self, name: &str) -> String;
    fn cvar_take_modified(&mut self, name: &str) -> bool;
    fn fs_list_files(&mut self, dir: &str, ext: &str, want_subs: bool) -> Vec<String>;
    fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32>;    // CONSUMED by this doc (PAK checksum, ruling 59a)
}
```

(Doc-comments and `Source:` cites are on each method in the crate file; reproduced
in `ghoul2-server.md` `## Seam definition`. The cvar trio
`cvar_register`/`cvar_string`/`cvar_take_modified` + `fs_list_files` were forced
by StringEd, ruling 55; tr_model consumes only `cvar_register`/`cvar_integer` of
that surface — `Cvar_Get`-then-read, `TRM-D2`. The 20th method
`fs_file_is_in_pak` (`engine_host.rs:209`, ruling 59a) collapses Raven's
`FS_FileIsInPAK` per §C7: `Some(pure_checksum)` = the `==1` found-in-pure-pak
path, `None` = every `-1` path (disk-only, not-found, illegal `..`/`::`,
non-pure-pak-skipped). tr_model is its only consumer — the `iPAKFileCheckSum`
stamp and `DumpNonPure` purity re-check, `TRM-D5`.)

**The two methods this doc BACKS** resolve exactly as Raven
`R_GetModelByHandle(model)->mdxm` / `->mdxa` (`G2_API.cpp:2716-2739`): index
`render_models.models[model]` (out-of-range → the default `models[0]`,
`tr_model.cpp:593-604`), read its `.mdxm`/`.mdxa` raw pointer (a pointer **into**
the owning `CachedEndianedModelBinary.disk_image` `AlignedBytes`), and return it as
`*mut c_void` — **NULL exactly where Raven's `model_t.mdxm`/`.mdxa` is NULL**
(`TRM-D3`(b)). That pointer is the `*mut mdxmHeader_t`/`*mut mdxaHeader_t` cast of
the owning `AlignedBytes` 16-byte-aligned base (`TRM-D4`/ruling 58), so it is
well-aligned; the cast stays `unsafe` confined at this seam (§D11) with a debug
alignment assert at the cast site. This read seam itself is zero-copy — it returns
a pointer **into** the already-owned block, no re-parse (`TRM-D3`(a); the single
ingest copy from the `fs_read_file` `Vec<u8>` into the `AlignedBytes` happens once
at the producer, `TRM-D4`(a), never here); the block is
address-stable and alive while the handle is registered (`TRM-D3`(c)); the mdx header types are
**never named** at this seam — `mp_engine_ghoul2` does its byte arithmetic off the
`c_void` unchanged (`TRM-D3`(d)/`G2SV-D5`). They are answered by the
`EngineHostView` whose `render_models` field is `Some` (the `ghoul2_call` view,
`TRM-D1`(b)); a `None` `render_models` here is a contract-violation panic.

**The pub `mp_renderer` API this doc exposes** (idiomatic §F shapes; internal to
`mp_renderer` + the `Engine` impl — none crosses a `trap_*`/`refexport_t` seam on
the dedicated build, `GetRefAPI` `tr_init.cpp:1472`):

```rust
// mp_renderer — the renderer-models state (TRM-D3, ruling 53). Engine holds it
// directly: Engine.render_models: RenderModels
// This adds the first mp_engine_core -> mp_renderer Cargo edge (acyclic:
// mp_renderer's deps transitively never reach mp_engine_core), and mp_renderer's
// own Cargo.toml gains an mp_host_interface path dep — LICENSED by the ruling-56c
// blanket §F-consumer authorization (TRM-D1(c)): these RenderModels methods take
// `host: &mut impl EngineHost`, EngineHost is frozen in mp_host_interface, and
// mp_host_interface deps only mp_qshared (acyclic). Both edges land with the
// field, same commit. RenderModels is NOT ZeroValid (its Vec/String/BTreeMap
// fields are not all-zero-valid), so Engine::new() writes it in place through
// MaybeUninit — `addr_of_mut!((*p).render_models).write(RenderModels::default())`
// — joining the existing cl/snd/common.modules non-ZeroValid writes
// (engine.rs:87-93, LIFE-Q9). Reached via render_models_call (TRM-D3/ruling 43),
// never a bare &mut.
pub struct RenderModels {
    models: Vec<Box<ModelData>>,                       // tr.models; qhandle_t = index (cap MAX_MOD_KNOWN=1024)
    num_models: i32,                                   // tr.numModels
    hash: /* name→qhandle_t map, §D12 */,               // replaces mhHashTable chains
    cached: BTreeMap<String, CachedEndianedModelBinary>,// CachedModels; sorted-order eviction
    current_level: i32,                                // giRegisterMedia_CurrentLevel
    prev_map_name: String,                             // LevelLoadBegin's sPrevMapName static
    inside_register_model: bool,                       // gbInsideRegisterModel
    num_bsp_models: i32,                               // tr.numBSPModels
}

// AlignedBytes — the model disk-image buffer (TRM-D4, ruling 58). Owns a 16-byte-
// aligned heap block: alloc::alloc with Layout::from_size_align(len, 16), mirroring
// Z_Malloc's alignment guarantee; Drop deallocs the same Layout. Heap-pinned/
// address-stable, in-place mutable for the LL() swap. The 16-byte alignment is what
// makes the *mut mdx*Header_t casts (server_load.rs) sound; those casts stay unsafe-
// confined at the seam (§D11) with a debug alignment assert at the cast site.
pub struct AlignedBytes { /* ptr: NonNull<u8>, len: usize — alloc/dealloc via Layout (TRM-D4) */ }

// CachedEndianedModelBinary (ruling 40 naming: _s-struct → C-suffix dropped; internal).
pub struct CachedEndianedModelBinary {
    disk_image: Option<AlignedBytes>,// pModelDiskImage — 16-byte-aligned, heap-pinned;
                                     // None == NULL (TRM-D4/ruling 58; ruling 52 ownership).
    alloc_size: i32,                 // iAllocSize
    shader_register_data: Vec<(i32, i32)>, // ShaderRegisterData (nameOffset, pokeOffset) — kept (TRM-D3/ruling 54)
    last_level_used_on: i32,         // iLastLevelUsedOn (-1 init)
    pak_file_checksum: i32,          // iPAKFileCheckSum (-1 if not from PAK)
}

impl RenderModels {
    // R_ModelInit / R_SVModelInit (tr_model.cpp:1665,1655) — reserve models[0] as MOD_BAD.
    pub fn model_init(&mut self);
    pub fn model_free(&mut self);                          // R_ModelFree (:1692)
    pub fn hunk_clear(&mut self);                          // R_HunkClearCrap (:1683)
    pub fn get_model(&self, handle: qhandle_t) -> &ModelData; // R_GetModelByHandle (:593) — models[0] fallback
    // RE_RegisterServerModel (:1003) — the sole live model entry; needs host for
    // FS read (RE_RegisterModels_GetDiskFile), print, and the r_noServerGhoul2
    // register-then-read (cvar_register at :1020-1023 + cvar_integer, TRM-D2).
    // Returns mod->index on success (:1142); returns a LITERAL 0 on the fail:
    // label (:1153), NOT mod->index — the MOD_BAD entry stays hashed under its
    // nonzero index while callers get 0 (the G2_API.cpp zero-check for failure,
    // TRM-D3/ruling 53). Do not `return mod.index` unconditionally.
    pub fn register_server_model(&mut self, host: &mut impl EngineHost, name: &str) -> qhandle_t;
    // Cache lifecycle. LevelLoadEnd reads r_modelpoolmegs via cvar_integer (TRM-D2);
    // DumpNonPure/GetDiskFile read sv_pure / FS via host.
    pub fn media_level_load_begin(&mut self, host: &mut impl EngineHost, map_name: &str, force: ForceReload_e);
    pub fn media_get_level(&self) -> i32;                  // RE_RegisterMedia_GetLevel (:568)
    pub fn models_level_load_end(&mut self, host: &mut impl EngineHost, delete_all_unused: bool) -> bool; // (:337)
    pub fn models_info_f(&self, host: &mut impl EngineHost);// RE_RegisterModels_Info_f (:467)
    pub fn modellist_f(&self, host: &mut impl EngineHost);  // R_Modellist_f (:1708)
}
```

Internal (private to `RenderModels`, per §D12): `R_AllocModel` (`:611`),
`RE_InsertModelIntoHash`/`generateHashValue` (`:653,635`),
`RE_RegisterModels_GetDiskFile`/`_ServerModels_Malloc`/`_StoreShaderRequest`
(`:125,253,70`), `RE_RegisterModels_DumpNonPure`/`_DeleteAll`/`GetModelDataAllocSize`
(`:418,496,326`), `ServerLoadMDXA`/`ServerLoadMDXM` (`:683,799`). `ServerLoadMDXM`
recurses `register_server_model` for the `.gla` (`:867`). The header `LL()` swaps
(`:734-739,857-863,905-991`) stay live (identity on LE, `TRM-D3`).

**The `EngineHostView` self-borrow shape** (`TRM-D1`(b), ruling 56b — the residual
ruling 43 pinned the pattern but not the field-that-is-both case; the shape lives
in `mp_engine_core`/`state-ownership.md`, cited here as a settled extern):

```rust
// mp_engine_core (owner of all Engine fields; state-ownership.md, ruling 43/56b)
pub struct EngineHostView<'a> {
    // ... &mut borrows of the Common/Server/CollisionWorld/loader fields the
    //     other 17 EngineHost methods need (ruling 43) ...
    render_models: Option<&'a mut RenderModels>,   // Some in ghoul2_call, None in render_models_call
}
// model_mdxm/model_mdxa: self.render_models.as_mut()
//   .expect("model_* on a view without render_models — contract violation (fork 1)")
//   ... R_GetModelByHandle(model)->mdxm/->mdxa ...
```

**matcomp signatures** (crate `mp_engine_ghoul2`, `TRM-D1`(a) — settled):

```rust
// crates/mp/engine/ghoul2/src/matcomp.rs
pub fn mc_uncompress_quat(mat: &mut [[f32; 4]; 3], comp: &[u8]); // MC_UnCompressQuat (matcomp.c:219) — live
pub fn mc_compress(mat: &[[f32; 4]; 3], comp: &mut [u8]);        // MC_Compress (:50)  — §20 note iff dead at port
pub fn mc_uncompress(mat: &mut [[f32; 4]; 3], comp: &[u8]);      // MC_UnCompress (:163) — §20 note iff dead at port
```

`MC_UnCompressQuat`'s consumer `UnCompressBone` (`tr_ghoul2.cpp:1158`) lives in
`mp_engine_ghoul2` (frozen bone subset); placing matcomp there keeps the frozen
`G2SV-D5` boundary intact — no `mp_engine_ghoul2` → `mp_renderer` edge, no
dependency inversion (matcomp is self-contained pure math; `mp_engine_ghoul2`
already deps only `mp_qshared`).

## Decisions

**TRM-D1** (ruling 56 — closeout, closes `TRM-Q1`/`TRM-Q2` and the systemic
Cargo-edge hole). Three sub-rulings:
- **(a) matcomp lives in `mp_engine_ghoul2`.** Beside its sole live consumer
  (`MC_UnCompressQuat` ← `UnCompressBone`, `tr_ghoul2.cpp:1158`; the codec is
  part of the mdxa format the bone subset decodes). Because `MC_UnCompressQuat`'s
  only caller is the frozen bone subset, which `G2SV-D5` forbids from edging to
  `mp_renderer`; matcomp is pure math with no `mp_renderer` dependency, so it
  moves to the consumer's crate with **no dependency inversion** and the FROZEN
  ghoul2 boundary holds. `MC_Compress`/`MC_UnCompress` get §20 zero-caller notes
  **iff their callers prove dead at port** (the verify-at-port condition: the TU
  links so they compile; `MC_UnCompress` appears only in commented-out
  `tr_ghoul2.cpp:1705-1746` and `MC_Compress` has no `codemp/` caller — confirm
  no live caller emerged before dropping). Rejected `mp_renderer` (matcomp's
  literal `renderer/` origin): would force the forbidden `mp_engine_ghoul2 →
  mp_renderer` edge.
- **(b) The `EngineHostView` self-borrow** (refines ruling 43). The view carries
  `render_models: Option<&mut RenderModels>`: `ghoul2_call()` fills `Some`,
  `render_models_call()` sets `None`. Sound because `RenderModels`' own methods
  **never call `host.model_*`** — they own the registry and read it directly
  (`get_model`, `&self`; the `## State ownership` intro + `## Slice hooks`
  host-service map are the enumerated evidence). A `model_*` call on a `None`
  `render_models` is a contract violation → panic, fatal-bug class per fork 1.
  **One view type**; the invariant is also recorded in `state-ownership.md`.
  Rejected two view flavors / a marker split: one `Option` field is the minimal
  shape and keeps a single `EngineHostView` type.
- **(c) Blanket §F Cargo-edge authorization (ruling 56c).** Every §F consumer
  crate is AUTHORIZED to add the `mp_host_interface` path dependency to its
  `Cargo.toml` as part of its first slice — acyclic, since `mp_host_interface`
  deps only `mp_qshared`. This doc cites 56c where `mp_renderer`'s `Cargo.toml`
  gains the edge (`## Seam definition`, `## Files roster`) instead of
  re-litigating it. Rejected leaving the edge unlicensed per-doc: a porter would
  hit an undeclared dep with no ruling to point to.

**TRM-D2** (ruling 55 — `EngineHost` = **19 methods**, commit `b2855df2`;
ruling 59a later grew it to **20**, `TRM-D5`). The trait is BUILT and frozen in
`mp_host_interface` and **re-quoted verbatim** in `## Seam definition` at its
20-method surface (commit `4c303bd1`). Its ruling-55 growth is the StringEd-forced cvar surface
(`cvar_register`/`cvar_string`/`cvar_take_modified`) + `fs_list_files`. This doc
**implements the `model_mdxm`/`model_mdxa` backing** (via the `EngineHostView`
from `ghoul2_call`, `TRM-D1`(b)) and **consumes** `fs_read_file`/`fs_free_file`
(disk load), `print` (`Com_Printf`/`Com_DPrintf`), and — for its own cvar reads —
**`cvar_register` at init + `cvar_integer` per read**, matching Raven's
`Cvar_Get`-then-read: `r_noServerGhoul2` registers at the lazy `Cvar_Get`
(`tr_model.cpp:1020-1023`), `r_modelpoolmegs`/`sv_pure` read via `cvar_integer`
(missing → 0). Because ruling 55 froze the 19-method surface and the register-
then-read pattern mirrors Raven's cvar idiom exactly. Rejected collapsing the
lazy `Cvar_Get` to a no-op (an earlier draft's choice): ruling 55 makes
registration explicit so a default-establishing `cvar_register` is faithful.

**TRM-D3** (rulings 11-54 stand — carried forward, no re-litigation). The
standing rulings this pass folds in:
- **Ruling 52** — each `CachedEndianedModelBinary` **owns its parsed disk image
  as an `AlignedBytes` buffer** (`TRM-D4`/ruling 58 supersedes the `Box<[u8]>`
  spelling): heap-pinned/address-stable (the frozen ghoul2 seam derefs raw
  pointers into it across frames — `CBoneCache` parent-seeding, skeleton build,
  per-call ragdoll basepose, `tr_ghoul2.cpp:416-421,614-615`), in-place mutable
  for the `LL()` endian swap (`tr_model.cpp:734-739`), drop = `Z_Free`. Eviction
  stays **faithful** (level-keyed `iLastLevelUsedOn` + PAK checksum +
  `r_modelpoolmegs`, `:351-352,436`) — SAFE because ghoul2 re-resolves via
  `R_GetModelByHandle` on each use and never caches a `model_t*` across a level
  change. That invariant is the **memory contract**; the producer
  (`register_server_model` → `RE_RegisterServerModels_Malloc`) guarantees four
  points: **(a)** the live parsed block, **no re-parse** (`:717-731`) — but see
  the ingest-copy reconciliation below: Raven's zero-copy `Z_MorphMallocTag`
  cannot be reproduced over the frozen `fs_read_file` `Vec<u8>`, so `TRM-D4`
  forces exactly one copy of the file bytes into the `AlignedBytes` on first
  registration; **(b)**
  NULL exactly where Raven's `model_t.mdxm`/`.mdxa` is NULL (`G2_API.cpp:2716-2739`);
  **(c)** address-stable and alive while the handle is registered
  (`G2_API.cpp:2716` caches `currentModel`); **(d)** opaque `c_void` at the seam
  — `mp_renderer` names `mdxmHeader_t`/`mdxaHeader_t`, ghoul2 never does
  (`G2SV-D5`). Rejected a re-parsing accessor / a reallocating `Vec`: (a)/(c)
  forbid moving or re-deriving the block under the frozen raw-pointer readers.
  Ruling 52 settled ownership (heap-pinned, in-place mutable, drop = `Z_Free`); the
  alignment-safe access strategy it left open (`TRM-Q4`) is now closed by
  `TRM-D4`/ruling 58 — the `AlignedBytes` 16-byte buffer makes the
  `*mut mdx*Header_t` casts sound (`## Resolved questions`).
- **Ruling 53** — the `CachedModels` map + `tr.models` pool are **fields of
  `RenderModels` in `mp_renderer`** (the crate that already owns
  `model_s`/`trGlobals_t`), a direct `Engine` field. The `tr.models[1024]` Hunk
  pool → `Vec<Box<ModelData>>` with `qhandle_t = index`, and a **map side-index
  replaces the `mhHashTable` intrusive chains** (`tr_model.cpp:35-36`). `MOD_BAD`
  failed-entry retention is faithful (the failed `model_t` is kept and hashed,
  `:1148-1153`). The `mp_engine_core → mp_renderer` Cargo edge lands with the
  field (acyclic); reached by ghoul2 **only** through the `EngineHost` impl —
  `mp_engine_ghoul2` never edges to `mp_renderer` (`G2SV-D5`). Rejected
  re-exposing the ABI `trGlobals_t.models` array as the pool: it is layout-frozen
  for a different purpose; the §F pool is idiomatic (`Vec`) per §F17.
- **Ruling 54** — the header `LL()` endian swaps stay **live** (identity on LE,
  golden-exercised). §20/§C10 per-function with cites (`## Raven ground truth`
  table): the `#ifndef _M_IX86` big-endian blocks (`:751-790,938-983`) are §20
  dead-arm drops; the client shader-poke replay (`#ifndef DEDICATED`, `:221-242`;
  server variant commented out `:293-316`) is §C10 client-dead; the
  `tr_shader`/`tr_image`/`tr_init`/`tr_main`/`tr_mesh`/`null_renderer` draw
  surface (~73 fns) is §20 (`GetRefAPI` exports only `re.Shutdown` under
  `DEDICATED`, `tr_init.cpp:1472-1529`). The `ShaderRegisterData` record vector
  **stays** on the cache struct (server-side `StoreShaderRequest` recording is
  live, `:898`); **only** the poke replay is client-dead. Rejected dropping
  `ShaderRegisterData`: it is written server-side even though never replayed.
- **Ruling 40** — `CachedEndianedModelBinary_s` → `CachedEndianedModelBinary`
  (internal, the `_s` C-suffix dropped; internal types get idiomatic names).
  ABI-frozen types keep Raven names and are **imported, never re-declared**:
  `model_s`/`model_t` is already ported (`crates/mp/renderer/src/tr_local/model_s.rs`)
  — reuse it (the `ModelData` pool entry is that `model_t`); likewise
  `trGlobals_t` and the `mdxm*`/`mdxa*` headers (`src/mdx_format/`). Rejected a
  fresh `ModelData` struct duplicating `model_t`: ruling 40 mandates reuse.
- **Ruling 44** — Win32 `long` inside any binary file format is 4 bytes; here
  matcomp's packing is `unsigned int`/`unsigned short` (`matcomp.c:64` etc.),
  so i32/u16-width, no `c_long` ambiguity.

**TRM-D4** (ruling 58, 2026-07-10 — refines ruling 52, closes `TRM-Q4`; confirms
`TRM-Q3` non-blocking). Two parts:
- **(a) The disk image is an `AlignedBytes` buffer, not `Box<[u8]>`.** Each
  `CachedEndianedModelBinary.disk_image` owns an `AlignedBytes`: `alloc::alloc`
  with `Layout::from_size_align(len, 16)` (mirroring `Z_Malloc`'s alignment
  guarantee), `Drop` deallocs the same `Layout`. Same heap-pinned/address-stable/
  in-place-mutable contract as ruling 52 — the `Box<[u8]>` spelling is superseded
  everywhere. Because the header leading fields are `i32` (4-byte-aligned) but
  Rust's ordinary `Vec<u8>`/`Box<[u8]>` (and the `EngineHost::fs_read_file`
  `Vec<u8>`) carries only 1-byte allocator alignment, an explicit over-aligned
  buffer is required for the in-place field swaps to be defined. **Ingest-copy
  reconciliation (`TRM-D3`(a)):** because `fs_read_file`'s `Vec<u8>` cannot be
  re-aligned in place and cannot be reinterpreted as the aligned buffer, the
  "just loaded" path (`pvDiskBufferIfJustLoaded != NULL`, `tr_model.cpp:271-273`)
  **copies the file bytes once** from the `fs_read_file` `Vec<u8>` into a freshly
  `alloc::alloc`'d `AlignedBytes` — the one place the port cannot reproduce
  Raven's zero-copy `Z_MorphMallocTag` re-tag-in-place morph (which reuses the
  `FS_ReadFile` `Z_Malloc` block directly). This is a §F19 noted divergence, not
  a contradiction of `TRM-D3`(a): the "no re-parse" and address-stable-ownership
  guarantees still hold (the copy is a byte `memcpy`, not a re-parse; the
  `AlignedBytes` is then heap-pinned for the block's whole life), and the
  `bAlreadyCached` flag still suppresses the caller's `FS_FreeFile` exactly as
  Raven's morph does (`:722-731`) — the port simply drops the source `Vec` after
  the copy. The NULL "limb hierarchy creation" path (`Z_Malloc`, `:273`) has no
  source buffer and so no copy (`:277`). The 16-byte
  alignment makes the `*mut mdxmHeader_t`/`*mut mdxaHeader_t` casts
  (`tr_model.cpp:734-739,857-863`) **sound**: they stay `unsafe` confined at the
  seam (§D11) with a debug alignment assert at each cast site. Rejected
  `read_unaligned`/`write_unaligned` field access and a documented reliance on the
  platform allocator's de-facto over-alignment: an explicit 16-byte-aligned buffer
  matches `Z_Malloc` and needs no per-field unaligned dance.
- **(b) `TRM-Q3` is non-blocking and in-campaign.** The `ghoul2_call` accessor's
  shape is pinned by rulings 43/56b (`EngineHostView` + `render_models:
  Option<&mut RenderModels>`); its exact signature lands with the wave-20 packet
  when `Engine` gains the fields — inside the campaign, not a deferral. `TRM-Q3`
  moves to `## Resolved questions`; `TRM-D1`(b)'s soundness argument never depended
  on the accessor's second tuple element type.

**TRM-D5** (ruling 59, 2026-07-10, delegated — closes `TRM-Q5` (a) and folds two
mechanical corrections: the `RE_RegisterMedia_LevelLoadEnd` misclassification (b)
and the `§A1`→§1 citation rewrite (c)). Three sub-rulings:
- **(a) `EngineHost` gained a 20th method, `fs_file_is_in_pak`** (commit
  `4c303bd1`, `engine_host.rs:209`): `fn fs_file_is_in_pak(&mut self, qpath:
  &str) -> Option<i32>`. Raven's `FS_FileIsInPAK` (`files.cpp:1602-1659`) is an
  `int` that returns `1` or `-1` and **never `0`**; it is collapsed per
  porting-rules §C7 to `Some(pak->pure_checksum)` — the `==1` path, reached only
  when the file is found in a **pure-allowed** pak (a non-pure pak is skipped by
  the `FS_PakIsPure` `continue` at `:1640-1642` and falls through to the trailing
  `return -1`) — and `None` for every `-1` path: disk-only, not-found, and the
  `..`/`::` illegal-path early-out (`:1623`). Both live consumers are **binary**
  on it: `RE_RegisterServerModels_Malloc`'s `== 1` `iPAKFileCheckSum` stamp
  (`tr_model.cpp:212,284`) becomes "stamp on `Some`", and `DumpNonPure`'s
  `iInPak == -1 || iCheckSum != CachedModel.iPAKFileCheckSum` dump check
  (`tr_model.cpp:434-436`, whose `iCheckSum = -1` sentinel `:433` collapses
  naturally under `Option`) becomes "dump on `None` or checksum-mismatch". The
  trait is **re-quoted verbatim at 20 methods** in `## Seam definition`;
  `MockHost`'s `pak_files: BTreeMap<String, i32>` fixture map
  (`mock.rs:148,427-430`) drives the goldens (`Some(checksum)` for a mapped path,
  `None` otherwise). This **closes `TRM-Q5`** — the frozen seam now exposes the
  PAK-checksum method the first-slice `cached_model_binary.rs` needed. Rejected a
  `(i32, i32)` `(in_pak, checksum)` tuple (the DRAFT's own straw-man): the
  `-1`/`1` return has no `0`, so `Option<i32>` is the faithful §C7 collapse.
- **(b) `RE_RegisterMedia_LevelLoadEnd` is §20-DROPPED** (`tr_model.cpp:577`). Its
  **sole caller is the client** `cl_cgame.cpp:1942`; it has **zero dedicated
  callers** (the doc's own live-chain ground truth already omits it — the live
  eviction is reached directly at `z_memman_pc.cpp:226` →
  `RE_RegisterModels_LevelLoadEnd(qtrue)`, `tr_model.cpp:337`, and its
  media/image/sound tail is `#ifndef DEDICATED`, `:579-584`). It is removed from
  the `cached_model_binary.rs` roster grouping and the method-transcription row
  and given a §20 zero-caller note; **the live eviction path is
  `models_level_load_end`** (backing `RE_RegisterModels_LevelLoadEnd`, `:337`).
  Rejected keeping it as a §C10-folded tail: with no dedicated caller the whole
  function is unreachable, i.e. §20, not a compiled-out arm of a live function.
- **(c) MECHANICAL — `§A1` citations become porting-rules §1.** Any `§A1`
  shorthand resolves to porting-rules §1 ("behavioral parity at the seam;
  internals are free, the seam is not" — section A rule 1; the rules are numbered
  globally 1-21). This draft carries **no** `§A1` citations (verified), so the
  rewrite is a no-op here; the sub-ruling is recorded for provenance.

No decision here re-litigates a standing ruling; verification (§F18/DEC-09) is in
`## Verification strategy`.

## Files roster

Machine-readable file plan for `port-cpp-subsystem`'s `designPath` (rule 6). All
`mode: mp`; `crate: mp_renderer` **except** `matcomp` (`crate:
mp_engine_ghoul2`, `TRM-D1`(a)). Sharding follows §F21 (one logical unit per
file; the free-function API groups split by concern). The DEDICATED-dead TUs
(`tr_shader`/`tr_image`/`tr_init`/`tr_main`/`tr_mesh`/`null_renderer`) get **no**
`files:` entry — they are §20-dropped in the dedicated slice (`divergences`).

```yaml
files:
  - path: crates/mp/renderer/src/tr_model/render_models.rs
    crate: mp_renderer
    mode: mp
    class: RenderModels
    summary: The Engine.render_models direct field (TRM-D3/ruling 53) — fields models (Vec<Box<ModelData>>, qhandle_t=index, MAX_MOD_KNOWN=1024 cap; ModelData = the already-ported model_t per ruling 40, Box-pinned for address-stable model_t* per TRM-D3/ruling 52), num_models, hash (name→qhandle_t, §D12 container — replaces mhHashTable chains), cached (BTreeMap<String,CachedEndianedModelBinary>, sorted eviction order), current_level, prev_map_name, inside_register_model, num_bsp_models. Not ZeroValid (Vec/String/BTreeMap) so Engine::new MaybeUninit-writes RenderModels::default() joining cl/snd/modules (engine.rs:87-93, LIFE-Q9); the field adds the first mp_engine_core->mp_renderer Cargo edge (acyclic, ruling 53), and mp_renderer's own Cargo.toml gains an mp_host_interface path dep LICENSED by ruling 56c (TRM-D1(c), the blanket §F-consumer authorization; forced by the host: &mut impl EngineHost seam; acyclic since mp_host_interface deps only mp_qshared). Its (&mut self, host: &mut impl EngineHost) methods are reached via the ruling-43 render_models_call split-borrow accessor, never a bare &mut (TRM-D3/TRM-D1(b)). Methods: R_AllocModel/R_GetModelByHandle/RE_InsertModelIntoHash/generateHashValue (tr_model.cpp:611,593,653,635), R_ModelInit/R_SVModelInit/R_ModelFree/R_HunkClearCrap (:1665,1655,1692,1683 — reserve models[0] as MOD_BAD; KillTheShaderHashTable is a §20 tr_shader cross-ref, host-free), R_Modellist_f (:1708). The model_mdxm/model_mdxa EngineHost backing (G2SV-D5, TRM-D1(b)) resolves through get_model here. One type per file (CLAUDE.md).
  - path: crates/mp/renderer/src/tr_model/aligned_bytes.rs
    crate: mp_renderer
    mode: mp
    class: AlignedBytes
    summary: The model disk-image buffer type (TRM-D4/ruling 58) — owns a 16-byte-aligned heap block (alloc::alloc with Layout::from_size_align(len,16), mirroring Z_Malloc's alignment; Drop deallocs the same Layout), heap-pinned/address-stable, in-place mutable for the LL() endian swap. Its 16-byte alignment is what makes the *mut mdxmHeader_t/*mut mdxaHeader_t casts in server_load.rs (and the model_mdxm/model_mdxa seam deref) sound (§D11 unsafe-confined + debug alignment assert at each cast site). Consumed by CachedEndianedModelBinary.disk_image (as Option<AlignedBytes>), server_load.rs (the cast/swap sites), and the model_mdxm/model_mdxa backing in render_models.rs — its own file per one-type-per-file (CLAUDE.md), not a Raven class (no oracle cite; idiomatic infra forced by ruling 58 over the 1-byte-aligned fs_read_file Vec<u8>). Constructed from a &[u8] by copying the bytes into the aligned block (the single ingest copy, TRM-D4(a)/§F19 — Raven's zero-copy Z_MorphMallocTag morph is not reproducible over the 1-byte-aligned Vec); also a zero-fill ctor for the NULL limb-hierarchy Z_Malloc path (tr_model.cpp:277).
  - path: crates/mp/renderer/src/tr_model/cached_model_binary.rs
    crate: mp_renderer
    mode: mp
    class: CachedEndianedModelBinary
    summary: The CachedEndianedModelBinary cache entry (TRM-D3/rulings 52,40,58; _s-suffix dropped per ruling 40) — disk_image Option<AlignedBytes> (16-byte-aligned alloc::alloc + Layout::from_size_align(len,16), heap-pinned, in-place LL() swap, drop deallocs the same Layout; None==NULL; TRM-D4/ruling 58, superseding ruling 52's Box<[u8]>), alloc_size, shader_register_data Vec<(i32,i32)> (kept, ruling 54), last_level_used_on(-1), pak_file_checksum(-1). Plus the cache free-fns operating over RenderModels.cached: RE_RegisterModels_GetDiskFile (:125, FS via host; the sDEFAULT_GLA_NAME FakeGLAFile intercept :95-152), RE_RegisterServerModels_Malloc (:253, one ingest copy of the fs_read_file Vec into a fresh AlignedBytes on the just-loaded path :271-273 — Raven's zero-copy Z_MorphMallocTag is not reproducible over the 1-byte-aligned Vec, §F19/TRM-D4(a); Z_Malloc NULL path :277; the iPAKFileCheckSum stamp via fs_file_is_in_pak :212,284 stamps on Some, TRM-D5/ruling 59a — the 20th EngineHost method now backs it), RE_RegisterModels_StoreShaderRequest (:70, live server-side :898), RE_RegisterModels_LevelLoadEnd (:337, r_modelpoolmegs via cvar_integer, TRM-D2 — the live eviction path, Rust models_level_load_end), RE_RegisterModels_DumpNonPure (:418, sv_pure via cvar_integer; never dumps *default.gla; fs_file_is_in_pak-gated — dump on None||checksum-mismatch :434-436, TRM-D5), RE_RegisterModels_DeleteAll (:496), GetModelDataAllocSize (:326 — sum cached[*].alloc_size locally over RenderModels.cached, NOT a Zone query; byte-exact on the dedicated build per TRM-D3/ruling 54 since every other TAG_MODEL_* producer is §20-dropped/frozen-dead), RE_RegisterMedia_LevelLoadBegin/GetLevel (:522,568 — LevelLoadBegin's #ifndef DEDICATED R_Images_DeleteLightMaps tail §C10-folded; RE_RegisterMedia_LevelLoadEnd :577 is §20-DROPPED per TRM-D5/ruling 59b — client-only, sole caller cl_cgame.cpp:1942, zero dedicated callers). The FakeGLAFile[] const (:95-116) and gbInsideRegisterModel re-entrancy guard (:1406) colocate. RE_RegisterModels_Malloc (client, :179) + its #ifndef DEDICATED poke replay is §20/§C10 (divergences), not ported.
  - path: crates/mp/renderer/src/tr_model/server_load.rs
    crate: mp_renderer
    mode: mp
    class: ServerLoad
    summary: The sole live dedicated model entry — RE_RegisterServerModel (tr_model.cpp:1003; hash lookup, LOD loop, ident dispatch MDXA/MDXM, default→fail, MOD_BAD retention, r_noServerGhoul2 lazy Cvar_Get :1020-1023 → cvar_register + cvar_integer read per TRM-D2), ServerLoadMDXA (:683), ServerLoadMDXM (:799; recurses register_server_model for the .gla :867). The header LL() swaps stay live (TRM-D3/ruling 54, identity on LE); the surface-hierarchy walk + LOD/surface field swaps + SHADER_MAX_VERTEXES/INDEXES bounds checks + ident=SF_MDX run on intel too (:904); the #ifndef _M_IX86 skeletal/vertex swaps (:751-790,938-983) are §20-dropped. The buffer→*mut mdxaHeader_t/*mut mdxmHeader_t casts (tr_model.cpp:734-739,857-863) and every in-place header/surface/LOD field read+swap operate on the 16-byte-aligned AlignedBytes base (TRM-D4/ruling 58), so they are sound: keep the casts unsafe-confined at the seam (§D11) with a debug alignment assert at each cast site — transcribe them directly, do NOT invent an alignment strategy (Vec into_boxed_slice, read_unaligned, etc.). Host: fs_read_file/fs_free_file (GetDiskFile), print, cvar_register/cvar_integer.
  - path: crates/mp/engine/ghoul2/src/matcomp.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: matcomp
    summary: MC_UnCompressQuat/MC_Compress/MC_UnCompress (matcomp.c:219,50,163) — pure float[3][4] bit-packing, no globals. Home is mp_engine_ghoul2 per TRM-D1(a)/ruling 56a — beside the sole live consumer MC_UnCompressQuat <- UnCompressBone (tr_ghoul2.cpp:1158-1162, FROZEN bone subset); no dependency inversion (matcomp is self-contained, the crate already deps only mp_qshared), G2SV-D5 crate boundary intact. MC_UnCompressQuat ports (live). MC_Compress/MC_UnCompress get §20 zero-caller notes IFF verified dead at port (MC_UnCompress only in commented-out :1705-1746, MC_Compress no codemp/ caller — the TU links so they compile; confirm no live caller before dropping). Ruling 44 governs Win32-long width (here u32/u16 packing).

divergences:
  - "_M_IX86 defined (x86 WinDed target, TRM-D3/ruling 54): every #ifndef _M_IX86 big-endian block compiles out — the ServerLoadMDXA skeletal/frame swaps (tr_model.cpp:751-790), the ServerLoadMDXM bone-ref/triangle/vertex swaps (:938-983), and the R_LoadMD3 frame/tag/tri/st/xyz swaps (:1504-1530,1584-1614). §20 dead-arm drops; the port is LE-correct because LittleLong/Short/Float are identity on LE."
  - "Client shader-poke replay is client-dead (TRM-D3/ruling 54): RE_RegisterModels_Malloc's #ifndef DEDICATED loop (tr_model.cpp:221-242) that R_FindShader-resolves and pokes sh->index is §C10-folded; the server RE_RegisterServerModels_Malloc variant already has it commented out ('No. Bad.', :293-316). The ShaderRegisterData record vector STAYS on the cache struct — server-side StoreShaderRequest recording is live (:898)."
  - "Client model path §20-dropped in the dedicated slice (TRM-D3/ruling 54; belongs to the deferred client renderer, DEC-01): RE_RegisterModel/_Actual (tr_model.cpp:1407,1169), R_LoadMD3 (:1427), RE_BeginRegistration (:1629, whole body #ifndef DEDICATED), R_LerpTag (:1768), R_ModelBounds (:1811), R_GetTag (:1744) have no live dedicated caller — GetRefAPI exports only re.Shutdown under DEDICATED (tr_init.cpp:1472-1529), and the G2_API.cpp:589,2619,2710 else-arm is dead when the server registers."
  - "RE_RegisterMedia_LevelLoadEnd §20-dropped (TRM-D5/ruling 59b): tr_model.cpp:577's sole caller is the client cl_cgame.cpp:1942, zero dedicated callers — its live body reduces to RE_RegisterModels_LevelLoadEnd(qfalse) with a #ifndef DEDICATED media/image/sound tail (:579-584), but the dedicated server reaches that eviction directly at z_memman_pc.cpp:226 → RE_RegisterModels_LevelLoadEnd (:337, Rust models_level_load_end). Whole fn unreachable on the dedicated build, so §20 not §C10. Only RE_RegisterMedia_LevelLoadBegin/GetLevel (:522,568) port from that trio."
  - "tr_shader/tr_image/tr_init/tr_main/tr_mesh/null_renderer draw surface §20-dropped (~73 fns, TRM-D3/ruling 54): the client GL/shader-compile/image-upload/scene surface the DEDICATED refexport_t excludes. Live cross-refs kept: KillTheShaderHashTable (tr_shader, called by R_HunkClearCrap :1685), RE_Shutdown (tr_init:1333, sole DEDICATED refexport entry, only Cmd_RemoveCommand survives, touches no model state — §C10). The loader cvars r_modelpoolmegs/r_noServerGhoul2 Cvar_Get (tr_init:1159,1180) map to cvar_register/cvar_integer (TRM-D2)."
  - "Loader cvars are host register-then-reads, not renderer state (TRM-D2, ruling 55): the lazy Cvar_Get('r_noserverghoul2','0',0) at RE_RegisterServerModel:1020-1023 becomes EngineHost::cvar_register (establishes the default once), and reads go through cvar_integer (missing→0, matching Cvar_VariableIntegerValue); sv_pure/r_modelpoolmegs read via cvar_integer. R_Register (tr_init.cpp:985) is dead (reached only through the #ifndef DEDICATED R_Init→RE_BeginRegistration path). Values injected via MockHost for the eviction goldens."
  - "CachedModels is a BTreeMap<String,_> not HashMap (TRM-D3 parity consequence): the eviction loops (RE_RegisterModels_LevelLoadEnd :355, DumpNonPure :426) and RE_RegisterModels_Info_f (:478) iterate std::map in sorted key order and erase in place; BTreeMap keeps that order (same precedent as stringed.md ruling 50 and ghoul2-server.md GoreRecords). Key = lowercased name (Q_strlwr)."
  - "mhHashTable intrusive chains → a name→qhandle_t map (TRM-D3/ruling 53): lookup-only (Q_stricmp exact name match, one handle per name), no ordered iteration, so container choice is §D12 latitude (HashMap suffices, unlike the ordered CachedModels). generateHashValue (:635) is not reproduced — the map subsumes the bucket."
  - "CachedEndianedModelBinary owns pModelDiskImage as a 16-byte-aligned AlignedBytes buffer (TRM-D4/ruling 58, superseding ruling 52's Box<[u8]>): alloc::alloc + Layout::from_size_align(len,16) mirroring Z_Malloc, Drop deallocs the same Layout; heap-pinned/address-stable (the frozen ghoul2 seam derefs raw pointers into it across frames), in-place mutable for the LL() endian swap; None mirrors pModelDiskImage==NULL. The *mut mdx*Header_t casts (tr_model.cpp:734-739,857-863) are sound off the 16-byte base, unsafe-confined at the seam (§D11) with a debug alignment assert at cast sites. model_mdxm/model_mdxa return pointers INTO it as *mut c_void, never naming the mdx header types (G2SV-D5)."
  - "One ingest copy replaces Raven's zero-copy morph (TRM-D4(a)/TRM-D3(a), §F19): Raven's RE_RegisterServerModels_Malloc Z_MorphMallocTag-re-tags the FS_ReadFile Z_Malloc block in place with no copy (tr_model.cpp:271-273). The port cannot — EngineHost::fs_read_file returns a 1-byte-aligned Vec<u8> that cannot be re-aligned in place, and ruling 58 mandates a 16-byte AlignedBytes — so the just-loaded path copies the file bytes once into a fresh AlignedBytes; the NULL limb-hierarchy path (:277) has no source and no copy. TRM-D3(a)'s no-re-parse + address-stable-ownership guarantees still hold (a memcpy is not a re-parse; the AlignedBytes is heap-pinned for the block's life) and bAlreadyCached still suppresses the caller FS_FreeFile (:722-731). The read seam (model_mdxm/model_mdxa) stays zero-copy."
  - "GetModelDataAllocSize is a local sum, not a Zone query (TRM-D3/ruling 54 consequence): the Rust GetModelDataAllocSize returns Σ cached[*].alloc_size over RenderModels.cached instead of Z_MemSize(TAG_MODEL_MD3)+GLM+GLA (tr_model.cpp:326-331). Byte-exact on the dedicated build because Z_MemSize sums each live allocation's requested iSize (z_memman_pc.cpp:292,338,446) and every non-server TAG_MODEL_* producer — R_LoadMD3 (:1467) and the client R_LoadMDXM/R_LoadMDXA (tr_ghoul2.cpp:4859,5311) — is §20-dropped/frozen-dead, leaving only the server disk_image buffers (each recorded as iAllocSize, :718,841). Needs no Zone-allocator seam and no new EngineHost method (the real Zone allocator is type-only today)."
  - "matcomp home = mp_engine_ghoul2 (TRM-D1(a), ruling 56a): MC_UnCompressQuat is consumed by the FROZEN ghoul2 bone subset (UnCompressBone tr_ghoul2.cpp:1158), which cannot edge to mp_renderer (G2SV-D5); matcomp is pure math and moves to the consumer's crate — no dependency inversion, boundary intact. MC_Compress/MC_UnCompress §20-noted iff verified dead at port."
  - "EngineHostView carries render_models: Option<&mut RenderModels> (TRM-D1(b), ruling 56b): ghoul2_call fills Some, render_models_call sets None; a model_* call on None is a contract-violation panic (fatal-bug class, fork 1). Sound because no RenderModels method calls host.model_* (they own the registry, read via get_model &self). One view type; invariant recorded in state-ownership.md."
```

## Method transcription table

Anchors for the non-obvious transcription targets. Each row is one target.

| Raven symbol | oracle cite | Rust file | notes |
|---|---|---|---|
| `R_ModelInit` / `R_SVModelInit` | `tr_model.cpp:1665,1655` | `render_models.rs` | `new CachedModels_t`; reserve `models[0]` as `MOD_BAD`; `R_SVModelInit` is the live dedicated init (`sv_init.cpp:578`) |
| `R_GetModelByHandle` | `tr_model.cpp:593` | `render_models.rs` | out-of-range → `models[0]`; **the `model_mdxm`/`model_mdxa` backing** (`G2SV-D5`, `TRM-D1`(b)) resolves through it |
| `R_AllocModel` | `tr_model.cpp:611` | `render_models.rs` | `None` at the `MAX_MOD_KNOWN=1024` cap; `->index = num_models` |
| `RE_InsertModelIntoHash` / `generateHashValue` | `tr_model.cpp:653,635` | `render_models.rs` | replaced by the name→handle map (`TRM-D3`/ruling 53); `generateHashValue` not reproduced |
| `R_HunkClearCrap` | `tr_model.cpp:1683` | `render_models.rs` | zero `num_models`/`hash`; `KillTheShaderHashTable` is a §20 `tr_shader` cross-ref |
| `RE_RegisterServerModel` | `tr_model.cpp:1003` | `server_load.rs` | the **sole live model entry**; LOD loop, ident dispatch, `MOD_BAD` retention; `r_noServerGhoul2` lazy `Cvar_Get` → `cvar_register` + `cvar_integer` read (`TRM-D2`). **Returns `mod->index` on success (`:1142`); the `fail:` label returns a literal `0`, NOT `mod->index` (`:1153`)** — a bad-ident entry stays hashed under a nonzero index but hands callers `0` |
| `ServerLoadMDXA` / `ServerLoadMDXM` | `tr_model.cpp:683,799` | `server_load.rs` | header `LL()` swaps live (`TRM-D3`); `MDXM` recurses `register_server_model` for the `.gla` (`:867`); `#ifndef _M_IX86` swaps dropped |
| `RE_RegisterModels_GetDiskFile` | `tr_model.cpp:125` | `cached_model_binary.rs` | cache hit vs `FS_ReadFile` (host); the `sDEFAULT_GLA_NAME` `FakeGLAFile` intercept (`:95-152`) |
| `RE_RegisterServerModels_Malloc` | `tr_model.cpp:253` | `cached_model_binary.rs` | morph/`Z_Malloc` → `AlignedBytes` (16-byte, `TRM-D4`/ruling 58); `iPAKFileCheckSum` stamp via `fs_file_is_in_pak` (`:212,284`; stamp on `Some`, `TRM-D5`/ruling 59a); **poke replay commented out** |
| `RE_RegisterModels_StoreShaderRequest` | `tr_model.cpp:70` | `cached_model_binary.rs` | live server-side (`ServerLoadMDXM:898`); records offsets, replay is client-dead (`TRM-D3`) |
| `RE_RegisterModels_LevelLoadEnd` | `tr_model.cpp:337` | `cached_model_binary.rs` | level-keyed + `r_modelpoolmegs` (`cvar_integer`) eviction; `gbInsideRegisterModel` guard; returns "freed ≥1" |
| `GetModelDataAllocSize` | `tr_model.cpp:326-331` | `cached_model_binary.rs` | **local sum** `Σ cached[*].alloc_size` over `RenderModels.cached`, **not** a Zone-allocator query (`TRM-D3`, ruling 54 consequence) — byte-exact on the dedicated build because every non-server `TAG_MODEL_*` producer (`R_LoadMD3` `:1467`, `R_LoadMDXM`/`R_LoadMDXA` `tr_ghoul2.cpp:4859,5311`) is §20-dropped/frozen-dead and `Z_MemSize` sums the same `iSize`s (`z_memman_pc.cpp:292,338,446`). No new host seam |
| `RE_RegisterModels_DumpNonPure` | `tr_model.cpp:418` | `cached_model_binary.rs` | `sv_pure` gate (`cvar_integer`); dump on `None`‖checksum-mismatch via `fs_file_is_in_pak` (`:434-436`, `TRM-D5`/ruling 59a); never `*default.gla` (`:438`) |
| `RE_RegisterMedia_LevelLoadBegin` / `GetLevel` | `tr_model.cpp:522,568` | `cached_model_binary.rs` | `sPrevMapName` level bump; `LevelLoadBegin`'s `#ifndef DEDICATED` `R_Images_DeleteLightMaps` tail §C10-folded |
| `RE_RegisterMedia_LevelLoadEnd` | `tr_model.cpp:577` | — (§20 drop) | client-only; sole caller `cl_cgame.cpp:1942`, zero dedicated callers (`TRM-D5`/ruling 59b); the live eviction path is `models_level_load_end` (`RE_RegisterModels_LevelLoadEnd` `:337`) |
| `MC_UnCompressQuat` / `MC_Compress` / `MC_UnCompress` | `matcomp.c:219,50,163` | `matcomp.rs` (`mp_engine_ghoul2`) | pure math; only `UnCompressQuat` live (`tr_ghoul2.cpp:1158`); home settled (`TRM-D1`(a)); `Compress`/`UnCompress` §20-noted iff dead at port |
| `RE_RegisterModel` (client) + subtree | `tr_model.cpp:1407,1169,1427,1629,1768,1811` | — (§20 drop) | client-only, no live dedicated caller (`divergences`); deferred client renderer (DEC-01) |

## Verification strategy

Governing clause: `porting-rules.md` §F18 (differential goldens), DEC-09
(oracle-differential parity). Harness `tools/trmodel-oracle/` copies the GP2 /
`ghoul2-oracle` pattern: `run.sh` compiles the **unmodified**
`codemp/renderer/tr_model.cpp` + `matcomp.c` against stub headers with
`-DDEDICATED`; `main.cpp` dumps canonical behavior over committed fixtures;
goldens under `golden/` so `cargo test` needs no C++ toolchain; Rust parity tests
(in `mp_renderer` for the cache/pool/pipeline, in `mp_engine_ghoul2` for matcomp)
reproduce the dump byte-for-byte, with cvar/FS values injected via the
fixture-backed `MockHost` (`crates/mp/host-interface/src/mock.rs`, whose ruling-55
cvar registry is the string source of truth for `cvar_register`/`cvar_integer`).

Fixtures (**no retail blobs**, ruling 14 — hand-authored minimal or `ibi-gen`-style
generated `.glm`/`.gla` bytes; the 294-byte `FakeGLAFile` is program-internal and
committable, `tr_model.cpp:95-116`):
- **Header-parse + endian goldens** — `ServerLoadMDXA`/`ServerLoadMDXM` over a
  minimal `.gla`/`.glm`: dump the parsed `mdxaHeader_t`/`mdxmHeader_t` fields, the
  `animIndex` recursion result, `mod->numLods`/`dataSize`, and the version-reject
  path. The `LL()` swaps are identity on LE (`TRM-D3`), so parity is the byte
  layout.
- **Cache goldens** — hit/miss/eviction: register a model twice (`pqbAlreadyFound`
  flips), then drive `RE_RegisterModels_LevelLoadEnd` across a level-bump
  sequence with an injected `r_modelpoolmegs` (via `MockHost` cvar registry)
  forcing eviction; dump the surviving `CachedModels` keys **in sorted order**
  (the BTreeMap/`std::map` parity point). `DumpNonPure` over an injected `sv_pure`
  + PAK-checksum fixture (`*default.gla` never dumped) — the PAK-checksum delivery
  uses `fs_file_is_in_pak` (`TRM-D5`/ruling 59a, the 20th `EngineHost` method),
  and `MockHost`'s `pak_files: BTreeMap<String, i32>` fixture map
  (`mock.rs:148,427-430`) injects it: `Some(checksum)` for a mapped path (the
  `==1` in-pure-pak stamp), `None` otherwise (the `-1` disk-only/not-found/
  non-pure paths). This golden runs (no longer gated).
- **`MC_UnCompressQuat` goldens** — the quantized-quaternion → matrix table over
  a spread of packed inputs (the sole live matcomp path); `MC_Compress`/
  `MC_UnCompress` round-trip goldens for completeness (they link even without a
  live caller; the §20-note decision is verify-at-port, `TRM-D1`(a)).
- **Handle/pool goldens** — `R_AllocModel`/`R_GetModelByHandle` including the
  out-of-range → `models[0]` fallback and the `MOD_BAD` failed-entry retention
  (`register_server_model` on a bad ident still hashes the entry).
- **Seam goldens** — `model_mdxm`/`model_mdxa(handle)` over registered/unregistered
  handles: NULL exactly where `model_t.mdxm`/`.mdxa` is NULL (`TRM-D3`(b)); the
  returned pointer addresses **into** the `disk_image` `AlignedBytes` (`TRM-D3`,
  `TRM-D4`); and
  the `render_models`-`None` view panics (a `#[should_panic]` unit test on the
  contract violation, `TRM-D1`(b)).
- UB inputs (the `assert(0)` in `StoreShaderRequest` on a missing block `:83`,
  the `assert(bAlreadyCached == bAlreadyFound)` `:720`) are kept out of shared
  fixtures or normalized in the dumper with a comment (§F19).

## Slice hooks

- **M3 renderer waves** (`GOAL-engine.md`, ruling 51) — this doc **gates** them:
  the frozen ghoul2 bone subsystem (`ghoul2-server.md`) reads model memory only
  through `model_mdxm`/`model_mdxa`, whose backing is this doc's `RenderModels` +
  `CachedEndianedModelBinary`. Needs frozen first: the already-ported `model_t` /
  `trGlobals_t` / `mdx_format/` types (done) and the BUILT `EngineHost` trait
  (rulings 55 + 59a, 20 methods, commit `4c303bd1`, done — the 20th
  `fs_file_is_in_pak` backs this doc's PAK-checksum sites, `TRM-D5`). The §F
  signatures freeze here; goldens run against `MockHost`.
  **Engine integration mechanics** (`TRM-D3`/`TRM-D1`): the `mp_engine_core →
  mp_renderer` Cargo edge (ruling 53) and the `mp_renderer → mp_host_interface`
  edge (ruling 56c, `TRM-D1`(c)) and the `Engine::new` `MaybeUninit`
  `RenderModels::default()` write land **with** the `Engine.render_models` field;
  the `render_models_call` split-borrow accessor and the `EngineHostView`
  `render_models: Option<&mut RenderModels>` self-borrow (`TRM-D1`(b)) + the
  `EngineHost`-over-`Engine` impl that backs `model_mdxm`/`model_mdxa` are
  **wave-20** work (ruling 43's "impl at wave 20"). Porting `RenderModels`'s state
  + methods against `MockHost` is not gated on the wave-20 view work.
  **First slice (`TRM-D4`/ruling 58):** the header-field-touching
  interior of `ServerLoadMDXA`/`ServerLoadMDXM` (buffer→`mdx*Header_t` cast +
  in-place `LL()` swaps + surface/LOD field swaps) and the `model_mdxm`/`model_mdxa`
  seam deref were the one prior blocker (`TRM-Q4`); ruling 58 settles the
  `AlignedBytes` 16-byte cast strategy, so they port **now** alongside the
  registry/pool/hash state, the cache lifecycle/eviction (`GetModelDataAllocSize`
  is a local `alloc_size` sum, no Zone seam — `## Raven ground truth`), and the
  `RE_RegisterServerModel` control skeleton (all against `MockHost`). **The PAK-
  checksum parts port too now (`TRM-D5`/ruling 59a closed the prior residual gate
  `TRM-Q5`):** the `iPAKFileCheckSum` stamp in `RE_RegisterServerModels_Malloc`
  (`tr_model.cpp:212,284`) and all of `RE_RegisterModels_DumpNonPure` (`:434-436`)
  bind the 20th `EngineHost` method `fs_file_is_in_pak` (frozen at commit
  `4c303bd1`), and `MockHost`'s `pak_files` fixture delivers the checksum — so the
  **whole first slice** (registration, LOD dispatch, header/surface swaps, the
  pool/hash, the full eviction incl. `DumpNonPure`, and the PAK stamp) ports now
  with no residual gate.
- **Cross-doc unblock:** `ghoul2-server.md`'s `render/bone_transform.rs` /
  `render/bone_cache.rs` transcription needs `MC_UnCompressQuat` reachable. With
  `TRM-D1`(a) settling matcomp's home as `mp_engine_ghoul2` (the bone subset's own
  crate), that reachability is **no longer blocked** — `MC_UnCompressQuat` is a
  local call, `G2SV-D5` intact. matcomp lands with (or before) the bone-eval
  porters.
- **Host-service map:** `server_load.rs` and `cached_model_binary.rs` bind
  `fs_read_file`/`fs_free_file` (disk load), `print` (`Com_Printf`/`Com_DPrintf`),
  and `cvar_register`/`cvar_integer` (`r_modelpoolmegs`/`sv_pure`/`r_noServerGhoul2`
  register-then-read, `TRM-D2`). `render_models.rs` and `matcomp.rs` are host-free
  (pure state/math), except `models_info_f`/`modellist_f` which `print`. **None of
  these methods binds `model_mdxm`/`model_mdxa`** — the enumerated evidence for
  the `TRM-D1`(b) `render_models: None` soundness on the `render_models_call` view.

## Resolved questions

Every hole raised across passes 1-3 is now closed; `## Open questions` is empty.
The two pass-2 dry-run holes are closed by `TRM-D4` (ruling 58, 2026-07-10);
`TRM-Q5` (the FS PAK-membership/checksum seam, the last open hole) is closed by
`TRM-D5` (ruling 59a, 2026-07-10 — the 20th `EngineHost` method). (`TRM-Q1`
matcomp's crate home and `TRM-Q2` the `render_models` self-referential borrow were
already closed by `TRM-D1` — ruling 56 — and the systemic Cargo-edge omission by
its blanket §F-consumer authorization, `TRM-D1`(c)/ruling 56c.)

- **`TRM-Q3` — the `ghoul2_call` accessor's formal Seam signature.**
  **RESOLVED by `TRM-D4`(b) (ruling 58): non-blocking and in-campaign.** Ruling 56b
  **names** `ghoul2_call()` as the `EngineHostView` constructor that fills
  `render_models: Some` (`engine-fork-discovery.md:594-595`), and ruling 43 fixes
  the `Engine::<x>_call(&mut self) -> (EngineHostView<'_>, &mut <State>)`
  split-borrow **pattern** (this doc's own `render_models_call` is the worked
  instance, `## State ownership`). The accessor's **exact** signature is not yet
  spelled in a doc — the sibling FROZEN `ghoul2-server.md` never records it (unlike
  `npcnav.md`'s `Engine::nav_call` and `icarus.md`'s `Engine::icarus_call`), and
  `state-ownership.md`'s STATE-Q2 gives only the generic pattern with `nav_call` as
  the example. Ruling 58 confirms this is **not a deferral**: the shape is pinned
  (43/56b), and the exact signature lands with the wave-20 packet when `Engine`
  gains the fields. This doc never depended on it — `TRM-D1`(b)'s soundness argument
  uses only the settled name plus the `Some`-filling, not the accessor's second
  tuple element type (which names ghoul2's own state struct, `ghoul2-server.md`'s
  territory; the eventual record is a dated `ghoul2-server.md` amendment at its
  wave-20 surface). Not blocking this doc's first slice: porting `RenderModels`'
  state + methods against `MockHost` never touches it.
- **`TRM-Q4` — alignment of the `disk_image` buffer for header-struct
  reinterpretation.** **RESOLVED by `TRM-D4`(a) (ruling 58): the `AlignedBytes`
  16-byte buffer.** The seam reinterprets the cached bytes as `#[repr(C)]` headers:
  `ServerLoadMDXA`/`ServerLoadMDXM` cast `disk_image` to `*mut mdxaHeader_t`/
  `*mut mdxmHeader_t` and dereference/swap multi-byte fields in place — including
  the `LL()` swap writes (`tr_model.cpp:734-739`, `:857-863`) — and
  `model_mdxm`/`model_mdxa` hand those same pointers to ghoul2. The header leading
  fields are `i32` (4-byte-aligned), but an ordinary `Box<[u8]>`/`Vec<u8>` (or the
  `EngineHost::fs_read_file` `Vec<u8>`) carries only 1-byte allocator alignment, so
  a direct `*mut mdxmHeader_t` deref would be UB. Ruling 58 replaces the buffer with
  `AlignedBytes` (`alloc::alloc` + `Layout::from_size_align(len, 16)`, mirroring
  `Z_Malloc`; `Drop` deallocs the same `Layout`), making the casts sound; they stay
  `unsafe` confined at the seam (§D11) with a debug alignment assert at each cast
  site (§F17 "design before transcription" answered). This **unblocks the whole
  first slice** — the `ServerLoadMDXA`/`ServerLoadMDXM` interior (the
  `version`/`ofsEnd` read-and-reject `:703-707`/`:826-830`, the in-place header
  swaps `:734-739`/`:857-863`, the intel-live surface/LOD field swaps +
  `SHADER_MAX_VERTEXES`/`INDEXES` bounds checks `:905-991`),
  `RE_RegisterServerModel`'s `ident` read/dispatch (`:1100-1111`), and the
  `model_mdxm`/`model_mdxa` seam deref now port with the rest (`## Slice hooks`).
- **`TRM-Q5` — the FS PAK-membership/checksum seam.** **RESOLVED by `TRM-D5`(a)
  (ruling 59a): the 20th `EngineHost` method `fs_file_is_in_pak`** (commit
  `4c303bd1`, `engine_host.rs:209`). `FS_FileIsInPAK(name, &checksum)`
  (`oracle/codemp/qcommon/files.cpp:1602-1659`, decl `qcommon/qcommon.h:551` — an
  `int` returning `1` or `-1`, never `0`, writing the pak `pure_checksum` into
  `*pChecksum` on the `1` path) is called **live on the dedicated server** at two
  sites this slice ports: `RE_RegisterServerModels_Malloc` (`tr_model.cpp:212,284`,
  stamps `iPAKFileCheckSum` on every fresh server-model entry) and
  `RE_RegisterModels_DumpNonPure` (`tr_model.cpp:434-436`, dumps entries whose
  current PAK checksum no longer matches — reached from
  `RE_RegisterMedia_LevelLoadBegin`'s `sv_pure` arm, `:535-538`, off the live
  `sv_init.cpp:481` chain). The pass-3 draft flagged this because the then-frozen
  19-method `EngineHost` exposed no PAK method; ruling 59a added the 20th, a §C7
  collapse to `fn fs_file_is_in_pak(&mut self, qpath: &str) -> Option<i32>`
  (`Some(pure_checksum)` = the `==1` in-pure-pak path, `None` = every `-1` path —
  disk-only, not-found, illegal `..`/`::`, non-pure-pak-skipped). Both consumers
  are binary on it (stamp-on-`Some`; dump-on-`None`‖mismatch, the `iCheckSum = -1`
  sentinel collapsing naturally under `Option`), and `MockHost`'s
  `pak_files: BTreeMap<String, i32>` fixture (`mock.rs:148,427-430`) delivers the
  checksum to the goldens. This unblocks `cached_model_binary.rs`'s PAK paths — no
  residual first-slice gate remains (`## Slice hooks`).

## Open questions

None. Every hole raised across passes 1-3 is resolved: `TRM-Q1`/`TRM-Q2` by
`TRM-D1` (ruling 56), `TRM-Q3`/`TRM-Q4` by `TRM-D4` (ruling 58), and `TRM-Q5` by
`TRM-D5` (ruling 59a) — all recorded in `## Resolved questions`. The empty section
is the REVIEWED-gate condition.

## Amendment (user ruling 2026-07-12) — server skins are a name-only pool

The FROZEN content above is unchanged. This records the closure-campaign ruling
(`DEC-18`, commit `64a48bb8`) that extends the model-loader slice with the skin pool
`R_HunkClearCrap` already zeroes.

- **Skins ownership.** `tr.skins`/`numSkins` — the skin pool the frozen `hunk_clear`
  already zeros (`R_HunkClearCrap`, `tr_model.cpp:1683`; State ownership row) — now
  have a stored owner: `RenderModels.skins` + `RenderModels.num_skins`, threaded via
  the same `render_models_call` split-borrow as the model pool. `RenderModels` was
  the sorted-map/`Vec` owner already; the skin pool joins it, not `Ghoul2System`.
- **Shader resolution is name-only.** A skin surface's `shader` is a **name-only pool**
  on the dedicated build: the server reads only `shader->name` (`G2_surfaces.cpp:212`),
  never a compiled `shader_t` — consistent with this doc's §20 classification of the
  whole `tr_shader.cpp` TU (the DEDICATED `refexport_t` has no shader entry, `GetRefAPI`
  `tr_init.cpp:1472`). The skin surface stores the shader **name**; no `R_FindShader`
  poke runs server-side (the same client-dead arm the `_Malloc` replay is, ruling 54).
- **`R_GetSkinByHandle` is the host accessor.** `R_GetSkinByHandle` (the pooled skin for
  a `qhandle_t`) is exposed as an `EngineHost` accessor so `mp_engine_ghoul2` reaches
  skins across the service seam, never by naming an `mp_renderer` type (`G2SV-D5`
  preserved) — exactly as `model_mdxm`/`model_mdxa` back the `.glm`/`.gla` block read.
  This doc **backs** it (owns `RenderModels.skins`); the frozen bone/surface subset in
  `ghoul2-server.md` **consumes** it (that doc's matching amendment closes its model-
  memory gap #2).
