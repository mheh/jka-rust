#![allow(non_camel_case_types, non_snake_case)]

use mp_engine_qcommon::qfiles::draw_vert_t::drawVert_t;
use mp_qshared::shared::vec3_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfGridMesh_t` — bezier patch surface (curved surface tessellation).
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:750-774`
#[repr(C)]
pub struct srfGridMesh_t {
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
	pub lodFixed: i32,
	pub lodStitched: i32,

	// vertexes
	pub width: i32,
	pub height: i32,
	pub widthLodError: *mut f32,
	pub heightLodError: *mut f32,
	pub verts: [drawVert_t; 1], // variable sized
}

/// C tag name; Raven typedefs `srfGridMesh_s` to `srfGridMesh_t`.
pub type srfGridMesh_s = srfGridMesh_t;

const _: () = assert!(core::mem::size_of::<srfGridMesh_t>() == 176);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, dlightBits) == 4);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, meshBounds) == 8);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, localOrigin) == 32);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, meshRadius) == 44);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodOrigin) == 48);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodRadius) == 60);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodFixed) == 64);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, lodStitched) == 68);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, width) == 72);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, height) == 76);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, widthLodError) == 80);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, heightLodError) == 88);
const _: () = assert!(core::mem::offset_of!(srfGridMesh_t, verts) == 96);
