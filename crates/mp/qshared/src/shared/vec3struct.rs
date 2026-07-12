#![allow(non_camel_case_types)]

use core::ffi::c_float;

/// Raven `vec3struct_t` — a struct-wrapped vec3 used by the LCC interpreter for
/// efficient `VectorCopy` (defined under `#ifdef __LCC__`; ported as a plain
/// struct).
///
/// Type definition source: `oracle/codemp/game/q_shared.h:1389-1391`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct vec3struct_t {
    pub v: [c_float; 3],
}

const _: () = {
    assert!(core::mem::size_of::<vec3struct_t>() == 12);
};
