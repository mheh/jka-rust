//! SP trace result definition copied from Raven `code/game/q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:1380-1395`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::{cplane_t, qboolean, vec3_t};

pub const MAX_G2_COLLISIONS: usize = 16;

// a trace is returned when a box is swept through the world
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct trace_t {
    pub allsolid: qboolean,   // if true, plane is not valid
    pub startsolid: qboolean, // if true, the initial point was in a solid area
    pub fraction: c_float,    // time completed, 1.0 = didn't hit anything
    pub endpos: vec3_t,       // final position
    pub plane: cplane_t,      // surface normal at impact, transformed to world space
    pub surfaceFlags: c_int,  // surface hit
    pub contents: c_int,      // contents on other side of surface hit
    pub entityNum: c_int,     // entity the contacted sirface is a part of

    // Ghoul2 Insert Start
    // FIXME: create type CCollisionRecord
    // pub G2CollisionMap: [CCollisionRecord; MAX_G2_COLLISIONS], // map that describes all of the parts of ghoul2 models that got hit
    // Ghoul2 Insert End
}

// trace->entityNum can also be 0 to (MAX_GENTITIES-1)
// or ENTITYNUM_NONE, ENTITYNUM_WORLD
