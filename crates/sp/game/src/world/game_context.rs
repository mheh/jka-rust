//! SP `GameContext` — the module-side receiver mirror (settled SP mapping,
//! 2026-07-03 skeleton-findings).
//!
//! SP has **no `vmMain`, no command decode, no `Dispatch<C>` routing** — the
//! `game_export_t` fn pointers are the entry surface. Each export fn derives
//! its own `*mut GameWorld` from the jagame shell's SP `WORLD` cell in its
//! prologue and constructs this context itself (per-export construction
//! replaces MP's once-per-vmMain construction).

use sp_abi::game::public::game_import_t::game_import_t;

use super::game_world::GameWorld;

/// The copyable per-export receiver, mirroring MP's `GameContext` (SEAM-Q12).
/// Built as a plain struct literal in each `game_export_t` export fn's prologue
/// by the jagame shell, from the SP `WORLD` cell + the stored import table;
/// fields are `pub` per the round-5 resolution (a `Copy` struct of raw pointers
/// has no invariant to protect; the `WorldPtr` precedent, STATE-D8). The SP
/// engine handle is the stored `game_import_t` the engine passed into
/// `GetGameAPI` (Raven `gi = *import`, `g_main.cpp:878`) — bound directly per
/// SEAM-D2 (SP has no `mp_engine_select`-style alias crate; no alias name is
/// minted here, see skeleton checkpoint-3 findings).
///
/// Source: `oracle/oracle/code/game/g_public.h:168-471` (the import table);
/// `docs/handoffs/2026-07-03-skeleton-findings.md` § Settled fork from the
/// round-4 gate.
#[derive(Clone, Copy)]
pub struct GameContext<'e> {
    pub world: *mut GameWorld,
    pub engine: &'e game_import_t,
}

// The per-export logic entry points (InitGame, G_RunFrame, ClientThink, …) that
// take this context are logic-port work, not frozen skeleton surface. Each
// unpacks `self.world` via STATE-D6 leaf reborrows and threads `self.engine`
// into `gi::X(engine, …)` call sites.
//TODO: Port SP export logic fns taking GameContext (per-export, logic-port)
// Source: oracle/oracle/code/game/g_main.cpp:875-916 (the export table fills)
