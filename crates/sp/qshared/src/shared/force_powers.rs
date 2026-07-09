#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `forcePowers_t` — force-power index.
///
/// SP declares this as a named `typedef enum`, but two enumerators share value 0
/// (`FP_FIRST` and `FP_HEAL`), which a Rust enum cannot express — so it is modeled
/// as an int alias + `const`s.
///
/// SP-vs-MP: SP has no `FP_TEAM_HEAL`/`FP_TEAM_FORCE`, orders the saber powers
/// differently, and ends at `NUM_FORCE_POWERS == 16` (MP: 18).
///
/// Type definition source: `oracle/code/game/q_shared.h:1538-1559`
pub type forcePowers_t = c_int;

pub const FP_FIRST: forcePowers_t = 0; // marker
pub const FP_HEAL: forcePowers_t = 0; // instant
pub const FP_LEVITATION: forcePowers_t = 1; // hold/duration
pub const FP_SPEED: forcePowers_t = 2; // duration
pub const FP_PUSH: forcePowers_t = 3; // hold/duration
pub const FP_PULL: forcePowers_t = 4; // hold/duration
pub const FP_TELEPATHY: forcePowers_t = 5; // instant
pub const FP_GRIP: forcePowers_t = 6; // hold/duration
pub const FP_LIGHTNING: forcePowers_t = 7; // hold/duration
pub const FP_SABERTHROW: forcePowers_t = 8;
pub const FP_SABER_DEFENSE: forcePowers_t = 9;
pub const FP_SABER_OFFENSE: forcePowers_t = 10;
// new Jedi Academy powers
pub const FP_RAGE: forcePowers_t = 11; // duration
pub const FP_PROTECT: forcePowers_t = 12; // duration
pub const FP_ABSORB: forcePowers_t = 13; // duration
pub const FP_DRAIN: forcePowers_t = 14; // hold/duration
pub const FP_SEE: forcePowers_t = 15; // duration
pub const NUM_FORCE_POWERS: forcePowers_t = 16;
