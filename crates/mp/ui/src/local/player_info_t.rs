#![allow(non_camel_case_types, non_snake_case)]

use mp_bg::public::animation::animation_t;
use mp_bg::weapons::weapon_t::weapon_t;
use mp_qshared::shared::{qboolean, qhandle_t, vec3_t};

use super::lerp_frame_t::lerpFrame_t;

/// Raven `playerInfo_t`.
///
/// Raven: model info, currently in use drawing parms, animation vars.
///
/// `animations` is sized `MAX_TOTALANIMATIONS` (`oracle/codemp/game/anims.h:1789-1790`);
/// that enum is unported, so the array length (1544) is taken from the assert-verified
/// layout below.
/// Type definition source: `oracle/codemp/ui/ui_local.h:480-527`
#[repr(C)]
pub struct playerInfo_t {
	// model info
	pub legsModel: qhandle_t,
	pub legsSkin: qhandle_t,
	pub legs: lerpFrame_t,

	pub torsoModel: qhandle_t,
	pub torsoSkin: qhandle_t,
	pub torso: lerpFrame_t,

	//	qhandle_t		headModel;
	//	qhandle_t		headSkin;
	pub animations: [animation_t; 1544],

	pub weaponModel: qhandle_t,
	pub barrelModel: qhandle_t,
	pub flashModel: qhandle_t,
	pub flashDlightColor: vec3_t,
	pub muzzleFlashTime: i32,

	// currently in use drawing parms
	pub viewAngles: vec3_t,
	pub moveAngles: vec3_t,
	pub currentWeapon: weapon_t,
	pub legsAnim: i32,
	pub torsoAnim: i32,

	// animation vars
	pub weapon: weapon_t,
	pub lastWeapon: weapon_t,
	pub pendingWeapon: weapon_t,
	pub weaponTimer: i32,
	pub pendingLegsAnim: i32,
	pub torsoAnimationTimer: i32,

	pub pendingTorsoAnim: i32,
	pub legsAnimationTimer: i32,

	pub chat: qboolean,
	pub newModel: qboolean,

	pub barrelSpinning: qboolean,
	pub barrelAngle: f32,
	pub barrelTime: i32,

	pub realWeapon: i32,
}

const _: () = assert!(core::mem::size_of::<playerInfo_t>() == 11056);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, legsModel) == 0);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, legsSkin) == 4);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, legs) == 8);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, torsoModel) == 64);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, torsoSkin) == 68);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, torso) == 72);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, animations) == 128);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, weaponModel) == 10936);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, barrelModel) == 10940);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, flashModel) == 10944);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, flashDlightColor) == 10948);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, muzzleFlashTime) == 10960);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, viewAngles) == 10964);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, moveAngles) == 10976);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, currentWeapon) == 10988);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, legsAnim) == 10992);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, torsoAnim) == 10996);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, weapon) == 11000);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, lastWeapon) == 11004);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, pendingWeapon) == 11008);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, weaponTimer) == 11012);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, pendingLegsAnim) == 11016);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, torsoAnimationTimer) == 11020);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, pendingTorsoAnim) == 11024);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, legsAnimationTimer) == 11028);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, chat) == 11032);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, newModel) == 11036);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, barrelSpinning) == 11040);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, barrelAngle) == 11044);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, barrelTime) == 11048);
const _: () = assert!(core::mem::offset_of!(playerInfo_t, realWeapon) == 11052);
