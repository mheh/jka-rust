//! MP `level_locals_t`, the world container.
//!
//! Type definition source: `oracle/codemp/game/g_local.h:819-930`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use crate::entity::gentity_t;
use mp_bg::TEAM_NUM_TEAMS;
use mp_qshared::shared::{fileHandle_t, qboolean, vec3_t, MAX_CLIENTS};

use crate::ai::{AIGroupInfo_t, MAX_FRAME_GROUPS};
use crate::client::gclient_t;

use super::alert_event::{alertEvent_t, MAX_ALERT_EVENTS};
use super::combat_point::{combatPoint_t, MAX_COMBAT_POINTS};
use super::interest_point::{interestPoint_t, MAX_INTEREST_POINTS};

/// Raven `BODY_QUEUE_SIZE`. Source: `oracle/codemp/game/g_local.h:31`
pub const BODY_QUEUE_SIZE: usize = 8;

/// Raven `level_locals_t`, the game-internal world state, cleared as each map is entered.
/// Nothing outside `mp_game` reads it by layout: the engine aliases `g_entities` and `clients`
/// through `trap_LocateGameData`, not `level`.
/// It carries no ABI layout contract, so it has no `#[repr(C)]` and no layout asserts.
/// The owned `String` vote and filter fields make it non-`Copy` and not zero-valid.
/// `Default` below supplies the zero state.
///
/// Type definition source: `oracle/codemp/game/g_local.h:819-930`
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

    pub newSession: qboolean, // don't use any old session data, because
    // we changed gametype

    pub restarted: qboolean, // waiting for a map_restart to fire

    pub numConnectedClients: c_int,
    pub numNonSpectatorClients: c_int, // includes connecting clients
    pub numPlayingClients: c_int,      // connected, non-spectators
    pub sortedClients: [c_int; MAX_CLIENTS], // sorted by score
    pub follow1: c_int,                // clientNums for auto-follow spectators
    pub follow2: c_int,

    pub snd_fry: c_int, // sound index for standing in lava

    pub snd_hack: c_int,        //hacking loop sound
    pub snd_medHealed: c_int,   //being healed by supply class
    pub snd_medSupplied: c_int, //being supplied by supply class

    pub warmupModificationCount: c_int, // for detecting if g_warmup is changed

    // voting state
    pub voteString: String,
    pub voteDisplayString: String,
    pub voteTime: c_int,        // level.time vote was called
    pub voteExecuteTime: c_int, // time the vote is executed
    pub voteYes: c_int,
    pub voteNo: c_int,
    pub numVotingClients: c_int, // set by CalculateRanks

    pub votingGametype: qboolean,
    pub votingGametypeTo: c_int,

    // team voting state
    pub teamVoteString: [String; 2],
    pub teamVoteTime: [c_int; 2], // level.time vote was called
    pub teamVoteYes: [c_int; 2],
    pub teamVoteNo: [c_int; 2],
    pub numteamVotingClients: [c_int; 2], // set by CalculateRanks

    // spawn variables
    pub spawning: qboolean, // the G_Spawn*() functions are valid
    /// Raven's `numSpawnVars`, `spawnVars[][2]`, `numSpawnVarChars`, and `spawnVarChars[]`
    /// (the key/value pointer table plus its backing char pool) collapse into one
    /// owned `Vec` of `(key, value)` pairs.
    /// The count is `spawnVars.len()`.
    pub spawnVars: Vec<(String, String)>,

    // intermission state
    pub intermissionQueued: c_int, // intermission was qualified, but
    // wait INTERMISSION_DELAY_TIME before
    // actually going there so the last
    // frag can be watched.  Disable future
    // kills during this delay
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

    //Interest points- squadmates automatically look at these if standing around and close to them
    pub interestPoints: [interestPoint_t; MAX_INTEREST_POINTS],
    pub numInterestPoints: c_int,

    //Combat points- NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
    pub combatPoints: [combatPoint_t; MAX_COMBAT_POINTS],
    pub numCombatPoints: c_int,

    //rwwRMG - added:
    pub mNumBSPInstances: c_int,
    pub mBSPInstanceDepth: c_int,
    pub mOriginAdjust: vec3_t,
    pub mRotationAdjust: f32,
    pub mTargetAdjust: *mut c_char,

    pub mTeamFilter: String,
}

impl Default for level_locals_t {
    /// Raven zero-fills `level` wholesale with `memset(&level, 0, sizeof(level))` in
    /// `G_InitGame`, `g_main.c`.
    /// Every field is all-zero-valid except the owned `String`s (`voteString`, `voteDisplayString`,
    /// `teamVoteString[2]`, `mTeamFilter`) and the owned `spawnVars` `Vec`, whose zeroed bytes are invalid.
    /// We zero the whole image, then write a valid empty value into each of those slots before anything reads it.
    /// The result matches Raven's zero state: every scalar 0, every pointer null, every vote and filter
    /// string empty, and the spawn-var table empty.
    fn default() -> Self {
        let mut u = core::mem::MaybeUninit::<level_locals_t>::uninit();
        let p = u.as_mut_ptr();
        // SAFETY: `p` is freshly allocated, aligned storage for one `level_locals_t`.
        // `write_bytes` zeroes every field, and every field except the owned `String`s and the `Vec`
        // is all-zero-valid.
        // Each `ptr::write` overwrites one non-zero-valid slot with a valid empty value, without
        // dropping its zeroed bytes.
        // So `assume_init` observes a fully valid value.
        unsafe {
            core::ptr::write_bytes(p, 0, 1);
            core::ptr::write(core::ptr::addr_of_mut!((*p).voteString), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).voteDisplayString), String::new());
            core::ptr::write(
                core::ptr::addr_of_mut!((*p).teamVoteString),
                [String::new(), String::new()],
            );
            core::ptr::write(core::ptr::addr_of_mut!((*p).mTeamFilter), String::new());
            core::ptr::write(core::ptr::addr_of_mut!((*p).spawnVars), Vec::new());
            u.assume_init()
        }
    }
}
