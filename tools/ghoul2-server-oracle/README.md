# ghoul2-server-oracle — differential golden harness (partial)

Verifies part of the `mp_engine_ghoul2` server-side bone/arena port
(`docs/subsystems/ghoul2-server.md`, FROZEN, § Verification strategy) against the
**unmodified** Raven `codemp/ghoul2/*.cpp`, exactly like `tools/gp2-oracle` /
`tools/icarus-oracle` / `tools/trmodel-oracle` (porting-rules §18). The oracle
`.cpp`/`.h` are copied into `build/` and compiled standalone against stub
headers; canonical dumps are stored under `goldens/` and committed, so the Rust
parity tests need **no** C++ toolchain — only `build.sh` does, to (re)generate or
check.

`oracle/` is never edited.

> **Scope is partial by design.** This harness covers the three model-free,
> standalone-compilable islands of the subsystem — the **arena/handle** scheme,
> the **bolt-list** bookkeeping, and the **generated-surface list**. The
> bone-transform / bolt-matrix / collision / ragdoll / gore goldens named in the
> doc need the full renderer + collision + model-memory closure and are **not
> yet** standalone. Every uncovered area is enumerated in § Uncovered (gaps) — no
> silent coverage is claimed.

## Usage

```sh
sh build.sh          # build + run all modes, diff dumps against goldens/
sh build.sh --regen  # regenerate goldens/*
```

Toolchain: Homebrew `g++-16` (override with `CXX=`). Flags mirror the sibling
harnesses (`-fsigned-char -ffp-contract=off -fno-fast-math`) plus the WinDed
DEDICATED Release macro set the port models (§ Raven ground truth): **`-DDEDICATED`**,
**`-DNDEBUG`** (asserts no-op), **`-D_M_IX86`** (shipped x86 target — the
`#ifndef _M_IX86` big-endian swaps compile OUT), **`-D_G2_GORE`** (ON in MP,
`q_shared.h:3110`). `-fpermissive` downgrades LP64 pointer-width warnings (exact
on the 32-bit ship, never dumped). `-Wl,-dead_strip` drops every function the
dumpers never reach, so only the live-path engine-seam symbols need a body in
`host.cpp`.

## Compiled oracle TUs

| Oracle TU | Covered surface |
| --- | --- |
| `codemp/ghoul2/G2_API.cpp` | `Ghoul2InfoArray` arena (`:310-493`) — `New`/`IsValid`/`Delete`/`DeleteLow`, the singleton + `Ghoul2InfoArray_Free`. |
| `codemp/ghoul2/G2_bolts.cpp` | Whole TU — bolt-list add/find/remove/prune. |
| `codemp/ghoul2/G2_surfaces.cpp` | Whole TU compiles; the generated-surface list fns are driven; the model-memory name-lookup fns are linked but not exercised (gap). |

All three compile clean under the stubs with **no** source normalizations (only
build flags + the stub-header environment Raven's MSVC PCH supplied ambiently).

## Goldens (each pins a § Verification strategy unit)

| Golden | Pins | Doc unit |
| --- | --- | --- |
| `goldens/arena.txt` | `Ghoul2InfoArray` `New`/`IsValid`/`Delete` — the packed handle scheme (idx in low `G2_MODEL_BITS=10`, generation in high bits), the per-`Delete` generation bump (`+MAX_G2_MODELS`), LIFO slot reuse (`DeleteLow` `push_front`), stale-handle invalidation, and multi-slot free-list ordering. | "Arena/handle goldens" (`G2SV-D6`) — **partial**: the rollover **reset** arm is excluded as oracle UB (see Normalizations). |
| `goldens/bolts.txt` | `G2_Add_Bolt_Surf_Num` / `G2_Find_Bolt_Surface_Num` / `G2_Find_Bolt_Bone_Num` / `G2_Remove_Bolt` (boltUsed decrement + tail resize) / `G2_RemoveRedundantBolts` (incl. the original-surface fall-through quirk) / `G2_Init_Bolt_List`. | "Bolt goldens" — **partial**: bolt-**list** management only; the `G2API_GetBoltMatrix` write-through matrix math (`G2SV-D1`) needs a live bone cache (gap). |
| `goldens/surfaces.txt` | `G2_AddSurface` (generated-surface marker `surface=10000`, packed `genPolySurfaceIndex`, lod clamp, free-slot reuse) / `G2_FindOverrideSurface` / `G2_RemoveSurface` (tail resize). | Surface-list bookkeeping (supports the collision/`G2SV-D6` path) — **partial**: the name-lookup on/off fns are a gap. |

The eventual Rust parity tests (`tests/ghoul2_parity.rs` in `mp_engine_ghoul2`)
read `goldens/*` from here and must reproduce each byte-for-byte.

## Fixtures (hand-authored, no retail assets — ruling 14)

The three covered units are **model-free**: their fixtures are the hand-authored
operation sequences encoded in `dump_arena.cpp` / `dump_bolts.cpp` /
`dump_surfaces.cpp` over synthetic in-memory `CGhoul2Info` / `boltInfo_v` /
`surfaceInfo_v` state. No disk model is loaded, so **no `.glm`/`.gla` fixture
files are committed** — none is needed to reach this surface.

`tools/trmodel-oracle`'s `modelgen` (`.glm`/`.gla` generator, hand-authored, no
retail data) is the **designated** fixture generator for the deferred
model-memory goldens below (bone-transform / collision / ragdoll load a
`.glm`/`.gla` set); it is available and reused verbatim when those units come
online. It is intentionally **not** wired in yet, to avoid committing fixtures no
golden consumes.

## Deterministic host (`host.cpp`)

Provides only the engine-seam symbols the live code paths reach:
`Com_Error`/`Com_Printf` (console/fatal), a no-op `RemoveBoneCache` (the arena
never builds a bone cache), and a **verbatim transcription** of
`G2_DecideTraceLod` (oracle `G2_misc.cpp:376-395`) — supplied directly because
its owning TU (`G2_misc.cpp`) pulls the whole collision/gore/server closure and
is out of this harness's standalone scope. No raw pointer/address is ever emitted
(only handles, indices, markers, packed ints), so every golden is run-twice
byte-identical (verified: a full clean rebuild reproduces each golden exactly).

## Normalizations (documented; NONE edit oracle source)

The oracle `.cpp` is compiled **unmodified**; the only accommodations are build
flags and the stub-header environment:

- **`-D_M_IX86` / `-DDEDICATED` / `-DNDEBUG` / `-D_G2_GORE`** — the WinDed
  DEDICATED Release config the port models (§ Raven ground truth), not source
  edits.
- **`-fpermissive`** downgrades LP64 pointer-width warnings (exact on the 32-bit
  ship; the affected values are never dumped).
- **Stub headers** (`stubs/`, seeded from `tools/trmodel-oracle`) declare exactly
  the shared/renderer surface the compiled TUs name; types that only appear in
  unreached prototypes (`CollisionRecord_t`, `SSkinGoreData`, the IK param
  blocks, `CMiniHeap`) are present for compile and never dumped.
- **Arena generation ROLLOVER excluded (§F19 UB).** `DeleteLow`'s reset arm
  (`G2_API.cpp:328-333`) fires only when `(mId>>10) > (1<<21)`, but the `int`
  `mId` reaches exactly `2^31` (`== INT_MIN`, signed-overflow UB) at that same
  generation, so the reset test never observes a positive over-threshold value —
  the reset is unreachable without UB. `arena.txt` therefore goldens the
  **defined** surface (per-delete bump, reuse, invalidation, ordering) and keeps
  the UB rollover out of the shared golden, exactly as the doc's §F19 clause
  prescribes.
- **`G2_DecideTraceLod`** is a harness-local verbatim transcription, not a linked
  oracle TU (see Deterministic host). The Rust port verifies the real function
  under `misc.rs`; here it only supplies the surface golden's `genLod` clamp.

## Uncovered (gaps) — the deferred § Verification units

These need surface this harness does not yet compile standalone; each is a
distinct future extension:

1. **Bone-transform goldens** (`G2SV-D6`) — `G2_TransformBone` /
   `Multiply_3x4Matrix` / `G2_CreateQuaterion` / `CBoneCache` `EvalLow`/`Eval`/
   `EvalRender` / `G2_ConstructGhoulSkeleton`. Live in `renderer/tr_ghoul2.cpp`
   (the full GL renderer TU: `client.h`, `tr_local.h` GL state, `matcomp`). Needs
   a loaded `.glm`/`.gla` (modelgen) + the `EngineHost` model-memory read. Not
   standalone here.
2. **Bolt-matrix goldens** (`G2SV-D1`, write-through + bool) —
   `G2API_GetBoltMatrix` across angles/position/scale incl. the `gG2_GBM*` flags.
   Needs a live `CBoneCache` (⇒ gap 1) and model memory.
3. **Collision goldens** — `G2API_CollisionDetect` → `G2_TraceModels`
   (`G2_misc.cpp`), which pulls `server/server.h`, `CM_BoxTrace`, the transform
   pipeline, and model memory. Not standalone.
4. **RagDoll determinism goldens** (`G2SV-D3`, load-bearing) — `G2_bones.cpp`
   (4,907 LOC): the settle/IK solver, its `flrand` seeding, `broadsword` cvar
   reads, and `model_mdxa` basepose resolve — all via `EngineHost`. Needs the
   fixture-backed `MockHost` + model memory.
5. **Gore goldens** — `AllocGoreRecord`/`FindGoreSet`/`DeleteGoreSet` tag
   sequencing (`G2_misc.cpp`, same closure as collision). The server slice is
   all-null (no `TS.gore` setter, `G2SV-Q4`), so this is low-priority and not
   M3-gating.
6. **Surface name-lookup on/off** — `G2_SetSurfaceOnOff` / `G2_IsSurfaceLegal`
   (walk the real mdxm surface hierarchy) and `G2_SetSurfaceOnOffFromSkin`
   (`R_GetSkinByHandle`). Compiled but not driven — needs a loaded `.glm` +
   `R_GetModelByHandle` host wiring (modelgen would supply the fixture).
7. **Model-name bolt path** — `G2_Add_Bolt` (bone/surface name lookup over mdxm/
   mdxa memory). Same model-memory prerequisite as gap 6.

Bringing gaps 1–5 online is the substantive remaining work: it requires either
compiling `tr_ghoul2.cpp` / `G2_bones.cpp` / `G2_misc.cpp` standalone against a
much larger stub-header + `MockHost` environment, or driving them through the
game-DLL A/B referee (the doc's other DEC-09 verification arm).

## On-disk size

Committed artifacts: 3 goldens (~3 KB) + 6 stub headers + 4 dumpers/host + the
harness scripts. `build/` is git-ignored.
