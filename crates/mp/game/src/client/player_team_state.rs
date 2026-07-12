//! MP `playerTeamStateState_t` and `playerTeamState_t`.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:380-401`

#![allow(non_camel_case_types)]

use core::ffi::c_int;

/// Raven `playerTeamStateState_t`.
///
/// Verified against oracle: a **named `typedef enum`**, not a `typedef int`.
/// Type definition source: `oracle/codemp/game/g_local.h:380-383`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum playerTeamStateState_t {
    #[default]
    TEAM_BEGIN = 0, // Beginning a team game, spawn at base
    TEAM_ACTIVE, // Now actively playing
}

/// Raven `playerTeamState_t` — status in teamplay games.
///
/// Type definition source: `oracle/codemp/game/g_local.h:385-401`
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct playerTeamState_t {
    pub state: playerTeamStateState_t,

    pub location: c_int,

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
const _: () = assert!(core::mem::size_of::<playerTeamState_t>() == 48);
