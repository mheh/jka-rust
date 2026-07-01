//! MP `bg_public.h` player movement type enumeration.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:360-370`

#![allow(non_camel_case_types)]

/// Raven `pmtype_t` — player movement type.
///
/// Raven: Enumeration indicating the current movement mode or physics state
/// of a player character, ranging from normal movement to spectator to dead.
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:360-370`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum pmtype_t {
    PM_NORMAL = 0,
    PM_JETPACK = 1,
    PM_FLOAT = 2,
    PM_NOCLIP = 3,
    PM_SPECTATOR = 4,
    PM_DEAD = 5,
    PM_FREEZE = 6,
    PM_INTERMISSION = 7,
    PM_SPINTERMISSION = 8,
}
