#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_short};

use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;

use super::client_connected_t::clientConnected_t;
use super::player_team_state_t::playerTeamState_t;

/// Raven `clientPersistant_t`.
///
/// Type definition source: `oracle/code/game/g_shared.h:341-350`
#[repr(C)]
pub struct clientPersistant_t {
    pub connected: clientConnected_t,
    pub lastCommand: usercmd_t,
    pub netname: [c_char; 34],
    /// for handicapping
    pub maxHealth: c_int,
    /// level.time the client entered the game
    pub enterTime: c_int,
    /// angles sent over in the last command
    pub cmd_angles: [c_short; 3],

    /// status in teamplay games
    pub teamState: playerTeamState_t,
}

const _: () = assert!(core::mem::size_of::<clientPersistant_t>() == 128);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, connected) == 0);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, lastCommand) == 4);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, netname) == 32);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, maxHealth) == 68);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, enterTime) == 72);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, cmd_angles) == 76);
const _: () = assert!(core::mem::offset_of!(clientPersistant_t, teamState) == 84);
