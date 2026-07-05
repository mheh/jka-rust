#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_schar;

use mp_qshared::common::mp::entity_id::EntityId;
use mp_qshared::common::mp::qcommon::b_state_t::bState_t;
use mp_qshared::common::mp::qcommon::usercmd_t;
use mp_qshared::shared::{qboolean, vec3_t};

use crate::ai::AIGroupInfo_t;

use super::g_npcstats_e::gNPCstats_t;
use super::jump_state_t::jumpState_t;
use super::visibility_t::visibility_t;

/// Raven `MAX_ENEMY_POS_LAG`.
///
/// Definition source: `oracle/oracle/codemp/game/b_public.h:113`
pub const MAX_ENEMY_POS_LAG: i32 = 2400;

/// Raven `ENEMY_POS_LAG_INTERVAL`.
///
/// Definition source: `oracle/oracle/codemp/game/b_public.h:114`
pub const ENEMY_POS_LAG_INTERVAL: i32 = 100;

/// Raven `ENEMY_POS_LAG_STEPS`.
///
/// Definition source: `oracle/oracle/codemp/game/b_public.h:115`
pub const ENEMY_POS_LAG_STEPS: usize = (MAX_ENEMY_POS_LAG / ENEMY_POS_LAG_INTERVAL) as usize;

/// Raven `gNPC_t` — per-entity NPC/behavior state.
///
/// Type definition source: `oracle/oracle/codemp/game/b_public.h:116-264`
#[repr(C)]
pub struct gNPC_t {
	//FIXME: Put in playerInfo or something
	/// FIXME do we really need both of these
	pub timeOfDeath: i32,
	pub touchedByPlayer: Option<EntityId>,

	pub enemyLastVisibility: visibility_t,

	pub aimTime: i32,
	pub desiredYaw: f32,
	pub desiredPitch: f32,
	pub lockedDesiredYaw: f32,
	pub lockedDesiredPitch: f32,
	/// debugging aid
	pub aimingBeam: Option<EntityId>,

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
	// Source: oracle/oracle/codemp/game/ai.h:29-41
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
	pub eventOwner: Option<EntityId>,

	//bState-specific fields
	pub coverTarg: Option<EntityId>,
	pub jumpState: jumpState_t,
	pub followDist: f32,

	// goal, navigation & pathfinding
	/// used for locational goals (player's last seen/heard position)
	pub tempGoal: Option<EntityId>,
	pub goalEntity: Option<EntityId>,
	pub lastGoalEntity: Option<EntityId>,
	pub eventualGoal: Option<EntityId>,
	/// Where we should try to capture
	pub captureGoal: Option<EntityId>,
	/// Who we're trying to protect
	pub defendEnt: Option<EntityId>,
	/// Who we're greeting
	pub greetEnt: Option<EntityId>,
	/// FIXME: This is never actually used
	pub goalTime: i32,
	/// move straight at navgoals
	pub straightToGoal: qboolean,
	pub distToGoal: f32,
	pub navTime: i32,
	pub blockingEntNum: i32,
	pub blockedSpeechDebounceTime: i32,
	pub lastSideStepSide: i32,
	pub sideStepHoldTime: i32,
	pub homeWp: i32,
	pub group: *mut AIGroupInfo_t,

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
	pub consecutiveBlockedMoves: i32,
	pub blockedDebounceTime: i32,
	pub shoveCount: i32,
	pub blockedDest: vec3_t,

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

	/// Lagging enemy position - FIXME: seems awful wasteful...
	pub enemyLaggedPos: [vec3_t; ENEMY_POS_LAG_STEPS],

	/// for BS_CINEMATIC, keeps facing this ent
	pub watchTarget: Option<EntityId>,

	/// sigh... you'd think I'd be able to find a way to do this without having to use 3 int fields, but...
	pub ffireCount: i32,
	pub ffireDebounce: i32,
	pub ffireFadeDebounce: i32,
}

const _: () = assert!(core::mem::offset_of!(gNPC_t, timeOfDeath) == 0);
// This struct's stored `gentity_t*` fields became `Option<EntityId>` (align 4 vs a
// pointer's align 8), so the private tail's byte offsets shift. This struct is
// game-internal / not ABI-fixed beyond its prefix — the engine learns the full
// stride at runtime via `trap_LocateGameData`. The `size_of` assert and every
// `offset_of` assert at/after the first flipped field are therefore dropped;
// only the fixed-prefix asserts above (declared before the first flip) remain.
