# Marker inventory — PORT-NOTE / TODO (regenerated 2026-07-05, post-audit)

Totals: 795 markers — 184 `TODO: Port`,
554 `PORT-NOTE`, 57 other TODO forms.

Every TODO comment was audited by the 2026-07-05 validation run (342 markers
judged; 102 stale/malformed fixed in commit ac144a74). Verdicts:

- **LEGIT** — subject genuinely unported; marker accurate. (158)
- **COMPLEX** — subject (or its blockers) now ported, but resolving the marker
  needs non-mechanical work: authoring missing logic, threading `static mut`
  globals through `GameWorld`, or cross-tier naming. Listed first — these are
  the actionable findings. (23; weaponData/Pickup_Item done in 43d83f4)
- **STALE-escalated** — mechanical fix known but crosses files; left in place. (3)

PORT-NOTE section is a plain re-grep (not audited).

## TODO: Port — COMPLEX (23)

### crates/mp/engine/server/src/server_host.rs
- L16: Server fields (sv: server_t incl. the SS_DEAD liveness state,
  - server_t and serverStatic_t are already ported (crates/mp/engine/server/src/server/server_t.rs, server_static_t.rs) but the marker also covers bot/master/savegame state that is not ported, and Server has no constructors anywhere yet — wiring this in is a design decision (how sv/svs/bot compose, whether Server stays a plain aggregate) plus multiple unported sub-fields, not a single mechanical retype.

### crates/mp/game/src/ai_main.rs
- L907: bot_settings_s  (C: `struct bot_settings_s *`)
  - bot_settings_t IS already ported (crates/mp/game/src/level/bot_settings.rs, re-exported via crates/mp/game/src/botai/bot_settings_s.rs) and even imported/used in this same file (line 89, line 932's cast) — the `settings: *mut c_void` param is a stale overlay cast exactly like the saberFace_t exemplar. However BotAISetupClient is `pub fn` and its one caller in crates/mp/game/src/g_bot.rs:803 also does the matching `&mut settings as *mut bot_settings_t as *mut c_void` overlay cast, so the mechanical fix is not file-local (touches g_bot.rs too) — per the STALE/COMPLEX tie-break, reporting as COMPLEX. Marker text is also malformed (trailing '(C: `struct bot_settings_s *`)' junk, no '// Source:' line) but is moot since the subject is ported, not unported.

### crates/mp/game/src/g_init_game.rs
- L38: G_InitGame body (G_RegisterCvars, level wiring, back-pointers,
  - The individual building blocks this marker lists are now ported (G_RegisterCvars at g_main.rs:182, trap::LocateGameData at trap.rs:1400, G_SpawnEntitiesFromString at g_spawn.rs:1459), but g_init_game() is still the live-wired GAME_INIT entrypoint (world/game_context.rs:86) and is explicitly a 'Slice-0 minimal' stub that calls none of them. Assembling the real G_InitGame body (level/back-pointer wiring, trap_SV_RegisterSharedMemory, cvar registration order, entity-spawn sequencing per g_main.c:897-1015) is substantial new orchestration logic, not a mechanical retype/placeholder swap.

### crates/mp/game/src/g_items.rs
- L1963: bg_itemlist
  - bg_itemlist is ported (bg_itemlist.rs:26) and BG_CycleInven is ported (bg_misc.rs:907), so the stated blocker is gone, but the branch itself (oracle g_items.c:1471-1478: check `STAT_HOLDABLE_ITEM`'s bg_itemlist entry is IT_HOLDABLE/HI_EWEB, then reset the stat and call BG_CycleInven) was never written — the `if` block here is empty. Completing it requires authoring new conditional logic, not a mechanical retype.

### crates/mp/game/src/g_mover.rs
- L2926: MAT_DRK_STONE, MAT_LT_STONE, MAT_GREY_STONE, MAT_SNOWY_ROCK
  - All four material_t values are ported (crates/mp/qshared/src/common/mp/gentity.rs) and already used elsewhere in this same file (CacheChunkEffects, line 2614), but the fix is not a small mechanical rename: the entire stone-chunk-spawning branch of funcBBrushPain (g_mover.c:2556-2588, ~20 lines of vector math, Q_irand, and a G_Chunks call) is missing and must be transcribed faithfully — real logic addition, not a retype.

### crates/mp/game/src/g_shutdown_game.rs
- L17: G_ShutdownGame body (G_SaveBanIP, fake-client + ghoul2 cleanup)
  - All referenced dependencies now exist (G_SaveBanIP, G_CleanAllFakeClients, BG_ClearAnimsets, trap::G2API_CleanGhoul2Models, trap::G2_HaveWeGhoul2Models), but the fix is a full function-body transcription — a MAX_GENTITIES loop over g_entities checking `ent.ghoul2`/`ent.client`, a nested MAX_SABERS loop over weaponGhoul2 — not a mechanical retype.

### crates/mp/game/src/game_globals.rs
- L661: navInfo_t
  - navInfo_t is ported (crates/mp/game/src/npc/nav_info_s.rs:12, with size/offset asserts), but the corresponding oracle global (frameNavInfo) is currently implemented as NPC_move.rs's own `static mut FRAME_NAV_INFO: navInfo_t` rather than threaded through GameWorld, in violation of porting-rules B3. Wiring the () placeholder to a real field requires refactoring NPC_move.rs's static-mut global and its several call sites (NPC_move.rs, g_nav.rs callers) to read/write ctx.world instead — not a file-local mechanical retype.
- L1027: gtimer_t **
  - gtimer_t is ported (crates/mp/game/src/g_timer.rs:27), but g_timerFreeList is currently its own `static mut` global in g_timer.rs (line 47) rather than a threaded GameWorld field, violating porting-rules B3/B4. Wiring the () placeholder requires removing that static mut and rethreading every g_timer.rs call site (TIMER_GetNew/TIMER_GetExisting/etc.) through ctx.world — not file-local.
- L1030: gtimer_t[ MAX_GTIMERS ]
  - Same situation as g_timerFreeList: gtimer_t is ported but g_timerPool is its own `static mut [gtimer_t; MAX_GTIMERS]` in g_timer.rs (line 40), not threaded via GameWorld. Fixing requires cross-file rework of g_timer.rs's global state, not a local retype.
- L1033: gtimer_t *[ MAX_GENTITIES ]*
  - Same situation: g_timers is its own `static mut [*mut gtimer_t; MAX_GENTITIES]` in g_timer.rs (line 45), not a GameWorld field. Wiring the placeholder needs a cross-file refactor of g_timer.rs's static-mut globals into ctx.world, not a file-local fix.

### crates/mp/qshared/src/common/mp/gentity.rs
- L94: Vehicle_t
  - Marker itself correctly states Vehicle_t IS ported (crates/mp/bg/src/vehicles/vehicle_s.rs) but cannot be named here because gentity_t lives in mp_qshared, which sits below mp_bg in the tier graph (native < qshared < bg < game) and mp_qshared cannot depend on mp_bg. Resolving requires an abi-seam/tier refactor moving gentity_t (or restructuring the tier graph), not a file-local fix; also shared with the `client: *mut c_void` field on the same struct.
- L168: gNPC_t
  - gNPC_t is already ported at crates/mp/game/src/npc/g_npc_t.rs:35, but it lives in mp_game (above mp_qshared in the tier graph), so gentity_t (in mp_qshared) cannot name it without an abi-seam/tier refactor — the same tiering blocker as the Vehicle_t/client fields on this same struct.

### crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs
- L29: Vehicle_t
  - Same tiering blocker as gentity.rs: Vehicle_t is ported at crates/mp/bg/src/vehicles/vehicle_s.rs but sharedEntity_t lives in mp_qshared, which cannot depend on mp_bg. Requires an abi-seam/tier refactor, not a file-local mechanical fix.

### crates/sp/cgame/src/media/cgs_t.rs
- L47: clientInfo_t (cross-crate, sp_game -> sp_cgame not wired)
  - clientInfo_t is fully ported at sp_game::shared::client_info_t::clientInfo_t (confirmed: `pub struct clientInfo_t` at that path), exactly as the marker itself states. But resolving this in cgs_t.rs would mean replacing `OpaqueClientInfo_t = [u64; 62]` (a #[repr(C)] field inside cgs_t, size/offset-asserted at line 259+) with the real cross-crate type, which requires adding a new sp_cgame -> sp_game dependency edge (confirmed absent in crates/sp/cgame/Cargo.toml) — an architecture-layering change with effects beyond this file, not a mechanical local fix.

### crates/sp/engine/server/src/server_host.rs
- L30: server_t/serverStatic_t fields (SP Server island)
  - server_t and serverStatic_t ARE already ported as full #[repr(C)] types (crates/sp/engine/server/src/server/server_t.rs and .../server_static_t.rs), but wiring `sv`/`svs` fields of those types into the (non-repr(C), no size/offset asserts) `Server` struct is not a mechanical retype: it requires designing ownership/initialization for large arrays (server_t alone is ~397KB with a 1024-entry svEntity_t array), and downstream `sv_init_game_progs`/`sv_shutdown_game_progs` — both still todo!() stubs — would need real init/shutdown logic to populate them. Not file-local single-line fix; genuine design work.

### crates/sp/game/src/shared/weapon_info_s.rs
- L40: centity_t
  - centity_t IS ported (crates/sp/cgame/src/local/centity_s.rs, pub struct centity_t). But sp_game and sp_cgame are sibling tier-3 module crates with no dependency edge between them (docs/workspace-architecture.md); retyping this callback param to the real type would require adding an sp_game -> sp_cgame crate dependency, which the architecture forbids. The comment already documents this as a deliberate cross-tier opaque-pointer design, not an oversight.
- L46: centity_t
  - Same as line 40 — centity_t exists in sp_cgame but sp_game cannot depend on sp_cgame (same tier, no edge), so the opaque *mut c_void callback param is architecturally required, not a stale mechanical leftover.

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
  - Real struct gNPC_t is ported at crates/sp/game/src/npc/g_npc_t.rs (audited in this same shard), but it lives in sp_game (tier 3) while gentity_t lives in sp_qshared (tier 0); same cross-tier dependency block prevents retyping the NPC field.


## TODO: Port — STALE-escalated (3)

### crates/mp/engine/qcommon/src/lib.rs
- L1: module mp_engine_qcommon
  - Validator's claim does not hold. The current one-line form `//! \`mp_engine_qcommon\` crate. //TODO: Port module mp_engine_qcommon` (no separate Source line, subject = crate name) is the established, uniformly-replicated house convention for crate-level module docs: grep shows the identical pattern verbatim in 22 other crates' lib.rs files (mp_ui, mp_renderer, mp_uishared, mp_engine_icarus, mp_engine_ghoul2, mp_engine_server, mp_engine_botlib, mp_engine_client, mp_cgame, sp_ui, sp_renderer, sp_bg, sp_uishared, sp_engine_icarus, sp_engine_ghoul2, sp_engine_server, sp_engine_qcommon, sp_engine_client, sp_cgame, native_containers, plus multi-line variants in mp_bg/sp_game with the same no-Source, crate-name-subject shape). None of these 23 sibling markers carry a `// Source:` line or use an oracle-directory subject like `qcommon`. Rewriting only this file to the validator's proposed 3-line form with subject `qcommon` and a `// Source: oracle/oracle/codemp/qcommon/` line would make it the sole outlier in an otherwise perfectly consistent set — this reads as an intentional crate-level convention (whole-crate placeholder, not a single Raven identifier with a citable source line), not a per-file formatting defect. Fixing it unilaterally in isolation from its 23 siblings would be a systemic style change outside this file's scope, so I left it untouched rather than apply a fix that contradicts the codebase's own established pattern.

### crates/mp/engine/server/src/lib.rs
- L1: module mp_engine_server
  - Validator claim does not match the file. Actual line 1 is `//! `mp_engine_server` crate. //TODO: Port module mp_engine_server` — subject is already `mp_engine_server` (not `server` as claimed), and there is no `// Source:` line to normalize. This exact one-line, no-Source-line form is the established house convention for crate-root module docs, verified identical across all 22 sibling crate lib.rs files (e.g. crates/mp/engine/qcommon/src/lib.rs, crates/mp/engine/client/src/lib.rs, crates/sp/engine/server/src/lib.rs, crates/mp/bg/src/lib.rs). Git history (git log -p --follow on this file) shows the line unchanged since crate creation in 71ec41c7. Fixing/normalizing it as described would actually break consistency with the rest of the codebase, so no edit was made.

### crates/sp/abi/src/game/public/game_import_t.rs
- L435: CMiniHeap
  - Validator claim is false: CMiniHeap IS used by a field in this struct — G2API_CollisionDetect's G2VertSpace parameter (line 448, currently `*mut c_void`) is Raven's `CMiniHeap *G2VertSpace` (oracle/oracle/code/game/g_public.h:403-404). The marker correctly documents that placeholder. Retyping it to `*mut CMiniHeap` (ported at crates/sp/engine/qcommon/src/miniheap/cmini_heap.rs) requires adding sp_engine_qcommon as a dependency in crates/sp/abi/Cargo.toml, which is outside my assigned file — so I left the marker and field untouched rather than deleting it.


## TODO: Port — LEGIT (158)

### crates/abi-transport/src/generic/engine.rs
- L78: RunStatic per-call handler surface

### crates/cgame/src/lib.rs
- L8: cgame live entrypoint exports (vmMain match, SEAM-D10)

### crates/jagame/src/lib.rs
- L130: GI_Init + gameinfo_import_t wiring

### crates/jampgame/src/lib.rs
- L336: GetModuleAPI — contract is SEAM-Q7 (open)

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

### crates/mp/cgame/src/local/client_info_t.rs
- L57: ghoul2Weapons element type (CGhoul2Info_v*)
- L142: ghoul2Model (CGhoul2Info_v*)

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
- L43: ServerGame reborrow ctx (ServerGame concrete shape unpinned)

### crates/mp/engine/ghoul2/src/lib.rs
- L1: module mp_engine_ghoul2
- L3: SSkinGoreData

### crates/mp/engine/icarus/src/blockstream/cblock_stream.rs
- L16: FILE

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

### crates/mp/engine/qcommon/src/files/pack_t.rs
- L17: unzFile

### crates/mp/engine/qcommon/src/timing/timing_c.rs
- L12: timing_c::start, timing_c::end, timing_c::reset

### crates/mp/engine/rmg/src/lib.rs
- L7: CRMManager (C++ track)

### crates/mp/engine/server/src/server/sv_entity_s.rs
- L18: worldSector_s

### crates/mp/engine/server/src/server_host.rs
- L30: ServerGame concrete shape (alias vs wrapper — STATE-Q7)
- L49: SV_GameSystemCalls exhaustive dispatch

### crates/mp/game/src/FighterNPC.rs
- L103: BG_FighterUpdate traceFunc callback signature

### crates/mp/game/src/NPC_AI_Sniper.rs
- L744: Sniper_FireDecide miss loop body (VectorMA flrand(1.5,4)

### crates/mp/game/src/NPC_stats.rs
- L361: rank_t (unported return enum; C int-width)

### crates/mp/game/src/bg_vehicleLoad.rs
- L300: BG_SetSharedVehicleFunctions

### crates/mp/game/src/g_main.rs
- L189: G_RegisterCvars per-row trap_Cvar_Register loop
- L260: G_UpdateCvars per-row trap_Cvar_Update loop
- L3134: BG_GetTime level.time reach-through
- L3150: level.time reach-through

### crates/mp/game/src/g_misc.rs
- L99: bSet_e
- L2255: FX_STATE_OFF
- L2258: FX_STATE_ONE_SHOT
- L2261: FX_STATE_ONE_SHOT_LIMIT
- L2264: FX_STATE_CONTINUOUS

### crates/mp/game/src/g_trigger.rs
- L54: bSet_e
- L78: PUSH_CONSTANT
- L81: PUSH_LINEAR
- L84: PUSH_RELATIVE
- L87: PUSH_MULTIPLE
- L90: HYPERSPACE_TIME
- L93: HYPERSPACE_TELEPORT_FRAC
- L98: INITIAL_SUFFOCATION_DELAY

### crates/mp/game/src/g_turret.rs
- L44: CLASS_VEHICLE

### crates/mp/game/src/g_vehicles.rs
- L2793: RegisterAssets

### crates/mp/game/src/npc/g_npc_t.rs
- L75: rank_t

### crates/mp/game/src/q_shared.rs
- L1306: MAX_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_RemoveKey: oversize infostring")) once the const is ported.
- L1368: BIG_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_RemoveKey_Big: oversize infostring")) once the const is ported.
- L1445: MAX_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring")) once the const is ported.
- L1475: MAX_INFO_STRING (currently hardcoded 1024 — not yet a
- L1496: BIG_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring")) once the const is ported.
- L1526: BIG_INFO_STRING (currently hardcoded 8192 — not yet a

### crates/mp/game/src/world/game_context.rs
- L304: Dispatch<C> for GameContext (GAME_ICARUS_* commands)

### crates/mp/qshared/src/shared/cvar.rs
- L16: cvar_t

### crates/mp/renderer/src/lib.rs
- L2: renderer subsystem (rd-vanilla logic; types already ported)

### crates/mp/renderer/src/tr_landscape/ctrland_scape.rs
- L23: CCMLandScape

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
- L31: Printf variadic args
- L40: Error variadic args
- L50: cvar_t
- L124: SendServerCommand variadic args
- L229: CGhoul2Info
- L458: IGhoul2InfoArray
- L501: CRagDollUpdateParams
- L563: SSkinGoreData

### crates/sp/abi/src/ui/public/uiimport_t.rs
- L26: Printf variadic args
- L31: Error variadic args

### crates/sp/app/src/main.rs
- L1: module sp_app

### crates/sp/bg/src/lib.rs
- L1: module sp_bg

### crates/sp/cgame/src/lib.rs
- L1: module sp_cgame

### crates/sp/cgame/src/media/cg_media_t.rs
- L209: ffHandle_t
- L212: ffHandle_t
- L216: ffHandle_t
- L220: ffHandle_t
- L224: ffHandle_t
- L228: ffHandle_t
- L231: ffHandle_t
- L235: ffHandle_t
- L238: ffHandle_t
- L241: ffHandle_t
- L245: ffHandle_t
- L248: ffHandle_t
- L251: ffHandle_t
- L254: ffHandle_t

### crates/sp/cgame/src/media/cgs_t.rs
- L85: ffHandle_t

### crates/sp/engine/client/src/lib.rs
- L1: module sp_engine_client

### crates/sp/engine/ghoul2/src/lib.rs
- L1: module sp_engine_ghoul2
- L3: SSkinGoreData

### crates/sp/engine/icarus/src/blockstream/cblock_stream.rs
- L15: FILE

### crates/sp/engine/icarus/src/lib.rs
- L1: module sp_engine_icarus

### crates/sp/engine/qcommon/src/cm/clip_map_t.rs
- L73: CCMLandScape

### crates/sp/engine/qcommon/src/files/pack_t.rs
- L13: unzFile

### crates/sp/engine/qcommon/src/lib.rs
- L1: module sp_engine_qcommon

### crates/sp/engine/qcommon/src/timing/timing_c.rs
- L12: timing_c::start, timing_c::end, timing_c::reset

### crates/sp/engine/rmg/src/lib.rs
- L7: CRMManager (C++ track)

### crates/sp/engine/server/src/lib.rs
- L1: module sp_engine_server

### crates/sp/engine/server/src/server/sv_entity_s.rs
- L18: worldSector_s

### crates/sp/game/src/gi.rs
- L24: gi::* outbound-call wrappers (one per game_import_t member)

### crates/sp/game/src/lib.rs
- L3: module sp_game (dependency types ported first; SP places

### crates/sp/game/src/local/anim_file_set_t.rs
- L18: animNumber_t

### crates/sp/game/src/npc/g_npc_t.rs
- L75: rank_t

### crates/sp/game/src/shared/weapon_info_s.rs
- L64: ffHandle_t

### crates/sp/game/src/world/game_context.rs
- L37: SP export logic fns taking GameContext (per-export, logic-port)

### crates/sp/game/src/world/game_world.rs
- L28: GameWorld::zeroed (SP) — native_platform::zeroed_box for entities/level

### crates/sp/qshared/src/common/sp/qcommon/saber/saber_info.rs
- L15: saberInfoRetail_t

### crates/sp/qshared/src/shared/cvar.rs
- L16: cvar_t

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

### crates/sp/uishared/src/shared/cached_assets_t.rs
- L52: ffHandle_t

### crates/sp/uishared/src/shared/display_context_def_t.rs
- L137: CGhoul2Info
- L146: CGhoul2Info
- L176: CGhoul2Info
- L198: CGhoul2Info
- L211: ffHandle_t
- L214: ffHandle_t

### crates/sp/uishared/src/shared/item_def_s.rs
- L86: ffHandle_t

### crates/ui/src/lib.rs
- L8: ui live entrypoint exports (vmMain match, SEAM-D10)


## TODO-other (57)

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
- L668 [LEGIT]: // TODO: Play sleeping shuffle animation (Raven comment).
- L969 [LEGIT]: // TODO: Play a sound along the lines of, "Huh? What was that?" (Raven comment).

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
- L3574 [?]: //TODO: Use

### crates/mp/game/src/g_arenas.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_exphysics.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_log.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_main.rs
- L14 [STALE]: //! reflection Rust has none of and are left untranscribed with a `//TODO:

### crates/mp/game/src/g_mem.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_misc.rs
- L3396 [LEGIT]: //TODO: Find the target and set our angles to that direction

### crates/mp/game/src/g_mover.rs
- L2890 [?]: // `CacheChunkEffects` in this file); left as an explicit `//TODO: Port`

### crates/mp/game/src/g_nav.rs
- L950 [LEGIT]: // TODO: Handle all ents

### crates/mp/game/src/g_session.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_target.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_timer.rs
- L6 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_turret.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the
- L43 [?]: // Unported constants with TODO markers

### crates/mp/game/src/g_vehicleTurret.rs
- L8 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/g_vehicles.rs
- L496 [LEGIT]: // TODO: Set pilot should do all this stuff....
- L715 [LEGIT]: // TODO: Setup a default escape for every vehicle type.
- L1336 [LEGIT]: // MP: TODO MP Shift Sound Playback (no playback in MP)

### crates/mp/game/src/g_weapon.rs
- L7 [LEGIT]: //! ones carry `//TODO: Port <type>` markers. Re-run after editing the

### crates/mp/game/src/game_globals.rs
- L7 [LEGIT]: //! `//TODO: Port <type>` marker — the porter fills the real type when
- L831 [LEGIT]: // it"); shapes are exactly what the `g_log.md` packet's TODO comments

### crates/mp/game/src/npc_c.rs
- L1416 [LEGIT]: // TODO: Add vehicle behaviors here.

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
- L13 [LEGIT]: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` does not include a `UI_FLOOR` case in its visible switch

### crates/sp/abi/src/ui/syscalls/UI_FS_FCLOSEFILE.rs
- L12 [LEGIT]: /// TODO: using MP transport parity until SP table entry is confirmed.

### crates/sp/abi/src/ui/syscalls/UI_FS_READ.rs
- L17 [LEGIT]: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_READ` case in this branch.

### crates/sp/abi/src/ui/syscalls/UI_FS_WRITE.rs
- L17 [LEGIT]: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_WRITE` case in this branch.

### crates/sp/abi/src/ui/syscalls/UI_SIN.rs
- L13 [LEGIT]: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_SQRT.rs
- L13 [LEGIT]: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_STRNCPY.rs
- L16 [LEGIT]: /// TODO: SP-side `UI_STRNCPY` transport is known only via fallback (`TRAP_STRNCPY`) evidence.

### crates/sp/abi/src/ui/syscalls/UI_S_STARTLOCALSOUND.rs
- L15 [LEGIT]: /// TODO: SP transport path does not provide a direct `UI_S_STARTLOCALSOUND` case in `oracle/oracle/code/client/cl_ui.cpp`.


## NOTE (554)

### crates/mp/game/src/FighterNPC.rs
- L1309: (untopiced)
- L1364: bg-anim-dispatch

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
- L734: unresolved-callee-type

### crates/mp/game/src/NPC_AI_Jedi.rs
- L6: (untopiced)
- L1211: jediSpeechDebounceTime
- L1948: jediSpeechDebounceTime
- L2345: BG_AnimLength
- L4639: jediSpeechDebounceTime
- L5733: jediSpeechDebounceTime

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
- L1117: ambient-state
- L1152: ambient-state

### crates/mp/game/src/NPC_AI_Rancor.rs
- L6: (untopiced)

### crates/mp/game/src/NPC_AI_Remote.rs
- L318: static-vec3-locals

### crates/mp/game/src/NPC_AI_Sniper.rs
- L6: (untopiced)

### crates/mp/game/src/NPC_AI_Stormtrooper.rs
- L1296: vec3-out-param

### crates/mp/game/src/NPC_AI_Wampa.rs
- L13: (untopiced)

### crates/mp/game/src/NPC_behavior.rs
- L1128: goal-invariant

### crates/mp/game/src/NPC_combat.rs
- L1407: varargs-seam
- L1779: SVF_IGNORE_ENEMIES
- L2039: vec3-out-param-reshape

### crates/mp/game/src/NPC_misc.rs
- L32: varargs
- L75: varargs

### crates/mp/game/src/NPC_reactions.rs
- L20: (untopiced)
- L394: bgAllAnims-access
- L955: va-formatting
- L1088: vehicle-vtable

### crates/mp/game/src/NPC_spawn.rs
- L50: packet-contract
- L477: weaponData
- L2966: unresolved
- L2993: unresolved
- L3026: fn-ptr
- L3061: fn-ptr
- L3084: fn-ptr

### crates/mp/game/src/NPC_stats.rs
- L1470: vehicle-info-shape
- L2075: unported-fn
- L2128: unported-global

### crates/mp/game/src/NPC_utils.rs
- L31: (untopiced)

### crates/mp/game/src/SpeederNPC.rs
- L87: vtable-access
- L96: vtable-access
- L124: vtable-access
- L158: vtable-access
- L202: trap-access
- L371: vtable-access
- L482: bg-vehicle-table

### crates/mp/game/src/WalkerNPC.rs
- L125: pm-global

### crates/mp/game/src/ai_main.rs
- L191: StateDescriptions
- L398: seam-threading
- L564: ACTION_*/ANGLE2SHORT
- L676: SHORT2ANGLE/ACTION_*
- L790: botstates/PRT_FATAL/SHORT2ANGLE
- L911: botstates/PRT_FATAL
- L1260: bot_pvstype
- L1410: FORCEJUMP_INSTANTMETHOD
- L1490: forceJumpStrength/FORCEJUMP_INSTANTMETHOD
- L1592: FORCEJUMP_INSTANTMETHOD
- L1688: DEFAULT_MAXS_2/STRAFEAROUND_*
- L2093: botstates/ENEMY_FORGET_MS
- L2533: MAX_CHICKENWUSS_TIME/BOT_RUN_HEALTH
- L2646: ENEMY_FORGET_MS
- L2785: BASE_GUARD_DISTANCE
- L2820: BASE_GETENEMYFLAG_DISTANCE
- L6508: preproc
- L7350: gWPArray-oob §S19
- L7402: gWPArray-oob §S19
- L7546: §S19
- L7563: mLen-quirk
- L8114: fn-statics
- L8235: bot-cvars

### crates/mp/game/src/ai_wpnav.rs
- L2: (untopiced)
- L22: cvar-placement
- L23: missing-const
- L342: cvar-placement
- L1566: cvar-placement
- L1967: cvar-placement
- L2017: missing-global
- L2523: bg-0143
- L2616: bg-temp-alloc
- L3221: missing-const
- L3236: cvar-placement
- L3267: cvar-placement
- L3641: missing-const

### crates/mp/game/src/bg_g2_utils.rs
- L55: vec3-outparam

### crates/mp/game/src/bg_misc.rs
- L261: ctx-cascade
- L1430: escalation-resolution
- L1461: escalation-resolution
- L1768: qagame-cfg

### crates/mp/game/src/bg_panimate.rs
- L14: own-file-static-no-world-handle

### crates/mp/game/src/bg_pmove.rs
- L12: (untopiced)
- L240: law-signature-by-value
- L366: vehicle-angles-normal-by-value
- L693: vehicle-angles-normal-by-value
- L1172: meditate-timer-quirk
- L1777: byte-select-shift
- L4840: g_gametype

### crates/mp/game/src/bg_saber.rs
- L528: saber-move-name-coverage
- L1622: faithful-oracle-bug
- L1997: scope
- L2812: faithful-empty-branch
- L2940: client-saber-field

### crates/mp/game/src/bg_saberLoad.rs
- L332: missing-symbol
- L765: missing-const
- L928: missing-symbols
- L1082: missing-trait-method
- L2399: rng-cascade
- L2422: vec-as-fixed-buffer

### crates/mp/game/src/bg_slidemove.rs
- L68: bg-tier-gap
- L307: fn-ptr-skip
- L645: bg-tier-gap

### crates/mp/game/src/bg_vehicleLoad.rs
- L72: sscanf-parity
- L883: traps-cascade
- L918: traps-cascade

### crates/mp/game/src/g_ICARUScb.rs
- L74: variadic-c-abi
- L760: die-dispatch-invoke
- L832: vehicle-eject
- L897: unported-global
- L1201: unported-global
- L1272: unported-global
- L1573: client-still-void
- L2037: unported-global
- L2064: unported-global
- L2161: unported-global
- L2250: unported-global
- L2281: unported-global
- L2422: client-still-void
- L2579: unported-global
- L2962: dropped-warnings
- L3786: unported-consts
- L3858: unported-consts
- L4076: unported-consts
- L4101: sscanf

### crates/mp/game/src/g_client.rs
- L26: (untopiced)
- L63: unported-const
- L295: fn-ptr-store-shape-mismatch
- L331: fn-ptr-store-shape-mismatch
- L530: traps-va-plus-fn-ptr-store
- L626: dead-code
- L635: fn-ptr-store-shape-mismatch
- L1221: fn-ptr-store-plus-traps
- L1364: traps-plus-body-queue
- L1649: g2-ghoul2-traps
- L1795: multi-global-trap-va-cluster
- L1845: static-buffer
- L1958: multi-global-trap-va-cluster
- L2226: va-varargs-plus-rng
- L2313: g2-trap-plus-bg
- L2319: dead-code
- L2547: mega-fn-globals-traps-bg-fnptr
- L3371: multi-global-trap-g2
- L3654: vehicle-pointer-overlay

### crates/mp/game/src/g_cmds.rs
- L1: (untopiced)
- L12: <topic>
- L13: (untopiced)
- L20: (untopiced)
- L104: raw-ptr-skeleton-no-world-handle
- L212: raw-ptr-skeleton-no-world-handle
- L218: G_GetStringEdString
- L259: static-scratch-buffer-vs-raw-ptr-return
- L340: raw-ptr-skeleton-no-world-handle
- L418: raw-ptr-skeleton-no-world-handle
- L633: raw-ptr-skeleton-no-world-handle
- L666: raw-ptr-skeleton-no-world-handle
- L697: raw-ptr-skeleton-no-world-handle
- L769: raw-ptr-skeleton-no-world-handle
- L822: raw-ptr-skeleton-no-world-handle
- L941: raw-ptr-skeleton-no-world-handle
- L1117: raw-ptr-skeleton-no-world-handle
- L1224: G_LogPrintf
- L1266: raw-ptr-skeleton-no-world-handle
- L1476: raw-ptr-skeleton-no-world-handle
- L1508: raw-ptr-skeleton-no-world-handle
- L1608: raw-ptr-skeleton-no-world-handle
- L1725: unresolved-siege-team-consts
- L1733: siege-team-consts
- L1779: raw-ptr-skeleton-no-world-handle
- L1905: raw-ptr-skeleton-no-world-handle
- L1969: raw-ptr-skeleton-no-world-handle
- L1987: bgSiegeClasses
- L2127: raw-ptr-skeleton-no-world-handle
- L2149: G_Error
- L2198: raw-ptr-skeleton-no-world-handle
- L2285: raw-ptr-skeleton-no-world-handle
- L2394: G_Printf
- L2450: raw-ptr-skeleton-no-world-handle
- L2510: raw-ptr-skeleton-no-world-handle
- L2592: raw-ptr-skeleton-no-world-handle
- L2638: raw-ptr-skeleton-no-world-handle
- L2757: raw-ptr-skeleton-no-world-handle
- L3137: raw-ptr-skeleton-no-world-handle
- L3249: raw-ptr-skeleton-no-world-handle
- L3578: raw-ptr-skeleton-no-world-handle
- L3698: raw-ptr-skeleton-no-world-handle
- L3772: raw-ptr-skeleton-no-world-handle
- L3789: bg_itemlist
- L4025: raw-ptr-skeleton-no-world-handle
- L4147: bgSiegeClasses
- L4239: raw-ptr-skeleton-no-world-handle
- L4515: raw-ptr-skeleton-no-world-handle
- L4549: animTable/saberMoveData
- L4562: raw-ptr-skeleton-no-world-handle
- L4589: animTable
- L4714: raw-ptr-skeleton-no-world-handle
- L4853: FOFS-targetname
- L4949: debug-build-gated-cmds

### crates/mp/game/src/g_combat.rs
- L1825: static-death-anim-counter
- L3018: boltPoint-outparam
- L3217: bg-traps-handle
- L3667: point-null
- L3700: point-null
- L4328: point-null
- L4545: ctx-and-dir
- L5683: dir/point-null
- L5753: point-null
- L5769: ctx-and-dir

### crates/mp/game/src/g_items.rs
- L12: (untopiced)
- L165: raw-ptr-skeleton-no-world-handle
- L222: raw-ptr-skeleton-no-world-handle
- L243: raw-ptr-skeleton-no-world-handle
- L268: raw-ptr-skeleton-no-world-handle
- L293: raw-ptr-skeleton-no-world-handle
- L320: raw-ptr-skeleton-no-world-handle
- L376: raw-ptr-skeleton-no-world-handle
- L403: raw-ptr-skeleton-no-world-handle
- L437: raw-ptr-skeleton-no-world-handle
- L618: vec3-outparam-seam
- L759: raw-ptr-skeleton-no-world-handle
- L801: raw-ptr-skeleton-no-world-handle
- L845: raw-ptr-skeleton-no-world-handle
- L983: raw-ptr-skeleton-no-world-handle
- L1056: missing-const
- L1066: raw-ptr-skeleton-no-world-handle
- L1294: raw-ptr-skeleton-no-world-handle
- L1353: raw-ptr-skeleton-no-world-handle
- L1399: raw-ptr-skeleton-no-world-handle
- L1479: raw-ptr-skeleton-no-world-handle
- L1579: raw-ptr-skeleton-no-world-handle
- L1619: raw-ptr-skeleton-no-world-handle
- L1662: raw-ptr-skeleton-no-world-handle
- L1707: raw-ptr-skeleton-no-world-handle
- L1739: raw-ptr-skeleton-no-world-handle
- L1744: unported-global
- L1788: missing-const
- L1810: vec3-outparam-seam
- L1882: raw-ptr-skeleton-no-world-handle
- L1913: raw-ptr-skeleton-no-world-handle
- L1973: raw-ptr-skeleton-no-world-handle
- L1997: raw-ptr-skeleton-no-world-handle
- L2097: raw-ptr-skeleton-no-world-handle
- L2145: raw-ptr-skeleton-no-world-handle
- L2216: vec3-outparam-seam
- L2395: raw-ptr-skeleton-no-world-handle
- L2498: raw-ptr-skeleton-no-world-handle
- L2700: raw-ptr-skeleton-no-world-handle
- L2750: vec3-outparam-seam
- L2855: raw-ptr-skeleton-no-world-handle
- L2879: missing-const
- L2882: raw-ptr-skeleton-no-world-handle
- L2903: raw-ptr-skeleton-no-world-handle
- L2964: raw-ptr-skeleton-no-world-handle
- L3035: missing-const
- L3101: raw-ptr-skeleton-no-world-handle
- L3182: raw-ptr-skeleton-no-world-handle
- L3219: raw-ptr-skeleton-no-world-handle
- L3525: vec3-outparam-seam
- L3612: missing-const
- L3616: vec3-outparam-seam
- L3656: raw-ptr-skeleton-no-world-handle
- L3821: raw-ptr-skeleton-no-world-handle
- L3855: raw-ptr-skeleton-no-world-handle
- L3878: raw-ptr-skeleton-no-world-handle
- L3894: raw-ptr-skeleton-no-world-handle
- L3923: raw-ptr-skeleton-no-world-handle
- L3937: raw-ptr-skeleton-no-world-handle
- L3998: raw-ptr-skeleton-no-world-handle
- L4085: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_main.rs
- L11: <topic>
- L12: (untopiced)
- L68: variadic-c-abi
- L81: variadic-c-abi
- L93: variadic-c-abi
- L103: variadic-c-abi
- L169: unresolved-cvar-flags
- L180: no-field-reflection
- L245: unresolved-cvar-flags
- L255: no-field-reflection
- L273: variadic-c-abi
- L283: ctx-free-boundary
- L293: variadic-c-abi
- L302: ctx-free-boundary
- L631: unported-dep
- L724: qsort-fn-pointer-registration
- L830: unported-dep
- L924: qsort-ctx-mismatch
- L1152: unported-dep
- L1230: unported-const
- L1347: unported-dep
- L1436: variadic-c-abi
- L1446: variadic-c-abi
- L1478: unported-const
- L1551: unported-const
- L1925: unported-dep
- L2534: raw-ptr-skeleton-no-world-handle
- L2569: raw-ptr-skeleton-no-world-handle
- L2732: raw-ptr-skeleton-no-world-handle
- L2759: raw-ptr-skeleton-no-world-handle
- L2819: raw-ptr-skeleton-no-world-handle
- L2864: raw-ptr-skeleton-no-world-handle
- L2960: raw-ptr-skeleton-no-world-handle
- L2971: cross-frame-static
- L3012: raw-ptr-skeleton-no-world-handle
- L3049: raw-ptr-skeleton-no-world-handle
- L3123: raw-ptr-skeleton-no-world-handle
- L3129: ctx-free-boundary
- L3139: ctx-free-boundary
- L3155: raw-ptr-skeleton-no-world-handle
- L3692: raw-ptr-skeleton-no-world-handle
- L3711: static-scratch-buffer

### crates/mp/game/src/g_mem.rs
- L33: state-threading-g-alloc

### crates/mp/game/src/g_misc.rs
- L4: ...
- L472: raw-ptr-skeleton-no-world-handle
- L586: unported-const
- L790: raw-ptr-skeleton-no-world-handle
- L935: unported-const
- L1068: control-flow
- L1430: raw-ptr-skeleton-no-world-handle
- L1477: unported-const
- L2007: seam-threading
- L2138: seam-threading
- L2152: raw-ptr-skeleton-no-world-handle
- L2626: raw-ptr-skeleton-no-world-handle
- L2658: unported-const
- L2705: raw-ptr-skeleton-no-world-handle
- L2762: unported-const
- L2878: raw-ptr-skeleton-no-world-handle
- L2910: raw-ptr-skeleton-no-world-handle
- L3006: raw-ptr-skeleton-no-world-handle
- L3019: raw-ptr-skeleton-no-world-handle
- L3361: bg-dep
- L3375: bg-dep
- L3440: unported-const
- L3503: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_missile.rs
- L1216: gclient_t

### crates/mp/game/src/g_mover.rs
- L18: (untopiced)
- L589: bg-boundary
- L2886: unported-consts

### crates/mp/game/src/g_nav.rs
- L127: missing-const
- L143: dead-oracle-code
- L2183: ifdef-elided

### crates/mp/game/src/g_navnew.rs
- L29: (untopiced)

### crates/mp/game/src/g_object.rs
- L316: signature-mismatch

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
- L846: unported-const
- L862: unported-const
- L893: unported-const
- L926: raw-ptr-skeleton-no-world-handle
- L944: unported-const
- L1017: bg-boundary
- L1026: missing-fields
- L1145: bg-boundary
- L1307: bg-boundary
- L1380: unported-const
- L1389: unported-const
- L1510: unported-const
- L1557: bg-boundary
- L1656: bg-boundary
- L1728: unported-const
- L1748: bg-boundary
- L1806: bg-boundary
- L1885: bg-boundary
- L1938: bg-boundary
- L2014: unported-const
- L2024: unported-const
- L2270: unported-const
- L2337: bg-boundary
- L2606: bg-boundary
- L2616: unported-const

### crates/mp/game/src/g_spawn.rs
- L31: (untopiced)
- L1183: bg-panimate-method
- L1363: string-formatting

### crates/mp/game/src/g_target.rs
- L1040: EntityId-deref

### crates/mp/game/src/g_team.rs
- L9: (untopiced)

### crates/mp/game/src/g_timer.rs
- L49: level-global-access

### crates/mp/game/src/g_trigger.rs
- L2136: variadic-c-abi

### crates/mp/game/src/g_turret_G2.rs
- L11: (untopiced)
- L519: damage-flags
- L1274: atoi-usage

### crates/mp/game/src/g_utils.rs
- L20: (untopiced)
- L227: ctx-free-boundary-callee-mismatch
- L254: ctx-free-boundary-callee-mismatch
- L272: ctx-free-boundary-callee-mismatch
- L486: missing-vehicle-pool-storage
- L521: missing-vehicle-pool-storage
- L658: fn-pointer-dispatch-no-ctx
- L739: (untopiced)
- L1301: ctx-free-boundary-needs-worldstate
- L1330: ctx-free-boundary-needs-entity-alloc
- L1349: ctx-free-boundary-needs-entity-alloc
- L1556: missing-weapondata-table
- L1600: missing-weapondata-table
- L1643: missing-bgSiegeClasses
- L1742: vehicle-dispatch-not-wired
- L1815: vehicle-dispatch-not-wired
- L1880: fn-pointer-dispatch
- L2208: unported-callee

### crates/mp/game/src/g_vehicles.rs
- L6: (untopiced)
- L126: bg-channel
- L274: (untopiced)
- L412: m_vOrientation
- L688: G_Damage-null-dir
- L927: bg-channel
- L942: bg-boundary
- L1229: PM_BGEntForNum
- L1856: bg-boundary
- L2011: bg-boundary
- L2051: G_Damage-null-dir/point
- L2175: bg-boundary

### crates/mp/game/src/g_weapon.rs
- L200: seam-threading
- L258: seam-threading
- L263: file-static-globals
- L326: bg-dep
- L362: bg-dep
- L396: bg-dep
- L474: bg-dep
- L516: seam-threading
- L540: seam-threading
- L749: bg-dep
- L777: seam-threading
- L782: file-static-globals
- L1108: seam-threading
- L1148: seam-threading
- L1248: seam-threading
- L1279: seam-threading
- L1328: seam-threading
- L1353: seam-threading
- L1392: seam-threading
- L1557: seam-threading
- L1587: seam-threading
- L1683: seam-threading
- L1739: seam-threading
- L1789: seam-threading
- L1871: rng-threading
- L1924: seam-threading
- L1963: seam-threading
- L2147: seam-threading
- L2253: seam-threading
- L2337: seam-threading
- L2434: seam-threading
- L2439: file-static-globals
- L2452: seam-threading
- L2634: out-param-vec3
- L2639: seam-threading
- L2692: seam-threading
- L2727: seam-threading
- L2761: seam-threading
- L2823: seam-threading
- L2879: seam-threading
- L2972: seam-threading
- L3049: seam-threading
- L3147: seam-threading
- L3277: seam-threading
- L3369: seam-threading
- L3447: seam-threading
- L3493: seam-threading
- L3592: seam-threading
- L3853: seam-threading
- L3905: seam-threading
- L4025: seam-threading
- L4211: weapon-muzzle-table
- L4227: weapon-muzzle-table
- L4301: seam-threading
- L4361: seam-threading
- L4382: seam-threading
- L4624: bg-dep
- L4669: seam-threading
- L4688: unported-const
- L4785: out-param-vec3
- L4792: parked-dep
- L4802: unported-type
- L4912: parked-dep

### crates/mp/game/src/npc_c.rs
- L10: (untopiced)
- L313: raw-ptr-skeleton-no-world-handle
- L336: raw-ptr-skeleton-no-world-handle
- L471: raw-ptr-skeleton-no-world-handle
- L642: raw-ptr-skeleton-no-world-handle
- L658: raw-ptr-skeleton-no-world-handle
- L673: raw-ptr-skeleton-no-world-handle
- L687: raw-ptr-skeleton-no-world-handle
- L702: raw-ptr-skeleton-no-world-handle
- L749: raw-ptr-skeleton-no-world-handle
- L815: raw-ptr-skeleton-no-world-handle
- L881: raw-ptr-skeleton-no-world-handle
- L899: raw-ptr-skeleton-no-world-handle
- L944: raw-ptr-skeleton-no-world-handle
- L1267: raw-ptr-skeleton-no-world-handle
- L1440: raw-ptr-skeleton-no-world-handle
- L1604: raw-ptr-skeleton-no-world-handle
- L1693: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/q_shared.rs
- L340: variadic-c-abi
- L366: variadic-c-abi
- L384: variadic-c-abi
- L1144: variadic-c-abi
- L1165: variadic-c-abi

### crates/mp/game/src/w_force.rs
- L277: bot-forcepowers
- L1594: vehicle-vtable
- L3298: unported-global-and-vehicle-vtable
- L3883: vehicle-vtable
- L4246: unported-global-table
- L5465: unported-global-table

### crates/mp/game/src/w_saber.rs
- L210: bg-boundary
- L872: trailingLegsAngles
- L2291: PM_SaberBounceForAttack
- L2352: PM_SaberBounceForAttack
- L2786: faces
- L3432: BG_AnimLength
- L3841: missing-global-field
- L4205: missing-global-field
- L4296: saberClashPos/saberClashNorm
- L4300: BG_BrokenParryForAttack/BG_InKnockDownOnGround
- L7430: g2-seam-argshape
- L7449: g2-seam-argshape
- L8693: kickEnd-null
- L8863: BG_AnimLength-shape
- L9247: kickEnd-null
- L10305: array-decay
- L10329: array-decay

