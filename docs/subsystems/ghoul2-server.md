# Server-side Ghoul2 + renderer bone/model internals Design
Status: DRAFT     Supersedes: none
Decision prefix: G2SV     Ledger deps: DEC-04 (per-mode), DEC-09 (verification)

This is a C++-track (`porting-rules.md` §F) design doc. It carries the machine-
readable `files:` roster and `divergences:` list (doc-standards rule 6) that
`.claude/workflows/port-cpp-subsystem.js` consumes via `designPath`; both live
under `## Files roster` and `## Divergences` below.

## Standing context

Links only — never restated here:
- `docs/workspace-architecture.md` — crate graph; target crates `mp_engine_ghoul2`
  (`crates/mp/engine/ghoul2`) and `mp_renderer` (`crates/mp/renderer`).
- `docs/porting-rules.md` — §B (state spine), §F (C++ track: §F17 shape-first,
  §F18 differential goldens, §F19 UB divergence, §F20 drop-dead-surface, §F21
  one-class-per-file), the comment/source-ref rules.
- `docs/decisions.md` — DEC-04 (per-mode), DEC-09 (oracle-differential parity).
- `docs/handoffs/engine-fork-discovery.md` — fork-2 (global state → `Engine`
  sub-structs), fork-3 (function-statics → jampgame three-kind rule); ghoul2 is
  named there (fork 7) as one of the five §F design-doc subsystems.
- `docs/architecture/state-ownership.md` (**FROZEN**) — the master state spine and
  the two-island model; **STATE-Q2** is the still-open question of whether/how the
  four §F engine subcrates (botlib/ghoul2/icarus/rmg) attach their engine-side
  state to the `Engine` island. `Engine::new`'s zeroed-alloc constructor has **no
  `g2` field yet** (`crates/mp/engine/core/src/engine.rs:35-36`, STATE-Q2 caveat).
  This doc's `Engine`-attachment (`G2SV-D5`) is gated on it — see `G2SV-Q6`.
- `docs/plans/2026-07-08-mp-engine-build-out.md` — the WinDed DEDICATED link set,
  the ghoul2⇄renderer weld (§"ghoul2 ⇄ renderer entanglement is structural").
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
  (incl. the RagDoll + IK solver), `G2_bolts.cpp`, `G2_surfaces.cpp`,
  `G2_misc.cpp`, the `CGoreSet`/gore-record store (`G2_gore.h` + the gore parts
  of `G2_misc.cpp`).
- The welded renderer bone/model subset in `oracle/codemp/renderer/tr_ghoul2.cpp`
  (5,509 LOC): `CBoneCache`, `CTransformBone`, `SBoneCalc`, `G2_TransformBone`
  (`tr_ghoul2.cpp:1541`), `G2_TransformGhoulBones` (`:2075`),
  `G2_ConstructGhoulSkeleton` (`:3567`), `EvalBoneCache` (`:585`), the low-level
  matrix helpers (`Multiply_3x4Matrix`, `G2_CreateQuaterion`,
  `G2_CreateMatrixFromQuaterion`), and the `mdxa*` bone
  accessors the server's collision path needs (`G2_GetBoneMatrixLow`,
  `G2_GetBoneBasepose`). These live in `renderer/` but are pure bone math the
  dedicated server links for collision (plan §"ghoul2 ⇄ renderer entanglement").
- The `Ghoul2InfoArray` singleton → owned arena (`G2SV-D1`).

**Non-goals (punted, with pointers).**
- The `mdxa`/`mdxm` model, mesh, and shader **loader** (`tr_model.cpp`,
  `tr_mesh.cpp`, `tr_surface.cpp` GL-draw arms). The bone pipeline *reads*
  `mdxaHeader_t`/`mdxaSkel_t` out of model memory that loader owns; the exact fn
  boundary between "model data the server needs" and "GL drawing it doesn't"
  runs through `#ifndef DEDICATED` inside individual files, not along file lines
  (plan §"entanglement", point 3). Which of those functions land in *this*
  doc vs. a `tr_model` subsystem doc is **`G2SV-Q1`** — unresolved.
- SP ghoul2 (`oracle/code/ghoul2/`, `jasp` engine, statically linked). This doc
  is the MP/`jamp` server slice only; every roster entry is `mode: mp`. SP is a
  future diff per DEC-04 / porting-rules §F20 (duplicate, don't unify).
- Client/`cgame`-side rendering (`RB_SurfaceGhoul`, `R_AddGhoulSurfaces`) —
  compiled out under `DEDICATED`.

## Raven ground truth

**Build config (the WinDed DEDICATED Release macro set).** From the plan
appendix (`docs/plans/2026-07-08-mp-engine-build-out.md:570`): `-DNDEBUG
-DDEDICATED -DBOTLIB`, `FINAL_BUILD` undefined, no platform macro. On top of
that, source-level defines decide the ghoul2 `#ifdef` map:
- `_G2_GORE` — **ON**. Defined at `oracle/codemp/game/q_shared.h:3110` (a source
  `#define`, reached through the `q_shared.h` include chain), not a vcproj macro.
  So all gore code (`CGoreSet`, `GoreTextureCoordinates`, `G2API_AddSkinGore`,
  `mGoreSetTag`, `goreShader`) **is compiled** in MP.
- `_G2_LISTEN_SERVER_OPT` — **OFF**. No `#define` exists anywhere in `codemp/`.
  `CGhoul2Info::entityNum` (`ghoul2_shared.h:277`), `G2API_OverrideServerWith‑
  ClientData` (`G2_API.cpp:239`), and `CopyBoneCache` (`tr_ghoul2.cpp:579`)
  compile out.
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
`int mItem` (the handle) and forwarding `[]`/`resize`/`size`/`push_back` through
`TheGhoul2InfoArray().Get(mItem)` — this is exactly the §B5 arena + id +
borrow-wrapper triad. `Get()` on an invalid handle returns a shared
function-`static` `null` vector it first `.clear()`s (`:427-439`) — a
non-reentrant aliasing hack.

**Per-instance state that is NOT in the arena.** Each `CGhoul2Info`
(`ghoul2_shared.h:240`) owns three STL vectors (`mSlist`, `mBltlist`, `mBlist`),
a save-serialized middle band (`mModelindex`..`mFlags`, incl. `mGoreSetTag`
under `_G2_GORE`), and non-serialized runtime pointers: `mTransformedVertsArray`,
`CBoneCache *mBoneCache` (`:265`), and validity/model pointers set by
`G2_SetupModelPointers` (`G2_misc.cpp:1839`, `tr_ghoul2.cpp:107`). `DeepCopy`
(`:382`) copies the vector but zeroes `mBoneCache`, `mTransformedVertsArray`,
`mSkelFrameNum`, `mMeshFrameNum` on every element — runtime state is per-instance
and never shared across a copy.

**The bone cache (render side).** `CBoneCache` (`tr_ghoul2.cpp:206`) is built per
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
dedicated-awareness is redundant here (`!com_dedicated->integer` is also false
server-side). It is therefore not threaded state; a porter may fold the dead
false arms (§C10).

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
(...,&gore,qtrue) :2601`, using `G2VertSpaceServer`, no DEDICATED guard). Separately
`tr_ghoul2.cpp:866-867` holds a render-surface pool `RSStorage
[MAX_RENDER_SURFACES=2048]` with a rolling `NextRS` cursor handed out by
`AllocRS()` (`:869`); its sole caller (`:2660`) is inside `#ifndef DEDICATED`
(`:2520-2736`), so the pool is dead in the DEDICATED build. (The `#else`
non-`_G2_GORE` second `GoreVerts` at `:1088` is dead — `_G2_GORE` is ON.)

**Const bone-name tables.** `tr_ghoul2.cpp` carries read-only bone-name lookup
tables for the skeleton/remap helpers: `rootParents`/`otherParents`/`bottomBones`
(`:5061-5097`) and the null-terminated `BoneHierarchyList` (`:5173`), plus
`OldToNewRemapTable[72]` (`:4469`, declared non-`const` `int[]` but only ever
read, `:5034`). Like `identityMatrix` (`:128`) these are the const-table kind of
the three-kind rule.

**The RagDoll + IK solver (the fn-statics).** `G2_bones.cpp:1214-1241` is one
block of file-scope statics, sized `MAX_BONES_RAG=256` (`:1163`), that the solver
reuses across the multi-pass settle: the parallel per-bone arrays `ragBasepose`,
`ragBaseposeInv`, `ragBones`, `ragEffectors` (`SRagEffector`, `:1165`),
`ragBoneData`, `tempDependents`, `ragBlistIndex`; the scalars/vectors `numRags`,
`ragBoneMins`/`ragBoneMaxs`/`ragBoneCM`, `haveDesiredPelvisOffset`,
`desiredPelvisOffset`, `ragOriginChange`, `ragOriginChangeDir`, `handPos`,
`handPos2`, `ragState`; and `vector<boneInfo_t*> rag` (`:1241`). These are read
and written across `G2_RagDollSetup` (`:2254`), `G2_RagDoll` (`:2403`),
`G2_RagDollCurrentPosition` (`:2609`), `G2_RagDollSettlePositionNumeroTrois`
(`:2927`/`:3449`), `G2_RagDollSolve` (`:3970`), and the IK arm
(`G2_IKSolve` `:4297`, `G2_DoIK` `:4453`) — cross-call, cross-frame state, not
per-invocation scratch. (Additional `static const` matrices and `static`
locals inside individual solver functions — e.g. the identity `id` at `:1423`,
the settle-pass locals at `:3452-3475` — are the const-table / scratch kinds of
the three-kind rule, not persistent state.)

**API-level globals (`G2_API.cpp`).** `g2ClientAttachments[MAX_GENTITIES]`
(`:197`, `CGhoul2Info_v*` per entity, listen-server attachment), `G2TimeBases`
(`int[NUM_G2T_TIME]`, `:160`, driving `G2API_SetTime`/`GetTime` `:162`),
`gG2_GBMNoReconstruct`/`gG2_GBMUseSPMethod` (`:1724-1725`, `GetBoltMatrix`
reconstruct-skip flags), plus debug-only `g_Ghoul2Allocations`/`g_G2ServerAlloc`/`g_G2ClientAlloc`/
`g_G2AllocServer` (`:34-37`), `g_G2AllocTrack`/`g_G2AllocTrackInit` (`:43-44`,
`MAX_TRACKED_ALLOC=4096`) — all under `_FULL_G2_LEAK_CHECKING` — and the
`_DEBUG`-only `g_goreAllocs`/`g_goreTexAllocs` (`G2_misc.cpp:138-139`).

## State ownership

Every global the survey found. Owner placement follows fork-2 (globals →
`Engine` sub-structs grouped by owning `.c` file) and fork-3 (fn-statics → the
three-kind rule); `G2SV-D5` names the two sub-structs. `Engine.g2:
Ghoul2System` lives in `mp_engine_ghoul2`; the render-side bone state lives in
`mp_renderer`. **The `&mut Engine.g2` threading column below is the intended
target contingent on STATE-Q2** — the FROZEN `state-ownership.md` has not yet
attached the four §F subcrates' state to the `Engine` island, and `Engine::new`
has no `g2` field today (`crates/mp/engine/core/src/engine.rs:35-36`). Read every
"`&mut Engine.g2`"/"`Engine.g2` init" cell as "the `Ghoul2System` value, reached
by whatever channel STATE-Q2 pins" — see `G2SV-D5` scope note and `G2SV-Q6`.

| Raven global | oracle cite | Rust owner | constructed by | threaded via |
|---|---|---|---|---|
| `Ghoul2InfoArray *singleton` | `G2_API.cpp:477` | `mp_engine_ghoul2::Ghoul2System.info_array: Ghoul2InfoArray` | lazily on first `the_info_array()`; freed by `Ghoul2InfoArray_Free` | `&mut Engine.g2` into every `G2API_*` |
| `g2ClientAttachments[MAX_GENTITIES]` | `G2_API.cpp:197` | `Ghoul2System.attachments: [Option<Ghoul2Handle>; MAX_GENTITIES]` (see `G2SV-Q3`) | `Engine.g2` init | `&mut Engine.g2` |
| `G2TimeBases[NUM_G2T_TIME]` | `G2_API.cpp:160` | `Ghoul2System.time_bases: [i32; NUM_G2T_TIME]` | `Engine.g2` init | `&mut Engine.g2` |
| `gG2_GBMNoReconstruct`, `gG2_GBMUseSPMethod` | `G2_API.cpp:1724-1725` | `Ghoul2System.gbm_no_reconstruct/gbm_use_sp_method: bool` | `Engine.g2` init | `&mut Engine.g2` |
| `g_Ghoul2Allocations`, `g_G2ServerAlloc`, `g_G2ClientAlloc`, `g_G2AllocServer`, `g_G2AllocTrack*`, `g_G2AllocTrackInit` (all `_FULL_G2_LEAK_CHECKING`); `g_goreAllocs`, `g_goreTexAllocs` (`_DEBUG`) | `G2_API.cpp:34-37,43-44`, `G2_misc.cpp:138-139` | dropped (debug alloc tracking, no parity surface, §F20) | — | — (divergences list) |
| `G2Time_*`, `G2PerformanceCounter_*`, `G2PerformanceTimer_*` (`G2_PERFORMANCE_ANALYSIS`, ON) | `tr_ghoul2.cpp:42-62` | dropped (timing instrumentation, no parity surface, §F20 — same treatment as the leak-checking globals) | — | — (divergences list) |
| `GoreRecords`, `GoreTagsTemp`, `CurrentTag`, `CurrentTagUpper` | `G2_misc.cpp:35,36,32,33` | `Ghoul2System.gore: GoreState { records: BTreeMap<i32, GoreTextureCoordinates>, tags_temp, current_tag, current_tag_upper }` | `Engine.g2` init | `&mut Engine.g2.gore` |
| `GoreSets`, `CurrentGoreSet` | `G2_misc.cpp:125,124` | `GoreState.sets: BTreeMap<i32, CGoreSet>`, `GoreState.current_set` | `Engine.g2` init | `&mut Engine.g2.gore` |
| `GoreTouch` (persistent gen counter) | `G2_misc.cpp:795` | `GoreState.gore_touch: i32` (fork-2, same file/subsystem as the gore store; three-kind persistent). Server-reachable via the collision path (`:890`); load-bearingness `G2SV-Q4` | `Engine.g2` init | `&mut Engine.g2.gore` |
| `GoreVerts`, `GoreIndexCopy`, `GoreIndecies` | `G2_misc.cpp:793,794,798` | scratch buffers (three-kind scratch; per-`G2_GorePolys` rebuild, invalidated by `gore_touch`) — impl-local, not a global; server load-bearingness pending `G2SV-Q4` | — | — |
| `goreModelIndex` | `G2_misc.cpp:38` | scratch (three-kind scratch; set in the `G2_TraceModels` model loop `:1539`, read as the `GoreTagsTemp` key `:959,1000`) — impl-local, threaded through the trace, not a global; server load-bearingness pending `G2SV-Q4` | — | — |
| `cg_g2MarksAllModels` | `G2_misc.cpp:40` | `Ghoul2System.cvars: Ghoul2Cvars` (EngineCvars per fork-2) | cvar registration | `&Engine.g2.cvars` |
| RagDoll fn-statics block (`ragBasepose`…`rag`) | `G2_bones.cpp:1214-1241` | `Ghoul2System.rag: RagDollSolver { basepose, basepose_inv, bones, effectors, bone_data, temp_dependents, blist_index, num_rags, bone_mins/maxs/cm, desired_pelvis_offset, have_desired_pelvis_offset, origin_change, origin_change_dir, hand_pos, hand_pos2, rag_state, rag: Vec<..> }` | `Engine.g2` init | `&mut Engine.g2.rag` (fork-3 cross-frame kind, `G2SV-D2`) |
| solver `static const` matrices / settle-pass `static` locals | `G2_bones.cpp:1423,3452-3475` | `const` items / function locals (three-kind rule: const-table / scratch) | — | — |
| `CBoneCache *mBoneCache` per instance | `ghoul2_shared.h:265` | `mp_renderer::RenderG2State.bone_caches: SlotMap<BoneCacheId, CBoneCache>`; `CGhoul2Info.mBoneCache` → `BoneCacheId` (keyed by model instance, `G2SV-D5`) | `G2_ConstructGhoulSkeleton` on demand; freed by `Ghoul2InfoArray::delete_low` | `&mut RenderG2State` |
| `worldMatrix`, `worldMatrixInv` | `tr_ghoul2.cpp:136-137` | per-construct scratch threaded through the skeleton build (three-kind: scratch), NOT a global | set by `G2_GenerateWorldMatrix` | passed into the transform chain |
| `identityMatrix` | `tr_ghoul2.cpp:128` | `const` item | — | — |
| `rootParents`, `otherParents`, `bottomBones`, `BoneHierarchyList`, `OldToNewRemapTable` | `tr_ghoul2.cpp:5061-5097,5173,4469` | `const` items (three-kind const-table, as `identityMatrix`; `OldToNewRemapTable` is decl'd non-`const` but read-only, `:5034`) | — | — |
| `HackadelicOnClient` (render-traversal flag) | `tr_ghoul2.cpp:104` | none — const-`false` server-side (only writers in `R_AddGhoulSurfaces`, `#ifndef DEDICATED` `:3384-3537`); reads take the false arm (§C10 fold) | — | — (divergences list) |
| `RSStorage`, `NextRS` (render-surface pool) | `tr_ghoul2.cpp:866-867` | dropped — dead server-side (`AllocRS` sole caller `:2660` is `#ifndef DEDICATED`) | — | — (divergences list) |
| `goreShader` | `tr_ghoul2.cpp:139` (`_G2_GORE`) | `RenderG2State.gore_shader: qhandle_t` | render init | `&mut RenderG2State` |

## Seam definition

Per doc-standards rule 5 the pub signatures freeze here; porters transcribe into
them without changing them.

**ABI-crossing / already-ported types (imported, never re-declared).** `#[repr(C)]`
layout-frozen: `CGhoul2Info_v` (the 4-byte arena **handle**, `mItem: i32`,
`crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs` — **not** the per-instance
class), `boneInfo_t`, `boltInfo_t`, `surfaceInfo_t`, `mdxaBone_t`, `mdxaHeader_t`,
`mdxaSkel_t`, `SSkinGoreData`, `CRagDollParams`, `SRagDollEffectorCollision`,
`EG2_Collision`, and `CollisionRecord_t` (`crates/mp/qshared/src/shared/collision.rs`,
with `G2Trace_t = [CollisionRecord_t; MAX_G2_COLLISIONS]`, `MAX_G2_COLLISIONS = 16`;
`mEntityNum == -1` = unused record). **Not** already ported (this doc defines them,
see the roster): the per-instance **class** `CGhoul2Info` (`ghoul2_shared.h:240`,
a §F idiomatic reimplementation with owned `Vec`s — only the handle `CGhoul2Info_v`
above is layout-frozen) and `CRagDollUpdateParams` (`G2_gore.h:94`, a virtual-method
C++ class distinct from the already-ported plain-data `sharedRagDollUpdateParams_t`
— its §F17 shape is **unsettled**, `G2SV-Q5`). The ragdoll
pointer members of `boneInfo_t` (`basepose`, `baseposeInv`, `baseposeParent`,
`baseposeInvParent`, `ghoul2_shared.h:109-112`) keep their ported repr (they are
inside an ABI struct); the render code fills them from `RenderG2State`-owned
matrices (see `G2SV-Q2` for whether that path runs server-side).

**Traps.** This subsystem crosses no `trap_*`/syscall boundary itself — it is
engine-internal C++ reached from `SV_GameSystemCalls` (`sv_game.cpp`) which
already sits above the interface-crate seam (fork-5, plan §"VM dispatch"). The
consumer surface is the `G2API_*` free-function set (`G2_local.h:96-224`), kept
**1:1 in signature** (`G2SV-D3`) because those names are the switch targets the
server calls. Illustrative frozen signatures (Rust idiom per porting-rules §C7:
out-params → returns, `qboolean` → `bool`), all taking `&mut Ghoul2System` (or
`&mut Engine`) as the threaded world:

```rust
// mp_engine_ghoul2 — the syscall-switch target surface (G2SV-D3, 1:1 with G2_local.h)
pub fn g2api_init_ghoul2_model(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v,
    file_name: &str, model_index: i32, custom_skin: qhandle_t,
    custom_shader: qhandle_t, model_flags: i32, lod_bias: i32) -> i32;
pub fn g2api_remove_ghoul2_model(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32) -> bool;
pub fn g2api_set_bone_anim(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32,
    bone_name: &str, start_frame: i32, end_frame: i32, flags: i32, anim_speed: f32,
    current_time: i32, set_frame: f32, blend_time: i32) -> bool;
pub fn g2api_set_bone_angles(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32,
    bone_name: &str, angles: Vec3, flags: i32, up: Eorientations, left: Eorientations,
    forward: Eorientations, model_list: &[qhandle_t], blend_time: i32, current_time: i32) -> bool;
pub fn g2api_add_bolt(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, model_index: i32, bone_name: &str) -> i32;
pub fn g2api_get_bolt_matrix(g2: &mut Ghoul2System, r: &mut RenderG2State, ghoul2: &mut CGhoul2Info_v,
    model_index: i32, bolt_index: i32, angles: Vec3, position: Vec3, frame_num: i32,
    model_list: &[qhandle_t], scale: Vec3) -> Option<mdxaBone_t>;   // out-param mdxaBone_t* -> return
pub fn g2api_collision_detect(g2: &mut Ghoul2System, r: &mut RenderG2State, ghoul2: &mut CGhoul2Info_v,
    angles: Vec3, position: Vec3, frame_number: i32, ent_num: i32, ray_start: Vec3, ray_end: Vec3,
    scale: Vec3, trace_flags: i32, use_lod: i32, f_radius: f32) -> Vec<CollisionRecord_t>; // populated collRecMap entries (the already-ported CollisionRecord_t, G2Trace_t members with mEntityNum != -1), not a new wrapper
pub fn g2api_set_ragdoll(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, parms: &mut CRagDollParams);
pub fn g2api_animate_g2_models_rag(g2: &mut Ghoul2System, r: &mut RenderG2State,
    ghoul2: &mut CGhoul2Info_v, a_current_time: i32, params: &mut CRagDollUpdateParams);
pub fn g2api_add_skin_gore(g2: &mut Ghoul2System, ghoul2: &mut CGhoul2Info_v, gore: &SSkinGoreData); // _G2_GORE
pub fn g2api_set_time(g2: &mut Ghoul2System, current_time: i32, clock: i32);
pub fn g2api_get_time(g2: &Ghoul2System, arg_time: i32) -> i32;

// The arena (G2SV-D1): §B5 arena + handle + copyable borrow wrapper.
pub struct Ghoul2Handle(pub i32);                 // packs slot | generation, id scheme frozen (G2SV-D4)
pub struct Ghoul2InfoArray { /* mInfos, mIds, free_indices */ }
impl Ghoul2InfoArray {
    pub fn new_handle(&mut self) -> i32;          // Raven New()
    pub fn delete(&mut self, handle: i32);        // Raven Delete()
    pub fn is_valid(&self, handle: i32) -> bool;
    pub fn get(&self, handle: i32) -> &[CGhoul2Info];
    pub fn get_mut(&mut self, handle: i32) -> &mut Vec<CGhoul2Info>;
}
```

```rust
// mp_renderer — the welded bone pipeline (§F17 shape; not part of the G2API 1:1 set)
pub struct SBoneCalc { /* newFrame, currentFrame, backlerp, blendFrame, blendOldFrame, blendMode, blendLerp */ }
pub struct CTransformBone { pub bone_matrix: mdxaBone_t, pub parent: i32, pub touch: i32, pub touch_render: i32 }
pub struct CBoneCache { /* mBones, mFinalBones, mSmoothBones, header, mod, rootBoneList, rootMatrix,
    incomingTime, mCurrentTouch/mLastTouch/mLastLastTouch, mSmoothingActive, mUnsquash, mSmoothFactor */ }
impl CBoneCache {
    pub fn new(a_mod: ModelId, header: MdxaHeaderRef) -> Self;         // ctor, seeds parents from mdxaSkel_t
    pub fn eval(&mut self, index: i32) -> mdxaBone_t;                    // memoized by touch
    pub fn eval_render(&mut self, index: i32) -> mdxaBone_t;             // applies SmoothLow
    pub fn eval_unsmooth(&mut self, index: i32) -> mdxaBone_t;
    pub fn get_parent(&self, index: i32) -> i32;
    pub fn was_rendered(&self, index: i32) -> bool;
}
pub fn g2_transform_bone(bc: &mut CBoneCache, child: i32);            // tr_ghoul2.cpp:1541
pub fn g2_construct_ghoul_skeleton(g2: &mut Ghoul2System, r: &mut RenderG2State,
    ghoul2: &mut CGhoul2Info_v, frame_num: i32, check_for_new_origin: bool, scale: Vec3);
pub fn eval_bone_cache(r: &mut RenderG2State, cache: BoneCacheId, index: i32) -> mdxaBone_t;
pub fn multiply_3x4_matrix(out: &mut mdxaBone_t, in2: &mdxaBone_t, inm: &mdxaBone_t); // -ffp-contract=off (G2SV-D4)
```

*(Exact struct fields and the full ~130-entry `G2API_*` list are the roster's
per-file transcription target; the method-mapping table below enumerates them.)*

**Gated seam types (`G2SV-Q1`).** `ModelId` and `MdxaHeaderRef` (the `CBoneCache::new`
args) are **placeholder names** for a model handle and a borrow of loader-owned
`mdxaHeader_t` model memory; their concrete shape and owning file sit on the
renderer-subset boundary that `G2SV-Q1` leaves unresolved, so those two seam lines
are **provisional pending `G2SV-Q1`** — `bone_cache.rs`/`skeleton.rs` cannot be
written concretely until it settles. `BoneCacheId` is the `SlotMap` key for
`RenderG2State.bone_caches` introduced by this doc (`G2SV-D5`), defined with
`RenderG2State` (roster); it is **not** gated on `G2SV-Q1`.

**Crate-edge direction (`G2SV-Q7`).** These signatures cross the
`mp_engine_ghoul2`⇄`mp_renderer` split **both ways** — the `mp_engine_ghoul2`
entries above take `r: &mut RenderG2State` (a `mp_renderer` type) while the
`mp_renderer` block takes `g2: &mut Ghoul2System` / `ghoul2: &mut CGhoul2Info_v`
(both `mp_engine_ghoul2` types), a `[dependencies]` cycle Cargo rejects. Which
crate is upstream (or whether a third shared crate hosts the crossing types) is
**unresolved** — see `G2SV-Q7`. The individual signatures stand; only the
`[dependencies]` edge between the two crates is gated.

## Decisions

**G2SV-D1.** The `Ghoul2InfoArray` singleton becomes the §B5 arena + handle +
copyable borrow-wrapper pattern; `CGhoul2Info_v` keeps its already-ported layout.
Because consumers pass `CGhoul2Info_v` by value and index it while it aliases one
shared table (`ghoul2_shared.h:328-457`) — the exact case §B5/§17 names. Rejected
an owned tree per instance: it would break the copy-is-a-handle semantics
`DeepCopy`/`kill`/`operator=` depend on.

**G2SV-D2.** The RagDoll/IK fn-statics (`G2_bones.cpp:1214-1241`) become fields
on an owned `RagDollSolver` host struct, per the fork-3 three-kind rule (blessed
in `engine-fork-discovery.md` fork 3). Because they are read/written across
`G2_RagDollSetup`→`G2_RagDoll`→`…Solve` within and across frames — genuine
cross-frame state, not per-call scratch. Rejected `thread_local`/returned
buffers: the arrays persist the settle across calls, so they are the "host-struct
fields" kind, not the "scratch" kind.

**G2SV-D3.** The `G2API_*` surface (`G2_local.h:96-224`) keeps exact 1:1
signatures. Because it is the target set of the `SV_GameSystemCalls` switch — the
seam the server dispatches through — so name/arity/order must round-trip.
Rejected consolidating overloads (e.g. the two `G2API_SetBoltInfo`): the switch
references them individually.

**G2SV-D4.** Bone math uses the `-ffp-contract=off` parity class; goldens are
bit-exact per porting-rules §F18. Because `Multiply_3x4Matrix`, the
quaternion↔matrix conversions, and the LERP/normalize passes accumulate across
the recursive bone walk, and FMA contraction would diverge the low bits the
referee diffs. Also frozen bit-exact: the `Ghoul2InfoArray` handle/generation
arithmetic (`G2_MODEL_BITS`, rollover at `:330`) so handle values match the
oracle. Rejected "close enough" float tolerance: parity is byte-for-byte (§A1).

**G2SV-D5.** State lands on `Engine` sub-structs per fork-2: `Engine.g2:
Ghoul2System` (arena, attachments, time bases, GBM flags, `gore: GoreState`,
`rag: RagDollSolver`) in `mp_engine_ghoul2`; the bone caches (keyed by model
instance) live in `mp_renderer`'s render state and are reached by
`BoneCacheId` from `CGhoul2Info`. Because fork-2 groups globals by owning `.c`
file and the two dirs are separate crates in the weld. Rejected one flat God
struct: the ghoul2⇄renderer crate split (66/11 edges) is the natural seam.

*Scope of what D5 settles vs. `G2SV-Q6`.* D5 fixes the **crate-level** ownership —
`Ghoul2System` is a `mp_engine_ghoul2` type, `RenderG2State`/bone caches are
`mp_renderer` — and that the `G2API_*` seam threads a `&mut Ghoul2System` (its
existence as a type is not in doubt; roster gives its file). D5 does **not** settle
the `Engine`-**island attachment** — whether the field literally reads `Engine.g2:
Ghoul2System` on `mp_engine_core::Engine` and how `Engine::new`'s zeroed-alloc
constructor accommodates it. The FROZEN `state-ownership.md` leaves that open for
all four §F subcrates (**STATE-Q2**; `Engine::new` has no `g2` field today,
`crates/mp/engine/core/src/engine.rs:35-36`). The state table's "`&mut Engine.g2`"
threading is therefore the **intended** target contingent on STATE-Q2 — see
`G2SV-Q6`. This is not self-resolvable here (it is a cross-doc STATE decision).

## Verification strategy

Governing clause: porting-rules §F18 (differential goldens), DEC-09
(oracle-differential parity). Harness `tools/ghoul2-oracle/` copies the GP2
pattern (`tools/gp2-oracle/`): `run.sh` compiles the **unmodified** oracle TUs
(`codemp/ghoul2/*.cpp` + the `tr_ghoul2.cpp` bone subset) against stub headers,
`main.cpp` dumps canonical behavior over committed fixtures, goldens under
`golden/` so `cargo test` needs no C++ toolchain; Rust parity tests
(`tests/ghoul2_parity.rs` in `mp_engine_ghoul2` / `mp_renderer`) mirror the dump
byte-for-byte.

Fixtures / goldens (the M3 gate is "G2 bone/bolt/collision goldens",
`GOAL-engine.md:72`):
- **Bone-transform goldens** — load a `.glm`/`.gla` fixture set, run
  `G2_ConstructGhoulSkeleton` + `EvalBoneCache` over a frame sequence, dump every
  bone's `mdxaBone_t` bit-exact (`G2SV-D4`). Covers the memoized `touch` path and
  the smoothing arms.
- **Bolt goldens** — `G2API_AddBolt` + `G2API_GetBoltMatrix` matrices across
  angles/position/scale, incl. the `gG2_GBM*` reconstruct-skip flags.
- **Collision goldens** — `G2API_CollisionDetect` `CollisionRecord_t` sets over
  ray fixtures (the server's real use, plan §"entanglement").
- **Arena/handle goldens** — `New`/`Delete`/`IsValid` handle values across the
  generation rollover (`G2SV-D4`).
- **Gore goldens** — `AllocGoreRecord`/`FindGoreSet` tag sequencing incl. the
  `MAX_GORE_RECORDS` eviction (`_G2_GORE` on). The gore-apply/`GoreTouch`
  vert-buffer goldens (`G2API_AddSkinGore` → `GoreVerts`/`GoreIndecies`) are
  load-bearing only if the dedicated slice drives gore application (**`G2SV-Q4`**).
- **RagDoll determinism** — `G2API_SetRagDoll`→settle over a fixed frame count,
  dumping the settled bone matrices; only load-bearing for the referee if the
  dedicated-server slice actually drives the solver (**`G2SV-Q2`**). UB inputs
  (the `Get()` shared-`null` aliasing, `_DEBUG` `_isnan` asserts) are kept OUT of
  shared fixtures or normalized in the dumper with a comment (§F19).

## Files roster

Machine-readable file plan for `port-cpp-subsystem`'s `designPath` (rule 6). All
`mode: mp`. Sharding follows porting-rules §F21 (one Raven class / logical unit
per file; free-function API groups split by concern to keep porter units bounded,
since `G2_API.cpp` is 2,783 LOC and `G2_bones.cpp` 4,907 LOC).

```yaml
files:
  - path: crates/mp/engine/ghoul2/src/ghoul2_system.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2System
    summary: The mp_engine_ghoul2 Engine sub-struct aggregate (G2SV-D5) — fields info_array, attachments (G2SV-Q3), time_bases, gbm_no_reconstruct/gbm_use_sp_method, cvars, gore GoreState, rag RagDollSolver. One type per file (CLAUDE.md); its Engine-island attachment is gated on STATE-Q2/G2SV-Q6.
  - path: crates/mp/engine/ghoul2/src/shared/cghoul2_info.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: CGhoul2Info
    summary: The per-instance CGhoul2Info class (ghoul2_shared.h:240) — a §F idiomatic reimplementation with owned Vecs (mSlist/mBltlist/mBlist), the save-serialized middle band, and runtime mBoneCache(->BoneCacheId)/mTransformedVertsArray/validity ptrs; DeepCopy zeroes runtime state. NOT the already-ported handle CGhoul2Info_v. Colocated in shared/ (mirrors the owning ghoul2_shared.h).
  - path: crates/mp/engine/ghoul2/src/info_array.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: Ghoul2InfoArray
    summary: Arena + handle (Ghoul2Handle newtype colocated) + IGhoul2InfoArray impl; New/Delete/DeleteLow/IsValid/Get, TheGhoul2InfoArray accessor, Ghoul2InfoArray_Free, id-generation arithmetic (G2SV-D1, D4).
  - path: crates/mp/engine/ghoul2/src/api_models.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API models
    summary: Init/Remove/Clean/Copy/Duplicate Ghoul2 models, PrecacheGhoul2Model, SetLodBias/Skin/Shader/Flags, SetGhoul2ModelIndexes, HaveWeGhoul2Models, Ghoul2Size, SkinlessModel (G2SV-D3 1:1).
  - path: crates/mp/engine/ghoul2/src/api_bones.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API bones
    summary: G2API_SetBoneAnim/GetBoneAnim/GetAnimRange/PauseBoneAnim/StopBoneAnim/SetBoneAngles(+Matrix,+Index)/RemoveBone/GetBoneIndex/DoesBoneExist/AnimateG2Models wrappers over G2_bones internals.
  - path: crates/mp/engine/ghoul2/src/api_bolts.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API bolts+attach
    summary: AddBolt/AddBoltSurfNum/RemoveBolt/SetBoltInfo/GetBoltMatrix (gG2_GBM* flags), AttachG2Model/DetachG2Model/AttachEnt/DetachEnt, AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments, SetNewOrigin.
  - path: crates/mp/engine/ghoul2/src/api_surfaces.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API surfaces
    summary: SetSurfaceOnOff/GetSurfaceOnOff/SetRootSurface/AddSurface/RemoveSurface/GetParentSurface/GetSurfaceIndex/GetSurfaceName/GetSurfaceRenderStatus/ListSurfaces.
  - path: crates/mp/engine/ghoul2/src/api_ragdoll.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API ragdoll+IK
    summary: SetRagDoll/ResetRagDoll/AnimateG2Models(rag), RagPCJConstraint/RagPCJGradientSpeed/RagEffectorGoal/GetRagBonePos/RagEffectorKick/RagForceSolve, SetBoneIKState/IKMove, AbsurdSmoothing.
  - path: crates/mp/engine/ghoul2/src/api_collision.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API collision+time
    summary: CollisionDetect/CollisionDetectCache, GiveMeVectorFromMatrix, SetTime/GetTime (G2TimeBases), OverrideServerWithClientData (see G2SV-Q3).
  - path: crates/mp/engine/ghoul2/src/api_gore.rs
    crate: mp_engine_ghoul2
    mode: mp
    class: G2API gore
    summary: AddSkinGore/ClearSkinGore/GetNumGoreMarks (_G2_GORE on, G2SV-D5 gore state).
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
    summary: The RagDoll + IK solver; fn-statics block -> RagDollSolver host fields (G2SV-D2). G2_RagDollSetup/RagDoll/RagDollSolve/SettlePositionNumeroTrois/RagSetState/IKSolve/DoIK/BoneSnap, SRagEffector.
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
    summary: CGoreSet + gore-record store; AllocGoreRecord/FindGoreRecord/DeleteGoreRecord, FindGoreSet/NewGoreSet/DeleteGoreSet, GoreState (G2SV-D5). _G2_GORE on.
  - path: crates/mp/renderer/src/g2/render_g2_state.rs
    crate: mp_renderer
    mode: mp
    class: RenderG2State
    summary: The mp_renderer render-side G2 state aggregate (G2SV-D5) — bone_caches SlotMap<BoneCacheId, CBoneCache> (+ the BoneCacheId key), gore_shader qhandle_t. One type per file (CLAUDE.md). CBoneCache::new's ModelId/MdxaHeaderRef args are provisional pending G2SV-Q1.
  - path: crates/mp/renderer/src/g2/bone_cache.rs
    crate: mp_renderer
    mode: mp
    class: CBoneCache
    summary: CBoneCache/CTransformBone/SBoneCalc; EvalLow/Eval/EvalRender/EvalUnsmooth/SmoothLow/GetParent/WasRendered, EvalBoneCache, RemoveBoneCache, ctor parent-seeding. _XBOX arm dropped.
  - path: crates/mp/renderer/src/g2/bone_transform.rs
    crate: mp_renderer
    mode: mp
    class: G2_TransformBone
    summary: G2_TransformBone, Multiply_3x4Matrix, G2_CreateQuaterion, G2_CreateMatrixFromQuaterion (-ffp-contract=off, G2SV-D4). Inverse_Matrix is in misc.rs (defined G2_misc.cpp:1656), not here.
  - path: crates/mp/renderer/src/g2/skeleton.rs
    crate: mp_renderer
    mode: mp
    class: G2_ConstructGhoulSkeleton
    summary: G2_ConstructGhoulSkeleton, G2_TransformGhoulBones, G2_GetBoneMatrixLow, G2_GetBoneBasepose, G2_RagGetBoneBasePoseMatrixLow, worldMatrix/worldMatrixInv scratch threading.
divergences:
  - "_G2_LISTEN_SERVER_OPT OFF in the WinDed set: CGhoul2Info::entityNum, G2API_OverrideServerWithClientData, CopyBoneCache compile out -> dropped as dead surface pending G2SV-Q3."
  - "_XBOX OFF: CTransformBone::renderMatrix/pad, the Z_Malloc mFinalBones/mSmoothBones raw arrays, EvalFull, SetRenderMatrix dropped; the vector<> arm is the live path."
  - "_SOF2 OFF: the ghoul2_shared.h SSkinGoreData/goreEnum_t variant is dead; MP uses SSkinGoreData_s (q_shared.h:3112)."
  - "Ghoul2InfoArray::Get on an invalid handle returns a shared function-static null vector it first .clear()s (non-reentrant aliasing, G2_API.cpp:427-439) -> Rust returns an empty slice; kept out of shared fixtures (F19)."
  - "G2API_DEBUG destructor leak-report + g_Ghoul2Allocations/g_G2AllocTrack debug alloc tracking dropped (debug-only, no parity surface)."
  - "boneInfo_t basepose/baseposeInv/... raw mdxaBone_t* members keep their ported ABI repr but are filled from RenderG2State-owned matrices, not shared raw pointers (B5 seam)."
  - "_DEBUG _isnan / assert paths in SmoothLow/EvalLow are debug-only; normalized out of the dumper with a comment (F19)."
  - "HackadelicOnClient (tr_ghoul2.cpp:104) is const-false server-side: its only writers are in R_AddGhoulSurfaces, #ifndef DEDICATED (:3384-3537); the render-traversal branches fold to their false arm (C10)."
  - "RSStorage/NextRS/AllocRS render-surface pool dropped: dead server-side (sole caller tr_ghoul2.cpp:2660 is #ifndef DEDICATED); the #else non-_G2_GORE second GoreVerts (G2_misc.cpp:1088) is also dead (_G2_GORE ON)."
  - "G2_PERFORMANCE_ANALYSIS ON (FINAL_BUILD undefined) but its G2Time_*/G2PerformanceCounter_*/G2PerformanceTimer_* globals are timing instrumentation with no parity surface; dropped (F20), same as the leak-checking globals."
```

## Method transcription table

Anchors for the non-obvious internal + renderer methods (the full 1:1 `G2API_*`
surface is `G2_local.h:96-224`, mapped file-by-file in the roster; §F21). Each
row is one transcription target.

| Raven symbol | oracle cite | Rust file | notes |
|---|---|---|---|
| `Ghoul2InfoArray::New/Delete/DeleteLow/IsValid/Get` | `G2_API.cpp:386,413,315,399,427` | `info_array.rs` | id/generation arithmetic bit-exact (G2SV-D4) |
| `TheGhoul2InfoArray` / `Ghoul2InfoArray_Free` | `G2_API.cpp:477-493` | `info_array.rs` | lazy singleton → owned `Engine.g2` field |
| `CBoneCache::EvalLow` | `tr_ghoul2.cpp:236` | `bone_cache.rs` | recurse-parent, memoize by `touch` |
| `CBoneCache::Eval / EvalRender / EvalUnsmooth` | `tr_ghoul2.cpp:455,520,446` | `bone_cache.rs` | public read paths; `EvalRender`→`SmoothLow` |
| `CBoneCache::SmoothLow` | `tr_ghoul2.cpp:267` | `bone_cache.rs` | render smoothing; `_isnan` asserts dropped (F19) |
| `CBoneCache::CBoneCache` (ctor) | `tr_ghoul2.cpp:390` | `bone_cache.rs` | seed `parent` from `mdxaSkel_t`; `_XBOX` arm dropped |
| `EvalBoneCache` / `RemoveBoneCache` | `tr_ghoul2.cpp:585,569` | `bone_cache.rs` | free-fn entry; delete owned by arena |
| `G2_TransformBone` | `tr_ghoul2.cpp:1541` | `bone_transform.rs` | LERP + `Multiply_3x4Matrix` chain |
| `Multiply_3x4Matrix` | `tr_ghoul2.cpp:1128` | `bone_transform.rs` | `-ffp-contract=off` (G2SV-D4) |
| `G2_CreateQuaterion` / `G2_CreateMatrixFromQuaterion` | `tr_ghoul2.cpp:1048,1097` | `bone_transform.rs` | quaternion↔matrix |
| `G2_ConstructGhoulSkeleton` | `tr_ghoul2.cpp:3567` | `skeleton.rs` | drives `G2_TransformGhoulBones` per model |
| `G2_TransformGhoulBones` | `tr_ghoul2.cpp:2075` | `skeleton.rs` | builds/refreshes `CBoneCache` |
| `G2_GetBoneMatrixLow` / `G2_GetBoneBasepose` | `tr_ghoul2.cpp:727,656` | `skeleton.rs` | server collision bone accessors |
| `G2_GenerateWorldMatrix` | `G2_misc.cpp:1678` | `misc.rs` | sets `worldMatrix`/`worldMatrixInv` scratch |
| `G2_SetupModelPointers` | `G2_misc.cpp:1839` | `misc.rs` | revalidates `mValid`/model ptrs post vid_restart |
| `G2_TraceModels` / `G2_TransformModel` | `G2_local.h:69,75` (`_G2_GORE` arm) | `misc.rs` | collision + gore-apply transform |
| `G2_RagDollSetup/RagDoll/RagDollSolve` | `G2_bones.cpp:2254,2403,3970` | `ragdoll.rs` | fn-statics → `RagDollSolver` (G2SV-D2) |
| `G2_RagDollSettlePositionNumeroTrois` | `G2_bones.cpp:3449` | `ragdoll.rs` | settle-pass `static` locals = scratch kind |
| `G2_IKSolve / G2_DoIK` | `G2_bones.cpp:4297,4453` | `ragdoll.rs` | IK arm shares the solver statics |
| `AllocGoreRecord/FindGoreRecord/DeleteGoreRecord` | `G2_misc.cpp:58,103,118` | `gore.rs` | `MAX_GORE_RECORDS` eviction |
| `FindGoreSet/NewGoreSet/DeleteGoreSet`, `CGoreSet::~CGoreSet` | `G2_misc.cpp:127,142,153,174` | `gore.rs` | `_G2_GORE` on |

## Slice hooks

- **M3 waves 13–19** (`GOAL-engine.md:71`) — "renderer, RMG, botlib, ghoul2
  complete"; gate = the bone/bolt/collision goldens above. Needs frozen first:
  the already-ported `shared/` + `gore/` layout types (done), and the `mdxa*`
  model-memory accessor boundary (`G2SV-Q1`) before `skeleton.rs`/`bone_cache.rs`
  can read `mdxaHeader_t`.
- **`SV_GameSystemCalls`** (wave 20, plan §"server is the integrator") — the
  50 server→ghoul2 edges call the `G2API_*` surface frozen by `G2SV-D3`; that
  seam must be stable before the switch arm ports.

## Open questions

- **`G2SV-Q1`** — Renderer-subset boundary. The scope names "148 fns / 9,900 LOC"
  of welded renderer code, but `tr_ghoul2.cpp` alone is 5,509 LOC; the remaining
  ~4,400 LOC is the `mdxa`/`mdxm` model+mesh **loader** (`tr_model.cpp`,
  `tr_mesh.cpp`) that owns the `mdxaHeader_t`/`mdxaSkel_t` the bone pipeline
  reads. The inputs settle only the bone pipeline; which loader functions belong
  to *this* doc vs. a `tr_model` subsystem doc — and therefore where
  `MdxaHeaderRef`/`ModelId` are defined — is not settled by the inputs and cannot
  be derived from oracle ground truth (it is a doc-partition choice). Escalate.
- **`G2SV-Q2`** — Does the dedicated-server slice actually execute the RagDoll/IK
  solver (`G2_RagDoll`/`G2_IKSolve`), or is it `cgame`-only dead surface under
  `DEDICATED`? `G2API_SetRagDoll` etc. are in the server-visible `G2_local.h` API
  (so they are transcribed per `G2SV-D2`), but the M3 gate lists only
  "bone/bolt/collision goldens" (`GOAL-engine.md:72`), not ragdoll. Whether
  ragdoll-determinism goldens are load-bearing for the referee or the solver is
  §F20 dead surface server-side is a verification-scope decision the inputs do
  not settle. **Blocks `ragdoll.rs` + `api_ragdoll.rs` outright**: the build-out
  plan's port-process discipline (`docs/plans/2026-07-08-mp-engine-build-out.md`
  §"Port-process discipline") forbids `todo!()`/speculative dead-code AND forbids
  marking the solver dead without a settled §F20 zero-caller finding — so a porter
  can neither stub these files nor drop them until this is ruled. Escalate.
- **`G2SV-Q3`** — With `_G2_LISTEN_SERVER_OPT` OFF (never `#define`d in
  `codemp/`), `G2API_OverrideServerWithClientData`/`entityNum`/`CopyBoneCache`
  and the `g2ClientAttachments[]` override path compile out. Drop them as dead
  surface (§F20) or retain the `attachments` field for a future listen-server
  build? porting-rules §F20 covers "zero callers in either tree", not
  "ifdef-disabled in this build config"; the inputs do not settle it. **Blocks the
  exact shape of `Ghoul2System.attachments`** (present as `[Option<Ghoul2Handle>;
  MAX_GENTITIES]` or absent) and the `OverrideServerWithClientData` row in
  `api_collision.rs`. Escalate.
- **`G2SV-Q4`** — Gore apply-path server load-bearingness. `G2API_AddSkinGore`
  (`G2_API.cpp:2569`) compiles server-side (uses `G2VertSpaceServer`, no DEDICATED
  guard) and is in the server-visible `G2_local.h` API, so its scratch buffers
  (`GoreVerts`/`GoreIndexCopy`/`GoreIndecies`, `G2_misc.cpp:793-798`) and the
  persistent `GoreTouch` counter are transcribed (owners per fork-2/fork-3 in the
  State table). But whether the dedicated-server game DLL actually drives gore
  application (populating `TS.gore`) — hence whether the gore-apply/vert-buffer
  goldens are load-bearing for the referee or gore-apply is `cgame`-only §F20 dead
  surface server-side — is the same verification-scope axis as `G2SV-Q2` and is
  not settled by the inputs. Escalate. (`GoreTouch++` itself does run server-side
  via the collision path `:890`, independent of this question.)
- **`G2SV-Q5`** — `CRagDollUpdateParams` virtual-dispatch shape (`G2_gore.h:94`).
  It is a C++ class with four (five under `_DEBUG`) virtual methods, one of which,
  `RagDollSettled()`, is genuinely called live from in-scope code (`params->
  RagDollSettled()`, `G2_bones.cpp:2497,2505` inside `G2_RagDoll`); the others
  (`EffectorCollision`/`RagDollBegin`/`Collision`/`DebugLine`) are effectively
  no-op overridable hooks. It appears in the frozen seam
  (`g2api_animate_g2_models_rag(..., params: &mut CRagDollUpdateParams)`) but is
  **distinct** from the already-ported plain-data `sharedRagDollUpdateParams_t`
  (`crates/mp/qshared/.../shared_ragdoll_update_params.rs`, same data members, no
  vtable). porting-rules §F17 requires the closed virtual hierarchy's Rust shape
  (enum vs. trait object vs. injected callback) be **designed once, in the doc**,
  before transcription — and a home file assigned — neither of which the inputs
  settle. A drafting agent must NOT invent it (that is the §F17 decision itself).
  Escalate. Until ruled, `api_ragdoll.rs`/`bones.rs` cannot name the `params` type
  and no roster row owns its definition.
- **`G2SV-Q6`** — `Engine`-island attachment for `Ghoul2System` (gated on
  **STATE-Q2**). `G2SV-D5` settles the crate-level ownership (`Ghoul2System` ∈
  `mp_engine_ghoul2`, bone caches ∈ `mp_renderer`) and the state table threads a
  `&mut Engine.g2`, but the FROZEN `state-ownership.md` explicitly leaves whether/
  how the four §F subcrates (ghoul2 included) attach to the `Engine` island open
  (STATE-Q2), and `mp_engine_core::Engine` has **no `g2` field** yet — `Engine::new`
  is a zeroed-alloc + in-place-`MaybeUninit`-write constructor with no slot for one
  (`crates/mp/engine/core/src/engine.rs:20-37,68-94`, with the explicit STATE-Q2
  caveat comment). Whether the field is literally `Engine.g2: Ghoul2System` (plus
  one `addr_of_mut!((*p).g2).write(...)` in `Engine::new`) or reached by some other
  channel STATE-Q2 pins is a cross-doc STATE decision this doc cannot self-resolve.
  Escalate (resolve jointly with STATE-Q2). Until then, the `G2API_*` seam's
  `&mut Ghoul2System` first argument stands, but its derivation from `&mut Engine`
  is provisional.
- **`G2SV-Q7`** — `mp_engine_ghoul2`⇄`mp_renderer` crate-edge direction (the
  two-crate cycle the frozen seam implies). `G2SV-D5` places `Ghoul2System`,
  `CGhoul2Info`, and the already-ported `CGhoul2Info_v`
  (`crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs`) in `mp_engine_ghoul2`
  and `RenderG2State`/`CBoneCache` in `mp_renderer`; the Seam then freezes
  signatures crossing **both** directions. The `mp_engine_ghoul2` entries
  `g2api_get_bolt_matrix`/`g2api_collision_detect`/`g2api_animate_g2_models_rag`
  take `r: &mut RenderG2State` (⇒ `mp_engine_ghoul2` → `mp_renderer`), while the
  `mp_renderer` entry `g2_construct_ghoul_skeleton` takes `g2: &mut Ghoul2System`
  and `ghoul2: &mut CGhoul2Info_v` (⇒ `mp_renderer` → `mp_engine_ghoul2`) — a
  `[dependencies]` cycle Cargo rejects. The underlying C-level entanglement is
  genuinely bidirectional (`docs/plans/2026-07-08-mp-engine-build-out.md:70,87,494-501`,
  §"Ghoul2 ⇄ renderer entanglement is structural": 66 ghoul2→renderer + 11
  renderer→ghoul2 edges, "welded together"), so this is a real crate-boundary
  question, not a transcription slip. `docs/workspace-architecture.md:192-194`
  states only that "ghoul2 is depended on by both `engine/*` and `cgame`" — i.e.
  ghoul2 as an upstream shared crate, which the frozen `mp_engine_ghoul2` →
  `mp_renderer` edge contradicts; it neither picks a single upstream direction for
  the welded types nor names a third shared crate to host the crossing types.
  Breaking the cycle (invert which side owns the crossing call, relocate
  `RenderG2State`/the bone-pipeline signatures, or add a third shared crate for the
  crossing types) is a crate-boundary design decision the inputs and standing docs
  do not settle, and porting-rules §F17/§B do not dictate. **Blocks the
  `[dependencies]` wiring between `mp_engine_ghoul2` and `mp_renderer`** — hence any
  cross-crate skeleton (`skeleton.rs` ↔ `bone_cache.rs`/`ghoul2_system.rs`). A
  drafting agent must NOT pick the direction. Escalate (resolve with
  `workspace-architecture.md`).
