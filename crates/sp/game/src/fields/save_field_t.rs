#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use super::fieldtype_save_t::fieldtypeSAVE_t;

/// Raven `save_field_t` — one entry in a save/restore field table.
///
/// Type definition source: `oracle/code/game/fields.h:55-60`
#[repr(C)]
pub struct save_field_t {
	pub psName: *mut c_char,
	pub iOffset: c_int,
	pub eFieldType: fieldtypeSAVE_t,
}

const _: () = assert!(core::mem::size_of::<save_field_t>() == 16);
const _: () = assert!(core::mem::offset_of!(save_field_t, psName) == 0);
const _: () = assert!(core::mem::offset_of!(save_field_t, iOffset) == 8);
const _: () = assert!(core::mem::offset_of!(save_field_t, eFieldType) == 12);
