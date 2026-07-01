#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `flagStatus_t` CTF flag state.
///
/// Raven declares this as `typedef int` alongside a separate anonymous
/// `_flag_status` enum, so the alias stays an int and the enumerators are
/// `const`s.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:3043-3050`
pub type flagStatus_t = c_int;

pub const FLAG_ATBASE: flagStatus_t = 0;
/// Raven: CTF
pub const FLAG_TAKEN: flagStatus_t = 1;
/// Raven: One Flag CTF
pub const FLAG_TAKEN_RED: flagStatus_t = 2;
/// Raven: One Flag CTF
pub const FLAG_TAKEN_BLUE: flagStatus_t = 3;
pub const FLAG_DROPPED: flagStatus_t = 4;
