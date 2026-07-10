# rmg-oracle — differential harness for the RMG + qcommon terrain port

Compiles the **unmodified** Raven RMG/terrain TUs
(`codemp/qcommon/cm_terrain.cpp`, `codemp/qcommon/cm_randomterrain.cpp`,
`codemp/RMG/RM_Manager.cpp`) + the real `GenericParser2.cpp` into one standalone
dumper, runs it, and stores canonical golden dumps under `golden/`. The Rust §F
port (`crates/mp/engine/rmg/`, `crates/mp/engine/qcommon/`) must reproduce them
byte-for-byte. Spec: `docs/subsystems/rmg-terrain.md` § **Verification strategy**
(FROZEN); rulings 25/28/38/41/46/47 in `docs/handoffs/engine-fork-discovery.md`.
Precedent: `tools/gp2-oracle/` (§18 discipline).

**CRITICAL — compiled `WITH -DDEDICATED` (ruling 25 / RMG-D1).** The whole MP
engine is built `DEDICATED`, which is the governing fact of the design: the RMG
*generation* path is dead code there. Under `-DDEDICATED` the harness pins
exactly that shipped behavior — `CreateRandomTerrain` is compiled out of the
ctor's reachable path, `mHeightMap` is allocated but never populated, and
`LoadMission` early-outs `false` before constructing any mission.

## Usage

```sh
sh build.sh           # build the dumper, diff current output against golden/
sh build.sh --regen   # rebuild golden/ (after changing a fixture)
```

`build.sh` mirrors the `codemp/` tree into `build/` and copies the oracle TUs +
the real headers-under-test next to the **stub headers** in `stubs/`, so the
sources' relative `#include`s (`../qcommon/`, `../server/`) resolve to the stubs;
`oracle/` is never edited (§18). Toolchain: Homebrew `g++-16`,
`-std=c++14 -w -DDEDICATED`. Unreferenced generation externs are left unresolved
with `-Wl,-undefined,dynamic_lookup` (none is ever *called* under DEDICATED, so
none needs a definition — cheaper and safer than stubbing the whole dead tree;
`-dead_strip` was rejected, it corrupts `RMG_CreateSeed`'s static-table
relocations). Goldens are committed, so `cargo test` needs no C++ toolchain.

## Goldens (the doc's enumeration; #2 and #3 are DROPPED under DEDICATED)

The design (`rmg-terrain.md:1248-1268`) numbers four goldens; #2 (post-`Generate`
heightmap/flatten bytes) and #3 (automap list after a mission spawn) produce
**no output under DEDICATED** — no generation runs, the spawn never happens — so
they are dropped, exactly as RMG-D1 states. Two golden files remain:

### `golden/seed.txt` — golden #1 (`rmg-terrain.md:1248-1251`, :1279-1284)

Pins the **platform-width `holdrand` LCG substrate** — the deterministic draw
sequence `RMG_CreateSeed` and the ctor seed consume (irand `4..9`/`0..100`/
`0..255`, flrand), with the raw `holdrand` state printed after each draw. This is
the doc's stated golden purpose: *"pinning the engine LCG via
`EngineHost::flrand`/`irand`"* (RMG-D4f).

> **Deviation (§F.19 / §19).** The doc's *vehicle* — dumping `RMG_CreateSeed`'s
> seed **string** — is undefined behavior at the **ruled** width. `holdrand` is
> platform-width `c_ulong` (64-bit on this LP64 build; ruling 2026-07-09,
> `jampgame-fork-discovery`, matching the Rust `Rng`), so `result = holdrand >>
> 17` pulls high bits and `irand(a,b)` returns values far outside `[a,b]` (e.g.
> `irand(0,50) → -28577`). `RMG_CreateSeed`'s `FindPiece` then walks its weighted
> table **unbounded** (`cm_randomterrain.cpp:990-1005`) and reads out of bounds —
> a crash no Rust port (`.get() → None`) can reproduce. Per §19 the golden pins
> the **defined substrate** (the LCG draws) the helper is built on, not the UB
> string. That is strictly stronger for the stated purpose (it pins the LCG
> directly) and is reproducible by the port's `Rng`.

### `golden/dedicated.txt` — golden #4, the dedicated-server outcome (`rmg-terrain.md:1256-1278`)

*"An RMG mission on the dedicated server fails identically to C."* One dump
covering all four live observables (In-scope items 1-4, ruling 28):

1. **CCMLandScape construction under DEDICATED** (`cm_terrain.cpp:116-219`) —
   config parse, bounds/heightmap/flatten allocation, the ctor-seeded
   `holdrand = 0x89abcdef`, `UpdatePatches` collision build. Pins dims,
   `patchScalarSize`, `get_rand_seed`, and the memset-0 `flattenMap` stream.
2. **The real `.terrain` parse** (`LoadTerrainDef`, ruling 47) — the ctor
   GP2-parses the committed fixture; the `water` case pins
   `water_contents()`/`water_surface_flags()` and the `altitudetexture` cases
   pin per-height `surfaceFlags`/`contentFlags` (bands: ground `< 64`, cliff
   `≥ 64`), both read through the stubbed `CM_GetShaderInfo` (contract below).
3. **The live `EngineHost::error`** (`cm_terrain.cpp:190-193`) — a second ctor
   with an empty `heightMap` key hits `Com_Error(ERR_FATAL, …)`; the harness
   diverts it through the fork-1 panic model and pins the caught message.
4. **The RmManager lifecycle through the early-out** (`RM_Manager.cpp`) —
   `SetLandScape` + `LoadMission` prints the `#ifndef FINAL_BUILD` banner then
   returns **`false`** (`mTerrain` is `NULL` under DEDICATED); no `SpawnMission`;
   `GetAutomapSymbolCount()` is `0`.

> **§F.19 exclusion.** The streamed `GetHeightMap()` bytes are **excluded** from
> the compare: `mHeightMap` is `Z_Malloc`'d non-zeroing (`cm_terrain.cpp:157`)
> and never written under DEDICATED, so the oracle streams uninitialized heap
> (UB, non-deterministic). Only the deterministic flatten/seed/count/water/
> altitude streams are pinned. No printed value depends on the heightmap bytes.

## `CM_GetShaderInfo` stub contract

`CM_GetShaderInfo` is the un-ported wider-clipmap extern (`cm_shader.cpp:498`,
cm-C-track-owned, RMG-D5 / ruling 41). No `CM_LoadMap` runs in the harness, so
`cmShaderTable` is unpopulated; the harness supplies the shader records
`LoadTerrainDef` reads. Contract (`src/rmg_host_stubs.cpp`):

- For a shader `name`, returns a **stable** (pool-cached) `CCMShader*` whose flags
  are a documented FNV-1a-32 function of the name string:
  - `h = 2166136261; for each byte c: h = (h ^ c) * 16777619`
  - `contentFlags = (int)( h        & 0xffff )`
  - `surfaceFlags = (int)( (h >> 16) & 0xffff )`
- The `name` overload **never returns NULL** (matches `cm_shader.cpp:498`).

Deterministic and reproducible without any retail shader data. The golden pins
whatever these flags produce for the fixture's three shader names
(`textures/rmg/{ground,cliff,water}`).

## Fixtures

- `fixtures/ext_data/RMG/dedicated.terrain` — the **hand-authored** terrain def
  (no retail data, ruling 47). GP2 syntax: a `terrainDef` group containing two
  `altitudetexture` sub-groups (heights 64 then 0, so the bands differ) and one
  `water` sub-group (height 32). The `.terrain` path the ctor builds from
  `CONFIG`'s `terrainDef` key (`ext_data/RMG/dedicated.terrain`) resolves here
  via the `Com_ParseTextFile` stub (option (b): the fixture is returned for the
  first `ext_data/RMG/…` lookup). The two `CONFIG` infostrings live in
  `main.cpp`.

## Stub fidelity

Stubs supply only what the three TUs reference; oracle behavior is transcribed
**verbatim** where it is behavior-under-test: the `holdrand` LCG
(`flrand`/`irand`/`Rand_Init`, `q_math.c:1432-1470`), `Info_ValueForKey`
(`q_shared.c`), `SetPlaneSignbits`, and the vec math macros
(`q_shared.h:1354-1399`). `Com_Printf`→stdout, `Com_DPrintf`→silent (developer
0, matching the shipped server), `Com_Error`→fork-1 panic (throw), `Z_Malloc`→
non-zeroing `malloc` (the §F.19 default), `Com_ParseTextFile`→real FS read of the
fixture feeding the **real** `GenericParser2`. The generation types `LoadMission`
names past its early-out (`CRMMission`/`CRMObjective`) are inline no-ops
(`stubs/RMG/RM_Headers.h`) — never constructed under DEDICATED; `RM_Manager.h`
itself is the **real** header, compiled unmodified.

## No OpenJK peer

OpenJK dropped RMG entirely (RMG-D4i), so there is no engine-vs-engine A/B square
for these paths — a hard constraint, not a choice.
