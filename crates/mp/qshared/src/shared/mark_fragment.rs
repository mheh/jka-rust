#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_int;

/// Raven `markFragment_t` — returned by `CM_MarkFragments()`.
///
/// Type definition source: `oracle/codemp/game/q_shared.h:1919-1922`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct markFragment_t {
    pub firstPoint: c_int,
    pub numPoints: c_int,
}

const _: () = {
    assert!(core::mem::size_of::<markFragment_t>() == 8);
};
