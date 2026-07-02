#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `pinglist_t`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:654-657`
#[repr(C)]
pub struct pinglist_t {
	pub adrstr: [c_char; 64],
	pub start: i32,
}

const _: () = assert!(core::mem::size_of::<pinglist_t>() == 68);
const _: () = assert!(core::mem::offset_of!(pinglist_t, adrstr) == 0);
const _: () = assert!(core::mem::offset_of!(pinglist_t, start) == 64);
