#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `modInfo_t` — mod name/description pair.
///
/// Type definition source: `oracle/code/ui/ui_local.h:98-101`
#[repr(C)]
pub struct modInfo_t {
	pub modName: *const c_char,
	pub modDescr: *const c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<modInfo_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(modInfo_t, modName) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(modInfo_t, modDescr) == 8);
