//! `CgWeaponsState` — `cg_weapons.c`'s mutable file-scope globals as one `CgWorld`
//! sub-struct.

#![allow(non_snake_case)]

use core::ffi::{c_int, c_void};
use core::ptr::null_mut;

use mp_qshared::common::mp::qcommon::player_state::MAX_WEAPONS;

/// `cg_weapons.c`'s mutable file-scope globals, grouped by owning `.c` file
/// (§B3: file-scope globals become owned state, they never become Rust
/// globals). Raven's read-only tables beside them are compiled-in data, not
/// state; they land as `const`s beside the functions that read them (§C8).
///
/// Source: `oracle/codemp/cgame/cg_weapons.c:130-131,2322`
#[derive(Debug, Clone)]
pub struct CgWeaponsState {
    /// Raven `static int cgWeapFrame` — the busy-holster weapon frame latch.
    ///
    /// Raven: "rww - this was done as a last resort. Forgive me."
    /// Source: `oracle/codemp/cgame/cg_weapons.c:130`
    pub cgWeapFrame: c_int,

    /// Raven `static int cgWeapFrameTime` — when [`Self::cgWeapFrame`] may next
    /// advance.
    /// Source: `oracle/codemp/cgame/cg_weapons.c:131`
    pub cgWeapFrameTime: c_int,

    /// Raven `static void *g2WeaponInstances[MAX_WEAPONS]` — one ghoul2
    /// instance per weapon, built once at init and copied into each client's
    /// gun object. Opaque engine tokens, never dereferenced module-side
    /// (DEC-46.2).
    ///
    /// Raven: "create one instance of all the weapons we are going to use so we
    /// can just copy this info into each clients gun ghoul2 object in fast way".
    /// Source: `oracle/codemp/cgame/cg_weapons.c:2322`
    pub g2WeaponInstances: [*mut c_void; MAX_WEAPONS],
}

impl Default for CgWeaponsState {
    /// Raven's BSS start: both latches 0 and every ghoul2 slot NULL. Hand-written
    /// because a raw-pointer array has no derivable `Default`.
    fn default() -> Self {
        CgWeaponsState {
            cgWeapFrame: 0,
            cgWeapFrameTime: 0,
            g2WeaponInstances: [null_mut(); MAX_WEAPONS],
        }
    }
}
