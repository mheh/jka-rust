#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use native_math::vector::{vec2_t, vec3_t};
use native_types::qhandle_t;

/// Raven `addpolyArgStruct_t` (`addpolyArgStruct_s`) — `CG_ADDPOLY` VM syscall args.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:2538-2556`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct addpolyArgStruct_t {
    pub p: [vec3_t; 4],
    pub ev: [vec2_t; 4],
    pub numVerts: core::ffi::c_int,
    pub vel: vec3_t,
    pub accel: vec3_t,
    pub alpha1: f32,
    pub alpha2: f32,
    pub alphaParm: f32,
    pub rgb1: vec3_t,
    pub rgb2: vec3_t,
    pub rgbParm: f32,
    pub rotationDelta: vec3_t,
    pub bounce: f32,
    pub motionDelay: core::ffi::c_int,
    pub killTime: core::ffi::c_int,
    pub shader: qhandle_t,
    pub flags: core::ffi::c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<addpolyArgStruct_t>() == 180);
    assert!(offset_of!(addpolyArgStruct_t, ev) == 48);
    assert!(offset_of!(addpolyArgStruct_t, numVerts) == 80);
    assert!(offset_of!(addpolyArgStruct_t, flags) == 176);
};
