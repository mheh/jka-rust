//! `GameWorld` — the one owned module-island instance (STATE-D1/D9, FROZEN).

use core::ffi::c_int;

use mp_qshared::common::mp::gentity_t;
use mp_qshared::shared::{MAX_CLIENTS, MAX_GENTITIES};

use crate::client::gclient_t;
use crate::game_cvars::GameCvars;
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
    /// Raven's ~136 file-scope `vmCvar_t` cvar handles, grouped as one
    /// GameWorld sub-struct (fork ruling 1: file-scope globals become GameWorld
    /// fields). Not part of the `LocateGameData` alias set.
    /// Source: `oracle/oracle/codemp/game/g_main.c:230-475`
    pub cvars: GameCvars,

    /// `w_force.c` file-scope loop-sound handles (fork ruling 1: file-scope
    /// mutable globals become GameWorld fields, grouped by owning .c file).
    /// Cached `G_SoundIndex` results, lazily filled in `WP_InitForcePowers`.
    /// Source: `oracle/oracle/codemp/game/w_force.c:24-34`
    pub speedLoopSound: c_int,
    pub rageLoopSound: c_int,
    pub protectLoopSound: c_int,
    pub absorbLoopSound: c_int,
    pub seeLoopSound: c_int,
    pub ysalamiriLoopSound: c_int,

    /// `NPC_utils.c` file-scope globals (fork ruling 1: file-scope mutable
    /// globals become GameWorld fields, grouped by owning .c file).
    /// Source: `oracle/oracle/codemp/game/NPC_utils.c:7-9`
    pub teamNumbers: [c_int; 4],
    pub teamStrength: [c_int; 4],
    pub teamCounter: [c_int; 4],

    /// `g_mem.c` file-scope globals (fork ruling 1: file-scope mutable
    /// globals become GameWorld fields, grouped by owning .c file).
    /// Memory pool for G_Alloc (256 KB), and current allocation point.
    /// Source: `oracle/oracle/codemp/game/g_mem.c:13-14`
    pub memoryPool: Box<[u8; 262144]>, // 256 * 1024
    pub allocPoint: c_int,
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
        let memoryPool = native_platform::zeroed_box::<[u8; 262144]>();
        GameWorld {
            level,
            entities,
            clients,
            cvars: GameCvars::default(),
            speedLoopSound: 0,
            rageLoopSound: 0,
            protectLoopSound: 0,
            absorbLoopSound: 0,
            seeLoopSound: 0,
            ysalamiriLoopSound: 0,
            teamNumbers: [0; 4],
            teamStrength: [0; 4],
            teamCounter: [0; 4],
            memoryPool,
            allocPoint: 0,
        }
    }
}
