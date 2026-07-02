#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::renderer::ref_entity_t::refEntity_t;
use sp_qshared::shared::{qboolean, vec3_t};

/// Raven `trRefEntity_t`.
///
/// Type definition source: `oracle/oracle/code/renderer/tr_local.h:54-66`
#[repr(C)]
pub struct trRefEntity_t {
	pub e: refEntity_t,

	/// compensate for non-normalized axis
	pub axisLength: f32,

	/// true for bmodels that touch a dlight
	pub needDlights: qboolean,
	pub lightingCalculated: qboolean,
	/// normalized direction towards light
	pub lightDir: vec3_t,
	/// color normalized to 0-255
	pub ambientLight: vec3_t,
	/// 32 bit rgba packed
	pub ambientLightInt: i32,
	pub directedLight: vec3_t,
	pub dlightBits: i32,
}

const _: () = assert!(core::mem::size_of::<trRefEntity_t>() == 232);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, e) == 0);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, axisLength) == 176);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, needDlights) == 180);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, lightingCalculated) == 184);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, lightDir) == 188);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, ambientLight) == 200);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, ambientLightInt) == 212);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, directedLight) == 216);
const _: () = assert!(core::mem::offset_of!(trRefEntity_t, dlightBits) == 228);
