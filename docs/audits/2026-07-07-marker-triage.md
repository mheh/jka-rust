# Marker triage: aggregated findings (2026-07-07)

## 1. Scope, method, motivation

**Scope**: every `PORT-NOTE` / `TODO` / `FIXME` / `todo!()` / `XXX` marker across
`crates/` — 1,159 markers total (520 `PORT-NOTE`, 414 `FIXME`, 159 `TODO`, 59
`todo!()`, 7 `XXX`).

**Method**: six parallel agents each swept a disjoint slice of `crates/`,
read every marker in context, cross-checked against `oracle/` where the
marker text alone didn't decide the call, and classified each into one of
four classes:

- **Class A** — silent no-op or placeholder-return stub. Code runs, produces
  no crash, but silently fails to do what the oracle does (or discards
  computed output).
- **Class B** — reachable panic (`todo!()`/`unimplemented!()` on a live or
  soon-to-be-live call path).
- **Class C** — partial implementation gap: a documented edge case, cosmetic
  omission, or narrow-but-real behavior gap short of a full no-op.
- **Class D** — benign note: either a stale leftover from an earlier
  skeleton/backfill pass describing a state that has since been fully wired
  up, or a faithful, verbatim preservation of Raven's own original C-source
  comment (not a porting gap at all).

Agents worked read-only against the current tree; findings below are
transcribed, not re-investigated, from their reports.

**Why this audit ran**: two previous bugs shipped past review because a
marker-flagged gap wasn't caught in time:

- **GlobalUse stub** — a marker-flagged silent no-op stub shipped into the
  MP path and produced a real gameplay regression before it was caught,
  demonstrating that Class-A-style stubs can hide behind a marker comment
  indefinitely if nobody sweeps for them.
- **Option\<enum\> niche-layout bug** — zeroed `Option<enum>` fields decode
  as `Some(variant 0)` instead of `None` under Rust's niche-optimized
  layout, so faithful-looking port code can misbehave without any marker
  at all. This audit doesn't re-check for that class directly, but it's
  the standing reminder that marker text is not a reliable signal on its
  own — behavior has to be checked, which is why every agent was asked to
  verify against oracle rather than trust the comment.

## 2. Totals

Split by shipping surface: **MP-shipping** = `crates/mp/game` + `mp/bg` +
`mp/qshared` + `mp/abi` (game-side) + deps that link into the `jampgame`
dylib; **MP-host-side** = `crates/mp/engine/*`, `crates/mp/app`,
`crates/native/*`, `crates/abi-transport`, `crates/mp/engine-select`,
`crates/ui`, `crates/cgame`, `crates/mp/renderer` (separate client/server
host binaries and VMs, not the jampgame dylib); **SP** = everything under
`crates/sp/*` and `crates/jagame`.

| Class | MP-shipping | MP-host-side | SP | Total |
|---|---:|---:|---:|---:|
| A (silent no-op/stub) | 17 | 10 | 2 | 29 |
| B (reachable panic) | 0 | 7 | 18 | 25 |
| C (partial gap) | 9 | 7 | 2 | 18 |
| D (benign note) | 855 | 40 | 69 | 964 |
| **Total** | **881** | **64** | **91** | **1,036** |

Note: the 1,036 tallied here is lower than the 1,159-marker scope count
because agents sampled/deduplicated dense repeating marker shapes (e.g. a
single stale `PORT-NOTE(raw-ptr-skeleton-no-world-handle)` boilerplate
pattern repeated dozens of times per file) into one Class-D judgment per
shape rather than re-litigating every individual line; several D-counts are
explicitly stated as sampled/approximate in the source reports.

## 3. Class A findings (worst-first, clustered)

### Event/effect core stubs

The single highest-impact cluster: three sibling functions in `g_utils.rs`
that sit in what one report calls the "ctx-free boundary set" and were
never retrofitted when `level.time`/entity-alloc access landed elsewhere.

- **A** `crates/mp/game/src/g_utils.rs:1323` `G_AddEvent` — never sets
  `ent->eventTime` (nor client `externalEventTime`); the "BLOCKED: no ctx to
  reach level.time" writes are commented out, so `g_main.rs`'s stale-event
  sweep (clears events older than `EVENT_VALID_MSEC` by reading
  `eventTime`) treats every event as instantly stale once `level.time >
  300ms`. [effects/temp-entity/dropped-item/freeAfterEvent event playback
  silently broken for essentially the whole match; 98 call sites]
- **A** `crates/mp/game/src/g_utils.rs:1353` `G_PlayEffect` — entire body is
  commented out ("G_TempEntity requires ctx"); returns null without
  spawning the `EV_PLAY_EFFECT` temp entity. [every game-side visual effect
  spawned via G_PlayEffect (~100 call sites across g_combat, g_missile,
  g_trigger, NPC AI, turrets, etc.) silently never appears; callers
  null-check so no crash, just missing effects]
- **A** `crates/mp/game/src/g_utils.rs:1372` `G_PlayEffectID` — same
  commented-out stub returning null. [additionally breaks the bg_channel
  `play_effect_id` upcall in `bg_channel/game_impl.rs:578`, which then
  reports `ENTITYNUM_NONE` to bg-tier callers]

### Dropped out-parameter class (vec3 taken by value)

A systematic bug shape, not a one-off: a `vec3_t` out-parameter is declared
`Copy`/by-value instead of `&mut`, so the callee's computed result never
reaches the caller. The source reports flag this as deserving a signature
sweep across the codebase beyond just the marker-flagged instances found
here.

- **A** `crates/mp/game/src/NPC_AI_Stormtrooper.rs:1267` `ST_OffsetLook` —
  `out: vec3_t` is taken by value (no `&mut`), so the offset-look vector it
  computes is discarded; caller `ST_LookAround` (line ~1316-1322) passes
  `lookPos` which stays `[0,0,0]` for 3 of 4 investigate-animation phases
  (perc ≥ 0.25), then feeds that zero vector into `NPC_FacePosition`.
  [Stormtrooper "look around while investigating" animation makes the NPC
  snap to face world origin (0,0,0) instead of scanning around, for 75% of
  the investigate-look cycle.]
- **A** `crates/mp/game/src/bg_pmove.rs:244` `PM_pitch_roll_for_slope` —
  `storeAngles: vec3_t` taken by value (Copy) instead of out-pointer, so
  computed slope PITCH/ROLL banking is written to a local copy and
  discarded. [vehicles never bank to match terrain; silent, every frame]
- **A** `crates/mp/game/src/bg_g2_utils.rs:20` `BG_AttachToRancor` —
  `out_origin`/`out_angles` are by-value `vec3_t` not out-pointers; all
  writes discarded on return. [player held by a Rancor never snaps to
  mouth/hand bolt; stale origin/viewangles]

### Vehicle cluster

Every vehicle interaction path found by these agents has a live gap.

- **A** `crates/mp/game/src/g_utils.rs:1758` `TryUse` — rider-side vehicle
  dispatch unwired (`PORT-NOTE(vehicle-dispatch-not-wired)`): pressing use
  while in a vehicle just clears `BUTTON_USE` and returns instead of
  calling the vehicle's Eject. [players can never exit a vehicle via use]
- **A** `crates/mp/game/src/g_utils.rs:1835` `TryUse` — target-side vehicle
  dispatch unwired: using a looked-at `CLASS_VEHICLE` entity clears
  `BUTTON_USE` and returns instead of dispatching board logic. [players can
  never board a vehicle via use]
- **A** `crates/mp/game/src/NPC_reactions.rs:1088` `NPC_Use` —
  `CLASS_VEHICLE` branch: EjectAll/Eject/Board are all empty (comment-only)
  arms; `CLASS_GONK` Add_Batteries call is also commented out, not
  implemented. [Using/interacting with a vehicle NPC entity (self-use to
  eject all, use-while-riding to eject, use-to-board) is a complete no-op;
  using a Gonk droid to recharge battery charge is also a no-op.]
- **C** `crates/mp/game/src/w_force.rs:3895` `ForceThrow` — force-push/pull
  against a vehicle-mounted player skips
  `vehEnt->m_pVehicle->m_pVehicleInfo->Eject(...)` (vehicle vtable dispatch
  unresolved); left as a no-op instead of ejecting the rider. [runtime
  effect: pushing/pulling a player who is on a vehicle within range
  triggers the knockdown/vehicle branch but never ejects them — vehicle
  riders are immune to force-push ejection] *(filed as Class C by its
  source report, but part of this cluster.)*
- **C** `crates/mp/game/src/w_force.rs:1601` `ForceGrip` — same vehicle
  vtable Eject() gap: gripping a vehicle-mounted target enters the vehicle
  branch but performs no eject action (silent no-op inside an otherwise
  faithful branch). [runtime effect: Force Grip cannot pull vehicle riders
  off their vehicle] *(filed as Class C, same cluster.)*
- **A** `crates/mp/game/src/g_vehicles.rs:126` `Vehicle_SetAnim` — MP body
  drops the actual `BG_SetAnim(&client->ps, ...)` call (staged pending
  bg-channel retrofit) and only copies the stale `ps.legsAnim` back to
  `s.legsAnim`. [vehicle dismount-roll, animal bucking/mount, and walker
  board animations silently never play on any MP vehicle]
- **A** `crates/mp/game/src/bg_pmove.rs:369` `PM_SetVehicleAngles` — C's
  `else if (normal)` / `else` in-air split collapsed into one unconditional
  `PM_pitch_roll_for_slope` call; in-air pitch calc and in-air
  banking-speed slowdown (×0.125×frametime) dropped. [airborne vehicles
  bank at full ground speed with zero air pitch]

### Singles

- **A** `crates/mp/game/src/NPC_reactions.rs:393` (in `NPC_Pain`, unnamed
  pain-anim-timing block) — `anim_length`/`frame_lerp` are hardcoded
  placeholders (`30`, `1`) replacing the oracle's real lookup
  `bgAllAnims[localAnimIndex].anims[pain_anim].numFrames * frameLerp`, used
  to set `painDebounceTime`. [Every NPC's post-pain-reaction lockout is a
  fixed ~30ms instead of the real per-animation duration (typically
  several hundred ms+), so pain-reaction spam/anim-interrupt timing is
  wrong network-wide for all humanoid NPCs.]
- **A** `crates/mp/game/src/bg_misc.rs:1401` `BG_ValidateSkinForTeam` —
  unconditionally forces skin to plain "red"/"blue" instead of only when
  `BG_FileExists(model_<skin>.skin)` fails. [custom _red/_blue team skin
  variants always reset to default on every clientinfo update]
- **A** `crates/mp/game/src/bg_saber.rs:2812` `PM_SetSaberMove` —
  PORT-NOTE claims the oracle's walkback-anim branch is empty, but oracle
  `bg_saber.c:3950-3953` assigns `anim = PM_GetSaberStance()`; Rust branch
  is a true no-op. [walking backward with saber drawn keeps raw walk anim —
  reintroduces the saber-through-leg bug Raven fixed]
- **A** `crates/mp/game/src/g_utils.rs:1669` `TryHeal` — returns `qfalse`
  unconditionally with a "missing bgSiegeClasses" placeholder (note:
  `bgSiegeClasses` now exists on `GameWorld.bg_state` per g_saga wiring, so
  the block is stale but the stub is still live). [Siege class-based
  healing of objective objects is completely non-functional]
- **A** `crates/mp/game/src/g_utils.rs:1578` `G_UseDispenserOn` —
  `HI_AMMODISP` branch only bumps the debounce timer, never grants ammo
  (missing weaponData/ammoData table wiring). [Siege ammo dispensers
  silently give no ammo]
- **A** `crates/mp/game/src/g_utils.rs:1620` `G_CanUseDispOn` —
  `HI_AMMODISP` branch always returns 0 (same missing-table placeholder,
  plus a hardcoded `LAST_USEABLE_WEAPON=16` placeholder). [ammo dispensers
  always report unusable, compounding the G_UseDispenserOn gap]

### Host-side / boot-shell stubs (agent-6, own subsection)

MP-host-side (not game logic — never touches `jampgame`):

- **A** `crates/mp/engine/qcommon/src/common/boot_stubs.rs:32`
  `fs_init_filesystem` — no-op where `FS_InitFilesystem` loads pak/base
  dirs and `ERR_FATAL`s on unreadable `mpdefault.cfg`. [server boots with
  zero filesystem/asset access, silently]
- **A** `crates/mp/engine/qcommon/src/common/boot_stubs.rs:13` `cvar_init`
  — no-op stub where `Cvar_Init` registers the entire cvar system; called
  unconditionally at com_init step 3. [engine has no functioning cvars]
- **A** `crates/mp/engine/qcommon/src/common/boot_stubs.rs:25` `cmd_init`
  — no-op stub where `Cmd_Init` registers the console command table. [no
  console commands ever registered]
- **A** `crates/mp/engine/qcommon/src/common/boot_stubs.rs:19` `cbuf_init`
  — no-op stub where `Cbuf_Init` sets up the deferred command buffer.
  [command buffering/execution pipeline absent]
- **A** `crates/mp/engine/core/src/lifecycle.rs:50` `com_init_body` —
  `Com_ParseCommandLine` skipped; `command_line` discarded via `let _ =`.
  [all +set/+exec/+map CLI args silently thrown away]
- **A** `crates/mp/engine/core/src/lifecycle.rs:55` `com_init_body` —
  `Com_StartupVariable` and `Rand_Init` skipped. [+set cvars never applied;
  RNG never seeded]
- **A** `crates/mp/engine/core/src/lifecycle.rs:58` `com_init_body` —
  `Com_InitJournaling`, config execs (mpdefault.cfg/jampserver.cfg/
  autoexec.cfg), and steps 13-29 cvar-registration block skipped. [none of
  Raven's default cvars registered, no configs executed]
- **A** `crates/mp/engine/core/src/lifecycle.rs:62` `com_init_body` —
  `SV_Init()` (and dedicated/client-init tail) never invoked during
  com_init. [server subsystem never initialized at boot]
- **A** `crates/ui/src/lib.rs:15` `vmMain` — unconditionally returns 0,
  never dispatches any `AbiCommand`; entire MP `ui.so` is a no-op stub
  (documented SEAM-D10 milestone, issue #1). [any engine call into ui.so
  silently no-ops]
- **A** `crates/cgame/src/lib.rs:15` `vmMain` — same pattern; entire MP
  `cgame.so` is a no-op stub (documented SEAM-D10 milestone). [any engine
  call into cgame.so silently no-ops]

SP:

- **[SP] A** `crates/sp/app/src/main.rs:1` `main` — SP host binary
  `fn main() {}` is a fully empty stub wiring nothing. [entire SP app
  produces no runtime behavior]
- **[SP] A** `crates/jagame/src/lib.rs:130` `GetGameAPI` — silently omits
  `GI_Init` + `gameinfo_import_t` wiring the original performs at
  `g_main.cpp:906-914`. [valid-looking export table but game-info
  subsystem never initialized]

## 4. Class B (reachable panics)

MP host-side — `server_host` shim panics; these are the panics that matter
first once ctx injection lands, since the null-ctx dispatch path is live
today:

- **B** `crates/mp/engine/server/src/server_host.rs:200`
  `game_system_calls_shim` — null-ctx fallback dispatch implements only
  `G_PRINT`; every other jampgame syscall hits `todo!()`. [live path: ctx
  is always null today, so the host panics on the first non-G_PRINT
  syscall during GAME_INIT]
- **B** `crates/mp/engine/core/src/sv_init_game_progs.rs:43`
  `sv_init_game_progs` — ctx hardcoded to null instead of injecting
  `&mut Engine.sv`, forcing the G_PRINT-only null branch. [same panic;
  GAME_INIT runs right after this in mp/app/src/main.rs]
- **B** `crates/mp/engine/server/src/server_host.rs:146`
  `sv_game_system_calls` — real (non-null-ctx) dispatcher also implements
  only `G_PRINT`, `todo!()` on every other trap. [currently unreachable;
  becomes the live panic path once ctx injection lands]
- **B** `crates/mp/engine/core/src/lifecycle.rs:156` `com_frame_body` —
  entire per-frame body is `todo!()`. [panics on first call; dormant only
  because main.rs never calls com_frame yet]
- **B** `crates/mp/engine/core/src/lifecycle.rs:164` `com_error_recover` —
  `ERR_DROP` catch-side recovery is `todo!()`, reached via com_frame's
  catch arm. [panics on first ComError inside a frame; dormant with
  com_frame]
- **B** `crates/mp/engine/core/src/lifecycle.rs:115` `com_shutdown` —
  entire body is `todo!()`. [panics if invoked; quit/shutdown path
  currently unwired]
- **B** `crates/mp/engine/qcommon/src/vm/module_registry.rs:166`
  `ModuleRegistry::restart` — `todo!()` unconditionally; VM_Restart
  drop+recreate not ported. [crash on any VM restart path; latent, no
  caller yet]

SP — the entire `jagame`/SP host surface is `todo!()`-stubbed; these are
expected/unshipped rather than surprises (SP is simply not wired up yet):

- **[SP] B** `crates/jagame/src/lib.rs:156` `init` — `todo!()`; InitGame,
  called by engine `ge->Init` on every level load. [panics before any SP
  gameplay can start; masks everything below]
- **[SP] B** `crates/jagame/src/lib.rs:250` `run_frame` — `todo!()`;
  `G_RunFrame` per-server-frame tick. [immediate panic on first frame
  after init]
- **[SP] B** `crates/jagame/src/lib.rs:244` `client_think` — `todo!()`;
  `ClientThink` per client usercmd. [panics on first player input tick]
- **[SP] B** `crates/jagame/src/lib.rs:206` `client_connect` — `todo!()`;
  `ClientConnect`. [panics on player join]
- **[SP] B** `crates/jagame/src/lib.rs:216` `client_begin` — `todo!()`;
  `ClientBegin`. [panics on spawn-in]
- **[SP] B** `crates/jagame/src/lib.rs:238` `client_command` — `todo!()`;
  `ClientCommand`. [panics on any client console command]
- **[SP] B** `crates/jagame/src/lib.rs:226` `client_userinfo_changed` —
  `todo!()`; `ClientUserinfoChanged`. [panics on userinfo update]
- **[SP] B** `crates/jagame/src/lib.rs:232` `client_disconnect` —
  `todo!()`; `ClientDisconnect`. [panics on disconnect]
- **[SP] B** `crates/jagame/src/lib.rs:184` `shutdown` — `todo!()`;
  `ShutdownGame`. [panics on map change or exit]
- **[SP] B** `crates/jagame/src/lib.rs:189` `write_level` — `todo!()`;
  `WriteLevel`. [panics on any save attempt]
- **[SP] B** `crates/jagame/src/lib.rs:195` `read_level` — `todo!()`;
  `ReadLevel`. [panics on any savegame load]
- **[SP] B** `crates/jagame/src/lib.rs:201` `game_allowed_to_save_here` —
  `todo!()`; `GameAllowedToSaveHere`. [panics when save-allowed check is
  queried]
- **[SP] B** `crates/jagame/src/lib.rs:262` `console_command` — `todo!()`;
  `ConsoleCommand`. [panics on sv-side console command]
- **[SP] B** `crates/jagame/src/lib.rs:256` `connect_navs` — `todo!()`;
  `G_ConnectNavs` during map load. [panics during map load if reached]
- **[SP] B** `crates/jagame/src/lib.rs:267` `game_spawn_rmg_entity` —
  `todo!()`; `G_GameSpawnRMGEntity`. [panics only on RMG-based maps, rare
  in JKA SP]
- **[SP] B** `crates/sp/engine/server/src/server_host.rs:116`
  `sv_init_game_progs` — `todo!()`; SP `SV_InitGameProgs` equivalent
  (sv_game.cpp:478), reached at server/map startup. [panics before any SP
  server can host a map]
- **[SP] B** `crates/sp/engine/server/src/server_host.rs:125`
  `sv_shutdown_game_progs` — `todo!()`; SP `SV_ShutdownGameProgs`
  (sv_game.cpp:403). [panics on shutdown/map change]
- **[SP] B** `crates/sp/game/src/world/game_world.rs:30`
  `GameWorld::zeroed` — `todo!()` in the STATE-D9 constructor for all
  per-level entity/level state; would be reached from InitGame once
  `jagame lib.rs:156` is filled in. [SP game module cannot initialize a
  level; currently masked by the earlier InitGame todo!()]

## 5. Class C findings

MP-shipping:

- **C** `crates/mp/game/src/g_cmds.rs:1720` `G_TeamForSiegeClass` /
  `Cmd_SiegeClass_f` — `SIEGETEAM_TEAM1`/`SIEGETEAM_TEAM2`/
  `MAX_SIEGE_CLASSES` aren't yet resolved from the ported bg_saga surface;
  implemented with local literal consts (1/2/12) matching Raven's enum
  discriminants, flagged for verification once the real enum lands.
  [runtime effect: none currently observed — functionally faithful, but a
  future enum reorder in bg_saga could silently desync this]
- **C** `crates/mp/game/src/NPC_spawn.rs:2982` `NPC_Kill` (console cmd,
  team-name-not-found path) — `Com_Printf("%s\n", TeamNames[n])` loop is
  commented out entirely, so the "Valid team names are:" error listing
  prints nothing after the header. [runtime effect: `npc kill team
  <badname>` server console command loses its diagnostic team list; core
  kill logic unaffected]
- **C** `crates/mp/game/src/NPC_spawn.rs:3009` `NPC_Kill` (console cmd,
  unrecognized-team path) — same missing `TeamNames[n]` printout, duplicate
  of the above in the second error branch. [runtime effect: same cosmetic
  gap in a different error branch of the same debug command]
- **C** `crates/mp/game/src/NPC_AI_Remote.rs:318` `Remote_Fire` — Raven
  cached last enemy origin in `static vec3_t enemy_org1`; port uses a
  zero-initialized local, so if called with `npc.enemy == None` it aims
  the missile at the world origin instead of retaining the last known
  enemy position. [runtime effect: only manifests if Remote_Fire is ever
  invoked without a current enemy, which does not appear to happen on the
  normal Remote-droid fire path; likely unreachable in practice]
- **C** `crates/mp/game/src/g_ICARUScb.rs:832` `Q3_RemoveEnt` — skips
  Raven's `pVeh->m_pVehicleInfo->EjectAll(pVeh)` before freeing a
  `CLASS_VEHICLE` NPC (vehicle-eject parked). [ICARUS-scripted removal of a
  vehicle leaves riders logically attached to an entity freed 100ms later]
- **C** `crates/mp/game/src/g_vehicles.rs:927` `Initialize` — skips the MP
  `BG_SetAnim(..., BOTH_VS_IDLE, ...)` that applies the vehicle's initial
  landed/idle pose (`VEH_GEARSOPEN` flag itself is still set). [cosmetic:
  vehicle may spawn without its landed pose explicitly applied]
- **C** `crates/mp/game/src/g_utils.rs:1906` `TryUse` — touch-pointer
  identity comparison can't be reproduced faithfully after the
  fn-pointer-to-enum port. [known parked item, previously ruled on]
- (ForceThrow/ForceGrip vehicle-eject gaps at `w_force.rs:3895`/`1601` are
  filed as Class C but listed above under the vehicle cluster, section 3.)

MP-host-side:

- **C** `crates/mp/app/src/main.rs:92` `main` — dedicated server loop
  (sleep/console_poll/net_poll/com_frame) never runs; process exits after
  one GAME_INIT/GAME_SHUTDOWN round-trip. [server cannot actually serve;
  acceptance-driver only]
- **C** `crates/mp/app/src/main.rs:77` `main` — GAME_INIT called with
  zeroed `svs.time`/`Com_Milliseconds` args. [game module inits with wrong
  timing/seed inputs]
- **C** `crates/mp/engine/qcommon/src/common/common.rs:52` `com_printf` —
  only stdout; `rd_buffer` redirect capture and logfile persistence
  unported. [rcon output never captured; no dedicated-console disk
  logging]
- **C** `crates/native/platform/src/module_loader/loader.rs:35`
  `sys_load_dll` — `Sys_UnpackDLL` pure-server pk3 unpack pre-step not
  run. [game DLL shipped only inside a pk3 fails to load; on-disk DLL
  works]
- **C** `crates/native/platform/src/module_loader/loader.rs:75`
  `sys_load_dll` — release-build in-loader `ERR_FATAL` branch on dlsym
  failure not implemented; both builds take print+return-None. [still
  fatal via caller, but wrong error message/mechanism vs Raven]
- **C** `crates/abi-transport/src/generic/engine.rs:78` `RunStatic` —
  trait has no call implementations yet. [Static/wasm32 outbound backend
  nonfunctional; default native CEngine path unaffected]
- **C** `crates/mp/engine-select/src/lib.rs:17` (type alias) — wasm32
  outbound backend concrete type unresolved, currently aliases the
  incomplete `Static`. [wasm32 build not functional; native unaffected]

SP:

- **[SP] C** `crates/sp/cgame/src/media/cgs_t.rs:48` `cgs_t`
  (`OpaqueClientInfo_t`) — `clientInfo_t` kept as opaque `[u64; 62]` blob
  because sp_cgame lacks a dep on sp_game where it's ported. [cgame logic
  needing per-client info via cgs_t blocked until cross-crate wiring
  lands]
- **[SP] C** `crates/sp/engine/rmg/src/lib.rs:7` crate root — only
  faithful RMG C enums ported; `CRMManager`/`CRMMission`/`CRMInstance*`/
  `CRMPathManager` unimplemented though `RMG_Init` is called from a real
  path (g_misc.cpp:769). [RMG terrain/mission generation has no
  implementation]

## 6. Corpus coverage caveat

The 2,400-frame A/B rig PASSED despite these stubs, so the current bot
corpus never exercises these paths — corpus coverage must grow (item
pickups/combat events, vehicles, sieges) before rig-verification of any
fix to the findings above means anything. A large fraction of the Class A
list — G_AddEvent/G_PlayEffect/G_PlayEffectID, both TryUse vehicle
branches, Vehicle_SetAnim, TryHeal, the ammo dispenser pair, the vehicle
Force-push/grip eject gaps — sit behind gameplay actions (item events,
vehicle boarding, siege objective interaction) that a bot corpus doing
plain movement/combat sampling simply never triggers. A rig run that stays
green through all of these fixes is not evidence the fixes are correct; it
is only evidence the corpus doesn't touch them yet.

## 7. Class D

D-count: 964 (dominant patterns only, not itemized — see source reports in
`/private/tmp/claude-502/.../scratchpad/marker-triage/agent-{1..6}*.md`
for full per-file line lists).

Two patterns account for essentially all of it:

1. **Stale skeleton-era notes over now-complete code** — `PORT-NOTE`
   headers (`raw-ptr-skeleton-no-world-handle`, `unported-fn/global`,
   `missing-const`/`unported-const`, `missing-global-field`/
   `unported-global-table`/`MISSING-SYMBOL`, `vec3-outparam-seam`,
   `parked-dep`/`bg-dep`, `unported-type`) written during an early
   signature-skeleton/`fnskel.py` pass, describing an architecture
   question ("no `&Engine`/`&mut GameWorld` in this signature", "symbol X
   doesn't exist yet") that a later backfill pass fully resolved. Every
   sampled instance across all six reports now has a full implementation
   wired to `ctx: GameContext` with the cited symbol actually defined and
   in use — the comment simply wasn't deleted after backfill.
2. **Raven's own comments, preserved verbatim** — `FIXME`/`rwwFIXMEFIXME`/
   `OVERRIDEFIXME` comments and dead `if(0)` branches confirmed
   byte-for-byte identical to the oracle C source (checked via direct
   `grep` against `oracle/oracle/codemp/game/*.c`). These document
   upstream Raven dev notes and known engine-era gaps, not porting gaps.
