#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

use sp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfGridMesh_t` — a bi-cubic patch (curved surface) mesh.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:652-674`
#[repr(C)]
pub struct srfGridMesh_s {
    pub surfaceType: surfaceType_t,

    // dynamic lighting information
    pub dlightBits: i32,

    // culling information
    pub meshBounds: [vec3_t; 2],
    pub localOrigin: vec3_t,
    pub meshRadius: f32,

    // lod information, which may be different
    // than the culling information to allow for
    // groups of curves that LOD as a unit
    pub lodOrigin: vec3_t,
    pub lodRadius: f32,

    // vertexes
    pub width: i32,
    pub height: i32,
    pub widthLodError: *mut f32,
    pub heightLodError: *mut f32,
    pub verts: [drawVert_t; 1], // variable sized
}

pub type srfGridMesh_t = srfGridMesh_s;

const _: () = assert!(core::mem::size_of::<srfGridMesh_t>() == 168);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, dlightBits) == 4);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, meshBounds) == 8);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, localOrigin) == 32);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, meshRadius) == 44);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodOrigin) == 48);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodRadius) == 60);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, width) == 64);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, height) == 68);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, widthLodError) == 72);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, heightLodError) == 80);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, verts) == 88);
