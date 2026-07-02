#![allow(non_camel_case_types, non_snake_case)]

use mp_bg::public::animation::animation_t;
use mp_qshared::shared::qboolean;

/// Raven `lerpFrame_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:461-478`
#[repr(C)]
pub struct lerpFrame_t {
	pub oldFrame: i32,
	/// time when ->oldFrame was exactly on
	pub oldFrameTime: i32,

	pub frame: i32,
	/// time when ->frame will be exactly on
	pub frameTime: i32,

	pub backlerp: f32,

	pub yawAngle: f32,
	pub yawing: qboolean,
	pub pitchAngle: f32,
	pub pitching: qboolean,

	pub animationNumber: i32,
	pub animation: *mut animation_t,
	/// time when the first frame of the animation will be exact
	pub animationTime: i32,
}

const _: () = assert!(core::mem::size_of::<lerpFrame_t>() == 56);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, oldFrame) == 0);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, oldFrameTime) == 4);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, frame) == 8);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, frameTime) == 12);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, backlerp) == 16);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, yawAngle) == 20);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, yawing) == 24);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, pitchAngle) == 28);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, pitching) == 32);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, animationNumber) == 36);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, animation) == 40);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, animationTime) == 48);
