//! SP `bg_public.h` weapon state enumeration.
//!
//! Type definition source: `oracle/oracle/code/game/bg_public.h:72-80`

#![allow(non_camel_case_types)]

/// Raven `weaponstate_t` — weapon state.
///
/// Enumeration indicating the current state of a weapon,
/// such as ready, firing, charging, or idle.
/// Type definition source: `oracle/oracle/code/game/bg_public.h:72-80`
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum weaponstate_t {
    WEAPON_READY = 0,
    WEAPON_RAISING = 1,
    WEAPON_DROPPING = 2,
    WEAPON_FIRING = 3,
    WEAPON_CHARGING = 4,
    WEAPON_CHARGING_ALT = 5,
    WEAPON_IDLE = 6,
}
