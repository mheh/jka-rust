# Engine pre-port fork discovery (2026-07-09) — ALL RULINGS SETTLED

The engine equivalent of `jampgame-fork-discovery.md`: the design forks that
must be user-ruled BEFORE the mega-pass transcription window, so porters are
blind executors of settled decisions (see plan §"Port-process discipline" and
`docs/GOAL-engine.md`). Derived from the corrected dependency walk
(`tools/closure-prototype/out/engine/engine-port-order.{json,tsv,md}`, 2,481
fns / 87,728 LOC / 5,081 edges). All ten forks were ruled in the 2026-07-09 session; rulings recorded inline.

## Fork classes (blast-radius order)

1. **Error recovery: `Com_Error` is a `longjmp`** (`abortframe` setjmp in
   `Com_Frame`/`Com_Init`; ERR_DROP unwinds mid-frame from anywhere — the
   110-fn core knot exists *because* error paths call back into everything).
   RECOMMENDED: Rust panic + `catch_unwind` at exactly the Raven setjmp
   sites; payload carries the error level/message; `com_error_recover` (the
   existing stub) becomes the landing pad. No `Result` threading — it would
   rewrite every signature in the engine and break transcription-first.
   **RULING: panic + catch_unwind at Raven's setjmp sites; payload = level+message; com_error_recover is the landing pad (user, 2026-07-09)**

2. **Global state placement** (~680 file-scope globals: `sv` 665KB, `svs`,
   `cvar_indexes`/hash, `fs_*` pak state, `cm` clipmap, `msgHuff` 102KB,
   `com_*` cvar handles, botlib's `aasworld`/`botlibglobals`, tr_ state).
   RECOMMENDED: the jampgame fork-1 pattern — everything becomes fields on
   the owning subsystem struct under the existing `Engine` aggregate
   (`Engine { common, sv, cm, fs, net, bot, g2, ... }`), grouped by owning
   .c file; engine cvar *handles* in one `EngineCvars` sub-struct per
   subsystem. No `static mut` anywhere (const tables stay `const`).
   **RULING: Engine sub-structs, grouped by owning .c file; EngineCvars per subsystem; no static mut (user, 2026-07-09)**

3. **Function-scope statics (119: qcommon 48, ghoul2 40 — the RagDoll
   solver, `CM_LoadMap_Actual::last_checksum`, botlib `AAS_ContinueInit*`
   frame counters).** RECOMMENDED: bless the jampgame fork-5 three-kind rule
   unchanged — const tables → `const`; rotating scratch/return buffers →
   owned return values; genuine cross-frame state → host-struct fields
   (fork 2). **RULING: jampgame fork-5 three-kind rule blessed unchanged (user, 2026-07-09)**

4. **Memory allocators: Zone (`Z_Malloc` tags) + Hunk (two-sided marks,
   temp/perm) + `Hunk_AllocateTempMemory`.** Allocation ORDER and reuse are
   parity-visible wherever pointers/indices leak into state the referee
   diffs. RECOMMENDED: port allocator logic faithfully as owned arenas
   (`Vec<u8>`-backed, same mark/free-list semantics, deterministic layout);
   no Rust global allocator substitution on parity paths; idiomatization
   deferred to the safe-state migration. **RULING: faithful owned arenas (Vec<u8>-backed, same mark/free-list/tag semantics); idiomatization deferred to safe-state migration (user, 2026-07-09)**

5. **Internal dispatch tables** (`botlib_export_t`/`botlib_import_t`,
   `refexport_t` (1 live arm under DEDICATED), ICARUS `interface_export_t`
   ~40 fns, `ucmds[]`-style command tables). These never cross the module
   ABI; grep found no address-comparison of their members (unlike the
   jampgame entity handlers). RECOMMENDED: plain Rust structs of `fn` items
   populated at the same init sites (1:1 shape, zero indirection cost, keeps
   the 261 ref-edges meaningful); command tables as `&[(&str, fn(...))]`
   consts. Fn-ID enums NOT needed absent address compares. **RULING: plain Rust structs of fn items at the same init sites; command tables as const (&str, fn) slices; no fn-ID enums (user, 2026-07-09)**

6. **VM subsystem stance** (`vm.cpp`/`vm_interpreted.cpp`/`vm_x86.cpp`; plan
   §5.4). RECOMMENDED: all three port; interpreter is portable logic;
   `vm_x86` ports as data-faithful emitter (executes only on x86 hosts, same
   as C); runtime path for our module stays native-dylib; interface-crate
   arg slots are `intptr_t`-width. **RULING: all three port; vm_x86 data-faithful emitter (golden-testable everywhere, executes only on x86 like Raven); native-dylib runtime path; intptr_t arg slots (user, 2026-07-09)**

7. **C++-track design docs (§F) — which subsystems get one, before the
   window.** GP2 is already done (the pilot). RECOMMENDED docs: **icarus**
   (253 fns: CTaskManager/CSequencer/CBlockStream tree), **RMG** (113:
   CRMManager/instance hierarchy — closed → enums), **ghoul2 + renderer
   class internals** (CGhoul2Info_v arena + CBoneCache), **CNavigator**
   (server/NPCNav, 81 fns — engine-side twin of the game's nav API),
   **CROFFSystem** (RoffSystem.cpp). Everything else is C-track packets.
   **RULING: the 5-doc list blessed — icarus, RMG, ghoul2+CBoneCache, CNavigator, CROFFSystem; GP2 exemplar; REVIEWED before the window (user, 2026-07-09)**

8. **The platform trait** (the 26 `Sys_*`/`NET_*` externals + the main loop
   from `null/win_main.cpp`, excluded from the port set). RECOMMENDED: one
   `PlatformHost` trait in the interface crate (clock, console I/O, UDP,
   file listing, dylib loading — the module loader already exists);
   deterministic test impl for the referee (fixed clock, scripted packets),
   std impl for the real binary. Unix path semantics (the referee platform),
   not Win32. **RULING: one PlatformHost trait in the interface crate; deterministic test impl for the referee, std impl for the binary; unix semantics (user, 2026-07-09)**

9. **Filesystem semantics** (`files_common.cpp`/`files_pc.cpp`): search-path
   order, pure-server pak checksum lists, `fs_homepath`/`fs_basepath`,
   macOS case-insensitivity vs Raven's case assumptions. Parity-visible
   through configstrings (`sv_paks`) and download lists. RECOMMENDED:
   faithful port of ordering/checksum logic over the platform trait's
   directory enumeration, with enumeration order pinned (sorted) and a
   golden fixture on the retail-assets pak list. **RULING: faithful ordering/checksum logic over the platform trait; enumeration order pinned (sorted); retail pak-list golden fixture (user, 2026-07-09)**

10. **Console/print routing** (`Com_Printf` → console/logfile — feeds the
    referee's syscall-stream digest via `G_PRINT` echo and `Sys_Print`).
    RECOMMENDED: route through the platform trait; byte-identical
    formatting (`va`/`Com_sprintf` already ported in qshared); no timestamps
    unless Raven prints them. **RULING: all output via PlatformHost; byte-identical formatting; no non-Raven decoration (user, 2026-07-09)**

## The type rosetta (agent packet reference)

`tools/closure-prototype/out/engine/type-rosetta.tsv` — generated by
`tools/closure-prototype/typemap.py` from the house-style `Source:` cites:
**2,702 ported items, 2,002 distinct Raven type names** → Rust name, kind,
crate, and file path. Every porting packet includes (or references) this
index. Agent rules it enforces:

- **All ABI/layout types already exist** (type port complete). A porter
  NEVER declares a struct/enum/typedef — it imports from the listed path.
- **A name missing from the rosetta is an escalation, not a stub** — the
  no-stub discipline (plan §"Port-process discipline") applies to types
  exactly as to functions. The finisher triages misses (usually a naming
  variant; the tool is regenerated after any legitimate addition).
- Regenerate with `.venv/bin/python typemap.py` after any type lands or
  moves; the TSV is generated, never hand-edited.

## Not forks (already settled elsewhere)

- Transcription-first / no safety refactoring during the port (plan §1).
- Vendored zlib/png via Rust crates; platform files excluded (plan §1).
- FINAL_BUILD undefined; WinDed vcproj Release macro set (plan appendix).
- One engine in the repo; interface crate first (referee plan).

## §F doc-session rulings (user, 2026-07-09 — second session)

The five NEEDS_SESSION design docs' contested points, ruled after evidence
queries against `engine-port-order.json` and the oracle:

11. **Host seam (ICARUS-Q1 = NAV-Q1 = ROFF-Q1 = STATE-Q2 service half):**
    ONE `EngineHost` services trait in the Stage-0 interface crate (trace, FS,
    print/error, VM_Call, shared memory). §F methods take
    `(&mut SubsystemState, &mut impl EngineHost)`; `Engine` implements it via
    a split-borrow view struct. Referee injects a deterministic impl.
    **RULING: EngineHost trait + view-struct impl (user, 2026-07-09)**
12. **Attachment (STATE-Q2 placement half):** the five §F states are plain
    Default-initialized direct fields on `Engine` (`icarus`, `nav`, `g2`,
    `roff`, `rmg`) — no Option/Box, no nesting. Lazy-init timing modeled with
    Raven's own initialized flags.
    **RULING: direct Engine fields (user, 2026-07-09)**
13. **ICARUS-Q2:** owned objects replace the TAG_ICARUS2/3/4 class allocs;
    the fork-4 arena covers only ICARUS_Malloc/Free raw blobs (TAG_ICARUS5).
    **RULING: residual raw blobs only (user, 2026-07-09)**
14. **ICARUS-Q3:** committed .IBI goldens are hand-authored scripts compiled
    once by a `tools/ibi-gen` harness built from the oracle's out-of-set
    Interpreter/Tokenizer; no retail blobs in the public repo (retail corpus
    may run locally, uncommitted).
    **RULING: hand-authored, tool-compiled (user, 2026-07-09)**
15. **ICARUS-Q4:** the unchecked `gSequencers[ent_num]` UB paths (5 of 11
    inbound fns) guard-and-return per §19, ≤2-line note per site.
    **RULING: guard-and-return (user, 2026-07-09)**
16. **RMG-Q1:** the qcommon terrain twins (CCMLandScape, CRandomTerrain,
    CTerrainMap, CPathInfo, CArea, CCMPatch, CCMHeightDetails) fold into the
    rmg-terrain doc; the cm C-track packets exclude those classes.
    **RULING: fold them in (user, 2026-07-09)**
17. **Dead surface (§20 drops, evidence-backed):** `Svcmd_ICARUS_f`,
    `RmManager.mCurObjective`, `noiseTable`/`noisePerm`, and the RM_Terrain
    client-model chain (`RM_CreateRandomModels`, `CTerrainMap::Upload`,
    `CTerrainMap::SaveImageToDisk` — zero engine callers under DEDICATED,
    graph-confirmed) are recorded with §20 zero-caller notes, not ported.
    **RULING: §20-drop all four (user, 2026-07-09)**
18. **Doc defects:** ghoul2's `g2api_get_bolt_matrix` seam signature becomes
    write-through + qboolean (Raven ALWAYS writes the out-matrix); npcnav
    transcribes Raven's own priority-queue implementation faithfully instead
    of std::BinaryHeap (equal-cost tie order is parity-visible).
    **RULING: fix both as stated (user, 2026-07-09)**

Evidence resolutions (mechanical, no ruling needed — recorded for the docs):
- **G2SV-Q2:** ragdoll/IK is server-live — 36 rag/IK functions reachable from
  `SV_GameSystemCalls`. In scope, statics per fork-3.
- **G2SV-Q1:** the renderer-subset boundary is exactly the 8 WinDed vcproj
  renderer files; fn-extent LOC: tr_ghoul2 3,505, tr_shader 3,139, tr_model
  1,547, tr_image 845, tr_init 538, matcomp 240, tr_main 47, tr_mesh 39.
- **ROFF-Q3:** the syscall switch has FIVE roff arms (G_ROFF_CLEAN,
  G_ROFF_UPDATE_ENTITIES, G_ROFF_CACHE, G_ROFF_PLAY, G_ROFF_PURGE_ENT).
- **NAV-Q3:** the G_NAV_SETCHECKEDNODE→FLAGALLNODES→GETPATHSCALCULATED
  fall-through (sv_game.cpp:928-933, a real Raven bug) is owned by the
  wave-20 `SV_GameSystemCalls` transcription, not the nav subsystem.
- **RMG-Q2:** the RM_Terrain client-model chain is dead under DEDICATED
  (see ruling 17).

## §F doc-session rulings, round 2 (user, 2026-07-09 — third session)

19. **Ruling-11 amendment — EngineHost gains gentity access:**
    `gentity(ent_num) -> *mut sharedEntity_t` (the `SV_GentityNum` dual over
    the G_LOCATE_GAME_DATA base; raw pointer, transcription-first). Needed by
    the icarus Q3_TaskID*/InitEnt/FreeEnt/AssociateEnt/RunScript seam, which
    reads/writes game-entity fields.
    **RULING: add the service (user, 2026-07-09)**
20. **Ruling-13 amendment — the ICARUS arena is dropped entirely** (its
    remaining tag-5 users went owned `Vec<u8>` in the roster, leaving it
    empty); `ICARUS_Malloc`/`ICARUS_Free` are not ported. Plus two explicit
    doc statements: `Icarus` gets a hand-written `impl Default`
    (`ent_filter: -1` — its only writer was §20-dropped and the value gates
    referee-digested prints; boxed MAX_GENTITIES arrays via `from_fn`), and
    `InterfaceExport` is constructed with the real `Q3_*`/`I_*` fn items at
    Default time with Raven's own `initialized` flag preserving
    Interface_Init timing.
    **RULING: bless all three (user, 2026-07-09)**
21. **RMG shapes:** `rmAutomapSymbol_t` relocates to `mp_qshared`; NO
    RandomTerrainHandle newtype (methods use `CmLandScape`'s
    `Option<RandomTerrain>` directly, the seam converts Raven's handle int);
    the ENGINE's own q_math LCG instance is a qshared `QRand`-type field on
    `Engine.common` exposed via EngineHost `flrand`/`irand`; `CRMArea*` →
    `AreaId` + arena on `CRMAreaManager` per §B5; the
    `CCMPatch::owner`/`CRandomTerrain::mLandScape`/`CRMMission::mLandScape`
    back-pointers are dropped and `&CmLandScape` threaded through the
    affected methods.
    **RULING: bless all five (user, 2026-07-09)**
22. **G2/NAV/ROFF:** gore APPLY path (`G2API_AddSkinGore`,
    `DestroyGoreTexCoordinates`, `ResetGoreTag`, `G2_GetGoreRecord`) is
    graph-dead server-side → §20 notes; gore RECORD infra (`AllocGoreRecord`,
    `Find/New/DeleteGoreSet`, `ClearSkinGore`, `G2_GorePolys`) is live and
    ports. `CRagDollUpdateParams` closed hierarchy → enum per §17. "SlotMap"
    = the hand-rolled generational arena matching Ghoul2InfoArray's bit-exact
    handle scheme; `CGhoul2Info_v` forwarding methods colocate in
    `cghoul2_info_v.rs`. `Q3_INFINITE`/`WORLD_SIZE`/`STEPSIZE`/
    `WAYPOINT_NONE` + the vec3 primitives get their engine-reachable home in
    `mp_qshared` (moved or re-exported from mp_game's copies). ROFF's
    InitROFF-failure fallthrough (`map::find(0)` end-deref, oracle UB) →
    guard-and-return per §19 with a 2-line note.
    **RULING: bless all five (user, 2026-07-09)**

## §F doc-session rulings, round 3 (user, 2026-07-09 — fourth session)

23. **Ruling-19 CORRECTION (premise was wrong, reviewer-proven from
    sv_game.cpp:740-807):** the entity-field G_ICARUS_* arms pass
    `(sharedEntity_t *)VMA(1)` through `ConvertedEntity()` — a pointer, not
    an entnum. The inbound icarus seam CARRIES THE POINTER
    (`*mut sharedEntity_t`) exactly as the trap does; ConvertedEntity's
    VM-shuffle is a documented no-op in the native-dylib model. The 3
    presence-check arms keep their int entID. The `gentity()` EngineHost
    service remains for genuinely index-based access.
    **RULING: carry the pointer (user, 2026-07-09)**
24. **ICARUS shapes:** the ~194 `m_ie->I_*` dispatch sites become free fns
    taking `(&mut <icarus state>, &mut dyn EngineHost)`; `InterfaceExport`
    slots are fn pointers taking `&mut dyn EngineHost` (dyn at the table
    boundary). The Stage-0 crate is PINNED: `crates/mp/host-interface`,
    package `mp_host_interface` — docs cite real paths from now on.
    **RULING: free fns + &mut dyn; crate pinned (user, 2026-07-09)**
25. **RMG generation is dead under DEDICATED in Raven itself**
    (`CreateRandomTerrain`'s only call site is in the `#else` of
    `#ifdef DEDICATED`, cm_terrain.cpp:167-186; LoadMission early-outs, the
    mission never spawns). Accept Raven: §20-drop the generation path
    (CreateRandomTerrain, CRandomTerrain::Generate, CRMMission::Spawn/
    PreSpawn/Smooth/PlaceBridges, heightmap goldens); the live surface is
    the reachable syscall arms + LoadMission's early-out behavior, refereed
    as such. Making generation live is a post-parity feature branch.
    **RULING: accept Raven, §20-drop generation (user, 2026-07-09)**
26. **G2/NAV corrections:** `DestroyGoreTexCoordinates` and
    `DeleteGoreRecord` move to the LIVE gore bucket (called from
    `~CGoreSet`, reachable via `DeleteGoreSet` — implicit destructor calls
    are a documented engineorder graph blind spot). npcnav equal-cost heap
    tie-order is pinned to the oracle-harness toolchain (Homebrew
    g++-16/libstdc++), the same reference every golden uses; retail-MSVC
    tie-order divergence accepted exactly as for FP parity.
    **RULING: bless both (user, 2026-07-09)**

## §F doc-session rulings, round 4 (user, 2026-07-09 — fifth session)

27. **ICARUS ownership graph (ICARUS-Q10/Q11):** faithful Vec arenas + id
    newtypes. `IcarusInstance` owns `Vec<Sequence>` / `Vec<Sequencer>`;
    `SequenceId(i32)` / `SequencerId(i32)` carry Raven's monotonic
    never-reused `m_GUID`; `GetSequence(id)` stays a linear scan (faithful
    O(n), insertion-ordered iteration). A sequencer's non-owning subset is
    `Vec<SequenceId>`; its `map<CTaskGroup*,CSequence*>` becomes
    `BTreeMap<TaskGroupId, SequenceId>`. `TaskManager`: ONE owning
    `Vec<TaskGroup>` arena + `TaskGroupId`; the string/int maps become
    `BTreeMap` side-indexes of ids (three parallel owners collapse to
    owner + side-indexes). Matches rulings 21/22 precedent.
    **RULING: faithful arenas + ids (user, 2026-07-09)**
28. **RMG live collision surface + landscape accessor (RMG-Q8/Q9):** the
    per-frame terrain-collision surface IS live under DEDICATED — amend
    RMG-D1's live enumeration to four items (+ `CmLandScape::PatchCollide` /
    `WaterCollide` / `GetBounds` / water accessors). They port as
    `CmLandScape` methods with faithful signatures threaded through
    `&`/`&mut CollisionWorld`, frozen in the rmg-terrain doc now; they land
    with the early clipmap-trace waves their cm_trace/cm_test callers
    occupy (the doc keeps ownership). `RmManager` stores
    `land: Option<TerrainHandle>` (Raven inits `mLandScape` null); the
    accessor returns `Option<TerrainHandle>`, callers resolve through
    `CollisionWorld`; `GetHeightMap`/`GetFlattenMap` return `&[u8]`.
    Naming (RMG-Q7): `CRMArea` → `RmArea`, qcommon `CArea` → `CmArea`
    (both §20-dropped shape-map entries).
    **RULING: collision live; Option handle; RmArea/CmArea (user, 2026-07-09)**
29. **Ghoul2 shape holes (delete / ragdoll pointers / gore buffers):**
    (a) `delete`/`delete_low` move UP to `Ghoul2System` methods — free the
    slot's bone caches from `bone_caches`, then drop the info slot (Raven's
    method placement is an artifact of its globals). (b) `RagDollSolver`
    stores bone INDICES into the model's blist and resolves basepose
    matrices per call through EngineHost — no stored raw pointers outside
    the ABI seam (write-through GetBoltMatrix pattern, ruling 21).
    (c) `GoreState` owns each per-LOD gore buffer as `Vec<f32>`; the
    ABI-frozen `GoreTextureCoordinates.tex` pointers point into those Vecs;
    teardown order mirrors `Z_Free`.
    **RULING: system-level delete; ids not ptrs; owned Vec buffers (user, 2026-07-09)**
30. **NAV entity seam:** the five ent-taking nav arms (G_NAV_GETNEARESTNODE,
    CHECKFAILEDNODES, ADDFAILEDNODE, NODEFAILED, GETBESTPATHBETWEENENTS)
    carry `*mut sharedEntity_t` exactly as the trap marshals
    `(sharedEntity_t *)VMA(1)` — ruling 23's precedent applies verbatim;
    methods deref the pointer like Raven. The `gentity()` EngineHost
    service stays for genuinely index-based access. The npcnav doc's
    `ent: EntityId` model and its "reached through SV_GentityNum" note are
    both corrected.
    **RULING: carry the pointer, per ruling 23 (user, 2026-07-09)**
31. **Stage-0 sequencing (EngineHost signature gap):** the Stage-0 crate
    `crates/mp/host-interface` (pkg `mp_host_interface`) is BUILT BEFORE
    the doc relaunch — PlatformHost + EngineHost traits land as real
    compiled code first, and the four docs cite the crate's actual
    signatures instead of a paper spec. The doc loop resumes only after
    `cargo build` is green on the crate.
    **RULING: build Stage 0 first (user, 2026-07-09)**
32. **NAV first-slice goldens (NAV-Q7):** no test-only constructor — the
    3a golden harness implements EngineHost as a fixture-backed mock (FS
    reads serve committed `.nav` fixture bytes; print/error captured;
    flrand deterministic), so `Load` ports in the first slice with its real
    frozen signature and populates the graph through the front door. The
    mock is the reusable goldens vehicle for every host-taking subsystem
    (icarus, RMG, G2).
    **RULING: fixture-backed mock EngineHost (user, 2026-07-09)**

### Round-4 mechanical resolutions (evidence, not forks)

- **TODO-marker conflict (rmg doc):** the engine-wide no-TODO/no-FIXME rule
  (GOAL-engine.md, user-directed) WINS unconditionally; the rmg doc's
  "leave a `//TODO: Port CArea` marker" fallback is struck — §20-dropped
  items get a zero-caller §20 note, never a marker.
- **`rmAutomapSymbol_t` destination (RMG-D2d):** already ported at
  `crates/mp/engine/client/src/client/rm_automap_symbol_t.rs`; relocates to
  `crates/mp/qshared/src/common/mp/rmg/rm_automap_symbol_t.rs` (new `rmg/`
  folder mirroring `oracle/codemp/RMG/RM_Manager.h` ownership), client
  import updated in the same commit.
- **NAV-D6 migration mechanics:** the four consts and four vec3 fns MOVE
  (never duplicate) to mp_qshared; vec3 fns → a new
  `crates/mp/qshared/src/shared/q_math.rs` (sibling of `q_math_rand.rs`,
  mirroring `oracle/codemp/game/q_math.c`); each const lands in the folder
  mirroring its owning Raven header per existing convention. The existing
  `mp_game` copies are deleted and re-imported in the SAME commit — no
  re-export shims. This migration is in-scope for the npcnav doc's first
  slice.

## Stage-0 crate escalations (user, 2026-07-09 — "no deferrals")

33. **No deferrals in the Stage-0 seam.** (a) The UDP surface
    (`Sys_GetPacket`/`Sys_SendPacket` family) lands in `PlatformHost` NOW
    with faithful Raven signatures; the wire types it needs
    (`netadr_t`/`netsrc_t`/`msg_t`) relocate below the crate into
    `mp_qshared` (`common/mp/qcommon/`, one type per file — same treatment
    as `rmAutomapSymbol_t`), imports updated in the same change. (b)
    `vm_call` gains the VM selector mirroring Raven's `VM_Call(vm, …)`
    first parameter (gvm/cgvm) — ROFF's `VM_Call(cgvm, …)` call sites
    transcribe 1:1 even though cgvm is NULL under DEDICATED. The agent's
    other two provisional choices stand as faithful defaults: `trace` keeps
    the out-param shape; `fs_read_file` returns exact file bytes (Raven's
    trailing NUL is an FS-impl detail, noted at the site).
    **RULING: no deferrals (user, 2026-07-09)**

34. **No "port later" (user, 2026-07-09).** The pre-campaign scaffolding
    markers in the engine-side crates (48 as of this census: `TODO: Port` +
    `todo!()` + the LIFE-Q8 boot-success no-ops + DEV-GLUE provisionals in
    `crates/mp/engine`, `crates/mp/app`, `crates/native`) are ENUMERATED
    IN-SCOPE WORK for this campaign — not items that "burn down eventually."
    Every marker subject maps to a wave in engine-port-order (function
    bodies), a §F doc (C++ classes), or a design ruling to be taken during
    the campaign (`ZeroValid` for Engine, LOAD-Q1 macOS module suffix,
    SV_InitGameProgs ctx injection). The campaign is NOT COMPLETE while any
    of them exists; nothing is re-deferred to a later campaign. Sole flagged
    exception pending a user call: the wasm-host wiring marker (LOAD-D10)
    belongs to the DEC-settled WASM-transport track, outside the WinDed
    parity scope.
    **RULING: no port later — the marker inventory is the work list (user, 2026-07-09)**

35. **WASM dropped entirely (user, 2026-07-09; supersedes ruling 34's
    wasm-host exception and REVERSES DEC-05 item 5).** WebAssembly is not a
    goal of this project in any direction — not as a module transport, not
    as an engine target, not as a scripting sandbox. Remove all plumbing:
    the `Wasm` ModuleTransport variant, `WasmPtr<T>` design text,
    `WasmInPlaceReset`, the `mp_engine_wasm_host` crate, and any wasm
    feature gates. Transports become `NativeDll | Static`. Historical
    audits/dossiers keep their point-in-time references; living docs are
    scrubbed. DEC-05 gets a dated reversal amendment, not a rewrite.
    **RULING: drop wasm, remove all of it (user, 2026-07-09)**

## §F doc-session rulings, round 5 (user, 2026-07-09 — sixth session)

36. **EngineHost extension (no-deferrals applied to verified call sites):**
    the trait gains `cvar_integer(&mut self, name: &str) -> i32` (per-call
    lookup; Raven's cached `cvar_t->integer` reads collapse to it — serves
    com_developer/cg_g2MarksAllModels/d_altRoutes/d_patched),
    `sv_time(&mut self) -> i32` (`svs.time` — nav recheck timers),
    `fs_write_file(&mut self, qpath: &str, data: &[u8]) -> bool`
    (FS_Write semantics — CNavigator::Save, live G_NAV_SAVE arm), and a
    loader-model-memory accessor (raw seam pointer to the parsed model
    block per G2SV-D5's usage sites; exact shape derived from the cites,
    no re-parsing). `g2api_set_ragdoll` gains the host param its
    unconditional flrand calls require. MockHost grows matching fixtures.
    **RULING: extend EngineHost with all of it (user, 2026-07-09)**
37. **ConvertedEntity write-drop (corrects ruling 23's rationale):**
    `ConvertedEntity` copies `ent->s`/`r`/`taskID` BY VALUE into a
    file-static and returns the copy (sv_game.cpp:420-451) — retail C
    DROPS writes to it (ICARUS_InitEnt's `memset(taskID,-1)` included).
    Port ConvertedEntity itself faithfully (it is in the 2,481-fn list)
    and route the 5 entity-field ICARUS arms through it exactly as Raven
    does; the pointer-carrying seam stands, the "no-op shuffle" rationale
    is struck. Zero divergence vs retail at M4/M5.
    **RULING: faithful copy semantics, port ConvertedEntity (user, 2026-07-09)**
38. **RMG seam repair (E0502-proven):** the terrain-collision entry points
    become `&mut self` methods ON `CollisionWorld`
    (`cm.terrain_patch_collide(...)` etc.) resolving `self.land_scape`
    internally — no double borrow; mutation (checkcount writes into
    landscape-owned brushes) legal. `spawn_mission` is DROPPED from
    Seam-A: the syscall arm ports with the provably-dead if-body collapsed
    per §C10 (`load_mission` always false under DEDICATED) + §20 note —
    no stub.
    **RULING: CollisionWorld methods + collapse dead arm (user, 2026-07-09)**
39. **Evidence-driven corrections bundle:** (a) BlockStream's
    writer/duplicator half (Create/Init/WriteBlock/WriteMember/Duplicate)
    is zero-caller in the WinDed link set → §20-drop (icarus's "3 drops
    total" corrected). (b) The three ghoul2 attach fns
    (AttachInstanceToEntNum/ClearAttachedInstance/CleanEntAttachments) are
    compiled NO-OPS reached by live syscall arms → keep as callable
    empty-body fns per §C10, NOT §20. (c) Engine-side nav RNG routes
    through `host.irand` (ruling 21's engine-owned LCG), never the qshared
    free fn. (d) Mechanical: id newtypes declared beside their owning
    arena; CCMPatch/CCMHeightDetails get their own files per §21;
    TerrainHandle gets its roster row; NAV-D6 is the full cross-crate edit
    at its verified file counts; moved vec3 fns keep Raven's
    underscore-prefixed names (`_DotProduct` etc.).
    **RULING: bless all four (user, 2026-07-09)**

## §F doc-session rulings, round 6 (user, 2026-07-09 — seventh session)

40. **ICARUS corrections:** `CBlockStream::Init` is LIVE (corrects ruling
    39a's drop-list — the live reader `Open()` calls `Init()` at
    BlockStream.cpp:670; the pass-5 "zero callers" claim was wrong; the
    other five writer/duplicator drops stand, independently verified).
    NAMING RULE, project-wide for §F internal types: Raven's bare
    hungarian `C` prefix DROPS (CSequence→Sequence, CTaskManager→
    TaskManager, CBlock→Block, CNavigator→Navigator, …); subsystem
    acronym prefixes stay in Pascal form (CCMLandScape→CmLandScape,
    CRMArea→RmArea — the ruled precedents); ABI-frozen types keep exact
    Raven names as always.
    **RULING: Init lives; drop the bare C prefix (user, 2026-07-09)**
41. **RMG-Q10 + wave-order principle:** `CM_GetShaderInfo` ports as a
    `CollisionWorld` method (`&mut self` — it registers shader text on
    miss), signature transcribed faithfully from its oracle decl,
    returning `Option<&CCMShader>`; it lands with the cm C-track waves and
    rmg-terrain.md cites it as a settled extern. GENERAL PRINCIPLE
    (recorded once, applies campaign-wide): a §F/C++ struct DEFINITION
    (all fields, cites, no method bodies) lands at the earliest wave any
    of its methods or consumers occupies; bodies follow the tool order.
    Types precede function waves — not a stub, the same relationship the
    already-ported C-track types have.
    **RULING: pin accessor now + struct-defs-early (user, 2026-07-09)**
42. **NAV-Q14 (.nav golden fixtures):** build `tools/npcnav-oracle` —
    compile the unmodified navigator.cpp TU standalone (§18), feed
    hand-authored waypoint layouts through the real
    AddRawPoint/HardConnect/Save path, commit the emitted `.nav` bytes
    plus the query/rank goldens the same run produces. The C oracle
    generates its own ground truth; Rust Load/queries reproduce it
    byte-for-byte. No retail blobs (same logic as the IBI ruling).
    **RULING: tools/npcnav-oracle, Save-generated (user, 2026-07-09)**
43. **EngineHostView pinned (the split-borrow view, ruling 11's missing
    concrete shape):** `pub struct EngineHostView<'a>` lives in
    `mp_engine_core` (the one crate seeing all `Engine` fields), holding
    `&mut` borrows of the Common/Server/CollisionWorld/loader fields
    EngineHost needs; `Engine` gains per-subsystem split constructors
    (`fn nav_call(&mut self) -> (EngineHostView<'_>, &mut Navigator)`
    pattern) that split-borrow disjoint fields — plain field-level
    reborrowing, no unsafe. The trait impl itself is wave-20 work,
    landing with the SV_GameSystemCalls arms that need it; docs cite the
    pinned shape. Recorded in state-ownership.md alongside STATE-Q2's
    ruling-12 closure.
    **RULING: pin the shape now, impl at wave 20 (user, 2026-07-09)**

## §F doc-session rulings, round 7 (user, 2026-07-09 — eighth session)

44. **`.nav` binary long width — 4 bytes, pinned.** Raven's on-disk truth
    is Win32: every `long`/`unsigned long` in the `.nav` format
    (header/ids/GetLong reads, navigator.cpp:388,428,557-564,614,676)
    reads/writes as exactly 4 bytes (`i32`/`u32`) in Rust.
    `tools/npcnav-oracle` builds the unmodified TU with a 4-byte-long shim
    via compile flags/stub-header typedef (flags are not source edits —
    LittleShort= precedent), so its fixtures are retail-shaped; goldens,
    retail pk3 `.nav` files, and the OpenJK referee agree. GENERAL RULE:
    Win32 `long` inside any BINARY FILE FORMAT = 4 bytes; `c_long` in
    compiled-ABI structs is the separate, correct case.
    **RULING: pin 4 bytes; shim the harness TU (user, 2026-07-09)**
45. **Heap-algorithm ground truth (completes ruling 26):** the npcnav doc
    states libstdc++'s push_heap/pop_heap behaviorally (two-phase Floyd:
    pop swaps root/last, __adjust_heap percolates the hole down with
    libstdc++'s child-comparison order, then sifts up from the leaf) with
    a reference-only cite to the Homebrew g++-16 stl_heap.h path — no GPL
    text enters the repo. The committed rank-order goldens are the
    enforcing gate for any sift-order mistake.
    **RULING: behavioral spec + goldens gate (user, 2026-07-09)**
46. **CmLandScape brush arena:** one shared Vec-backed arena owned by
    `CmLandScape` mirroring Raven's single
    `Z_Malloc(size × GetBlockCount())` buffer (cm_terrain.cpp:215);
    each `CmPatch` stores offset/length RANGES into it (no raw pointers);
    `UpdatePatches`' carving transcribes as range arithmetic. Faithful
    single-allocation topology per ruling 4.
    **RULING: one shared arena + range indices (user, 2026-07-09)**
47. **RMG closeout bundle:** (1) golden #4 exercises the REAL `.terrain`
    parse with a hand-authored fixture; the standalone oracle TU stubs the
    `CM_GetShaderInfo` extern (closes RMG-Q12). (2) `load_mission` KEEPS
    the faithful full signature (cm/is_server dead-on-live-path params
    included) — ruling 38 collapsed a dead call; this is a live call, and
    the wave-20 arm transcribes 1:1. (3) Mechanical blessings: RMG-D5/D6
    append numbering stands (no renumber); automap symbols hardcode
    0/None with no backing array; `GetAutomapSymbol`'s unchecked C index
    becomes `.get()` → `Option` per §19.
    **RULING: real parse fixture; keep full signature; bless (user, 2026-07-09)**

48. **Nav seam accuracy (the last open item):** `clear_failed_edge` DROPS
    its unused host param — Raven's ClearFailedEdge (navigator.cpp:
    1835-1865) uses no engine service; the signature becomes host-free,
    matching its body and its sibling `clear_all_failed_edges`.
    **RULING: drop the host param (user, 2026-07-09)**

## Undocumented-C++ referee rulings, round 8 (user, 2026-07-09 — ninth session)

49. **CDraw32 §20-dropped (22 methods, cm_draw.cpp):** provably dead under
    DEDICATED — sole caller `CTerrainMap` is header-only in WinDed.vcproj
    and already §20-dropped (rmg-terrain.md, ruling 17/RMG-D4c); the only
    instantiation site is `#ifndef DEDICATED` + `!com_dedicated->integer`
    double-guarded (RM_Mission.cpp:1501-1504). One-line addendum lands in
    FROZEN rmg-terrain.md's divergences (this ruling IS the amendment
    sign-off); the manifest reclassifies the 22 methods §20.
    **RULING: §20-drop (user, 2026-07-09)**
50. **StringEd (CStringEdPackage + SE_Entry_s — LIVE, game-module trap at
    sv_game.cpp:699): new §F doc, seeded with the faithful set** —
    (1) the package is an `Engine.common` sub-struct field, threaded, no
    singleton; (2) the store owns Strings, lookups borrow `&str`, the
    syscall arm copies bytes out exactly as Raven's trap does;
    (3) `Filename_*` scratch statics follow ruling 3's three-kind rule;
    (4) `BTreeMap` — Raven's sorted `std::map` iteration order kept.
    docs/subsystems/stringed.md authored via the established workflow.
    **RULING: faithful set (user, 2026-07-09)**
51. **tr_model doc queued NOW (no-deferrals):** the tr_model subsystem doc
    ghoul2-server.md anticipates (loader TUs + the
    `CachedEndianedModelBinary_s` cache — pModelDiskImage ownership and
    cache home settle inside it) enters the §F pipeline immediately:
    fork-discovery → rulings → author-design-doc → FROZEN. It gates the
    M3 renderer waves.
    **RULING: queue it now (user, 2026-07-09)**

## tr_model doc rulings, round 9 (user, 2026-07-09 — tenth session)

52. **Model-memory contract:** each `CachedEndianedModelBinary` owns its
    disk image as `Box<[u8]>` — heap-pinned/address-stable (the frozen
    ghoul2 seam derefs raw pointers into it across frames), in-place
    mutable for the endian swap, drop = Z_Free. Eviction stays faithful
    (level-keyed `iLastLevelUsedOn` + pak checksum + r_modelpoolmegs) —
    safe because ghoul2 re-resolves via `R_GetModelByHandle` each use and
    never caches `model_t*` across level change; the doc records that
    invariant as the contract.
    **RULING: Box<[u8]> + faithful level eviction (user, 2026-07-09)**
53. **Model-state homes:** the `CachedModels` map and the model pool
    become fields of a renderer-models state struct in `mp_renderer`
    (owner of `model_s`/`trGlobals_t`), attached to `Engine`, reached by
    ghoul2 ONLY through the EngineHost impl (`model_mdxm`/`model_mdxa`
    resolve through it; `mp_engine_ghoul2` never edges to `mp_renderer`).
    The Hunk `tr.models[1024]` pool → `Vec<Box<ModelData>>` with
    `qhandle_t` = index and a map side-index replacing the intrusive hash
    chains; `MOD_BAD` failed-entry retention kept faithful.
    **RULING: mp_renderer Engine field + Vec pool (user, 2026-07-09)**
54. **tr_model dead surface:** header `LL()` swaps stay live (identity on
    LE, golden-exercised). §20/§C10 with cites: the `#ifndef _M_IX86`
    big-endian block, the client poke replay (`#ifndef DEDICATED`; server
    variant commented out), and the tr_shader/tr_image/draw surface
    (~73 fns) — classified per-function in the doc. The
    `ShaderRegisterData` record vector STAYS on the cache struct (the
    server-side StoreShaderRequest recording is live); only the poke
    replay is client-dead.
    **RULING: §20 the dead, keep the live records (user, 2026-07-09)**

55. **EngineHost extension #2 (StringEd's verified call sites; ruling-36
    pattern):** four methods — `cvar_register(name, default, flags)`
    (Cvar_Get's registration side; establishes defaults exactly once),
    `cvar_string(&mut self, name) -> String` (`->string` reads),
    `cvar_take_modified(&mut self, name) -> bool` (read-and-clear
    `->modified`, Raven's update-check idiom, stringed_ingame.cpp:
    1169-1171), `fs_list_files(&mut self, dir, ext, want_subs) ->
    Vec<String>` (FS_ListFiles + FS_FreeFileList collapsed — VFS/pk3-
    aware, distinct from PlatformHost::list_files). MockHost grows
    matching fixtures. EngineHost = 19 methods. ALSO (mechanical,
    evidence-forced): stringed.md's construction story is corrected —
    there is no `Common::default()`; `Common.stringed` is written
    explicitly in `Engine::new()`'s zeroed-alloc write-list
    (`StringEdPackage::default()` = Raven's `Clear(SE_FALSE)`), exactly
    the existing `modules`/`time_base` precedent, because BTreeMap/Vec/
    String fields are NOT all-zero-valid.
    **RULING: extend — cvar trio + fs_list_files (user, 2026-07-09)**

56. **tr-model closeout (TRM-Q1/Q2 + the systemic Cargo-edge omission):**
    (a) matcomp lives in `mp_engine_ghoul2` beside its sole live consumer
    (MC_UnCompressQuat ← UnCompressBone, tr_ghoul2.cpp:1158; the codec is
    part of the mdxa format the bone subset decodes); MC_Compress/
    MC_UnCompress get §20 notes if their callers prove dead at port. No
    dependency inversion; the FROZEN ghoul2 boundary holds. (b) The
    EngineHostView self-borrow (refines ruling 43):
    `render_models: Option<&mut RenderModels>` on the view —
    `ghoul2_call()` fills `Some`, `render_models_call()` sets `None`
    (sound: RenderModels' own methods never call `host.model_*`; they own
    the registry); a `model_*` call on `None` is a contract violation →
    panic, fatal-bug class per fork 1. One view type, invariant recorded
    in state-ownership.md. (c) MECHANICAL, systemic: every §F consumer
    crate is AUTHORIZED to add the `mp_host_interface` path dependency to
    its Cargo.toml as part of its first slice (acyclic — host-interface
    depends only on mp_qshared); the §F docs cite this ruling instead of
    each re-litigating the edge.
    **RULING: ghoul2 home; Option view; blanket Cargo-edge authorization (user, 2026-07-09)**
