//! MP `saber_colors_t` and its color constants.
//!
//! Type definition source: `oracle/codemp/game/q_shared.h:575-588`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `saber_colors_t`.
///
/// Raven names the colors via an anonymous `enum { SABER_RED..NUM_SABER_COLORS }`,
/// then `typedef int saber_colors_t` for storage.
/// Type definition source: `oracle/codemp/game/q_shared.h:588`
pub type saber_colors_t = c_int;

pub const SABER_RED: saber_colors_t = 0;
pub const SABER_ORANGE: saber_colors_t = 1;
pub const SABER_YELLOW: saber_colors_t = 2;
pub const SABER_GREEN: saber_colors_t = 3;
pub const SABER_BLUE: saber_colors_t = 4;
pub const SABER_PURPLE: saber_colors_t = 5;
pub const NUM_SABER_COLORS: saber_colors_t = 6;
