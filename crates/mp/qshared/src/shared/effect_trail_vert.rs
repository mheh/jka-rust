#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use native_math::vector::vec3_t;

/// Raven `effectTrailVertStruct_t` — one vertex of an effect trail segment.
///
/// Raven: color/alpha and ST coords carry current + destination values so a
/// segment can interpolate as it progresses through its life.
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2595-2614`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct effectTrailVertStruct_t {
    pub origin: vec3_t,
    pub rgb: vec3_t,
    pub destrgb: vec3_t,
    pub curRGB: vec3_t,
    pub alpha: f32,
    pub destAlpha: f32,
    pub curAlpha: f32,
    pub ST: [f32; 2],
    pub destST: [f32; 2],
    pub curST: [f32; 2],
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<effectTrailVertStruct_t>() == 84);
    assert!(offset_of!(effectTrailVertStruct_t, alpha) == 48);
    assert!(offset_of!(effectTrailVertStruct_t, ST) == 60);
    assert!(offset_of!(effectTrailVertStruct_t, curST) == 76);
};
