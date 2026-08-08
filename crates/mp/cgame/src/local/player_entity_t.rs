#![allow(non_camel_case_types, non_snake_case)]

use super::lerp_frame_t::lerpFrame_t;
use mp_qshared::shared::qboolean;

/// Raven `playerEntity_t` — per-player-entity animation/render state.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:168-178`
#[repr(C)]
pub struct playerEntity_t {
    pub legs: lerpFrame_t,
    pub torso: lerpFrame_t,
    pub flag: lerpFrame_t,
    pub painTime: i32,
    pub painDirection: i32, // flip from 0 to 1
    pub lightningFiring: i32,

    // machinegun spinning
    pub barrelAngle: f32,
    pub barrelTime: i32,
    pub barrelSpinning: qboolean,
}

const _: () = assert!(core::mem::offset_of!(playerEntity_t, legs) == 0);
// The three `lerpFrame_t` members carry a pointer each, so every offset after `legs` moves with the pointer width.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<playerEntity_t>() == 264);
    assert!(core::mem::offset_of!(playerEntity_t, torso) == 80);
    assert!(core::mem::offset_of!(playerEntity_t, flag) == 160);
    assert!(core::mem::offset_of!(playerEntity_t, painTime) == 240);
    assert!(core::mem::offset_of!(playerEntity_t, painDirection) == 244);
    assert!(core::mem::offset_of!(playerEntity_t, lightningFiring) == 248);
    assert!(core::mem::offset_of!(playerEntity_t, barrelAngle) == 252);
    assert!(core::mem::offset_of!(playerEntity_t, barrelTime) == 256);
    assert!(core::mem::offset_of!(playerEntity_t, barrelSpinning) == 260);
};
// ILP32 twin: clang i386 ground truth, where msvc and linux-gnu agree.
// These numbers are the retail 32-bit module ABI.
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<playerEntity_t>() == 240);
    assert!(core::mem::offset_of!(playerEntity_t, torso) == 72);
    assert!(core::mem::offset_of!(playerEntity_t, flag) == 144);
    assert!(core::mem::offset_of!(playerEntity_t, painTime) == 216);
    assert!(core::mem::offset_of!(playerEntity_t, painDirection) == 220);
    assert!(core::mem::offset_of!(playerEntity_t, lightningFiring) == 224);
    assert!(core::mem::offset_of!(playerEntity_t, barrelAngle) == 228);
    assert!(core::mem::offset_of!(playerEntity_t, barrelTime) == 232);
    assert!(core::mem::offset_of!(playerEntity_t, barrelSpinning) == 236);
};
