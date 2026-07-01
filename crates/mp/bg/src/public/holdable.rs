//! MP `bg_public.h` holdable item definitions.
//!
//! Type definition source: `oracle/oracle/codemp/game/bg_public.h:686-704`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `holdable_t`.
///
/// Raven: NOTE: Names the holdables via an anonymous `enum { HI_NONE..HI_NUM_HOLDABLE }`,
/// then `typedef int holdable_t` for storage.
/// Type definition source: `oracle/oracle/codemp/game/bg_public.h:704`
pub type holdable_t = c_int;

pub const HI_NONE: holdable_t = 0;
pub const HI_SEEKER: holdable_t = 1;
pub const HI_SHIELD: holdable_t = 2;
pub const HI_MEDPAC: holdable_t = 3;
pub const HI_MEDPAC_BIG: holdable_t = 4;
pub const HI_BINOCULARS: holdable_t = 5;
pub const HI_SENTRY_GUN: holdable_t = 6;
pub const HI_JETPACK: holdable_t = 7;
pub const HI_HEALTHDISP: holdable_t = 8;
pub const HI_AMMODISP: holdable_t = 9;
pub const HI_EWEB: holdable_t = 10;
pub const HI_CLOAK: holdable_t = 11;
pub const HI_NUM_HOLDABLE: holdable_t = 12;
