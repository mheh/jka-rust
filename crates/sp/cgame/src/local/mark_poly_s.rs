#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::shared::qhandle_t;
use sp_qshared::common::sp::renderer::poly_s::poly_t;
use sp_qshared::common::sp::renderer::poly_vert_t::polyVert_t;
use sp_qshared::shared::qboolean;

/// Raven `MAX_VERTS_ON_POLY`.
///
/// Source: `oracle/code/cgame/cg_local.h:47`
pub const MAX_VERTS_ON_POLY: usize = 10;

/// Raven `markPoly_t`.
///
/// Type definition source: `oracle/code/cgame/cg_local.h:184-192`
#[repr(C)]
pub struct markPoly_t {
	pub prevMark: *mut markPoly_t,
	pub nextMark: *mut markPoly_t,
	pub time: i32,
	pub markShader: qhandle_t,
	/// fade alpha instead of rgb
	pub alphaFade: qboolean,
	pub color: [f32; 4],
	pub poly: poly_t,
	pub verts: [polyVert_t; MAX_VERTS_ON_POLY],
}

const _: () = assert!(core::mem::size_of::<markPoly_t>() == 304);
const _: () = assert!(core::mem::offset_of!(markPoly_t, prevMark) == 0);
const _: () = assert!(core::mem::offset_of!(markPoly_t, nextMark) == 8);
const _: () = assert!(core::mem::offset_of!(markPoly_t, time) == 16);
const _: () = assert!(core::mem::offset_of!(markPoly_t, markShader) == 20);
const _: () = assert!(core::mem::offset_of!(markPoly_t, alphaFade) == 24);
const _: () = assert!(core::mem::offset_of!(markPoly_t, color) == 28);
const _: () = assert!(core::mem::offset_of!(markPoly_t, poly) == 48);
const _: () = assert!(core::mem::offset_of!(markPoly_t, verts) == 64);
