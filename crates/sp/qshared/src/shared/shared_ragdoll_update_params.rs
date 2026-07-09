#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;
use native_math::vector::vec3_t;

/// Raven `sharedRagDollUpdateParams_t` — per-frame ragdoll update parameters.
///
/// Type definition source: `oracle/code/game/q_shared.h:2571-2578`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct sharedRagDollUpdateParams_t {
    pub angles: vec3_t,
    pub position: vec3_t,
    pub scale: vec3_t,
    pub velocity: vec3_t,
    pub me: c_int,
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<sharedRagDollUpdateParams_t>() == 52);
    assert!(offset_of!(sharedRagDollUpdateParams_t, velocity) == 36);
    assert!(offset_of!(sharedRagDollUpdateParams_t, me) == 48);
};
