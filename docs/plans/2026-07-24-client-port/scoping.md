# cgame + ui port — scoping (pre-sit-down)

Ruled 2026-07-24: the next major phase is MP cgame + ui, ported **directly to
the current idiomatic shape** (String/`&str`, `bool`, threaded ctx, owned
worlds, native/* crates) — the blind-faithful-then-migrate pipeline is retired
for these modules; the translation dictionary the campaigns settled is applied
during transcription. Differential validation is retained: modules ship as
drop-in DLLs under an existing client (OpenJK/retail) with a trap-stream
referee. End state after both modules + the client engine island: our own
`jamp` executable (see memory `next-phase-cgame-ui-2026-07-24`).

Survey ground truth (2026-07-24, two oracle surveys; this doc is the digest —
full reports in session transcripts).

## Sizing

| module | lines | character |
|---|---:|---|
| cgame | 58,552 | pure C, no C++ anywhere in the module |
| ui | ~29,600 | pure C; 74% = ui_main.c (11.7k) + ui_shared.c (10.2k) |
| bg shared into cgame | 38,701 | ALREADY PORTED (mp_bg + the four vehicle-NPC files) |

## Frozen-vs-free classification (PRECEDES the root-type design)

Ruled 2026-07-24 (user): the root types are designed only after every piece of
state is classified against the lessons the game module taught — engine
pointers into shared memory (the gentity prefix / trap_LocateGameData class),
layout-punning casts (gentity_t → sharedEntity_t class), and the bg↔module
boundary (the G_SoundIndex/GameCallbacks class). Four classes:

- **Class A — engine-RETAINED shared memory.** The engine stores a pointer
  into module memory and reads/writes through it across calls. Byte-frozen
  forever: `#[repr(C)]`, Raven field names/order, size/offset asserts, NO
  owned-String/bool fields, writes through a choke point if the module also
  writes. **Census COMPLETE (2026-07-24, verified against oracle
  cl_cgame.cpp/cl_ui.cpp/RoffSystem/FxSystem/G2_API):**
  - `cg.sharedBuffer` (2048 bytes, `cg_local.h:997`) — the ONE retained
    pointer: engine stores it in `cl.mSharedMemory` (`cl_cgame.cpp:1682`)
    and actively READS AND WRITES ~15 typed POD structs through it
    (`TCGTrace`, `TCGG2Mark`, `TCGCameraShake`, `TCGGetBoltData`,
    `TCGVectorData`, `TCGPointContents`, `autoMapInput_t`, `TCGMiscEnt`, six
    `ragCallback*_t` — all `cg_public.h:442-591`). Engine-interpreted RPC
    mailbox, sibling of the game module's `sv.mSharedMemory`/ICARUS channel.
  - **A-lite:** `CG_GET_ORIGIN_TRAJECTORY`/`CG_GET_ANGLE_TRAJECTORY` return
    `&cg_entities[n].nextState.pos/apos` (`cg_main.c:293-297`) and the
    engine's ROFF system writes trajectory fields DIRECTLY through that
    pointer (`RoffSystem.cpp:1024-1039`) within the call. So `trajectory_t`
    inside cgame's persistent `centity_t.nextState` is byte-frozen (it
    already is — wire type), and `cg_entities` storage must be
    address-stable while such a call can occur.
  - **Dead surface (do not port):** `CG_IMPACT_MARK`, `CG_GET_GHOUL2`,
    `CG_GET_MODEL_LIST`, `CG_CALC_LERP_POSITIONS`, `TCGPositionOnBolt` — no
    engine call site exists (§20 dead-surface rule).
  - **ghoul2 fields are neither A nor B:** the engine `new`s/`delete`s
    `CGhoul2Info_v` on ITS OWN heap and writes only the handle VALUE back
    (`G2_API.cpp:601-664,496-565`) — cgame/ui never dereference it. Port as
    an opaque handle type; zero layout obligations module-side.
  - **ui: ZERO Class A.** No shared-buffer registration exists for uivm;
    every `VM_Call(uivm,…)` engine-wide passes scalars only; all ui traps
    are copy-semantics. The entire ui module state is Class C except the
    Class-B copy shapes.
- **Class B — copied at the seam per call.** The struct's layout is frozen
  (`#[repr(C)]` + asserts) because the bytes cross, but the engine retains
  nothing — so module-side CONTAINERS of these are free, and data sourced
  from them may decode to owned types at the copy boundary (the
  MSG_ReadString → Latin-1 String discipline). Expected members: snapshot_t/
  gameState_t/usercmd_t reads, refEntity_t/refdef_t submissions,
  uiClientState_t, pc_token_t. Copy semantics to be cite-confirmed per trap.
- **Class C — module-private.** Never engine-visible: String/bool/Option/
  arenas apply directly. Expected: cg/cgs interiors (vote strings, chat
  buffers, clientInfo name fields), uiInfo_t, MenuSystem, media handle bags.
  Only what the Class-A census clears lands here.
- **Class D — in-module shared with bg.** Not engine-visible but consumed by
  compiled-in bg code (centity_t's hand-unified bgEntity_t prefix +
  baseEnt/entSize stride-aliasing; Vehicle_t). The C aliasing dissolves ONLY
  by routing through the mp_bg accessor seam (as game did); until each
  struct's accessor design is settled it is treated as frozen. The
  G_SoundIndex lesson applies as design law: bg reaches module services
  exclusively through the callbacks trait with own-state impls — no ctx-less
  boundary fns, no ambient cells, in either new module, from day one.

## Owned-state designs

### CgWorld (analog of GameWorld)

- `cg_t cg` (~153 fields: prediction state incl. `predictedPlayerState`,
  refdef, snapshot ptrs, scoreboard, chat/centerprint char buffers → String),
  `cgs_t cgs` (~55 fields; embeds `cgMedia_t` ~330 fields of which ~206 are
  pure qhandle/sfx/fx handles, plus `clientinfo[MAX_CLIENTS]` — ~90
  fields/client, 11 char[] each → String), `cg_entities[MAX_GENTITIES]`
  (centity_t ~72 fields), `cg_weapons[]`/`cg_items[]`, ~144 vmCvar mirrors +
  130-entry cvarTable.
- **Pools redesigned, not transcribed**: `localEntity_t` (512, intrusive
  prev/next free-list, 9-variant union with per-instance `thinkFn` C fn ptrs)
  and `markPoly_t` → arena/slab + index handles; think callbacks → enum
  dispatch. The only two idiom-mismatch pockets in the module.
- Per-file statics fold into CgWorld/subsystem structs: cg_main's MiscEnts +
  strPool + spawn-var scratch, predict's `cg_pmove`/`cg_vehPmove`, weapons'
  `g2WeaponInstances[]` + frame latches, draw/view debounce latches, marks'
  shader-anim cache, saga string scratch.
- `centity_t.npcClient` (malloc'd clientInfo with manual "always free, never
  stomp" discipline) → owned `Option<Box<…>>`; the `void*` ghoul2 handles stay
  opaque handles across traps.
- Threading: `CgContext { world: &mut CgWorld, … }` mirroring GameContext;
  vmMain entrypoints own the instance.

### UiWorld

- `uiInfo_t` is the spine (~50 fields incl. server-browser state, list
  caches, scrolltext). `uiStatic_t uis` is DEAD (extern with no definition
  anywhere — Q3 lineage) — not ported.
- ui_force.c's free-floating externs (uiForceSide/Rank/Used… ) fold into
  UiWorld — outside uiInfo_t only by file-org accident.
- Per-file statics fold in: gameinfo's arena/bot caches, saber parse buffer +
  shader caches, players' preview timers, main's anim-file scratch + connect
  latches + siege globals.
- ui_main.c hand-maintains a forked mini-copy of bg_panimate's animation
  cache (Raven comment admits "kept in sync manually") — **collapse it**:
  ui reuses mp_bg's real animation module. Homogenization win over Raven.

### MenuSystem (ui_shared.c) — the shared-framework crate

ui_shared.c compiles into BOTH ui and cgame (vcproj-confirmed), each host
getting its own instance of `Menus[64]`, the open-menu stack, the bump
`memoryPool` (128KB cgame / 2MB ui) and the `String_Alloc` intern pool.
Port: ONE canonical crate (working name `mp_menu`), generic over a
`DisplayContext` trait (Raven's `displayContextDef_t`, a ~50-fn-pointer
vtable each host fills — render/text/cvar/feeder/ownerDraw/sound/cinematic).
Owned per host by composition: `UiWorld.menus: MenuSystem`,
`CgWorld.menus: MenuSystem`. menuDef/itemDef raw-pointer graph → arena +
indices; the 83+34 keyword→fn-ptr parse tables → match dispatch.

## bg reuse

- cgame compiles in 13 bg/q files + AnimalNPC/FighterNPC/SpeederNPC/WalkerNPC
  (vehicle prediction) — all already in mp_bg/mp_game's ported set.
- The `#elif defined CGAME` branches (concentrated: bg_saberLoad 14,
  bg_vehicleLoad 13) are the `GameCallbacks` surface: cgame implements the
  same trait, supplying `trap_S_RegisterSound`/`trap_R_RegisterShader` where
  game supplies `G_SoundIndex`/noop. Task #19's own-state impl pattern is the
  template. ui similarly needs the `UI_EXPORTS`/`WE_ARE_IN_THE_UI` branch
  story (cut-down vehicle load).
- bg_pmove has ZERO cgame ifdefs (branch-free shared movement). The seam is
  `cg_pmove.baseEnt = (bgEntity_t*)cg_entities; entSize = sizeof(centity_t)`
  — pointer-stride aliasing Rust cannot pun; cgame feeds pmove via the same
  entity-accessor abstraction mp_bg gave the game side, plus the trace/
  pointcontents callback pair backed by cgame's LOCAL snapshot-built clip
  list (CG_BuildSolidList/CG_ClipMoveToEntities — client-side collision, a
  real subsystem), foot-bolt handles, and the parallel `cg_vehPmove` context.

## ABI surface

- vmMain: cgame 32 entries, ui 12 (`UI_API_VERSION 7`).
- Traps: cgame 216 (G2API 54, render/scene/CM 55, FX 20, sound 16, misc 32…),
  ui 124 (G2API ~30 for menu 3D previews, LAN 17, renderer 15…). Under
  OpenJK/retail hosting the ENGINE provides all of these — no engine work
  needed for the drop-in stage.
- Layout-frozen set: already-ported (entityState/playerState/usercmd/trace/
  gameState/snapshot shapes) + NEW: `refEntity_t`/`refdef_t`/`poly(Vert)_t`
  (tr_types.h — renderer scene ABI), `glconfig_t`, `uiClientState_t`,
  `pc_token_t`, and cgame's `TCG*` sharedBuffer structs. `cg.sharedBuffer` is
  an untyped ABI mailbox for many vmMain calls — at the seam it stays
  byte-faithful (drop-in law); inland it decodes immediately to typed values.

## Validation rig

1. Drop-in: our cgame/ui dylib loaded by an existing client (OpenJK) against
   any server — live A/B like the jampgame era. Early step: verify module
   dlopen ABI under openjk.app with a stub.
2. Referee: cgame's observable output is its outbound trap stream given
   deterministic inputs (snapshots/gamestate/usercmds). Record real sessions,
   replay through oracle-cgame vs rust-cgame, byte-diff the trap streams —
   same architecture as the game referee. ui: trap-stream diff over scripted
   key/mouse event sequences + visual pass in-person.

## Ordering (sit-down decision)

Recommendation: **ui first**. Smaller (29.6k), self-contained, no prediction
timing risk, and it forces the MenuSystem crate + DisplayContext trait into
existence — which cgame needs anyway (cg_newDraw). It also shakes down the
port-to-idiom-during-transcription process on the lower-risk module. cgame
second, delivering the gameplay payoff on a proven process. Alternative
(cgame first) front-loads the payoff but builds the framework under pressure.

## Open decision points for the sit-down

1. Ordering (above).
2. Crate layout: NOT greenfield — `mp_ui`, `mp_cgame`, `mp_uishared`
   (shared framework home) already exist as wired workspace members from
   the type-port campaign; the decision is how each EVOLVES (esp.
   mp_uishared → idiomatic MenuSystem), plus bg callback impls per module.
3. CgWorld/UiWorld shapes as proposed (esp. pools→arena, callbacks→enum).
4. sharedBuffer policy: byte-faithful at seam, typed inland (proposed).
5. Referee rig design + what gets recorded.
6. Walker tooling: re-aim sweep.py/packets3.py at cgame/ui TUs with the
   translation dictionary baked into packet instructions (dictionary = the
   campaigns' settled mappings; enumerate in the kickoff doc).
