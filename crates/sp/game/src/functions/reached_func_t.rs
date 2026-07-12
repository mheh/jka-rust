#![allow(non_camel_case_types, non_snake_case)]

/// Raven `reachedFunc_t` — enumeration of mover reached-destination callback function IDs.
///
/// Type definition source: `oracle/code/game/g_functions.h:249-258`
#[repr(i32)]
pub enum reachedFunc_t {
    reachedF_NULL = 0,
    //
    reachedF_Reached_BinaryMover,
    reachedF_Reached_Train,
    reachedF_moverCallback,
    reachedF_moveAndRotateCallback,
}
