//! MP trace result definition copied from Raven `codemp/game/q_shared.h`.
//!
//! Source: `oracle/oracle/codemp/game/q_shared.h:1894-1912`

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::{c_float, c_int, c_short};

use crate::shared::{cplane_t, vec3_t};

// a trace is returned when a box is swept through the world
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct trace_t {
    pub allsolid: u8,       // if true, plane is not valid
    pub startsolid: u8,     // if true, the initial point was in a solid area
    pub entityNum: c_short, // entity the contacted sirface is a part of

    pub fraction: c_float,   // time completed, 1.0 = didn't hit anything
    pub endpos: vec3_t,      // final position
    pub plane: cplane_t,     // surface normal at impact, transformed to world space
    pub surfaceFlags: c_int, // surface hit
    pub contents: c_int,     // contents on other side of surface hit

    // Ghoul2 Insert Start
    //rww - removed this for now, it's just wasting space in the trace structure.
    // pub G2CollisionMap: [CollisionRecord_t; MAX_G2_COLLISIONS], // map that describes all of the parts of ghoul2 models that got hit
    // Ghoul2 Insert End
}

// trace->entityNum can also be 0 to (MAX_GENTITIES-1)
// or ENTITYNUM_NONE, ENTITYNUM_WORLD
