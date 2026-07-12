#![allow(non_camel_case_types, non_snake_case)]

use sp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use sp_qshared::shared::vec3_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfTriangles_t` — Q3 BSP/MD3 triangle soup surface.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:745-762`
#[repr(C)]
pub struct srfTriangles_t {
    pub surfaceType: surfaceType_t,
    /// dynamic lighting information
    pub dlightBits: i32,
    /// culling information (FIXME: use this!)
    pub bounds: [vec3_t; 2],
    // vec3_t localOrigin;
    // float radius;
    /// triangle definitions
    pub numIndexes: i32,
    pub indexes: *mut i32,
    pub numVerts: i32,
    pub verts: *mut drawVert_t,
}

const _: () = assert!(core::mem::size_of::<srfTriangles_t>() == 64);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, dlightBits) == 4);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, bounds) == 8);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, numIndexes) == 32);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, indexes) == 40);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, numVerts) == 48);
const _: () = assert!(core::mem::offset_of!(srfTriangles_t, verts) == 56);
