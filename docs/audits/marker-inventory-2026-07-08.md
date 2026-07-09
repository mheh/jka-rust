# Marker inventory — PORT-NOTE / TODO (regenerated 2026-07-08, post-audit)

Totals: 645 markers — 108 `TODO: Port`,
486 `PORT-NOTE`, 51 other TODO forms.

Regenerated from the current tree (branch `skeleton`). Same methodology as the
2026-07-05 inventory: `TODO: Port` markers are verdicted; `PORT-NOTE` and the
other TODO forms are a plain re-grep. The `TODO: Port` verdict taxonomy is:

- **LEGIT** — subject genuinely unported; marker accurate. (91)
- **COMPLEX** — subject (or its blockers) now ported, but resolving the marker
  needs non-mechanical work: authoring missing logic, threading `static mut`
  globals through `GameWorld`, or cross-tier naming. Listed first — these are
  the actionable findings. (15)
- **STALE-escalated** — mechanical fix known but crosses files; left in place. (2)

**Carry-forward convention.** Every `TODO: Port` marker in this tree is present
verbatim at the same site the 2026-07-05 inventory audited it (only line numbers
shifted). The COMPLEX/STALE verdicts turn on the crate tier graph
(native < qshared < bg < game) and the crate-level module-doc house convention —
neither changed across the five intervening campaigns — so those 17 verdicts are
carried forward unchanged. Deltas vs 2026-07-05: `GetModuleAPI` at
`crates/jampgame/src/lib.rs:336` (was LEGIT) closed; a new LEGIT marker appeared
at `crates/mp/engine/wasm-host/Cargo.toml:15` (wasmtime wiring, LOAD-D10). Net
`TODO: Port` count unchanged at 108. `PORT-NOTE` fell 546 → 486 and other TODO
forms 54 → 51 as the intervening integrate/oracle-review campaigns resolved
sites.

PORT-NOTE section is a plain re-grep (not audited); each entry shows its
`PORT-NOTE(<topic>)` tag, or `(untopiced)` for prose mentions of the marker.

## TODO: Port — COMPLEX (15)

### crates/mp/cgame/src/local/client_info_t.rs
- L57: ghoul2Weapons element type (CGhoul2Info_v*)
  - CGhoul2Info_v IS ported (crates/mp/engine/ghoul2/src/shared/cghoul2_info_v.rs) but mp_cgame naming engine-side ghoul2 state rides the open STATE-Q2 attachment question (ghoul2 is shared engine<->cgame).
- L142: ghoul2Model (CGhoul2Info_v*)
  - Same as ghoul2Weapons above — CGhoul2Info_v ported, blocked on STATE-Q2.

### crates/mp/qshared/src/common/mp/gentity.rs
- L94: Vehicle_t
  - Marker itself correctly states Vehicle_t IS ported (crates/mp/bg/src/vehicles/vehicle_s.rs) but cannot be named here because gentity_t lives in mp_qshared, which sits below mp_bg in the tier graph (native < qshared < bg < game) and mp_qshared cannot depend on mp_bg. Resolving requires an abi-seam/tier refactor moving gentity_t (or restructuring the tier graph), not a file-local fix; also shared with the `client: *mut c_void` field on the same struct.
- L168: gNPC_t
  - gNPC_t is already ported at crates/mp/game/src/npc/g_npc_t.rs:35, but it lives in mp_game (above mp_qshared in the tier graph), so gentity_t (in mp_qshared) cannot name it without an abi-seam/tier refactor — the same tiering blocker as the Vehicle_t/client fields on this same struct.

### crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs
- L29: Vehicle_t
  - Same tiering blocker as gentity.rs: Vehicle_t is ported at crates/mp/bg/src/vehicles/vehicle_s.rs but sharedEntity_t lives in mp_qshared, which cannot depend on mp_bg. Requires an abi-seam/tier refactor, not a file-local mechanical fix.

### crates/sp/abi/src/game/public/game_import_t.rs
- L431: CMiniHeap
  - CMiniHeap IS ported (crates/sp/engine/qcommon/src/miniheap/cmini_heap.rs) but sp_abi cannot depend on the engine tier — same cross-tier-naming class as Vehicle_t/gNPC_t; pointer param stays opaque pending a DEC ruling.

### crates/sp/cgame/src/media/cgs_t.rs
- L48: clientInfo_t (cross-crate, sp_game -> sp_cgame not wired)
  - clientInfo_t is fully ported at sp_game::shared::client_info_t::clientInfo_t (confirmed: `pub struct clientInfo_t` at that path), exactly as the marker itself states. But resolving this in cgs_t.rs would mean replacing `OpaqueClientInfo_t = [u64; 62]` (a #[repr(C)] field inside cgs_t, size/offset-asserted at line 259+) with the real cross-crate type, which requires adding a new sp_cgame -> sp_game dependency edge (confirmed absent in crates/sp/cgame/Cargo.toml) — an architecture-layering change with effects beyond this file, not a mechanical local fix.

### crates/sp/game/src/shared/weapon_info_s.rs
- L41: centity_t
  - centity_t IS ported (crates/sp/cgame/src/local/centity_s.rs, pub struct centity_t). But sp_game and sp_cgame are sibling tier-3 module crates with no dependency edge between them (docs/workspace-architecture.md); retyping this callback param to the real type would require adding an sp_game -> sp_cgame crate dependency, which the architecture forbids. The comment already documents this as a deliberate cross-tier opaque-pointer design, not an oversight.
- L47: centity_t
  - Same as line 41 — centity_t exists in sp_cgame but sp_game cannot depend on sp_cgame (same tier, no edge), so the opaque *mut c_void callback param is architecturally required, not a stale mechanical leftover.

### crates/sp/qshared/src/common/sp/gentity.rs
- L47: moverState_t (cross-tier: real enum lives in sp_game)
  - The real #[repr(i32)] enum exists at crates/sp/game/src/shared/mover_state_t.rs, but gentity_t lives in sp_qshared (tier 0), below sp_game (tier 3) in the crate graph (docs/workspace-architecture.md); qshared cannot depend on game. The c_int alias plus comment is the deliberate, permanent cross-tier design, not a stale leftover — fixing it would require restructuring the crate graph, not a file-local edit.
- L56: material_t (cross-tier: real enum lives in sp_game)
  - Real #[repr(i32)] enum exists at crates/sp/game/src/shared/material_t.rs; same cross-tier constraint as moverState_t (sp_qshared cannot depend on sp_game) prevents a file-local retype.
- L65: team_t (cross-tier: real enum lives in sp_game)
  - Real #[repr(i32)] enum exists at crates/sp/game/src/teams/team.rs; same cross-tier constraint (sp_qshared below sp_game) prevents a file-local retype of the noDamageTeam field.
- L130: gclient_s (cross-tier: real struct lives in sp_game)
  - Real struct gclient_t is ported at crates/sp/game/src/shared/gclient_s.rs, but it's a large #[repr(C)] type in sp_game (tier 3); gentity_t's `client` field would need to become *mut gclient_t, which both crosses the forbidden tier dependency and is a #[repr(C)] layout-relevant field guarded by offset asserts (though offset itself is pointer-size-stable) — not a simple file-local swap.
- L348: Vehicle_t (cross-tier: real struct lives in sp_game)
  - Real struct Vehicle_t is ported at crates/sp/game/src/vehicles/vehicle_t.rs, but it lives in sp_game (tier 3) while gentity_t lives in sp_qshared (tier 0); same forbidden upward-dependency issue blocks a file-local fix for m_pVehicle.
- L355: gNPC_t (cross-tier: real struct lives in sp_game)
  - Real struct gNPC_t is ported at crates/sp/game/src/npc/g_npc_t.rs, but it lives in sp_game (tier 3) while gentity_t lives in sp_qshared (tier 0); same cross-tier dependency block prevents retyping the NPC field.


## TODO: Port — STALE-escalated (2)

### crates/mp/engine/qcommon/src/lib.rs
- L1: module mp_engine_qcommon
  - Validator's claim does not hold. The current one-line form `//! \`mp_engine_qcommon\` crate. //TODO: Port module mp_engine_qcommon` (no separate Source line, subject = crate name) is the established, uniformly-replicated house convention for crate-level module docs: grep shows the identical pattern verbatim across the other crates' lib.rs files (mp_ui, mp_renderer, mp_uishared, mp_engine_icarus, mp_engine_ghoul2, mp_engine_server, mp_engine_botlib, mp_engine_client, mp_cgame, sp_ui, sp_renderer, sp_bg, sp_uishared, sp_engine_icarus, sp_engine_ghoul2, sp_engine_server, sp_engine_qcommon, sp_engine_client, sp_cgame, native_containers, plus multi-line variants in mp_bg/sp_game with the same no-Source, crate-name-subject shape). None of these sibling markers carry a `// Source:` line or use an oracle-directory subject like `qcommon`. Rewriting only this file to the validator's proposed 3-line form with subject `qcommon` and a `// Source: oracle/codemp/qcommon/` line would make it the sole outlier in an otherwise perfectly consistent set — this reads as an intentional crate-level convention (whole-crate placeholder, not a single Raven identifier with a citable source line), not a per-file formatting defect. Fixing it unilaterally in isolation from its siblings would be a systemic style change outside this file's scope, so it is left untouched rather than apply a fix that contradicts the codebase's own established pattern.

### crates/mp/engine/server/src/lib.rs
- L1: module mp_engine_server
  - Validator claim does not match the file. Actual line 1 is `//! \`mp_engine_server\` crate. //TODO: Port module mp_engine_server` — subject is already `mp_engine_server` (not `server` as claimed), and there is no `// Source:` line to normalize. This exact one-line, no-Source-line form is the established house convention for crate-root module docs, verified identical across all sibling crate lib.rs files (e.g. crates/mp/engine/qcommon/src/lib.rs, crates/mp/engine/client/src/lib.rs, crates/sp/engine/server/src/lib.rs, crates/mp/bg/src/lib.rs). Fixing/normalizing it as described would actually break consistency with the rest of the codebase, so no edit was made.


## TODO: Port — LEGIT (91)

### crates/abi-transport/src/generic/engine.rs
- L78: RunStatic per-call handler surface

### crates/cgame/src/lib.rs
- L8: cgame live entrypoint exports (vmMain match, SEAM-D10)

### crates/jagame/src/lib.rs
- L130: GI_Init + gameinfo_import_t wiring

### crates/mp/app/src/main.rs
- L8: Com_Frame dedicated loop wiring
- L30: <macOS module suffix> (LOAD-Q1) — this dev-glue value is not it
- L77: SV_InitGameVM GAME_INIT args (svs.time, Com_Milliseconds)
- L92: the dedicated OS loop (sleep/console_poll/net_poll/com_frame)

### crates/mp/bg/src/lib.rs
- L3: module mp_bg (only the types needed by the game-code migration

### crates/mp/bg/src/local/mod.rs
- L3: module mp_bg::local — subsystem dir only; porters add flat

### crates/mp/bg/src/siege/mod.rs
- L3: module mp_bg::siege — subsystem dir only; porters add flat

### crates/mp/bg/src/vehicles/mod.rs
- L3: module mp_bg::vehicles — subsystem dir only; porters add flat

### crates/mp/bg/src/weapons/mod.rs
- L3: module mp_bg::weapons — subsystem dir only; porters add flat

### crates/mp/cgame/src/lib.rs
- L3: module mp_cgame

### crates/mp/engine-select/src/lib.rs
- L17: wasm32 outbound backend type (SEAM-Q11 — concrete type/file open)

### crates/mp/engine/botlib/src/lib.rs
- L1: module mp_engine_botlib

### crates/mp/engine/client/src/client_host.rs
- L10: Client fields (cl, clc, cls, keys, console, screen)
- L20: SoundSystem fields (channels, dma, listener, knownSfx)

### crates/mp/engine/client/src/fx/seffect_template.rs
- L24: CPrimitiveTemplate

### crates/mp/engine/client/src/lib.rs
- L1: module mp_engine_client

### crates/mp/engine/core/src/engine.rs
- L39: ZeroValid for Engine

### crates/mp/engine/core/src/lifecycle.rs
- L43: CPUSTRING/__DATE__ banner fields
- L47: Com_InitPushEvent — step 2
- L50: Com_ParseCommandLine — step 4
- L55: Com_StartupVariable/Rand_Init/CL_InitKeyCommands — steps 8-11
- L58: Com_InitJournaling + config execs + cvar block — steps 13-29
- L62: SV_Init + the dedicated/client-init tail — steps 31-39
- L126: Sys_Error console teardown + IN_Shutdown (client-shell slice)

### crates/mp/engine/core/src/sv_init_game_progs.rs
- L43: SV_InitGameProgs ctx injection (&mut Engine.sv into the game slot)

### crates/mp/engine/ghoul2/src/lib.rs
- L1: module mp_engine_ghoul2

### crates/mp/engine/icarus/src/interface/interface_export_s.rs
- L157: CSequencer
- L159: CTaskManager

### crates/mp/engine/icarus/src/lib.rs
- L1: module mp_engine_icarus

### crates/mp/engine/qcommon/src/cm/clip_map_t.rs
- L74: CCMLandScape

### crates/mp/engine/qcommon/src/collision_world.rs
- L11: CollisionWorld fields (cmg + SubBSP + trace counters)

### crates/mp/engine/qcommon/src/common/boot_stubs.rs
- L10: Cvar_Init
- L16: Cbuf_Init
- L22: Cmd_Init
- L28: FS_InitFilesystem

### crates/mp/engine/qcommon/src/common/common.rs
- L41: Common cvars/cmd/cbuf/fs/net sub-structs + com_printf print state
- L54: Com_Printf rd_buffer redirect + logfile + console routing

### crates/mp/engine/qcommon/src/timing/timing_c.rs
- L12: timing_c::start, timing_c::end, timing_c::reset

### crates/mp/engine/rmg/src/lib.rs
- L7: CRMManager (C++ track)

### crates/mp/engine/server/src/server_host.rs
- L134: SV_GameSystemCalls exhaustive dispatch
- L186: SV_InitGameProgs ctx injection (&mut Engine.sv)

### crates/mp/engine/wasm-host/Cargo.toml
- L15: wasmtime wiring — docs/architecture/module-loading.md LOAD-D10

### crates/mp/game/src/bg_vehicleLoad.rs
- L303: BG_SetSharedVehicleFunctions

### crates/mp/renderer/src/lib.rs
- L2: renderer subsystem (rd-vanilla logic; types already ported)

### crates/mp/renderer/src/tr_landscape/ctrland_scape.rs
- L26: CCMLandScape

### crates/mp/renderer/src/tr_landscape/ctrpatch.rs
- L17: CCMLandScape
- L21: CCMPatch

### crates/mp/renderer/src/tr_local/crenderable_surface.rs
- L15: CBoneCache

### crates/native/containers/src/lib.rs
- L2: Ratl/Ravl/Rufl/Ragl container templates

### crates/native/platform/src/module_loader/loader.rs
- L35: Sys_UnpackDLL — pure-server pk3 unpack pre-step, deferred (LOAD-D7)
- L75: NDEBUG in-loader fatal (LOAD-Q13 mechanism)

### crates/native/platform/src/module_loader/naming.rs
- L14: ModuleNaming macOS suffix

### crates/sp/abi/src/game/public/game_import_t.rs
- L32: Printf variadic args
- L41: Error variadic args
- L120: SendServerCommand variadic args
- L225: CGhoul2Info
- L454: IGhoul2InfoArray
- L497: CRagDollUpdateParams

### crates/sp/abi/src/ui/public/uiimport_t.rs
- L26: Printf variadic args
- L31: Error variadic args

### crates/sp/app/src/main.rs
- L1: module sp_app

### crates/sp/bg/src/lib.rs
- L1: module sp_bg

### crates/sp/cgame/src/lib.rs
- L1: module sp_cgame

### crates/sp/engine/client/src/lib.rs
- L1: module sp_engine_client

### crates/sp/engine/ghoul2/src/lib.rs
- L1: module sp_engine_ghoul2

### crates/sp/engine/icarus/src/lib.rs
- L1: module sp_engine_icarus

### crates/sp/engine/qcommon/src/cm/clip_map_t.rs
- L73: CCMLandScape

### crates/sp/engine/qcommon/src/lib.rs
- L1: module sp_engine_qcommon

### crates/sp/engine/qcommon/src/timing/timing_c.rs
- L12: timing_c::start, timing_c::end, timing_c::reset

### crates/sp/engine/rmg/src/lib.rs
- L7: CRMManager (C++ track)

### crates/sp/engine/server/src/lib.rs
- L1: module sp_engine_server

### crates/sp/game/src/gi.rs
- L24: gi::* outbound-call wrappers (one per game_import_t member)

### crates/sp/game/src/lib.rs
- L3: module sp_game (dependency types ported first; SP places

### crates/sp/game/src/world/game_context.rs
- L37: SP export logic fns taking GameContext (per-export, logic-port)

### crates/sp/game/src/world/game_world.rs
- L28: GameWorld::zeroed (SP) — native_platform::zeroed_box for entities/level

### crates/sp/renderer/src/lib.rs
- L1: module sp_renderer

### crates/sp/renderer/src/tr_landscape/ctrland_scape.rs
- L23: CCMLandScape

### crates/sp/renderer/src/tr_landscape/ctrpatch.rs
- L17: CCMLandScape
- L21: CCMPatch

### crates/sp/renderer/src/tr_local/crenderable_surface.rs
- L15: CBoneCache

### crates/sp/ui/src/gameinfo/gameinfo_import_t.rs
- L26: Printf variadic args

### crates/sp/ui/src/lib.rs
- L1: module sp_ui

### crates/sp/uishared/src/lib.rs
- L1: module sp_uishared

### crates/sp/uishared/src/shared/display_context_def_t.rs
- L138: CGhoul2Info
- L147: CGhoul2Info
- L177: CGhoul2Info
- L199: CGhoul2Info

### crates/ui/src/lib.rs
- L8: ui live entrypoint exports (vmMain match, SEAM-D10)


## TODO-other (51)

### crates/mp/engine/core/src/lifecycle.rs
- L33 [LEGIT]: /// carry `//TODO: Port` markers in step order so the transcript diff (DEC-09.2)

### crates/mp/game/src/FighterNPC.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Atst.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Droid.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Grenadier.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Howler.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Mark2.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Stormtrooper.rs
- L642 [LEGIT]: // TODO: Play sleeping shuffle animation (Raven comment).
- L943 [LEGIT]: // TODO: Play a sound along the lines of, "Huh? What was that?" (Raven comment).

### crates/mp/game/src/NPC_combat.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_move.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_sounds.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_spawn.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_stats.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/SpeederNPC.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/ai_main.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/bg_panimate.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/bg_saga.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_ICARUScb.rs
- L14 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_active.rs
- L3561 [?]: //TODO: Use

### crates/mp/game/src/g_arenas.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_exphysics.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_misc.rs
- L3379 [LEGIT]: //TODO: Find the target and set our angles to that direction

### crates/mp/game/src/g_nav.rs
- L950 [LEGIT]: // TODO: Handle all ents

### crates/mp/game/src/g_session.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_target.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_turret.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_vehicleTurret.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_vehicles.rs
- L506 [LEGIT]: // TODO: Set pilot should do all this stuff....
- L725 [LEGIT]: // TODO: Setup a default escape for every vehicle type.
- L1346 [LEGIT]: // MP: TODO MP Shift Sound Playback (no playback in MP)

### crates/mp/game/src/g_weapon.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/game_globals.rs
- L7 [LEGIT]: //! `//TODO: Port <type>` marker — the porter fills the real type when
- L925 [?]: // it"); shapes are exactly what the `g_log.md` packet's TODO comments

### crates/mp/game/src/npc_c.rs
- L1412 [LEGIT]: // TODO: Add vehicle behaviors here.

### crates/mp/game/src/prelude.rs
- L10 [LEGIT]: //! No new behavior lives here — only re-exports. The `//TODO: Port` markers for

### crates/mp/game/src/q_shared.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/tri_coll_test.rs
- L12 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/w_saber.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/sp/abi/src/ui/syscalls/UI_ATAN2.rs
- L10 [LEGIT]: /// TODO: SP transport evidence for float ABI is still missing; this follows MP float syscall pattern.

### crates/sp/abi/src/ui/syscalls/UI_CEIL.rs
- L11 [LEGIT]: /// TODO: SP transport evidence for float ABI is still missing; this follows MP float syscall pattern.

### crates/sp/abi/src/ui/syscalls/UI_CM_LERPTAG.rs
- L15 [LEGIT]: /// TODO: SP `UI_CM_LERPTAG` has no explicit engine switch case in

### crates/sp/abi/src/ui/syscalls/UI_COS.rs
- L13 [LEGIT]: /// TODO: SP transport evidence for float ABI is still missing; this mirrors MP float syscall pattern.

### crates/sp/abi/src/ui/syscalls/UI_FLOOR.rs
- L13 [LEGIT]: /// TODO: SP `oracle/code/client/cl_ui.cpp` does not include a `UI_FLOOR` case in its visible switch

### crates/sp/abi/src/ui/syscalls/UI_FS_FCLOSEFILE.rs
- L12 [LEGIT]: /// TODO: using MP transport parity until SP table entry is confirmed.

### crates/sp/abi/src/ui/syscalls/UI_FS_READ.rs
- L17 [LEGIT]: /// TODO: SP `oracle/code/client/cl_ui.cpp` has no `UI_FS_READ` case in this branch.

### crates/sp/abi/src/ui/syscalls/UI_FS_WRITE.rs
- L17 [LEGIT]: /// TODO: SP `oracle/code/client/cl_ui.cpp` has no `UI_FS_WRITE` case in this branch.

### crates/sp/abi/src/ui/syscalls/UI_SIN.rs
- L13 [LEGIT]: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_SQRT.rs
- L13 [LEGIT]: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_STRNCPY.rs
- L16 [LEGIT]: /// TODO: SP-side `UI_STRNCPY` transport is known only via fallback (`TRAP_STRNCPY`) evidence.

### crates/sp/abi/src/ui/syscalls/UI_S_STARTLOCALSOUND.rs
- L15 [LEGIT]: /// TODO: SP transport path does not provide a direct `UI_S_STARTLOCALSOUND` case in `oracle/code/client/cl_ui.cpp`.


## NOTE (486)

### crates/mp/game/src/FighterNPC.rs
- L1342: (untopiced)
- L1397: bg-anim-dispatch

### crates/mp/game/src/NPC_AI_Atst.rs
- L118: ai-context
- L144: ai-context
- L180: ai-context
- L294: ai-context
- L320: ai-context
- L342: ai-context

### crates/mp/game/src/NPC_AI_Droid.rs
- L48: globals-access

### crates/mp/game/src/NPC_AI_GalakMech.rs
- L724: unresolved-callee-type

### crates/mp/game/src/NPC_AI_Jedi.rs
- L6: (untopiced)
- L1201: jediSpeechDebounceTime
- L1939: jediSpeechDebounceTime
- L2337: BG_AnimLength
- L4639: jediSpeechDebounceTime
- L5735: jediSpeechDebounceTime

### crates/mp/game/src/NPC_AI_Mark1.rs
- L6: (untopiced)
- L68: ambient-state
- L111: ambient-state
- L161: ambient-state
- L182: ambient-state
- L277: ambient-state
- L353: variadic-c-abi
- L411: client-cast
- L491: ambient-state
- L631: ambient-state
- L656: ambient-state
- L797: ambient-state
- L868: ambient-state
- L974: ambient-state
- L1007: ambient-state
- L1119: ambient-state
- L1154: ambient-state

### crates/mp/game/src/NPC_AI_Rancor.rs
- L6: (untopiced)

### crates/mp/game/src/NPC_AI_Remote.rs
- L316: static-vec3-locals

### crates/mp/game/src/NPC_AI_Sniper.rs
- L6: (untopiced)

### crates/mp/game/src/NPC_AI_Wampa.rs
- L13: (untopiced)

### crates/mp/game/src/NPC_behavior.rs
- L1133: goal-invariant

### crates/mp/game/src/NPC_combat.rs
- L1372: varargs-seam
- L1744: SVF_IGNORE_ENEMIES

### crates/mp/game/src/NPC_misc.rs
- L35: varargs
- L78: varargs

### crates/mp/game/src/NPC_reactions.rs
- L953: va-formatting

### crates/mp/game/src/NPC_spawn.rs
- L51: packet-contract
- L479: weaponData

### crates/mp/game/src/NPC_stats.rs
- L2075: unported-fn
- L2128: unported-global

### crates/mp/game/src/NPC_utils.rs
- L31: (untopiced)

### crates/mp/game/src/SpeederNPC.rs
- L93: vtable-access
- L171: trap-access
- L340: vtable-access

### crates/mp/game/src/ai_main.rs
- L191: StateDescriptions
- L398: seam-threading
- L565: ACTION_*/ANGLE2SHORT
- L677: SHORT2ANGLE/ACTION_*
- L791: botstates/PRT_FATAL/SHORT2ANGLE
- L911: botstates/PRT_FATAL
- L1274: bot_pvstype
- L1424: FORCEJUMP_INSTANTMETHOD
- L1504: forceJumpStrength/FORCEJUMP_INSTANTMETHOD
- L1606: FORCEJUMP_INSTANTMETHOD
- L2105: botstates/ENEMY_FORGET_MS
- L2545: MAX_CHICKENWUSS_TIME/BOT_RUN_HEALTH
- L2658: ENEMY_FORGET_MS
- L2797: BASE_GUARD_DISTANCE
- L2832: BASE_GETENEMYFLAG_DISTANCE
- L6520: preproc
- L7364: gWPArray-oob §S19
- L7416: gWPArray-oob §S19
- L7560: §S19
- L7577: mLen-quirk
- L8135: fn-statics
- L8256: bot-cvars

### crates/mp/game/src/ai_wpnav.rs
- L2: (untopiced)
- L22: cvar-placement
- L23: missing-const
- L332: cvar-placement
- L1556: cvar-placement
- L1957: cvar-placement
- L2007: missing-global
- L2513: bg-0143
- L2606: bg-temp-alloc
- L3212: missing-const
- L3227: cvar-placement
- L3258: cvar-placement
- L3632: missing-const

### crates/mp/game/src/bg_misc.rs
- L260: ctx-cascade
- L1804: qagame-cfg

### crates/mp/game/src/bg_panimate.rs
- L14: own-file-static-no-world-handle

### crates/mp/game/src/bg_pmove.rs
- L12: (untopiced)
- L229: (untopiced)
- L1165: meditate-timer-quirk
- L1773: byte-select-shift
- L4838: g_gametype

### crates/mp/game/src/bg_saber.rs
- L528: saber-move-name-coverage
- L1622: faithful-oracle-bug
- L1997: scope
- L2940: client-saber-field

### crates/mp/game/src/bg_saberLoad.rs
- L331: missing-symbol
- L739: missing-const
- L902: missing-symbols
- L1055: missing-trait-method
- L2372: rng-cascade
- L2395: vec-as-fixed-buffer

### crates/mp/game/src/bg_slidemove.rs
- L65: bg-tier-gap
- L304: fn-ptr-skip
- L638: bg-tier-gap

### crates/mp/game/src/bg_vehicleLoad.rs
- L906: traps-cascade
- L941: traps-cascade

### crates/mp/game/src/g_ICARUScb.rs
- L72: variadic-c-abi
- L758: die-dispatch-invoke
- L830: vehicle-eject
- L895: unported-global
- L1199: unported-global
- L1269: unported-global
- L1570: client-still-void
- L2041: unported-global
- L2068: unported-global
- L2165: unported-global
- L2254: unported-global
- L2285: unported-global
- L2426: client-still-void
- L2583: unported-global
- L2966: dropped-warnings
- L3793: unported-consts
- L3865: unported-consts
- L4083: unported-consts

### crates/mp/game/src/g_client.rs
- L26: (untopiced)
- L284: fn-ptr-store-shape-mismatch
- L320: fn-ptr-store-shape-mismatch
- L519: traps-va-plus-fn-ptr-store
- L615: dead-code
- L624: fn-ptr-store-shape-mismatch
- L1210: fn-ptr-store-plus-traps
- L1353: traps-plus-body-queue
- L1638: g2-ghoul2-traps
- L1784: multi-global-trap-va-cluster
- L1834: static-buffer
- L1947: multi-global-trap-va-cluster
- L2216: va-varargs-plus-rng
- L2303: g2-trap-plus-bg
- L2309: dead-code
- L2539: mega-fn-globals-traps-bg-fnptr
- L3364: multi-global-trap-g2
- L3647: vehicle-pointer-overlay

### crates/mp/game/src/g_cmds.rs
- L1: (untopiced)
- L12: <topic>
- L13: (untopiced)
- L18: (untopiced)
- L98: raw-ptr-skeleton-no-world-handle
- L209: raw-ptr-skeleton-no-world-handle
- L215: G_GetStringEdString
- L256: static-scratch-buffer-vs-raw-ptr-return
- L337: raw-ptr-skeleton-no-world-handle
- L410: raw-ptr-skeleton-no-world-handle
- L625: raw-ptr-skeleton-no-world-handle
- L658: raw-ptr-skeleton-no-world-handle
- L689: raw-ptr-skeleton-no-world-handle
- L761: raw-ptr-skeleton-no-world-handle
- L816: raw-ptr-skeleton-no-world-handle
- L935: raw-ptr-skeleton-no-world-handle
- L1111: raw-ptr-skeleton-no-world-handle
- L1218: G_LogPrintf
- L1260: raw-ptr-skeleton-no-world-handle
- L1470: raw-ptr-skeleton-no-world-handle
- L1502: raw-ptr-skeleton-no-world-handle
- L1602: raw-ptr-skeleton-no-world-handle
- L1764: raw-ptr-skeleton-no-world-handle
- L1890: raw-ptr-skeleton-no-world-handle
- L1954: raw-ptr-skeleton-no-world-handle
- L1972: bgSiegeClasses
- L2112: raw-ptr-skeleton-no-world-handle
- L2134: G_Error
- L2183: raw-ptr-skeleton-no-world-handle
- L2270: raw-ptr-skeleton-no-world-handle
- L2384: G_Printf
- L2440: raw-ptr-skeleton-no-world-handle
- L2500: raw-ptr-skeleton-no-world-handle
- L2587: raw-ptr-skeleton-no-world-handle
- L2636: raw-ptr-skeleton-no-world-handle
- L2754: raw-ptr-skeleton-no-world-handle
- L3135: raw-ptr-skeleton-no-world-handle
- L3247: raw-ptr-skeleton-no-world-handle
- L3589: raw-ptr-skeleton-no-world-handle
- L3709: raw-ptr-skeleton-no-world-handle
- L3783: raw-ptr-skeleton-no-world-handle
- L3800: bg_itemlist
- L4036: raw-ptr-skeleton-no-world-handle
- L4158: bgSiegeClasses
- L4250: raw-ptr-skeleton-no-world-handle
- L4526: raw-ptr-skeleton-no-world-handle
- L4560: animTable/saberMoveData
- L4573: raw-ptr-skeleton-no-world-handle
- L4600: animTable
- L4727: raw-ptr-skeleton-no-world-handle
- L4866: FOFS-targetname
- L4962: debug-build-gated-cmds

### crates/mp/game/src/g_combat.rs
- L1830: static-death-anim-counter
- L3033: boltPoint-outparam
- L3232: bg-traps-handle
- L3682: point-null
- L3715: point-null
- L4343: point-null
- L4560: ctx-and-dir
- L5698: dir/point-null
- L5768: point-null
- L5784: ctx-and-dir

### crates/mp/game/src/g_items.rs
- L12: (untopiced)
- L150: raw-ptr-skeleton-no-world-handle
- L207: raw-ptr-skeleton-no-world-handle
- L228: raw-ptr-skeleton-no-world-handle
- L253: raw-ptr-skeleton-no-world-handle
- L278: raw-ptr-skeleton-no-world-handle
- L305: raw-ptr-skeleton-no-world-handle
- L361: raw-ptr-skeleton-no-world-handle
- L388: raw-ptr-skeleton-no-world-handle
- L422: raw-ptr-skeleton-no-world-handle
- L603: vec3-outparam-seam
- L744: raw-ptr-skeleton-no-world-handle
- L786: raw-ptr-skeleton-no-world-handle
- L830: raw-ptr-skeleton-no-world-handle
- L968: raw-ptr-skeleton-no-world-handle
- L1041: missing-const
- L1051: raw-ptr-skeleton-no-world-handle
- L1283: raw-ptr-skeleton-no-world-handle
- L1342: raw-ptr-skeleton-no-world-handle
- L1388: raw-ptr-skeleton-no-world-handle
- L1468: raw-ptr-skeleton-no-world-handle
- L1568: raw-ptr-skeleton-no-world-handle
- L1608: raw-ptr-skeleton-no-world-handle
- L1651: raw-ptr-skeleton-no-world-handle
- L1696: raw-ptr-skeleton-no-world-handle
- L1728: raw-ptr-skeleton-no-world-handle
- L1733: unported-global
- L1777: missing-const
- L1799: vec3-outparam-seam
- L1871: raw-ptr-skeleton-no-world-handle
- L1902: raw-ptr-skeleton-no-world-handle
- L1978: raw-ptr-skeleton-no-world-handle
- L2002: raw-ptr-skeleton-no-world-handle
- L2102: raw-ptr-skeleton-no-world-handle
- L2148: raw-ptr-skeleton-no-world-handle
- L2222: vec3-outparam-seam
- L2401: raw-ptr-skeleton-no-world-handle
- L2504: raw-ptr-skeleton-no-world-handle
- L2706: raw-ptr-skeleton-no-world-handle
- L2756: vec3-outparam-seam
- L2861: raw-ptr-skeleton-no-world-handle
- L2885: missing-const
- L2888: raw-ptr-skeleton-no-world-handle
- L2909: raw-ptr-skeleton-no-world-handle
- L2970: raw-ptr-skeleton-no-world-handle
- L3041: missing-const
- L3107: raw-ptr-skeleton-no-world-handle
- L3188: raw-ptr-skeleton-no-world-handle
- L3226: raw-ptr-skeleton-no-world-handle
- L3554: vec3-outparam-seam
- L3641: missing-const
- L3645: vec3-outparam-seam
- L3685: raw-ptr-skeleton-no-world-handle
- L3850: raw-ptr-skeleton-no-world-handle
- L3884: raw-ptr-skeleton-no-world-handle
- L3907: raw-ptr-skeleton-no-world-handle
- L3923: raw-ptr-skeleton-no-world-handle
- L3952: raw-ptr-skeleton-no-world-handle
- L3966: raw-ptr-skeleton-no-world-handle
- L4027: raw-ptr-skeleton-no-world-handle
- L4114: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_main.rs
- L11: <topic>
- L12: (untopiced)
- L70: variadic-c-abi
- L83: variadic-c-abi
- L95: variadic-c-abi
- L105: variadic-c-abi
- L503: variadic-c-abi
- L513: ctx-free-boundary
- L528: variadic-c-abi
- L537: ctx-free-boundary
- L872: unported-dep
- L1129: unported-dep
- L1467: unported-dep
- L1545: unported-const
- L1662: unported-dep
- L1751: variadic-c-abi
- L1761: variadic-c-abi
- L1793: unported-const
- L1866: unported-const
- L2240: unported-dep
- L2849: raw-ptr-skeleton-no-world-handle
- L2884: raw-ptr-skeleton-no-world-handle
- L3047: raw-ptr-skeleton-no-world-handle
- L3074: raw-ptr-skeleton-no-world-handle
- L3134: raw-ptr-skeleton-no-world-handle
- L3179: raw-ptr-skeleton-no-world-handle
- L3275: raw-ptr-skeleton-no-world-handle
- L3324: raw-ptr-skeleton-no-world-handle
- L3364: raw-ptr-skeleton-no-world-handle
- L3454: raw-ptr-skeleton-no-world-handle
- L3992: raw-ptr-skeleton-no-world-handle
- L4011: static-scratch-buffer

### crates/mp/game/src/g_misc.rs
- L463: raw-ptr-skeleton-no-world-handle
- L579: unported-const
- L784: raw-ptr-skeleton-no-world-handle
- L929: unported-const
- L1062: control-flow
- L1426: raw-ptr-skeleton-no-world-handle
- L1473: unported-const
- L2003: seam-threading
- L2134: seam-threading
- L2148: raw-ptr-skeleton-no-world-handle
- L2609: raw-ptr-skeleton-no-world-handle
- L2641: unported-const
- L2688: raw-ptr-skeleton-no-world-handle
- L2745: unported-const
- L2861: raw-ptr-skeleton-no-world-handle
- L2893: raw-ptr-skeleton-no-world-handle
- L2989: raw-ptr-skeleton-no-world-handle
- L3002: raw-ptr-skeleton-no-world-handle
- L3344: bg-dep
- L3358: bg-dep
- L3423: unported-const
- L3486: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_missile.rs
- L1220: gclient_t

### crates/mp/game/src/g_mover.rs
- L18: (untopiced)
- L579: bg-boundary

### crates/mp/game/src/g_nav.rs
- L127: missing-const
- L143: dead-oracle-code
- L2187: ifdef-elided

### crates/mp/game/src/g_navnew.rs
- L29: (untopiced)

### crates/mp/game/src/g_object.rs
- L286: signature-mismatch

### crates/mp/game/src/g_saga.rs
- L109: unported-const
- L119: unported-const
- L131: bg-boundary
- L141: missing-fields
- L459: missing-field
- L579: bg-boundary
- L593: missing-field
- L662: bg-boundary
- L669: missing-field
- L825: unported-const
- L841: unported-const
- L872: unported-const
- L905: raw-ptr-skeleton-no-world-handle
- L923: unported-const
- L996: bg-boundary
- L1005: missing-fields
- L1124: bg-boundary
- L1286: bg-boundary
- L1359: unported-const
- L1368: unported-const
- L1489: unported-const
- L1536: bg-boundary
- L1635: bg-boundary
- L1707: unported-const
- L1727: bg-boundary
- L1785: bg-boundary
- L1864: bg-boundary
- L1917: bg-boundary
- L1993: unported-const
- L2003: unported-const
- L2249: unported-const
- L2316: bg-boundary
- L2588: bg-boundary
- L2598: unported-const

### crates/mp/game/src/g_spawn.rs
- L31: (untopiced)
- L1187: bg-panimate-method
- L1368: string-formatting

### crates/mp/game/src/g_target.rs
- L1044: EntityId-deref

### crates/mp/game/src/g_team.rs
- L9: (untopiced)

### crates/mp/game/src/g_trigger.rs
- L2124: variadic-c-abi

### crates/mp/game/src/g_turret_G2.rs
- L501: damage-flags

### crates/mp/game/src/g_utils.rs
- L17: (untopiced)
- L743: (untopiced)
- L2017: fn-pointer-dispatch
- L2345: unported-callee

### crates/mp/game/src/g_vehicles.rs
- L6: (untopiced)
- L284: (untopiced)
- L422: m_vOrientation
- L698: G_Damage-null-dir
- L937: bg-channel
- L952: bg-boundary
- L1239: PM_BGEntForNum
- L1869: bg-boundary
- L2024: bg-boundary
- L2064: G_Damage-null-dir/point
- L2188: bg-boundary

### crates/mp/game/src/g_weapon.rs
- L196: seam-threading
- L254: seam-threading
- L259: file-static-globals
- L508: seam-threading
- L532: seam-threading
- L741: bg-dep
- L769: seam-threading
- L774: file-static-globals
- L1100: seam-threading
- L1140: seam-threading
- L1240: seam-threading
- L1271: seam-threading
- L1320: seam-threading
- L1345: seam-threading
- L1384: seam-threading
- L1551: seam-threading
- L1581: seam-threading
- L1679: seam-threading
- L1735: seam-threading
- L1785: seam-threading
- L1867: rng-threading
- L1920: seam-threading
- L1959: seam-threading
- L2143: seam-threading
- L2249: seam-threading
- L2333: seam-threading
- L2430: seam-threading
- L2435: file-static-globals
- L2448: seam-threading
- L2633: out-param-vec3
- L2638: seam-threading
- L2691: seam-threading
- L2726: seam-threading
- L2760: seam-threading
- L2822: seam-threading
- L2878: seam-threading
- L2971: seam-threading
- L3048: seam-threading
- L3146: seam-threading
- L3276: seam-threading
- L3370: seam-threading
- L3448: seam-threading
- L3494: seam-threading
- L3593: seam-threading
- L3859: seam-threading
- L3911: seam-threading
- L4031: seam-threading
- L4303: seam-threading
- L4363: seam-threading
- L4384: seam-threading
- L4626: bg-dep
- L4671: seam-threading
- L4787: out-param-vec3
- L4794: parked-dep
- L4804: unported-type
- L4914: parked-dep

### crates/mp/game/src/npc_c.rs
- L10: (untopiced)
- L312: raw-ptr-skeleton-no-world-handle
- L335: raw-ptr-skeleton-no-world-handle
- L471: raw-ptr-skeleton-no-world-handle
- L644: raw-ptr-skeleton-no-world-handle
- L660: raw-ptr-skeleton-no-world-handle
- L675: raw-ptr-skeleton-no-world-handle
- L689: raw-ptr-skeleton-no-world-handle
- L704: raw-ptr-skeleton-no-world-handle
- L751: raw-ptr-skeleton-no-world-handle
- L812: raw-ptr-skeleton-no-world-handle
- L878: raw-ptr-skeleton-no-world-handle
- L896: raw-ptr-skeleton-no-world-handle
- L941: raw-ptr-skeleton-no-world-handle
- L1263: raw-ptr-skeleton-no-world-handle
- L1436: raw-ptr-skeleton-no-world-handle
- L1600: raw-ptr-skeleton-no-world-handle
- L1689: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/q_shared.rs
- L321: variadic-c-abi
- L351: variadic-c-abi
- L370: variadic-c-abi
- L1153: variadic-c-abi
- L1183: variadic-c-abi

### crates/mp/game/src/w_force.rs
- L268: bot-forcepowers
- L3304: unported-global-and-vehicle-vtable
- L4274: unported-global-table
- L5499: unported-global-table

### crates/mp/game/src/w_saber.rs
- L240: bg-boundary
- L923: trailingLegsAngles
- L2342: PM_SaberBounceForAttack
- L2403: PM_SaberBounceForAttack
- L2837: faces
- L3535: BG_AnimLength
- L3944: missing-global-field
- L4307: missing-global-field
- L4398: saberClashPos/saberClashNorm
- L4402: BG_BrokenParryForAttack/BG_InKnockDownOnGround
- L7537: g2-seam-argshape
- L7556: g2-seam-argshape
- L8800: kickEnd-null
- L8970: BG_AnimLength-shape
- L9354: kickEnd-null
- L10416: array-decay
- L10440: array-decay

