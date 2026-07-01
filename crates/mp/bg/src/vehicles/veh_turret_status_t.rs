#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Raven `vehTurretStatus_t` — per-turret runtime firing/targeting state.
///
/// Type definition source: `oracle/oracle/codemp/game/bg_vehicles.h:462-474`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct vehTurretStatus_t {
	//current weapon ammo
	pub ammo: i32,
	//debouncer for ammo recharge
	pub lastAmmoInc: i32,
	//which muzzle will fire next
	pub nextMuzzle: i32,
	//which entity they're after
	pub enemyEntNum: i32,
	//how long to hold on to our current enemy
	pub enemyHoldTime: i32,
}

const _: () = assert!(core::mem::size_of::<vehTurretStatus_t>() == 20);
const _: () = assert!(core::mem::offset_of!(vehTurretStatus_t, ammo) == 0);
const _: () = assert!(core::mem::offset_of!(vehTurretStatus_t, lastAmmoInc) == 4);
const _: () = assert!(core::mem::offset_of!(vehTurretStatus_t, nextMuzzle) == 8);
const _: () = assert!(core::mem::offset_of!(vehTurretStatus_t, enemyEntNum) == 12);
const _: () = assert!(core::mem::offset_of!(vehTurretStatus_t, enemyHoldTime) == 16);
