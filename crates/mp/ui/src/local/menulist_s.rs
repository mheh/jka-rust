#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::menucommon_s::menucommon_s;

/// Raven `menulist_s`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:204-219`
#[repr(C)]
pub struct menulist_s {
	pub generic: menucommon_s,

	pub oldvalue: i32,
	pub curvalue: i32,
	pub numitems: i32,
	pub top: i32,

	pub itemnames: *mut *const c_char,

	pub width: i32,
	pub height: i32,
	pub columns: i32,
	pub seperation: i32,
}

const _: () = assert!(core::mem::size_of::<menulist_s>() == 128);
const _: () = assert!(core::mem::offset_of!(menulist_s, generic) == 0);
const _: () = assert!(core::mem::offset_of!(menulist_s, oldvalue) == 88);
const _: () = assert!(core::mem::offset_of!(menulist_s, curvalue) == 92);
const _: () = assert!(core::mem::offset_of!(menulist_s, numitems) == 96);
const _: () = assert!(core::mem::offset_of!(menulist_s, top) == 100);
const _: () = assert!(core::mem::offset_of!(menulist_s, itemnames) == 104);
const _: () = assert!(core::mem::offset_of!(menulist_s, width) == 112);
const _: () = assert!(core::mem::offset_of!(menulist_s, height) == 116);
const _: () = assert!(core::mem::offset_of!(menulist_s, columns) == 120);
const _: () = assert!(core::mem::offset_of!(menulist_s, seperation) == 124);
