#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::shared::wpneighbor::wpneighbor_t;
use core::ffi::c_int;
use native_math::vector::vec3_t;

/// Raven `MAX_NEIGHBOR_SIZE`.
///
/// Source: `oracle/codemp/game/q_shared.h:994`
pub const MAX_NEIGHBOR_SIZE: usize = 32;

/// Raven `wpobject_t` (`wpobject_s`) — a bot-navigation waypoint node.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:1007-1020`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct wpobject_t {
    pub origin: vec3_t,
    pub inuse: c_int,
    pub index: c_int,
    pub weight: f32,
    pub disttonext: f32,
    pub flags: c_int,
    pub associated_entity: c_int,
    pub forceJumpTo: c_int,
    pub neighbornum: c_int,
    pub neighbors: [wpneighbor_t; MAX_NEIGHBOR_SIZE],
}

const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(size_of::<wpobject_t>() == 300);
    assert!(offset_of!(wpobject_t, neighbornum) == 40);
    assert!(offset_of!(wpobject_t, neighbors) == 44);
};
