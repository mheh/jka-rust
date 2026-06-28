//! The world container: `level_locals_t`, owning the entity/client arrays (Raven `g_local.h`).
//!
//! Game-internal (the engine never sees these). `level_locals_t` embeds the ai.h
//! `AIGroupInfo_t groups[MAX_FRAME_GROUPS]` and the alert/interest/combat arrays by
//! value, and is pointer-bearing => arch-dependent (64-bit layout asserted).

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

// TODO: NOT-PORTED
use crate::bg::{MAX_SPAWN_VARS, MAX_SPAWN_VARS_CHARS, TEAM_NUM_TEAMS};
// TODO: NOT-PORTED
use crate::game::ai::{AIGroupInfo_t, MAX_FRAME_GROUPS};
// TODO: NOT-PORTED
use crate::shared::{fileHandle_t, qboolean, vec3_t, MAX_CLIENTS, MAX_QPATH, MAX_STRING_CHARS};
use core::ffi::{c_char, c_int};

use super::client::gclient_s;
use super::entity::gentity_s;

pub const BODY_QUEUE_SIZE: usize = 8;

//Interest points
pub const MAX_INTEREST_POINTS: usize = 64;

/// `interestPoint_t` (g_local.h) — squadmates look at these when idle and close.
/// Carries a `char *target` => arch-dependent.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct interestPoint_t {
    pub origin: vec3_t,
    pub target: *mut c_char,
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<interestPoint_t>() == 24);

//Combat points
pub const MAX_COMBAT_POINTS: usize = 512;

/// `combatPoint_t` (g_local.h) — NPCs in bState BS_COMBAT_POINT find their closest
/// empty combat_point. Pointer-free (the `NPC_targetname`/`team` members are
/// commented out in the original).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct combatPoint_t {
    pub origin: vec3_t,
    pub flags: c_int,
    //	char		*NPC_targetname;
    //	team_t		team;
    pub occupied: qboolean,
    pub waypoint: c_int,
    pub dangerTime: c_int,
}
const _: () = assert!(core::mem::size_of::<combatPoint_t>() == 28);

// Alert events
pub const MAX_ALERT_EVENTS: usize = 32;

/// `alertEventType_e` (g_local.h).
pub type alertEventType_e = c_int;
pub const AET_SIGHT: alertEventType_e = 0;
pub const AET_SOUND: alertEventType_e = 1;

/// `alertEventLevel_e` (g_local.h).
pub type alertEventLevel_e = c_int;
pub const AEL_MINOR: alertEventLevel_e = 0; //Enemy responds to the sound, but only by looking
pub const AEL_SUSPICIOUS: alertEventLevel_e = 1; //Enemy looks at the sound, and will also investigate it
pub const AEL_DISCOVERED: alertEventLevel_e = 2; //Enemy knows the player is around, and will actively hunt
pub const AEL_DANGER: alertEventLevel_e = 3; //Enemy should try to find cover
pub const AEL_DANGER_GREAT: alertEventLevel_e = 4; //Enemy should run like hell!

/// `alertEvent_t` (g_local.h). Carries a `gentity_t *owner` => arch-dependent.
/// `type` is a Rust keyword, hence `r#type` (the C field is `type`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct alertEvent_t {
    pub position: vec3_t,         //Where the event is located
    pub radius: f32,              //Consideration radius
    pub level: alertEventLevel_e, //Priority level of the event
    pub r#type: alertEventType_e, //Event type (sound,sight)
    pub owner: *mut gentity_s,    //Who made the sound
    pub light: f32,               //ambient light level at point
    pub addLight: f32,            //additional light- makes it more noticable, even in darkness
    pub ID: c_int, //unique... if get a ridiculous number, this will repeat, but should not be a problem as it's just comparing it to your lastAlertID
    pub timestamp: c_int, //when it was created
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<alertEvent_t>() == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(alertEvent_t, owner) == 24);

/// `waypointData_t` (g_local.h) — "this structure is cleared as each map is
/// entered". Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct waypointData_t {
    pub targetname: [c_char; MAX_QPATH],
    pub target: [c_char; MAX_QPATH],
    pub target2: [c_char; MAX_QPATH],
    pub target3: [c_char; MAX_QPATH],
    pub target4: [c_char; MAX_QPATH],
    pub nodeID: c_int,
}
const _: () = assert!(core::mem::size_of::<waypointData_t>() == 324);

/// `level_locals_t` (g_local.h) — "this structure is cleared as each map is
/// entered". Game-internal (not engine-visible). Pointer-bearing => arch-dependent;
/// embeds `AIGroupInfo_t groups[MAX_FRAME_GROUPS]` and the alert/interest/combat
/// arrays by value.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct level_locals_t {
    pub clients: *mut gclient_s, // [maxclients]

    pub gentities: *mut gentity_s,
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
    pub spawnVars: [[*mut c_char; 2]; MAX_SPAWN_VARS as usize], // key / value pairs
    pub numSpawnVarChars: c_int,
    pub spawnVarChars: [c_char; MAX_SPAWN_VARS_CHARS as usize],

    // intermission state
    pub intermissionQueued: c_int, // intermission was qualified, but
    // wait INTERMISSION_DELAY_TIME before
    // actually going there so the last
    // frag can be watched.  Disable future
    // kills during this delay
    pub intermissiontime: c_int, // time the intermission was started
    pub changemap: *mut c_char,
    pub readyToExit: qboolean, // at least one client wants to exit
    pub exitTime: c_int,
    pub intermission_origin: vec3_t, // also used for spectator spawns
    pub intermission_angle: vec3_t,

    pub locationLinked: qboolean,     // target_locations get linked
    pub locationHead: *mut gentity_s, // head of the location list
    pub bodyQueIndex: c_int,          // dead bodies
    pub bodyQue: [*mut gentity_s; BODY_QUEUE_SIZE],
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

// ===========================================================================
// Remaining g_local.h data: damage flags, mover spawnflags, and the two later
// pointer-free structs (reference_tag_t in g_misc.c, bot_settings_t in ai_util.c).
// ===========================================================================

// damage flags (used by G_Damage's `dflags`)
pub const DAMAGE_NORMAL: c_int = 0x00000000; // No flags set.
pub const DAMAGE_RADIUS: c_int = 0x00000001; // damage was indirect
pub const DAMAGE_NO_ARMOR: c_int = 0x00000002; // armour does not protect from this damage
pub const DAMAGE_NO_KNOCKBACK: c_int = 0x00000004; // do not affect velocity, just view angles
pub const DAMAGE_NO_PROTECTION: c_int = 0x00000008; // armor, shields, invulnerability, and godmode have no effect
pub const DAMAGE_NO_TEAM_PROTECTION: c_int = 0x00000010; // armor, shields, invulnerability, and godmode have no effect
                                                         //JK2 flags
pub const DAMAGE_EXTRA_KNOCKBACK: c_int = 0x00000040; // add extra knockback to this damage
pub const DAMAGE_DEATH_KNOCKBACK: c_int = 0x00000080; // only does knockback on death of target
pub const DAMAGE_IGNORE_TEAM: c_int = 0x00000100; // damage is always done, regardless of teams
pub const DAMAGE_NO_DAMAGE: c_int = 0x00000200; // do no actual damage but react as if damage was taken
pub const DAMAGE_HALF_ABSORB: c_int = 0x00000400; // half shields, half health
pub const DAMAGE_HALF_ARMOR_REDUCTION: c_int = 0x00000800; // This damage doesn't whittle down armor as efficiently.
pub const DAMAGE_HEAVY_WEAP_CLASS: c_int = 0x00001000; // Heavy damage
pub const DAMAGE_NO_HIT_LOC: c_int = 0x00002000; // No hit location
pub const DAMAGE_NO_SELF_PROTECTION: c_int = 0x00004000; // Dont apply half damage to self attacks
pub const DAMAGE_NO_DISMEMBER: c_int = 0x00008000; // Dont do dismemberment
pub const DAMAGE_SABER_KNOCKBACK1: c_int = 0x00010000; // Check the attacker's first saber for a knockbackScale
pub const DAMAGE_SABER_KNOCKBACK2: c_int = 0x00020000; // Check the attacker's second saber for a knockbackScale
pub const DAMAGE_SABER_KNOCKBACK1_B2: c_int = 0x00040000; // Check the attacker's first saber for a knockbackScale2
pub const DAMAGE_SABER_KNOCKBACK2_B2: c_int = 0x00080000; // Check the attacker's second saber for a knockbackScale2

// g_mover.c button spawnflags
pub const SPF_BUTTON_USABLE: c_int = 1;
pub const SPF_BUTTON_FPUSHABLE: c_int = 2;

// g_misc.c reference tags
pub const MAX_REFNAME: usize = 32;
pub const RTF_NONE: c_int = 0;
pub const RTF_NAVGOAL: c_int = 0x00000001;

/// `reference_tag_t` (g_local.h, g_misc.c). Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct reference_tag_t {
    pub name: [c_char; MAX_REFNAME],
    pub origin: vec3_t,
    pub angles: vec3_t,
    pub flags: c_int,  //Just in case
    pub radius: c_int, //For nav goals
    pub inuse: qboolean,
}
const _: () = assert!(core::mem::size_of::<reference_tag_t>() == 68);

/// `MAX_FILEPATH` (g_local.h, ai_main.c).
pub const MAX_FILEPATH: usize = 144;

/// `bot_settings_t` (g_local.h, ai_util.c). Pointer-free.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct bot_settings_t {
    pub personalityfile: [c_char; MAX_FILEPATH],
    pub skill: f32,
    pub team: [c_char; MAX_FILEPATH],
}
const _: () = assert!(core::mem::size_of::<bot_settings_t>() == 292);
