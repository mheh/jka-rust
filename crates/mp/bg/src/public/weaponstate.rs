//! MP `bg_public.h` weapon state enumeration.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:372-380`

#![allow(non_camel_case_types)]

/// Raven `weaponstate_t` — weapon firing and animation state.
///
/// Raven: Enumeration defining the various states a weapon can be in,
/// ranging from ready to firing, charging, and idle.
/// Type definition source: `oracle/codemp/game/bg_public.h:372-380`
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
