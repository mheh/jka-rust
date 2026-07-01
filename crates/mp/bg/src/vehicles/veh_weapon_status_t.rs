#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use mp_qshared::shared::qboolean;

/// Raven `vehWeaponStatus_t` — per-weapon-slot runtime vehicle weapon state.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:450-460`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct vehWeaponStatus_t {
	//linked firing mode
	pub linked: qboolean, //weapon 1's muzzles are in linked firing mode
	//current weapon ammo
	pub ammo: i32,
	//debouncer for ammo recharge
	pub lastAmmoInc: i32,
	//which muzzle will fire next
	pub nextMuzzle: i32,
}

const _: () = assert!(core::mem::size_of::<vehWeaponStatus_t>() == 16);
const _: () = assert!(core::mem::offset_of!(vehWeaponStatus_t, linked) == 0);
const _: () = assert!(core::mem::offset_of!(vehWeaponStatus_t, ammo) == 4);
const _: () = assert!(core::mem::offset_of!(vehWeaponStatus_t, lastAmmoInc) == 8);
const _: () = assert!(core::mem::offset_of!(vehWeaponStatus_t, nextMuzzle) == 12);
