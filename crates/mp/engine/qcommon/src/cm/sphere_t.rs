#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `sphere_t` — bounding sphere used by capsule/sphere trace collision.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/cm_local.h:230-236`
#[repr(C)]
pub struct sphere_t {
    pub r#use: qboolean,
    pub radius: f32,
    pub halfheight: f32,
    pub offset: vec3_t,
}

const _: () = assert!(core::mem::size_of::<sphere_t>() == 24);
const _: () = assert!(core::mem::offset_of!(sphere_t, r#use) == 0);
const _: () = assert!(core::mem::offset_of!(sphere_t, radius) == 4);
const _: () = assert!(core::mem::offset_of!(sphere_t, halfheight) == 8);
const _: () = assert!(core::mem::offset_of!(sphere_t, offset) == 12);
