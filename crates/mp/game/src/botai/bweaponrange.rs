#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `BWEAPONRANGE_*` — bot preferred-engagement-range classes for the
/// current weapon.
///
/// Raven: plain `#define`s, not an enum; ported as loose consts.
/// Source: `oracle/codemp/game/ai_main.h:46-49`
pub const BWEAPONRANGE_MELEE: c_int = 1;
pub const BWEAPONRANGE_MID: c_int = 2;
pub const BWEAPONRANGE_LONG: c_int = 3;
pub const BWEAPONRANGE_SABER: c_int = 4;
