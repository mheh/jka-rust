#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::shared::vec3_t;

/// Raven `ragCallbackDebugLine_t` — ragdoll debug-line callback payload.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_public.h:550-557`
#[repr(C)]
pub struct ragCallbackDebugLine_t {
	pub start: vec3_t,
	pub end: vec3_t,
	pub time: i32,
	pub color: i32,
	pub radius: i32,
}

const _: () = assert!(core::mem::size_of::<ragCallbackDebugLine_t>() == 36);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugLine_t, start) == 0);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugLine_t, end) == 12);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugLine_t, time) == 24);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugLine_t, color) == 28);
const _: () = assert!(core::mem::offset_of!(ragCallbackDebugLine_t, radius) == 32);
