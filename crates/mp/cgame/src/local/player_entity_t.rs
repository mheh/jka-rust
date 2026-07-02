#![allow(non_camel_case_types, non_snake_case)]

use super::lerp_frame_t::lerpFrame_t;
use mp_qshared::shared::qboolean;

/// Raven `playerEntity_t` — per-player-entity animation/render state.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:168-178`
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

const _: () = assert!(core::mem::size_of::<playerEntity_t>() == 264);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, legs) == 0);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, torso) == 80);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, flag) == 160);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, painTime) == 240);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, painDirection) == 244);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, lightningFiring) == 248);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, barrelAngle) == 252);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, barrelTime) == 256);
const _: () = assert!(core::mem::offset_of!(playerEntity_t, barrelSpinning) == 260);
