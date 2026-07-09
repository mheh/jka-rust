#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::ffi::c_int;

/// Raven `weapon_t` — weapon type identifier (int-based alias with enum-like consts).
///
/// Type definition source: `oracle/codemp/game/bg_weapons.h:8-40`
#[allow(non_camel_case_types)]
pub type weapon_t = c_int;

pub const WP_NONE: c_int = 0;

pub const WP_STUN_BATON: c_int = 1;
pub const WP_MELEE: c_int = 2;
pub const WP_SABER: c_int = 3;
pub const WP_BRYAR_PISTOL: c_int = 4;
pub const WP_BLASTER: c_int = 5;
pub const WP_DISRUPTOR: c_int = 6;
pub const WP_BOWCASTER: c_int = 7;
pub const WP_REPEATER: c_int = 8;
pub const WP_DEMP2: c_int = 9;
pub const WP_FLECHETTE: c_int = 10;
pub const WP_ROCKET_LAUNCHER: c_int = 11;
pub const WP_THERMAL: c_int = 12;
pub const WP_TRIP_MINE: c_int = 13;
pub const WP_DET_PACK: c_int = 14;
pub const WP_CONCUSSION: c_int = 15;
pub const WP_BRYAR_OLD: c_int = 16;
pub const WP_EMPLACED_GUN: c_int = 17;
pub const WP_TURRET: c_int = 18;

pub const WP_NUM_WEAPONS: c_int = 19;
