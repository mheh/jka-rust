# Server-side Ghoul2 + renderer bone/model internals Design
Status: DRAFT     Supersedes: none
Decision prefix: G2SV     Ledger deps: DEC-04 (per-mode), DEC-09 (verification);
engine-fork-discovery rulings 2 (state placement), 3 (fn-statics), 7 (this doc),
11 (EngineHost), 12 (direct Engine fields), 18 (GetBoltMatrix defect), 22
(2026-07-09: gore apply/record split, `CRagDollUpdateParams` §F17 enum, bone-cache
arena, `CGhoul2Info_v` method colocation)

This is a C++-track (`porting-rules.md` §F) design doc. It carries the machine-
readable `files:` roster and `divergences:` list (doc-standards rule 6) that
`.claude/workflows/port-cpp-subsystem.js` consumes via `designPath`; both live
under `## Files roster` and `## Divergences` below.

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
  rendered here as `G2SV-D7`–`G2SV-D10`; rulings 11–18 stand unchanged), and the
  G2SV-Q1/G2SV-Q2 evidence resolutions.
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
- **The exact method roster of the `EngineHost` trait** — the concrete Rust method
  signatures for the four services ghoul2 consumes: (a) the loader model-memory
  read of `mdxaHeader_t`/`mdxaSkel_t` (for `CBoneCache::new` parent-seeding and
  `render/skeleton.rs`), (b) collision trace (`G2_TraceModels`, `G2_GorePolys`),
  (c) print/error, and (d) cvar register/read (`cg_g2MarksAllModels`,
  `G2_misc.cpp:40`) — is owned by the **Stage-0 interface-crate design**
  (`docs/plans/2026-07-08-mp-engine-build-out.md:250`, ruling 11), not here —
  mirroring the sibling §F docs (`docs/subsystems/roff.md` non-goals,
  `docs/subsystems/npcnav.md` NAV-D2, `docs/subsystems/icarus.md` ICARUS-D1). This
  doc names the services ghoul2 consumes and takes `&mut impl EngineHost` in every
  §F signature; it does not define the trait. No cvar-registration API exists in
  `crates/mp/engine` yet (`cvar_init` is a documented no-op stub, build-out plan
  §0.5), so cvar register/read is one of the deferred host methods, not a gap this
  doc fills.
- **Sequencing consequence of the two punts above.** The host-consuming bodies —
  `render/bone_cache.rs` ctor, `render/skeleton.rs`, `misc.rs` trace,
  `api_gore.rs`/`cvars.rs` cvar registration, and every print/error path — are
  transcribed **after** the Stage-0 `EngineHost` trait freezes; a porter reaching
  them earlier binds against that frozen trait, never a stub (plan §"no
  dead-code"). This is a legitimate cross-doc deferral (doc-standards Gate 3, the
  2026-07-03 amendment), not a doc hole: the §F signatures freeze here, the method
  *bodies* wait on Stage-0. The `## Slice hooks` M3 note records this as a
  freeze-first dependency.
- **Model-memory type-location reconciliation (`G2SV-D5`-forced, not a new
  decision).** `mdxaHeader_t`/`mdxaSkel_t` live only in `mp_renderer`
  (`crates/mp/renderer/src/mdx_format/`, type-rosetta) and `G2SV-D5` forbids a
  `mp_engine_ghoul2` → `mp_renderer` crate edge. Whatever return shape Stage-0
  picks for the model-memory accessor (a re-exported `mp_renderer` struct vs
  primitive per-bone field reads), any `mp_renderer` coupling lives in the
  **host-interface crate**, never in `mp_engine_ghoul2`; this doc keeps those two
  types out of every §F signature (`CBoneCache::new` takes only `qhandle_t`,
  `G2SV-D5`), so `mp_engine_ghoul2` never names them as Rust types. The accessor's
  exact type-crossing shape is part of the Stage-0 roster, decided there.

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

**Ruling 22's apply/record split (`G2SV-D7`, resolving `G2SV-Q4`).** The gore
surface splits into a graph-dead *apply* entry set and a server-live *record-store*
infra set:
- **Graph-dead server-side (zero reachability from engine roots → §20 zero-caller
  notes):** `G2API_AddSkinGore` (`G2_API.cpp:2569`; only the client
  `CG_G2_ADDSKINGORE` trap, `cg_public.h:280`, no `G_G2_*` arm), `ResetGoreTag`
  (`G2_misc.cpp:96`; sole caller is `G2API_AddSkinGore` at `:2590`), and
  `G2_GetGoreRecord` (`G2_misc.cpp:113`; **no** caller anywhere in `codemp/`).
  These are dropped with §20 zero-caller notes.
- **Server-live (ports fully):** the record store `AllocGoreRecord`
  (`G2_misc.cpp:58`), `FindGoreRecord` (`:103`), `DeleteGoreRecord` (`:118`),
  `FindGoreSet` (`:127`), `NewGoreSet` (`:142`), `DeleteGoreSet` (`:153`),
  `CGoreSet::~CGoreSet` (`:174`), `G2API_ClearSkinGore` (`G2_API.cpp:2549`), and
  `G2_GorePolys` (`G2_misc.cpp:804`). `G2API_ClearSkinGore` is reached from the
  live `G2API_CleanGhoul2Models` (`G2_API.cpp:496`, its call at `:545`) behind the
  `G_G2_CLEANMODELS` trap (`g_public.h:529`) and from `G2API_LoadSaveCodeDestructGhoul2Info`
  (`:2493`); the record store's `DeleteGoreSet` is additionally reached **directly** from the
  `G_G2_REMOVEGHOUL2MODEL`/`G_G2_REMOVEGHOUL2MODELS` removal paths (`G2_API.cpp:814,901`, which
  call `DeleteGoreSet`, not `ClearSkinGore`). `ClearSkinGore` drives `DeleteGoreSet` (`:2557`) →
  `~CGoreSet` (`:174` → `delete` at `G2_misc.cpp:163`) → `DeleteGoreRecord`
  (`:179`); `G2_GorePolys` is reached from the collision `G2_TraceModels` loop
  (`G2_misc.cpp:1494`, `#ifdef _G2_GORE`, no DEDICATED guard).
- **Ground-truth conflict (`G2SV-Q8`, escalated):** ruling 22 as transcribed lists
  `DestroyGoreTexCoordinates` (`G2_misc.cpp:43`) in the graph-dead apply set, but
  the oracle shows it **reached** server-side via `DeleteGoreRecord` (`:120`) ←
  `~CGoreSet` (`:179`) ← `DeleteGoreSet` ← `G2API_ClearSkinGore` (all live above).
  It compiles (`#ifdef _G2_GORE`, ON, `:26`) and is a private helper of
  `DeleteGoreRecord`. Pending the user's confirmation it is ported with the live
  record store in `gore.rs`, not §20-noted (see Open questions).

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
  server-side `Rag_Trace` unconditionally does the real `CM_BoxTrace` (`:2708`).
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
(model-memory read, trace, print/error, cvar register/read) cross the **one
`EngineHost` trait** (ruling 11); §F methods take `(&mut Ghoul2System, &mut impl
EngineHost)`, and `Engine` implements `EngineHost` via a split-borrow view struct.
The concrete host method signatures are Stage-0's (non-goals); this doc names the
consumed services and freezes the §F call shape.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `Ghoul2InfoArray *singleton` | `G2_API.cpp:477` | `mp_engine_ghoul2::Ghoul2System.info_array: Ghoul2InfoArray` | lazily on first `the_info_array()`; freed by `Ghoul2InfoArray_Free` | `&mut Ghoul2System` into every `G2API_*` |
| `g2ClientAttachments[MAX_GENTITIES]` | `G2_API.cpp:197` (`_G2_LISTEN_SERVER_OPT`) | **dropped** — compiles out in the WinDed set (`G2SV-D4`, §20-note) | — | — (divergences list) |
| `G2TimeBases[NUM_G2T_TIME]` | `G2_API.cpp:160` | `Ghoul2System.time_bases: [i32; NUM_G2T_TIME]` | `Ghoul2System::default` | `&mut Ghoul2System` |
| `gG2_GBMNoReconstruct`, `gG2_GBMUseSPMethod` | `G2_API.cpp:1724-1725` | `Ghoul2System.gbm_no_reconstruct/gbm_use_sp_method: bool` | `Ghoul2System::default` | `&mut Ghoul2System` |
| `g_Ghoul2Allocations`, `g_G2ServerAlloc`, `g_G2ClientAlloc`, `g_G2AllocServer`, `g_G2AllocTrack*`, `g_G2AllocTrackInit` (all `_FULL_G2_LEAK_CHECKING`); `g_goreAllocs`, `g_goreTexAllocs` (`_DEBUG`) | `G2_API.cpp:34-37,43-44`, `G2_misc.cpp:138-139` | dropped (debug alloc tracking, no parity surface, §F20) | — | — (divergences list) |
| `G2Time_*`, `G2PerformanceCounter_*`, `G2PerformanceTimer_*` (`G2_PERFORMANCE_ANALYSIS`, ON) | `tr_ghoul2.cpp:42-62` | dropped (timing instrumentation, no parity surface, §F20 — same treatment as the leak-checking globals) | — | — (divergences list) |
| `GoreRecords`, `GoreTagsTemp`, `CurrentTag`, `CurrentTagUpper` | `G2_misc.cpp:35,36,32,33` | `Ghoul2System.gore: GoreState { records: BTreeMap<i32, GoreTextureCoordinates>, tags_temp, current_tag, current_tag_upper }` | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreSets`, `CurrentGoreSet` | `G2_misc.cpp:125,124` | `GoreState.sets: BTreeMap<i32, CGoreSet>`, `GoreState.current_set` | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreTouch` (persistent gen counter) | `G2_misc.cpp:795` | `GoreState.gore_touch: i32` (ruling 2, same file/subsystem; three-kind persistent). Runs server-side via the collision path (`:890`); gore-*apply* has no server caller (`G2SV-D7`) | `Ghoul2System::default` | `&mut Ghoul2System.gore` |
| `GoreVerts`, `GoreIndexCopy`, `GoreIndecies` | `G2_misc.cpp:793,794,798` | scratch buffers (three-kind scratch; per-`G2_GorePolys` rebuild, invalidated by `gore_touch`) — impl-local, not a global; never server-driven (`G2SV-D7`: no `AddSkinGore` server trap), transcribed faithfully, goldens optional | — | — |
| `goreModelIndex` | `G2_misc.cpp:38` | scratch (three-kind scratch; set in the `G2_TraceModels` model loop `:1539`, read as the `GoreTagsTemp` key `:959,1000`) — impl-local, threaded through the trace, not a global | — | — |
| `cg_g2MarksAllModels` | `G2_misc.cpp:40` | `Ghoul2System.cvars: Ghoul2Cvars` (the ruling-2 per-subsystem EngineCvars sub-struct; own file `cvars.rs` per one-type-per-file, roster) | registered via the `EngineHost` cvar register/read service (ruling 11, method sig Stage-0's — non-goals; `cvar_init` is a no-op stub today, plan §0.5) | `&Ghoul2System.cvars` |
| RagDoll fn-statics block (`ragBasepose`…`rag`) | `G2_bones.cpp:1214-1241` | `Ghoul2System.rag: RagDollSolver { basepose, basepose_inv, bones, effectors, bone_data, temp_dependents, blist_index, num_rags, bone_mins/maxs/cm, desired_pelvis_offset, have_desired_pelvis_offset, origin_change, origin_change_dir, hand_pos, hand_pos2, rag_state, rag: Vec<..> }` | `Ghoul2System::default` | `&mut Ghoul2System.rag` (ruling 3 cross-frame kind, `G2SV-D3`) |
| solver `static const` matrices / settle-pass `static` locals | `G2_bones.cpp:1423,3452-3475` | `const` items / function locals (three-kind rule: const-table / scratch) | — | — |
| `CBoneCache *mBoneCache` per instance | `ghoul2_shared.h:265` | `Ghoul2System.bone_caches` — a hand-rolled owned in-crate generational arena of `CBoneCache` keyed by `BoneCacheId` (§B5 arena, same kind as `Ghoul2InfoArray`; **not** an external `slotmap` crate — `G2SV-D9`, zero workspace precedent, container shape free per §A1), folded from the former RenderG2State per ruling 12; `CGhoul2Info.mBoneCache` → `Option<BoneCacheId>` | `G2_ConstructGhoulSkeleton` on demand; freed by `Ghoul2InfoArray::delete_low` | `&mut Ghoul2System` |
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
EngineHost, ...)`. `host` provides the loader-owned model-memory read
(`mdxaHeader_t`/`mdxaSkel_t`), trace, print/error, and cvar register/read
(`cg_g2MarksAllModels`); the exact host method signatures are Stage-0's
(non-goals). There is **no `RenderG2State` parameter** (folded into
`Ghoul2System`, ruling 12) and **no `mp_renderer` crate dependency** (`G2SV-D5`);
the model-memory types stay out of these signatures (`G2SV-D5` type-location
reconciliation, non-goals).

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
  needed.** `qhandle_t` and `mdxaBone_t` are `native_types`-owned
  (`crates/native/types/src/lib.rs`, type-rosetta) but `mp_qshared` **re-exports**
  them (`crates/mp/qshared/src/shared/mod.rs:136-137`), so a porter writes `use
  mp_qshared::shared::{qhandle_t, mdxaBone_t}` and adds nothing to `Cargo.toml`.
  `CollisionRecord_t` is `mp_qshared`-owned directly
  (`crates/mp/qshared/src/shared/collision.rs`, with `G2Trace_t =
  [CollisionRecord_t; MAX_G2_COLLISIONS]`, `MAX_G2_COLLISIONS = 16`; `mEntityNum ==
  -1` = unused record).
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
    ghoul2: &mut CGhoul2Info_v, model_index: i32, bone_name: &str, angles: Vec3,
    flags: i32, up: Eorientations, left: Eorientations, forward: Eorientations,
    model_list: &[qhandle_t], blend_time: i32, current_time: i32) -> bool;
pub fn g2api_add_bolt(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32, bone_name: &str) -> i32;
// G2SV-D1 (ruling 18): write-through + qboolean, EXACTLY like Raven's
// `qboolean G2API_GetBoltMatrix(..., mdxaBone_t *matrix)` — the out-matrix is
// ALWAYS written (failure paths write the identity/fallback too), NOT Option.
pub fn g2api_get_bolt_matrix(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, model_index: i32, bolt_index: i32, angles: Vec3,
    position: Vec3, frame_num: i32, model_list: &[qhandle_t], scale: Vec3,
    bolt_matrix: &mut mdxaBone_t) -> bool;   // out-matrix write-through, qboolean return
pub fn g2api_collision_detect(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, angles: Vec3, position: Vec3, frame_number: i32,
    ent_num: i32, ray_start: Vec3, ray_end: Vec3, scale: Vec3, trace_flags: i32,
    use_lod: i32, f_radius: f32) -> Vec<CollisionRecord_t>; // populated collRecMap entries (mEntityNum != -1)
pub fn g2api_set_ragdoll(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, parms: &mut CRagDollParams);
pub fn g2api_animate_g2_models_rag(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, a_current_time: i32, params: &mut RagDollUpdateParams); // §F17 enum, G2SV-D8
// CRagDollUpdateParams (G2_gore.h:94) as a §F17 enum (G2SV-D8): the six data
// members + a single-variant kind. MP instantiates only the base (sv_game.cpp:1539);
// the four virtuals (:106-123) are no-op base bodies, so Server's hooks do nothing.
// SP's two subclasses (code/, out of scope) would add variants as a DEC-04 diff.
pub struct RagDollUpdateParams {
    pub angles: Vec3, pub position: Vec3, pub scale: Vec3, pub velocity: Vec3,
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
    pub fn delete(&mut self, handle: i32);        // Raven Delete()
    pub fn is_valid(&self, handle: i32) -> bool;
    pub fn get(&self, handle: i32) -> &[CGhoul2Info];
    pub fn get_mut(&mut self, handle: i32) -> &mut Vec<CGhoul2Info>;
}

// The welded bone pipeline (§F17 shape; not part of the G2API 1:1 set) — same crate.
pub struct SBoneCalc { /* newFrame, currentFrame, backlerp, blendFrame, blendOldFrame, blendMode, blendLerp */ }
pub struct CTransformBone { pub bone_matrix: mdxaBone_t, pub parent: i32, pub touch: i32, pub touch_render: i32 }
pub struct CBoneCache { /* mBones, mFinalBones, mSmoothBones, header, mod, rootBoneList, rootMatrix,
    incomingTime, mCurrentTouch/mLastTouch/mLastLastTouch, mSmoothingActive, mUnsquash, mSmoothFactor */ }
impl CBoneCache {
    // header read comes over EngineHost (loader-owned model memory, G2SV-D5); no mp_renderer type in the sig.
    pub fn new(host: &impl EngineHost, a_mod: qhandle_t) -> Self;        // ctor, seeds parents from mdxaSkel_t
    pub fn eval(&mut self, index: i32) -> mdxaBone_t;                    // memoized by touch
    pub fn eval_render(&mut self, index: i32) -> mdxaBone_t;             // applies SmoothLow
    pub fn eval_unsmooth(&mut self, index: i32) -> mdxaBone_t;
    pub fn get_parent(&self, index: i32) -> i32;
    pub fn was_rendered(&self, index: i32) -> bool;
}
pub fn g2_transform_bone(bc: &mut CBoneCache, child: i32);            // tr_ghoul2.cpp:1541
pub fn g2_construct_ghoul_skeleton(g2: &mut Ghoul2System, host: &mut impl EngineHost,
    ghoul2: &mut CGhoul2Info_v, frame_num: i32, check_for_new_origin: bool, scale: Vec3);
pub fn eval_bone_cache(g2: &mut Ghoul2System, cache: BoneCacheId, index: i32) -> mdxaBone_t;
pub fn multiply_3x4_matrix(out: &mut mdxaBone_t, in2: &mdxaBone_t, inm: &mdxaBone_t); // -ffp-contract=off (G2SV-D6)
```

*(Exact struct fields and the full `G2API_*` list are the roster's per-file
transcription target; the method-mapping table below enumerates the non-obvious
ones.)* `BoneCacheId` is the key for the hand-rolled `Ghoul2System.bone_caches`
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
observable contract the server depends on.

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

**G2SV-D4.** `_G2_LISTEN_SERVER_OPT` is never `#define`d in `codemp/`, so this doc
ports **what the WinDed macro set compiles** and §20-notes the rest:
`G2API_OverrideServerWithClientData` keeps its live `#ifndef` arm — a trivial
`return qfalse` (`G2_API.cpp:241-242`) → `-> bool { false }`; the compiled-out
override body, the `g2ClientAttachments[]` array + `AttachInstanceToEntNum`/
`ClearAttachedInstance`/`CleanEntAttachments` bodies, `CGhoul2Info::entityNum`,
and `CopyBoneCache` are dropped with §20 zero-compile notes. Because ruling 12's
build config pins the macro set and porting-rules §F20 governs dead surface.
Rejected retaining an `attachments` field for a future listen-server build: it has
zero compiled callers in this config (speculative, plan §"no dead-code").

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

`G2SV-D7`–`G2SV-D10` fold ruling 22 (2026-07-09); `G2SV-D1`–`G2SV-D6` (rendering
rulings 11–18) stand unchanged per that ruling.

**G2SV-D7.** The gore surface splits into a graph-dead *apply* set and a
server-live *record store* (resolving `G2SV-Q4`). The apply entries with zero
reachability from engine roots — `G2API_AddSkinGore` (`G2_API.cpp:2569`, only the
client `CG_G2_ADDSKINGORE` trap), `ResetGoreTag` (`G2_misc.cpp:96`, sole caller
`AddSkinGore:2590`), `G2_GetGoreRecord` (`G2_misc.cpp:113`, no caller) — get §20
zero-caller notes and no roster row. The record store — `AllocGoreRecord`/
`FindGoreRecord`/`DeleteGoreRecord`/`FindGoreSet`/`NewGoreSet`/`DeleteGoreSet`/
`CGoreSet::~CGoreSet`, `G2API_ClearSkinGore` (`G2_API.cpp:2549`, live via the
`G_G2_CLEANMODELS` trap `:545` and the save/load destruct path `:2493`; the record
store's `DeleteGoreSet` is additionally reached directly from the
`G_G2_REMOVEGHOUL2MODEL`/`MODELS` removal paths `:814,901`), and `G2_GorePolys`
(`G2_misc.cpp:804`, live via the collision `G2_TraceModels` loop `:1494`) — ports
fully into `gore.rs`. Because ruling 22 settled that gore-apply produces no
server-observable state (no populator behind a `G_G2_*` trap), so its vert-buffer/
`GoreTouch` goldens are not referee-gating (`G2SV-Q4`), while the record store is
reached by the live model-cleanup path. Rejected transcribing `AddSkinGore` as a
live seam function (the prior draft): it is graph-dead server-side (§20).
`DestroyGoreTexCoordinates`'s bucketing is escalated (`G2SV-Q8`).

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

## Verification strategy

Governing clause: porting-rules §F18 (differential goldens), DEC-09
(oracle-differential parity). Harness `tools/ghoul2-oracle/` copies the GP2
pattern (`tools/gp2-oracle/`): `run.sh` compiles the **unmodified** oracle TUs
(`codemp/ghoul2/*.cpp` + the `tr_ghoul2.cpp` bone subset) against stub headers,
`main.cpp` dumps canonical behavior over committed fixtures, goldens under
`golden/` so `cargo test` needs no C++ toolchain; Rust parity tests
(`tests/ghoul2_parity.rs` in `mp_engine_ghoul2`) mirror the dump byte-for-byte.

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
  is server-live (`G2SV-D3`, 36 arms from `SV_GameSystemCalls`).
- **Gore goldens** — `AllocGoreRecord`/`FindGoreSet` tag sequencing incl. the
  `MAX_GORE_RECORDS` eviction (`_G2_GORE` on). The gore-apply/`GoreTouch`
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
    summary: The Engine.g2 direct field (G2SV-D5, ruling 12) — fields info_array, time_bases, gbm_no_reconstruct/gbm_use_sp_method, cvars (Ghoul2Cvars, defined in cvars.rs), gore GoreState, rag RagDollSolver, bone_caches (a hand-rolled owned in-crate generational arena of CBoneCache keyed by BoneCacheId, §B5 — same kind as Ghoul2InfoArray, no external slotmap crate, G2SV-D9; + the BoneCacheId key), gore_shader qhandle_t (both folded from the former RenderG2State). Plain Default-init. One type per file (CLAUDE.md).
  - path: crates/mp/engine/ghoul2/src/cvars.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2Cvars
    summary: The ghoul2-subsystem per-subsystem EngineCvars sub-struct (ruling 2) holding the cg_g2MarksAllModels handle (G2_misc.cpp:40); registered at cvar init, read via &Ghoul2System.cvars. Own file per the one-type-per-file rule (CLAUDE.md), as Ghoul2System is a distinct type in ghoul2_system.rs.
  - path: crates/mp/engine/ghoul2/src/shared/cghoul2_info.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGhoul2Info
    summary: The per-instance CGhoul2Info class (ghoul2_shared.h:240) — a §F idiomatic reimplementation with owned Vecs (mSlist/mBltlist/mBlist), the save-serialized middle band, and runtime mBoneCache(->Option<BoneCacheId>)/mTransformedVertsArray/validity ptrs; DeepCopy zeroes runtime state. NOT the already-ported handle CGhoul2Info_v. entityNum dropped (_G2_LISTEN_SERVER_OPT off, G2SV-D4). Colocated in shared/ (mirrors ghoul2_shared.h).
  - path: crates/mp/engine/ghoul2/src/info_array.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2InfoArray
    summary: Arena + handle (Ghoul2Handle newtype colocated) + IGhoul2InfoArray impl; New/Delete/DeleteLow/IsValid/Get, TheGhoul2InfoArray accessor, Ghoul2InfoArray_Free, id-generation arithmetic bit-exact (G2SV-D6). The get/get_mut(handle) forwarding target the CGhoul2Info_v wrapper methods call into (§B4). The wrapper methods themselves colocate in shared/cghoul2_info_v.rs (G2SV-D10), NOT here. The generational-arena container is hand-rolled in-crate, not a slotmap crate (G2SV-D9).
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
    summary: AddBolt/AddBoltSurfNum/RemoveBolt/SetBoltInfo/GetBoltMatrix (write-through+bool, G2SV-D1; gG2_GBM* flags), AttachG2Model/DetachG2Model/AttachEnt/DetachEnt, SetNewOrigin. AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments bodies compile out (_G2_LISTEN_SERVER_OPT off) -> §20 note (G2SV-D4).
  - path: crates/mp/engine/ghoul2/src/api_surfaces.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API surfaces
    summary: SetSurfaceOnOff/GetSurfaceOnOff/SetRootSurface/AddSurface/RemoveSurface/GetParentSurface/GetSurfaceIndex/GetSurfaceName/GetSurfaceRenderStatus/ListSurfaces.
  - path: crates/mp/engine/ghoul2/src/api_ragdoll.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API ragdoll+IK
    summary: SetRagDoll/ResetRagDoll/AnimateG2Models(rag), RagPCJConstraint/RagPCJGradientSpeed/RagEffectorGoal/GetRagBonePos/RagEffectorKick/RagForceSolve, SetBoneIKState/IKMove, AbsurdSmoothing (server-live, G2SV-D3). params type is the RagDollUpdateParams §F17 enum (G2SV-D8, defined in ragdoll_update_params.rs).
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
    summary: The RagDoll + IK solver; fn-statics block -> RagDollSolver host fields (G2SV-D3, server-live). G2_RagDollSetup/RagDoll/RagDollSolve/SettlePositionNumeroTrois/RagSetState/IKSolve/DoIK/BoneSnap, SRagEffector.
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
  - path: crates/mp/engine/ghoul2/src/gore.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGoreSet
    summary: CGoreSet + ~CGoreSet + the server-live gore-record store (G2SV-D7): AllocGoreRecord/FindGoreRecord/DeleteGoreRecord (+ its private helper DestroyGoreTexCoordinates, reached via ~CGoreSet -> pending G2SV-Q8), FindGoreSet/NewGoreSet/DeleteGoreSet, GoreState (G2SV-D5), G2_GorePolys (live via collision trace, G2_misc.cpp:1494). ResetGoreTag/G2_GetGoreRecord are graph-dead (G2SV-D7) -> §20 notes, not ported. _G2_GORE on.
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
  - "_G2_LISTEN_SERVER_OPT OFF in the WinDed set (G2SV-D4): CGhoul2Info::entityNum, the g2ClientAttachments[] array + AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments bodies, and CopyBoneCache compile out -> §20 zero-compile notes; G2API_OverrideServerWithClientData keeps its live #ifndef arm as -> bool { false } (G2_API.cpp:241-242), taking a single CGhoul2Info (=&g2[0], sv_game.cpp:1599), NOT the CGhoul2Info_v wrapper (1:1 arity, G2SV-D6)."
  - "_XBOX OFF: CTransformBone::renderMatrix/pad, the Z_Malloc mFinalBones/mSmoothBones raw arrays, EvalFull, SetRenderMatrix dropped; the vector<> arm is the live path."
  - "_SOF2 OFF: the ghoul2_shared.h SSkinGoreData/goreEnum_t variant is dead; MP uses SSkinGoreData_s (q_shared.h:3112)."
  - "Ghoul2InfoArray::Get on an invalid handle returns a shared function-static null vector it first .clear()s (non-reentrant aliasing, G2_API.cpp:427-439) -> Rust returns an empty slice; kept out of shared fixtures (F19)."
  - "g2api_get_bolt_matrix is write-through + qboolean (G2SV-D1, ruling 18): out-matrix ALWAYS written incl. failure paths; NOT Option-returning."
  - "RenderG2State folded into Ghoul2System (G2SV-D5, ruling 12): bone_caches + gore_shader are Ghoul2System fields; the whole server-side bone pipeline is in mp_engine_ghoul2, no mp_renderer crate edge (model memory reached via EngineHost)."
  - "G2API_DEBUG destructor leak-report + g_Ghoul2Allocations/g_G2AllocTrack debug alloc tracking dropped (debug-only, no parity surface)."
  - "boneInfo_t basepose/baseposeInv/... raw mdxaBone_t* members keep their ported ABI repr but are filled from Ghoul2System-owned matrices, not shared raw pointers (B5 seam)."
  - "_DEBUG _isnan / assert paths in SmoothLow/EvalLow are debug-only; normalized out of the dumper with a comment (F19)."
  - "HackadelicOnClient (tr_ghoul2.cpp:104) is const-false server-side: its only writers are in R_AddGhoulSurfaces, #ifndef DEDICATED (:3384-3537); the render-traversal branches fold to their false arm (C10)."
  - "cgvm ragdoll-callback dead branches in G2_bones.cpp (client.h #include :32) all fold to their DEDICATED arm: Rag_Trace (:2684, #ifndef DEDICATED cgvm TRACELINE) -> real CM_BoxTrace else-arm (:2708); G2_BoneSnap (:3951, #ifdef DEDICATED return;) -> compiled no-op (caller G2_RagDollSolve :4244); the four RAG_CALLBACK_BONEINSOLID sites in G2_RagDollSettlePositionNumeroTrois (:3056,3085,3180,3217, #ifndef DEDICATED) compile out (surrounding if(params) logic stays); the :3826 site is doubly dead (#if 0 + #ifndef DEDICATED); G2_RagDebugBox/G2_RagDebugLine (:2884,2905, compiled via _DEBUG_BONE_NAMES #define :2577 but #ifdef DEDICATED return;) are compiled no-ops. No cgvm global resolves in this build (C10/§20)."
  - "RSStorage/NextRS/AllocRS render-surface pool dropped: dead server-side (sole caller tr_ghoul2.cpp:2660 is #ifndef DEDICATED); the #else non-_G2_GORE second GoreVerts (G2_misc.cpp:1088) is also dead (_G2_GORE ON)."
  - "G2_PERFORMANCE_ANALYSIS ON (FINAL_BUILD undefined) but its G2Time_*/G2PerformanceCounter_*/G2PerformanceTimer_* globals are timing instrumentation with no parity surface; dropped (F20), same as the leak-checking globals."
  - "Gore apply/record split (G2SV-D7, ruling 22): the gore-APPLY entries G2API_AddSkinGore (G2_API.cpp:2569), ResetGoreTag (G2_misc.cpp:96, sole caller AddSkinGore:2590), G2_GetGoreRecord (G2_misc.cpp:113, no caller) are graph-dead server-side (only client CG_G2_ADDSKINGORE trap, no G_G2_* arm) -> §20 zero-caller notes, not ported. The record store (AllocGoreRecord/FindGoreRecord/DeleteGoreRecord, FindGoreSet/NewGoreSet/DeleteGoreSet, ~CGoreSet, G2API_ClearSkinGore, G2_GorePolys) is server-live (ClearSkinGore via G_G2_CLEANMODELS :545 + the save/load destruct path :2493; the record-store DeleteGoreSet also reached directly from the REMOVEGHOUL2MODEL/MODELS removal paths :814,901; GorePolys via collision trace G2_misc.cpp:1494) and ports fully. AddSkinGore vert-buffer/GoreTouch goldens are not M3-gating (G2SV-Q4)."
  - "CRagDollUpdateParams (G2_gore.h:94) reimplemented as the §F17 RagDollUpdateParams enum (G2SV-D8, ruling 22), NOT a vtable class: MP instantiates only the base (sv_game.cpp:1539); the four virtuals (:106-123) are no-op base bodies, so the sole MP variant RagDollUpdateKind::Server has no-op hooks and params->RagDollSettled() (G2_bones.cpp:2505) matches to nothing. Distinct from the already-ported plain-data sharedRagDollUpdateParams_t. SP's two subclasses (code/) are a future DEC-04 diff."
  - "CGhoul2Info_v forwarding/lifecycle methods (ghoul2_shared.h:335-435) colocate in shared/cghoul2_info_v.rs (G2SV-D10, ruling 22, §F21), not info_array.rs; the #[repr(C)] struct layout (mItem: i32) stays frozen, only the impl is added; methods forward into Ghoul2InfoArray."
  - "Ghoul2System.bone_caches is a hand-rolled in-crate generational arena (BoneCacheId), matching Ghoul2InfoArray's bit-exact handle scheme (§B5), NOT an external slotmap crate (G2SV-D9, ruling 22)."
```

## Method transcription table

Anchors for the non-obvious internal + renderer methods (the full 1:1 `G2API_*`
surface is `G2_local.h:96-224`, mapped file-by-file in the roster; §F21). Each
row is one transcription target.

| Raven symbol | oracle cite | Rust file | notes |
|---|---|---|---|
| `Ghoul2InfoArray::New/Delete/DeleteLow/IsValid/Get` | `G2_API.cpp:386,413,315,399,427` | `info_array.rs` | id/generation arithmetic bit-exact (G2SV-D6) |
| `TheGhoul2InfoArray` / `Ghoul2InfoArray_Free` | `G2_API.cpp:477-493` | `info_array.rs` | lazy singleton → owned `Ghoul2System` field |
| `CGhoul2Info_v::operator[]/size/resize/push_back` (borrow-wrapper) | `ghoul2_shared.h:399-435` | `shared/cghoul2_info_v.rs` | colocated with the frozen struct (G2SV-D10); forwards through `get`/`get_mut(handle)` (§B4) |
| `CGhoul2Info_v::Alloc/Free/clear/DeepCopy/operator=` (lifecycle) | `ghoul2_shared.h:335-435` (`clear` at `:426`) | `shared/cghoul2_info_v.rs` | colocated (G2SV-D10); `New`/`Delete` + DeepCopy runtime-state zeroing (`:385-397`); `operator=` = handle copy |
| `G2API_GetBoltMatrix` | `G2_API.cpp:1795` | `api_bolts.rs` | write-through out-matrix + `bool` (G2SV-D1) |
| `G2API_OverrideServerWithClientData` | `G2_API.cpp:239` | `api_collision.rs` | takes a single `CGhoul2Info *serverInstance` (`=&g2[0]`, sv_game.cpp:1599), NOT the wrapper (1:1, G2SV-D6); WinDed live arm = `return qfalse` (G2SV-D4) |
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
| `AllocGoreRecord/FindGoreRecord/DeleteGoreRecord` | `G2_misc.cpp:58,103,118` | `gore.rs` | server-live record store (G2SV-D7); `MAX_GORE_RECORDS` eviction |
| `DestroyGoreTexCoordinates` | `G2_misc.cpp:43` | `gore.rs` | private helper of `DeleteGoreRecord` (`:120`), reached via `~CGoreSet`; ruling-22 graph-dead bucketing conflicts (G2SV-Q8) |
| `FindGoreSet/NewGoreSet/DeleteGoreSet`, `CGoreSet::~CGoreSet` | `G2_misc.cpp:127,142,153,174` | `gore.rs` | server-live (G2SV-D7); `_G2_GORE` on |
| `G2API_ClearSkinGore` | `G2_API.cpp:2549` | `api_gore.rs` | server-live via `G_G2_CLEANMODELS` (`:545`) + save/load destruct (`:2493`) (G2SV-D7); drives `DeleteGoreSet` (`:2557`), which the `REMOVEGHOUL2MODEL`/`MODELS` paths (`:814,901`) also call directly |
| `G2API_AddSkinGore/ResetGoreTag/G2_GetGoreRecord` | `G2_API.cpp:2569`, `G2_misc.cpp:96,113` | — (dropped) | graph-dead server-side (G2SV-D7) → §20 zero-caller notes; no roster row |
| `RagDollUpdateParams` (`CRagDollUpdateParams` §F17 enum) | `G2_gore.h:94` | `ragdoll_update_params.rs` | single MP variant `Server`; `params->RagDollSettled()` (`G2_bones.cpp:2505`) → no-op `match` (G2SV-D8) |

## Slice hooks

- **M3 waves 13–19** (`GOAL-engine.md:71`) — "renderer, RMG, botlib, ghoul2
  complete"; gate = the bone/bolt/collision goldens above. Needs frozen first:
  the already-ported `shared/` + `gore/` layout types (done), and the **Stage-0
  `EngineHost` trait** (ruling 11) carrying all four consumed services —
  model-memory read (`mdxaHeader_t`/`mdxaSkel_t`), trace, print/error, cvar
  register/read. The host-consuming bodies (`render/bone_cache.rs` ctor,
  `render/skeleton.rs`, `misc.rs` trace, `api_gore.rs`/`cvars.rs` cvar
  registration, print/error paths) are transcribed against that frozen trait; their
  §F signatures freeze in this doc, their method bodies wait on Stage-0 (non-goals,
  sequencing consequence).
- **`SV_GameSystemCalls`** (wave 20, plan §"server is the integrator") — the
  server→ghoul2 edges call the `G2API_*` surface frozen by `G2SV-D6` (incl. the 12
  rag/IK arms, `G2SV-D3`); that seam must be stable before the switch arm ports.

## Resolved questions

Questions the 2026-07-09 fork session and its evidence queries closed (were open
in the prior draft), plus `G2SV-Q4` and `G2SV-Q5` closed by ruling 22
(`G2SV-D7`/`G2SV-D8`):
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

## Open questions

- **`G2SV-Q8`** — `DestroyGoreTexCoordinates` (`G2_misc.cpp:43`) bucketing. Ruling 22
  as transcribed into this revision lists it in the **graph-dead** gore-apply set
  ("zero reachability from engine roots"), but oracle ground truth shows it **is**
  reached server-side: it is the private helper `DeleteGoreRecord` calls (`:120`),
  and `DeleteGoreRecord` is invoked by `CGoreSet::~CGoreSet` (`:179`) ←
  `DeleteGoreSet` (its call at `G2_API.cpp:2557` inside the **live** `G2API_ClearSkinGore`,
  reached behind `G_G2_CLEANMODELS`; the `G_G2_REMOVEGHOUL2MODEL`/`MODELS` paths reach
  `DeleteGoreSet` directly at `:814,901`). It compiles
  (`#ifdef _G2_GORE`, ON, `G2_misc.cpp:26`). Only the runtime container it iterates
  (`CGoreSet::mGoreRecords`) is always empty server-side, because its sole populator
  is the graph-dead apply path — so it never does observable work, but it is **not**
  statically graph-dead. This revision ports it as a live private helper of the
  record store in `gore.rs` (the `~CGoreSet` path needs it) rather than §20-noting
  it, but the classification conflicts with the ruling's transcribed wording. A
  drafting agent must not self-resolve a ruling/ground-truth conflict: **escalate**
  for the user to confirm whether `DestroyGoreTexCoordinates` ports with the record
  store (this revision's reading) or is genuinely dropped. Not M3-gating either way
  (gore-apply is out of the referee surface, `G2SV-Q4`).
