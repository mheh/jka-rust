#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `graphsamp_t` — one sample of the `SCR_DebugGraph` ring.
///
/// Type definition source: `oracle/codemp/client/cl_scrn.cpp:312-316`
#[repr(C)]
pub struct graphsamp_t {
    pub value: f32,
    pub color: c_int,
}

const _: () = assert!(core::mem::size_of::<graphsamp_t>() == 8);
const _: () = assert!(core::mem::offset_of!(graphsamp_t, value) == 0);
const _: () = assert!(core::mem::offset_of!(graphsamp_t, color) == 4);

// Both fields are scalars, and Raven's `values` is a zero-filled file static.
unsafe impl native_platform::ZeroValid for graphsamp_t {}
