#![allow(non_camel_case_types, non_snake_case)]

/// Raven `dmodel_t` — BSP submodel.
///
/// Type definition source: `oracle/oracle/codemp/qcommon/../qcommon/qfiles.h:441-445`
#[repr(C)]
pub struct dmodel_t {
	pub mins: [f32; 3],
	pub maxs: [f32; 3],
	pub firstSurface: i32,
	pub numSurfaces: i32,
	pub firstBrush: i32,
	pub numBrushes: i32,
}

const _: () = assert!(core::mem::size_of::<dmodel_t>() == 40);
const _: () = assert!(core::mem::offset_of!(dmodel_t, mins) == 0);
const _: () = assert!(core::mem::offset_of!(dmodel_t, maxs) == 12);
const _: () = assert!(core::mem::offset_of!(dmodel_t, firstSurface) == 24);
const _: () = assert!(core::mem::offset_of!(dmodel_t, numSurfaces) == 28);
const _: () = assert!(core::mem::offset_of!(dmodel_t, firstBrush) == 32);
const _: () = assert!(core::mem::offset_of!(dmodel_t, numBrushes) == 36);
