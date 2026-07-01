#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `e_status` cinematic playback state.
///
/// Raven declares this as `typedef int` alongside a separate anonymous enum of
/// FMV states, so the alias stays an int and the enumerators are `const`s.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:3032-3041`
pub type e_status = c_int;

pub const FMV_IDLE: e_status = 0;
/// Raven: play
pub const FMV_PLAY: e_status = 1;
/// Raven: all other conditions, i.e. stop/EOF/abort
pub const FMV_EOF: e_status = 2;
pub const FMV_ID_BLT: e_status = 3;
pub const FMV_ID_IDLE: e_status = 4;
pub const FMV_LOOPED: e_status = 5;
pub const FMV_ID_WAIT: e_status = 6;
