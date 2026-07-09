#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use mp_qshared::shared::qboolean;

/// Raven `vehWeaponStats_t` — per-weapon-slot static vehicle weapon data.
///
/// Type definition source: `oracle/codemp/game/bg_vehicles.h:112-129`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct vehWeaponStats_t {
	//*** IMPORTANT!!! *** See note at top of next structure!!! ***
	// Weapon stuff.
	pub ID: i32, //index into the weapon data
	// The delay between shots for each weapon.
	pub delay: i32,
	// Whether or not all the muzzles for each weapon can be linked together (linked delay = weapon delay * number of muzzles linked!)
	pub linkable: i32,
	// Whether or not to auto-aim the projectiles/tracelines at the thing under the crosshair when we fire
	pub aimCorrect: qboolean,
	//maximum ammo
	pub ammoMax: i32,
	//ammo recharge rate - milliseconds per unit (minimum of 100, which is 10 ammo per second)
	pub ammoRechargeMS: i32,
	//sound to play when out of ammo (plays default "no ammo" sound if none specified)
	pub soundNoAmmo: i32,
}

const _: () = assert!(core::mem::size_of::<vehWeaponStats_t>() == 28);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, ID) == 0);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, delay) == 4);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, linkable) == 8);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, aimCorrect) == 12);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, ammoMax) == 16);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, ammoRechargeMS) == 20);
const _: () = assert!(core::mem::offset_of!(vehWeaponStats_t, soundNoAmmo) == 24);
