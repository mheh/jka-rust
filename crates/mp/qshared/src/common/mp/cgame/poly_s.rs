//! MP `tr_types.h` polygon — arbitrary-vertex-count renderer polygon.

#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use native_types::qhandle_t;

use super::poly_vert_t::polyVert_t;

/// Raven `poly_t`.
///
/// Type definition source: `oracle/oracle/codemp/cgame/tr_types.h:77-81`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct poly_t {
    pub hShader: qhandle_t,
    pub numVerts: c_int,
    pub verts: *mut polyVert_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<poly_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, hShader) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, numVerts) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, verts) == 8);
