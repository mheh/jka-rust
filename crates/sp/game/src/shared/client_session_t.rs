#![allow(non_camel_case_types, non_snake_case)]

use crate::teams::team::team_t;

use super::mission_stats_s::missionStats_t;
use super::objectives_s::objectives_t;

/// Raven `MAX_MISSION_OBJ` — DO NOT CHANGE. IT AFFECTS THE SAVEGAME STRUCTURE.
///
/// Source: `oracle/code/game/g_shared.h:299`
pub const MAX_MISSION_OBJ: usize = 100;

/// Raven `clientSession_t`.
///
/// Type definition source: `oracle/code/game/g_shared.h:331-336`
#[repr(C)]
pub struct clientSession_t {
    /// Number of times mission objectives have been updated.
    pub missionObjectivesShown: core::ffi::c_int,
    pub sessionTeam: team_t,
    pub mission_objectives: [objectives_t; MAX_MISSION_OBJ],
    /// Various totals while on a mission.
    pub missionStats: missionStats_t,
}

const _: () = assert!(core::mem::size_of::<clientSession_t>() == 1036);
const _: () = assert!(core::mem::offset_of!(clientSession_t, missionObjectivesShown) == 0);
const _: () = assert!(core::mem::offset_of!(clientSession_t, sessionTeam) == 4);
const _: () = assert!(core::mem::offset_of!(clientSession_t, mission_objectives) == 8);
const _: () = assert!(core::mem::offset_of!(clientSession_t, missionStats) == 808);
