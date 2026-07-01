//! MP `npcteam_t` and its team constants.
//!
//! Type definition source: `oracle/oracle/codemp/game/teams.h:4-14`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `npcteam_t`.
///
/// Raven names the teams via an anonymous `enum { NPCTEAM_FREE..NPCTEAM_NUM_TEAMS }`,
/// then `typedef int npcteam_t` for storage.
/// Type definition source: `oracle/oracle/codemp/game/teams.h:14`
pub type npcteam_t = c_int;

pub const NPCTEAM_FREE: npcteam_t = 0; // also TEAM_FREE
pub const NPCTEAM_ENEMY: npcteam_t = 1; // also TEAM_RED
pub const NPCTEAM_PLAYER: npcteam_t = 2; // also TEAM_BLUE
pub const NPCTEAM_NEUTRAL: npcteam_t = 3; // also TEAM_SPECTATOR - most droids are team_neutral
pub const NPCTEAM_NUM_TEAMS: npcteam_t = 4;
