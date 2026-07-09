#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::player_team_state_state_t::playerTeamStateState_t;

/// Raven `playerTeamState_t` — status in teamplay games.
///
/// Type definition source: `oracle/code/game/g_shared.h:275-289`
#[repr(C)]
pub struct playerTeamState_t {
    pub state: playerTeamStateState_t,

    pub captures: c_int,
    pub basedefense: c_int,
    pub carrierdefense: c_int,
    pub flagrecovery: c_int,
    pub fragcarrier: c_int,
    pub assists: c_int,

    pub lasthurtcarrier: f32,
    pub lastreturnedflag: f32,
    pub flagsince: f32,
    pub lastfraggedcarrier: f32,
}

const _: () = assert!(core::mem::size_of::<playerTeamState_t>() == 44);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, captures) == 4);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, basedefense) == 8);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, carrierdefense) == 12);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, flagrecovery) == 16);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, fragcarrier) == 20);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, assists) == 24);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, lasthurtcarrier) == 28);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, lastreturnedflag) == 32);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, flagsince) == 36);
const _: () = assert!(core::mem::offset_of!(playerTeamState_t, lastfraggedcarrier) == 40);
