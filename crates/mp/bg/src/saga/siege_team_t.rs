#![allow(non_camel_case_types, non_snake_case)]

use super::siege_class_t::siegeClass_t;
use mp_qshared::shared::limits::MAX_CLIENTS_I32;

/// Raven `MAX_SIEGE_CLASSES_PER_TEAM`.
///
/// Source: `oracle/codemp/game/bg_saga.h:13`
pub const MAX_SIEGE_CLASSES_PER_TEAM: usize = 16;

/// Raven `MAX_SIEGE_INFO_SIZE`.
///
/// Source: `oracle/codemp/game/bg_saga.h:1`
pub const MAX_SIEGE_INFO_SIZE: i32 = 16384;

/// Raven `SIEGE_POINTS_OBJECTIVECOMPLETED`.
///
/// Source: `oracle/codemp/game/bg_saga.h:6`
pub const SIEGE_POINTS_OBJECTIVECOMPLETED: i32 = 20;

/// Raven `SIEGE_POINTS_FINALOBJECTIVECOMPLETED`.
///
/// Source: `oracle/codemp/game/bg_saga.h:7`
pub const SIEGE_POINTS_FINALOBJECTIVECOMPLETED: i32 = 30;

/// Raven `SIEGE_POINTS_TEAMWONROUND`.
///
/// Source: `oracle/codemp/game/bg_saga.h:8`
pub const SIEGE_POINTS_TEAMWONROUND: i32 = 10;

/// Raven `SIEGE_ROUND_BEGIN_TIME`.
///
/// Raven: delay 5 secs after players are in game.
/// Source: `oracle/codemp/game/bg_saga.h:10`
pub const SIEGE_ROUND_BEGIN_TIME: i32 = 5000;

/// Raven `MAX_EXDATA_ENTS_TO_SEND` — max number of extended data for ents to send.
///
/// Source: `oracle/codemp/game/bg_saga.h:17`
pub const MAX_EXDATA_ENTS_TO_SEND: i32 = MAX_CLIENTS_I32;

/// Raven `SIEGETEAM_TEAM1` (e.g. TEAM_RED).
///
/// Source: `oracle/codemp/game/bg_saga.h:3`
pub const SIEGETEAM_TEAM1: i32 = 1;

/// Raven `SIEGETEAM_TEAM2` (e.g. TEAM_BLUE).
///
/// Source: `oracle/codemp/game/bg_saga.h:4`
pub const SIEGETEAM_TEAM2: i32 = 2;

/// Raven `siegeTeam_t` — one team's siege class roster.
///
/// Type definition source: `oracle/codemp/game/bg_saga.h:82-88`
#[repr(C)]
pub struct siegeTeam_t {
    pub name: [core::ffi::c_char; 512],
    pub classes: [*mut siegeClass_t; MAX_SIEGE_CLASSES_PER_TEAM],
    pub numClasses: i32,
    pub friendlyShader: i32,
}

const _: () = assert!(core::mem::offset_of!(siegeTeam_t, name) == 0);
const _: () = assert!(core::mem::offset_of!(siegeTeam_t, classes) == 512);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<siegeTeam_t>() == 648);
    assert!(core::mem::offset_of!(siegeTeam_t, numClasses) == 640);
    assert!(core::mem::offset_of!(siegeTeam_t, friendlyShader) == 644);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<siegeTeam_t>() == 584);
    assert!(core::mem::offset_of!(siegeTeam_t, numClasses) == 576);
    assert!(core::mem::offset_of!(siegeTeam_t, friendlyShader) == 580);
};
