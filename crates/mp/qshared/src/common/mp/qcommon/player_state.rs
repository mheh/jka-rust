//! MP `playerState_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/codemp/game/q_shared.h:2068-2123`
//! Source: `oracle/codemp/game/q_shared.h:2159-2435`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::{qboolean, vec3_t};

pub const NUM_FORCE_POWERS: usize = 18;
/// Raven `TRACK_CHANNEL_MAX` = NUM_TRACK_CHANNELS(56) - 50.
/// Source: `oracle/codemp/game/q_shared.h:2066`
pub const TRACK_CHANNEL_MAX: usize = 6;
pub const MAX_STATS: usize = 16;
pub const MAX_PERSISTANT: usize = 16;
pub const MAX_POWERUPS: usize = 16;
pub const MAX_WEAPONS: usize = 19;
pub const MAX_PS_EVENTS: usize = 2;

/// Raven MP `forcedata_t`.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2068-2123`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct forcedata_t {
    pub forcePowerDebounce: [c_int; NUM_FORCE_POWERS],
    pub forcePowersKnown: c_int,
    pub forcePowersActive: c_int,
    pub forcePowerSelected: c_int,
    pub forceButtonNeedRelease: c_int,
    pub forcePowerDuration: [c_int; NUM_FORCE_POWERS],
    pub forcePower: c_int,
    pub forcePowerMax: c_int,
    pub forcePowerRegenDebounceTime: c_int,
    pub forcePowerLevel: [c_int; NUM_FORCE_POWERS],
    pub forcePowerBaseLevel: [c_int; NUM_FORCE_POWERS],
    pub forceUsingAdded: c_int,
    pub forceJumpZStart: f32,
    pub forceJumpCharge: f32,
    pub forceJumpSound: c_int,
    pub forceJumpAddTime: c_int,
    pub forceGripEntityNum: c_int,
    pub forceGripDamageDebounceTime: c_int,
    pub forceGripBeingGripped: f32,
    pub forceGripCripple: c_int,
    pub forceGripUseTime: c_int,
    pub forceGripSoundTime: f32,
    pub forceGripStarted: f32,
    pub forceHealTime: c_int,
    pub forceHealAmount: c_int,
    pub forceMindtrickTargetIndex: c_int,
    pub forceMindtrickTargetIndex2: c_int,
    pub forceMindtrickTargetIndex3: c_int,
    pub forceMindtrickTargetIndex4: c_int,
    pub forceRageRecoveryTime: c_int,
    pub forceDrainEntNum: c_int,
    pub forceDrainTime: f32,
    pub forceDoInit: c_int,
    pub forceSide: c_int,
    pub forceRank: c_int,
    pub forceDeactivateAll: c_int,
    pub killSoundEntIndex: [c_int; TRACK_CHANNEL_MAX],
    pub sentryDeployed: qboolean,
    pub saberAnimLevelBase: c_int,
    pub saberAnimLevel: c_int,
    pub saberDrawAnimLevel: c_int,
    pub suicides: c_int,
    pub privateDuelTime: c_int,
}

/// Raven MP `playerState_t`.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2169-2435`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct playerState_t {
    pub commandTime: c_int,
    pub pm_type: c_int,
    pub bobCycle: c_int,
    pub pm_flags: c_int,
    pub pm_time: c_int,
    pub origin: vec3_t,
    pub velocity: vec3_t,
    /// NOT sent over the net.
    pub moveDir: vec3_t,
    pub weaponTime: c_int,
    pub weaponChargeTime: c_int,
    pub weaponChargeSubtractTime: c_int,
    pub gravity: c_int,
    pub speed: f32,
    pub basespeed: c_int,
    pub delta_angles: [c_int; 3],
    pub slopeRecalcTime: c_int,
    pub useTime: c_int,
    pub groundEntityNum: c_int,
    pub legsTimer: c_int,
    pub legsAnim: c_int,
    pub torsoTimer: c_int,
    pub torsoAnim: c_int,
    pub legsFlip: qboolean,
    pub torsoFlip: qboolean,
    pub movementDir: c_int,
    pub eFlags: c_int,
    pub eFlags2: c_int,
    pub eventSequence: c_int,
    pub events: [c_int; MAX_PS_EVENTS],
    pub eventParms: [c_int; MAX_PS_EVENTS],
    pub externalEvent: c_int,
    pub externalEventParm: c_int,
    pub externalEventTime: c_int,
    pub clientNum: c_int,
    pub weapon: c_int,
    pub weaponstate: c_int,
    pub viewangles: vec3_t,
    pub viewheight: c_int,
    pub damageEvent: c_int,
    pub damageYaw: c_int,
    pub damagePitch: c_int,
    pub damageCount: c_int,
    pub damageType: c_int,
    pub painTime: c_int,
    pub painDirection: c_int,
    pub yawAngle: f32,
    pub yawing: qboolean,
    pub pitchAngle: f32,
    pub pitching: qboolean,
    pub stats: [c_int; MAX_STATS],
    pub persistant: [c_int; MAX_PERSISTANT],
    pub powerups: [c_int; MAX_POWERUPS],
    pub ammo: [c_int; MAX_WEAPONS],
    pub generic1: c_int,
    pub loopSound: c_int,
    pub jumppad_ent: c_int,
    pub ping: c_int,
    pub pmove_framecount: c_int,
    pub jumppad_frame: c_int,
    pub entityEventSequence: c_int,
    pub lastOnGround: c_int,
    pub saberInFlight: qboolean,
    pub saberMove: c_int,
    pub saberBlocking: c_int,
    pub saberBlocked: c_int,
    pub saberLockTime: c_int,
    pub saberLockEnemy: c_int,
    pub saberLockFrame: c_int,
    pub saberLockHits: c_int,
    pub saberLockHitCheckTime: c_int,
    pub saberLockHitIncrementTime: c_int,
    pub saberLockAdvance: qboolean,
    pub saberEntityNum: c_int,
    pub saberEntityDist: f32,
    pub saberEntityState: c_int,
    pub saberThrowDelay: c_int,
    pub saberCanThrow: qboolean,
    pub saberDidThrowTime: c_int,
    pub saberDamageDebounceTime: c_int,
    pub saberHitWallSoundDebounceTime: c_int,
    pub saberEventFlags: c_int,
    pub rocketLockIndex: c_int,
    pub rocketLastValidTime: f32,
    pub rocketLockTime: f32,
    pub rocketTargetTime: f32,
    pub emplacedIndex: c_int,
    pub emplacedTime: f32,
    pub isJediMaster: qboolean,
    pub forceRestricted: qboolean,
    pub trueJedi: qboolean,
    pub trueNonJedi: qboolean,
    pub saberIndex: c_int,
    pub genericEnemyIndex: c_int,
    pub droneFireTime: f32,
    pub droneExistTime: f32,
    pub activeForcePass: c_int,
    pub hasDetPackPlanted: qboolean,
    pub holocronsCarried: [f32; NUM_FORCE_POWERS],
    pub holocronCantTouch: c_int,
    pub holocronCantTouchTime: f32,
    pub holocronBits: c_int,
    pub electrifyTime: c_int,
    pub saberAttackSequence: c_int,
    pub saberIdleWound: c_int,
    pub saberAttackWound: c_int,
    pub saberBlockTime: c_int,
    pub otherKiller: c_int,
    pub otherKillerTime: c_int,
    pub otherKillerDebounceTime: c_int,
    pub fd: forcedata_t,
    pub forceJumpFlip: qboolean,
    pub forceHandExtend: c_int,
    pub forceHandExtendTime: c_int,
    pub forceRageDrainTime: c_int,
    pub forceDodgeAnim: c_int,
    pub quickerGetup: qboolean,
    pub groundTime: c_int,
    pub footstepTime: c_int,
    pub otherSoundTime: c_int,
    pub otherSoundLen: f32,
    pub forceGripMoveInterval: c_int,
    pub forceGripChangeMovetype: c_int,
    pub forceKickFlip: c_int,
    pub duelIndex: c_int,
    pub duelTime: c_int,
    pub duelInProgress: qboolean,
    pub saberAttackChainCount: c_int,
    pub saberHolstered: c_int,
    pub forceAllowDeactivateTime: c_int,
    pub zoomMode: c_int,
    pub zoomTime: c_int,
    pub zoomLocked: qboolean,
    pub zoomFov: f32,
    pub zoomLockTime: c_int,
    pub fallingToDeath: c_int,
    pub useDelay: c_int,
    pub inAirAnim: qboolean,
    pub lastHitLoc: vec3_t,
    pub heldByClient: c_int,
    pub ragAttach: c_int,
    pub iModelScale: c_int,
    pub brokenLimbs: c_int,
    pub hasLookTarget: qboolean,
    pub lookTarget: c_int,
    pub customRGBA: [c_int; 4],
    pub standheight: c_int,
    pub crouchheight: c_int,
    pub m_iVehicleNum: c_int,
    pub vehOrientation: vec3_t,
    pub vehBoarding: qboolean,
    pub vehSurfaces: c_int,
    pub vehTurnaroundIndex: c_int,
    pub vehTurnaroundTime: c_int,
    pub vehWeaponsLinked: qboolean,
    pub hyperSpaceTime: c_int,
    pub hyperSpaceAngles: vec3_t,
    pub hackingTime: c_int,
    pub hackingBaseTime: c_int,
    pub jetpackFuel: c_int,
    pub cloakFuel: c_int,
    pub userInt1: c_int,
    pub userInt2: c_int,
    pub userInt3: c_int,
    pub userFloat1: f32,
    pub userFloat2: f32,
    pub userFloat3: f32,
    pub userVec1: vec3_t,
    pub userVec2: vec3_t,
}

const _: () = assert!(core::mem::size_of::<forcedata_t>() == 464);
const _: () = assert!(core::mem::offset_of!(forcedata_t, forcePowerDebounce) == 0);
const _: () = assert!(core::mem::offset_of!(forcedata_t, forcePowersKnown) == 72);
const _: () = assert!(core::mem::offset_of!(forcedata_t, forcePowerDuration) == 88);
const _: () = assert!(core::mem::offset_of!(forcedata_t, forcePowerLevel) == 172);
const _: () = assert!(core::mem::offset_of!(forcedata_t, killSoundEntIndex) == 416);
const _: () = assert!(core::mem::offset_of!(forcedata_t, privateDuelTime) == 460);

const _: () = assert!(core::mem::size_of::<playerState_t>() == 1552);
const _: () = assert!(core::mem::offset_of!(playerState_t, commandTime) == 0);
const _: () = assert!(core::mem::offset_of!(playerState_t, velocity) == 32);
const _: () = assert!(core::mem::offset_of!(playerState_t, fd) == 804);
const _: () = assert!(core::mem::offset_of!(playerState_t, forceJumpFlip) == 1268);
const _: () = assert!(core::mem::offset_of!(playerState_t, lastHitLoc) == 1376);
const _: () = assert!(core::mem::offset_of!(playerState_t, userVec2) == 1540);

/// Raven's `playerState_s` struct tag (elaborated `struct playerState_s *`
/// spellings in engine signatures resolve to the `playerState_t` typedef port).
pub type playerState_s = playerState_t;
