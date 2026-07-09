#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use mp_qshared::common::mp::qcommon::game_item::gitem_t;

use super::centity_s::centity_t;
use mp_qshared::shared::{fxHandle_t, qboolean, qhandle_t, sfxHandle_t, vec3_t};

/// Raven `weaponInfo_t`.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:652-702`
#[repr(C)]
pub struct weaponInfo_t {
	pub registered: qboolean,
	pub item: *mut gitem_t,

	/// the hands don't actually draw, they just position the weapon
	pub handsModel: qhandle_t,
	/// this is the pickup model
	pub weaponModel: qhandle_t,
	/// this is the in-view model used by the player
	pub viewModel: qhandle_t,
	pub barrelModel: qhandle_t,
	pub flashModel: qhandle_t,

	/// so it will rotate centered instead of by tag
	pub weaponMidpoint: vec3_t,

	pub flashDlight: c_float,
	pub flashDlightColor: vec3_t,

	pub weaponIcon: qhandle_t,
	pub ammoIcon: qhandle_t,

	pub ammoModel: qhandle_t,

	/// fast firing weapons randomly choose
	pub flashSound: [sfxHandle_t; 4],
	pub firingSound: sfxHandle_t,
	pub chargeSound: sfxHandle_t,
	pub muzzleEffect: fxHandle_t,
	pub missileModel: qhandle_t,
	pub missileSound: sfxHandle_t,
	pub missileTrailFunc: Option<unsafe extern "C" fn(cent: *mut centity_t, wi: *const weaponInfo_t)>,
	pub missileDlight: c_float,
	pub missileDlightColor: vec3_t,
	pub missileRenderfx: c_int,
	pub missileHitSound: sfxHandle_t,

	pub altFlashSound: [sfxHandle_t; 4],
	pub altFiringSound: sfxHandle_t,
	pub altChargeSound: sfxHandle_t,
	pub altMuzzleEffect: fxHandle_t,
	pub altMissileModel: qhandle_t,
	pub altMissileSound: sfxHandle_t,
	pub altMissileTrailFunc: Option<unsafe extern "C" fn(cent: *mut centity_t, wi: *const weaponInfo_t)>,
	pub altMissileDlight: c_float,
	pub altMissileDlightColor: vec3_t,
	pub altMissileRenderfx: c_int,
	pub altMissileHitSound: sfxHandle_t,

	pub selectSound: sfxHandle_t,

	pub readySound: sfxHandle_t,
	pub trailRadius: c_float,
	pub wiTrailTime: c_float,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<weaponInfo_t>() == 232);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, registered) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, item) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, handsModel) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponModel) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, viewModel) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, barrelModel) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, flashModel) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponMidpoint) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, flashDlight) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, flashDlightColor) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponIcon) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, ammoIcon) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, ammoModel) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, flashSound) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, firingSound) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, chargeSound) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, muzzleEffect) == 100);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileModel) == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileSound) == 108);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileTrailFunc) == 112);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileDlight) == 120);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileDlightColor) == 124);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileRenderfx) == 136);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileHitSound) == 140);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altFlashSound) == 144);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altFiringSound) == 160);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altChargeSound) == 164);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMuzzleEffect) == 168);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileModel) == 172);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileSound) == 176);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileTrailFunc) == 184);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileDlight) == 192);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileDlightColor) == 196);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileRenderfx) == 208);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altMissileHitSound) == 212);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, selectSound) == 216);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, readySound) == 220);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, trailRadius) == 224);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, wiTrailTime) == 228);
