#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::vec3_t;

/// Raven `aas_entityinfo_t` — AAS-visible entity state snapshot.
///
/// Type definition source: `oracle/codemp/game/be_aas.h:107-132`
#[repr(C)]
pub struct aas_entityinfo_t {
	/// true if updated this frame
	pub valid: i32,
	/// entity type
	pub r#type: i32,
	/// entity flags
	pub flags: i32,
	/// local time
	pub ltime: f32,
	/// time between last and current update
	pub update_time: f32,
	/// number of the entity
	pub number: i32,
	/// origin of the entity
	pub origin: vec3_t,
	/// angles of the model
	pub angles: vec3_t,
	/// for lerping
	pub old_origin: vec3_t,
	/// last visible origin
	pub lastvisorigin: vec3_t,
	/// bounding box minimums
	pub mins: vec3_t,
	/// bounding box maximums
	pub maxs: vec3_t,
	/// ground entity
	pub groundent: i32,
	/// solid type
	pub solid: i32,
	/// model used
	pub modelindex: i32,
	/// weapons, CTF flags, etc
	pub modelindex2: i32,
	/// model frame number
	pub frame: i32,
	/// impulse events -- muzzle flashes, footsteps, etc
	pub event: i32,
	/// even parameter
	pub eventParm: i32,
	/// bit flags
	pub powerups: i32,
	/// determines weapon and flash model, etc
	pub weapon: i32,
	/// current legs anim
	pub legsAnim: i32,
	/// current torso anim
	pub torsoAnim: i32,
}

pub type aas_entityinfo_s = aas_entityinfo_t;

const _: () = assert!(core::mem::size_of::<aas_entityinfo_t>() == 140);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, valid) == 0);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, r#type) == 4);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, flags) == 8);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, ltime) == 12);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, update_time) == 16);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, number) == 20);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, origin) == 24);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, angles) == 36);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, old_origin) == 48);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, lastvisorigin) == 60);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, mins) == 72);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, maxs) == 84);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, groundent) == 96);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, solid) == 100);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, modelindex) == 104);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, modelindex2) == 108);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, frame) == 112);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, event) == 116);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, eventParm) == 120);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, powerups) == 124);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, weapon) == 128);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, legsAnim) == 132);
const _: () = assert!(core::mem::offset_of!(aas_entityinfo_t, torsoAnim) == 136);
