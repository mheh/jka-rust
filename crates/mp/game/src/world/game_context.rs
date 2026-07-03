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

// The per-command `impl Dispatch<C> for GameContext` blocks (one per
// `MpGameExport` variant — GAME_INIT/GAME_RUN_FRAME/…) are logic-port work, not
// frozen skeleton surface. Each unpacks `self.world` via STATE-D6 leaf reborrows
// and threads `self.engine` into the ported logic fns.
//TODO: Port Dispatch<C> for GameContext (per-command, logic-port)
// Source: docs/architecture/engine-seam.md § inbound dual (SEAM-D8)
