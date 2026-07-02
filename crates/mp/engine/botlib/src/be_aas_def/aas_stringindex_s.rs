#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `aas_stringindex_t` — an indexed table of strings.
///
/// Type definition source: `oracle/oracle/codemp/botlib/be_aas_def.h:43-47`
#[repr(C)]
pub struct aas_stringindex_t {
	pub numindexes: i32,
	pub index: *mut *mut c_char,
}

/// Raven's C tag is `aas_stringindex_s`; the typedef name `aas_stringindex_t`
/// is house style for the struct itself.
pub type aas_stringindex_s = aas_stringindex_t;

const _: () = assert!(core::mem::size_of::<aas_stringindex_t>() == 16);
const _: () = assert!(core::mem::offset_of!(aas_stringindex_t, numindexes) == 0);
const _: () = assert!(core::mem::offset_of!(aas_stringindex_t, index) == 8);
