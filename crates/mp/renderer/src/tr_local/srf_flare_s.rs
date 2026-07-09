#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

use super::surface_type_t::surfaceType_t;

/// Raven `srfFlare_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:715-720`
#[repr(C)]
pub struct srfFlare_t {
	pub surfaceType: surfaceType_t,
	pub origin: vec3_t,
	pub normal: vec3_t,
	pub color: vec3_t,
}

/// Raven `srfFlare_s` is the C tag; `srfFlare_t` is the typedef used everywhere.
pub type srfFlare_s = srfFlare_t;

const _: () = assert!(core::mem::size_of::<srfFlare_t>() == 40);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, origin) == 4);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, normal) == 16);
const _: () = assert!(core::mem::offset_of!(srfFlare_t, color) == 28);
