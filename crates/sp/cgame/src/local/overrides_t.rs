#![allow(non_camel_case_types, non_snake_case)]

/// Raven `overrides_t` — third-person camera / FOV overrides.
///
/// Raven: NOTE: these probably get cleared in save/load!!!
/// NOTE: could put Alpha and HorzOffset and the target & camera damps, but
/// no-one is trying to override those, so...
/// Type definition source: `oracle/code/cgame/cg_local.h:277-289`
#[repr(C)]
pub struct overrides_t {
	/// bit-flag field of which overrides are active
	pub active: i32,
	/// who to center on
	pub thirdPersonEntity: i32,
	/// how far to be from them
	pub thirdPersonRange: f32,
	/// what angle to look at them from
	pub thirdPersonAngle: f32,
	/// how high to be above them
	pub thirdPersonVertOffset: f32,
	/// what offset pitch to apply the the camera view
	pub thirdPersonPitchOffset: f32,
	/// how tightly to move the camera pos behind the player
	pub thirdPersonCameraDamp: f32,
	/// how tightly to move the camera pos behind the player
	pub thirdPersonAlpha: f32,
	/// what fov to use
	pub fov: f32,
}

const _: () = assert!(core::mem::size_of::<overrides_t>() == 36);
const _: () = assert!(core::mem::offset_of!(overrides_t, active) == 0);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonEntity) == 4);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonRange) == 8);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonAngle) == 12);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonVertOffset) == 16);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonPitchOffset) == 20);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonCameraDamp) == 24);
const _: () = assert!(core::mem::offset_of!(overrides_t, thirdPersonAlpha) == 28);
const _: () = assert!(core::mem::offset_of!(overrides_t, fov) == 32);
