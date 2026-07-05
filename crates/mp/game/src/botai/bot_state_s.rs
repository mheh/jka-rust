#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_bg::weapons::weapon_t::WP_NUM_WEAPONS;
use mp_qshared::common::mp::entity_id::EntityId;
use mp_qshared::common::mp::qcommon::{playerState_t, usercmd_t};
use mp_qshared::shared::{qboolean, vec3_t, wpobject_t};

use super::bot_settings_s::bot_settings_t;
use super::botattachment_s::botattachment_t;
use super::botskills_s::botskills_t;

/// Raven MP `MAX_CHAT_LINE_SIZE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:12`
pub const MAX_CHAT_LINE_SIZE: usize = 128;

/// Raven MP `MAX_LOVED_ONES`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:17`
pub const MAX_LOVED_ONES: usize = 4;

/// Raven MP `MAX_FORCE_INFO_SIZE`.
///
/// Source: `oracle/oracle/codemp/game/ai_main.h:20`
pub const MAX_FORCE_INFO_SIZE: usize = 2048;

/// Raven `bot_state_t` — per-bot AI state (goals, waypoints, timers, chat, saber
/// combat, force powers, ...).
///
/// Raven: `FORCEJUMP_INSTANTMETHOD` is commented out at `ai_main.h:5`, so the
/// `#ifndef FORCEJUMP_INSTANTMETHOD` branch is always active and
/// `forceJumpChargeTime` is always present in this build.
/// Type definition source: `oracle/oracle/codemp/game/ai_main.h:148-342`
#[repr(C)]
pub struct bot_state_t {
    pub inuse: c_int,             // true if this state is used by a bot client
    pub botthink_residual: c_int, // residual for the bot thinks
    pub client: c_int,            // client number of the bot
    pub entitynum: c_int,         // entity number of the bot
    pub cur_ps: playerState_t,    // current player state
    pub lastucmd: usercmd_t,      // usercmd from last frame
    pub settings: bot_settings_t, // several bot settings
    pub thinktime: f32,           // time the bot thinks this frame
    pub origin: vec3_t,           // origin of the bot
    pub velocity: vec3_t,         // velocity of the bot
    pub eye: vec3_t,              // eye coordinates of the bot
    pub setupcount: c_int,        // true when the bot has just been setup
    pub ltime: f32,               // local bot time
    pub entergame_time: f32,      // time the bot entered the game
    pub ms: c_int,                // move state of the bot
    pub gs: c_int,                // goal state of the bot
    pub ws: c_int,                // weapon state of the bot
    pub viewangles: vec3_t,       // current view angles
    pub ideal_viewangles: vec3_t, // ideal view angles
    pub viewanglespeed: vec3_t,

    // rww - new AI values
    pub currentEnemy: Option<EntityId>,
    pub revengeEnemy: Option<EntityId>,

    pub squadLeader: Option<EntityId>,

    pub lastHurt: Option<EntityId>,
    pub lastAttacked: Option<EntityId>,

    pub wantFlag: Option<EntityId>,

    pub touchGoal: Option<EntityId>,
    pub shootGoal: Option<EntityId>,

    pub dangerousObject: Option<EntityId>,

    pub staticFlagSpot: vec3_t,

    pub revengeHateLevel: c_int,
    pub isSquadLeader: c_int,

    pub squadRegroupInterval: c_int,
    pub squadCannotLead: c_int,

    pub lastDeadTime: c_int,

    pub wpCurrent: *mut wpobject_t,
    pub wpDestination: *mut wpobject_t,
    pub wpStoreDest: *mut wpobject_t,
    pub goalAngles: vec3_t,
    pub goalMovedir: vec3_t,
    pub goalPosition: vec3_t,

    pub lastEnemySpotted: vec3_t,
    pub hereWhenSpotted: vec3_t,
    pub lastVisibleEnemyIndex: c_int,
    pub hitSpotted: c_int,

    pub wpDirection: c_int,

    pub destinationGrabTime: f32,
    pub wpSeenTime: f32,
    pub wpTravelTime: f32,
    pub wpDestSwitchTime: f32,
    pub wpSwitchTime: f32,
    pub wpDestIgnoreTime: f32,

    pub timeToReact: f32,

    pub enemySeenTime: f32,

    pub chickenWussCalculationTime: f32,

    pub beStill: f32,
    pub duckTime: f32,
    pub jumpTime: f32,
    pub jumpHoldTime: f32,
    pub jumpPrep: f32,
    pub forceJumping: f32,
    pub jDelay: f32,

    pub aimOffsetTime: f32,
    pub aimOffsetAmtYaw: f32,
    pub aimOffsetAmtPitch: f32,

    pub frame_Waypoint_Len: f32,
    pub frame_Waypoint_Vis: c_int,
    pub frame_Enemy_Len: f32,
    pub frame_Enemy_Vis: c_int,

    pub isCamper: c_int,
    pub isCamping: f32,
    pub wpCamping: *mut wpobject_t,
    pub wpCampingTo: *mut wpobject_t,
    pub campStanding: qboolean,

    pub randomNavTime: c_int,
    pub randomNav: c_int,

    pub saberSpecialist: c_int,

    pub canChat: c_int,
    pub chatFrequency: c_int,
    pub currentChat: [u8; MAX_CHAT_LINE_SIZE],
    pub chatTime: f32,
    pub chatTime_stored: f32,
    pub doChat: c_int,
    pub chatTeam: c_int,
    pub chatObject: Option<EntityId>,
    pub chatAltObject: Option<EntityId>,

    pub meleeStrafeTime: f32,
    pub meleeStrafeDir: c_int,
    pub meleeStrafeDisable: f32,

    pub altChargeTime: c_int,

    pub escapeDirTime: f32,

    pub dontGoBack: f32,

    pub doAttack: c_int,
    pub doAltAttack: c_int,

    pub forceWeaponSelect: c_int,
    pub virtualWeapon: c_int,

    pub plantTime: c_int,
    pub plantDecided: c_int,
    pub plantContinue: c_int,
    pub plantKillEmAll: c_int,

    pub runningLikeASissy: c_int,
    pub runningToEscapeThreat: c_int,

    // Raven: `chatBuffer[MAX_CHAT_BUFFER_SIZE]` is commented out here — since
    // bots are once again not allocated dynamically, shoving a 64k chat buffer
    // into one is a bad thing.
    pub skills: botskills_t,

    pub loved: [botattachment_t; MAX_LOVED_ONES],
    pub lovednum: c_int,

    pub loved_death_thresh: c_int,

    pub deathActivitiesDone: c_int,

    pub botWeaponWeights: [f32; WP_NUM_WEAPONS as usize],

    pub ctfState: c_int,

    pub siegeState: c_int,

    pub teamplayState: c_int,

    pub jmState: c_int,

    pub state_Forced: c_int, // set by player ordering menu

    pub saberDefending: c_int,
    pub saberDefendDecideTime: c_int,
    pub saberBFTime: c_int,
    pub saberBTime: c_int,
    pub saberSTime: c_int,
    pub saberThrowTime: c_int,

    pub saberPower: qboolean,
    pub saberPowerTime: c_int,

    pub botChallengingTime: c_int,

    pub forceinfo: [u8; MAX_FORCE_INFO_SIZE],

    pub forceJumpChargeTime: c_int,

    pub doForcePush: c_int,

    pub noUseTime: c_int,
    pub doingFallback: qboolean,

    pub iHaveNoIdeaWhereIAmGoing: c_int,
    pub lastSignificantAreaChange: vec3_t,
    pub lastSignificantChangeTime: c_int,

    pub forceMove_Forward: c_int,
    pub forceMove_Right: c_int,
    pub forceMove_Up: c_int,
    // end rww
}

const _: () = assert!(core::mem::offset_of!(bot_state_t, inuse) == 0);
const _: () = assert!(core::mem::offset_of!(bot_state_t, botthink_residual) == 4);
const _: () = assert!(core::mem::offset_of!(bot_state_t, client) == 8);
const _: () = assert!(core::mem::offset_of!(bot_state_t, entitynum) == 12);
const _: () = assert!(core::mem::offset_of!(bot_state_t, cur_ps) == 16);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastucmd) == 1568);
const _: () = assert!(core::mem::offset_of!(bot_state_t, settings) == 1596);
const _: () = assert!(core::mem::offset_of!(bot_state_t, thinktime) == 1888);
const _: () = assert!(core::mem::offset_of!(bot_state_t, origin) == 1892);
const _: () = assert!(core::mem::offset_of!(bot_state_t, velocity) == 1904);
const _: () = assert!(core::mem::offset_of!(bot_state_t, eye) == 1916);
const _: () = assert!(core::mem::offset_of!(bot_state_t, setupcount) == 1928);
const _: () = assert!(core::mem::offset_of!(bot_state_t, ltime) == 1932);
const _: () = assert!(core::mem::offset_of!(bot_state_t, entergame_time) == 1936);
const _: () = assert!(core::mem::offset_of!(bot_state_t, ms) == 1940);
const _: () = assert!(core::mem::offset_of!(bot_state_t, gs) == 1944);
const _: () = assert!(core::mem::offset_of!(bot_state_t, ws) == 1948);
const _: () = assert!(core::mem::offset_of!(bot_state_t, viewangles) == 1952);
const _: () = assert!(core::mem::offset_of!(bot_state_t, ideal_viewangles) == 1964);
const _: () = assert!(core::mem::offset_of!(bot_state_t, viewanglespeed) == 1976);
// This struct's stored `gentity_t*` fields are ported as `Option<EntityId>`
// (align 4 vs a pointer's align 8), so the private tail's byte offsets shift. This struct is
// game-internal / not ABI-fixed beyond its prefix — the engine learns the full
// stride at runtime via `trap_LocateGameData`. The `size_of` assert and every
// `offset_of` assert at/after the first flipped field are therefore dropped;
// only the fixed-prefix asserts above (declared before the first flip) remain.
