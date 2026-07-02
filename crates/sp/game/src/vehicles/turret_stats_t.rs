#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_char;
use sp_qshared::shared::qboolean;

/// Raven `MAX_VEHICLE_TURRET_MUZZLES`.
///
/// Type definition source: `oracle/oracle/code/game/g_vehicles.h:88`
pub const MAX_VEHICLE_TURRET_MUZZLES: usize = 2;

/// Raven `turretStats_t` — static data describing a vehicle-mounted turret.
///
/// Type definition source: `oracle/oracle/code/game/G_Vehicles.h:90-111`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct turretStats_t {
	/// what vehWeaponInfo index to use
	pub iWeapon: i32,
	/// delay between turret muzzle shots
	pub iDelay: i32,
	/// how much ammo it has
	pub iAmmoMax: i32,
	/// how many MS between every point of recharged ammo
	pub iAmmoRechargeMS: i32,
	/// bone on ship that this turret uses to yaw
	pub yawBone: *mut c_char,
	/// bone on ship that this turret uses to pitch
	pub pitchBone: *mut c_char,
	/// axis on yawBone to which we should to apply the yaw angles
	pub yawAxis: i32,
	/// axis on pitchBone to which we should to apply the pitch angles
	pub pitchAxis: i32,
	/// how far the turret is allowed to turn left
	pub yawClampLeft: f32,
	/// how far the turret is allowed to turn right
	pub yawClampRight: f32,
	/// how far the turret is allowed to title up
	pub pitchClampUp: f32,
	/// how far the turret is allowed to tilt down
	pub pitchClampDown: f32,
	/// iMuzzle-1 = index of ship's muzzle to fire this turret's 1st and 2nd shots from
	pub iMuzzle: [i32; MAX_VEHICLE_TURRET_MUZZLES],
	/// Where to put the view origin of the gunner (name)
	pub gunnerViewTag: *mut c_char,
	/// how quickly the turret can turn
	pub fTurnSpeed: f32,
	/// whether or not the turret auto-targets enemies when it's not manned
	pub bAI: qboolean,
	/// whether
	pub bAILead: qboolean,
	/// how far away the AI will look for enemies
	pub fAIRange: f32,
	/// which passenger, if any, has control of this turret (overrides AI)
	pub passengerNum: i32,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<turretStats_t>() == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iWeapon) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iDelay) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iAmmoMax) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iAmmoRechargeMS) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, yawBone) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, pitchBone) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, yawAxis) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, pitchAxis) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, yawClampLeft) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, yawClampRight) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, pitchClampUp) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, pitchClampDown) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, iMuzzle) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, gunnerViewTag) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, fTurnSpeed) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, bAI) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, bAILead) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, fAIRange) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(turretStats_t, passengerNum) == 88);
