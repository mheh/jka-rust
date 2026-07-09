#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::refdef_t::MAX_MAP_AREA_BYTES;
use sp_qshared::shared::{qboolean, vec3_t};

use super::dlight_s::dlight_t;
use super::draw_surf_s::drawSurf_t;
use super::srf_poly_s::srfPoly_t;
use super::tr_ref_entity_t::trRefEntity_t;

/// Raven `trRefdef_t`.
///
/// Type definition source: `oracle/code/renderer/tr_local.h:71-106`
#[repr(C)]
pub struct trRefdef_t {
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
	pub fov_x: f32,
	pub fov_y: f32,
	pub vieworg: vec3_t,
	/// transformation matrix
	pub viewaxis: [vec3_t; 3],

	/// time in milliseconds for shader effects and other time dependent rendering issues
	pub time: i32,
	pub frametime: i32,
	/// RDF_NOWORLDMODEL, etc
	pub rdflags: i32,

	/// 1 bits will prevent the associated area from rendering at all
	pub areamask: [u8; MAX_MAP_AREA_BYTES],
	/// qtrue if areamask changed since last scene
	pub areamaskModified: qboolean,

	/// tr.refdef.time / 1000.0
	pub floatTime: f32,

	// text messages for deform text shaders
	//	char		text[MAX_RENDER_STRINGS][MAX_RENDER_STRING_LENGTH];

	pub num_entities: i32,
	pub entities: *mut trRefEntity_t,

	pub num_dlights: i32,
	pub dlights: *mut dlight_t,

	pub numPolys: i32,
	pub polys: *mut srfPoly_t,

	pub numDrawSurfs: i32,
	pub drawSurfs: *mut drawSurf_t,

	/// what fog brush the vieworg is in
	pub fogIndex: i32,
}

const _: () = assert!(core::mem::size_of::<trRefdef_t>() == 192);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, x) == 0);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, y) == 4);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, width) == 8);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, height) == 12);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, fov_x) == 16);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, fov_y) == 20);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, vieworg) == 24);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, viewaxis) == 36);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, time) == 72);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, frametime) == 76);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, rdflags) == 80);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, areamask) == 84);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, areamaskModified) == 116);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, floatTime) == 120);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, num_entities) == 124);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, entities) == 128);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, num_dlights) == 136);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, dlights) == 144);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, numPolys) == 152);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, polys) == 160);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, numDrawSurfs) == 168);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, drawSurfs) == 176);
const _: () = assert!(core::mem::offset_of!(trRefdef_t, fogIndex) == 184);
