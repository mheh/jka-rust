#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::{qboolean, vec3_t};

/// Raven `pml_t` — pmove-local scratch state (not part of the wire-visible
/// `playerState_t`).
///
/// Type definition source: `oracle/oracle/codemp/game/bg_local.h:15-31`
#[repr(C)]
pub struct pml_t {
	pub forward: vec3_t,
	pub right: vec3_t,
	pub up: vec3_t,
	pub frametime: f32,

	pub msec: i32,

	pub walking: qboolean,
	pub groundPlane: qboolean,
	pub groundTrace: trace_t,

	pub impactSpeed: f32,

	pub previous_origin: vec3_t,
	pub previous_velocity: vec3_t,
	pub previous_waterlevel: i32,
}

const _: () = assert!(core::mem::size_of::<pml_t>() == 132);
const _: () = assert!(core::mem::offset_of!(pml_t, forward) == 0);
const _: () = assert!(core::mem::offset_of!(pml_t, right) == 12);
const _: () = assert!(core::mem::offset_of!(pml_t, up) == 24);
const _: () = assert!(core::mem::offset_of!(pml_t, frametime) == 36);
const _: () = assert!(core::mem::offset_of!(pml_t, msec) == 40);
const _: () = assert!(core::mem::offset_of!(pml_t, walking) == 44);
const _: () = assert!(core::mem::offset_of!(pml_t, groundPlane) == 48);
const _: () = assert!(core::mem::offset_of!(pml_t, groundTrace) == 52);
const _: () = assert!(core::mem::offset_of!(pml_t, impactSpeed) == 100);
const _: () = assert!(core::mem::offset_of!(pml_t, previous_origin) == 104);
const _: () = assert!(core::mem::offset_of!(pml_t, previous_velocity) == 116);
const _: () = assert!(core::mem::offset_of!(pml_t, previous_waterlevel) == 128);
