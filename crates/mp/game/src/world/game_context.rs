//! `GameContext` — the module-side `Dispatch<C>` receiver (SEAM-Q12 RESOLVED,
//! round-4; SUPERSEDES the round-3 `WorldPtr`).

use mp_engine_select::Engine;

use super::game_world::GameWorld;

/// The copyable `Dispatch<C>` receiver each `vmMain` command routes through
/// (SEAM-Q12 resolved 2026-07-03). Defined in `mp_game`; `Engine` is the
/// `mp_engine_select` module-side transport alias (NOT `mp_engine_core::Engine`).
/// `vmMain` constructs one per call — a plain struct literal built by the shell
/// from its `WORLD` + `ENGINE.get()` (SEAM-D1); fields are `pub` per the
/// round-5 resolution (a `Copy` struct of raw pointers has no invariant to
/// protect; the `WorldPtr` precedent, STATE-D8). Each `impl Dispatch<C> for
/// GameContext` unpacks `world` via STATE-D6 leaf reborrows and threads
/// `engine` into the logic fns' `trap::X(engine, …)` call sites (oracle syscall
/// order). The `&Engine` channel is the receiver's own field — no `dispatch`
/// parameter is added, SEAM-D8 stays untouched; the orphan rule is satisfied
/// because `GameContext` and its impls both live in `mp_game`.
///
/// Source: `docs/architecture/engine-seam.md` § inbound dual (SEAM-Q12 amendment 2026-07-03).
#[derive(Clone, Copy)]
pub struct GameContext<'e> {
    pub world: *mut GameWorld,
    pub engine: &'e Engine,
}

// The per-command `impl Dispatch<C> for GameContext` blocks colocate here
// (round-6 pinning: thin adapters only; per-command logic stays
// one-fn-per-file). Each unpacks `self.world` via STATE-D6 leaf reborrows and
// threads `self.engine` into the ported logic fns.

use mp_abi::game::vmcalls::GAME_INIT::{GameInit, GameInitArgs};
use mp_abi::game::vmcalls::GAME_SHUTDOWN::{GameShutdown, GameShutdownArgs};
use mp_abi::Dispatch;

/// `GAME_INIT` → `G_InitGame( arg0, arg1, arg2 )` (`g_main.c:517-519`).
impl Dispatch<GameInit> for GameContext<'_> {
    fn dispatch(&self, args: GameInitArgs) {
        crate::g_init_game::g_init_game(*self, args)
    }
}

/// `GAME_SHUTDOWN` → `G_ShutdownGame( arg0 )` (`g_main.c:520-522`).
impl Dispatch<GameShutdown> for GameContext<'_> {
    fn dispatch(&self, args: GameShutdownArgs) {
        crate::g_shutdown_game::g_shutdown_game(*self, args)
    }
}

//TODO: Port Dispatch<C> for GameContext (remaining MpGameExport commands)
// Source: docs/architecture/engine-seam.md § inbound dual (SEAM-D8)
