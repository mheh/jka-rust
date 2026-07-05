# Marker inventory — PORT-NOTE / TODO (generated 2026-07-05)

Totals: 916 markers — 314 `TODO: Port`, 570 `PORT-NOTE`, 32 other TODO forms.


## TODO (314)


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
- L1: module mp_cgame

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

### crates/mp/engine/qcommon/src/lib.rs
- L1: module mp_engine_qcommon

### crates/mp/engine/qcommon/src/timing/timing_c.rs
- L12: timing_c::start, timing_c::end, timing_c::reset

### crates/mp/engine/rmg/src/lib.rs
- L7: CRMManager (C++ track)

### crates/mp/engine/server/src/lib.rs
- L1: module mp_engine_server

### crates/mp/engine/server/src/server/sv_entity_s.rs
- L18: worldSector_s

### crates/mp/engine/server/src/server_host.rs
- L16: Server fields (sv: server_t incl. the SS_DEAD liveness state,
- L30: ServerGame concrete shape (alias vs wrapper — STATE-Q7)
- L49: SV_GameSystemCalls exhaustive dispatch

### crates/mp/game/src/AnimalNPC.rs
- L16: vehicleInfo_t`,

### crates/mp/game/src/FighterNPC.rs
- L7: <type>` markers. Re-run after editing the
- L103: void ()(trace_t , vec_t , vec_t , vec_t , vec_t , int, int)  (C: `void (*)(trace_t *, const vec_t *, const vec_t *, const vec_t *, const vec_t *, int, int)`)

### crates/mp/game/src/NPC_AI_Atst.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Droid.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Grenadier.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Howler.rs
- L8: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Mark2.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_AI_Sniper.rs
- L744: Sniper_FireDecide miss loop body (VectorMA flrand(1.5,4)

### crates/mp/game/src/NPC_AI_Utils.rs
- L1023: FRAMETIME

### crates/mp/game/src/NPC_combat.rs
- L7: <type>` markers. Re-run after editing the
- L2429: combatPt_t  (C: `combatPt_t *`)

### crates/mp/game/src/NPC_move.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_sounds.rs
- L8: <type>` markers. Re-run after editing the

### crates/mp/game/src/NPC_spawn.rs
- L7: <type>` markers. Re-run after editing the
- L1650: BG_ParseAnimationFile call wiring

### crates/mp/game/src/NPC_stats.rs
- L7: <type>` markers. Re-run after editing the
- L361: rank_t (unported return enum; C int-width)

### crates/mp/game/src/SpeederNPC.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/ai_main.rs
- L7: <type>` markers. Re-run after editing the
- L907: bot_settings_s  (C: `struct bot_settings_s *`)

### crates/mp/game/src/bg_panimate.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/bg_saga.rs
- L7: <type>` markers. Re-run after editing the
- L899: FPTable

### crates/mp/game/src/bg_vehicleLoad.rs
- L300: BG_SetSharedVehicleFunctions

### crates/mp/game/src/g_ICARUScb.rs
- L14: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_active.rs
- L82: VectorCompare           // Source: oracle/oracle/codemp/game/q_shared.h

### crates/mp/game/src/g_arenas.rs
- L8: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_client.rs
- L3425: m_pVehicleInfo->Eject

### crates/mp/game/src/g_exphysics.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_init_game.rs
- L38: G_InitGame body (G_RegisterCvars, level wiring, back-pointers,

### crates/mp/game/src/g_items.rs
- L1765: bg_itemlist
- L1967: bg_itemlist
- L2866: bg_itemlist
- L2897: ammoData
- L3018: weaponData
- L3390: weaponData
- L3555: bg_itemlist
- L3776: bg_itemlist
- L3858: bg_itemlist
- L3915: bg_itemlist
- L3932: bg_numItems

### crates/mp/game/src/g_log.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_main.rs
- L134: CONTENTS_TRIGGER
- L193: G_RegisterCvars per-row trap_Cvar_Register loop
- L264: G_UpdateCvars per-row trap_Cvar_Update loop
- L675: CS_CLIENT_DUELWINNER
- L998: CS_SCORES1/CS_SCORES2/CS_CLIENT_DUELWINNER/SCORE_NOT_PRESENT
- L1261: CS_CLIENT_DUELWINNER
- L1382: EXEC_APPEND
- L1510: CS_INTERMISSION
- L1715: CS_CLIENT_DUELISTS/CS_CLIENT_DUELWINNER
- L2176: CS_* PRINTREDTEAM/PRINTBLUETEAM string keys are fine (they
- L2278: CS_CLIENT_DUELISTS/CS_CLIENT_DUELHEALTHS/CS_WARMUP
- L2755: CS_VOTE_TIME
- L2985: CS_TEAMVOTE_TIME
- L3171: BG_GetTime level.time reach-through
- L3187: level.time reach-through

### crates/mp/game/src/g_mem.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_misc.rs
- L99: bSet_e
- L2255: FX_STATE_OFF
- L2258: FX_STATE_ONE_SHOT
- L2261: FX_STATE_ONE_SHOT_LIMIT
- L2264: FX_STATE_CONTINUOUS
- L3060: tagOwner_t (unported return type; C: `tagOwner_t *`)
- L3093: tagOwner_t  (C: `tagOwner_t *`)
- L3137: tagOwner_t (unported return type; C: `tagOwner_t *`)

### crates/mp/game/src/g_mover.rs
- L1763: EF_SHADER_ANIM
- L2299: EF_SHADER_ANIM
- L2321: EF2_HYPERSPACE
- L2338: EF_PERMANENT
- L2472: EF_RADAROBJECT
- L2696: EF_MISSILE_STICK
- L2939: MAT_DRK_STONE, MAT_LT_STONE, MAT_GREY_STONE, MAT_SNOWY_ROCK
- L3010: SVF_PLAYER_USABLE
- L3154: SVF_GLASS_BRUSH
- L3316: SVF_GLASS_BRUSH
- L3396: SVF_PLAYER_USABLE
- L3574: EF_SHADER_ANIM

### crates/mp/game/src/g_nav.rs
- L2074: fatalErrorString append
- L2198: fatalErrorString reset

### crates/mp/game/src/g_session.rs
- L7: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_shutdown_game.rs
- L17: G_ShutdownGame body (G_SaveBanIP, fake-client + ghoul2 cleanup)

### crates/mp/game/src/g_target.rs
- L7: <type>` markers. Re-run after editing the
- L465: fn-pointer assignment: (*ent).use = Use_Target_Speaker;
- L615: fn-pointer assignments:
- L716: fn-pointer assignment: (*self_).use = NULL;
- L719: fn-pointer assignments:
- L942: fn-pointer assignment: (*self_).use = NULL;
- L1117: fn-pointer assignment: (*self_).think = scriptrunner_run;

### crates/mp/game/src/g_timer.rs
- L6: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_trigger.rs
- L54: bSet_e
- L74: PMF_FOLLOW
- L77: CS_GLOBAL_AMBIENT_SET
- L80: SIEGETEAM_TEAM1
- L83: SIEGETEAM_TEAM2
- L86: PUSH_CONSTANT
- L89: PUSH_LINEAR
- L92: PUSH_RELATIVE
- L95: PUSH_MULTIPLE
- L98: HYPERSPACE_TIME
- L101: HYPERSPACE_TELEPORT_FRAC
- L104: EF2_HYPERSPACE
- L107: EF_RAG
- L110: INITIAL_SUFFOCATION_DELAY
- L1379: PM_DEAD / ps.pm_type check — playerState_t.pm_type field

### crates/mp/game/src/g_turret.rs
- L7: <type>` markers. Re-run after editing the
- L42: MASK_SHOT
- L46: CONTENTS_LIGHTSABER
- L50: MAT_METAL
- L54: CLASS_VEHICLE

### crates/mp/game/src/g_vehicleTurret.rs
- L8: <type>` markers. Re-run after editing the

### crates/mp/game/src/g_vehicles.rs
- L1032: SetClientViewAngle
- L1297: SetClientViewAngle
- L2639: SetClientViewAngle
- L2799: RegisterAssets

### crates/mp/game/src/g_weapon.rs
- L7: <type>` markers. Re-run after editing the
- L771: vehicleInfo_t (Vehicle_t::m_pVehicleInfo is still *mut c_void)
- L4012: vehicleInfo_t (Vehicle_t::m_pVehicleInfo is still *mut c_void)
- L4316: vehicleInfo_t (Vehicle_t::m_pVehicleInfo is still *mut c_void)

### crates/mp/game/src/game_globals.rs
- L7: <type>` marker — the porter fills the real type when
- L661: navInfo_t
- L1027: gtimer_t **
- L1030: gtimer_t[ MAX_GTIMERS ]
- L1033: gtimer_t *[ MAX_GENTITIES ]*

### crates/mp/game/src/lib.rs
- L4: the

### crates/mp/game/src/npc/g_npc_t.rs
- L75: rank_t

### crates/mp/game/src/q_shared.rs
- L7: <type>` markers. Re-run after editing the
- L1306: MAX_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_RemoveKey: oversize infostring")) once the const is ported
- L1368: BIG_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_RemoveKey_Big: oversize infostring")) once the const is ported
- L1445: MAX_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring")) once the const is ported
- L1475: MAX_INFO_STRING (currently hardcoded 1024 — not yet a
- L1496: BIG_INFO_STRING oversize guard (Com_Error(ERR_DROP, "Info_SetValueForKey: oversize infostring")) once the const is ported
- L1526: BIG_INFO_STRING (currently hardcoded 8192 — not yet a

### crates/mp/game/src/tri_coll_test.rs
- L12: <type>` markers. Re-run after editing the

### crates/mp/game/src/w_force.rs
- L117: FORCE_LIGHTNING_RADIUS   // Source: oracle/oracle/codemp/game/w_force.c
- L118: MAX_DRAIN_DISTANCE       // Source: oracle/oracle/codemp/game/w_force.c
- L119: MAX_TRICK_DISTANCE       // Source: oracle/oracle/codemp/game/w_force.c
- L120: MASK_SHOT                // Source: oracle/oracle/codemp/game/q_shared.h
- L121: MASK_PLAYERSOLID         // Source: oracle/oracle/codemp/game/q_shared.h
- L122: GRIP_DRAIN_AMOUNT        // Source: oracle/oracle/codemp/game/g_local.h
- L123: SVF_BOT                   // Source: oracle/oracle/codemp/game/q_shared.h
- L124: FJ_FORWARD               // Source: oracle/oracle/codemp/game/w_force.c
- L125: FJ_BACKWARD              // Source: oracle/oracle/codemp/game/w_force.c
- L126: FJ_RIGHT                 // Source: oracle/oracle/codemp/game/w_force.c
- L127: FJ_LEFT                  // Source: oracle/oracle/codemp/game/w_force.c
- L128: FJ_UP                    // Source: oracle/oracle/codemp/game/w_force.c

### crates/mp/game/src/w_saber.rs
- L7: <type>` markers. Re-run after editing the
- L1377: sabersLockMode_t  (C: `sabersLockMode_t`)
- L2785: saberFace_t  (C: `saberFace_t **`)
- L2962: saberFace_t  (C: `saberFace_t *`)

### crates/mp/game/src/world/game_context.rs
- L304: Dispatch<C> for GameContext (GAME_ICARUS_* commands)

### crates/mp/qshared/src/common/mp/botlib/aas_export_s.rs
- L52: aas_areainfo_s

### crates/mp/qshared/src/common/mp/cgame/mini_ref_entity_s.rs
- L19: refEntityType_t

### crates/mp/qshared/src/common/mp/gentity.rs
- L94: Vehicle_t
- L168: gNPC_t

### crates/mp/qshared/src/common/mp/qcommon/shared_entity_t.rs
- L29: Vehicle_t

### crates/mp/qshared/src/shared/cvar.rs
- L16: cvar_t

### crates/mp/renderer/src/lib.rs
- L1: module mp_renderer

### crates/mp/renderer/src/tr_landscape/ctrland_scape.rs
- L23: CCMLandScape

### crates/mp/renderer/src/tr_landscape/ctrpatch.rs
- L17: CCMLandScape
- L21: CCMPatch

### crates/mp/renderer/src/tr_local/crenderable_surface.rs
- L15: CBoneCache

### crates/mp/ui/src/lib.rs
- L1: module mp_ui

### crates/mp/ui/src/local/map_info.rs
- L7: MAX_GAMETYPES

### crates/mp/ui/src/local/player_species_info_t.rs
- L5: MAX_PLAYERMODELS

### crates/mp/uishared/src/lib.rs
- L1: module mp_uishared

### crates/mp/uishared/src/shared/command_def_t.rs
- L14: itemDef_t

### crates/mp/uishared/src/shared/display_context_def_t.rs
- L63: refEntity_t
- L154: itemDef_t

### crates/mp/uishared/src/shared/menu_def_t.rs
- L53: itemDef_t

### crates/mp/uishared/src/shared/script_def_t.rs
- L5: MAX_SCRIPT_ARGS

### crates/native/containers/src/lib.rs
- L1: module native_containers

### crates/native/platform/src/module_loader/loader.rs
- L35: Sys_UnpackDLL — pure-server pk3 unpack pre-step, deferred (LOAD-D7)
- L75: NDEBUG in-loader fatal (LOAD-Q13 mechanism)

### crates/native/platform/src/module_loader/naming.rs
- L14: ModuleNaming macOS suffix

### crates/sp/abi/src/game/public/game_export_t.rs
- L32: Init variadic-free but multi-arg pointer signature

### crates/sp/abi/src/game/public/game_import_t.rs
- L31: Printf variadic args
- L40: Error variadic args
- L50: cvar_t
- L124: SendServerCommand variadic args
- L229: CGhoul2Info
- L435: CMiniHeap
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

### crates/sp/bg/src/local/mod.rs
- L3: module sp_bg::local — subsystem dir only; porters add flat

### crates/sp/bg/src/vehicles/mod.rs
- L3: module sp_bg::vehicles — subsystem dir only; porters add flat

### crates/sp/bg/src/weapons/mod.rs
- L3: module sp_bg::weapons — subsystem dir only; porters add flat

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
- L47: clientInfo_t (cross-crate, sp_game -> sp_cgame not wired)
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

### crates/sp/engine/server/src/server_host.rs
- L30: server_t/serverStatic_t fields (SP Server island)

### crates/sp/game/src/gi.rs
- L24: gi::* outbound-call wrappers (one per game_import_t member)

### crates/sp/game/src/lib.rs
- L3: module sp_game (dependency types ported first; SP places

### crates/sp/game/src/local/anim_file_set_t.rs
- L18: animNumber_t

### crates/sp/game/src/npc/g_npc_t.rs
- L75: rank_t

### crates/sp/game/src/shared/weapon_info_s.rs
- L40: centity_t
- L46: centity_t
- L64: ffHandle_t

### crates/sp/game/src/world/game_context.rs
- L37: SP export logic fns taking GameContext (per-export, logic-port)

### crates/sp/game/src/world/game_world.rs
- L28: GameWorld::zeroed (SP) — native_platform::zeroed_box for entities/level

### crates/sp/qshared/src/common/sp/gentity.rs
- L47: moverState_t (cross-tier: real enum lives in sp_game)
- L56: material_t (cross-tier: real enum lives in sp_game)
- L65: team_t (cross-tier: real enum lives in sp_game)
- L130: gclient_s (cross-tier: real struct lives in sp_game)
- L348: Vehicle_t (cross-tier: real struct lives in sp_game)
- L355: gNPC_t (cross-tier: real struct lives in sp_game)

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

### crates/sp/ui/src/local/player_species_info_t.rs
- L5: MAX_PLAYERMODELS

### crates/sp/uishared/src/lib.rs
- L1: module sp_uishared

### crates/sp/uishared/src/shared/cached_assets_t.rs
- L52: ffHandle_t

### crates/sp/uishared/src/shared/command_def_t.rs
- L14: itemDef_t

### crates/sp/uishared/src/shared/display_context_def_t.rs
- L137: CGhoul2Info
- L146: CGhoul2Info
- L176: CGhoul2Info
- L198: CGhoul2Info
- L211: ffHandle_t
- L214: ffHandle_t

### crates/sp/uishared/src/shared/item_def_s.rs
- L86: ffHandle_t

### crates/sp/uishared/src/shared/menu_def_t.rs
- L53: itemDef_s

### crates/ui/src/lib.rs
- L8: ui live entrypoint exports (vmMain match, SEAM-D10)

## TODO-other (32)


### crates/mp/engine/core/src/lifecycle.rs
- L33: /// carry `//TODO: Port` markers in step order so the transcript diff (DEC-09.2)

### crates/mp/game/src/NPC_AI_Stormtrooper.rs
- L668: // TODO: Play sleeping shuffle animation (Raven comment).
- L969: // TODO: Play a sound along the lines of, "Huh? What was that?" (Raven comment).

### crates/mp/game/src/g_active.rs
- L3576: //TODO: Use

### crates/mp/game/src/g_main.rs
- L14: //! reflection Rust has none of and are left untranscribed with a `//TODO:

### crates/mp/game/src/g_misc.rs
- L3403: //TODO: Find the target and set our angles to that direction

### crates/mp/game/src/g_mover.rs
- L2269: // with those two statements each left as an explicit `//TODO: Port` no-op
- L2397: // anywhere in the crate graph yet; left as an explicit `//TODO: Port` no-op.
- L2903: // `CacheChunkEffects` in this file); left as an explicit `//TODO: Port`
- L3141: // crate graph yet; skipped with an explicit `//TODO: Port` — the remaining
- L3516: // that one assignment is left as an explicit `//TODO: Port` no-op, the rest

### crates/mp/game/src/g_nav.rs
- L950: // TODO: Handle all ents
- L2086: // into `fatalErrorString` (see the `//TODO: Port` above).

### crates/mp/game/src/g_turret.rs
- L41: // Unported constants with TODO markers

### crates/mp/game/src/g_vehicles.rs
- L496: // TODO: Set pilot should do all this stuff....
- L715: // TODO: Setup a default escape for every vehicle type.
- L1340: // MP: TODO MP Shift Sound Playback (no playback in MP)

### crates/mp/game/src/game_globals.rs
- L831: // it"); shapes are exactly what the `g_log.md` packet's TODO comments

### crates/mp/game/src/npc_c.rs
- L1416: // TODO: Add vehicle behaviors here.

### crates/mp/game/src/prelude.rs
- L10: //! No new behavior lives here — only re-exports. The `//TODO: Port` markers for

### crates/sp/abi/src/ui/syscalls/UI_ATAN2.rs
- L10: /// TODO: SP transport evidence for float ABI is still missing; this follows MP 

### crates/sp/abi/src/ui/syscalls/UI_CEIL.rs
- L11: /// TODO: SP transport evidence for float ABI is still missing; this follows MP 

### crates/sp/abi/src/ui/syscalls/UI_CM_LERPTAG.rs
- L15: /// TODO: SP `UI_CM_LERPTAG` has no explicit engine switch case in

### crates/sp/abi/src/ui/syscalls/UI_COS.rs
- L13: /// TODO: SP transport evidence for float ABI is still missing; this mirrors MP 

### crates/sp/abi/src/ui/syscalls/UI_FLOOR.rs
- L13: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` does not include a `UI_FLOOR`

### crates/sp/abi/src/ui/syscalls/UI_FS_FCLOSEFILE.rs
- L12: /// TODO: using MP transport parity until SP table entry is confirmed.

### crates/sp/abi/src/ui/syscalls/UI_FS_READ.rs
- L17: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_READ` case in t

### crates/sp/abi/src/ui/syscalls/UI_FS_WRITE.rs
- L17: /// TODO: SP `oracle/oracle/code/client/cl_ui.cpp` has no `UI_FS_WRITE` case in 

### crates/sp/abi/src/ui/syscalls/UI_SIN.rs
- L13: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_SQRT.rs
- L13: /// TODO: SP UI transport evidence for this call is currently missing in

### crates/sp/abi/src/ui/syscalls/UI_STRNCPY.rs
- L16: /// TODO: SP-side `UI_STRNCPY` transport is known only via fallback (`TRAP_STRNC

### crates/sp/abi/src/ui/syscalls/UI_S_STARTLOCALSOUND.rs
- L15: /// TODO: SP transport path does not provide a direct `UI_S_STARTLOCALSOUND` cas

## NOTE (570)


### crates/mp/game/src/FighterNPC.rs
- L1308: (untopiced)
- L1363: bg-anim-dispatch

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

### crates/mp/game/src/NPC_AI_Utils.rs
- L1015: unported-consts

### crates/mp/game/src/NPC_AI_Wampa.rs
- L13: (untopiced)

### crates/mp/game/src/NPC_behavior.rs
- L1128: goal-invariant

### crates/mp/game/src/NPC_combat.rs
- L1407: varargs-seam
- L1779: SVF_IGNORE_ENEMIES
- L2039: vec3-out-param-reshape
- L2435: combatPt_t

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
- L1645: bg-tier-panimate
- L2959: unresolved
- L2986: unresolved
- L3019: fn-ptr
- L3054: fn-ptr
- L3077: fn-ptr

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
- L3420: vehicle-eject-unported
- L3653: vehicle-pointer-overlay

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
- L1792: missing-const
- L1814: vec3-outparam-seam
- L1886: raw-ptr-skeleton-no-world-handle
- L1917: raw-ptr-skeleton-no-world-handle
- L1977: raw-ptr-skeleton-no-world-handle
- L2001: raw-ptr-skeleton-no-world-handle
- L2101: raw-ptr-skeleton-no-world-handle
- L2149: raw-ptr-skeleton-no-world-handle
- L2220: vec3-outparam-seam
- L2399: raw-ptr-skeleton-no-world-handle
- L2502: raw-ptr-skeleton-no-world-handle
- L2704: raw-ptr-skeleton-no-world-handle
- L2754: vec3-outparam-seam
- L2859: raw-ptr-skeleton-no-world-handle
- L2887: missing-const
- L2890: raw-ptr-skeleton-no-world-handle
- L2914: raw-ptr-skeleton-no-world-handle
- L2975: raw-ptr-skeleton-no-world-handle
- L3049: missing-const
- L3115: raw-ptr-skeleton-no-world-handle
- L3196: raw-ptr-skeleton-no-world-handle
- L3233: raw-ptr-skeleton-no-world-handle
- L3539: vec3-outparam-seam
- L3630: missing-const
- L3634: vec3-outparam-seam
- L3674: raw-ptr-skeleton-no-world-handle
- L3843: raw-ptr-skeleton-no-world-handle
- L3881: raw-ptr-skeleton-no-world-handle
- L3904: raw-ptr-skeleton-no-world-handle
- L3924: raw-ptr-skeleton-no-world-handle
- L3956: raw-ptr-skeleton-no-world-handle
- L3970: raw-ptr-skeleton-no-world-handle
- L4031: raw-ptr-skeleton-no-world-handle
- L4118: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_main.rs
- L11: <topic>
- L12: (untopiced)
- L70: variadic-c-abi
- L83: variadic-c-abi
- L95: variadic-c-abi
- L105: variadic-c-abi
- L173: unresolved-cvar-flags
- L184: no-field-reflection
- L249: unresolved-cvar-flags
- L259: no-field-reflection
- L277: variadic-c-abi
- L287: ctx-free-boundary
- L297: variadic-c-abi
- L306: ctx-free-boundary
- L635: unported-dep
- L730: qsort-fn-pointer-registration
- L836: unported-dep
- L869: unported-const
- L936: qsort-ctx-mismatch
- L1166: unported-dep
- L1244: unported-const
- L1363: unported-dep
- L1454: variadic-c-abi
- L1464: variadic-c-abi
- L1496: unported-const
- L1571: unported-const
- L1947: unported-dep
- L2265: unported-const
- L2567: raw-ptr-skeleton-no-world-handle
- L2602: raw-ptr-skeleton-no-world-handle
- L2767: raw-ptr-skeleton-no-world-handle
- L2794: raw-ptr-skeleton-no-world-handle
- L2854: raw-ptr-skeleton-no-world-handle
- L2899: raw-ptr-skeleton-no-world-handle
- L2997: raw-ptr-skeleton-no-world-handle
- L3008: cross-frame-static
- L3049: raw-ptr-skeleton-no-world-handle
- L3086: raw-ptr-skeleton-no-world-handle
- L3160: raw-ptr-skeleton-no-world-handle
- L3166: ctx-free-boundary
- L3176: ctx-free-boundary
- L3192: raw-ptr-skeleton-no-world-handle
- L3729: raw-ptr-skeleton-no-world-handle
- L3748: static-scratch-buffer

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
- L3057: bg-dep
- L3086: bg-dep
- L3368: bg-dep
- L3382: bg-dep
- L3447: unported-const
- L3510: raw-ptr-skeleton-no-world-handle

### crates/mp/game/src/g_missile.rs
- L1216: gclient_t

### crates/mp/game/src/g_mover.rs
- L18: (untopiced)
- L589: bg-boundary
- L2265: unported-consts
- L2395: unported-consts
- L2899: unported-consts
- L3139: unported-consts
- L3302: unported-consts
- L3387: unported-consts
- L3512: unported-consts

### crates/mp/game/src/g_nav.rs
- L127: missing-const
- L143: dead-oracle-code
- L2184: ifdef-elided

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
- L1042: EntityId-deref

### crates/mp/game/src/g_team.rs
- L9: (untopiced)

### crates/mp/game/src/g_timer.rs
- L49: level-global-access

### crates/mp/game/src/g_trigger.rs
- L2149: variadic-c-abi

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
- L1231: PM_BGEntForNum
- L1860: bg-boundary
- L2015: bg-boundary
- L2055: G_Damage-null-dir/point
- L2179: bg-boundary

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
- L766: bg-dep
- L783: seam-threading
- L788: file-static-globals
- L1114: seam-threading
- L1154: seam-threading
- L1254: seam-threading
- L1285: seam-threading
- L1334: seam-threading
- L1359: seam-threading
- L1398: seam-threading
- L1563: seam-threading
- L1593: seam-threading
- L1689: seam-threading
- L1745: seam-threading
- L1795: seam-threading
- L1877: rng-threading
- L1930: seam-threading
- L1969: seam-threading
- L2153: seam-threading
- L2259: seam-threading
- L2343: seam-threading
- L2440: seam-threading
- L2445: file-static-globals
- L2458: seam-threading
- L2640: out-param-vec3
- L2645: seam-threading
- L2698: seam-threading
- L2733: seam-threading
- L2767: seam-threading
- L2829: seam-threading
- L2885: seam-threading
- L2978: seam-threading
- L3055: seam-threading
- L3153: seam-threading
- L3283: seam-threading
- L3375: seam-threading
- L3453: seam-threading
- L3499: seam-threading
- L3598: seam-threading
- L3859: seam-threading
- L3911: seam-threading
- L4033: seam-threading
- L4219: weapon-muzzle-table
- L4235: weapon-muzzle-table
- L4309: seam-threading
- L4371: seam-threading
- L4392: seam-threading
- L4634: bg-dep
- L4679: seam-threading
- L4698: unported-const
- L4795: out-param-vec3
- L4802: parked-dep
- L4812: unported-type
- L4922: parked-dep

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
- L294: bot-forcepowers
- L1611: vehicle-vtable
- L3315: unported-global-and-vehicle-vtable
- L3900: vehicle-vtable
- L4263: unported-global-table
- L5482: unported-global-table

### crates/mp/game/src/w_saber.rs
- L213: bg-boundary
- L875: trailingLegsAngles
- L2293: PM_SaberBounceForAttack
- L2354: PM_SaberBounceForAttack
- L2789: faces
- L2956: saberFace_t
- L3439: BG_AnimLength
- L3848: missing-global-field
- L4212: missing-global-field
- L4303: saberClashPos/saberClashNorm
- L4307: BG_BrokenParryForAttack/BG_InKnockDownOnGround
- L7437: g2-seam-argshape
- L7456: g2-seam-argshape
- L8700: kickEnd-null
- L8870: BG_AnimLength-shape
- L9254: kickEnd-null
- L10312: array-decay
- L10336: array-decay
