//! `GameWorld` — the one owned module-island instance (STATE-D1/D9, FROZEN).

use mp_qshared::common::mp::gentity_t;
use mp_qshared::shared::{MAX_CLIENTS, MAX_GENTITIES};

use crate::client::gclient_t;
use crate::level::level_locals::level_locals_t;

/// A value type owned by the module crate. NOT a global. Field types are the
/// EXISTING Raven-faithful, already-offset-asserted structs (§D12) — exactly the
/// structs the raw `LocateGameData` seam aliases into.
///
/// Source: `docs/architecture/state-ownership.md` § `GameWorld` (STATE-D1).
pub struct GameWorld {
    /// `level` (`level_locals_t`, `g_main.c:9`).
    pub level: level_locals_t,
    /// `g_entities[MAX_GENTITIES]` (`g_main.c:27`; contiguous `#[repr(C)]`,
    /// size-asserted 1832 B).
    pub entities: Box<[gentity_t; MAX_GENTITIES]>,
    /// `g_clients[MAX_CLIENTS]` (reached as `level.clients`, `g_main.c:28`;
    /// asserted 7344 B). MP only.
    pub clients: Box<[gclient_t; MAX_CLIENTS]>,
}

impl GameWorld {
    /// Builds the zeroed island (STATE-D9), then wires `level`'s
    /// self-referencing back-pointers in the allocate-first order — the latter in
    /// `G_InitGame`'s dispatched arm, not here. Uses `native_platform::zeroed_box`
    /// for the ~1.83 MB entity array (heap-built, never transits the stack).
    ///
    /// Source: `docs/architecture/state-ownership.md` § `GameWorld::zeroed` (STATE-D9).
    pub fn zeroed() -> Self {
        // The frozen STATE-D9 sketch verbatim: zeroed heap boxes first; the
        // level.gentities/clients + entities[i].client back-pointers alias them
        // AFTER they exist, in G_InitGame's dispatched arm (g_main.c:978-988) —
        // not here.
        let entities = native_platform::zeroed_box::<[gentity_t; MAX_GENTITIES]>();
        let clients = native_platform::zeroed_box::<[gclient_t; MAX_CLIENTS]>();
        let level = *native_platform::zeroed_box::<level_locals_t>();
        GameWorld {
            level,
            entities,
            clients,
        }
    }
}
