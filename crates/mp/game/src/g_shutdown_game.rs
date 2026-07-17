//! `g_shutdown_game` — Raven `G_ShutdownGame`.

use std::ffi::CString;

use mp_abi::game::syscalls::G_CVAR_VARIABLE_INTEGER_VALUE::GCvarVariableIntegerValueArgs;
use mp_abi::game::syscalls::G_FS_FCLOSE_FILE::GFsFcloseFileArgs;
use mp_abi::game::syscalls::G_G2_CLEANMODELS::GG2CleanmodelsArgs;
use mp_abi::game::syscalls::G_G2_HAVEWEGHOULMODELS::GG2HaveweghoulmodelsArgs;
use mp_abi::game::syscalls::G_ICARUS_SHUTDOWN::GIcarusShutdownArgs;
use mp_abi::game::syscalls::G_ROFF_CLEAN::GRoffCleanArgs;
use mp_abi::game::vmcalls::GAME_SHUTDOWN::GameShutdownArgs;
use mp_qshared::shared::MAX_GENTITIES;

use crate::ai_main::BotAIShutdown;
use crate::ai_util::B_CleanupAlloc;
use crate::g_log::G_LogWeaponOutput;
use crate::g_main::G_LogPrintf;
use crate::g_misc::TAG_Init;
use crate::g_session::G_WriteSessionData;
use crate::g_svcmds::G_SaveBanIP;
use crate::g_utils::G_CleanAllFakeClients;
use crate::prelude::*;
use crate::trap;
use mp_bg::bg_panimate::BG_ClearAnimsets;

/// Raven `void G_ShutdownGame( int restart )` (`g_main.c:1128`).
///
/// Raven: the `==== ShutdownGame ====` banner is commented out in the oracle
/// (`g_main.c:1132`), so it is not reproduced here.
///
/// Source: `oracle/codemp/game/g_main.c:1128-1199`
pub fn g_shutdown_game(ctx: &mut GameContext, args: GameShutdownArgs) {
    let restart = args.restart();

    G_SaveBanIP(ctx);
    // get rid of dynamically allocated fake client structs.
    G_CleanAllFakeClients(ctx);

    // free all dynamic allocations made through the engine
    BG_ClearAnimsets();

    // Com_Printf("... Gameside GHOUL2 Cleanup\n");
    for i in 0..MAX_GENTITIES {
        // clean up all the ghoul2 instances. `ghoul2`/`weaponGhoul2[j]` are
        // `*mut c_void` handle values, not arena pointers — read the value out
        // and take field addresses through the owned world accessors (§B5).
        let ghoul2 = ctx.world.g_entities[i].ghoul2;
        if !ghoul2.is_null()
            && trap::G2_HaveWeGhoul2Models(ctx.engine, GG2HaveweghoulmodelsArgs::new(ghoul2))
                != qfalse
        {
            trap::G2API_CleanGhoul2Models(
                ctx.engine,
                GG2CleanmodelsArgs::new(&mut ctx.world.g_entities[i].ghoul2 as *mut *mut c_void),
            );
            ctx.world.g_entities[i].ghoul2 = core::ptr::null_mut();
        }
        // `ent->client` is non-null for player slots (arena `clients[i]`) AND
        // for NPCs at i >= MAX_CLIENTS, whose clients are BG_Alloc'd pool
        // structs (`gClPtrs`) that `G_FreeFakeClient` never frees or nulls —
        // so the weaponGhoul2 reads must go through the entity's client
        // pointer (gclient deref regime, task #7), not `clients[i]`.
        let client = ctx.world.g_entities[i].client;
        if !client.is_null() {
            for j in 0..MAX_SABERS {
                let weapon_ghoul2 = unsafe { (*client).weaponGhoul2[j] };
                if !weapon_ghoul2.is_null()
                    && trap::G2_HaveWeGhoul2Models(
                        ctx.engine,
                        GG2HaveweghoulmodelsArgs::new(weapon_ghoul2),
                    ) != qfalse
                {
                    trap::G2API_CleanGhoul2Models(
                        ctx.engine,
                        GG2CleanmodelsArgs::new(unsafe {
                            core::ptr::addr_of_mut!((*client).weaponGhoul2[j]) as *mut *mut c_void
                        }),
                    );
                }
            }
        }
    }
    if !ctx.world.globals.g2SaberInstance.is_null()
        && trap::G2_HaveWeGhoul2Models(
            ctx.engine,
            GG2HaveweghoulmodelsArgs::new(ctx.world.globals.g2SaberInstance),
        ) != qfalse
    {
        trap::G2API_CleanGhoul2Models(
            ctx.engine,
            GG2CleanmodelsArgs::new(&mut ctx.world.globals.g2SaberInstance as *mut *mut c_void),
        );
        ctx.world.globals.g2SaberInstance = core::ptr::null_mut();
    }
    if !ctx.world.globals.precachedKyle.is_null()
        && trap::G2_HaveWeGhoul2Models(
            ctx.engine,
            GG2HaveweghoulmodelsArgs::new(ctx.world.globals.precachedKyle),
        ) != qfalse
    {
        trap::G2API_CleanGhoul2Models(
            ctx.engine,
            GG2CleanmodelsArgs::new(&mut ctx.world.globals.precachedKyle as *mut *mut c_void),
        );
        ctx.world.globals.precachedKyle = core::ptr::null_mut();
    }

    // Com_Printf ("... ICARUS_Shutdown\n");
    trap::ICARUS_Shutdown(ctx.engine, GIcarusShutdownArgs::new()); // Shut ICARUS down

    // Com_Printf ("... Reference Tags Cleared\n");
    TAG_Init(ctx); // Clear the reference tags

    G_LogWeaponOutput(ctx);

    if ctx.world.level.logFile != 0 {
        G_LogPrintf(ctx, cstr("ShutdownGame:\n").as_ptr());
        G_LogPrintf(
            ctx,
            cstr("------------------------------------------------------------\n").as_ptr(),
        );
        trap::FS_FCloseFile(ctx.engine, GFsFcloseFileArgs::new(ctx.world.level.logFile));
    }

    // write all the client session data so we can get it back
    G_WriteSessionData(ctx);

    trap::ROFF_Clean(ctx.engine, GRoffCleanArgs::new());

    if trap::Cvar_VariableIntegerValue(
        ctx.engine,
        GCvarVariableIntegerValueArgs::new(CString::new("bot_enable").unwrap()),
    ) != 0
    {
        BotAIShutdown(ctx, restart);
    }

    // clean up all allocations made with B_Alloc
    B_CleanupAlloc();
}
