#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::poly_vert_t::polyVert_t;
use sp_qshared::shared::qhandle_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfPoly_t` — a dynamically-added polygon surface.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:620-626`
#[repr(C)]
pub struct srfPoly_s {
    pub surfaceType: surfaceType_t,
    pub hShader: qhandle_t,
    pub fogIndex: i32,
    pub numVerts: i32,
    pub verts: *mut polyVert_t,
}

pub type srfPoly_t = srfPoly_s;

const _: () = assert!(core::mem::size_of::<srfPoly_t>() == 24);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, hShader) == 4);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, fogIndex) == 8);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, numVerts) == 12);
const _: () = assert!(core::mem::offset_of!(srfPoly_t, verts) == 16);
