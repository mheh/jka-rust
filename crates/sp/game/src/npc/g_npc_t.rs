#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_schar;

use sp_qshared::common::sp::gentity::gentity_t;
use sp_qshared::common::sp::qcommon::usercmd::usercmd_t;
use sp_qshared::shared::{qboolean, vec3_t};

use crate::ai::AIGroupInfo_t;
use crate::bstate::b_state_t::bState_t;

use super::g_npcstats_e::gNPCstats_t;
use super::jump_state_t::jumpState_t;
use super::visibility_t::visibility_t;

/// Raven `MAX_ENEMY_POS_LAG`.
///
/// Definition source: `oracle/oracle/code/game/b_public.h:141`
pub const MAX_ENEMY_POS_LAG: i32 = 2400;

/// Raven `ENEMY_POS_LAG_INTERVAL`.
///
/// Definition source: `oracle/oracle/code/game/b_public.h:142`
pub const ENEMY_POS_LAG_INTERVAL: i32 = 100;

/// Raven `ENEMY_POS_LAG_STEPS`.
///
/// Definition source: `oracle/oracle/code/game/b_public.h:143`
pub const ENEMY_POS_LAG_STEPS: usize = (MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL) as usize;

/// Raven `gNPC_t` — per-entity NPC/behavior state.
///
/// Type definition source: `oracle/oracle/code/game/b_public.h:146-313`
#[repr(C)]
pub struct gNPC_t {
	//FIXME: Put in playerInfo or something
	/// FIXME do we really need both of these
	pub timeOfDeath: i32,
	pub touchedByPlayer: *mut gentity_t,

	pub enemyLastVisibility: visibility_t,

	pub aimTime: i32,
	pub desiredYaw: f32,
	pub desiredPitch: f32,
	pub lockedDesiredYaw: f32,
	pub lockedDesiredPitch: f32,
	/// debugging aid
	pub aimingBeam: *mut gentity_t,

	pub enemyLastSeenLocation: vec3_t,
	pub enemyLastSeenTime: i32,
	pub enemyLastHeardLocation: vec3_t,
	pub enemyLastHeardTime: i32,
	/// unique ID
	pub lastAlertID: i32,

	pub eFlags: i32,
	pub aiFlags: i32,

	/// this sucks, need to find a better way
	pub currentAmmo: i32,
	pub shotTime: i32,
	pub burstCount: i32,
	pub burstMin: i32,
	pub burstMean: i32,
	pub burstMax: i32,
	pub burstSpacing: i32,
	pub attackHold: i32,
	pub attackHoldTime: i32,
	/// Angles to where bot is shooting - fixme: make he torso turn to reflect these
	pub shootAngles: vec3_t,

	//extra character info
	//TODO: Port rank_t
	// Source: oracle/oracle/code/game/ai.h
	/// for pips
	pub rank: i32,

	//Behavior state info
	/// determines what actions he should be doing
	pub behaviorState: bState_t,
	/// State bot will default to if none other set
	pub defaultBehavior: bState_t,
	/// While valid, overrides other behavior
	pub tempBehavior: bState_t,

	/// only play pain scripts when take pain
	pub ignorePain: qboolean,

	/// Keeps them ducked for a certain time
	pub duckDebounceTime: i32,
	pub walkDebounceTime: i32,
	pub enemyCheckDebounceTime: i32,
	pub investigateDebounceTime: i32,
	pub investigateCount: i32,
	pub investigateGoal: vec3_t,
	pub investigateSoundDebounceTime: i32,
	/// when we can greet someone next
	pub greetingDebounceTime: i32,
	pub eventOwner: *mut gentity_t,

	//bState-specific fields
	pub coverTarg: *mut gentity_t,
	pub jumpState: jumpState_t,
	pub followDist: f32,

	// goal, navigation & pathfinding
	/// used for locational goals (player's last seen/heard position)
	pub tempGoal: *mut gentity_t,
	pub goalEntity: *mut gentity_t,
	pub lastGoalEntity: *mut gentity_t,
	pub eventualGoal: *mut gentity_t,
	/// Where we should try to capture
	pub captureGoal: *mut gentity_t,
	/// Who we're trying to protect
	pub defendEnt: *mut gentity_t,
	/// Who we're greeting
	pub greetEnt: *mut gentity_t,
	/// FIXME: This is never actually used
	pub goalTime: i32,
	/// move straight at navgoals
	pub straightToGoal: qboolean,
	pub distToGoal: f32,
	pub navTime: i32,
	pub blockingEntNum: i32,
	pub blockedSpeechDebounceTime: i32,

	pub homeWp: i32,
	pub avoidSide: i32,
	pub leaderAvoidSide: i32,
	pub lastAvoidSteerSide: i32,
	pub lastAvoidSteerSideDebouncer: i32,
	pub group: *mut AIGroupInfo_t,
	pub troop: i32,

	/// So we know which way to face generally when we stop
	pub lastPathAngles: vec3_t,

	//stats
	pub stats: gNPCstats_t,
	pub aimErrorDebounceTime: i32,
	pub lastAimErrorYaw: f32,
	pub lastAimErrorPitch: f32,
	pub aimOfs: vec3_t,
	pub currentAim: i32,
	pub currentAggression: i32,

	//scriptflags
	/// in b_local.h
	pub scriptFlags: i32,

	//moveInfo
	pub desiredSpeed: i32,
	pub currentSpeed: i32,
	pub last_forwardmove: c_schar,
	pub last_rightmove: c_schar,
	pub lastClearOrigin: vec3_t,
	pub shoveCount: i32,

	pub blockedDebounceTime: i32,
	/// The entity That Causes The Current Blockage
	pub blockedEntity: *mut gentity_t,

	/// Where the actor was trying to get TO before blocked
	pub blockedTargetPosition: vec3_t,
	/// Where the actor was trying to get TO before blocked
	pub blockedTargetEntity: *mut gentity_t,

	//jump info
	/// Where The Actor Is Trying To Jump TO
	pub jumpDest: vec3_t,
	/// What Entity The Actor Is Trying To Jump TO
	pub jumpTarget: *mut gentity_t,
	/// The Minimal Delta On The XY Plane Allowed To Jump To The Dest
	pub jumpMaxXYDist: f32,
	pub jumpMazZDist: f32,
	/// Which Side The Last Jump Occured On
	pub jumpSide: i32,
	/// When The Last Jump Started
	pub jumpTime: i32,
	/// If Active, Then The Guy Should Backup Before Jumping
	pub jumpBackupTime: i32,
	/// The Minimal Next Time To Check For A Jump
	pub jumpNextCheckTime: i32,

	//
	/// NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
	pub combatPoint: i32,
	/// NPCs in bState BS_COMBAT_POINT will find their closest empty combat_point
	pub lastFailedCombatPoint: i32,
	/// what to say when you first successfully move
	pub movementSpeech: i32,
	/// how likely you are to say it
	pub movementSpeechChance: f32,

	//Testing physics at 20fps
	pub nextBStateThink: i32,
	pub last_ucmd: usercmd_t,

	//
	//JWEIER ADDITIONS START
	pub combatMove: qboolean,
	pub goalRadius: i32,

	//FIXME: These may be redundant
	/*
	int			weaponTime;		//Time until refire is valid
	int			jumpTime;
	*/
	/// Time to stand still
	pub pauseTime: i32,
	pub standTime: i32,

	/// Tracking information local to entity
	pub localState: i32,
	/// Tracking information for team level interaction
	pub squadState: i32,

	//JWEIER ADDITIONS END
	//
	/// Doesn't respond to alerts or pick up enemies (unless shot) until this time is up
	pub confusionTime: i32,
	/// charmed to enemy team
	pub charmedTime: i32,
	/// controlled by player
	pub controlledTime: i32,
	/// Hands up
	pub surrenderTime: i32,
	/// kneeling (for troopers)
	pub kneelTime: i32,

	/// Lagging enemy position - FIXME: seems awful wasteful...
	pub enemyLaggedPos: [vec3_t; ENEMY_POS_LAG_STEPS],

	/// for BS_CINEMATIC, keeps facing this ent
	pub watchTarget: *mut gentity_t,

	/// sigh... you'd think I'd be able to find a way to do this without having to use 3 int fields, but...
	pub ffireCount: i32,
	pub ffireDebounce: i32,
	pub ffireFadeDebounce: i32,
}

const _: () = assert!(core::mem::size_of::<gNPC_t>() == 984);
const _: () = assert!(core::mem::offset_of!(gNPC_t, timeOfDeath) == 0);
const _: () = assert!(core::mem::offset_of!(gNPC_t, touchedByPlayer) == 8);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastVisibility) == 16);
const _: () = assert!(core::mem::offset_of!(gNPC_t, aimTime) == 20);
const _: () = assert!(core::mem::offset_of!(gNPC_t, desiredYaw) == 24);
const _: () = assert!(core::mem::offset_of!(gNPC_t, desiredPitch) == 28);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lockedDesiredYaw) == 32);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lockedDesiredPitch) == 36);
const _: () = assert!(core::mem::offset_of!(gNPC_t, aimingBeam) == 40);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastSeenLocation) == 48);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastSeenTime) == 60);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastHeardLocation) == 64);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLastHeardTime) == 76);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastAlertID) == 80);
const _: () = assert!(core::mem::offset_of!(gNPC_t, eFlags) == 84);
const _: () = assert!(core::mem::offset_of!(gNPC_t, aiFlags) == 88);
const _: () = assert!(core::mem::offset_of!(gNPC_t, currentAmmo) == 92);
const _: () = assert!(core::mem::offset_of!(gNPC_t, shotTime) == 96);
const _: () = assert!(core::mem::offset_of!(gNPC_t, burstCount) == 100);
const _: () = assert!(core::mem::offset_of!(gNPC_t, burstMin) == 104);
const _: () = assert!(core::mem::offset_of!(gNPC_t, burstMean) == 108);
const _: () = assert!(core::mem::offset_of!(gNPC_t, burstMax) == 112);
const _: () = assert!(core::mem::offset_of!(gNPC_t, burstSpacing) == 116);
const _: () = assert!(core::mem::offset_of!(gNPC_t, attackHold) == 120);
const _: () = assert!(core::mem::offset_of!(gNPC_t, attackHoldTime) == 124);
const _: () = assert!(core::mem::offset_of!(gNPC_t, shootAngles) == 128);
const _: () = assert!(core::mem::offset_of!(gNPC_t, rank) == 140);
const _: () = assert!(core::mem::offset_of!(gNPC_t, behaviorState) == 144);
const _: () = assert!(core::mem::offset_of!(gNPC_t, defaultBehavior) == 148);
const _: () = assert!(core::mem::offset_of!(gNPC_t, tempBehavior) == 152);
const _: () = assert!(core::mem::offset_of!(gNPC_t, ignorePain) == 156);
const _: () = assert!(core::mem::offset_of!(gNPC_t, duckDebounceTime) == 160);
const _: () = assert!(core::mem::offset_of!(gNPC_t, walkDebounceTime) == 164);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyCheckDebounceTime) == 168);
const _: () = assert!(core::mem::offset_of!(gNPC_t, investigateDebounceTime) == 172);
const _: () = assert!(core::mem::offset_of!(gNPC_t, investigateCount) == 176);
const _: () = assert!(core::mem::offset_of!(gNPC_t, investigateGoal) == 180);
const _: () = assert!(core::mem::offset_of!(gNPC_t, investigateSoundDebounceTime) == 192);
const _: () = assert!(core::mem::offset_of!(gNPC_t, greetingDebounceTime) == 196);
const _: () = assert!(core::mem::offset_of!(gNPC_t, eventOwner) == 200);
const _: () = assert!(core::mem::offset_of!(gNPC_t, coverTarg) == 208);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpState) == 216);
const _: () = assert!(core::mem::offset_of!(gNPC_t, followDist) == 220);
const _: () = assert!(core::mem::offset_of!(gNPC_t, tempGoal) == 224);
const _: () = assert!(core::mem::offset_of!(gNPC_t, goalEntity) == 232);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastGoalEntity) == 240);
const _: () = assert!(core::mem::offset_of!(gNPC_t, eventualGoal) == 248);
const _: () = assert!(core::mem::offset_of!(gNPC_t, captureGoal) == 256);
const _: () = assert!(core::mem::offset_of!(gNPC_t, defendEnt) == 264);
const _: () = assert!(core::mem::offset_of!(gNPC_t, greetEnt) == 272);
const _: () = assert!(core::mem::offset_of!(gNPC_t, goalTime) == 280);
const _: () = assert!(core::mem::offset_of!(gNPC_t, straightToGoal) == 284);
const _: () = assert!(core::mem::offset_of!(gNPC_t, distToGoal) == 288);
const _: () = assert!(core::mem::offset_of!(gNPC_t, navTime) == 292);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockingEntNum) == 296);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockedSpeechDebounceTime) == 300);
const _: () = assert!(core::mem::offset_of!(gNPC_t, homeWp) == 304);
const _: () = assert!(core::mem::offset_of!(gNPC_t, avoidSide) == 308);
const _: () = assert!(core::mem::offset_of!(gNPC_t, leaderAvoidSide) == 312);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastAvoidSteerSide) == 316);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastAvoidSteerSideDebouncer) == 320);
const _: () = assert!(core::mem::offset_of!(gNPC_t, group) == 328);
const _: () = assert!(core::mem::offset_of!(gNPC_t, troop) == 336);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastPathAngles) == 340);
const _: () = assert!(core::mem::offset_of!(gNPC_t, stats) == 352);
const _: () = assert!(core::mem::offset_of!(gNPC_t, aimErrorDebounceTime) == 424);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastAimErrorYaw) == 428);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastAimErrorPitch) == 432);
const _: () = assert!(core::mem::offset_of!(gNPC_t, aimOfs) == 436);
const _: () = assert!(core::mem::offset_of!(gNPC_t, currentAim) == 448);
const _: () = assert!(core::mem::offset_of!(gNPC_t, currentAggression) == 452);
const _: () = assert!(core::mem::offset_of!(gNPC_t, scriptFlags) == 456);
const _: () = assert!(core::mem::offset_of!(gNPC_t, desiredSpeed) == 460);
const _: () = assert!(core::mem::offset_of!(gNPC_t, currentSpeed) == 464);
const _: () = assert!(core::mem::offset_of!(gNPC_t, last_forwardmove) == 468);
const _: () = assert!(core::mem::offset_of!(gNPC_t, last_rightmove) == 469);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastClearOrigin) == 472);
const _: () = assert!(core::mem::offset_of!(gNPC_t, shoveCount) == 484);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockedDebounceTime) == 488);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockedEntity) == 496);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockedTargetPosition) == 504);
const _: () = assert!(core::mem::offset_of!(gNPC_t, blockedTargetEntity) == 520);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpDest) == 528);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpTarget) == 544);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpMaxXYDist) == 552);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpMazZDist) == 556);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpSide) == 560);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpTime) == 564);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpBackupTime) == 568);
const _: () = assert!(core::mem::offset_of!(gNPC_t, jumpNextCheckTime) == 572);
const _: () = assert!(core::mem::offset_of!(gNPC_t, combatPoint) == 576);
const _: () = assert!(core::mem::offset_of!(gNPC_t, lastFailedCombatPoint) == 580);
const _: () = assert!(core::mem::offset_of!(gNPC_t, movementSpeech) == 584);
const _: () = assert!(core::mem::offset_of!(gNPC_t, movementSpeechChance) == 588);
const _: () = assert!(core::mem::offset_of!(gNPC_t, nextBStateThink) == 592);
const _: () = assert!(core::mem::offset_of!(gNPC_t, last_ucmd) == 596);
const _: () = assert!(core::mem::offset_of!(gNPC_t, combatMove) == 624);
const _: () = assert!(core::mem::offset_of!(gNPC_t, goalRadius) == 628);
const _: () = assert!(core::mem::offset_of!(gNPC_t, pauseTime) == 632);
const _: () = assert!(core::mem::offset_of!(gNPC_t, standTime) == 636);
const _: () = assert!(core::mem::offset_of!(gNPC_t, localState) == 640);
const _: () = assert!(core::mem::offset_of!(gNPC_t, squadState) == 644);
const _: () = assert!(core::mem::offset_of!(gNPC_t, confusionTime) == 648);
const _: () = assert!(core::mem::offset_of!(gNPC_t, charmedTime) == 652);
const _: () = assert!(core::mem::offset_of!(gNPC_t, controlledTime) == 656);
const _: () = assert!(core::mem::offset_of!(gNPC_t, surrenderTime) == 660);
const _: () = assert!(core::mem::offset_of!(gNPC_t, kneelTime) == 664);
const _: () = assert!(core::mem::offset_of!(gNPC_t, enemyLaggedPos) == 668);
const _: () = assert!(core::mem::offset_of!(gNPC_t, watchTarget) == 960);
const _: () = assert!(core::mem::offset_of!(gNPC_t, ffireCount) == 968);
const _: () = assert!(core::mem::offset_of!(gNPC_t, ffireDebounce) == 972);
const _: () = assert!(core::mem::offset_of!(gNPC_t, ffireFadeDebounce) == 976);
