#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use native_math::vector::vec3_t;
use native_types::qhandle_t;

/// Raven `addspriteArgStruct_t` (`addspriteArgStruct_s`) — `CG_ADDSPRITE` VM args.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2579-2593`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct addspriteArgStruct_t {
    pub origin: vec3_t,
    pub vel: vec3_t,
    pub accel: vec3_t,
    pub scale: f32,
    pub dscale: f32,
    pub sAlpha: f32,
    pub eAlpha: f32,
    pub rotation: f32,
    pub bounce: f32,
    pub life: core::ffi::c_int,
    pub shader: qhandle_t,
    pub flags: core::ffi::c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<addspriteArgStruct_t>() == 72);
    assert!(offset_of!(addspriteArgStruct_t, scale) == 36);
    assert!(offset_of!(addspriteArgStruct_t, flags) == 68);
};
