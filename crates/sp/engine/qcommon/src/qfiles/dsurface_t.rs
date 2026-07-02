#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::vec3_t;

/// Light Style Constants.
///
/// Source: `oracle/oracle/code/qcommon/qfiles.h:310`
const MAXLIGHTMAPS: usize = 4;

/// Raven `dsurface_t` — on-disk BSP surface (drawsurface) record.
///
/// Type definition source: `oracle/oracle/code/qcommon/qfiles.h:548-569`
#[repr(C)]
pub struct dsurface_t {
	pub shaderNum: i32,
	pub fogNum: i32,
	pub surfaceType: i32,

	pub firstVert: i32,
	pub numVerts: i32,

	pub firstIndex: i32,
	pub numIndexes: i32,

	pub lightmapStyles: [u8; MAXLIGHTMAPS],
	pub vertexStyles: [u8; MAXLIGHTMAPS],
	pub lightmapNum: [i32; MAXLIGHTMAPS],
	pub lightmapX: [i32; MAXLIGHTMAPS],
	pub lightmapY: [i32; MAXLIGHTMAPS],
	pub lightmapWidth: i32,
	pub lightmapHeight: i32,

	pub lightmapOrigin: vec3_t,
	/// for patches, [0] and [1] are lodbounds
	pub lightmapVecs: [vec3_t; 3],

	pub patchWidth: i32,
	pub patchHeight: i32,
}

const _: () = assert!(core::mem::size_of::<dsurface_t>() == 148);
const _: () = assert!(core::mem::offset_of!(dsurface_t, shaderNum) == 0);
const _: () = assert!(core::mem::offset_of!(dsurface_t, fogNum) == 4);
const _: () = assert!(core::mem::offset_of!(dsurface_t, surfaceType) == 8);
const _: () = assert!(core::mem::offset_of!(dsurface_t, firstVert) == 12);
const _: () = assert!(core::mem::offset_of!(dsurface_t, numVerts) == 16);
const _: () = assert!(core::mem::offset_of!(dsurface_t, firstIndex) == 20);
const _: () = assert!(core::mem::offset_of!(dsurface_t, numIndexes) == 24);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapStyles) == 28);
const _: () = assert!(core::mem::offset_of!(dsurface_t, vertexStyles) == 32);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapNum) == 36);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapX) == 52);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapY) == 68);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapWidth) == 84);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapHeight) == 88);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapOrigin) == 92);
const _: () = assert!(core::mem::offset_of!(dsurface_t, lightmapVecs) == 104);
const _: () = assert!(core::mem::offset_of!(dsurface_t, patchWidth) == 140);
const _: () = assert!(core::mem::offset_of!(dsurface_t, patchHeight) == 144);
