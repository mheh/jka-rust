#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `aliasInfo`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:608-612`
#[repr(C)]
pub struct aliasInfo {
	pub name: *const c_char,
	pub ai: *const c_char,
	pub action: *const c_char,
}

const _: () = assert!(core::mem::size_of::<aliasInfo>() == 24);
const _: () = assert!(core::mem::offset_of!(aliasInfo, name) == 0);
const _: () = assert!(core::mem::offset_of!(aliasInfo, ai) == 8);
const _: () = assert!(core::mem::offset_of!(aliasInfo, action) == 16);
