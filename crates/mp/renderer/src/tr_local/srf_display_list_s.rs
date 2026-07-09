#![allow(non_camel_case_types, non_snake_case)]

use super::surface_type_t::surfaceType_t;

/// Raven `srfDisplayList_t`.
///
/// Type definition source: `oracle/codemp/renderer/tr_local.h:700-703`
#[repr(C)]
pub struct srfDisplayList_t {
	pub surfaceType: surfaceType_t,
	pub listNum: i32,
}

/// Raven `srfDisplayList_s` is the C tag; `srfDisplayList_t` is the typedef used everywhere.
pub type srfDisplayList_s = srfDisplayList_t;

const _: () = assert!(core::mem::size_of::<srfDisplayList_t>() == 8);
const _: () = assert!(core::mem::offset_of!(srfDisplayList_t, surfaceType) == 0);
const _: () = assert!(core::mem::offset_of!(srfDisplayList_t, listNum) == 4);
