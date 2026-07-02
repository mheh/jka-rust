#![allow(non_camel_case_types, non_snake_case)]
use core::ffi::c_int;

use mp_qshared::shared::vec3_t;

use super::msurface_s::msurface_t;

/// Raven `bmodel_t` — an inline (brush) model's bounds and surface range.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:938-942`
#[repr(C)]
pub struct bmodel_t {
    // for culling
    pub bounds: [vec3_t; 2],
    pub firstSurface: *mut msurface_t,
    pub numSurfaces: c_int,
}

const _: () = assert!(core::mem::size_of::<bmodel_t>() == 40);
const _: () = assert!(core::mem::offset_of!(bmodel_t, bounds) == 0);
const _: () = assert!(core::mem::offset_of!(bmodel_t, firstSurface) == 24);
const _: () = assert!(core::mem::offset_of!(bmodel_t, numSurfaces) == 32);
