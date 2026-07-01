//! MP `entityState_t` copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:2662-2832`
//! Xbox packed variant source: `oracle/oracle/codemp/game/q_shared.h:2841-2985`

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::{qboolean, trajectory_t, vec3_t};

/// Raven MP `entityState_t`.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2670-2832`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct entityState_t {
    /// Entity index.
    pub number: c_int,
    /// entityType_t.
    pub eType: c_int,
    pub eFlags: c_int,
    /// EF2_??? used much less frequently.
    pub eFlags2: c_int,
    /// For calculating position.
    pub pos: trajectory_t,
    /// For calculating angles.
    pub apos: trajectory_t,
    pub time: c_int,
    pub time2: c_int,
    pub origin: vec3_t,
    pub origin2: vec3_t,
    pub angles: vec3_t,
    pub angles2: vec3_t,
    /// Generic Ghoul2/shared value.
    pub bolt1: c_int,
    /// Generic Ghoul2/shared value.
    pub bolt2: c_int,
    /// Jedi mindtrick visibility index 0-15.
    pub trickedentindex: c_int,
    /// Jedi mindtrick visibility index 16-32.
    pub trickedentindex2: c_int,
    /// Jedi mindtrick visibility index 33-48.
    pub trickedentindex3: c_int,
    /// Jedi mindtrick visibility index 49-64.
    pub trickedentindex4: c_int,
    pub speed: f32,
    pub fireflag: c_int,
    pub genericenemyindex: c_int,
    pub activeForcePass: c_int,
    pub emplacedOwner: c_int,
    /// Shotgun sources, etc.
    pub otherEntityNum: c_int,
    pub otherEntityNum2: c_int,
    /// -1 = in air.
    pub groundEntityNum: c_int,
    /// r + (g<<8) + (b<<16) + (intensity<<24).
    pub constantLight: c_int,
    /// Constantly loop this sound.
    pub loopSound: c_int,
    /// qtrue if the loopSound index is actually a soundset index.
    pub loopIsSoundset: qboolean,
    pub soundSetIndex: c_int,
    pub modelGhoul2: c_int,
    pub g2radius: c_int,
    pub modelindex: c_int,
    pub modelindex2: c_int,
    /// 0 to (MAX_CLIENTS - 1), for players and corpses.
    pub clientNum: c_int,
    pub frame: c_int,
    pub saberInFlight: qboolean,
    pub saberEntityNum: c_int,
    pub saberMove: c_int,
    pub forcePowersActive: c_int,
    /// Sent in only only 2 bits - should be 0, 1 or 2.
    pub saberHolstered: c_int,
    pub isJediMaster: qboolean,
    pub isPortalEnt: qboolean,
    /// For client side prediction, trap_linkentity sets this properly.
    pub solid: c_int,
    /// Impulse events -- muzzle flashes, footsteps, etc.
    pub event: c_int,
    pub eventParm: c_int,
    /// So crosshair knows what it's looking at.
    pub owner: c_int,
    pub teamowner: c_int,
    pub shouldtarget: qboolean,
    /// Bit flags.
    pub powerups: c_int,
    /// Determines weapon and flash model, etc.
    pub weapon: c_int,
    pub legsAnim: c_int,
    pub torsoAnim: c_int,
    pub legsFlip: qboolean,
    pub torsoFlip: qboolean,
    /// If non-zero, force the anim frame.
    pub forceFrame: c_int,
    pub generic1: c_int,
    pub heldByClient: c_int,
    pub ragAttach: c_int,
    pub iModelScale: c_int,
    pub brokenLimbs: c_int,
    pub boltToPlayer: c_int,
    pub hasLookTarget: qboolean,
    pub lookTarget: c_int,
    pub customRGBA: [c_int; 4],
    pub health: c_int,
    pub maxhealth: c_int,
    pub npcSaber1: c_int,
    pub npcSaber2: c_int,
    pub csSounds_Std: c_int,
    pub csSounds_Combat: c_int,
    pub csSounds_Extra: c_int,
    pub csSounds_Jedi: c_int,
    pub surfacesOn: c_int,
    pub surfacesOff: c_int,
    pub boneIndex1: c_int,
    pub boneIndex2: c_int,
    pub boneIndex3: c_int,
    pub boneIndex4: c_int,
    pub boneOrient: c_int,
    pub boneAngles1: vec3_t,
    pub boneAngles2: vec3_t,
    pub boneAngles3: vec3_t,
    pub boneAngles4: vec3_t,
    pub NPC_class: c_int,
    pub m_iVehicleNum: c_int,
    pub userInt1: c_int,
    pub userInt2: c_int,
    pub userInt3: c_int,
    pub userFloat1: f32,
    pub userFloat2: f32,
    pub userFloat3: f32,
    pub userVec1: vec3_t,
    pub userVec2: vec3_t,
}

const _: () = assert!(core::mem::size_of::<entityState_t>() == 532);
const _: () = assert!(core::mem::offset_of!(entityState_t, number) == 0);
const _: () = assert!(core::mem::offset_of!(entityState_t, pos) == 16);
const _: () = assert!(core::mem::offset_of!(entityState_t, origin) == 96);
const _: () = assert!(core::mem::offset_of!(entityState_t, customRGBA) == 352);
const _: () = assert!(core::mem::offset_of!(entityState_t, boneAngles1) == 428);
const _: () = assert!(core::mem::offset_of!(entityState_t, userVec1) == 508);
