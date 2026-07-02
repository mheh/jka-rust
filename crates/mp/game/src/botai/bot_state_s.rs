#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use mp_bg::weapons::weapon_t::WP_NUM_WEAPONS;
use mp_qshared::common::mp::gentity::gentity_t;
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
	pub currentEnemy: *mut gentity_t,
	pub revengeEnemy: *mut gentity_t,

	pub squadLeader: *mut gentity_t,

	pub lastHurt: *mut gentity_t,
	pub lastAttacked: *mut gentity_t,

	pub wantFlag: *mut gentity_t,

	pub touchGoal: *mut gentity_t,
	pub shootGoal: *mut gentity_t,

	pub dangerousObject: *mut gentity_t,

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
	pub chatObject: *mut gentity_t,
	pub chatAltObject: *mut gentity_t,

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

const _: () = assert!(core::mem::size_of::<bot_state_t>() == 5096);
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
const _: () = assert!(core::mem::offset_of!(bot_state_t, currentEnemy) == 1992);
const _: () = assert!(core::mem::offset_of!(bot_state_t, revengeEnemy) == 2000);
const _: () = assert!(core::mem::offset_of!(bot_state_t, squadLeader) == 2008);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastHurt) == 2016);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastAttacked) == 2024);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wantFlag) == 2032);
const _: () = assert!(core::mem::offset_of!(bot_state_t, touchGoal) == 2040);
const _: () = assert!(core::mem::offset_of!(bot_state_t, shootGoal) == 2048);
const _: () = assert!(core::mem::offset_of!(bot_state_t, dangerousObject) == 2056);
const _: () = assert!(core::mem::offset_of!(bot_state_t, staticFlagSpot) == 2064);
const _: () = assert!(core::mem::offset_of!(bot_state_t, revengeHateLevel) == 2076);
const _: () = assert!(core::mem::offset_of!(bot_state_t, isSquadLeader) == 2080);
const _: () = assert!(core::mem::offset_of!(bot_state_t, squadRegroupInterval) == 2084);
const _: () = assert!(core::mem::offset_of!(bot_state_t, squadCannotLead) == 2088);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastDeadTime) == 2092);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpCurrent) == 2096);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpDestination) == 2104);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpStoreDest) == 2112);
const _: () = assert!(core::mem::offset_of!(bot_state_t, goalAngles) == 2120);
const _: () = assert!(core::mem::offset_of!(bot_state_t, goalMovedir) == 2132);
const _: () = assert!(core::mem::offset_of!(bot_state_t, goalPosition) == 2144);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastEnemySpotted) == 2156);
const _: () = assert!(core::mem::offset_of!(bot_state_t, hereWhenSpotted) == 2168);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastVisibleEnemyIndex) == 2180);
const _: () = assert!(core::mem::offset_of!(bot_state_t, hitSpotted) == 2184);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpDirection) == 2188);
const _: () = assert!(core::mem::offset_of!(bot_state_t, destinationGrabTime) == 2192);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpSeenTime) == 2196);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpTravelTime) == 2200);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpDestSwitchTime) == 2204);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpSwitchTime) == 2208);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpDestIgnoreTime) == 2212);
const _: () = assert!(core::mem::offset_of!(bot_state_t, timeToReact) == 2216);
const _: () = assert!(core::mem::offset_of!(bot_state_t, enemySeenTime) == 2220);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chickenWussCalculationTime) == 2224);
const _: () = assert!(core::mem::offset_of!(bot_state_t, beStill) == 2228);
const _: () = assert!(core::mem::offset_of!(bot_state_t, duckTime) == 2232);
const _: () = assert!(core::mem::offset_of!(bot_state_t, jumpTime) == 2236);
const _: () = assert!(core::mem::offset_of!(bot_state_t, jumpHoldTime) == 2240);
const _: () = assert!(core::mem::offset_of!(bot_state_t, jumpPrep) == 2244);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceJumping) == 2248);
const _: () = assert!(core::mem::offset_of!(bot_state_t, jDelay) == 2252);
const _: () = assert!(core::mem::offset_of!(bot_state_t, aimOffsetTime) == 2256);
const _: () = assert!(core::mem::offset_of!(bot_state_t, aimOffsetAmtYaw) == 2260);
const _: () = assert!(core::mem::offset_of!(bot_state_t, aimOffsetAmtPitch) == 2264);
const _: () = assert!(core::mem::offset_of!(bot_state_t, frame_Waypoint_Len) == 2268);
const _: () = assert!(core::mem::offset_of!(bot_state_t, frame_Waypoint_Vis) == 2272);
const _: () = assert!(core::mem::offset_of!(bot_state_t, frame_Enemy_Len) == 2276);
const _: () = assert!(core::mem::offset_of!(bot_state_t, frame_Enemy_Vis) == 2280);
const _: () = assert!(core::mem::offset_of!(bot_state_t, isCamper) == 2284);
const _: () = assert!(core::mem::offset_of!(bot_state_t, isCamping) == 2288);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpCamping) == 2296);
const _: () = assert!(core::mem::offset_of!(bot_state_t, wpCampingTo) == 2304);
const _: () = assert!(core::mem::offset_of!(bot_state_t, campStanding) == 2312);
const _: () = assert!(core::mem::offset_of!(bot_state_t, randomNavTime) == 2316);
const _: () = assert!(core::mem::offset_of!(bot_state_t, randomNav) == 2320);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberSpecialist) == 2324);
const _: () = assert!(core::mem::offset_of!(bot_state_t, canChat) == 2328);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatFrequency) == 2332);
const _: () = assert!(core::mem::offset_of!(bot_state_t, currentChat) == 2336);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatTime) == 2464);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatTime_stored) == 2468);
const _: () = assert!(core::mem::offset_of!(bot_state_t, doChat) == 2472);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatTeam) == 2476);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatObject) == 2480);
const _: () = assert!(core::mem::offset_of!(bot_state_t, chatAltObject) == 2488);
const _: () = assert!(core::mem::offset_of!(bot_state_t, meleeStrafeTime) == 2496);
const _: () = assert!(core::mem::offset_of!(bot_state_t, meleeStrafeDir) == 2500);
const _: () = assert!(core::mem::offset_of!(bot_state_t, meleeStrafeDisable) == 2504);
const _: () = assert!(core::mem::offset_of!(bot_state_t, altChargeTime) == 2508);
const _: () = assert!(core::mem::offset_of!(bot_state_t, escapeDirTime) == 2512);
const _: () = assert!(core::mem::offset_of!(bot_state_t, dontGoBack) == 2516);
const _: () = assert!(core::mem::offset_of!(bot_state_t, doAttack) == 2520);
const _: () = assert!(core::mem::offset_of!(bot_state_t, doAltAttack) == 2524);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceWeaponSelect) == 2528);
const _: () = assert!(core::mem::offset_of!(bot_state_t, virtualWeapon) == 2532);
const _: () = assert!(core::mem::offset_of!(bot_state_t, plantTime) == 2536);
const _: () = assert!(core::mem::offset_of!(bot_state_t, plantDecided) == 2540);
const _: () = assert!(core::mem::offset_of!(bot_state_t, plantContinue) == 2544);
const _: () = assert!(core::mem::offset_of!(bot_state_t, plantKillEmAll) == 2548);
const _: () = assert!(core::mem::offset_of!(bot_state_t, runningLikeASissy) == 2552);
const _: () = assert!(core::mem::offset_of!(bot_state_t, runningToEscapeThreat) == 2556);
const _: () = assert!(core::mem::offset_of!(bot_state_t, skills) == 2560);
const _: () = assert!(core::mem::offset_of!(bot_state_t, loved) == 2584);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lovednum) == 2856);
const _: () = assert!(core::mem::offset_of!(bot_state_t, loved_death_thresh) == 2860);
const _: () = assert!(core::mem::offset_of!(bot_state_t, deathActivitiesDone) == 2864);
const _: () = assert!(core::mem::offset_of!(bot_state_t, botWeaponWeights) == 2868);
const _: () = assert!(core::mem::offset_of!(bot_state_t, ctfState) == 2944);
const _: () = assert!(core::mem::offset_of!(bot_state_t, siegeState) == 2948);
const _: () = assert!(core::mem::offset_of!(bot_state_t, teamplayState) == 2952);
const _: () = assert!(core::mem::offset_of!(bot_state_t, jmState) == 2956);
const _: () = assert!(core::mem::offset_of!(bot_state_t, state_Forced) == 2960);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberDefending) == 2964);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberDefendDecideTime) == 2968);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberBFTime) == 2972);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberBTime) == 2976);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberSTime) == 2980);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberThrowTime) == 2984);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberPower) == 2988);
const _: () = assert!(core::mem::offset_of!(bot_state_t, saberPowerTime) == 2992);
const _: () = assert!(core::mem::offset_of!(bot_state_t, botChallengingTime) == 2996);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceinfo) == 3000);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceJumpChargeTime) == 5048);
const _: () = assert!(core::mem::offset_of!(bot_state_t, doForcePush) == 5052);
const _: () = assert!(core::mem::offset_of!(bot_state_t, noUseTime) == 5056);
const _: () = assert!(core::mem::offset_of!(bot_state_t, doingFallback) == 5060);
const _: () = assert!(core::mem::offset_of!(bot_state_t, iHaveNoIdeaWhereIAmGoing) == 5064);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastSignificantAreaChange) == 5068);
const _: () = assert!(core::mem::offset_of!(bot_state_t, lastSignificantChangeTime) == 5080);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceMove_Forward) == 5084);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceMove_Right) == 5088);
const _: () = assert!(core::mem::offset_of!(bot_state_t, forceMove_Up) == 5092);
