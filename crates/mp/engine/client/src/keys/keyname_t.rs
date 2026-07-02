#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `keyname_t` — a key name/binding table entry.
///
/// Type definition source: `oracle/oracle/codemp/client/keys.h:36-43`
#[repr(C)]
pub struct keyname_t {
    pub upper: u16,
    pub lower: u16,
    pub name: *mut core::ffi::c_char,
    pub keynum: c_int,
    pub menukey: bool,
}

const _: () = assert!(core::mem::size_of::<keyname_t>() == 24);
const _: () = assert!(core::mem::offset_of!(keyname_t, upper) == 0);
const _: () = assert!(core::mem::offset_of!(keyname_t, lower) == 2);
const _: () = assert!(core::mem::offset_of!(keyname_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(keyname_t, keynum) == 16);
const _: () = assert!(core::mem::offset_of!(keyname_t, menukey) == 20);
