//! MP `level_locals_t` — the world container.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:819-930`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::entity::gentity_t;
use mp_bg::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_NUM_TEAMS};
use mp_qshared::shared::{
    fileHandle_t, qboolean, vec3_t, MAX_CLIENTS, MAX_QPATH, MAX_STRING_CHARS,
};

use crate::ai::{AIGroupInfo_t, MAX_FRAME_GROUPS};
use crate::client::gclient_t;

use super::alert_event::{alertEvent_t, MAX_ALERT_EVENTS};
use super::combat_point::{combatPoint_t, MAX_COMBAT_POINTS};
use super::interest_point::{interestPoint_t, MAX_INTEREST_POINTS};

/// Raven `BODY_QUEUE_SIZE`. Source: `oracle/codemp/game/g_local.h:31`
pub const BODY_QUEUE_SIZE: usize = 8;

/// Raven `level_locals_t` — game-internal world state; cleared as each map is
/// entered. Pointer-bearing => arch-dependent; asserts pin the host-64-bit layout.
///
/// Type definition source: `oracle/codemp/game/g_local.h:819-930`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct level_locals_t {
    pub clients: *mut gclient_t, // [maxclients]

    pub gentities: *mut gentity_t,
    pub gentitySize: c_int,
    pub num_entities: c_int, // current number, <= MAX_GENTITIES

    pub warmupTime: c_int, // restart match at this time

    pub logFile: fileHandle_t,

    // store latched cvars here that we want to get at often
    pub maxclients: c_int,

    pub framenum: c_int,
    pub time: c_int,         // in msec
    pub previousTime: c_int, // so movers can back up when blocked

    pub startTime: c_int, // level.time the map was started

    pub teamScores: [c_int; TEAM_NUM_TEAMS as usize],
    pub lastTeamLocationTime: c_int, // last time of client team location update

    pub newSession: qboolean, // don't use any old session data (gametype changed)

    pub restarted: qboolean, // waiting for a map_restart to fire

    pub numConnectedClients: c_int,
    pub numNonSpectatorClients: c_int, // includes connecting clients
    pub numPlayingClients: c_int,      // connected, non-spectators
    pub sortedClients: [c_int; MAX_CLIENTS], // sorted by score
    pub follow1: c_int,                // clientNums for auto-follow spectators
    pub follow2: c_int,

    pub snd_fry: c_int, // sound index for standing in lava

    pub snd_hack: c_int,        // hacking loop sound
    pub snd_medHealed: c_int,   // being healed by supply class
    pub snd_medSupplied: c_int, // being supplied by supply class

    pub warmupModificationCount: c_int, // for detecting if g_warmup is changed

    // voting state
    pub voteString: [c_char; MAX_STRING_CHARS],
    pub voteDisplayString: [c_char; MAX_STRING_CHARS],
    pub voteTime: c_int,        // level.time vote was called
    pub voteExecuteTime: c_int, // time the vote is executed
    pub voteYes: c_int,
    pub voteNo: c_int,
    pub numVotingClients: c_int, // set by CalculateRanks

    pub votingGametype: qboolean,
    pub votingGametypeTo: c_int,

    // team voting state
    pub teamVoteString: [[c_char; MAX_STRING_CHARS]; 2],
    pub teamVoteTime: [c_int; 2], // level.time vote was called
    pub teamVoteYes: [c_int; 2],
    pub teamVoteNo: [c_int; 2],
    pub numteamVotingClients: [c_int; 2], // set by CalculateRanks

    // spawn variables
    pub spawning: qboolean, // the G_Spawn*() functions are valid
    pub numSpawnVars: c_int,
    pub spawnVars: [[*mut c_char; 2]; MAX_SPAWN_VARS], // key / value pairs
    pub numSpawnVarChars: c_int,
    pub spawnVarChars: [c_char; MAX_SPAWN_VARS_CHARS],

    // intermission state
    pub intermissionQueued: c_int, // wait INTERMISSION_DELAY_TIME before going there
    pub intermissiontime: c_int,   // time the intermission was started
    pub changemap: *mut c_char,
    pub readyToExit: qboolean, // at least one client wants to exit
    pub exitTime: c_int,
    pub intermission_origin: vec3_t, // also used for spectator spawns
    pub intermission_angle: vec3_t,

    pub locationLinked: qboolean,     // target_locations get linked
    pub locationHead: *mut gentity_t, // head of the location list
    pub bodyQueIndex: c_int,          // dead bodies
    pub bodyQue: [*mut gentity_t; BODY_QUEUE_SIZE],
    pub portalSequence: c_int,

    pub alertEvents: [alertEvent_t; MAX_ALERT_EVENTS],
    pub numAlertEvents: c_int,
    pub curAlertID: c_int,

    pub groups: [AIGroupInfo_t; MAX_FRAME_GROUPS],

    // Interest points — squadmates look at these when standing around nearby
    pub interestPoints: [interestPoint_t; MAX_INTEREST_POINTS],
    pub numInterestPoints: c_int,

    // Combat points — NPCs in BS_COMBAT_POINT find their closest empty one
    pub combatPoints: [combatPoint_t; MAX_COMBAT_POINTS],
    pub numCombatPoints: c_int,

    // rwwRMG - added:
    pub mNumBSPInstances: c_int,
    pub mBSPInstanceDepth: c_int,
    pub mOriginAdjust: vec3_t,
    pub mRotationAdjust: f32,
    pub mTargetAdjust: *mut c_char,

    pub mTeamFilter: [c_char; MAX_QPATH],
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<level_locals_t>() == 47176);
const _: () = assert!(core::mem::offset_of!(level_locals_t, clients) == 0); // arch-independent anchor
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, groups) == 11232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, combatPoints) == 32740);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(level_locals_t, mTeamFilter) == 47112);

// The STATE-D9 zeroed-construction contract (round-5 STATE-Q10 resolution):
// all-zero bytes are a valid level_locals_t — the same property the layout asserts above
// pin and Raven's memset/static zero-init relies on.
// Source: oracle/codemp/game/g_local.h (all-zero-valid #[repr(C)]; Raven zero-inits `level`, g_main.c:9)
unsafe impl native_platform::ZeroValid for level_locals_t {}
