//! `g_init_game` — Raven `G_InitGame` (Slice-0 minimal port).

use std::ffi::CString;

use mp_abi::game::syscalls::G_PRINT::GPrintArgs;
use mp_abi::game::vmcalls::GAME_INIT::GameInitArgs;

use crate::trap;
use crate::world::GameContext;

/// Raven `void G_InitGame( int levelTime, int randomSeed, int restart )`
/// (`g_main.c:897`). Slice-0 minimal body: the init banner emitted through the
/// real outbound seam (`G_Printf` → `trap_Printf` → `G_PRINT`,
/// `g_main.c:925-927`), in oracle order. The zeroed `GameWorld` was written
/// into the shell's `WORLD` cell by the `vmMain` GAME_INIT bootstrap (STATE-D6)
/// before dispatch reached here.
///
/// Source: `oracle/oracle/codemp/game/g_main.c:897,925-927`
pub fn g_init_game(ctx: GameContext<'_>, args: GameInitArgs) {
    let _ = args;
    // G_Printf banner block (g_main.c:925-927). GAMEVERSION = "basejka"
    // (g_local.h:29); the __DATE__ line carries the port's marker text.
    trap::Printf(
        ctx.engine,
        GPrintArgs::new(CString::new("------- Game Initialization -------\n").unwrap()),
    );
    trap::Printf(
        ctx.engine,
        GPrintArgs::new(CString::new("gamename: basejka\n").unwrap()),
    );
    trap::Printf(
        ctx.engine,
        GPrintArgs::new(CString::new("gamedate: (jka-rust slice 0)\n").unwrap()),
    );
    //TODO: Port G_InitGame body (G_RegisterCvars, level wiring, back-pointers,
    // trap_LocateGameData, trap_SV_RegisterSharedMemory, entity spawn)
    // Source: oracle/oracle/codemp/game/g_main.c:897-1015
}
