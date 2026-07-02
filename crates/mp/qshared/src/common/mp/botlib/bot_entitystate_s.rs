#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::shared::vec3_t;

/// Raven `bot_entitystate_t` — a snapshot of an entity's state as seen by the bot AI.
///
/// Type definition source: `oracle/oracle/codemp/game/botlib.h:134-154`
#[repr(C)]
pub struct bot_entitystate_t {
	/// entity type
	pub r#type: c_int,
	/// entity flags
	pub flags: c_int,
	/// origin of the entity
	pub origin: vec3_t,
	/// angles of the model
	pub angles: vec3_t,
	/// for lerping
	pub old_origin: vec3_t,
	/// bounding box minimums
	pub mins: vec3_t,
	/// bounding box maximums
	pub maxs: vec3_t,
	/// ground entity
	pub groundent: c_int,
	/// solid type
	pub solid: c_int,
	/// model used
	pub modelindex: c_int,
	/// weapons, CTF flags, etc
	pub modelindex2: c_int,
	/// model frame number
	pub frame: c_int,
	/// impulse events -- muzzle flashes, footsteps, etc
	pub event: c_int,
	/// even parameter
	pub eventParm: c_int,
	/// bit flags
	pub powerups: c_int,
	/// determines weapon and flash model, etc
	pub weapon: c_int,
	pub legsAnim: c_int,
	pub torsoAnim: c_int,
}

pub type bot_entitystate_s = bot_entitystate_t;

const _: () = assert!(core::mem::size_of::<bot_entitystate_t>() == 112);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, r#type) == 0);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, flags) == 4);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, origin) == 8);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, angles) == 20);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, old_origin) == 32);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, mins) == 44);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, maxs) == 56);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, groundent) == 68);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, solid) == 72);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, modelindex) == 76);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, modelindex2) == 80);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, frame) == 84);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, event) == 88);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, eventParm) == 92);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, powerups) == 96);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, weapon) == 100);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, legsAnim) == 104);
const _: () = assert!(core::mem::offset_of!(bot_entitystate_t, torsoAnim) == 108);
