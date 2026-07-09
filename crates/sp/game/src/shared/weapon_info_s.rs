#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

use sp_qshared::common::sp::ff::ff_handle_t::ffHandle_t;
use sp_qshared::common::sp::qcommon::game_item::gitem_t;
use sp_qshared::shared::{qboolean, qhandle_t, sfxHandle_t, vec3_t};

/// Raven `weaponInfo_t`.
///
/// Type definition source: `oracle/code/game/g_shared.h:836-885`
#[repr(C)]
pub struct weaponInfo_t {
	pub registered: qboolean,
	pub item: *mut gitem_t,

	/// the hands don't actually draw, they just position the weapon
	pub handsModel: qhandle_t,
	/// for in view
	pub weaponModel: qhandle_t,
	/// for in their hands
	pub weaponWorldModel: qhandle_t,
	pub barrelModel: [qhandle_t; 4],

	/// so it will rotate centered instead of by tag
	pub weaponMidpoint: vec3_t,

	/// The version of the icon with a glowy background
	pub weaponIcon: qhandle_t,
	/// The version of the icon with no ammo warning
	pub weaponIconNoAmmo: qhandle_t,
	pub ammoIcon: qhandle_t,

	pub ammoModel: qhandle_t,

	pub missileModel: qhandle_t,
	pub missileSound: sfxHandle_t,
	// `centity_t` is defined in SP cgame (`sp_cgame::local::centity_s`), which
	// sits above this crate in the graph — the callback param stays an opaque
	// pointer here (ABI-identical).
	//TODO: Port centity_t
	// Source: oracle/code/cgame/cg_local.h:130-174
	pub missileTrailFunc: Option<unsafe extern "C" fn(cent: *mut c_void, wi: *const weaponInfo_t)>,

	pub alt_missileModel: qhandle_t,
	pub alt_missileSound: sfxHandle_t,
	//TODO: Port centity_t
	// Source: oracle/code/cgame/cg_local.h:130-176
	pub alt_missileTrailFunc: Option<unsafe extern "C" fn(cent: *mut c_void, wi: *const weaponInfo_t)>,

	pub firingSound: sfxHandle_t,
	pub altFiringSound: sfxHandle_t,

	pub stopSound: sfxHandle_t,

	pub missileHitSound: sfxHandle_t,
	pub altmissileHitSound: sfxHandle_t,

	pub chargeSound: sfxHandle_t,
	pub altChargeSound: sfxHandle_t,

	/// sound played when weapon is selected
	pub selectSound: sfxHandle_t,

	// Raven: `#ifdef _IMMERSION` — force-feedback handles; only present under
	// Raven's `_IMMERSION` build, which this SP layout has enabled (per the
	// packet's verbatim offsets).
	pub firingForce: ffHandle_t,
	pub altFiringForce: ffHandle_t,
	pub stopForce: ffHandle_t,
	pub chargeForce: ffHandle_t,
	pub altChargeForce: ffHandle_t,
	pub selectForce: ffHandle_t,
}

const _: () = assert!(core::mem::size_of::<weaponInfo_t>() == 160);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, registered) == 0);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, item) == 8);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, handsModel) == 16);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponModel) == 20);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponWorldModel) == 24);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, barrelModel) == 28);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponMidpoint) == 44);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponIcon) == 56);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, weaponIconNoAmmo) == 60);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, ammoIcon) == 64);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, ammoModel) == 68);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileModel) == 72);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileSound) == 76);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileTrailFunc) == 80);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, alt_missileModel) == 88);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, alt_missileSound) == 92);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, alt_missileTrailFunc) == 96);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, firingSound) == 104);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altFiringSound) == 108);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, stopSound) == 112);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, missileHitSound) == 116);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altmissileHitSound) == 120);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, chargeSound) == 124);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altChargeSound) == 128);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, selectSound) == 132);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, firingForce) == 136);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altFiringForce) == 140);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, stopForce) == 144);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, chargeForce) == 148);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, altChargeForce) == 152);
const _: () = assert!(core::mem::offset_of!(weaponInfo_t, selectForce) == 156);
