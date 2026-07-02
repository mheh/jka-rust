#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::{qboolean, vec3_t};

/// Raven `sphere_t` — bounding sphere used for capsule/sphere collision.
///
/// Type definition source: `oracle/oracle/code/qcommon/cm_local.h:228-234`
#[repr(C)]
pub struct sphere_t {
    pub use_: qboolean,
    pub radius: f32,
    pub halfheight: f32,
    pub offset: vec3_t,
}

const _: () = assert!(core::mem::size_of::<sphere_t>() == 24);
const _: () = assert!(core::mem::offset_of!(sphere_t, use_) == 0);
const _: () = assert!(core::mem::offset_of!(sphere_t, radius) == 4);
const _: () = assert!(core::mem::offset_of!(sphere_t, halfheight) == 8);
const _: () = assert!(core::mem::offset_of!(sphere_t, offset) == 12);
