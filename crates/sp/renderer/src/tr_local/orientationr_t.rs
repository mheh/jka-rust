#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_float;

use sp_qshared::shared::vec3_t;

/// Raven `orientationr_t` — a resolved render-time orientation (origin, axis,
/// view-relative origin, and model matrix).
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:108-113`
#[repr(C)]
pub struct orientationr_t {
    /// in world coordinates
    pub origin: vec3_t,
    /// orientation in world
    pub axis: [vec3_t; 3],
    /// viewParms->or.origin in local coordinates
    pub viewOrigin: vec3_t,
    pub modelMatrix: [c_float; 16],
}

const _: () = assert!(core::mem::size_of::<orientationr_t>() == 124);
const _: () = assert!(core::mem::offset_of!(orientationr_t, origin) == 0);
const _: () = assert!(core::mem::offset_of!(orientationr_t, axis) == 12);
const _: () = assert!(core::mem::offset_of!(orientationr_t, viewOrigin) == 48);
const _: () = assert!(core::mem::offset_of!(orientationr_t, modelMatrix) == 60);
