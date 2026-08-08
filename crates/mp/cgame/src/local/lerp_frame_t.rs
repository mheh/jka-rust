#![allow(non_camel_case_types, non_snake_case)]

use mp_bg::public::animation::animation_t;
use mp_qshared::shared::qboolean;

/// Raven `lerpFrame_t` — animation interpolation state for a single body part.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:137-165`
#[repr(C)]
pub struct lerpFrame_t {
    pub oldFrame: i32,
    pub oldFrameTime: i32, // time when ->oldFrame was exactly on

    pub frame: i32,
    pub frameTime: i32, // time when ->frame will be exactly on

    pub backlerp: f32,

    pub lastFlip: qboolean, //if does not match torsoFlip/legsFlip, restart the anim.

    pub lastForcedFrame: i32,

    pub yawAngle: f32,
    pub yawing: qboolean,
    pub pitchAngle: f32,
    pub pitching: qboolean,

    pub yawSwingDif: f32,

    pub animationNumber: i32,
    pub animation: *mut animation_t,
    pub animationTime: i32, // time when the first frame of the animation will be exact

    pub animationSpeed: f32, // scale the animation speed
    pub animationTorsoSpeed: f32,

    pub torsoYawing: qboolean,
}

// The head of the struct holds only 4-byte members, so these offsets hold on both pointer widths.
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, oldFrame) == 0);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, oldFrameTime) == 4);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, frame) == 8);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, frameTime) == 12);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, backlerp) == 16);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, lastFlip) == 20);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, lastForcedFrame) == 24);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, yawAngle) == 28);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, yawing) == 32);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, pitchAngle) == 36);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, pitching) == 40);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, yawSwingDif) == 44);
const _: () = assert!(core::mem::offset_of!(lerpFrame_t, animationNumber) == 48);
// `animation` is a pointer, so the size and every offset from `animation` onward change with the pointer width.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<lerpFrame_t>() == 80);
    assert!(core::mem::offset_of!(lerpFrame_t, animation) == 56);
    assert!(core::mem::offset_of!(lerpFrame_t, animationTime) == 64);
    assert!(core::mem::offset_of!(lerpFrame_t, animationSpeed) == 68);
    assert!(core::mem::offset_of!(lerpFrame_t, animationTorsoSpeed) == 72);
    assert!(core::mem::offset_of!(lerpFrame_t, torsoYawing) == 76);
};
// ILP32 twin: clang i386 ground truth, where msvc and linux-gnu agree.
// These numbers are the retail 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<lerpFrame_t>() == 72);
    assert!(core::mem::offset_of!(lerpFrame_t, animation) == 52);
    assert!(core::mem::offset_of!(lerpFrame_t, animationTime) == 56);
    assert!(core::mem::offset_of!(lerpFrame_t, animationSpeed) == 60);
    assert!(core::mem::offset_of!(lerpFrame_t, animationTorsoSpeed) == 64);
    assert!(core::mem::offset_of!(lerpFrame_t, torsoYawing) == 68);
};
