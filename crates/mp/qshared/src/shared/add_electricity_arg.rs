#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use native_math::vector::vec3_t;
use native_types::qhandle_t;

/// Raven `addElectricityArgStruct_t` — `CG_ADDELECTRICITY` VM syscall args.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2622-2640`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct addElectricityArgStruct_t {
    pub start: vec3_t,
    pub end: vec3_t,
    pub size1: f32,
    pub size2: f32,
    pub sizeParm: f32,
    pub alpha1: f32,
    pub alpha2: f32,
    pub alphaParm: f32,
    pub sRGB: vec3_t,
    pub eRGB: vec3_t,
    pub rgbParm: f32,
    pub chaos: f32,
    pub killTime: core::ffi::c_int,
    pub shader: qhandle_t,
    pub flags: core::ffi::c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<addElectricityArgStruct_t>() == 92);
    assert!(offset_of!(addElectricityArgStruct_t, sRGB) == 48);
    assert!(offset_of!(addElectricityArgStruct_t, flags) == 88);
};
