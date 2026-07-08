//! `g_init_game` — Raven `G_InitGame` (`g_main.c:897-1118`).
#![allow(non_snake_case, unused, clippy::all)]

use core::ffi::CStr;

use crate::prelude::*;

use crate::ai_main::{BotAILoadMap, BotAISetup};
use crate::ai_util::B_InitAlloc;
use crate::bg_channel::{GameBgTraps, GameCallbacksImpl};
use crate::bg_saberLoad::WP_SaberLoadParms;
use crate::bg_vehicleLoad::BG_VehicleLoadParms;
use crate::g_bot::G_InitBots;
use crate::g_client::InitBodyQue;
use crate::g_items::{ClearRegisteredItems, G_CheckTeamItems, SaveRegisteredItems};
use crate::g_log::G_LogWeaponInit;
use crate::g_main::{G_FindTeams, G_LogPrintf, G_Printf, G_RegisterCvars, G_RemapTeamShaders};
use crate::g_mem::G_InitMemory;
use crate::g_saga::InitSiegeMode;
use crate::g_session::G_InitWorldSession;
use crate::g_spawn::G_SpawnEntitiesFromString;
use crate::g_local_consts::{SP_PODIUM_MODEL, START_TIME_NAV_CALC};
use crate::g_svcmds::G_LoadIPBans;
use crate::g_timer::TIMER_Clear;
use crate::g_utils::{G_ModelIndex, G_SoundIndex};
use crate::level::level_locals::level_locals_t;
use crate::npc_c::NPC_InitGame;
use crate::NPC_combat::CP_FindCombatPointWaypoints;

// Explicit import: `mp_bg::public::configstring::*` and `crate::g_client::*`
// both glob-export `CS_CLIENT_JEDIMASTER` (prelude ambiguous-glob-reexports
// warning); an explicit import resolves the ambiguity for this file, matching
// the g_main.rs precedent for the same configstring family.
use mp_bg::public::configstring::CS_CLIENT_JEDIMASTER;

use mp_abi::game::vmcalls::GAME_INIT::GameInitArgs;

use mp_abi::game::syscalls::G_CVAR_REGISTER::GCvarRegisterArgs;
use mp_abi::game::syscalls::G_CVAR_SET::GCvarSetArgs;
use mp_abi::game::syscalls::G_CVAR_VARIABLE_INTEGER_VALUE::GCvarVariableIntegerValueArgs;
use mp_abi::game::syscalls::G_FS_FOPEN_FILE::GFsFopenFileArgs;
use mp_abi::game::syscalls::G_G2_CLEANENTATTACHMENTS::GG2CleanentattachmentsArgs;
use mp_abi::game::syscalls::G_GET_SERVERINFO::GGetServerinfoArgs;
use mp_abi::game::syscalls::G_ICARUS_INIT::GIcarusInitArgs;
use mp_abi::game::syscalls::G_LOCATE_GAME_DATA::GLocateGameDataArgs;
use mp_abi::game::syscalls::G_NAV_LOAD::GNavLoadArgs;
use mp_abi::game::syscalls::G_NAV_SETPATHSCALCULATED::GNavSetpathscalculatedArgs;
use mp_abi::game::syscalls::G_SET_CONFIGSTRING::GSetConfigstringArgs;
use mp_abi::game::syscalls::G_SET_SHARED_BUFFER::GSetSharedBufferArgs;

// PORT-NOTE(unported-const): `MAX_INFO_STRING` has no ported home; 1024 is the
// oracle's usual value (matches the g_client.rs/g_misc.rs/g_bot.rs precedent).
// Source: `oracle/oracle/codemp/game/q_shared.h:384`
const MAX_INFO_STRING: usize = 1024;

/// Raven `void G_InitGame( int levelTime, int randomSeed, int restart )`.
///
/// Source: `oracle/oracle/codemp/game/g_main.c:897-1118`
pub fn g_init_game(ctx: GameContext<'_>, args: GameInitArgs) {
    // Arm the ctx-less `strap_*` seam engine cell from the GAME_INIT
    // entrypoint that owns the engine, before any bg logic can call `strap_*`.
    crate::g_strap::init_strap_engine(ctx.engine);

    unsafe {
        let world = &mut *ctx.world;

        // Raven: `#ifdef _XBOX` guards `BG_ClearVehicleParseParms()` +
        // `RemoveAllWP()` here (`g_main.c:902-907`) — dead branch on this
        // (non-Xbox) build, dropped per the `_XBOX`-branch-drop precedent
        // (`q_math.rs`, `NPC_utils.rs`).

        // Init RMG to 0, it will be autoset to 1 if there is terrain on the level.
        trap::Cvar_Set(ctx.engine, GCvarSetArgs::new(cstr("RMG"), cstr("0")));
        world.cvars.g_RMG.integer = 0;

        // Clean up any client-server ghoul2 instance attachments that may
        // still exist exe-side.
        trap::G2API_CleanEntAttachments(ctx.engine, GG2CleanentattachmentsArgs::new());

        // The bg-channel handles this transcription threads through the
        // BgTraps/GameCallbacks seam (bg has no `Engine`/`GameContext`).
        let bg_traps = GameBgTraps::new(ctx.engine);

        // BG_InitAnimsets(); //clear it out
        {
            let mut callbacks = GameCallbacksImpl {
                world: ctx.world,
                engine: ctx.engine,
            };
            let mut pmc = PmoveContext::new(&mut world.bg_state, &bg_traps, &mut callbacks);
            pmc.BG_InitAnimsets();
        }

        B_InitAlloc(ctx); //make sure everything is clean

        trap::SV_RegisterSharedMemory(
            ctx.engine,
            GSetSharedBufferArgs::new(world.gSharedBuffer.as_mut_ptr() as *mut c_char),
        );

        // Load external vehicle data
        BG_VehicleLoadParms(&mut world.bg_state, &bg_traps);

        G_Printf(ctx, c"------- Game Initialization -------\n".as_ptr());
        G_Printf(ctx, c"gamename: basejka\n".as_ptr());
        // Raven's `__DATE__` (compile-time C macro) has no faithful Rust
        // reproduction; left empty (matches the oracle Rust port's own
        // `gamedate` handling, `oracle/src/codemp/game/g_main.rs:4245`).
        G_Printf(ctx, c"gamedate: \n".as_ptr());

        // Raven `srand( randomSeed )` — resolves to `bg_lib.c`'s own
        // `srand`/`rand` pair (not the platform libc), which seed the
        // `randSeed` LCG dozens of gameplay sites read via bare `rand()`
        // (ai_main.c bot aim jitter, g_client.c/g_items.c/g_team.c/g_utils.c
        // spawn-choice picks). That LCG is `BgState::rng`'s `randSeed` field
        // (`bg_channel/rng.rs`); reseed it here.
        world.bg_state.rng.srand(args.random_seed() as c_uint);

        G_RegisterCvars(ctx);

        // Raven: `//G_ProcessIPBans();` — already commented out in the oracle.
        G_LoadIPBans(ctx);

        G_InitMemory(ctx);

        // set some level globals
        world.level = *native_platform::zeroed_box::<level_locals_t>();
        world.level.time = args.level_time();
        world.level.startTime = args.level_time();

        world.level.snd_fry = G_SoundIndex(c"sound/player/fry.wav".as_ptr()); // FIXME standing in lava / slime

        world.level.snd_hack = G_SoundIndex(c"sound/player/hacking.wav".as_ptr());
        world.level.snd_medHealed = G_SoundIndex(c"sound/player/supp_healed.wav".as_ptr());
        world.level.snd_medSupplied = G_SoundIndex(c"sound/player/supp_supplied.wav".as_ptr());

        // Raven: `//trap_SP_RegisterServer("mp_svgame");` — already commented
        // out in the oracle.

        // Raven guards this block with `#ifndef _XBOX` — live on this build.
        if world.cvars.g_log.string[0] != 0 {
            let mode = if world.cvars.g_logSync.integer != 0 {
                FS_APPEND_SYNC
            } else {
                FS_APPEND
            };
            let log_path = CStr::from_ptr(world.cvars.g_log.string.as_ptr()).to_owned();
            let _ = trap::FS_FOpenFile(
                ctx.engine,
                GFsFopenFileArgs::new(
                    log_path,
                    &mut world.level.logFile as *mut fileHandle_t,
                    mode,
                ),
            );
            if world.level.logFile == 0 {
                G_Printf(
                    ctx,
                    cstr(&format!(
                        "WARNING: Couldn't open logfile: {}\n",
                        cstr_to_str(world.cvars.g_log.string.as_ptr())
                    ))
                    .as_ptr(),
                );
            } else {
                let mut serverinfo: [c_char; MAX_INFO_STRING] = [0; MAX_INFO_STRING];
                trap::GetServerinfo(
                    ctx.engine,
                    GGetServerinfoArgs::new(serverinfo.as_mut_ptr(), MAX_INFO_STRING as c_int),
                );

                G_LogPrintf(
                    ctx,
                    cstr("------------------------------------------------------------\n")
                        .as_ptr(),
                );
                G_LogPrintf(
                    ctx,
                    cstr(&format!("InitGame: {}\n", cstr_to_str(serverinfo.as_ptr()))).as_ptr(),
                );
            }
        } else {
            G_Printf(ctx, c"Not logging to disk.\n".as_ptr());
        }

        G_LogWeaponInit(ctx);

        G_InitWorldSession(ctx);

        // initialize all entities for this game
        core::ptr::write_bytes(world.g_entities.as_mut_ptr(), 0, MAX_GENTITIES);
        // Niche-layout fixup (interim, see gentity_t::reset_fn_ids_after_zero):
        // the byte-wise zero above (Raven `memset(g_entities, 0, ...)`,
        // g_main.c) leaves each entity's Option<EntXxx> fn-ID fields decoding
        // as Some(variant 0) instead of None (no reserved 0 in those enums;
        // Option's None niche sits AFTER the last variant). C NULL-fn-pointer
        // semantics require None; assign it explicitly on every slot.
        for ent in world.g_entities.iter_mut() {
            ent.reset_fn_ids_after_zero();
        }
        world.level.gentities = world.g_entities.as_mut_ptr();

        // initialize all clients for this game
        world.level.maxclients = world.cvars.g_maxclients.integer;
        core::ptr::write_bytes(world.clients.as_mut_ptr(), 0, MAX_CLIENTS);
        world.level.clients = world.clients.as_mut_ptr();

        // set client fields on player ents
        for i in 0..(world.level.maxclients as usize) {
            world.g_entities[i].client = world.clients.as_mut_ptr().add(i) as *mut c_void;
        }

        // always leave room for the max number of clients,
        // even if they aren't all used, so numbers inside that
        // range are NEVER anything but clients
        world.level.num_entities = MAX_CLIENTS as c_int;

        // let the server system know where the entites are
        let entities_base = world.g_entities.as_mut_ptr();
        let clients_base = &mut world.clients[0] as *mut gclient_t as *mut playerState_t;
        trap::LocateGameData(
            ctx.engine,
            GLocateGameDataArgs::new(
                entities_base,
                world.level.num_entities,
                core::mem::size_of::<gentity_t>() as c_int,
                clients_base,
                core::mem::size_of::<gclient_t>() as c_int,
            ),
        );

        // Load sabers.cfg data
        WP_SaberLoadParms(&mut world.bg_state, &bg_traps);

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
            GCvarRegisterArgs::new(
                &mut mapname as *mut vmCvar_t,
                cstr("mapname"),
                cstr(""),
                CVAR_SERVERINFO | CVAR_ROM,
            ),
        );
        trap::Cvar_Register(
            ctx.engine,
            GCvarRegisterArgs::new(
                &mut ck_sum as *mut vmCvar_t,
                cstr("sv_mapChecksum"),
                cstr(""),
                CVAR_ROM,
            ),
        );

        let mapname_cstr = CStr::from_ptr(mapname.string.as_ptr()).to_owned();
        let nav_loaded = trap::Nav_Load(ctx.engine, GNavLoadArgs::new(mapname_cstr, ck_sum.integer));
        world.globals.navCalculatePaths = if nav_loaded == qfalse { qtrue } else { qfalse };

        // parse the key/value pairs and spawn gentities
        G_SpawnEntitiesFromString(ctx, qfalse);

        // general initialization
        G_FindTeams(ctx);

        // make sure we have flags for CTF, etc
        if world.cvars.g_gametype.integer >= GT_TEAM {
            G_CheckTeamItems(ctx);
        } else if world.cvars.g_gametype.integer == GT_JEDIMASTER {
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(CS_CLIENT_JEDIMASTER, cstr("-1")),
            );
        }

        if world.cvars.g_gametype.integer == GT_POWERDUEL {
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(CS_CLIENT_DUELISTS, cstr("-1|-1|-1")),
            );
        } else {
            trap::SetConfigstring(
                ctx.engine,
                GSetConfigstringArgs::new(CS_CLIENT_DUELISTS, cstr("-1|-1")),
            );
        }
        // nmckenzie: DUEL_HEALTH: Default.
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_CLIENT_DUELHEALTHS, cstr("-1|-1|!")),
        );
        trap::SetConfigstring(
            ctx.engine,
            GSetConfigstringArgs::new(CS_CLIENT_DUELWINNER, cstr("-1")),
        );

        SaveRegisteredItems(ctx);

        // Raven: `//G_Printf ("-----------------------------------\n");` —
        // already commented out in the oracle.

        if world.cvars.g_gametype.integer == GT_SINGLE_PLAYER
            || trap::Cvar_VariableIntegerValue(
                ctx.engine,
                GCvarVariableIntegerValueArgs::new(cstr("com_buildScript")),
            ) != 0
        {
            G_ModelIndex(SP_PODIUM_MODEL.as_ptr());
            G_SoundIndex(c"sound/player/gurp1.wav".as_ptr());
            G_SoundIndex(c"sound/player/gurp2.wav".as_ptr());
        }

        if trap::Cvar_VariableIntegerValue(
            ctx.engine,
            GCvarVariableIntegerValueArgs::new(cstr("bot_enable")),
        ) != 0
        {
            BotAISetup(ctx, args.restart());
            BotAILoadMap(ctx, args.restart());
            G_InitBots(ctx, args.restart());
        }

        G_RemapTeamShaders();

        if world.cvars.g_gametype.integer == GT_DUEL || world.cvars.g_gametype.integer == GT_POWERDUEL
        {
            G_LogPrintf(
                ctx,
                cstr(&format!(
                    "Duel Tournament Begun: kill limit {}, win limit: {}\n",
                    world.cvars.g_fraglimit.integer, world.cvars.g_duel_fraglimit.integer
                ))
                .as_ptr(),
            );
        }

        if world.globals.navCalculatePaths != qfalse {
            //not loaded - need to calc paths
            world.globals.navCalcPathTime = world.level.time + START_TIME_NAV_CALC; //make sure all ents are in and linked
        } else {
            //loaded
            trap::Nav_SetPathsCalculated(ctx.engine, GNavSetpathscalculatedArgs::new(qtrue));
            //need to do this, because combatpoint waypoints aren't saved out...?
            CP_FindCombatPointWaypoints(ctx);
            world.globals.navCalcPathTime = 0;
            // Raven's commented-out `eSavedGameJustLoaded` failed-edge clear
            // is SP-only ("No loading games in MP.").
        }

        if world.cvars.g_gametype.integer == GT_SIEGE {
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
