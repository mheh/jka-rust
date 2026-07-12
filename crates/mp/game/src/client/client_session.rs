//! MP `clientSession_t`.
//!
//! Source: `oracle/codemp/game/g_local.h:408`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use mp_bg::team_t;
use mp_qshared::shared::qboolean;

use super::spectator_state::spectatorState_t;

// the auto following clients don't follow a specific client
// number, but instead follow the first two active players
pub const FOLLOW_ACTIVE1: c_int = -1;
pub const FOLLOW_ACTIVE2: c_int = -2;

/// Raven `clientSession_t` — client data that stays across levels/tournament restarts.
///
/// Raven: achieved by writing all the data to cvar strings at game shutdown and
/// reading them back at connection time; anything added MUST be handled in
/// `G_InitSessionData()` / `G_ReadSessionData()` / `G_WriteSessionData()`.
/// Source: `oracle/codemp/game/g_local.h:408`
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct clientSession_t {
    pub sessionTeam: team_t,
    pub spectatorTime: c_int, // for determining next-in-line to play
    pub spectatorState: spectatorState_t,
    pub spectatorClient: c_int, // for chasecam and follow mode
    pub wins: c_int,            // tournament stats
    pub losses: c_int,
    pub selectedFP: c_int, // check against this, if doesn't match value in playerstate then update userinfo
    pub saberLevel: c_int, // similar to above method, but for current saber attack level
    pub setForce: qboolean, // set to true once player is given the chance to set force powers
    pub updateUITime: c_int, // only update userinfo for FP/SL if < level.time
    pub teamLeader: qboolean, // true when this client is a team leader
    pub siegeClass: [c_char; 64],
    pub saberType: [c_char; 64],
    pub saber2Type: [c_char; 64],
    pub duelTeam: c_int,
    pub siegeDesiredTeam: c_int,
    pub killCount: c_int,
    pub TKCount: c_int,
    pub IPstring: [c_char; 32], // yeah, I know, could be 16, but, just in case...
}
const _: () = assert!(core::mem::size_of::<clientSession_t>() == 284);
