#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `modInfo_t` — a single loadable-mod list entry.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:711-714`
#[repr(C)]
pub struct modInfo_t {
	pub modName: *const c_char,
	pub modDescr: *const c_char,
}

const _: () = assert!(core::mem::size_of::<modInfo_t>() == 16);
const _: () = assert!(core::mem::offset_of!(modInfo_t, modName) == 0);
const _: () = assert!(core::mem::offset_of!(modInfo_t, modDescr) == 8);
