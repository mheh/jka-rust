#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::cplane_t;

use super::surface_type_t::surfaceType_t;

/// `VERTEXSIZE` — non-`_XBOX` build: `6 + (MAXLIGHTMAPS * 3)` = 18 floats per point.
///
/// Source: `oracle/code/renderer/tr_local.h:684`
const VERTEXSIZE: usize = 18;

/// Raven `srfSurfaceFace_t` — planar surface (Q3 "face"), variable-sized.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:727-740`
#[repr(C)]
pub struct srfSurfaceFace_t {
    pub surfaceType: surfaceType_t,
    pub plane: cplane_t,
    /// dynamic lighting information
    pub dlightBits: i32,
    /// triangle definitions (no normals at points)
    pub numPoints: i32,
    pub numIndices: i32,
    pub ofsIndices: i32,
    /// variable sized; there is a variable length list of indices here also
    pub points: [[f32; VERTEXSIZE]; 1],
}

const _: () = assert!(core::mem::size_of::<srfSurfaceFace_t>() == 112);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, plane) == 4);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, dlightBits) == 24);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, numPoints) == 28);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, numIndices) == 32);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, ofsIndices) == 36);
const _: () = assert!(core::mem::offset_of!(srfSurfaceFace_t, points) == 40);
