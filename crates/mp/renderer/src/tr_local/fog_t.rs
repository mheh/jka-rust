#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::{qboolean, vec3_t};

use super::fog_parms_t::fogParms_t;

/// Raven `fog_t` — a fog volume.
///
/// Type definition source: `oracle/oracle/codemp/renderer/tr_local.h:616-627`
#[repr(C)]
pub struct fog_t {
	pub originalBrushNumber: i32,
	pub bounds: [vec3_t; 2],

	/// in packed byte format
	pub colorInt: u32,
	/// texture coordinate vector scales
	pub tcScale: f32,
	pub parms: fogParms_t,

	// for clipping distance in fog when outside
	pub hasSurface: qboolean,
	pub surface: [f32; 4],
}

const _: () = assert!(core::mem::size_of::<fog_t>() == 72);
const _: () = assert!(core::mem::offset_of!(fog_t, originalBrushNumber) == 0);
const _: () = assert!(core::mem::offset_of!(fog_t, bounds) == 4);
const _: () = assert!(core::mem::offset_of!(fog_t, colorInt) == 28);
const _: () = assert!(core::mem::offset_of!(fog_t, tcScale) == 32);
const _: () = assert!(core::mem::offset_of!(fog_t, parms) == 36);
const _: () = assert!(core::mem::offset_of!(fog_t, hasSurface) == 52);
const _: () = assert!(core::mem::offset_of!(fog_t, surface) == 56);
