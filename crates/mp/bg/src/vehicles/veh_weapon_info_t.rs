#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use core::ffi::c_char;
use mp_qshared::shared::qboolean;

/// Raven `vehWeaponInfo_t` — static data describing one vehicle weapon.
///
/// Raven: *** IMPORTANT!!! *** the number of variables in the vehWeaponStats_t
/// struct (including all elements of arrays) must be reflected by
/// NUM_VWEAP_PARMS!!! *** IMPORTANT!!! *** vWeapFields table correponds to
/// this structure!
/// Type definition source: `oracle/codemp/game/bg_vehicles.h:35-64`
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct vehWeaponInfo_t {
	pub name: *mut c_char,
	/// traceline or entity?
	pub bIsProjectile: qboolean,
	/// if a projectile, drops
	pub bHasGravity: qboolean,
	/// disables ship shields and sends them out of control
	pub bIonWeapon: qboolean,
	/// lightsabers can deflect this projectile
	pub bSaberBlockable: qboolean,
	/// index of Muzzle Effect
	pub iMuzzleFX: i32,
	/// handle to the model used by this projectile
	pub iModel: i32,
	/// index of Shot Effect
	pub iShotFX: i32,
	/// index of Impact Effect
	pub iImpactFX: i32,
	/// index of shader to use for G2 marks made on other models when hit by this projectile
	pub iG2MarkShaderHandle: i32,
	/// size (diameter) of the ghoul2 mark
	pub fG2MarkSize: f32,
	/// index of loopSound
	pub iLoopSound: i32,
	/// speed of projectile/range of traceline
	pub fSpeed: f32,
	/// 0.0 = not homing, 0.5 = half vel to targ, half cur vel, 1.0 = all vel to targ
	pub fHoming: f32,
	/// missile will lose lock on if DotProduct of missile direction and direction to target ever
	/// drops below this (-1 to 1, -1 = never lose target, 0 = lose if ship gets behind missile,
	/// 1 = pretty much will lose it's target right away)
	pub fHomingFOV: f32,
	/// 0 = no lock time needed, else # of ms needed to lock on
	pub iLockOnTime: i32,
	/// damage done when traceline or projectile directly hits target
	pub iDamage: i32,
	/// damage done to ents in splashRadius of end of traceline or projectile origin on impact
	pub iSplashDamage: i32,
	/// radius that ent must be in to take splashDamage (linear fall-off)
	pub fSplashRadius: f32,
	/// how much "ammo" each shot takes
	pub iAmmoPerShot: i32,
	/// if non-zero, projectile can be shot, takes this much damage before being destroyed
	pub iHealth: i32,
	/// width of traceline or bounding box of projecile (non-rotating!)
	pub fWidth: f32,
	/// height of traceline or bounding box of projecile (non-rotating!)
	pub fHeight: f32,
	/// removes itself after this amount of time
	pub iLifeTime: i32,
	/// when iLifeTime is up, explodes rather than simply removing itself
	pub bExplodeOnExpire: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<vehWeaponInfo_t>() == 104);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bIsProjectile) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bHasGravity) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bIonWeapon) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bSaberBlockable) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iMuzzleFX) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iModel) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iShotFX) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iImpactFX) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iG2MarkShaderHandle) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fG2MarkSize) == 44);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iLoopSound) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fSpeed) == 52);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fHoming) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fHomingFOV) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iLockOnTime) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iDamage) == 68);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iSplashDamage) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fSplashRadius) == 76);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iAmmoPerShot) == 80);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iHealth) == 84);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fWidth) == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, fHeight) == 92);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, iLifeTime) == 96);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(vehWeaponInfo_t, bExplodeOnExpire) == 100);
