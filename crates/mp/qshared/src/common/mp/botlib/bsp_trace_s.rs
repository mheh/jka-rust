#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_float, c_int};

use crate::shared::{cplane_t, qboolean, vec3_t};

use super::bsp_surface_s::bsp_surface_t;

/// Raven `bsp_trace_t` — result of a bot BSP collision trace.
///
/// Type definition source: `oracle/oracle/codemp/game/botlib.h:117-129`
#[repr(C)]
pub struct bsp_trace_t {
	/// if true, plane is not valid
	pub allsolid: qboolean,
	/// if true, the initial point was in a solid area
	pub startsolid: qboolean,
	/// time completed, 1.0 = didn't hit anything
	pub fraction: c_float,
	/// final position
	pub endpos: vec3_t,
	/// surface normal at impact
	pub plane: cplane_t,
	/// expanded plane distance
	pub exp_dist: c_float,
	/// number of the brush side hit
	pub sidenum: c_int,
	/// the hit point surface
	pub surface: bsp_surface_t,
	/// contents on other side of surface hit
	pub contents: c_int,
	/// number of entity hit
	pub ent: c_int,
}

pub type bsp_trace_s = bsp_trace_t;

const _: () = assert!(core::mem::size_of::<bsp_trace_t>() == 84);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, allsolid) == 0);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, startsolid) == 4);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, fraction) == 8);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, endpos) == 12);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, plane) == 24);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, exp_dist) == 44);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, sidenum) == 48);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, surface) == 52);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, contents) == 76);
const _: () = assert!(core::mem::offset_of!(bsp_trace_t, ent) == 80);
