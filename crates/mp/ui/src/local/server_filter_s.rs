#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `serverFilter_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:649-652`
#[repr(C)]
pub struct serverFilter_t {
	pub description: *const c_char,
	pub basedir: *const c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<serverFilter_t>() == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverFilter_t, description) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(serverFilter_t, basedir) == 8);
