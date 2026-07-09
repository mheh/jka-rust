#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, qhandle_t};

/// Raven `characterInfo`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:599-606`
#[repr(C)]
pub struct characterInfo {
	pub name: *const c_char,
	pub imageName: *const c_char,
	pub headImage: qhandle_t,
	pub base: *const c_char,
	pub active: qboolean,
	pub reference: i32,
}

const _: () = assert!(core::mem::size_of::<characterInfo>() == 40);
const _: () = assert!(core::mem::offset_of!(characterInfo, name) == 0);
const _: () = assert!(core::mem::offset_of!(characterInfo, imageName) == 8);
const _: () = assert!(core::mem::offset_of!(characterInfo, headImage) == 16);
const _: () = assert!(core::mem::offset_of!(characterInfo, base) == 24);
const _: () = assert!(core::mem::offset_of!(characterInfo, active) == 32);
const _: () = assert!(core::mem::offset_of!(characterInfo, reference) == 36);
