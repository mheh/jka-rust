#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use mp_bg::vehicles::vehicle_s::Vehicle_t;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::shared::{qboolean, vec3_t};

use super::cg_loop_sound_s::cgLoopSound_t;
use super::client_info_t::clientInfo_t;
use super::player_entity_t::playerEntity_t;

/// Raven `MAX_CG_LOOPSOUNDS`.
///
/// Source: `oracle/oracle/codemp/cgame/cg_local.h:321`
pub const MAX_CG_LOOPSOUNDS: usize = 8;

/// Raven `centity_t` — client-side representation of an entity, tracking
/// interpolation, animation, and effects state between snapshots.
///
/// Raven: This comment below is correct, but now m_pVehicle is the first
/// thing in bg shared entity, so it goes first. - AReis
/// rww - entstate must be first, to correspond with the bg shared entity
/// structure.
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:333-462`
#[repr(C)]
pub struct centity_t {
	/// Raven: from cg.frame
	pub currentState: entityState_t,
	/// Raven: ptr to playerstate if applicable (for bg ents)
	pub playerState: *mut playerState_t,
	/// Raven: vehicle data
	pub m_pVehicle: *mut Vehicle_t,
	/// Raven: g2 instance
	pub ghoul2: *mut c_void,
	/// Raven: index locally (game/cgame) to anim data for this skel
	pub localAnimIndex: i32,
	/// Raven: needed for g2 collision
	pub modelScale: vec3_t,

	// Raven: from here up must be unified with bgEntity_t -rww

	/// Raven: from cg.nextFrame, if available
	pub nextState: entityState_t,
	/// Raven: true if next is valid to interpolate to
	pub interpolate: qboolean,
	/// Raven: true if cg.frame holds this entity
	pub currentValid: qboolean,

	/// Raven: move to playerEntity?
	pub muzzleFlashTime: i32,
	pub previousEvent: i32,

	/// Raven: so missile trails can handle dropped initial packets
	pub trailTime: i32,
	pub dustTrailTime: i32,
	pub miscTime: i32,

	pub damageAngles: vec3_t,
	pub damageTime: i32,

	/// Raven: last time this entity was found in a snapshot
	pub snapShotTime: i32,

	pub pe: playerEntity_t,

	pub rawAngles: vec3_t,

	pub beamEnd: vec3_t,

	// Raven: exact interpolated position of entity on this frame
	pub lerpOrigin: vec3_t,
	pub lerpAngles: vec3_t,

	pub ragLastOrigin: vec3_t,
	pub ragLastOriginTime: i32,

	/// Raven: if true only do anims and things on model_root instead of
	/// lower_lumbar, this will be the case for some NPCs.
	pub noLumbar: qboolean,
	pub noFace: qboolean,

	// Raven: For keeping track of the current surface status in relation to
	// the entitystate surface fields.
	pub npcLocalSurfOn: i32,
	pub npcLocalSurfOff: i32,

	pub eventAnimIndex: i32,

	/// Raven: dynamically allocated - always free it, and never stomp over it.
	pub npcClient: *mut clientInfo_t,

	pub weapon: i32,

	/// Raven: rww - pointer to ghoul2 instance of the current 3rd person weapon
	pub ghoul2weapon: *mut c_void,

	pub radius: f32,
	pub boltInfo: i32,

	// Raven: sometimes used as a bolt index, but these values are also used
	// as generic values for clientside entities at times
	pub bolt1: i32,
	pub bolt2: i32,
	pub bolt3: i32,
	pub bolt4: i32,

	pub bodyHeight: f32,

	pub torsoBolt: i32,

	pub turAngles: vec3_t,

	pub frame_minus1: vec3_t,
	pub frame_minus2: vec3_t,

	pub frame_minus1_refreshed: i32,
	pub frame_minus2_refreshed: i32,

	/// Raven: pointer to a ghoul2 instance
	pub frame_hold: *mut c_void,

	pub frame_hold_time: i32,
	pub frame_hold_refreshed: i32,

	/// Raven: pointer to a ghoul2 instance
	pub grip_arm: *mut c_void,

	pub trickAlpha: i32,
	pub trickAlphaTime: i32,

	pub teamPowerEffectTime: i32,
	/// Raven: 0 regen, 1 heal, 2 drain, 3 absorb
	pub teamPowerType: qboolean,

	pub isRagging: qboolean,
	pub ownerRagging: qboolean,
	pub overridingBones: i32,

	pub bodyFadeTime: i32,
	pub pushEffectOrigin: vec3_t,

	pub loopingSound: [cgLoopSound_t; MAX_CG_LOOPSOUNDS],
	pub numLoopingSounds: i32,

	pub serverSaberHitIndex: i32,
	pub serverSaberHitTime: i32,
	/// Raven: true if flesh, false if anything else.
	pub serverSaberFleshImpact: qboolean,

	pub ikStatus: qboolean,

	pub saberWasInFlight: qboolean,

	pub smoothYaw: f32,

	pub uncloaking: i32,
	pub cloaked: qboolean,

	pub vChatTime: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<centity_t>() == 1984);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, currentState) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, playerState) == 536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, m_pVehicle) == 544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ghoul2) == 552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, localAnimIndex) == 560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, modelScale) == 564);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, nextState) == 576);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, interpolate) == 1108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, currentValid) == 1112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, muzzleFlashTime) == 1116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, previousEvent) == 1120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, trailTime) == 1124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, dustTrailTime) == 1128);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, miscTime) == 1132);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, damageAngles) == 1136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, damageTime) == 1148);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, snapShotTime) == 1152);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, pe) == 1160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, rawAngles) == 1424);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, beamEnd) == 1436);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, lerpOrigin) == 1448);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, lerpAngles) == 1460);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ragLastOrigin) == 1472);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ragLastOriginTime) == 1484);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, noLumbar) == 1488);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, noFace) == 1492);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, npcLocalSurfOn) == 1496);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, npcLocalSurfOff) == 1500);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, eventAnimIndex) == 1504);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, npcClient) == 1512);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, weapon) == 1520);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ghoul2weapon) == 1528);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, radius) == 1536);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, boltInfo) == 1540);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bolt1) == 1544);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bolt2) == 1548);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bolt3) == 1552);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bolt4) == 1556);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bodyHeight) == 1560);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, torsoBolt) == 1564);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, turAngles) == 1568);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_minus1) == 1580);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_minus2) == 1592);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_minus1_refreshed) == 1604);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_minus2_refreshed) == 1608);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_hold) == 1616);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_hold_time) == 1624);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, frame_hold_refreshed) == 1628);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, grip_arm) == 1632);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, trickAlpha) == 1640);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, trickAlphaTime) == 1644);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, teamPowerEffectTime) == 1648);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, teamPowerType) == 1652);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, isRagging) == 1656);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ownerRagging) == 1660);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, overridingBones) == 1664);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, bodyFadeTime) == 1668);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, pushEffectOrigin) == 1672);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, loopingSound) == 1684);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, numLoopingSounds) == 1940);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, serverSaberHitIndex) == 1944);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, serverSaberHitTime) == 1948);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, serverSaberFleshImpact) == 1952);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, ikStatus) == 1956);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, saberWasInFlight) == 1960);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, smoothYaw) == 1964);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, uncloaking) == 1968);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, cloaked) == 1972);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(centity_t, vChatTime) == 1976);
