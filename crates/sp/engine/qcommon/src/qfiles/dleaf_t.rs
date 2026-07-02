#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dleaf_t` — BSP leaf.
///
/// Type definition source: `oracle/oracle/code/qcommon/../qcommon/qfiles.h:477-489`
#[repr(C)]
pub struct dleaf_t {
	pub cluster: i32, // -1 = opaque cluster (do I still store these?)
	pub area: i32,

	pub mins: [i32; 3], // for frustum culling
	pub maxs: [i32; 3],

	pub firstLeafSurface: i32,
	pub numLeafSurfaces: i32,

	pub firstLeafBrush: i32,
	pub numLeafBrushes: i32,
}

const _: () = assert!(core::mem::size_of::<dleaf_t>() == 48);
const _: () = assert!(core::mem::offset_of!(dleaf_t, cluster) == 0);
const _: () = assert!(core::mem::offset_of!(dleaf_t, area) == 4);
const _: () = assert!(core::mem::offset_of!(dleaf_t, mins) == 8);
const _: () = assert!(core::mem::offset_of!(dleaf_t, maxs) == 20);
const _: () = assert!(core::mem::offset_of!(dleaf_t, firstLeafSurface) == 32);
const _: () = assert!(core::mem::offset_of!(dleaf_t, numLeafSurfaces) == 36);
const _: () = assert!(core::mem::offset_of!(dleaf_t, firstLeafBrush) == 40);
const _: () = assert!(core::mem::offset_of!(dleaf_t, numLeafBrushes) == 44);
