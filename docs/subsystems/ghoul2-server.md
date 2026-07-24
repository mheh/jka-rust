# Server-side Ghoul2 + renderer bone/model internals Design
Status: FROZEN (user sign-off 2026-07-09)     Supersedes: none
Decision prefix: G2SV     Ledger deps: DEC-04 (per-mode), DEC-09 (verification);
engine-fork-discovery rulings 2 (state placement), 3 (fn-statics), 7 (this doc),
11 (EngineHost), 12 (direct Engine fields), 18 (GetBoltMatrix defect), 22
(2026-07-09: gore apply/record split, `CRagDollUpdateParams` §F17 enum, bone-cache
arena, `CGhoul2Info_v` method colocation), 24 (2026-07-09: Stage-0 host-interface
crate PINNED — `crates/mp/host-interface`, package `mp_host_interface`), 26
(2026-07-09: gore correction — `DestroyGoreTexCoordinates`/`DeleteGoreRecord` are
LIVE, closing `G2SV-Q8`), **29** (2026-07-09, pass 5: the three linked raw-pointer
shape holes — `delete`/`delete_low` UP to `Ghoul2System`, ragdoll blist-**indices**
+ per-call `EngineHost` basepose resolve, gore per-LOD `Vec<f32>` buffers backing
the frozen `tex` pointers — rendered as `G2SV-D13`, closing `G2SV-Q9`/`G2SV-Q10`),
**31**+**33** (2026-07-09, pass 5: the Stage-0 `mp_host_interface` crate is BUILT
and green, commit `4b7f01b0`; real `EngineHost` signatures + fixture-backed
`MockHost` — rendered as `G2SV-D14`), **36** (2026-07-09, pass 6: `EngineHost`
EXTENDED and BUILT, commit `a9820853` — `+ model_mdxm`/`model_mdxa` loader
model-block accessors, `+ cvar_integer`, `+ sv_time`, `+ fs_write_file`; closes
**both halves** of `G2SV-Q11` — rendered as `G2SV-D15`), **39b** (2026-07-09, pass
6: the attach-trio `G2API_AttachInstanceToEntNum`/`ClearAttachedInstance`/
`CleanEntAttachments` are compiled **no-ops** reached by LIVE syscall arms, not §20
drops — rendered as `G2SV-D16`)

**Pass 6 (2026-07-09)** folds ruling 36 as `G2SV-D15` and ruling 39b as `G2SV-D16`;
`G2SV-D1`–`G2SV-D14` stand unchanged (the pass-5 `D13`/`D14` append numbering is
blessed — D1–D12 are not renumbered). Ruling 36 EXTENDED `EngineHost` to **15
methods** (commit `a9820853`, re-quoted verbatim in `## Seam definition`): the two
absent services that pass 5 escalated — loader model-memory read and cvar read —
are now served by `model_mdxm`/`model_mdxa` and `cvar_integer`, so **`G2SV-Q11` is
SETTLED (both (a) and (d))** and moves to Resolved questions; `## Open questions` is
now empty. Ruling 39b corrects the attach-trio's classification from §20-dropped to
§C10 compiled-no-op (`G2SV-D16`). **Pass 5 (2026-07-09)** had folded rulings 29/31/33
as `G2SV-D13`/`G2SV-D14` and resolved `G2SV-Q9`/`G2SV-Q10`.

This is a C++-track (`porting-rules.md` §F) design doc. It carries the machine-
readable `files:` roster and `divergences:` list (doc-standards rule 6) that
`.claude/workflows/port-cpp-subsystem.js` consumes via `designPath`; both live
in the `## Files roster` YAML block below (the `files:` and `divergences:` keys).

## Standing context

Links only — never restated here:
- `docs/workspace-architecture.md` — crate graph. This subsystem's server-side
  bone pipeline lands entirely in `mp_engine_ghoul2` (`crates/mp/engine/ghoul2`);
  `ghoul2` is the upstream shared crate ("depended on by both `engine/*` and
  `cgame`", `:193`) and does **not** depend on `mp_renderer` (`crates/mp/renderer`,
  whose `Cargo.toml` deps are qshared/engine-qcommon/native-platform — no ghoul2).
  The `mdxa`/`mdxm` model **loader** stays in `mp_renderer` and is reached across
  the `EngineHost` service seam, not by a crate edge (`G2SV-D5`).
- `docs/porting-rules.md` — §B (state spine), §F (C++ track: §F17 shape-first,
  §F18 differential goldens, §F19 UB divergence, §F20 drop-dead-surface, §F21
  one-class-per-file), the comment/source-ref rules.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — the WinDed DEDICATED link set,
  the ghoul2⇄renderer weld (§"ghoul2 ⇄ renderer entanglement is structural"), the
  port-process discipline (no `todo!()`/speculative dead-code).
- `docs/handoffs/engine-fork-discovery.md` — the settled rulings ledger: ruling 2
  (globals → `Engine` sub-structs by owning `.c` file), ruling 3 (fn-statics →
  three-kind rule), ruling 7 (this doc's five-subsystem §F list), ruling 11
  (`EngineHost` services trait), ruling 12 (direct `Engine` fields), ruling 18
  (the `g2api_get_bolt_matrix` write-through defect), **ruling 22** (2026-07-09:
  the ghoul2 gore apply/record split, the `CRagDollUpdateParams` §F17 enum shape,
  the bone-cache generational arena, and `CGhoul2Info_v` method colocation —
  rendered here as `G2SV-D7`–`G2SV-D10`; rulings 11–18 stand unchanged), **ruling
  24** (2026-07-09: the Stage-0 host-interface crate is PINNED —
  `crates/mp/host-interface`, package `mp_host_interface`; rendered here as
  `G2SV-D12`), **ruling 26** (2026-07-09: the gore correction —
  `DestroyGoreTexCoordinates` and `DeleteGoreRecord` move to the LIVE gore bucket,
  reached via `~CGoreSet` ← `DeleteGoreSet`; the ruling-22 dead-bucket listing was
  an engineorder graph blind spot — implicit destructor calls are not graph edges;
  rendered here as `G2SV-D11`, closing `G2SV-Q8`), the G2SV-Q1/G2SV-Q2 evidence
  resolutions, **ruling 29** (2026-07-09, pass 5: the three linked raw-pointer shape
  holes — `delete`/`delete_low` UP to `Ghoul2System`, ragdoll blist-indices + per-call
  `EngineHost` basepose resolve, gore per-LOD `Vec<f32>` buffers backing the frozen
  `tex` pointers — rendered as `G2SV-D13`, closing `G2SV-Q9`/`G2SV-Q10`), and
  **rulings 31+33** (2026-07-09, pass 5: the Stage-0 `mp_host_interface` crate is
  BUILT and green, commit `4b7f01b0`; real `EngineHost` signatures quoted verbatim +
  fixture-backed `MockHost` — rendered as `G2SV-D14`), **ruling 36** (2026-07-09,
  pass 6: `EngineHost` EXTENDED and BUILT, commit `a9820853` — `model_mdxm`/
  `model_mdxa` loader model-block accessors, `cvar_integer`, `sv_time`,
  `fs_write_file`; closes both halves of `G2SV-Q11` — rendered as `G2SV-D15`), and
  **ruling 39b** (2026-07-09, pass 6: the attach-trio is a compiled §C10 no-op, not a
  §20 drop — rendered as `G2SV-D16`).
- `crates/mp/host-interface/` — the BUILT, frozen host crate (package
  `mp_host_interface`): `src/engine_host.rs` holds the **15-method** `EngineHost`
  trait (Stage-0 at commit `4b7f01b0`, rulings 31/33; EXTENDED at commit `a9820853`,
  ruling 36 — quoted verbatim in `## Seam definition` so this doc is
  self-contained), `src/mock.rs` the fixture-backed `MockHost` goldens vehicle
  (ruling 32; ruling-36 fixtures add the `cvars` name→i32 map and the
  `mdxm_blocks`/`mdxa_blocks` model-byte maps). Reading this crate is permitted; a
  porter `use`s `mp_host_interface::EngineHost`.
- `docs/GOAL-engine.md` — M3 gate: "G2 bone/bolt/collision goldens".
- GP2 exemplar: `crates/mp/engine/qcommon/src/gp2/`, `tools/gp2-oracle/`.
- Already type-ported (layout frozen, this doc reuses, never re-declares):
  `crates/mp/engine/ghoul2/src/shared/` (`CGhoul2Info_v`, `boneInfo_t`,
  `boltInfo_t`, `surfaceInfo_t`, `EG2_Collision`) and `src/gore/`
  (`GoreTextureCoordinates`, `SGoreSurface`, `SSkinGoreData`, `CRagDollParams`,
  `SRagDollEffectorCollision`).

## Scope & non-goals

**In scope.** The idiomatic reimplementation of:
- `oracle/codemp/ghoul2/` (9,230 LOC / 224 fns): `G2_API.cpp`, `G2_bones.cpp`
  (incl. the RagDoll + IK solver — server-live, `G2SV-D3`), `G2_bolts.cpp`,
  `G2_surfaces.cpp`, `G2_misc.cpp`, the `CGoreSet`/gore-record store (`G2_gore.h`
  + the gore parts of `G2_misc.cpp`).
- The welded renderer **bone subset** of `oracle/codemp/renderer/tr_ghoul2.cpp`
  (fn-extent 3,505 LOC after the WinDed DEDICATED preprocess, `G2SV-D2`):
  `CBoneCache`, `CTransformBone`, `SBoneCalc`, `G2_TransformBone`
  (`tr_ghoul2.cpp:1541`), `G2_TransformGhoulBones` (`:2075`),
  `G2_ConstructGhoulSkeleton` (`:3567`), `EvalBoneCache` (`:585`), the low-level
  matrix helpers (`Multiply_3x4Matrix`, `G2_CreateQuaterion`,
  `G2_CreateMatrixFromQuaterion`), and the `mdxa*` bone accessors the server's
  collision path needs (`G2_GetBoneMatrixLow`, `G2_GetBoneBasepose`). These live
  in `renderer/tr_ghoul2.cpp` but are pure bone math the dedicated server links
  for collision; they port into `mp_engine_ghoul2` (`G2SV-D5`).
- The `Ghoul2InfoArray` singleton → owned arena (`G2SV-D6`, §B5 pattern).

**Non-goals (punted, with pointers).**
- The `mdxa`/`mdxm` model, mesh, and shader **loader**. `G2SV-D2` fixes the
  renderer subset at exactly the 8 WinDed vcproj renderer TUs; the loader lives in
  the other 7 (`tr_model` fn-extent 1,547 LOC, `tr_shader` 3,139, `tr_image` 845,
  `tr_init` 538, `matcomp` 240, `tr_main` 47, `tr_mesh` 39). The bone pipeline
  *reads* `mdxaHeader_t`/`mdxaSkel_t` out of loader-owned model memory; that read
  crosses the `EngineHost` service seam (`G2SV-D5`), not a crate edge. Those
  loader TUs are a **separate `tr_model` subsystem doc**, not this one.
- SP ghoul2 (`oracle/code/ghoul2/`, `jasp` engine, statically linked). This doc
  is the MP/`jamp` server slice only; every roster entry is `mode: mp`. SP is a
  future diff per DEC-04 / porting-rules §F20 (duplicate, don't unify).
- Client/`cgame`-side rendering (`RB_SurfaceGhoul`, `R_AddGhoulSurfaces`) —
  compiled out under `DEDICATED`.
- **The internals of the `EngineHost` trait's implementation** — the trait itself
  is BUILT and frozen (`G2SV-D14`/`G2SV-D15`, rulings 31/33/36,
  `crates/mp/host-interface/src/engine_host.rs`, Stage-0 commit `4b7f01b0` +
  ruling-36 extension commit `a9820853`) and its **15** signatures are quoted verbatim
  in `## Seam definition`; a porter `use`s `mp_host_interface::EngineHost` and does not
  re-declare it. What this doc does **not** own is how `Engine`/the split-borrow view
  implements each method (the aggregate-`Engine` side, ruling 11/24). Every service
  ghoul2 consumes now maps onto a frozen method: collision trace (`G2_TraceModels`,
  `G2_GorePolys`, `Rag_Trace`'s `CM_BoxTrace`, `G2_bones.cpp:2709`) → `trace`;
  print/error → `print`/`error`; save/load and model file reads → `fs_read_file`/
  `fs_free_file`/`fs_write_file`; `flrand` (the ragdoll init/settle seeding) → `flrand`;
  the loader model-memory read of the parsed `.glm`/`.gla` blocks (for `CBoneCache::new`
  parent-seeding, `render/skeleton.rs`, and — per `G2SV-D13`(b) / ruling 29 — the
  ragdoll basepose resolution) → **`model_mdxm`/`model_mdxa`** (ruling 36, `G2SV-D15`);
  and cvar read (`cg_g2MarksAllModels`, `G2_misc.cpp:40`, **and** the renderer-owned
  `broadsword*` ragdoll cvar family the solver reads, `G2_bones.cpp:1176-1189`) →
  **`cvar_integer`** (ruling 36, `G2SV-D15`; a missing name reads 0, matching
  `Cvar_VariableIntegerValue`, so the renderer-owned `broadsword` family is read
  without any `mp_renderer` edge). The two services pass 5 escalated as `G2SV-Q11` are
  thereby **served** — `G2SV-Q11` is SETTLED (Resolved questions), so no gap remains.
- **Sequencing consequence.** The host-consuming bodies bind the frozen `EngineHost`
  trait; the §F signatures freeze here and take `&mut impl EngineHost`. Because ruling
  36 filled the two formerly-absent services, **every roster body now transcribes
  against the built 15-method trait** — no body blocks on Stage-0 (done) any longer.
  (Pass 5's `G2SV-Q11` block over most of the roster is discharged; the `## Slice
  hooks` M3 note records the now-unblocked partition.)
- **Model-memory type-location reconciliation (`G2SV-D5`-forced, not a new
  decision).** `mdxmHeader_t`/`mdxaHeader_t`/`mdxaSkel_t` live only in `mp_renderer`
  (`crates/mp/renderer/src/mdx_format/`, type-rosetta) and `G2SV-D5` forbids a
  `mp_engine_ghoul2` → `mp_renderer` crate edge. Ruling 36 honors that: `model_mdxm`/
  `model_mdxa` returned `*mut c_void` (superseded by DEC-35 — see the
  amendment blockquote at the `G2SV-D15` entry below), **not** an
  `mp_renderer` struct — ghoul2 does its byte arithmetic off the returned pointer
  unchanged (`tr_ghoul2.cpp:416-421`), and `mp_engine_ghoul2` **never names** the
  mdxm/mdxa header types as Rust types. Any `mp_renderer` coupling lives in the
  host-interface crate's implementation, never in `mp_engine_ghoul2`; this doc keeps
  those types out of every §F signature (`CBoneCache::new` takes only `qhandle_t`,
  `G2SV-D5`). The accessors return NULL exactly where Raven's `model_t` pointers are
  NULL (`R_GetModelByHandle` → `model_t` → `->mdxm`/`->mdxa`, `G2_API.cpp:2716-2739`).

## Raven ground truth

**Build config (the WinDed DEDICATED Release macro set).** From the plan
appendix (`docs/plans/2026-07-08-mp-engine-build-out.md:570`): `-DNDEBUG
-DDEDICATED -DBOTLIB`, `FINAL_BUILD` undefined, no platform macro. On top of
that, source-level defines decide the ghoul2 `#ifdef` map:
- `_G2_GORE` — **ON**. Defined at `oracle/codemp/game/q_shared.h:3110` (a source
  `#define`, reached through the `q_shared.h` include chain), not a vcproj macro.
  So all gore code (`CGoreSet`, `GoreTextureCoordinates`, `G2API_AddSkinGore`,
  `mGoreSetTag`, `goreShader`) **is compiled** in MP.
- `_G2_LISTEN_SERVER_OPT` — **OFF**. No `#define` exists anywhere in `codemp/`
  (grep: only `#ifdef`/`#ifndef` guards at `tr_ghoul2.cpp:578,3540,3552,3611`,
  `ghoul2_shared.h:277,305`, `G2_API.cpp:196,202,216,223,235,241`). `CGhoul2Info::
  entityNum` (`ghoul2_shared.h:278`), the `g2ClientAttachments[]` override path
  (`G2_API.cpp:197` and its writers `:209,217,228`), and `CopyBoneCache`
  (`tr_ghoul2.cpp:579`) compile out. `G2API_OverrideServerWithClientData`
  (`G2_API.cpp:239`) keeps a **live arm** — `#ifndef _G2_LISTEN_SERVER_OPT →
  return qfalse;` (`:241-242`) — the rich override body (`:244-`) compiles out.
  Treatment: `G2SV-D4`.
- `_SOF2` — **OFF**. The `ghoul2_shared.h:184-219` `SSkinGoreData`/`goreEnum_t`
  variant is dead; MP uses `SSkinGoreData_s` (`q_shared.h:3112`, unconditional).
- `_XBOX` — **OFF**. `CTransformBone::renderMatrix`/`pad`, the raw-`Z_Malloc`
  `mFinalBones`/`mSmoothBones` arrays, `EvalFull` are dead; the `vector<>` arm is
  live (`tr_ghoul2.cpp:363-367`, `:408-411`).
- `G2_PERFORMANCE_ANALYSIS`, `_FULL_G2_LEAK_CHECKING` — **ON** (both `#define`d
  under `#ifndef FINAL_BUILD`, `q_shared.h:45-47`; the WinDed set leaves
  `FINAL_BUILD` undefined). Their file-scope globals are instrumentation only —
  the `G2Time_*`/`G2PerformanceCounter_*`/`G2PerformanceTimer_*` timers
  (`tr_ghoul2.cpp:42-62`, reset/report at `:64-90`) and the leak counters — with
  no ABI-seam parity surface, so both sets are dropped per §F20 (State ownership).

**The arena and its handle scheme.** `Ghoul2InfoArray` (`G2_API.cpp:310`) is a
fixed table of `MAX_G2_MODELS` (`=1024`, `:304`) slots: `vector<CGhoul2Info>
mInfos[1024]`, `int mIds[1024]`, and a `list<int> mFreeIndecies`. A handle packs
a slot index in its low `G2_MODEL_BITS` (`=10`, `:305`) bits (`G2_INDEX_MASK =
MAX_G2_MODELS-1`, `:308`); the high bits are a generation counter. `New()`
(`:386`) pops a free index and returns `mIds[idx]`. `Delete`/`DeleteLow`
(`:413`, `:315`) frees bone caches, clears the slot's vector, and bumps the
generation by `+MAX_G2_MODELS`, rolling over to `MAX_G2_MODELS+idx` when the
generation would exceed `1<<(31-G2_MODEL_BITS)` (`:330`). `IsValid(handle)`
(`:399`) is `mIds[handle & MASK] == handle`. The whole thing hangs off one
lazily-constructed `static Ghoul2InfoArray *singleton` reached only through
`TheGhoul2InfoArray()` (`:477-484`); `Ghoul2InfoArray_Free()` (`:487`) tears it
down. `CGhoul2Info_v` (`ghoul2_shared.h:328`) is a copyable wrapper holding just
`int mItem` (the handle) and forwarding `[]`/`resize`/`size`/`push_back` (plus the
lifecycle ops `Alloc`/`Free`/`clear`/`DeepCopy`, `ghoul2_shared.h:335-430`) through
`TheGhoul2InfoArray().Get(mItem)` — this is exactly the §B5 arena + id +
borrow-wrapper triad. In the port these wrapper methods are owned by `info_array.rs`
(roster): the read/index forms (`[]`/`size`/`resize`/`push_back`) become
`Ghoul2InfoArray::get`/`get_mut(handle)` forwarding (porting-rules §B4, state
threaded not reached), and `Alloc`/`Free`/`clear`/`DeepCopy` map to `New`/`Delete`
plus DeepCopy's runtime-state zeroing loop (`:385-397`); the already-ported
`shared/cghoul2_info_v.rs` stays layout-only (`mItem: i32`). `Get()` on an invalid handle returns a shared
function-`static` `null` vector it first `.clear()`s (`:427-439`) — a
non-reentrant aliasing hack.

**Per-instance state.** Each `CGhoul2Info` (`ghoul2_shared.h:240`) owns three STL
vectors (`mSlist`, `mBltlist`, `mBlist`), a save-serialized middle band
(`mModelindex`..`mFlags`, incl. `mGoreSetTag` under `_G2_GORE`), and
non-serialized runtime pointers: `mTransformedVertsArray`, `CBoneCache
*mBoneCache` (`:265`), and validity/model pointers set by `G2_SetupModelPointers`
(`G2_misc.cpp:1839`, `tr_ghoul2.cpp:107`). `DeepCopy` (`:382`) copies the vector
but zeroes `mBoneCache`, `mTransformedVertsArray`, `mSkelFrameNum`,
`mMeshFrameNum` on every element — runtime state is per-instance and never shared
across a copy.

**The bone cache.** `CBoneCache` (`tr_ghoul2.cpp:206`) is built per
`(model, mdxaHeader_t)` in its ctor (`:390`), sizing three parallel per-bone
arrays to `header->numBones`: `vector<SBoneCalc> mBones` (frame/lerp inputs),
`vector<CTransformBone> mFinalBones` (the evaluated matrix + parent + `touch`
stamp), and `vector<CTransformBone> mSmoothBones` (render-smoothing history). It
seeds each bone's `parent` from the model's `mdxaSkel_t` (`:419-425`). Evaluation
is lazy and memoized by an integer `touch` generation (`mCurrentTouch` starts at
3, `mLastTouch`/`mLastLastTouch` at 2/1, `:426-430`): `EvalLow` (`:236`)
recurses to the parent, copies down the `SBoneCalc`, calls the free function
`G2_TransformBone(index, *this)` (`:1541`), and stamps `touch`. `Eval` (`:455`),
`EvalUnsmooth` (`:446`), and `EvalRender` (`:520`, applies `SmoothLow`) are the
public read paths; `EvalBoneCache(index, boneCache)` (`:585`) is the free-fn
entry the ghoul2 dir calls. The skeleton is (re)built by
`G2_ConstructGhoulSkeleton` (`:3567`) driving `G2_TransformGhoulBones` (`:2075`)
per model; `worldMatrix`/`worldMatrixInv` (`tr_ghoul2.cpp:136-137`) are set once
per construct by `G2_GenerateWorldMatrix` (`G2_misc.cpp:1678`) and read by the
transform chain. `RemoveBoneCache` (`:569`) `delete`s the cache; the arena's
`DeleteLow` is the owner of that lifetime (`G2_API.cpp:321-325`).

**The render-traversal flag.** `HackadelicOnClient` (`tr_ghoul2.cpp:104`, `bool
HackadelicOnClient=false; // means this is a render traversal`) is a file-scope
bool branched on 14× — including inside functions this doc ports: the
`G2_TransformBone` chain (`:1638,1868-2023`) and `G2_TransformGhoulBones`
(`:2125`, `if (HackadelicOnClient && smooth && !com_dedicated->integer)`). Its
only writers are in `R_AddGhoulSurfaces` (`=true :3425`, `=false :3532`), whose
entire body is `#ifndef DEDICATED` (`:3384-3537`, client render only). In the
WinDed DEDICATED build those writes compile out, so server-side the flag is fixed
at its `false` initializer and every read takes the false arm — the `:2125`
dedicated-awareness is redundant here. It is therefore not threaded state; a
porter may fold the dead false arms (§C10).

**Gore store.** Two file-scope maps in `G2_misc.cpp`: `GoreRecords`
(`map<int,GoreTextureCoordinates>`, `:35`) capped at `MAX_GORE_RECORDS=500`
(`:56`) with a rolling `CurrentTag`/`CurrentTagUpper` UUID (`:32-33`,
`GORE_TAG_UPPER=256`, `GORE_TAG_MASK=~255`), and `GoreSets` (`map<int,CGoreSet*>`,
`:125`) keyed by a `CurrentGoreSet` UUID (`:124`). `CGoreSet` (`G2_gore.h:59`)
holds `mMyGoreSetTag`, a `mRefCount`, and a `multimap<int,SGoreSurface>
mGoreRecords`. `AllocGoreRecord`/`FindGoreRecord`/`DeleteGoreRecord`
(`G2_misc.cpp:58/103/118`) and `FindGoreSet`/`NewGoreSet`/`DeleteGoreSet`
(`:127/142/153`) are the store API; `GoreTagsTemp` (`:36`, keyed on
`(goreModelIndex, surfaceNum)`; the file-scope `goreModelIndex` `:38` is the
trace-loop-scoped model index the `G2_TraceModels` loop sets `:1539` and the
per-poly gore-tag lookup reads `:959,1000`) and `cg_g2MarksAllModels` (`:40`)
round out the map-side globals.

The gore *apply/trace* path adds more file-scope statics (all `#ifdef _G2_GORE`,
ON). `G2_misc.cpp:793-798`: the per-vertex scratch `GoreVerts[MAX_GORE_VERTS]`
(`SVertexTemp`, `:780-791`) with parallel `GoreIndexCopy`/`GoreIndecies`
(`MAX_GORE_INDECIES=6000`), rebuilt per `G2_GorePolys` (`:804`) call, and a
**persistent** `GoreTouch` generation counter (`:795`, init 1) bumped every call
(`:890`) and compared against each vertex's `touch` stamp (`:938,944`) to
invalidate stale scratch across calls. `G2_GorePolys` is reached from the
in-scope collision path `G2_TraceModels` (`:1514`, `#ifdef _G2_GORE`, no DEDICATED
guard), so `GoreTouch++` runs server-side on every trace; the vert buffers do
real work only when `TS.gore` is set — which happens via `G2API_AddSkinGore`
(`G2_API.cpp:2569`; `G2_TransformModel(...,true) :2598` + `G2_TraceModels
(...,&gore,qtrue) :2601`, using `G2VertSpaceServer`, no DEDICATED guard).
`G2API_AddSkinGore` is **not** a `SV_GameSystemCalls` arm (grep: no gore token in
`sv_game.cpp` beyond the `G2_gore.h` include `:20`; no `G_G2_*` gore trap in
`g_public.h`) — and its only trap is the client `CG_G2_ADDSKINGORE`
(`cg_public.h:280`), so server-side no caller ever sets `TS.gore` and the vert
buffers never do real work (`G2SV-Q4`, resolved). `GoreTouch++` still runs via the
collision path independently.

**Ruling 22's apply/record split, corrected by ruling 26 (`G2SV-D7`/`G2SV-D11`,
resolving `G2SV-Q4`/`G2SV-Q8`).** The gore surface splits into a graph-dead *apply*
entry set and a server-live *record-store* infra set:
- **Graph-dead server-side (zero reachability from engine roots → §20 zero-caller
  notes):** `G2API_AddSkinGore` (`G2_API.cpp:2569`; only the client
  `CG_G2_ADDSKINGORE` trap, `cg_public.h:280`, no `G_G2_*` arm), `ResetGoreTag`
  (`G2_misc.cpp:96`; sole caller is `G2API_AddSkinGore` at `:2590`), and
  `G2_GetGoreRecord` (`G2_misc.cpp:113`; **no** caller anywhere in `codemp/`).
  These three — and only these three — are dropped with §20 zero-caller notes
  (ruling 26 narrowed the apply set to exactly this trio, `G2SV-D11`).
- **Server-live (ports fully):** the record store `AllocGoreRecord`
  (`G2_misc.cpp:58`), `FindGoreRecord` (`:103`), `DeleteGoreRecord` (`:118`),
  `DestroyGoreTexCoordinates` (`:43`, the private helper `DeleteGoreRecord` calls
  at `:120`), `FindGoreSet` (`:127`), `NewGoreSet` (`:142`), `DeleteGoreSet`
  (`:153`), `CGoreSet::~CGoreSet` (`:174`), `G2API_ClearSkinGore`
  (`G2_API.cpp:2549`), and `G2_GorePolys` (`G2_misc.cpp:804`). `G2API_ClearSkinGore`
  is reached from the live `G2API_CleanGhoul2Models` (`G2_API.cpp:496`, its call at
  `:545`) behind the `G_G2_CLEANMODELS` trap (`g_public.h:529`) and from
  `G2API_LoadSaveCodeDestructGhoul2Info` (`:2493`); the record store's
  `DeleteGoreSet` is additionally reached **directly** from the
  `G_G2_REMOVEGHOUL2MODEL`/`G_G2_REMOVEGHOUL2MODELS` removal paths
  (`G2_API.cpp:814,901`, which call `DeleteGoreSet`, not `ClearSkinGore`).
  `ClearSkinGore` drives `DeleteGoreSet` (`:2557`) → `~CGoreSet` (`:174` → `delete`
  at `G2_misc.cpp:163`) → `DeleteGoreRecord` (`:179`) → `DestroyGoreTexCoordinates`
  (`:120`); `G2_GorePolys` is reached from the collision `G2_TraceModels` loop
  (`G2_misc.cpp:1494`, `#ifdef _G2_GORE`, no DEDICATED guard).
- **Ruling 26 (`G2SV-Q8` SETTLED, `G2SV-D11`).** Ruling 22 as first transcribed
  mislisted `DestroyGoreTexCoordinates` (`G2_misc.cpp:43`) — and, implicitly, its
  caller `DeleteGoreRecord` — in the graph-dead apply set. Oracle ground truth shows
  both are **reached** server-side via `~CGoreSet` (`:174`, its `DeleteGoreRecord`
  call at `:179`) ← `DeleteGoreSet` ← the live `G2API_ClearSkinGore` (and the direct
  `REMOVEGHOUL2MODEL`/`MODELS` paths). Both compile (`#ifdef _G2_GORE`, ON, `:26`).
  The prior mislisting was an **engineorder graph blind spot**: the reachability
  tool does not treat an implicit C++ destructor invocation (`~CGoreSet` running on
  `delete`, `G2_misc.cpp:163`) as a call edge, so the whole `~CGoreSet →
  DeleteGoreRecord → DestroyGoreTexCoordinates` chain read as unreachable. Ruling 26
  moves both to the LIVE bucket; they port with the record store in `gore/gore_set.rs`.

Separately `tr_ghoul2.cpp:866-867`
holds a render-surface pool `RSStorage[MAX_RENDER_SURFACES=2048]` with a rolling
`NextRS` cursor handed out by `AllocRS()` (`:869`); its sole caller (`:2660`) is
inside `#ifndef DEDICATED` (`:2520-2736`), so the pool is dead in the DEDICATED
build. (The `#else` non-`_G2_GORE` second `GoreVerts` at `:1088` is dead —
`_G2_GORE` is ON.)

**Const bone-name tables.** `tr_ghoul2.cpp` carries read-only bone-name lookup
tables for the skeleton/remap helpers: `rootParents`/`otherParents`/`bottomBones`
(`:5061-5097`) and the null-terminated `BoneHierarchyList` (`:5173`), plus
`OldToNewRemapTable[72]` (`:4469`, declared non-`const` `int[]` but only ever
read, `:5034`). Like `identityMatrix` (`:128`) these are the const-table kind of
the three-kind rule.

**The RagDoll + IK solver (the fn-statics) — server-live (`G2SV-D3`).**
`G2_bones.cpp:1214-1241` is one block of file-scope statics, sized
`MAX_BONES_RAG=256` (`:1163`), that the solver reuses across the multi-pass
settle: the parallel per-bone arrays `ragBasepose`, `ragBaseposeInv`, `ragBones`,
`ragEffectors` (`SRagEffector`, `:1165`), `ragBoneData`, `tempDependents`,
`ragBlistIndex`; the scalars/vectors `numRags`, `ragBoneMins`/`ragBoneMaxs`/
`ragBoneCM`, `haveDesiredPelvisOffset`, `desiredPelvisOffset`, `ragOriginChange`,
`ragOriginChangeDir`, `handPos`, `handPos2`, `ragState`; and
`vector<boneInfo_t*> rag` (`:1241`). These are read and written across
`G2_RagDollSetup` (`:2254`), `G2_RagDoll` (`:2403`), `G2_RagDollCurrentPosition`
(`:2609`), `G2_RagDollSettlePositionNumeroTrois` (`:2927`/`:3449`),
`G2_RagDollSolve` (`:3970`), and the IK arm (`G2_IKSolve` `:4297`, `G2_DoIK`
`:4453`) — cross-call, cross-frame state, not per-invocation scratch. The 12
ragdoll/IK entry points (`G2_local.h:190-204`, incl. `G2API_AbsurdSmoothing`
`:190` and `G2API_AnimateG2Models` `:194`) are all `SV_GameSystemCalls` arms
(`sv_game.cpp:1497,1509,1532,1554,1561,1563,1565,1567,1569,1571,1574,1576`), so the solver
runs server-side (`G2SV-D3`). (Additional `static const` matrices and `static`
locals inside individual solver functions — e.g. the identity `id` at `:1423`,
the settle-pass locals at `:3452-3475` — are the const-table / scratch kinds of
the three-kind rule, not persistent state.)

**The `cgvm` ragdoll-callback dead branches (`G2_bones.cpp`).** The solver
functions ported wholesale to `ragdoll.rs` embed a family of client-game-VM
callback branches — the file `#include`s `client.h` "only if the cgvm exists"
(`G2_bones.cpp:32`) — that all fold to their server arm in the WinDed DEDICATED
build, exactly like `HackadelicOnClient`/`RSStorage`/`g2ClientAttachments`
elsewhere. A porter folds them per §C10/§20; none is threaded state:
- `Rag_Trace` (`:2684`) wraps its `if (cgvm) { VM_Call(cgvm, CG_RAG_CALLBACK,
  RAG_CALLBACK_TRACELINE) }` client-callback trace in `#ifndef DEDICATED`
  (`:2688-2704`); under DEDICATED only the `#else`/fall-through arm compiles, so
  server-side `Rag_Trace` unconditionally does the real `CM_BoxTrace` (`:2709`).
  (The `#ifdef _DEBUG` `ragTraceTime` timing at `:2685,:2710` also drops under
  NDEBUG.)
- `G2_BoneSnap` (`:3951`) is `#ifdef DEDICATED return; #else …cgvm… #endif` — a
  compiled **no-op** server-side; its sole caller is `G2_RagDollSolve` (`:4244`).
- The four `RAG_CALLBACK_BONEINSOLID` callback sites inside
  `G2_RagDollSettlePositionNumeroTrois` (`:3056,:3085,:3180,:3217`) are each
  `#ifndef DEDICATED { if (cgvm) { … VM_Call … } } #endif`; they compile out
  server-side (the surrounding `if (params)` solid-handling stays, only the cgame
  callback drops). A fifth (`:3826`) is doubly dead — `#if 0` **and** `#ifndef
  DEDICATED`.
- `G2_RagDebugBox`/`G2_RagDebugLine` (`:2884,:2905`) are compiled (their
  `_DEBUG_BONE_NAMES` guard is a source `#define` at `:2577`) but each is `#ifdef
  DEDICATED return; #else …cgvm… #endif`, so both are compiled **no-ops**
  server-side; their callers (`:3516,3518,3676-3688`) sit in the same
  `_DEBUG_BONE_NAMES` region.
Because every `cgvm` use is behind a `#ifdef`/`#ifndef DEDICATED` guard, no `cgvm`
global resolves in this build — there is nothing to port. Treatment: divergences
list.

**API-level globals (`G2_API.cpp`).** `g2ClientAttachments[MAX_GENTITIES]`
(`:197`, `#ifdef _G2_LISTEN_SERVER_OPT` — compiles OUT, `G2SV-D4`), `G2TimeBases`
(`int[NUM_G2T_TIME]`, `:160`, driving `G2API_SetTime`/`GetTime` `:162`),
`gG2_GBMNoReconstruct`/`gG2_GBMUseSPMethod` (`:1724-1725`, `GetBoltMatrix`
reconstruct-skip flags), plus debug-only `g_Ghoul2Allocations`/`g_G2ServerAlloc`/
`g_G2ClientAlloc`/`g_G2AllocServer` (`:34-37`), `g_G2AllocTrack`/
`g_G2AllocTrackInit` (`:43-44`, `MAX_TRACKED_ALLOC=4096`) — all under
`_FULL_G2_LEAK_CHECKING` — and the `_DEBUG`-only `g_goreAllocs`/`g_goreTexAllocs`
(`G2_misc.cpp:138-139`).

## State ownership

Every global the survey found. Owner placement follows ruling 2 (globals →
`Engine` sub-structs grouped by owning `.c` file) and ruling 3 (fn-statics → the
three-kind rule). Per ruling 12 the subsystem state is **one plain
Default-initialized direct field `Engine.g2: Ghoul2System`** (no `Option`/`Box`/
nesting; lazy-init modeled with Raven's own initialized flags). There is **no
separate `RenderG2State`** — the render-side bone state (`mBoneCache`,
`goreShader`) folds into `Ghoul2System` because ruling 12 fixes the §F subsystem
count at five (`icarus`, `nav`, `g2`, `roff`, `rmg`) with a single `g2`. Services
(trace, print/error, file read/write, `flrand`, loader model-memory read, cvar read)
cross the **one BUILT, frozen 15-method `EngineHost` trait** (rulings 11/31/36,
`crates/mp/host-interface`, package `mp_host_interface`, `G2SV-D14`/`G2SV-D15`;
signatures quoted verbatim in `## Seam definition`); §F methods take `(&mut
Ghoul2System, &mut impl EngineHost)`, and `Engine` implements `EngineHost` via a
split-borrow view struct. The two services pass 5 left absent — the loader
model-memory read (the parsed `.glm`/`.gla` blocks) and cvar read
(`cg_g2MarksAllModels`) — are served by ruling 36's `model_mdxm`/`model_mdxa` and
`cvar_integer` (`G2SV-D15`), so the cvars / model-memory rows below reach the host,
not an open question (`G2SV-Q11` SETTLED, Resolved questions).

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `Ghoul2InfoArray *singleton` | `G2_API.cpp:477` | `mp_engine_ghoul2::Ghoul2System.info_array: Ghoul2InfoArray` | lazily on first `the_info_array()`; freed by `Ghoul2InfoArray_Free` | `&mut Ghoul2System` into every `G2API_*` |
| `g2ClientAttachments[MAX_GENTITIES]` | `G2_API.cpp:197` (`_G2_LISTEN_SERVER_OPT`) | **dropped** — compiles out in the WinDed set (`G2SV-D4`, §20-note) | — | — (divergences list) |
| `G2TimeBases[NUM_G2T_TIME]` | `G2_API.cpp:160` | `Ghoul2System.time_bases: [i32; NUM_G2T_TIME]` | `Ghoul2System::default` | `&mut Ghoul2System` |
| `gG2_GBMNoReconstruct`, `gG2_GBMUseSPMethod` | `G2_API.cpp:1724-1725` | `Ghoul2System.gbm_no_reconstruct/gbm_use_sp_method: bool` | `Ghoul2System::default` | `&mut Ghoul2System` |
| `g_Ghoul2Allocations`, `g_G2ServerAlloc`, `g_G2ClientAlloc`, `g_G2AllocServer`, `g_G2AllocTrack*`, `g_G2AllocTrackInit` (all `_FULL_G2_LEAK_CHECKING`); `g_goreAllocs`, `g_goreTexAllocs` (`_DEBUG`) | `G2_API.cpp:34-37,43-44`, `G2_misc.cpp:138-139` | dropped (debug alloc tracking, no parity surface, §F20) | — | — (divergences list) |
| `G2Time_*`, `G2PerformanceCounter_*`, `G2PerformanceTimer_*` (`G2_PERFORMANCE_ANALYSIS`, ON) | `tr_ghoul2.cpp:42-62` | dropped (timing instrumentation, no parity surface, §F20 — same treatment as the leak-checking globals) | — | — (divergences list) |
| `GoreRecords`, `GoreTagsTemp`, `CurrentTag`, `CurrentTagUpper` | `G2_misc.cpp:35,36,32,33` | `Ghoul2System.gore: GoreState { records: BTreeMap<i32, GoreTextureCoordinates>, tex_buffers, tags_temp, current_tag, current_tag_upper }` — per `G2SV-D13`(c) / ruling 29 `GoreState` **owns** each per-LOD gore buffer as a `Vec<f32>` (`tex_buffers`, keyed by record tag + LOD) and the frozen `GoreTextureCoordinates.tex: [*mut c_float; MAX_LODS]` pointers point INTO those `Vec`s; alloc at `AllocGoreRecord`/`G2_GorePolys` (`G2_misc.cpp:1020`), free/teardown order mirrors `Z_Free` at `DestroyGoreTexCoordinates` (`G2_gore.h:25-36`). Closes `G2SV-Q10` | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreSets`, `CurrentGoreSet` | `G2_misc.cpp:125,124` | `GoreState.sets: BTreeMap<i32, CGoreSet>`, `GoreState.current_set` | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreTouch` (persistent gen counter) | `G2_misc.cpp:795` | `GoreState.gore_touch: i32` (ruling 2, same file/subsystem; three-kind persistent). Runs server-side via the collision path (`:890`); gore-*apply* has no server caller (`G2SV-D7`) | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreVerts`, `GoreIndexCopy`, `GoreIndecies` | `G2_misc.cpp:793,794,798` | scratch buffers (three-kind scratch; per-`G2_GorePolys` rebuild, invalidated by `gore_touch`) — impl-local, not a global; never server-driven (`G2SV-D7`: no `AddSkinGore` server trap), transcribed faithfully, goldens optional | — | — |
| `goreModelIndex` | `G2_misc.cpp:38` | scratch (three-kind scratch; set in the `G2_TraceModels` model loop `:1539`, read as the `GoreTagsTemp` key `:959,1000`) — impl-local, threaded through the trace, not a global | — | — |
| `cg_g2MarksAllModels` | `G2_misc.cpp:40` | **no stored owner** — the frozen `cvar_integer(name) -> i32` is a stateless by-name read with no `Cvar_Get`/handle-registration step (built trait, `## Seam definition`), so there is no cvar handle to own and `Ghoul2System` carries **no** `cvars` field; the Raven `cvar_t*` global becomes a read on demand, exactly as the renderer-owned `broadsword` row below (`G2SV-D15`-forced reconciliation of the pre-ruling-36 "ruling-2 EngineCvars sub-struct" convention, **not** a new decision; the ruling-2 sub-struct convention applies only to globals that become owned state, and a stateless host read is none) | — (not stored; read on demand) | `&mut impl EngineHost` — `EngineHost::cvar_integer("cg_g2MarksAllModels")` at each call site (read at `G2_misc.cpp:1524`, a missing name reads 0 matching `Cvar_VariableIntegerValue`, `cvar.cpp:118-124`) — `G2SV-Q11`(d) SETTLED |
| `broadsword`, `broadsword_kickbones`, `broadsword_kickorigin`, `broadsword_dontstopanim`, `broadsword_waitforshot`, `broadsword_playflop`, `broadsword_effcorr`, `broadsword_ragtobase`, `broadsword_dircap`, `broadsword_extra1`… (the ragdoll cvar family) | `extern` in `G2_bones.cpp:1176-1189`; defined + `Cvar_Get`-registered in `renderer/tr_init.cpp:204-216,1164-1175` | **not ghoul2-owned** — renderer-owned (ruling 2, by owning file `tr_init.cpp`), so owned by the separate `tr_model` subsystem doc (non-goals); ghoul2 only **reads** them | the RagDoll solver reads them server-side (`G2_SetRagDoll` gate `:1628`; the settle/IK passes); the read is a cvar host service, served by ruling-36 `EngineHost::cvar_integer` (`G2SV-D15`) — a missing name reads 0, so ghoul2 reads the renderer-owned `broadsword` family **without** a `mp_renderer` edge (`G2SV-D5`); `G2SV-Q11`(d) SETTLED | `&mut impl EngineHost` (`cvar_integer`) |
| RagDoll fn-statics block (`ragBasepose`…`rag`) | `G2_bones.cpp:1214-1241` | `Ghoul2System.rag: RagDollSolver { bones: [mdxaBone_t; MAX_BONES_RAG], effectors, temp_dependents, blist_index: [i32; MAX_BONES_RAG], num_rags, bone_mins/maxs/cm, desired_pelvis_offset, have_desired_pelvis_offset, origin_change, origin_change_dir, hand_pos, hand_pos2, rag_state, rag: Vec<i32> }` — per `G2SV-D13`(b) / ruling 29 the raw-pointer arrays are **replaced by blist indices + per-call `EngineHost` resolution**, closing `G2SV-Q9`: `ragBoneData` (`boneInfo_t*`, `:1218`) and `rag` (`vector<boneInfo_t*>`, `:1241`) become `mBlist` **indices** (Raven already carries the parallel `ragBlistIndex[MAX_BONES_RAG]` `int` array `:1220`), resolved against the live model's `mBlist` at use; `ragBasepose`/`ragBaseposeInv` (`mdxaBone_t*`, `:1214-1215`) are **not stored** — the basepose/baseposeInv matrices are resolved per call via `G2_GetBoneMatrixLow` over `EngineHost` (`G2_bones.cpp:2622`, write-through pattern, ruling 21), so no raw pointer outside the ABI seam (porting-rules §B5/§D11). The basepose resolve reads model memory through ruling-36 `EngineHost::model_mdxa` (`G2SV-D15`; `G2SV-Q11`(a) SETTLED). The solver also consumes `flrand` server-side (ragdoll init `:1468-1469` + settle/IK `:2127-2129,3327,3925,4290`) — routed via `EngineHost::flrand` (ruling 21, a frozen method, so a service not owned state) — and reads the renderer-owned `broadsword` cvar family via `EngineHost::cvar_integer` (row below, `G2SV-D15`) | `Ghoul2System::default` | `&mut Ghoul2System.rag` (ruling 3 cross-frame kind, `G2SV-D3`) |
| solver `static const` matrices / settle-pass `static` locals | `G2_bones.cpp:1423,3452-3475` | `const` items / function locals (three-kind rule: const-table / scratch) | — | — |
| `CBoneCache *mBoneCache` per instance | `ghoul2_shared.h:265` | `Ghoul2System.bone_caches` — a hand-rolled owned in-crate generational arena of `CBoneCache` keyed by `BoneCacheId` (§B5 arena, same kind as `Ghoul2InfoArray`; **not** an external `slotmap` crate — `G2SV-D9`, zero workspace precedent, container shape free per §A1), folded from the former RenderG2State per ruling 12; `CGhoul2Info.mBoneCache` → `Option<BoneCacheId>` | `G2_ConstructGhoulSkeleton` on demand; freed by the **`Ghoul2System`-level** `delete`/`delete_low` teardown (`G2SV-D13`(a) / ruling 29: `DeleteLow`'s `RemoveBoneCache` loop `G2_API.cpp:319-326` moves UP to `Ghoul2System` because the sibling `bone_caches` arena is unreachable from `Ghoul2InfoArray` alone) | `&mut Ghoul2System` |
| `worldMatrix`, `worldMatrixInv` | `tr_ghoul2.cpp:136-137` | per-construct scratch threaded through the skeleton build (three-kind: scratch), NOT a global | set by `G2_GenerateWorldMatrix` | passed into the transform chain |
| `identityMatrix` | `tr_ghoul2.cpp:128` | `const` item | — | — |
| `rootParents`, `otherParents`, `bottomBones`, `BoneHierarchyList`, `OldToNewRemapTable` | `tr_ghoul2.cpp:5061-5097,5173,4469` | `const` items (three-kind const-table, as `identityMatrix`; `OldToNewRemapTable` is decl'd non-`const` but read-only, `:5034`) | — | — |
| `HackadelicOnClient` (render-traversal flag) | `tr_ghoul2.cpp:104` | none — const-`false` server-side (only writers in `R_AddGhoulSurfaces`, `#ifndef DEDICATED` `:3384-3537`); reads take the false arm (§C10 fold) | — | — (divergences list) |
| `RSStorage`, `NextRS` (render-surface pool) | `tr_ghoul2.cpp:866-867` | dropped — dead server-side (`AllocRS` sole caller `:2660` is `#ifndef DEDICATED`) | — | — (divergences list) |
| `goreShader` | `tr_ghoul2.cpp:139` (`_G2_GORE`) | `Ghoul2System.gore_shader: qhandle_t` (folded from the former RenderG2State per ruling 12) | render init (loaded via `EngineHost`) | `&mut Ghoul2System` |

## Seam definition

Per doc-standards rule 5 the pub signatures freeze here; porters transcribe into
them without changing them. Per ruling 11 every §F entry takes the subsystem
state plus the services trait — `(g2: &mut Ghoul2System, host: &mut impl
EngineHost, ...)`; `EngineHost` is the BUILT, frozen trait in
`crates/mp/host-interface` (package `mp_host_interface`, Stage-0 commit `4b7f01b0` +
ruling-36 extension commit `a9820853`, `G2SV-D14`/`G2SV-D15`). `host` provides trace,
print/error, `VM_Call`, FS read/free/write, the shared-memory window, `flrand`/`irand`,
`gentity`, the loader model-memory accessors (`model_mdxm`/`model_mdxa`), `cvar_integer`,
and `sv_time`. There is **no `RenderG2State` parameter** (folded into `Ghoul2System`,
ruling 12) and **no `mp_renderer` crate dependency** (`G2SV-D5`); the model-memory
types stay out of these signatures — ruling 36's `model_mdxm`/`model_mdxa` return
`*mut c_void`, never an `mp_renderer` struct (`G2SV-D5`/`G2SV-D15` type-location
reconciliation, non-goals). The two services pass 5 escalated — the loader model-memory
read and cvar read — are now **served** by ruling 36 (`G2SV-Q11` SETTLED), so no gap
remains.

**The frozen `EngineHost` trait, quoted verbatim** (rulings 11/31/36,
`G2SV-D14`/`G2SV-D15`) from `crates/mp/host-interface/src/engine_host.rs` so this doc
is self-contained. **Fifteen** methods (ruling 36 extended the Stage-0 ten with
`cvar_integer`, `sv_time`, `fs_write_file`, `model_mdxm`, `model_mdxa`);
dyn-compatible (ruling 24: no generics, no by-value `Self` returns). Doc comments
elided; each `Source:` cite is the Raven host function transcribed.

```rust
// crates/mp/host-interface/src/engine_host.rs — trait EngineHost (verbatim signatures)
pub trait EngineHost {
    // Raven SV_Trace — oracle/codemp/server/sv_world.cpp:803
    #[allow(clippy::too_many_arguments)]
    fn trace(&mut self, results: &mut trace_t, start: &vec3_t, mins: &vec3_t,
        maxs: &vec3_t, end: &vec3_t, pass_entity_num: i32, contentmask: i32,
        capsule: bool, trace_flags: i32, use_lod: i32);
    // Raven FS_ReadFile — oracle/codemp/qcommon/files.cpp:1670 (None = -1/NULL)
    fn fs_read_file(&mut self, qpath: &str) -> Option<Vec<u8>>;
    // Raven FS_FreeFile — oracle/codemp/qcommon/files.cpp:1798 (default: drop)
    fn fs_free_file(&mut self, _buffer: Vec<u8>) {}
    // Raven Com_Printf — oracle/codemp/qcommon/common.cpp:128
    fn print(&mut self, msg: &str);
    // Raven Com_Error — oracle/codemp/qcommon/common.cpp:249 (panic + catch_unwind)
    fn error(&mut self, code: errorParm_t, msg: &str) -> !;
    // Raven VM_Call(vm, callnum, ...) — oracle/codemp/qcommon/vm.cpp:787
    fn vm_call(&mut self, vm: VmSlot, callnum: i32, args: &[isize]) -> isize;
    // Raven sv.mSharedMemory — oracle/codemp/server/server.h:87
    fn shared_memory(&mut self) -> *mut c_char;
    // Raven Q_flrand — oracle/codemp/game/q_math.c:1451
    fn flrand(&mut self, min: f32, max: f32) -> f32;
    // Raven Q_irand — oracle/codemp/game/q_math.c:1471
    fn irand(&mut self, min: i32, max: i32) -> i32;
    // Raven SV_GentityNum — oracle/codemp/server/sv_game.cpp:54
    fn gentity(&mut self, ent_num: i32) -> *mut sharedEntity_t;
    // Ruling 36 — Raven cached cvar_t->integer read; oracle/codemp/qcommon/cvar.cpp:118-124
    // (cg_g2MarksAllModels G2_misc.cpp:40 read :1524; unregistered name reads 0)
    fn cvar_integer(&mut self, name: &str) -> i32;
    // Ruling 36 — Raven svs.time frame clock; oracle/codemp/server/server.h:211
    fn sv_time(&mut self) -> i32;
    // Ruling 36 — Raven FS_FOpenFileByMode(FS_WRITE)+FS_Write+FS_FCloseFile;
    // oracle/codemp/server/NPCNav/navigator.cpp:670-699
    fn fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool;
    // Ruling 36 — Raven R_GetModelByHandle(model)->mdxm (parsed .glm block, c_void
    // per G2SV-D5); oracle/codemp/renderer/tr_local.h:1128, chain G2_API.cpp:2716-2721
    fn model_mdxm(&mut self, model: qhandle_t) -> *mut c_void;
    // Ruling 36 — Raven R_GetModelByHandle(model)->mdxa (parsed .gla block, c_void;
    // byte arithmetic off it at tr_ghoul2.cpp:416-421); tr_local.h:1129, chain :2735-2739
    fn model_mdxa(&mut self, model: qhandle_t) -> *mut c_void;
}
```

Of these, ghoul2 uses `trace` (collision/gore/ragdoll — `G2_TraceModels`,
`G2_GorePolys`, `Rag_Trace`'s `CM_BoxTrace` `G2_bones.cpp:2709`), `print`/`error`,
`fs_read_file`/`fs_free_file`/`fs_write_file` (save/load), `flrand`, `model_mdxm`/
`model_mdxa` (loader model memory — `CBoneCache::new` parent-seeding, `render/
skeleton.rs`, ragdoll basepose `G2SV-D13`(b)), and `cvar_integer`
(`cg_g2MarksAllModels` + the renderer-owned `broadsword` family). The `flrand` use is
server-live: `G2API_SetRagDoll` → `G2_SetRagDoll` (`G2_bones.cpp:1622`)
→ `G2_Set_Bone_Angles_Rag` (`:1855`+ per PCJ bone, no DEDICATED guard) calls bare
`flrand` (`:1468-1469`, the `#else` live arm of the `:1450` `#if 0`; the `#define
flrand Q_flrand` at `:1212` is **commented out**, so it is the real `flrand`,
`q_math.c:1441`, the `holdrand` LCG that `Q_flrand`/`EngineHost::flrand` `:1451`
forwards to) to seed each PCJ bone's initial angle at ragdoll init, and again in the
settle/IK passes (`:2127-2129,3327,3925,4290` — all `flrand`, verified). Ruling 21
routes it through `EngineHost::flrand` — **so `flrand` IS consumed by ghoul2**
(correcting any earlier prose that lumped it with the unused seam methods), and every
§F ragdoll entry that reaches it takes `host` (see the `g2api_set_ragdoll` signature
note below, ruling 36). Only `vm_call`/`shared_memory`/`irand`/`gentity` are **not**
consumed server-side by ghoul2 (`irand` genuinely unused — grep: no `irand` token in
`codemp/ghoul2/`; `vm_call`/`shared_memory` appear only in the `cgvm` callback arms that
are DEDICATED-dead, `G2_bones.cpp:2691,2700` etc.); they serve the other §F subsystems.
The formerly-absent model-memory-read and cvar methods are now `model_mdxm`/`model_mdxa`
and `cvar_integer` (ruling 36, `G2SV-D15`; `G2SV-Q11` SETTLED), consumed as listed
above.

**ABI-crossing / already-ported types (imported, never re-declared).** `#[repr(C)]`
layout-frozen, grouped by where the port already owns them — a porter `use`s each,
never re-declares, and **no new crate edge** is added beyond the existing
`mp_qshared` dep (`Cargo.toml`, the only dependency `mp_engine_ghoul2` carries):
- **In this crate already** (`crates/mp/engine/ghoul2/src/shared/` + `src/gore/`,
  the "already type-ported" set the Standing context lists; reached as `crate::…`):
  `CGhoul2Info_v` (the 4-byte arena **handle**, `mItem: i32`,
  `shared/cghoul2_info_v.rs` — **not** the per-instance class), `boneInfo_t`,
  `boltInfo_t`, `surfaceInfo_t`, `EG2_Collision`, and the `gore/` types
  (`SSkinGoreData`, `CRagDollParams`, `SRagDollEffectorCollision`,
  `GoreTextureCoordinates`, `SGoreSurface`).
- **Via the existing `mp_qshared` dependency — no direct `native_types` edge
  needed.** `qhandle_t`, `mdxaBone_t`, and `vec3_t` are `native_types`-owned
  (`crates/native/types/src/lib.rs`; `vec3_t = [vec_t; 3]` in
  `crates/native/math/src/vector.rs:12`, type-rosetta) but `mp_qshared`
  **re-exports** them (`crates/mp/qshared/src/shared/mod.rs:133` for `vec3_t`,
  `:137` for `qhandle_t`/`mdxaBone_t`), so a porter writes `use
  mp_qshared::shared::{qhandle_t, mdxaBone_t, vec3_t}` and adds nothing to
  `Cargo.toml`. **`vec3_t` is the type spelled in every §F angles/position/scale/
  velocity parameter and in `RagDollUpdateParams`' vector fields** — the frozen
  signatures use the already-ported `vec3_t` array alias (`[f32; 3]`) verbatim (it
  is the Raven `vec3_t` those params carry); they do **not** introduce a new
  `Vec3` wrapper struct (none exists in the tree and none is on the roster). Any
  ergonomic wrapper would be a new type + roster row + decision, which no settled
  ruling authorizes — so `vec3_t` it is.
  `CollisionRecord_t` (and `cplane_t`) are `mp_qshared`-owned directly in
  `crates/mp/qshared/src/shared/collision.rs` and re-exported from `shared`
  (`mod.rs:68`; `mEntityNum == -1` = unused record). The aggregate
  `G2Trace_t = [CollisionRecord_t; MAX_G2_COLLISIONS]` and `MAX_G2_COLLISIONS =
  16` live in a **different** module,
  `crates/mp/qshared/src/common/mp/qcommon/collision_record.rs:19,21` (which
  re-imports `CollisionRecord_t` from `shared`), and are **not** re-exported from
  `shared::collision` — a porter needing those two names imports them from that
  path. The §F `g2api_collision_detect` returns `Vec<CollisionRecord_t>` (the
  populated `collRecMap` entries), so it needs only `CollisionRecord_t`, not
  `G2Trace_t`.
- **NOT importable into this crate** (`mp_renderer`-owned,
  `crates/mp/renderer/src/mdx_format/`; `G2SV-D5` forbids the crate edge):
  `mdxaHeader_t`/`mdxaSkel_t`. They are **never named as Rust types** in
  `mp_engine_ghoul2` — the bone ctor/skeleton path reads their fields only through
  `EngineHost` accessors (non-goals, type-location reconciliation), so no
  `mp_engine_ghoul2` → `mp_renderer` dependency is ever added.

**Not** already ported (this doc defines them,
see the roster): the per-instance **class** `CGhoul2Info` (`ghoul2_shared.h:240`,
a §F idiomatic reimplementation with owned `Vec`s — only the handle `CGhoul2Info_v`
above is layout-frozen) and `CRagDollUpdateParams` (`G2_gore.h:94`, a virtual-method
C++ class distinct from the already-ported plain-data `sharedRagDollUpdateParams_t`
— reimplemented as the §F17 `RagDollUpdateParams` enum, `G2SV-D8`, owned by
`ragdoll_update_params.rs`). The ragdoll pointer members of
`boneInfo_t` (`basepose`, `baseposeInv`, `baseposeParent`, `baseposeInvParent`,
`ghoul2_shared.h:109-112`) keep their ported repr (they are inside an ABI struct);
the transform code fills them from `Ghoul2System`-owned matrices.

**Traps.** This subsystem crosses no `trap_*`/syscall boundary itself — it is
engine-internal C++ reached from `SV_GameSystemCalls` (`sv_game.cpp`) which
already sits above the interface-crate seam (ruling 5, plan §"VM dispatch"). The
consumer surface is the `G2API_*` free-function set (`G2_local.h:96-224`), kept
**1:1 in signature** (`G2SV-D6`) because those names are the switch targets the
server calls. Illustrative frozen signatures (Rust idiom per porting-rules §C7:
out-params → returns, `qboolean` → `bool` — **except** where Raven's contract is
write-through, `G2SV-D1`):

```rust
// mp_engine_ghoul2 — the syscall-switch target surface (G2SV-D6, 1:1 with G2_local.h)
pub fn g2api_init_ghoul2_model(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, file_name: &str, model_index: i32,
    custom_skin: qhandle_t, custom_shader: qhandle_t, model_flags: i32,
    lod_bias: i32) -> i32;
pub fn g2api_remove_ghoul2_model(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32) -> bool;
pub fn g2api_set_bone_anim(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32,
    bone_name: &str, start_frame: i32, end_frame: i32, flags: i32, anim_speed: f32,
    current_time: i32, set_frame: f32, blend_time: i32) -> bool;
pub fn g2api_set_bone_angles(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, model_index: i32, bone_name: &str, angles: vec3_t,
    flags: i32, up: Eorientations, left: Eorientations, forward: Eorientations,
    model_list: &[qhandle_t], blend_time: i32, current_time: i32) -> bool;
pub fn g2api_add_bolt(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32, bone_name: &str) -> i32;
// G2SV-D1 (ruling 18): write-through + qboolean, EXACTLY like Raven's
// `qboolean G2API_GetBoltMatrix(..., mdxaBone_t *matrix)` — the out-matrix is
// ALWAYS written (failure paths write the identity/fallback too), NOT Option.
pub fn g2api_get_bolt_matrix(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, model_index: i32, bolt_index: i32, angles: vec3_t,
    position: vec3_t, frame_num: i32, model_list: &[qhandle_t], scale: vec3_t,
    bolt_matrix: &mut mdxaBone_t) -> bool;   // out-matrix write-through, qboolean return
pub fn g2api_collision_detect(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, angles: vec3_t, position: vec3_t, frame_number: i32,
    ent_num: i32, ray_start: vec3_t, ray_end: vec3_t, scale: vec3_t, trace_flags: i32,
    use_lod: i32, f_radius: f32) -> Vec<CollisionRecord_t>; // populated collRecMap entries (mEntityNum != -1)
// Takes `host` (frozen by ruling 36, G2SV-D15): `G2_SetRagDoll` (G2_bones.cpp:1622) →
// `G2_Set_Bone_Angles_Rag` (:1855+ per PCJ bone, no DEDICATED guard) UNCONDITIONALLY
// calls `flrand` (:1468-1469) server-side to seed each PCJ bone's initial angle; ruling
// 11/21 route that through `EngineHost::flrand` (a frozen method), and a top-level free
// fn has no ambient state to source `host` from, so it is a parameter — like
// `g2api_animate_g2_models_rag` below. The completed body ALSO reads model memory
// (`G2_GetModA`, :1645) and the renderer-owned `broadsword` cvar (:1628) — now served by
// ruling-36 `model_mdxa`/`cvar_integer` (G2SV-Q11 SETTLED), so the whole body
// transcribes now, not just the signature.
pub fn g2api_set_ragdoll(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, parms: &mut CRagDollParams);
pub fn g2api_animate_g2_models_rag(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, a_current_time: i32, params: &mut RagDollUpdateParams); // §F17 enum, G2SV-D8
// CRagDollUpdateParams (G2_gore.h:94) as a §F17 enum (G2SV-D8): the six data
// members + a single-variant kind. MP instantiates only the base (sv_game.cpp:1539);
// the four virtuals (:106-123) are no-op base bodies, so Server's hooks do nothing.
// SP's two subclasses (code/, out of scope) would add variants as a DEC-04 diff.
pub struct RagDollUpdateParams {
    pub angles: vec3_t, pub position: vec3_t, pub scale: vec3_t, pub velocity: vec3_t,
    pub me: i32, pub settle_frame: i32,     // G2_gore.h:97-103
    pub kind: RagDollUpdateKind,
}
pub enum RagDollUpdateKind { Server }       // the base no-op hooks; the sole MP variant
// params->RagDollSettled() (G2_bones.cpp:2505) ⇒ match params.kind { Server => {} }
// G2API_AddSkinGore is graph-dead server-side (G2SV-D7): only the client
// CG_G2_ADDSKINGORE trap, no G_G2_* arm → §20 zero-caller note, NOT a live seam fn.
// Raven: `qboolean G2API_OverrideServerWithClientData(CGhoul2Info *serverInstance)` — a
// SINGLE instance (the caller unwraps: `&g2[0]`, sv_game.cpp:1599), NOT the CGhoul2Info_v
// wrapper. 1:1 arity (G2SV-D6). WinDed live arm is unconditional `return qfalse` (G2SV-D4),
// so serverInstance is unread; g2 threaded per ruling 11 (no-op here).
pub fn g2api_override_server_with_client_data(g2: &mut Ghoul2System, server_instance: &mut CGhoul2Info) -> bool;
pub fn g2api_set_time(g2: &mut Ghoul2System, current_time: i32, clock: i32);
pub fn g2api_get_time(g2: &Ghoul2System, arg_time: i32) -> i32;

// The arena (G2SV-D6): §B5 arena + handle + copyable borrow wrapper.
// §B5 internal id newtype wrapping the arena handle *value* (packs slot | generation).
// The handle VALUE is bit-exact vs the oracle (G2SV-D6; that decision's -ffp-contract
// float clause is irrelevant — this is integer packing). Ghoul2InfoArray's internal
// method arity below (Ghoul2Handle vs raw i32) is a free internal choice (§A1); only the
// CGhoul2Info_v.mItem: i32 already-ported ABI layout and the handle value are frozen.
pub struct Ghoul2Handle(pub i32);
pub struct Ghoul2InfoArray { /* mInfos, mIds, free_indices */ }
impl Ghoul2InfoArray {
    pub fn new_handle(&mut self) -> i32;          // Raven New()
    pub fn is_valid(&self, handle: i32) -> bool;
    pub fn get(&self, handle: i32) -> &[CGhoul2Info];
    pub fn get_mut(&mut self, handle: i32) -> &mut Vec<CGhoul2Info>;
    // Raven DeleteLow's slot half ONLY (G2_API.cpp:328-339): clear mInfos[idx],
    // bump generation / push free index. Bone-cache freeing is NOT here (it lives
    // in the sibling bone_caches arena, out of this struct's reach) — see the
    // Ghoul2System::delete method below (G2SV-D13(a)).
    pub(crate) fn clear_slot(&mut self, handle: i32);   // slot/generation bookkeeping only
}

// G2SV-D13(a) / ruling 29: Raven's Delete()/DeleteLow (G2_API.cpp:413,315) placed the
// whole teardown as a METHOD ON THE ARRAY only because the array was a global — but
// DeleteLow does TWO things: (1) frees every model instance's bone cache —
// RemoveBoneCache(mInfos[idx][model].mBoneCache) for model in 0..mInfos[idx].size()
// (:319-326) — then (2) clears the slot + bumps generation (:328-339). Step (1)'s
// CBoneCaches live in the SIBLING Ghoul2System.bone_caches arena (G2SV-D9), NOT inside
// Ghoul2InfoArray, so the array cannot reach them. Ruling 29 FIXES the frozen seam: delete/
// delete_low are Ghoul2System methods (it owns both fields) — free the freed slot's bone
// caches from bone_caches, THEN Ghoul2InfoArray::clear_slot. There is no &mut
// Ghoul2InfoArray-only delete. Every §F caller already holds &mut Ghoul2System. Observable
// behavior (bone caches freed on Delete; generation/free-list bit-exact, G2SV-D6) matches
// the oracle.
impl Ghoul2System {
    pub fn delete(&mut self, handle: i32);        // Raven Delete (G2_API.cpp:413): guard + delete_low
    fn delete_low(&mut self, idx: i32);           // Raven DeleteLow (G2_API.cpp:315): (1) then (2)
}

// The welded bone pipeline (§F17 shape; not part of the G2API 1:1 set) — same crate.
pub struct SBoneCalc { /* newFrame, currentFrame, backlerp, blendFrame, blendOldFrame, blendMode, blendLerp */ }
pub struct CTransformBone { pub bone_matrix: mdxaBone_t, pub parent: i32, pub touch: i32, pub touch_render: i32 }
pub struct CBoneCache { /* mBones, mFinalBones, mSmoothBones, header, mod, rootBoneList, rootMatrix,
    incomingTime, mCurrentTouch/mLastTouch/mLastLastTouch, mSmoothingActive, mUnsquash, mSmoothFactor */ }
impl CBoneCache {
    // header read comes over EngineHost (loader-owned model memory, G2SV-D5); no mp_renderer type in the sig.
    pub fn new(host: &mut impl EngineHost, a_mod: qhandle_t) -> Self;    // ctor, seeds parents from mdxaSkel_t
    pub fn eval(&mut self, index: i32) -> mdxaBone_t;                    // memoized by touch
    pub fn eval_render(&mut self, index: i32) -> mdxaBone_t;             // applies SmoothLow
    pub fn eval_unsmooth(&mut self, index: i32) -> mdxaBone_t;
    pub fn get_parent(&self, index: i32) -> i32;
    pub fn was_rendered(&self, index: i32) -> bool;
}
pub fn g2_transform_bone(bc: &mut CBoneCache, child: i32);            // tr_ghoul2.cpp:1541
pub fn g2_construct_ghoul_skeleton(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, frame_num: i32, check_for_new_origin: bool, scale: vec3_t);
pub fn eval_bone_cache(g2: &mut Ghoul2System, cache: BoneCacheId, index: i32) -> mdxaBone_t;
pub fn multiply_3x4_matrix(out: &mut mdxaBone_t, in2: &mdxaBone_t, inm: &mdxaBone_t); // -ffp-contract=off (G2SV-D6)
```

*(Exact struct fields and the full `G2API_*` list are the roster's per-file
transcription target; the method-mapping table below enumerates the non-obvious
ones.)*

**Out-param contract for the un-illustrated `G2API_*` functions (G2SV-D1
generalized — the frozen discriminator, not a per-function judgment).** The ~80
functions not spelled above are 1:1 per §C7, but §C7's mechanical out-param→return
default does **not** by itself settle the false-path write contract that `G2SV-D1`
flags as a real bug class. The discriminator is frozen here and classifies every
out-param function by reading **only its failure path** in the oracle (a
mechanical transcription check, not a design call):
- **Write-on-all-paths → `&mut T` out-param + `bool` return (write-through,
  `G2SV-D1`).** A function whose failure path still writes the out-param before
  returning `qfalse` (callers read it on false) keeps the `&mut` out-param.
  `G2API_GetBoltMatrix` (`G2_API.cpp:1795`) is the archetype and is frozen above.
- **Write-on-success-only → §C7 value/`Option` return (the default).** A function
  that `return qfalse`s **before touching** its out-params when
  `G2_SetupModelPointers` fails maps by the §C7 default to a returned value, i.e.
  `Option<…>` (`None` = the false path, `Some(values)` = the written success
  path). `G2API_GetBoneAnim` (`G2_API.cpp:1140`: `if (G2_SetupModelPointers(…)) {
  … return ret; } return qfalse;` — out-params `currentFrame`/`startFrame`/
  `endFrame`/`flags`/`animSpeed` untouched on the false path) and
  `G2API_GetAnimRange` (`:1191`, same shape) are the archetypes of this class;
  the remaining out-param functions (`GetRagBonePos`, `GetAnimFileName`,
  `SaveGhoul2Models`, …) each classify the same way from their own failure path.

This freezes the *contract* (which paths write); the exact returned tuple/struct
shape for the success-only class is the per-file transcription target's §A1
internal latitude, as for the rest of the un-illustrated set. A porter therefore
never re-derives the write-through-vs-drop question as an open judgment — the rule
above decides it, and `G2SV-D1`'s defect cannot recur because the two classes are
named with their oracle discriminator.

`BoneCacheId` is the key for the hand-rolled `Ghoul2System.bone_caches`
owned arena introduced by this doc (`G2SV-D9`; §B5, hand-rolled in-crate, no external `slotmap` crate); it lives beside `Ghoul2System` in
`mp_engine_ghoul2` (roster). The `mdxaHeader_t`/`mdxaSkel_t` model memory the bone
ctor reads is loader-owned in `mp_renderer` and reached over `EngineHost` (a
model-memory accessor service), so no `mp_engine_ghoul2` → `mp_renderer` crate
edge exists (`G2SV-D5`); `qhandle_t` model handles are the only crossing values.

## Decisions

**G2SV-D1.** `g2api_get_bolt_matrix` is **write-through + `qboolean`**, exactly
like Raven's `qboolean G2API_GetBoltMatrix(..., mdxaBone_t *matrix)` — the
out-matrix is ALWAYS written (every failure path writes the identity/fallback
before returning `qfalse`), so the seam takes `bolt_matrix: &mut mdxaBone_t` and
returns `bool`. Because ruling 18 settled the earlier `-> Option<mdxaBone_t>`
draft as a **defect**: callers read the matrix on the false path and an `Option`
would drop that write. Rejected the `Option` return: it silently changes the
observable contract the server depends on. **Generalized to every out-param
`G2API_*` function** (the ~80 un-illustrated ones) via the §F frozen
discriminator "Out-param contract for the un-illustrated `G2API_*` functions":
write-on-all-paths (incl. failure) → `&mut` + `bool` (this decision);
write-on-success-only, i.e. `return qfalse` before touching the out-param → §C7
value/`Option` return. The complementary (non-write-through) class is
`G2API_GetBoneAnim` (`G2_API.cpp:1140`) / `G2API_GetAnimRange` (`:1191`), which do
**not** write on the `G2_SetupModelPointers`-fail path. Classifying a function is
a mechanical failure-path read, not a per-function design choice, so this defect
class is closed for the whole surface, not just `GetBoltMatrix`.

**G2SV-D2.** The welded renderer subset is exactly the **8 WinDed vcproj renderer
TUs**; the bone pipeline is `tr_ghoul2.cpp`'s subset (fn-extent 3,505 LOC). The
other 7 (`tr_shader` 3,139, `tr_model` 1,547, `tr_image` 845, `tr_init` 538,
`matcomp` 240, `tr_main` 47, `tr_mesh` 39) are the model/shader loader — a
separate `tr_model` subsystem doc. Because the G2SV-Q1 evidence query measured the
boundary at the vcproj file set with those fn-extent LOC figures
(`engine-fork-discovery.md:162-164`). Rejected the earlier "5,509 raw-line" /
"148 fns / 9,900 LOC" figures: they conflated raw lines with the post-DEDICATED
fn-extent and blurred the loader boundary.

**G2SV-D3.** The RagDoll/IK solver is **server-live and ported in full**; its
fn-statics (`G2_bones.cpp:1214-1241`) become fields on an owned `RagDollSolver`
host struct per ruling 3. Because the G2SV-Q2 evidence query found 36 rag/IK
functions reachable from `SV_GameSystemCalls` (the 12 ragdoll/IK arms at
`sv_game.cpp:1497-1576`) — genuine cross-frame server state, not `cgame`-only
dead surface. Rejected `thread_local`/returned buffers: the arrays persist the
settle across calls, so they are the "host-struct fields" kind, not "scratch".

**G2SV-D4** (attach-trio classification corrected by `G2SV-D16`/ruling 39b).
`_G2_LISTEN_SERVER_OPT` is never `#define`d in `codemp/`, so this doc ports **what the
WinDed macro set compiles** and §20-notes the genuinely-dropped rest:
`G2API_OverrideServerWithClientData` keeps its live `#ifndef` arm — a trivial
`return qfalse` (`G2_API.cpp:241-242`) → `-> bool { false }`; the compiled-out
override body, the `g2ClientAttachments[]` array, `CGhoul2Info::entityNum`, and
`CopyBoneCache` are dropped with §20 zero-compile notes. **The attach-trio
`G2API_AttachInstanceToEntNum`/`ClearAttachedInstance`/`CleanEntAttachments` are NOT
in that §20 drop** — their signatures are **unconditional** (`G2_API.cpp:200,214,221`;
only the bodies are `#ifdef _G2_LISTEN_SERVER_OPT`), so with the macro off they compile
as **empty-body no-op functions** that LIVE syscall arms still call
(`sv_game.cpp:1587,1591,1594`); they are kept as callable empty-body fns per §C10
(compiled-no-op fold), not dropped — see `G2SV-D16`. Because ruling 12's build config
pins the macro set and porting-rules §F20 governs dead surface, while §C10 governs
compiled-out bodies behind live signatures. Rejected retaining an `attachments` field
for a future listen-server build: it has zero compiled callers in this config
(speculative, plan §"no dead-code").

**G2SV-D5.** State lands as **one direct `Engine.g2: Ghoul2System` field**
(ruling 12: plain, `Default`-init, no `Option`/`Box`/nesting; lazy-init via
Raven's own flags); the former split `RenderG2State` folds into it
(`bone_caches`, `gore_shader`), since ruling 12 fixes the §F subsystem count at
five with a single `g2`. Services (loader model-memory read, trace, print/error,
cvar register/read) cross the **one `EngineHost` trait** (ruling 11); §F methods
take `(&mut Ghoul2System, &mut impl EngineHost)`. The whole server-side bone
pipeline ports
into `mp_engine_ghoul2` (the upstream shared crate,
`workspace-architecture.md:193`); no `mp_engine_ghoul2` → `mp_renderer` crate edge
exists. Because ruling 11's service trait removes the cross-subsystem state
crossing that would have cycled the two crates. Rejected a separate
`mp_renderer::RenderG2State` Engine field: it is not one of ruling 12's five §F
states, and a bidirectional `mp_engine_ghoul2`⇄`mp_renderer` crate edge is a
Cargo cycle.

**G2SV-D6.** All prior settled ghoul2 decisions stand: the `Ghoul2InfoArray`
singleton is the §B5 arena + handle + copyable borrow-wrapper (`CGhoul2Info_v`
keeps its already-ported handle layout); the `G2API_*` surface
(`G2_local.h:96-224`) keeps exact **1:1** signatures (the `SV_GameSystemCalls`
switch references each individually, so no overload consolidation); and bone math
uses the **`-ffp-contract=off`** parity class with **bit-exact** goldens
(porting-rules §F18) — including the handle/generation arithmetic
(`G2_MODEL_BITS`, rollover `G2_API.cpp:330`) so handle values match the oracle.
Because these were settled before the fork session and the rulings did not disturb
them. Rejected float tolerance and overload merging: parity is byte-for-byte
(§A1) and the switch is name/arity-sensitive.

`G2SV-D7`–`G2SV-D10` fold ruling 22 (2026-07-09); `G2SV-D11`–`G2SV-D12` fold
rulings 26 and 24 (2026-07-09, fourth session); `G2SV-D13`–`G2SV-D14` fold rulings
29 and 31+33 (2026-07-09, pass 5); `G2SV-D15`–`G2SV-D16` fold rulings 36 and 39b
(2026-07-09, pass 6); `G2SV-D1`–`G2SV-D14` stand unchanged per the pass-6 inputs
(the pass-5 `D13`/`D14` append numbering is blessed — D1–D12 are not renumbered).

**G2SV-D7.** The gore surface splits into a graph-dead *apply* set and a
server-live *record store* (resolving `G2SV-Q4`; the record-store membership is
finalized by `G2SV-D11`/ruling 26). The apply entries with zero reachability from
engine roots — `G2API_AddSkinGore` (`G2_API.cpp:2569`, only the client
`CG_G2_ADDSKINGORE` trap), `ResetGoreTag` (`G2_misc.cpp:96`, sole caller
`AddSkinGore:2590`), `G2_GetGoreRecord` (`G2_misc.cpp:113`, no caller) — get §20
zero-caller notes and no roster row. The record store — `AllocGoreRecord`/
`FindGoreRecord`/`DeleteGoreRecord`/`DestroyGoreTexCoordinates` (`G2SV-D11`)/
`FindGoreSet`/`NewGoreSet`/`DeleteGoreSet`/`CGoreSet::~CGoreSet`,
`G2API_ClearSkinGore` (`G2_API.cpp:2549`, live via the `G_G2_CLEANMODELS` trap
`:545` and the save/load destruct path `:2493`; the record store's `DeleteGoreSet`
is additionally reached directly from the `G_G2_REMOVEGHOUL2MODEL`/`MODELS` removal
paths `:814,901`), and `G2_GorePolys` (`G2_misc.cpp:804`, live via the collision
`G2_TraceModels` loop `:1494`) — ports fully into `gore/gore_set.rs`. Because ruling 22
settled that gore-apply produces no server-observable state (no populator behind a
`G_G2_*` trap), so its vert-buffer/`GoreTouch` goldens are not referee-gating
(`G2SV-Q4`), while the record store is reached by the live model-cleanup path.
Rejected transcribing `AddSkinGore` as a live seam function (the prior draft): it
is graph-dead server-side (§20). Ruling 26 (`G2SV-D11`) narrows the apply drop to
exactly this trio; `DeleteGoreRecord`/`DestroyGoreTexCoordinates` are live.

**G2SV-D8.** `CRagDollUpdateParams` (`G2_gore.h:94`) is reimplemented as a §F17
enum, not a vtable class (resolving `G2SV-Q5`). Its four non-`_DEBUG` virtuals
(`EffectorCollision` `:106`, `RagDollBegin` `:110`, `RagDollSettled` `:114`,
`Collision` `:119`) are all empty no-op base bodies, and MP instantiates **only**
the base — `CRagDollUpdateParams rduParams;` (`sv_game.cpp:1539`), no subclass in
`codemp/` (the two subclasses `CRagDollInitialUpdateParams`/
`CGameRagDollUpdateParams` are SP-only, `code/game/g_main.cpp:1296`,
`code/ghoul2/G2_bones.cpp:1502`, out of scope). So the closed MP hierarchy has one
concrete kind; the enum carries the six data members plus a single-variant
`RagDollUpdateKind::Server` whose hooks are no-ops. The one live virtual call
`params->RagDollSettled()` (`G2_bones.cpp:2505`, the DEDICATED `#else` arm inside
`G2_RagDoll`; its `#ifndef DEDICATED` twin at `:2497`) becomes a `match` on the
`Server` variant that does nothing. Because §F17 requires the closed virtual
hierarchy's Rust shape be settled in the doc before transcription and ruling 22
settled it as an enum. Rejected a trait object / injected callback: MP has no
overriding subclass and every hook is a no-op, so dispatch is vestigial; the enum
keeps the closed set explicit and lets SP add variants as a DEC-04 diff.

**G2SV-D9.** The per-instance bone-cache store `Ghoul2System.bone_caches` is a
**hand-rolled generational arena keyed by `BoneCacheId`**, matching
`Ghoul2InfoArray`'s bit-exact handle scheme (§B5) — **not** an external `slotmap`
crate. Because ruling 22 clarified that "SlotMap" in the state table names that
in-crate generational arena (zero workspace precedent for a `slotmap` dependency;
the container shape is a free internal choice, porting-rules §A1). Rejected a
`slotmap`/`generational-arena` crate: it adds a dependency where the existing
`Ghoul2InfoArray` arena pattern already applies.

**G2SV-D10.** `CGhoul2Info_v`'s forwarding methods (`operator[]`/`resize`/`size`/
`push_back`, plus the lifecycle `Alloc`/`Free`/`clear`/`DeepCopy`/`operator=`,
`ghoul2_shared.h:335-435`) **colocate in `shared/cghoul2_info_v.rs`** with the
frozen struct, not in `info_array.rs`. The struct layout stays `#[repr(C)]`-frozen
(`mItem: i32`); the added impl forwards through `TheGhoul2InfoArray()` into
`Ghoul2InfoArray::get`/`get_mut` (owned by `info_array.rs`, §B4). A roster row is
added for `cghoul2_info_v.rs`. Because ruling 22 settled that the wrapper class's
methods belong in its own file per §F21 (one class per file, methods colocate).
Rejected the prior draft's placement of these methods in `info_array.rs`: that
split one Raven class across two files against §F21.

**G2SV-D11 (ruling 26, 2026-07-09).** `DestroyGoreTexCoordinates`
(`G2_misc.cpp:43`) **and** `DeleteGoreRecord` (`:118`) are in the **live** record
store, not the graph-dead apply set — settling `G2SV-Q8`. Because both are reached
server-side through `~CGoreSet` (`:174`, its `DeleteGoreRecord` call at `:179`,
whose `DestroyGoreTexCoordinates` call is at `:120`) ← `DeleteGoreSet` ← the live
`G2API_ClearSkinGore`/`REMOVEGHOUL2MODEL` paths (`G2SV-D7`), and both compile
(`#ifdef _G2_GORE`, ON). The ruling-22 dead-bucket mislisting was an **engineorder
graph blind spot**: the reachability tool does not model an implicit C++ destructor
invocation (`~CGoreSet` firing on `delete`, `G2_misc.cpp:163`) as a call edge, so
the destructor's callees read as unreachable. So the §20 gore-apply drop covers
**only** `G2API_AddSkinGore`, `ResetGoreTag`, and `G2_GetGoreRecord`; the two
correction targets port with the record store in `gore/gore_set.rs`. Rejected §20-noting
`DestroyGoreTexCoordinates`/`DeleteGoreRecord` (the prior draft's escalated
reading): the `~CGoreSet` teardown path genuinely calls them at runtime — the
apparent dead-ness was a tool artifact, not oracle ground truth. (The container
they iterate is empty server-side because its only populator is the apply path, so
they do no observable work — but that is a runtime fact, not static dead-ness, and
does not justify dropping the code the live destructor path invokes.)

**G2SV-D12 (ruling 24, 2026-07-09).** The `EngineHost` trait this doc's §F
signatures consume lives in the **pinned** Stage-0 crate `crates/mp/host-interface`
(package `mp_host_interface`), and this doc cites that real path rather than a
placeholder. Because ruling 24 pinned the interface crate and directed docs to cite
real paths from then on. This does **not** define the trait's method roster (still
Stage-0's, non-goals); it only fixes the crate/package a porter `use`s
(`use mp_host_interface::EngineHost;`). Rejected leaving the path unspecified: a
porter reaching a host-consuming body needs the concrete import, and the crate is
now settled.

**G2SV-D13 (ruling 29, 2026-07-09, pass 5).** The three linked raw-pointer shape
holes the Gate-3 dry-run left open (`G2SV-Q9`/`G2SV-Q10`) are closed with concrete
shapes:
- **(a) `delete`/`delete_low` move UP to `Ghoul2System` methods.** Raven placed the
  whole teardown as a method on `Ghoul2InfoArray` (`Delete`/`DeleteLow`,
  `G2_API.cpp:413,315`) only because the array was a global; but `DeleteLow` first
  frees each instance's bone cache (`RemoveBoneCache`, `:319-326`) — those `CBoneCache`s
  live in the **sibling** `Ghoul2System.bone_caches` arena (`G2SV-D9`), unreachable
  from `Ghoul2InfoArray` — then clears the slot + bumps generation (`:328-339`).
  `Ghoul2System::delete` owns both fields: it frees the freed slot's bone caches from
  `bone_caches`, then calls `Ghoul2InfoArray::clear_slot`. The frozen Seam is FIXED —
  there is **no `&mut Ghoul2InfoArray`-only `delete`**. Because the earlier "§A1
  internal latitude" note left the placement unfrozen, and a porter needs the seam
  fixed. Rejected keeping `delete` on the array with a threaded `&mut bone_caches`: the
  observable teardown crosses two owned fields, so it is a system-level method.
- **(b) `RagDollSolver` stores bone INDICES + resolves basepose per call via
  `EngineHost`.** The fn-statics' raw pointers (`ragBoneData`/`rag` into the model's
  `mBlist`; `ragBasepose`/`ragBaseposeInv` into loader model memory,
  `G2_bones.cpp:1214-1218,1241`) violate §B5/§D11 on a new §F type. `ragBoneData`/`rag`
  become `mBlist` **indices** (Raven already carries `ragBlistIndex[MAX_BONES_RAG]`,
  `:1220`), resolved at use; `ragBasepose`/`ragBaseposeInv` are **not stored** — the
  basepose matrices are resolved per call through `G2_GetBoneMatrixLow` over
  `EngineHost` (`:2622`, the write-through `GetBoltMatrix` pattern, ruling 21). Closes
  `G2SV-Q9`. Because Raven's own parallel index array shows the index is the load-bearing
  identity and the pointer a cache; §B5 forbids the raw field. Rejected storing owned
  `boneInfo_t`/`mdxaBone_t` copies: they alias live model state that other passes mutate,
  so an index/handle read is the faithful shape. (The basepose resolve reads model
  memory through ruling-36 `EngineHost::model_mdxa` — `G2SV-Q11`(a) SETTLED,
  `G2SV-D15`.)
- **(c) `GoreState` owns each per-LOD gore buffer as `Vec<f32>`.** Raven `Z_Malloc`s
  each `tex[TS.lod]` block (`G2_misc.cpp:1020`, freed by `Z_Free` in
  `~GoreTextureCoordinates` `G2_gore.h:25-36` and on realloc `:1028`). The frozen ABI
  field `tex: [*mut c_float; MAX_LODS]` cannot itself become a `Vec` and §D11 forbids
  raw `Z_Malloc`-mirroring unsafe in this internal type, so `GoreState` owns the buffers
  as `Vec<f32>` (`tex_buffers`) and the frozen `tex` pointers point INTO them; teardown
  order mirrors `Z_Free`. Document alloc at `AllocGoreRecord`/`G2_GorePolys`, free at
  `DestroyGoreTexCoordinates`. Closes `G2SV-Q10`. Because porting-rules §9 maps
  `Z_Malloc`→owned `Vec` and the owner must be the safe-Rust side while the ABI field
  stays frozen. Rejected seam-confined unsafe mirroring `Z_Malloc` in the internal type
  (§D11 confines unsafe to the seam) and a layout-free reimplement (supersedes the
  type-port layout the ABI needs). NB the server slice is all-null (no server caller sets
  `TS.gore`, `G2SV-D7`/`G2SV-Q4`), so this carries **no golden surface** — but
  `G2_GorePolys` ports fully, so the alloc code is transcribed against this shape.

**G2SV-D14 (rulings 31+33, 2026-07-09, pass 5).** The Stage-0 host crate is **BUILT
and green** at `crates/mp/host-interface` (package `mp_host_interface`, commit
`4b7f01b0`), superseding `G2SV-D12`'s "pinned but unbuilt" status: the real
`EngineHost` trait signatures are **quoted verbatim** into `## Seam definition` (ten
methods: `trace`, `fs_read_file`, `fs_free_file`, `print`, `error`, `vm_call`,
`shared_memory`, `flrand`, `irand`, `gentity`) so the doc is self-contained, and
goldens run against the fixture-backed `MockHost` (`src/mock.rs`, ruling 32) rather
than a hand-written stub. Because ruling 31 built the crate and ruling 33 settled its
`VmSlot`/UDP shape, and the doc-standards self-containment gate wants the frozen
signatures in-doc. Rejected re-declaring the trait or citing a placeholder: the built
crate is the ground truth a porter `use`s. This pass-5 decision did **not** add the
model-memory-read or cvar methods ghoul2 needs — that gap (`G2SV-Q11`) is closed by
`G2SV-D15` (ruling 36).

**G2SV-D15 (ruling 36, 2026-07-09, pass 6).** `EngineHost` is **EXTENDED and BUILT**
(commit `a9820853`), superseding `G2SV-D14`'s ten-method roster with **fifteen**
methods; the full trait is **re-quoted verbatim** into `## Seam definition`. The five
added methods are:
- **`model_mdxm(&mut self, model: qhandle_t) -> *mut c_void`** and
  **`model_mdxa(&mut self, model: qhandle_t) -> *mut c_void`** — the loader's live
  parsed `.glm`/`.gla` blocks, two accessors mirroring `R_GetModelByHandle` → `model_t`
  → `->mdxm`/`->mdxa` (`G2_API.cpp:2716-2739`), returning NULL exactly where Raven's
  `model_t` pointers are NULL. `c_void` because ghoul2 does its byte arithmetic off the
  returned pointer unchanged (`tr_ghoul2.cpp:416-421`) and `mp_engine_ghoul2` **still
  never names** the mdxm/mdxa header types as Rust types (`G2SV-D5`). These serve
  `CBoneCache::new` parent-seeding, `render/skeleton.rs`, and the ragdoll basepose
  resolve (`G2SV-D13`(b)) — closing **`G2SV-Q11`(a)**.
- **`cvar_integer(&mut self, name: &str) -> i32`** — a missing name reads 0
  (`Cvar_VariableIntegerValue`, `cvar.cpp:118-124`). `cg_g2MarksAllModels`
  (`G2_misc.cpp:40`, read `:1524`) and the renderer-owned `broadsword*` family
  (`G2_bones.cpp:1176-1189`) both read through it — so ghoul2 reads the renderer-owned
  cvars **without** a `mp_renderer` edge (`G2SV-D5`) — closing **`G2SV-Q11`(d)**.
- **`sv_time`** and **`fs_write_file`** — added for the sibling §F subsystems (nav's
  frame-clock timers and `CNavigator::Save`); ghoul2 does not consume either but they
  are part of the re-quoted 15-method trait.

Ruling 36 also settles the **`flrand` seam**: `g2api_set_ragdoll`'s frozen signature
**takes `host: &mut impl EngineHost`** because its body **unconditionally** reaches
`flrand` (`G2_bones.cpp:1468-1469` via `G2_Set_Bone_Angles_Rag`), routed through
`EngineHost::flrand` (ruling 21). This **corrects** any earlier prose claim that
`flrand`/`irand` are not consumed by ghoul2: `flrand` **is** consumed server-side
(`G2_bones.cpp:1468,2127-2129,3327,3925,4290` — all `flrand`, verified); `irand`
alone is unused (grep: no `irand` token in `codemp/ghoul2/`). Because ruling 36 built
the extension and the two pass-5-escalated services now have concrete methods; the
whole roster's host-consuming bodies transcribe against the built 15-method trait.
Rejected leaving `G2SV-Q11` open or re-parsing model files in `mp_engine_ghoul2`
(`G2SV-D5` forbids the re-parse; the accessors hand back the loader's live block).

> **Amended by DEC-35 (2026-07-23,
> `docs/plans/2026-07-23-ghoul2-ownership.md`):** the `*mut c_void` return
> shape of `model_mdxm`/`model_mdxa` is superseded — the `mdx/` view module
> hoisted to `mp_host_interface` (a crate below both consumers), so the
> accessors return `MdxmView`/`MdxaView` (later `MdxmRef`/`MdxaRef` with a
> parsed-once sidecar built by the loader at ingest). `G2SV-D5`'s substance is
> unchanged: still no `mp_engine_ghoul2 -> mp_renderer` edge, still no second
> file-parse path, and the accessors still hand back the loader's live block —
> now typed.

**G2SV-D16 (ruling 39b, 2026-07-09, pass 6).** The attach-trio
`G2API_AttachInstanceToEntNum`/`G2API_ClearAttachedInstance`/`G2API_CleanEntAttachments`
are **compiled no-ops kept as callable empty-body fns in `api_bolts.rs` per §C10**, NOT
§20-dropped — correcting `G2SV-D4`'s original classification. Because their **signatures
are unconditional** (`G2_API.cpp:200,214,221`); only the bodies sit inside `#ifdef
_G2_LISTEN_SERVER_OPT`, so with that macro off (`G2SV-D4`) they compile as empty-body
functions that **LIVE syscall arms still call** (`sv_game.cpp:1587,1591,1594` behind
`G_G2_ATTACHINSTANCETOENTNUM`/`G_G2_CLEARATTACHEDINSTANCE`/`G_G2_CLEANENTATTACHMENTS`).
A §20 drop would remove a symbol the server dispatch reaches — an incorrect classification.
The compiled-out **body content** (the `g2ClientAttachments[]` writes) is a §C10 dead-arm
fold, but the fn itself must exist and be callable (an empty no-op). This differs from
`G2API_OverrideServerWithClientData`, whose live arm returns a value (`qfalse`); the
attach-trio return `void`, so their no-op body is genuinely empty. Rejected §20-noting
them (the original `G2SV-D4` reading): their signatures compile and live arms call them.

## Verification strategy

Governing clause: porting-rules §F18 (differential goldens), DEC-09
(oracle-differential parity). Harness `tools/ghoul2-oracle/` copies the GP2
pattern (`tools/gp2-oracle/`): `run.sh` compiles the **unmodified** oracle TUs
(`codemp/ghoul2/*.cpp` + the `tr_ghoul2.cpp` bone subset) against stub headers,
`main.cpp` dumps canonical behavior over committed fixtures, goldens under
`golden/` so `cargo test` needs no C++ toolchain; Rust parity tests
(`tests/ghoul2_parity.rs` in `mp_engine_ghoul2`) mirror the dump byte-for-byte.

**Host injection (ruling 32, `G2SV-D14`/`G2SV-D15`).** The host-taking §F entries run
their real frozen signature against the **fixture-backed `MockHost`**
(`crates/mp/host-interface/src/mock.rs`) — no test-only constructor is added to
ghoul2. `MockHost` is a pure function of its injected fixtures (FS path→bytes map,
deterministic `trace` = empty space, captured `print`/`error`, holdrand LCG), so the
oracle-vs-Rust goldens byte-compare. It is always compiled (no feature gate). Ruling
36's `MockHost` fixtures (commit `a9820853`) add the two services pass 5 lacked: a
`cvars` name→i32 map (a missing name reads 0, `mock.rs` `cvar_integer`) and the
`mdxm_blocks`/`mdxa_blocks` maps of caller-provided model bytes that `model_mdxm`/
`model_mdxa` hand back pointers into (NULL where no fixture, mirroring Raven's NULL
`model_t.mdxm`/`mdxa`). So the model-memory and cvar-consuming bodies (`bone_cache.rs`
ctor, `skeleton.rs`, ragdoll basepose, `misc.rs` model reads + the `G2_TransformModel`
`cg_g2MarksAllModels` gate `:569`, gore cvar gate in `gore/gore_set.rs`) now have a
golden vehicle — `G2SV-Q11` is SETTLED, so none is blocked. (No `cvars.rs` body: the
`cg_g2MarksAllModels` read is inlined via `cvar_integer`, State ownership.)

Fixtures / goldens (the M3 gate is "G2 bone/bolt/collision goldens",
`GOAL-engine.md:72`):
- **Bone-transform goldens** — load a `.glm`/`.gla` fixture set, run
  `G2_ConstructGhoulSkeleton` + `EvalBoneCache` over a frame sequence, dump every
  bone's `mdxaBone_t` bit-exact (`G2SV-D6`). Covers the memoized `touch` path and
  the smoothing arms.
- **Bolt goldens** — `G2API_AddBolt` + `G2API_GetBoltMatrix` matrices across
  angles/position/scale, incl. the `gG2_GBM*` reconstruct-skip flags. Dump the
  written out-matrix on **both** the true and false return paths (`G2SV-D1`
  write-through contract).
- **Collision goldens** — `G2API_CollisionDetect` `CollisionRecord_t` sets over
  ray fixtures (the server's real use, plan §"entanglement").
- **Arena/handle goldens** — `New`/`Delete`/`IsValid` handle values across the
  generation rollover (`G2SV-D6`).
- **RagDoll determinism** — `G2API_SetRagDoll`→settle over a fixed frame count,
  dumping the settled bone matrices. **Load-bearing** for the referee: the solver
  is server-live (`G2SV-D3`, 36 reachable rag/IK fns from `SV_GameSystemCalls`'s
  12 ragdoll/IK arms). Determinism hinges on the `flrand` seeding (ragdoll init
  `:1468-1469` + settle/IK, Seam definition): `MockHost`'s holdrand-backed
  `EngineHost::flrand` reproduces the oracle LCG bit-for-bit, which is exactly why
  `g2api_set_ragdoll` takes `host` (`G2SV-D15`). The golden runs now: `G2_SetRagDoll`'s
  body reads model memory (`G2_GetModA`) via `MockHost.mdxa_blocks`/`model_mdxa` and the
  `broadsword` cvar via `MockHost.cvars`/`cvar_integer` (ruling 36) — both served, so it
  lands with the other model-memory bodies, not gated (`G2SV-Q11` SETTLED).
- **Gore goldens** — `AllocGoreRecord`/`FindGoreSet` tag sequencing incl. the
  `MAX_GORE_RECORDS` eviction (`_G2_GORE` on). The per-LOD `tex` buffer ownership
  (`G2SV-D13`(c), `Vec<f32>` backing the frozen `tex` pointers) carries **no** server
  golden surface — its allocator (`G2_GorePolys`'s `TS.gore` arm) is never reached
  server-side (`G2SV-D7`/`G2SV-Q4`), so the live store holds only all-null records
  whose `Z_Free`-mirroring teardown is a no-op. The gore-apply/`GoreTouch`
  vert-buffer goldens (`G2API_AddSkinGore` → `GoreVerts`/`GoreIndecies`) are
  **not** referee-gating and **not** part of the M3 gate — `G2SV-Q4` resolved:
  `AddSkinGore` has no server trap (only the client `CG_G2_ADDSKINGORE`), so the
  game-DLL A/B referee never drives it (optional TU-harness coverage only).
  `GoreTouch++` via the collision path (`G2_misc.cpp:890`) is always exercised.
- UB inputs (the `Get()` shared-`null` aliasing, `_DEBUG` `_isnan` asserts) are
  kept OUT of shared fixtures or normalized in the dumper with a comment (§F19).

## Files roster

Machine-readable file plan for `port-cpp-subsystem`'s `designPath` (rule 6). All
`mode: mp`, all `crate: mp_engine_ghoul2` (the server-side bone pipeline
co-locates here per `G2SV-D5`; there is no `mp_renderer` roster entry). Sharding
follows porting-rules §F21 (one Raven class / logical unit per file; free-function
API groups split by concern to keep porter units bounded, since `G2_API.cpp` is
2,783 LOC and `G2_bones.cpp` 4,907 LOC).

```yaml
files:
  - path: crates/mp/engine/ghoul2/src/ghoul2_system.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2System
    summary: The Engine.g2 direct field (G2SV-D5, ruling 12) — fields info_array, time_bases, gbm_no_reconstruct/gbm_use_sp_method, gore GoreState, rag RagDollSolver, bone_caches (a hand-rolled owned in-crate generational arena of CBoneCache keyed by BoneCacheId, §B5 — same kind as Ghoul2InfoArray, no external slotmap crate, G2SV-D9; + the BoneCacheId key), gore_shader qhandle_t (both folded from the former RenderG2State). Plain Default-init. Owns the delete/delete_low methods (G2SV-D13(a)/ruling 29): free the slot's bone caches from bone_caches then Ghoul2InfoArray::clear_slot — Raven's Delete/DeleteLow (G2_API.cpp:413,315) move UP here because they touch both the arena and the sibling bone_caches. One type per file (CLAUDE.md).
  # No cvars.rs / Ghoul2Cvars: G2SV-D15's frozen cvar_integer(name) is a stateless
  # by-name read with no Cvar_Get/handle-registration step, so there is no cvar handle
  # to store (the pre-ruling-36 "ruling-2 EngineCvars sub-struct" purpose is void). The
  # cg_g2MarksAllModels read is inlined at its call sites (gore/gore_set.rs G2_GorePolys,
  # api_gore.rs) via host.cvar_integer, exactly as the renderer-owned broadsword family is
  # already treated (state-ownership rows). Whether the "cg_g2MarksAllModels" name string
  # is inlined or lives in a module-level const is porter internal latitude (porting-rules
  # §A1 / §C10), not a doc-frozen point — either way carries no state field and no ABI seam.
  - path: crates/mp/engine/ghoul2/src/shared/cghoul2_info.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGhoul2Info
    summary: The per-instance CGhoul2Info class (ghoul2_shared.h:240) — a §F idiomatic reimplementation with owned Vecs (mSlist/mBltlist/mBlist), the save-serialized middle band, and runtime mBoneCache(->Option<BoneCacheId>)/mTransformedVertsArray/validity ptrs; DeepCopy zeroes runtime state. NOT the already-ported handle CGhoul2Info_v. entityNum dropped (_G2_LISTEN_SERVER_OPT off, G2SV-D4). Colocated in shared/ (mirrors ghoul2_shared.h).
  - path: crates/mp/engine/ghoul2/src/info_array.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2InfoArray
    summary: Arena + handle (Ghoul2Handle newtype colocated) + IGhoul2InfoArray impl; New/IsValid/Get, clear_slot (DeleteLow's slot half only — clear mInfos[idx] + bump generation, G2_API.cpp:328-339), TheGhoul2InfoArray accessor, Ghoul2InfoArray_Free, id-generation arithmetic bit-exact (G2SV-D6). Delete/DeleteLow themselves are NOT here — they move UP to Ghoul2System (G2SV-D13(a)/ruling 29: they free bone caches from the sibling bone_caches arena then call clear_slot; no &mut Ghoul2InfoArray-only delete). The get/get_mut(handle) forwarding target the CGhoul2Info_v wrapper methods call into (§B4). The wrapper methods themselves colocate in shared/cghoul2_info_v.rs (G2SV-D10), NOT here. The generational-arena container is hand-rolled in-crate, not a slotmap crate (G2SV-D9).
  - path: crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGhoul2Info_v
    summary: The 4-byte arena handle wrapper (mItem: i32), #[repr(C)] layout already frozen. G2SV-D10 (ruling 22) adds its forwarding/lifecycle impl colocated here per §F21: operator[]/size/resize/push_back realized as get/get_mut(handle) forwarding through TheGhoul2InfoArray() (§B4), plus Alloc/Free/clear/DeepCopy (New/Delete + DeepCopy's runtime-state zeroing loop ghoul2_shared.h:385-397) and operator= handle copy. Struct layout stays frozen; only the impl is added. Methods forward into Ghoul2InfoArray (info_array.rs).
  - path: crates/mp/engine/ghoul2/src/api_models.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API models
    summary: Init/Remove/Clean/Copy/Duplicate Ghoul2 models, PrecacheGhoul2Model, SetLodBias/Skin/Shader/Flags, SetGhoul2ModelIndexes, HaveWeGhoul2Models, Ghoul2Size, SkinlessModel (G2SV-D6 1:1).
  - path: crates/mp/engine/ghoul2/src/api_bones.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API bones
    summary: G2API_SetBoneAnim/GetBoneAnim/GetAnimRange/PauseBoneAnim/StopBoneAnim/SetBoneAngles(+Matrix,+Index)/RemoveBone/GetBoneIndex/DoesBoneExist/AnimateG2Models wrappers over G2_bones internals.
  - path: crates/mp/engine/ghoul2/src/api_bolts.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API bolts+attach
    summary: AddBolt/AddBoltSurfNum/RemoveBolt/SetBoltInfo/GetBoltMatrix (write-through+bool, G2SV-D1; gG2_GBM* flags), AttachG2Model/DetachG2Model/AttachEnt/DetachEnt, SetNewOrigin. AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments are COMPILED NO-OPS kept as callable empty-body fns here per §C10 (G2SV-D16/ruling 39b): their signatures are unconditional (G2_API.cpp:200,214,221), only the bodies are #ifdef _G2_LISTEN_SERVER_OPT (off, G2SV-D4), and LIVE syscall arms call them (sv_game.cpp:1587,1591,1594) — NOT §20-dropped. GetBoltMatrix reads model memory via EngineHost::model_mdxa (ruling 36).
  - path: crates/mp/engine/ghoul2/src/api_surfaces.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API surfaces
    summary: SetSurfaceOnOff/GetSurfaceOnOff/SetRootSurface/AddSurface/RemoveSurface/GetParentSurface/GetSurfaceIndex/GetSurfaceName/GetSurfaceRenderStatus/ListSurfaces.
  - path: crates/mp/engine/ghoul2/src/api_ragdoll.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API ragdoll+IK
    summary: SetRagDoll/ResetRagDoll/AnimateG2Models(rag), RagPCJConstraint/RagPCJGradientSpeed/RagEffectorGoal/GetRagBonePos/RagEffectorKick/RagForceSolve, SetBoneIKState/IKMove, AbsurdSmoothing (server-live, G2SV-D3). params type is the RagDollUpdateParams §F17 enum (G2SV-D8, defined in ragdoll_update_params.rs). g2api_set_ragdoll takes host (&mut impl EngineHost): G2_SetRagDoll -> G2_Set_Bone_Angles_Rag UNCONDITIONALLY calls flrand server-side (G2_bones.cpp:1468-1469, no DEDICATED guard) -> EngineHost::flrand (ruling 11/21/36, G2SV-D15). Its completed body also reads model memory (G2_GetModA :1645) via model_mdxa + the broadsword cvar (:1628) via cvar_integer -> both served by ruling 36 (G2SV-Q11 SETTLED), so the whole body transcribes now.
  - path: crates/mp/engine/ghoul2/src/ragdoll_update_params.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: RagDollUpdateParams
    summary: CRagDollUpdateParams (G2_gore.h:94) reimplemented as a §F17 enum (G2SV-D8) — the six data members (angles/position/scale/velocity/me/settle_frame) + a single-variant RagDollUpdateKind::Server whose four no-op virtual hooks (EffectorCollision/RagDollBegin/RagDollSettled/Collision, :106-123) match to nothing. MP has no subclass (base only, sv_game.cpp:1539); SP's two subclasses are a future DEC-04 diff. One type per file (CLAUDE.md). Distinct from the already-ported plain-data sharedRagDollUpdateParams_t.
  - path: crates/mp/engine/ghoul2/src/api_collision.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API collision+time
    summary: CollisionDetect/CollisionDetectCache, GiveMeVectorFromMatrix, SetTime/GetTime (G2TimeBases), OverrideServerWithClientData (WinDed live arm -> bool{false}, G2SV-D4).
  - path: crates/mp/engine/ghoul2/src/api_gore.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API gore
    summary: G2API_ClearSkinGore (server-live via the G_G2_CLEANMODELS trap :545 and the save/load destruct path :2493; the record-store DeleteGoreSet is also reached directly from the G_G2_REMOVEGHOUL2MODEL/MODELS removal paths :814,901 — ports fully). GetNumGoreMarks. _G2_GORE on (G2SV-D5 gore state). G2API_AddSkinGore is graph-dead server-side (G2SV-D7/ruling 22: only client CG_G2_ADDSKINGORE trap, no G_G2_* arm) -> §20 zero-caller note, not a live seam fn; its vert-buffer/GoreTouch goldens are not M3-gating (G2SV-Q4).
  - path: crates/mp/engine/ghoul2/src/api_saveload.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API save/load
    summary: SaveGhoul2Models/LoadGhoul2Models/FreeSaveBuffer/LoadSaveCodeDestructGhoul2Info, GetAnimFileName(+Index), GetGLAName.
  - path: crates/mp/engine/ghoul2/src/bones.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_Bones internal
    summary: G2_Set/Get/Stop/Pause_Bone_Anim/Angles(+Matrix,+Index), G2_Find_Bone_In_List, G2_Init_Bone_List, G2_RemoveRedundantBoneOverrides, G2_Animate_Bone_List (non-ragdoll bone logic).
  - path: crates/mp/engine/ghoul2/src/ragdoll.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: RagDollSolver
    summary: The RagDoll + IK solver; fn-statics block -> RagDollSolver host fields (G2SV-D3, server-live). G2_RagDollSetup/RagDoll/RagDollSolve/SettlePositionNumeroTrois/RagSetState/IKSolve/DoIK/BoneSnap, SRagEffector. Per G2SV-D13(b)/ruling 29 (closing G2SV-Q9): the raw-pointer statics become blist INDICES + per-call EngineHost basepose resolve — ragBoneData/rag -> mBlist indices (Vec<i32>/[i32;MAX_BONES_RAG]; Raven's parallel ragBlistIndex :1220 already the index), ragBasepose/ragBaseposeInv NOT stored (resolved per call via G2_GetBoneMatrixLow over EngineHost, :2622, write-through). The basepose resolve reads model memory via ruling-36 EngineHost::model_mdxa (G2SV-D15; G2SV-Q11(a) SETTLED). The solver also calls flrand server-side (init :1468-1469 + settle/IK :2127-2129,3327,3925,4290, all flrand) -> EngineHost::flrand (ruling 21) and reads the renderer-owned broadsword cvar family (G2_bones.cpp:1176-1189 extern; tr_init.cpp registers) -> EngineHost::cvar_integer (ruling 36, G2SV-D15; G2SV-Q11(d) SETTLED).
  - path: crates/mp/engine/ghoul2/src/bolts.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_Bolts internal
    summary: G2_Add_Bolt/Add_Bolt_Surf_Num/Remove_Bolt/Init_Bolt_List/Find_Bolt_Bone_Num/Find_Bolt_Surface_Num/RemoveRedundantBolts.
  - path: crates/mp/engine/ghoul2/src/surfaces.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_Surfaces internal
    summary: G2_SetSurfaceOnOff/IsSurfaceOff/SetRootSurface/AddSurface/RemoveSurface/FindOverrideSurface/IsSurfaceLegal/GetParentSurface/GetSurfaceIndex/IsSurfaceRendered.
  - path: crates/mp/engine/ghoul2/src/misc.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_Misc internal
    summary: G2_TraceModels/TransformModel/GenerateWorldMatrix/TransformPoint/TransformAndTranslatePoint/Inverse_Matrix/FindSurface, G2_SetupModelPointers, G2_SaveGhoul2Models/LoadGhoul2Model, list/name helpers.
  - path: crates/mp/engine/ghoul2/src/gore/gore_set.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGoreSet
    summary: CGoreSet + ~CGoreSet + the server-live gore-record store (G2SV-D7): AllocGoreRecord/FindGoreRecord/DeleteGoreRecord (+ its private helper DestroyGoreTexCoordinates, G2_misc.cpp:43) — both LIVE per G2SV-D11/ruling 26 (reached via ~CGoreSet -> DeleteGoreSet, G2SV-Q8 SETTLED), FindGoreSet/NewGoreSet/DeleteGoreSet, GoreState (G2SV-D5), G2_GorePolys (live via collision trace, G2_misc.cpp:1494). Per G2SV-D13(c)/ruling 29 (closing G2SV-Q10): GoreState OWNS each per-LOD gore buffer as Vec<f32> (tex_buffers) and the frozen GoreTextureCoordinates.tex [*mut c_float; MAX_LODS] pointers point INTO those Vecs; alloc at AllocGoreRecord/G2_GorePolys (Z_Malloc, :1020), free/teardown mirrors Z_Free at DestroyGoreTexCoordinates (G2_gore.h:25-36). Server slice is all-null (no TS.gore setter, G2SV-Q4) so no golden surface, but G2_GorePolys ports fully so the alloc code is transcribed. Only G2API_AddSkinGore/ResetGoreTag/G2_GetGoreRecord are graph-dead (G2SV-D7/D11) -> §20 notes, not ported. _G2_GORE on. FILE PLACEMENT (mechanical, not a new decision): this content lands as a new submodule INSIDE the existing gore/ directory module — declared `pub mod gore_set;` in gore/mod.rs and file-named after its primary class CGoreSet — NOT a top-level src/gore.rs, which is a compile-time collision with gore/mod.rs (Rust cannot resolve a module from both gore.rs and gore/mod.rs). Forced by the standing rules alone: one-type-per-file + folder-mirrors-owning-Raven-header (CLAUDE.md; CGoreSet lives in G2_gore.h, the gore-subsystem header -> gore/), same convention as render/bone_cache.rs <- CBoneCache, and it colocates with the already-type-ported gore/ data types exactly as G2SV-D10 colocates new impl with an already-ported module. The two alternatives are rejected by those same standing rules: folding into gore/mod.rs violates one-type-per-file (mod.rs is module glue), and renaming the frozen gore/ directory disturbs the already-type-ported layout the Standing context pins as src/gore/.
  - path: crates/mp/engine/ghoul2/src/render/bone_cache.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CBoneCache
    summary: CBoneCache/CTransformBone/SBoneCalc; EvalLow/Eval/EvalRender/EvalUnsmooth/SmoothLow/GetParent/WasRendered, EvalBoneCache, RemoveBoneCache, ctor parent-seeding (header read via EngineHost, G2SV-D5). _XBOX arm dropped.
  - path: crates/mp/engine/ghoul2/src/render/bone_transform.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_TransformBone
    summary: G2_TransformBone, Multiply_3x4Matrix, G2_CreateQuaterion, G2_CreateMatrixFromQuaterion (-ffp-contract=off, G2SV-D6). Inverse_Matrix is in misc.rs (G2_misc.cpp:1656), not here.
  - path: crates/mp/engine/ghoul2/src/render/skeleton.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2_ConstructGhoulSkeleton
    summary: G2_ConstructGhoulSkeleton, G2_TransformGhoulBones, G2_GetBoneMatrixLow, G2_GetBoneBasepose, G2_RagGetBoneBasePoseMatrixLow, worldMatrix/worldMatrixInv scratch threading.

divergences:
  - "_G2_LISTEN_SERVER_OPT OFF in the WinDed set (G2SV-D4): CGhoul2Info::entityNum, the g2ClientAttachments[] array, and CopyBoneCache compile out -> §20 zero-compile notes; G2API_OverrideServerWithClientData keeps its live #ifndef arm as -> bool { false } (G2_API.cpp:241-242), taking a single CGhoul2Info (=&g2[0], sv_game.cpp:1599), NOT the CGhoul2Info_v wrapper (1:1 arity, G2SV-D6)."
  - "Attach-trio is a COMPILED NO-OP, not a §20 drop (G2SV-D16/ruling 39b, correcting G2SV-D4): G2API_AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments have UNCONDITIONAL signatures (G2_API.cpp:200,214,221) with only their bodies inside #ifdef _G2_LISTEN_SERVER_OPT (off), so they compile as empty-body no-op fns that LIVE syscall arms still call (sv_game.cpp:1587,1591,1594 behind G_G2_ATTACHINSTANCETOENTNUM/CLEARATTACHEDINSTANCE/CLEANENTATTACHMENTS). Kept as callable empty-body fns in api_bolts.rs per §C10 (dead body-arm fold); NOT dropped. Unlike G2API_OverrideServerWithClientData (live arm returns qfalse), the trio return void, so the no-op body is genuinely empty."
  - "_XBOX OFF: CTransformBone::renderMatrix/pad, the Z_Malloc mFinalBones/mSmoothBones raw arrays, EvalFull, SetRenderMatrix dropped; the vector<> arm is the live path."
  - "_SOF2 OFF: the ghoul2_shared.h SSkinGoreData/goreEnum_t variant is dead; MP uses SSkinGoreData_s (q_shared.h:3112)."
  - "Ghoul2InfoArray::Get on an invalid handle returns a shared function-static null vector it first .clear()s (non-reentrant aliasing, G2_API.cpp:427-439) -> Rust returns an empty slice; kept out of shared fixtures (F19)."
  - "g2api_get_bolt_matrix is write-through + qboolean (G2SV-D1, ruling 18): out-matrix ALWAYS written incl. failure paths; NOT Option-returning."
  - "RenderG2State folded into Ghoul2System (G2SV-D5, ruling 12): bone_caches + gore_shader are Ghoul2System fields; the whole server-side bone pipeline is in mp_engine_ghoul2, no mp_renderer crate edge (model memory reached via EngineHost)."
  - "G2API_DEBUG destructor leak-report + g_Ghoul2Allocations/g_G2AllocTrack debug alloc tracking dropped (debug-only, no parity surface)."
  - "boneInfo_t basepose/baseposeInv/... raw mdxaBone_t* members keep their ported ABI repr but are filled from Ghoul2System-owned matrices, not shared raw pointers (B5 seam)."
  - "_DEBUG _isnan / assert paths in SmoothLow/EvalLow are debug-only; normalized out of the dumper with a comment (F19)."
  - "HackadelicOnClient (tr_ghoul2.cpp:104) is const-false server-side: its only writers are in R_AddGhoulSurfaces, #ifndef DEDICATED (:3384-3537); the render-traversal branches fold to their false arm (C10)."
  - "cgvm ragdoll-callback dead branches in G2_bones.cpp (client.h #include :32) all fold to their DEDICATED arm: Rag_Trace (:2684, #ifndef DEDICATED cgvm TRACELINE) -> real CM_BoxTrace else-arm (:2709); G2_BoneSnap (:3951, #ifdef DEDICATED return;) -> compiled no-op (caller G2_RagDollSolve :4244); the four RAG_CALLBACK_BONEINSOLID sites in G2_RagDollSettlePositionNumeroTrois (:3056,3085,3180,3217, #ifndef DEDICATED) compile out (surrounding if(params) logic stays); the :3826 site is doubly dead (#if 0 + #ifndef DEDICATED); G2_RagDebugBox/G2_RagDebugLine (:2884,2905, compiled via _DEBUG_BONE_NAMES #define :2577 but #ifdef DEDICATED return;) are compiled no-ops. No cgvm global resolves in this build (C10/§20)."
  - "RSStorage/NextRS/AllocRS render-surface pool dropped: dead server-side (sole caller tr_ghoul2.cpp:2660 is #ifndef DEDICATED); the #else non-_G2_GORE second GoreVerts (G2_misc.cpp:1088) is also dead (_G2_GORE ON)."
  - "G2_PERFORMANCE_ANALYSIS ON (FINAL_BUILD undefined) but its G2Time_*/G2PerformanceCounter_*/G2PerformanceTimer_* globals are timing instrumentation with no parity surface; dropped (F20), same as the leak-checking globals."
  - "Gore apply/record split (G2SV-D7, ruling 22; corrected by G2SV-D11/ruling 26): the gore-APPLY entries — and ONLY these three — G2API_AddSkinGore (G2_API.cpp:2569), ResetGoreTag (G2_misc.cpp:96, sole caller AddSkinGore:2590), G2_GetGoreRecord (G2_misc.cpp:113, no caller) are graph-dead server-side (only client CG_G2_ADDSKINGORE trap, no G_G2_* arm) -> §20 zero-caller notes, not ported. The record store (AllocGoreRecord/FindGoreRecord/DeleteGoreRecord/DestroyGoreTexCoordinates, FindGoreSet/NewGoreSet/DeleteGoreSet, ~CGoreSet, G2API_ClearSkinGore, G2_GorePolys) is server-live (ClearSkinGore via G_G2_CLEANMODELS :545 + the save/load destruct path :2493; the record-store DeleteGoreSet also reached directly from the REMOVEGHOUL2MODEL/MODELS removal paths :814,901; GorePolys via collision trace G2_misc.cpp:1494) and ports fully. Ruling 26 moved DestroyGoreTexCoordinates (G2_misc.cpp:43) and DeleteGoreRecord (:118) into this live bucket (reached via ~CGoreSet :174/:179 -> :120): the ruling-22 dead-listing was an engineorder graph blind spot — an implicit C++ destructor call (~CGoreSet on delete, :163) is not a reachability edge (G2SV-Q8 SETTLED). AddSkinGore vert-buffer/GoreTouch goldens are not M3-gating (G2SV-Q4)."
  - "CRagDollUpdateParams (G2_gore.h:94) reimplemented as the §F17 RagDollUpdateParams enum (G2SV-D8, ruling 22), NOT a vtable class: MP instantiates only the base (sv_game.cpp:1539); the four virtuals (:106-123) are no-op base bodies, so the sole MP variant RagDollUpdateKind::Server has no-op hooks and params->RagDollSettled() (G2_bones.cpp:2505) matches to nothing. Distinct from the already-ported plain-data sharedRagDollUpdateParams_t. SP's two subclasses (code/) are a future DEC-04 diff."
  - "CGhoul2Info_v forwarding/lifecycle methods (ghoul2_shared.h:335-435) colocate in shared/cghoul2_info_v.rs (G2SV-D10, ruling 22, §F21), not info_array.rs; the #[repr(C)] struct layout (mItem: i32) stays frozen, only the impl is added; methods forward into Ghoul2InfoArray."
  - "Ghoul2System.bone_caches is a hand-rolled in-crate generational arena (BoneCacheId), matching Ghoul2InfoArray's bit-exact handle scheme (§B5), NOT an external slotmap crate (G2SV-D9, ruling 22)."
  - "delete/delete_low move UP from Ghoul2InfoArray to Ghoul2System methods (G2SV-D13(a)/ruling 29): Raven's Delete/DeleteLow (G2_API.cpp:413,315) frees each instance's bone cache (RemoveBoneCache, :319-326) then clears the slot + bumps generation (:328-339); the bone caches live in the sibling Ghoul2System.bone_caches arena (G2SV-D9) unreachable from the array, so the system method owns both — free bone caches, then Ghoul2InfoArray::clear_slot. No &mut Ghoul2InfoArray-only delete; behavior (bit-exact generation/free-list, G2SV-D6) matches the oracle."
  - "RagDollSolver stores mBlist INDICES + resolves basepose per call via EngineHost (G2SV-D13(b)/ruling 29, closing G2SV-Q9): the fn-statics' raw pointers ragBoneData (boneInfo_t*, G2_bones.cpp:1218) and rag (vector<boneInfo_t*>, :1241) become mBlist indices (Vec<i32>/[i32;MAX_BONES_RAG]; Raven's parallel ragBlistIndex :1220 is already the index); ragBasepose/ragBaseposeInv (mdxaBone_t*, :1214-1215) are NOT stored — resolved per call through G2_GetBoneMatrixLow over EngineHost (:2622, write-through GetBoltMatrix pattern, ruling 21). No aliasing raw pointers on the §F type (§B5/§D11)."
  - "GoreState owns each per-LOD gore buffer as Vec<f32> (G2SV-D13(c)/ruling 29, closing G2SV-Q10): the ABI-frozen GoreTextureCoordinates.tex [*mut c_float; MAX_LODS] pointers point INTO GoreState.tex_buffers; Raven Z_Mallocs each tex[TS.lod] (G2_misc.cpp:1020) and Z_Frees in ~GoreTextureCoordinates (G2_gore.h:25-36)/on realloc (:1028) -> owned Vec + Drop mirroring Z_Free order (§9/§D11). Server slice is all-null (no TS.gore setter, G2SV-Q4): no golden surface, but G2_GorePolys ports fully so the alloc code is transcribed."
  - "EngineHost is the BUILT, frozen trait, EXTENDED to 15 methods by ruling 36 (G2SV-D14/G2SV-D15, rulings 31/33/36, mp_host_interface, Stage-0 commit 4b7f01b0 + extension commit a9820853); all 15 signatures are quoted verbatim in Seam definition and goldens run against the fixture-backed MockHost (src/mock.rs, ruling 32 + ruling-36 cvars/mdxm_blocks/mdxa_blocks fixtures). Ruling 36 added the two services ghoul2 needed that pass 5 lacked — model_mdxm/model_mdxa (loader model memory, *mut c_void, G2SV-D5: mp_engine_ghoul2 never names the mdxm/mdxa header types) and cvar_integer (missing name reads 0) — plus sv_time and fs_write_file for the sibling §F subsystems. So G2SV-Q11 is SETTLED (both halves): every roster body transcribes against the built 15-method trait, none blocked. flrand IS consumed by ghoul2 (g2api_set_ragdoll takes host, G2SV-D15); only irand/vm_call/shared_memory/gentity are unused server-side by ghoul2."
  - "Gore record-store file placement (mechanical, not a new decision): CGoreSet + GoreState + the record-store free fns land as a new submodule INSIDE the existing gore/ directory module — file crates/mp/engine/ghoul2/src/gore/gore_set.rs, declared `pub mod gore_set;` in gore/mod.rs — NOT a top-level src/gore.rs, which collides at compile time with gore/mod.rs (Rust cannot resolve a module from both gore.rs and gore/mod.rs). Forced by the standing rules alone: one-type-per-file + folder-mirrors-owning-Raven-header (CLAUDE.md; CGoreSet is declared in G2_gore.h -> gore/), file named after its primary class as render/bone_cache.rs <- CBoneCache, colocated with the already-type-ported gore/ data types per the G2SV-D10 pattern. Folding into gore/mod.rs (violates one-type-per-file — mod.rs is module glue) and renaming the frozen gore/ directory (disturbs the already-type-ported layout pinned as src/gore/) are both rejected by those same rules."
```

## Method transcription table

Anchors for the non-obvious internal + renderer methods (the full 1:1 `G2API_*`
surface is `G2_local.h:96-224`, mapped file-by-file in the roster; §F21). Each
row is one transcription target.

| Raven symbol | oracle cite | Rust file | notes |
|---|---|---|---|
| `Ghoul2InfoArray::New/IsValid/Get` + `clear_slot` (DeleteLow slot half) | `G2_API.cpp:386,399,427,328-339` | `info_array.rs` | id/generation arithmetic bit-exact (G2SV-D6) |
| `Ghoul2InfoArray::Delete/DeleteLow` → `Ghoul2System::delete/delete_low` | `G2_API.cpp:413,315` | `ghoul2_system.rs` | moved UP (G2SV-D13(a)/ruling 29): free bone caches from `bone_caches` (`:319-326`) then `clear_slot`; no array-only delete |
| `TheGhoul2InfoArray` / `Ghoul2InfoArray_Free` | `G2_API.cpp:477-493` | `info_array.rs` | lazy singleton → owned `Ghoul2System` field |
| `CGhoul2Info_v::operator[]/size/resize/push_back` (borrow-wrapper) | `ghoul2_shared.h:399-435` | `shared/cghoul2_info_v.rs` | colocated with the frozen struct (G2SV-D10); forwards through `get`/`get_mut(handle)` (§B4) |
| `CGhoul2Info_v::Alloc/Free/clear/DeepCopy/operator=` (lifecycle) | `ghoul2_shared.h:335-435` (`clear` at `:426`) | `shared/cghoul2_info_v.rs` | colocated (G2SV-D10); `New`/`Delete` + DeepCopy runtime-state zeroing (`:385-397`); `operator=` = handle copy |
| `G2API_GetBoltMatrix` | `G2_API.cpp:1795` | `api_bolts.rs` | write-through out-matrix + `bool` (G2SV-D1) |
| `G2API_OverrideServerWithClientData` | `G2_API.cpp:239` | `api_collision.rs` | takes a single `CGhoul2Info *serverInstance` (`=&g2[0]`, sv_game.cpp:1599), NOT the wrapper (1:1, G2SV-D6); WinDed live arm = `return qfalse` (G2SV-D4) |
| `G2API_AttachInstanceToEntNum` / `ClearAttachedInstance` / `CleanEntAttachments` | `G2_API.cpp:200,214,221` | `api_bolts.rs` | compiled empty-body no-ops (§C10, G2SV-D16/ruling 39b): unconditional signatures, bodies `#ifdef _G2_LISTEN_SERVER_OPT` (off); LIVE arms call them (`sv_game.cpp:1587,1591,1594`) — NOT §20-dropped |
| `CBoneCache::EvalLow` | `tr_ghoul2.cpp:236` | `render/bone_cache.rs` | recurse-parent, memoize by `touch` |
| `CBoneCache::Eval / EvalRender / EvalUnsmooth` | `tr_ghoul2.cpp:455,520,446` | `render/bone_cache.rs` | public read paths; `EvalRender`→`SmoothLow` |
| `CBoneCache::SmoothLow` | `tr_ghoul2.cpp:267` | `render/bone_cache.rs` | render smoothing; `_isnan` asserts dropped (F19) |
| `CBoneCache::CBoneCache` (ctor) | `tr_ghoul2.cpp:390` | `render/bone_cache.rs` | seed `parent` from `mdxaSkel_t` (header via EngineHost); `_XBOX` arm dropped |
| `EvalBoneCache` / `RemoveBoneCache` | `tr_ghoul2.cpp:585,569` | `render/bone_cache.rs` | free-fn entry; delete owned by arena |
| `G2_TransformBone` | `tr_ghoul2.cpp:1541` | `render/bone_transform.rs` | LERP + `Multiply_3x4Matrix` chain |
| `Multiply_3x4Matrix` | `tr_ghoul2.cpp:1128` | `render/bone_transform.rs` | `-ffp-contract=off` (G2SV-D6) |
| `G2_CreateQuaterion` / `G2_CreateMatrixFromQuaterion` | `tr_ghoul2.cpp:1048,1097` | `render/bone_transform.rs` | quaternion↔matrix |
| `G2_ConstructGhoulSkeleton` | `tr_ghoul2.cpp:3567` | `render/skeleton.rs` | drives `G2_TransformGhoulBones` per model |
| `G2_TransformGhoulBones` | `tr_ghoul2.cpp:2075` | `render/skeleton.rs` | builds/refreshes `CBoneCache` |
| `G2_GetBoneMatrixLow` / `G2_GetBoneBasepose` | `tr_ghoul2.cpp:727,656` | `render/skeleton.rs` | server collision bone accessors |
| `G2_GenerateWorldMatrix` | `G2_misc.cpp:1678` | `misc.rs` | sets `worldMatrix`/`worldMatrixInv` scratch |
| `G2_SetupModelPointers` | `G2_misc.cpp:1839` | `misc.rs` | revalidates `mValid`/model ptrs post vid_restart |
| `G2_TraceModels` / `G2_TransformModel` | `G2_local.h:69,75` (`_G2_GORE` arm) | `misc.rs` | collision + gore-apply transform |
| `G2_RagDollSetup/RagDoll/RagDollSolve` | `G2_bones.cpp:2254,2403,3970` | `ragdoll.rs` | fn-statics → `RagDollSolver` (G2SV-D3) |
| `G2_RagDollSettlePositionNumeroTrois` | `G2_bones.cpp:3449` | `ragdoll.rs` | settle-pass `static` locals = scratch kind |
| `G2_IKSolve / G2_DoIK` | `G2_bones.cpp:4297,4453` | `ragdoll.rs` | IK arm shares the solver statics |
| `AllocGoreRecord/FindGoreRecord` | `G2_misc.cpp:58,103` | `gore/gore_set.rs` | server-live record store (G2SV-D7); `MAX_GORE_RECORDS` eviction |
| `DestroyGoreTexCoordinates` / `DeleteGoreRecord` | `G2_misc.cpp:43,118` | `gore/gore_set.rs` | server-LIVE record store (G2SV-D11/ruling 26, G2SV-Q8 SETTLED); reached via `~CGoreSet` (`:174/:179` → `:120`); ruling-22 dead-listing was a destructor-edge graph blind spot |
| `FindGoreSet/NewGoreSet/DeleteGoreSet`, `CGoreSet::~CGoreSet` | `G2_misc.cpp:127,142,153,174` | `gore/gore_set.rs` | server-live (G2SV-D7); `_G2_GORE` on |
| `G2API_ClearSkinGore` | `G2_API.cpp:2549` | `api_gore.rs` | server-live via `G_G2_CLEANMODELS` (`:545`) + save/load destruct (`:2493`) (G2SV-D7); drives `DeleteGoreSet` (`:2557`), which the `REMOVEGHOUL2MODEL`/`MODELS` paths (`:814,901`) also call directly |
| `G2API_AddSkinGore/ResetGoreTag/G2_GetGoreRecord` | `G2_API.cpp:2569`, `G2_misc.cpp:96,113` | — (dropped) | graph-dead server-side (G2SV-D7) → §20 zero-caller notes; no roster row |
| `RagDollUpdateParams` (`CRagDollUpdateParams` §F17 enum) | `G2_gore.h:94` | `ragdoll_update_params.rs` | single MP variant `Server`; `params->RagDollSettled()` (`G2_bones.cpp:2505`) → no-op `match` (G2SV-D8) |

## Slice hooks

- **M3 waves 13–19** (`GOAL-engine.md:71`) — "renderer, RMG, botlib, ghoul2
  complete"; gate = the bone/bolt/collision goldens above. Needs frozen first:
  the already-ported `shared/` + `gore/` layout types (done), and the **`EngineHost`
  trait** (rulings 11/31/36) — now **BUILT and EXTENDED to 15 methods**
  (`G2SV-D14`/`G2SV-D15`, `mp_host_interface`, Stage-0 commit `4b7f01b0` + ruling-36
  extension commit `a9820853`), so **every** service ghoul2 consumes is available: trace,
  print/error, FS read/free/write, `flrand`, the loader model-memory accessors
  (`model_mdxm`/`model_mdxa` — for the bone ctor / `render/skeleton.rs` / the ragdoll
  basepose resolve `G2SV-D13`(b) / the `RE_RegisterModel`→`R_GetModelByHandle` model
  read at `G2_API.cpp:593`, `G2_surfaces.cpp:426`, save/load buffers `:2472,2477`), and
  `cvar_integer`. **`G2SV-Q11` is SETTLED (both halves)** — no consumed service lacks a
  method, so no body blocks on Stage-0 any longer. §F signatures freeze here; goldens run
  against `MockHost` (ruling 32 + ruling-36 model/cvar fixtures).
- **Per-file host-service map** (every roster file transcribes fully now; this records
  which host methods each body binds, no longer a blocked/unblocked split). Two
  pervasive host edges cut across the whole surface: (i) nearly every `G2API_*` wrapper
  opens with `G2_SetupModelPointers` (`G2_misc.cpp:1839`), a loader model-memory read →
  `model_mdxm`/`model_mdxa` (ruling 36); (ii) print/error (`Com_Printf`/`Com_Error`) →
  the frozen `print`/`error` methods.
  - **Host-free (no live body touches any host service):** `ghoul2_system.rs` (state
    struct, `Default`-init), `shared/cghoul2_info.rs` (per-instance data class +
    `DeepCopy` zeroing), `shared/cghoul2_info_v.rs` (wrapper forwarding into the in-crate
    arena, `G2SV-D10`), `info_array.rs` (arena `New`/`IsValid`/`Get`/`clear_slot`;
    `delete`/`delete_low` live on `ghoul2_system.rs` per `G2SV-D13`(a) and are equally
    host-free — they free bone caches + clear the slot; the `OutputDebugString` leak
    report is `_FULL_G2_LEAK_CHECKING` debug-only, dropped §F20), `ragdoll_update_params.rs`
    (the §F17 enum), `bolts.rs` (internal bolt-list ops — its one apparent host line
    `G2_bolts.cpp:194` is a **commented-out** `Com_Printf`, so genuinely host-free), and
    `render/bone_transform.rs` (pure `-ffp-contract=off` matrix/quaternion math).
  - **Host-consuming (all methods now available — transcribe fully):**
    `render/bone_cache.rs` (ctor header read → `model_mdxa`), `render/skeleton.rs`
    (model-memory read → `model_mdxm`/`model_mdxa`), `misc.rs` (`G2_TraceModels` →
    `trace`; `G2_SetupModelPointers` model read → `model_mdxm`/`model_mdxa`; print/error),
    `gore/gore_set.rs` (`G2_GorePolys` reads `cg_g2MarksAllModels` via `cvar_integer`,
    `G2_misc.cpp:1524`; `cg_g2MarksAllModels` is also read in `G2_TransformModel`
    `:569` — inlined at each read site via `host.cvar_integer`, no `cvars.rs` file per
    the roster note), `api_gore.rs`
    (`cvar_integer`), `api_ragdoll.rs` (`g2api_set_ragdoll` → `G2_SetRagDoll`: `flrand`
    seeding, `broadsword` cvar → `cvar_integer`, `G2_GetModA` model memory → `model_mdxa`
    — the §F signature takes `host`, Seam definition/`G2SV-D15`), `api_collision.rs`
    (`CollisionDetect` → `G2_TraceModels` `trace` + opening `G2_SetupModelPointers`
    model read), `api_models.rs` (`RE_RegisterModel` loader-register → model accessors;
    `print`), `api_bolts.rs` (`GetBoltMatrix` model-memory read → `model_mdxa`; **plus
    the attach-trio compiled no-ops, `G2SV-D16`**), `api_saveload.rs` (save/load file
    buffer → `fs_read_file`/`fs_write_file` + the model-touching load path → model
    accessors), `surfaces.rs` (`R_GetModelByHandle(RE_RegisterModel(...))`,
    `G2_surfaces.cpp:426`, model accessors), and `ragdoll.rs` (`Rag_Trace` →
    `CM_BoxTrace` `trace`; basepose resolve `G2_GetBoneMatrixLow` → `model_mdxa`,
    `G2SV-D13`(b); `flrand` seeding `:1468-1469`/settle-pass; `broadsword` cvar `:1628`
    → `cvar_integer`).
  - **Thin wrappers (host-free marshalling + a forwarded host edge):** `bones.rs`
    (internal bone-anim list logic is pure, plus a few live `Com_Printf` error arms →
    `print`), and `api_bones.rs`/`api_surfaces.rs` (their §C7 marshalling is host-free;
    each opens with `G2_SetupModelPointers` → model accessors and forwards into an
    internal). All transcribe fully against the 15-method trait.
- **`SV_GameSystemCalls`** (wave 20, plan §"server is the integrator") — the
  server→ghoul2 edges call the `G2API_*` surface frozen by `G2SV-D6` (incl. the 12
  rag/IK arms, `G2SV-D3`); that seam must be stable before the switch arm ports.

## Resolved questions

Questions the 2026-07-09 fork session and its evidence queries closed (were open
in the prior draft), plus `G2SV-Q4` and `G2SV-Q5` closed by ruling 22
(`G2SV-D7`/`G2SV-D8`), `G2SV-Q8` closed by ruling 26 (`G2SV-D11`), `G2SV-Q9`/`G2SV-Q10`
closed by ruling 29 (`G2SV-D13`, pass 5), and `G2SV-Q11` closed by ruling 36
(`G2SV-D15`, pass 6):
- **G2SV-Q1** (renderer-subset boundary) — **RESOLVED by `G2SV-D2`**: the 8 WinDed
  vcproj renderer TUs, fn-extent LOC per `engine-fork-discovery.md:162-164`; this
  doc owns the `tr_ghoul2` bone subset, the loader is a `tr_model` sibling doc.
- **G2SV-Q2** (ragdoll server-live?) — **RESOLVED by `G2SV-D3`**: 36 rag/IK
  functions reachable from `SV_GameSystemCalls` (`sv_game.cpp:1497-1576`);
  server-live, ported in full, ragdoll goldens load-bearing.
- **G2SV-Q3** (`_G2_LISTEN_SERVER_OPT` drop/retain) — **RESOLVED by `G2SV-D4`**:
  never `#define`d; port the compiled live arm, §20-note the compiled-out override
  path and `g2ClientAttachments[]`.
- **G2SV-Q6** (`Engine`-island attachment) — **RESOLVED by `G2SV-D5`** (ruling
  12): plain direct `Engine.g2: Ghoul2System` field, `Default`-init, no STATE-Q2
  gating remains.
- **G2SV-Q7** (`mp_engine_ghoul2`⇄`mp_renderer` crate cycle) — **RESOLVED by
  `G2SV-D5`** (rulings 11+12): `RenderG2State` folds into `Ghoul2System`, the whole
  server-side bone pipeline co-locates in `mp_engine_ghoul2`, and loader model
  memory is reached over the `EngineHost` trait — so no cross-crate cycle exists.
- **G2SV-Q4** (gore-apply server load-bearingness) — **RESOLVED by `G2SV-D7`**
  (ruling 22, on the same trap-reachability evidence method that settled its twin
  `G2SV-Q2`): `G2API_AddSkinGore` has **no** server-reachable caller — no `G_G2_*`
  gore trap in `g_public.h`, no arm in `sv_game.cpp`; its only trap is the client
  `CG_G2_ADDSKINGORE` (`cg_public.h:280`), reached only from `cgame`
  (`cg_players.c:5795`). So the game-DLL A/B referee (which drives only the
  `SV_GameSystemCalls` surface, DEC-09) never populates `TS.gore`: the gore-apply/
  vert-buffer goldens (`GoreVerts`/`GoreIndexCopy`/`GoreIndecies`) are **not**
  referee-gating and **not** part of the M3 gate ("bone/bolt/collision goldens",
  `GOAL-engine.md:72`). `G2SV-D7` sharpens this into the apply/record split: the
  apply entries (`AddSkinGore`/`ResetGoreTag`/`G2_GetGoreRecord`) are §20-noted, the
  record store (`AllocGoreRecord`…`DeleteGoreSet`, `G2API_ClearSkinGore`,
  `G2_GorePolys`) ports fully. `GoreTouch++` via the collision path
  (`G2_misc.cpp:890`) is exercised independently, as before.
- **G2SV-Q5** (`CRagDollUpdateParams` virtual-dispatch shape, `G2_gore.h:94`) —
  **RESOLVED by `G2SV-D8`** (ruling 22): a §F17 enum. In MP the closed hierarchy has
  one concrete kind — the base is instantiated directly (`sv_game.cpp:1539`), no
  subclass in `codemp/` (the two subclasses are SP-only, `code/`). The four
  non-`_DEBUG` virtuals (`EffectorCollision` `:106`, `RagDollBegin` `:110`,
  `RagDollSettled` `:114`, `Collision` `:119`) are empty base no-ops, and only
  `RagDollSettled()` is called live (`G2_bones.cpp:2505`, the DEDICATED `#else` arm;
  its `#ifndef DEDICATED` twin at `:2497`). `EffectorCollision`'s callsites (`:3054,
  3083,3178,3215`) and `DebugLine`'s (`:2423,2566`) are commented out;
  `RagDollBegin`/`Collision` have no in-tree callers. So the enum carries the six
  data members + a single `Server` variant with no-op hooks (`ragdoll_update_params.rs`),
  and `params->RagDollSettled()` becomes a no-op `match`. Distinct from the
  already-ported plain-data `sharedRagDollUpdateParams_t`.
- **G2SV-Q8** (`DestroyGoreTexCoordinates`/`DeleteGoreRecord` bucketing,
  `G2_misc.cpp:43,118`) — **RESOLVED by `G2SV-D11`** (ruling 26): both port with the
  **live** gore record store in `gore/gore_set.rs`, not the graph-dead apply set. Oracle
  ground truth (`G2_misc.cpp:174/179/120`) shows `~CGoreSet` → `DeleteGoreRecord` →
  `DestroyGoreTexCoordinates` fires on the live `DeleteGoreSet` teardown (`~CGoreSet`
  runs on `delete`, `:163`), reached from `G2API_ClearSkinGore`
  (`G_G2_CLEANMODELS`) and the `REMOVEGHOUL2MODEL`/`MODELS` paths (`G2SV-D7`); both
  compile (`#ifdef _G2_GORE`, ON). The prior draft's escalation rested on a
  reachability-tool artifact — an **engineorder graph blind spot** where an implicit
  C++ destructor invocation is not counted as a call edge — not on oracle ground
  truth. The §20 gore-apply drop is thereby narrowed to exactly `G2API_AddSkinGore`,
  `ResetGoreTag`, `G2_GetGoreRecord`. Not M3-gating either way (gore-apply is out of
  the referee surface, `G2SV-Q4`).
- **G2SV-Q9** (`RagDollSolver` raw-pointer array fields, `G2_bones.cpp:1214-1241`) —
  **RESOLVED by `G2SV-D13`(b)** (ruling 29, pass 5): store `mBlist` **indices**, not
  pointers. `ragBoneData`/`rag` become `mBlist` indices (Raven's parallel
  `ragBlistIndex[MAX_BONES_RAG]` `:1220` already carries the index), resolved against
  the live model's `mBlist` at use; `ragBasepose`/`ragBaseposeInv` are **not stored** —
  the basepose/baseposeInv matrices are resolved per call through `G2_GetBoneMatrixLow`
  over `EngineHost` (`:2622`, write-through `GetBoltMatrix` pattern, ruling 21). No
  aliasing raw pointers outside the ABI seam (§B5/§D11). (The basepose resolve reads
  model memory through ruling-36 `EngineHost::model_mdxa` — `G2SV-Q11`(a) SETTLED,
  `G2SV-D15`.)
- **G2SV-Q10** (`GoreTextureCoordinates.tex` per-LOD buffer ownership,
  `G2_misc.cpp:1020`, `G2_gore.h:25-36`) — **RESOLVED by `G2SV-D13`(c)** (ruling 29,
  pass 5): `GoreState` **owns** each per-LOD buffer as a `Vec<f32>` (`tex_buffers`) and
  the ABI-frozen `tex: [*mut c_float; MAX_LODS]` pointers point INTO those `Vec`s;
  alloc at `AllocGoreRecord`/`G2_GorePolys` (`Z_Malloc`, `:1020`), teardown mirrors
  `Z_Free` order at `DestroyGoreTexCoordinates` (`G2_gore.h:25-36`). No `Z_Malloc`-
  mirroring unsafe in the internal type (§9/§D11). Carries no server golden surface
  (all-null store, `G2SV-Q4`), but `G2_GorePolys` ports fully so the alloc code is
  transcribed against this shape.
- **G2SV-Q11** (two services ghoul2 consumes — the loader model-memory read and cvar
  read — had no method in the pass-5 frozen `EngineHost` trait) — **RESOLVED by
  `G2SV-D15`** (ruling 36, pass 6): the trait is EXTENDED and BUILT (commit `a9820853`)
  with the two missing methods, closing **both** halves.
  - **(a) loader model-memory read** — served by `model_mdxm`/`model_mdxa`
    (`-> *mut c_void`), two accessors mirroring `R_GetModelByHandle` → `model_t` →
    `->mdxm`/`->mdxa` (`G2_API.cpp:2716-2739`) that hand back the loader's live parsed
    block, NULL exactly where Raven's `model_t` pointer is NULL. Ghoul2 does its byte
    arithmetic off the returned pointer unchanged (`tr_ghoul2.cpp:416-421`) and
    `mp_engine_ghoul2` still **never names** the mdxm/mdxa header types (`G2SV-D5`); no
    re-parse (`fs_read_file` returns raw bytes, a different thing — the accessors return
    the loader's cached model). Serves the bone ctor parent-seeding, `render/skeleton.rs`,
    and the ragdoll basepose resolve (`G2SV-D13`(b)).
  - **(d) cvar read** — served by `cvar_integer(name) -> i32` (a missing name reads 0,
    `Cvar_VariableIntegerValue`, `cvar.cpp:118-124`): `cg_g2MarksAllModels`
    (`G2_misc.cpp:40`, read `:1524`) and the renderer-owned `broadsword*` family
    (`G2_bones.cpp:1176-1189`) both read through it, so ghoul2 reads a renderer-owned
    cvar **without** a `mp_renderer` edge (`G2SV-D5`).
  The pass-5 blast radius (most of the 20-file roster) is discharged — every roster body
  now transcribes against the built 15-method trait; the `## Slice hooks` per-file
  host-service map records which methods each binds. The interface-ownership gap resolved
  exactly as pass 5 predicted (by amending the `EngineHost` roster, the Stage-0 crate's
  decision) via ruling 36.

## Open questions

**None.** `G2SV-Q11` (the sole pass-5 open item) is **SETTLED by ruling 36**
(`G2SV-D15`, both halves — see Resolved questions); `G2SV-Q9`/`G2SV-Q10` were settled
by ruling 29 (pass 5), and `G2SV-Q1`–`G2SV-Q8` earlier. No item remains open, so this
section is empty (doc-standards: `## Open questions` MUST be empty at FROZEN — it is
already empty at DRAFT).

## Amendment (user ruling 2026-07-12) — server skins name-pool closes model-memory gap #2

The FROZEN content above is unchanged. This records the closure-campaign ruling
(`DEC-18`, commit `64a48bb8`) that closes the second loader model-memory gap — the
skin/shader-by-name read — sibling to the `.glm`/`.gla` block read `G2SV-D15` closed.

- **Gap #2 (skin/shader read) closed.** Beyond the `model_mdxm`/`model_mdxa` block read
  (`G2SV-D15`), the server surface path reads a skin's per-surface shader **by name**
  (`surf->shader->name`, `G2_surfaces.cpp:212`). That read is now served by a new
  `EngineHost` accessor **`R_GetSkinByHandle`**, backed by `tr-model.md`'s `RenderModels`
  skin pool (its matching amendment) — so `mp_engine_ghoul2` reaches the skin/shader name
  across the service seam, never a `mp_renderer` crate edge (`G2SV-D5` preserved),
  exactly as the model-memory accessors do.
- **Name-only, no compile.** The dedicated path uses only `shader->name`; no compiled
  `shader_t` is produced or read server-side (consistent with the client-draw §20 drops
  and the fixed-`false` `HackadelicOnClient` fold). `mp_engine_ghoul2` **never names** an
  `mp_renderer` skin/shader type — the same type-location discipline `G2SV-D5` fixes for
  the mdxm/mdxa headers.
- **State ownership.** `tr.skins`/`numSkins` are owned by `RenderModels` in `mp_renderer`
  (the `tr-model.md` amendment), not `Ghoul2System`; ghoul2 holds no skin pool and reads
  on demand through the host, mirroring the `cvar_integer`/`model_mdxa` service rows in
  State ownership.
