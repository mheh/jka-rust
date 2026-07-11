#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `keyname_t` — a named-key binding table entry (keyname -> keynum).
///
/// Type definition source: `oracle/code/client/keys.h:36-43`
#[repr(C)]
pub struct keyname_t {
    pub upper: u16,
    pub lower: u16,
    pub name: *mut c_char,
    pub keynum: i32,
    pub menukey: bool,
}

const _: () = assert!(core::mem::size_of::<keyname_t>() == 24);
const _: () = assert!(core::mem::offset_of!(keyname_t, upper) == 0);
const _: () = assert!(core::mem::offset_of!(keyname_t, lower) == 2);
const _: () = assert!(core::mem::offset_of!(keyname_t, name) == 8);
const _: () = assert!(core::mem::offset_of!(keyname_t, keynum) == 16);
const _: () = assert!(core::mem::offset_of!(keyname_t, menukey) == 20);
