//! MP `clientPersistant_t`.
//!
//! Type definition source: `oracle/oracle/codemp/game/g_local.h:441-458`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::shared::qboolean;

use super::client_connected::clientConnected_t;
use super::player_team_state::playerTeamState_t;

// playerstate mGameFlags
pub const PSG_VOTED: c_int = 1 << 0; // already cast a vote
pub const PSG_TEAMVOTED: c_int = 1 << 1; // already cast a team vote

/// Raven `MAX_NETNAME`. Source: `oracle/oracle/codemp/game/g_local.h:438`
pub const MAX_NETNAME: usize = 36;
/// Raven `MAX_VOTE_COUNT`. Source: `oracle/oracle/codemp/game/g_local.h:439`
pub const MAX_VOTE_COUNT: c_int = 3;

/// Raven `clientPersistant_t` — client data that stays across respawns, cleared
/// on each level/team change at `ClientBegin()`.
///
/// Type definition source: `oracle/oracle/codemp/game/g_local.h:443-458`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct clientPersistant_t {
    pub connected: clientConnected_t,
    pub cmd: usercmd_t, // we would lose angles if not persistant
    pub localClient: qboolean, // true if "ip" info key is "localhost"
    pub initialSpawn: qboolean, // the first spawn should be at a cool location
    pub predictItemPickup: qboolean, // based on cg_predictItems userinfo
    pub pmoveFixed: qboolean,
    pub netname: [c_char; MAX_NETNAME],
    pub netnameTime: c_int, // Last time the name was changed
    pub maxHealth: c_int,   // for handicapping
    pub enterTime: c_int,   // level.time the client entered the game
    pub teamState: playerTeamState_t, // status in teamplay games
    pub voteCount: c_int,   // to prevent people from constantly calling votes
    pub teamVoteCount: c_int, // to prevent people from constantly calling votes
    pub teamInfo: qboolean, // send team overlay updates?
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<clientPersistant_t>() == 156);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, netname) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, teamState) == 96);
