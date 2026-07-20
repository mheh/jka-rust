//! `g_init_game` — Raven `G_InitGame` (`g_main.c:897-1118`).
#![allow(non_snake_case, unused, clippy::all)]

use crate::prelude::*;

use crate::ai_main::{BotAILoadMap, BotAISetup};
use crate::ai_util::B_InitAlloc;
use crate::bg_channel::{GameBgTraps, GameCallbacksImpl};
use crate::g_bot::G_InitBots;
use crate::g_client::InitBodyQue;
use crate::g_items::{ClearRegisteredItems, G_CheckTeamItems, SaveRegisteredItems};
use crate::g_local_consts::{SP_PODIUM_MODEL, START_TIME_NAV_CALC};
use crate::g_log::G_LogWeaponInit;
use crate::g_main::{G_FindTeams, G_LogPrintf, G_Printf, G_RegisterCvars, G_RemapTeamShaders};
use crate::g_mem::G_InitMemory;
use crate::g_saga::InitSiegeMode;
use crate::g_session::G_InitWorldSession;
use crate::g_spawn::G_SpawnEntitiesFromString;
use crate::g_svcmds::G_LoadIPBans;
use crate::g_timer::TIMER_Clear;
use crate::g_utils::{G_ModelIndex, G_SoundIndex};
use crate::level::level_locals::level_locals_t;
use crate::npc_c::NPC_InitGame;
use crate::NPC_combat::CP_FindCombatPointWaypoints;
use mp_bg::bg_saberLoad::WP_SaberLoadParms;
use mp_bg::bg_vehicleLoad::BG_VehicleLoadParms;

// Explicit import: `mp_bg::public::configstring::*` and `crate::g_client::*`
// both glob-export `CS_CLIENT_JEDIMASTER` (prelude ambiguous-glob-reexports
// warning); an explicit import resolves the ambiguity for this file, matching
// the g_main.rs precedent for the same configstring family.
use mp_bg::public::configstring::CS_CLIENT_JEDIMASTER;

use mp_abi::game::vmcalls::GAME_INIT::GameInitArgs;

use mp_abi::game::syscalls::G_G2_CLEANENTATTACHMENTS::GG2CleanentattachmentsArgs;
use mp_abi::game::syscalls::G_ICARUS_INIT::GIcarusInitArgs;
use mp_abi::game::syscalls::G_LOCATE_GAME_DATA::GLocateGameDataArgs;
use mp_abi::game::syscalls::G_NAV_SETPATHSCALCULATED::GNavSetpathscalculatedArgs;
use mp_abi::game::syscalls::G_SET_SHARED_BUFFER::GSetSharedBufferArgs;

// `MAX_INFO_STRING` resolves via the crate prelude glob
// (`mp_qshared::shared::limits`); the shadowing local copy (and its stale "no
// ported home" note) was removed by the placeholder-const sweep.

/// Raven `void G_InitGame( int levelTime, int randomSeed, int restart )`.
///
/// Source: `oracle/codemp/game/g_main.c:897-1118`
pub fn g_init_game(ctx: &mut GameContext, args: GameInitArgs) {
    // Arm the ctx-less `strap_*` seam engine cell from the GAME_INIT
    // entrypoint that owns the engine, before any bg logic can call `strap_*`.
    crate::g_strap::init_strap_engine(ctx.engine);
    // Arm (re-arm) the seam world cell for the ctx-less boundary fns whose
    // oracle bodies read the `level` global (`G_AddEvent`/`G_PlayEffect`/
    // `G_PlayEffectID`, `g_utils.c`); the shell rebuilds the world Box every
    // GAME_INIT, so this re-arms each time.
    crate::g_strap::init_strap_world(ctx.world);

    unsafe {
        // Raven: `#ifdef _XBOX` guards `BG_ClearVehicleParseParms()` +
        // `RemoveAllWP()` here (`g_main.c:902-907`) — dead branch on this
        // (non-Xbox) build, dropped per the `_XBOX`-branch-drop precedent
        // (`q_math.rs`, `NPC_utils.rs`).

        // Init RMG to 0, it will be autoset to 1 if there is terrain on the level.
        trap::Cvar_Set(ctx.engine, "RMG", "0");
        ctx.world.cvars.g_RMG.integer = 0;

        // Clean up any client-server ghoul2 instance attachments that may
        // still exist exe-side.
        trap::G2API_CleanEntAttachments(ctx.engine, GG2CleanentattachmentsArgs::new());

        // The bg-channel handles this transcription threads through the
        // BgTraps/GameCallbacks seam (bg has no `Engine`/`GameContext`).
        let bg_traps = GameBgTraps::new(ctx.engine);

        // BG_InitAnimsets(); //clear it out
        {
            let mut callbacks = GameCallbacksImpl {
                // STAGE-2b: irreducible — `GameCallbacksImpl.world` is a `*mut GameWorld` bg-seam field; a raw store is required.
                world: ctx.world_raw(),
                engine: ctx.engine,
            };
            let mut pmc = PmoveContext::new(&mut ctx.world.bg_state, &bg_traps, &mut callbacks);
            pmc.BG_InitAnimsets();
        }

        B_InitAlloc(ctx); //make sure everything is clean

        trap::SV_RegisterSharedMemory(
            ctx.engine,
            GSetSharedBufferArgs::new(ctx.world.gSharedBuffer.as_registration_ptr()),
        );

        // Load external vehicle data
        BG_VehicleLoadParms(&mut ctx.world.bg_state, &bg_traps);

        G_Printf(ctx, "------- Game Initialization -------\n");
        G_Printf(ctx, "gamename: basejka\n");
        // Raven's `__DATE__` (compile-time C macro): `build.rs` emits it as
        // the `BUILD_DATE` env var (`__DATE__` format, computed at build time).
        G_Printf(
            ctx,
            &format!(
                "gamedate: {}\n",
                option_env!("BUILD_DATE").unwrap_or("")
            ),
        );

        // Raven `srand( randomSeed )` — resolves to `bg_lib.c`'s own
        // `srand`/`rand` pair (not the platform libc), which seed the
        // `randSeed` LCG dozens of gameplay sites read via bare `rand()`
        // (ai_main.c bot aim jitter, g_client.c/g_items.c/g_team.c/g_utils.c
        // spawn-choice picks). That LCG is `BgState::rng`'s `randSeed` field
        // (`bg_channel/rng.rs`); reseed it here.
        ctx.world.bg_state.rng.srand(args.random_seed() as c_uint);

        G_RegisterCvars(ctx);

        // Raven: `//G_ProcessIPBans();` — already commented out in the oracle.
        G_LoadIPBans(ctx);

        G_InitMemory(ctx);

        // set some level globals
        ctx.world.level = *native_platform::zeroed_box::<level_locals_t>();
        ctx.world.level.time = args.level_time();
        ctx.world.level.startTime = args.level_time();

        ctx.world.level.snd_fry = G_SoundIndex(c"sound/player/fry.wav".as_ptr()); // FIXME standing in lava / slime

        ctx.world.level.snd_hack = G_SoundIndex(c"sound/player/hacking.wav".as_ptr());
        ctx.world.level.snd_medHealed = G_SoundIndex(c"sound/player/supp_healed.wav".as_ptr());
        ctx.world.level.snd_medSupplied = G_SoundIndex(c"sound/player/supp_supplied.wav".as_ptr());

        // Raven: `//trap_SP_RegisterServer("mp_svgame");` — already commented
        // out in the oracle.

        // Raven guards this block with `#ifndef _XBOX` — live on this build.
        if ctx.world.cvars.g_log.string[0] != 0 {
            let mode = if ctx.world.cvars.g_logSync.integer != 0 {
                FS_APPEND_SYNC
            } else {
                FS_APPEND
            };
            let log_path = cstr_to_str(ctx.world.cvars.g_log.string.as_ptr());
            let _ = trap::FS_FOpenFile(
                ctx.engine,
                &log_path,
                &mut ctx.world.level.logFile,
                mode,
            );
            if ctx.world.level.logFile == 0 {
                G_Printf(
                    ctx,
                    &format!(
                        "WARNING: Couldn't open logfile: {}\n",
                        cstr_to_str(ctx.world.cvars.g_log.string.as_ptr())
                    ),
                );
            } else {
                let serverinfo = trap::GetServerinfo(ctx.engine, MAX_INFO_STRING);

                G_LogPrintf(
                    ctx,
                    "------------------------------------------------------------\n",
                );
                G_LogPrintf(ctx, &format!("InitGame: {}\n", serverinfo));
            }
        } else {
            G_Printf(ctx, "Not logging to disk.\n");
        }

        G_LogWeaponInit(ctx);

        G_InitWorldSession(ctx);

        // initialize all entities for this game
        core::ptr::write_bytes(ctx.world.g_entities.as_mut_ptr(), 0, MAX_GENTITIES);
        // The byte-wise zero above (Raven `memset(g_entities, 0, ...)`, g_main.c)
        // leaves every entity's FnId<EntXxx> handler fields as None by
        // construction (zero == None, std-guaranteed via Option<NonZeroU8>).
        ctx.world.level.gentities = ctx.world.g_entities.as_mut_ptr();

        // initialize all clients for this game
        ctx.world.level.maxclients = ctx.world.cvars.g_maxclients.integer;
        core::ptr::write_bytes(ctx.world.clients.as_mut_ptr(), 0, MAX_CLIENTS);
        ctx.world.level.clients = ctx.world.clients.as_mut_ptr();

        // set client fields on player ents
        for i in 0..(ctx.world.level.maxclients as usize) {
            ctx.world.g_entities[i].client = ctx.world.clients.as_mut_ptr().add(i);
        }

        // always leave room for the max number of clients,
        // even if they aren't all used, so numbers inside that
        // range are NEVER anything but clients
        ctx.world.level.num_entities = MAX_CLIENTS as c_int;

        // let the server system know where the entites are
        let entities_base = ctx.world.g_entities.as_mut_ptr();
        let clients_base = &mut ctx.world.clients[0] as *mut gclient_t as *mut playerState_t;
        trap::LocateGameData(
            ctx.engine,
            GLocateGameDataArgs::new(
                entities_base.cast(),
                ctx.world.level.num_entities,
                core::mem::size_of::<gentity_t>() as c_int,
                clients_base,
                core::mem::size_of::<gclient_t>() as c_int,
            ),
        );

        // Load sabers.cfg data
        WP_SaberLoadParms(&mut ctx.world.bg_state, &bg_traps);

        NPC_InitGame(ctx);

        TIMER_Clear(ctx);
        //
        //ICARUS INIT START

        trap::ICARUS_Init(ctx.engine, GIcarusInitArgs::new());

        //ICARUS INIT END
        //

        // reserve some spots for dead player bodies
        InitBodyQue(ctx);

        ClearRegisteredItems(ctx);

        //make sure saber data is loaded before this! (so we can precache the appropriate hilts)
        InitSiegeMode(ctx);

        let mut mapname = vmCvar_t::zeroed();
        let mut ck_sum = vmCvar_t::zeroed();
        trap::Cvar_Register(
            ctx.engine,
            Some(&mut mapname),
            "mapname",
            "",
            CVAR_SERVERINFO | CVAR_ROM,
        );
        trap::Cvar_Register(
            ctx.engine,
            Some(&mut ck_sum),
            "sv_mapChecksum",
            "",
            CVAR_ROM,
        );

        let mapname_str = unsafe { cstr_to_str(mapname.string.as_ptr()) };
        let nav_loaded = trap::Nav_Load(ctx.engine, &mapname_str, ck_sum.integer);
        ctx.world.globals.navCalculatePaths = if !nav_loaded { qtrue } else { qfalse };

        // parse the key/value pairs and spawn gentities
        G_SpawnEntitiesFromString(ctx, qfalse);

        // general initialization
        G_FindTeams(ctx);

        // make sure we have flags for CTF, etc
        if ctx.world.cvars.g_gametype.integer >= GT_TEAM {
            G_CheckTeamItems(ctx);
        } else if ctx.world.cvars.g_gametype.integer == GT_JEDIMASTER {
            trap::SetConfigstring(ctx.engine, CS_CLIENT_JEDIMASTER, "-1");
        }

        if ctx.world.cvars.g_gametype.integer == GT_POWERDUEL {
            trap::SetConfigstring(ctx.engine, CS_CLIENT_DUELISTS, "-1|-1|-1");
        } else {
            trap::SetConfigstring(ctx.engine, CS_CLIENT_DUELISTS, "-1|-1");
        }
        // nmckenzie: DUEL_HEALTH: Default.
        trap::SetConfigstring(ctx.engine, CS_CLIENT_DUELHEALTHS, "-1|-1|!");
        trap::SetConfigstring(ctx.engine, CS_CLIENT_DUELWINNER, "-1");

        SaveRegisteredItems(ctx);

        // Raven: `//G_Printf ("-----------------------------------\n");` —
        // already commented out in the oracle.

        if ctx.world.cvars.g_gametype.integer == GT_SINGLE_PLAYER
            || trap::Cvar_VariableIntegerValue(ctx.engine, "com_buildScript") != 0
        {
            G_ModelIndex(SP_PODIUM_MODEL.as_ptr());
            G_SoundIndex(c"sound/player/gurp1.wav".as_ptr());
            G_SoundIndex(c"sound/player/gurp2.wav".as_ptr());
        }

        if trap::Cvar_VariableIntegerValue(ctx.engine, "bot_enable") != 0 {
            BotAISetup(ctx, args.restart());
            BotAILoadMap(ctx, args.restart());
            G_InitBots(ctx, args.restart());
        }

        G_RemapTeamShaders();

        if ctx.world.cvars.g_gametype.integer == GT_DUEL
            || ctx.world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            G_LogPrintf(
                ctx,
                &format!(
                    "Duel Tournament Begun: kill limit {}, win limit: {}\n",
                    ctx.world.cvars.g_fraglimit.integer, ctx.world.cvars.g_duel_fraglimit.integer
                ),
            );
        }

        if ctx.world.globals.navCalculatePaths != qfalse {
            //not loaded - need to calc paths
            ctx.world.globals.navCalcPathTime = ctx.world.level.time + START_TIME_NAV_CALC;
        //make sure all ents are in and linked
        } else {
            //loaded
            trap::Nav_SetPathsCalculated(ctx.engine, GNavSetpathscalculatedArgs::new(qtrue));
            //need to do this, because combatpoint waypoints aren't saved out...?
            CP_FindCombatPointWaypoints(ctx);
            ctx.world.globals.navCalcPathTime = 0;
            // Raven's commented-out `eSavedGameJustLoaded` failed-edge clear
            // is SP-only ("No loading games in MP.").
        }

        if ctx.world.cvars.g_gametype.integer == GT_SIEGE {
            //just get these configstrings registered now...
            let names = &mp_bg::local::bg_customSiegeSoundNames;
            let mut i: usize = 0;
            while i < names.len() {
                match names[i] {
                    Some(name) => {
                        G_SoundIndex(name.as_ptr());
                    }
                    None => break,
                }
                i += 1;
            }
        }
    }
}
