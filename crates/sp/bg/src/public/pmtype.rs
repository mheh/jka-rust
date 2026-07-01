//! SP `bg_public.h` player movement type enumeration.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:63-70`

#![allow(non_camel_case_types)]

/// Raven `pmtype_t` — player movement type.
///
/// Enumeration indicating the current movement mode or physics state
/// of a player character, ranging from normal movement to spectator to dead.
/// Type definition source: `oracle/oracle/code/game/bg_public.h:63-70`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum pmtype_t {
    PM_NORMAL = 0,
    PM_NOCLIP = 1,
    PM_SPECTATOR = 2,
    PM_DEAD = 3,
    PM_FREEZE = 4,
    PM_INTERMISSION = 5,
}
