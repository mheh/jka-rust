#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `forcePowers_t` force-power index.
///
/// Raven declares this as `typedef int` alongside a separate anonymous enum of
/// power indices, so the alias stays an int and the enumerators are `const`s.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:590-613`
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
pub const FP_RAGE: forcePowers_t = 8; // duration
pub const FP_PROTECT: forcePowers_t = 9;
pub const FP_ABSORB: forcePowers_t = 10;
pub const FP_TEAM_HEAL: forcePowers_t = 11;
pub const FP_TEAM_FORCE: forcePowers_t = 12;
pub const FP_DRAIN: forcePowers_t = 13;
pub const FP_SEE: forcePowers_t = 14;
pub const FP_SABER_OFFENSE: forcePowers_t = 15;
pub const FP_SABER_DEFENSE: forcePowers_t = 16;
pub const FP_SABERTHROW: forcePowers_t = 17;
pub const NUM_FORCE_POWERS: forcePowers_t = 18;
