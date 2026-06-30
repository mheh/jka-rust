//! Shared Raven collision types from `q_shared.h`.
//!
//! Source: `oracle/oracle/code/game/q_shared.h:1355-1363`
//! Source: `oracle/oracle/codemp/game/q_shared.h:1858-1866`

#![allow(non_camel_case_types)]

use core::ffi::c_float;

use crate::shared::vec3_t;

// plane_t structure
// !!! if this is changed, it must be changed in asm code too !!!
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct cplane_t {
    pub normal: vec3_t,
    pub dist: c_float,
    pub r#type: u8,   // for fast side tests: 0,1,2 = axial, 3 = nonaxial
    pub signbits: u8, // signx + (signy<<1) + (signz<<2), used as lookup during collision
    pub pad: [u8; 2],
}
