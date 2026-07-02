#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::qboolean;

use super::lerp_frame_t::lerpFrame_t;

/// Raven `playerEntity_t` — extra data for a player entity's cgame model.
///
/// Type definition source: `oracle/oracle/code/cgame/cg_local.h:112-124`
#[repr(C)]
pub struct playerEntity_t {
	pub legs: lerpFrame_t,
	pub torso: lerpFrame_t,
	pub painTime: i32,
	/// flip from 0 to 1
	pub painDirection: i32,

	/// For persistent beam weapons, so they don't play their start sound more than once
	pub lightningFiring: qboolean,

	// machinegun spinning
	//	float			barrelAngle;
	//	int				barrelTime;
	//	qboolean		barrelSpinning;
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<playerEntity_t>() == 128);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, legs) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(playerEntity_t, torso) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(playerEntity_t, painTime) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(playerEntity_t, painDirection) == 116);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(playerEntity_t, lightningFiring) == 120);
