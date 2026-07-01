#![allow(non_camel_case_types, non_snake_case)]

use crate::shared::qhandle_t;

use super::poly_vert_t::polyVert_t;

/// Raven `poly_s` — a dynamically-added polygon.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:76-80`
#[repr(C)]
pub struct poly_s {
    pub hShader: qhandle_t,
    pub numVerts: i32,
    pub verts: *mut polyVert_t,
}

/// Raven `poly_t` — `typedef struct poly_s poly_t`.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_types.h:76-80`
pub type poly_t = poly_s;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<poly_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, hShader) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, numVerts) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(poly_t, verts) == 8);
