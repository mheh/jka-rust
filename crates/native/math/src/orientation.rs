//! `orientation_t` — a point plus a 3×`vec3_t` rotation basis.
//!
//! Identical in SP and MP; a cross-mode math primitive.
//! Type definition source: `oracle/oracle/code/game/q_shared.h:1409-1412`
//! Type definition source: `oracle/oracle/codemp/game/q_shared.h:1926-1929`

#![allow(non_camel_case_types)]

use crate::vector::vec3_t;

/// Raven `orientation_t`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct orientation_t {
    pub origin: vec3_t,
    pub axis: [vec3_t; 3],
}

const _: () = {
    assert!(core::mem::size_of::<orientation_t>() == 48);
    assert!(core::mem::offset_of!(orientation_t, origin) == 0);
    assert!(core::mem::offset_of!(orientation_t, axis) == 12);
};
