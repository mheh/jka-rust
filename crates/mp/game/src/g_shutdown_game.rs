//! `g_shutdown_game` — Raven `G_ShutdownGame` (Slice-0 minimal port).

use mp_abi::game::vmcalls::GAME_SHUTDOWN::GameShutdownArgs;

use crate::world::GameContext;

/// Raven `void G_ShutdownGame( int restart )` (`g_main.c:1128`). Slice-0
/// minimal body: nothing observable — Raven's banner is commented out in the
/// oracle (`// G_Printf ("==== ShutdownGame ====\n");`, `g_main.c:1132`) and
/// the teardown (ban-IP save, fake-client cleanup, ghoul2 cleanup) is unported.
/// The world itself is taken out of the shell's `WORLD` cell by the `vmMain`
/// GAME_SHUTDOWN take-side AFTER this dispatch returns (STATE-D6).
///
/// Source: `oracle/oracle/codemp/game/g_main.c:1128-1160`
pub fn g_shutdown_game(ctx: GameContext<'_>, args: GameShutdownArgs) {
    let _ = (ctx, args);
    //TODO: Port G_ShutdownGame body (G_SaveBanIP, fake-client + ghoul2 cleanup)
    // Source: oracle/oracle/codemp/game/g_main.c:1128-1160
}
