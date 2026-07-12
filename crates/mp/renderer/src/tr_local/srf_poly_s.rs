#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::qhandle_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfPoly_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:692-698`
#[repr(C)]
pub struct srfPoly_t {
    pub surfaceType: surfaceType_t,
    pub hShader: qhandle_t,
    pub fogIndex: i32,
    pub numVerts: i32,
    pub verts: *mut polyVert_t,
}

/// Raven `srfPoly_s` is the C tag; `srfPoly_t` is the typedef used everywhere.
pub type srfPoly_s = srfPoly_t;

const _: () = assert!(core::mem::offset_of!(srfPoly_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, hShader) == 4);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, fogIndex) == 8);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, numVerts) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<srfPoly_t>() == 24);
    assert!(core::mem::offset_of!(srfPoly_t, verts) == 16);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<srfPoly_t>() == 20);
    assert!(core::mem::offset_of!(srfPoly_t, verts) == 16);
};
