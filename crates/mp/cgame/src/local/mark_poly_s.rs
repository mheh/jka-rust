#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::qhandle_t;
use mp_qshared::common::mp::cgame::poly_s::poly_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::shared::qboolean;

/// Raven `MAX_VERTS_ON_POLY`.
///
/// Source: `oracle/codemp/cgame/cg_local.h:56`
pub const MAX_VERTS_ON_POLY: usize = 10;

/// Raven `markPoly_t`.
///
/// Type definition source: `oracle/codemp/cgame/cg_local.h:470-478`
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
