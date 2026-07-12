//! MP `team_t` and its team constants.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1008-1017`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `team_t`.
///
/// Raven names the teams via an anonymous `enum { TEAM_FREE..TEAM_NUM_TEAMS }`,
/// then `typedef int team_t` for storage.
/// Type definition source: `oracle/codemp/game/bg_public.h:1017`
pub type team_t = c_int;

pub const TEAM_FREE: team_t = 0;
pub const TEAM_RED: team_t = 1;
pub const TEAM_BLUE: team_t = 2;
pub const TEAM_SPECTATOR: team_t = 3;
pub const TEAM_NUM_TEAMS: team_t = 4;
