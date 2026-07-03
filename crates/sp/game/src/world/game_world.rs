//! SP `GameWorld` — the one owned module-island instance (STATE-D1/D9 SP mirror).

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::shared::MAX_GENTITIES;

use crate::local::level_locals_t::level_locals_t;

/// The SP module island: a value type owned by the module crate, NOT a global.
/// Field types are the EXISTING Raven-faithful, offset-asserted structs (§D12).
/// SP has **no `clients` box** — `g_clients` does not exist in the SP tree
/// (`MAX_CLIENTS = 1`; divergence preserved, STATE-D2/DEC-04).
///
/// Source: `oracle/oracle/code/game/g_main.cpp:46,49` (`level`, `g_entities`).
pub struct GameWorld {
    /// `level` (`level_locals_t`, `g_main.cpp:46`).
    pub level: level_locals_t,
    /// `g_entities[MAX_GENTITIES]` (`g_main.cpp:49`; contiguous `#[repr(C)]`).
    pub entities: Box<[gentity_t; MAX_GENTITIES]>,
}

impl GameWorld {
    /// Zeroed heap construction (STATE-D9 SP mirror, via
    /// `native_platform::zeroed_box`); back-pointer wiring runs in `InitGame`'s
    /// export body, allocate-first order — not here.
    ///
    /// Source: `docs/architecture/state-ownership.md` § `GameWorld::zeroed` (STATE-D9).
    pub fn zeroed() -> Self {
        //TODO: Port GameWorld::zeroed (SP) — native_platform::zeroed_box for entities/level
        // Source: oracle/oracle/code/game/g_main.cpp:715-749 (InitGame entity block)
        todo!("Port SP GameWorld::zeroed — oracle/oracle/code/game/g_main.cpp:715-749 (STATE-D9)")
    }
}
