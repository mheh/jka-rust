//! MP `bg_public.h` powerup definitions.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:652-684`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `powerup_t`.
///
/// Raven: NOTE: may not have more than 16. Names the powerups via an anonymous
/// `enum { PW_NONE..PW_NUM_POWERUPS }` (several members removed by Raven and left
/// as dead comments), then `typedef int powerup_t` for storage.
/// Type definition source: `oracle/codemp/game/bg_public.h:684`
pub type powerup_t = c_int;

pub const PW_NONE: powerup_t = 0;
pub const PW_QUAD: powerup_t = 1;
pub const PW_BATTLESUIT: powerup_t = 2;
pub const PW_PULL: powerup_t = 3;
pub const PW_REDFLAG: powerup_t = 4;
pub const PW_BLUEFLAG: powerup_t = 5;
pub const PW_NEUTRALFLAG: powerup_t = 6;
pub const PW_SHIELDHIT: powerup_t = 7;
pub const PW_SPEEDBURST: powerup_t = 8;
pub const PW_DISINT_4: powerup_t = 9;
pub const PW_SPEED: powerup_t = 10;
pub const PW_CLOAKED: powerup_t = 11;
pub const PW_FORCE_ENLIGHTENED_LIGHT: powerup_t = 12;
pub const PW_FORCE_ENLIGHTENED_DARK: powerup_t = 13;
pub const PW_FORCE_BOON: powerup_t = 14;
pub const PW_YSALAMIRI: powerup_t = 15;
pub const PW_NUM_POWERUPS: powerup_t = 16;
