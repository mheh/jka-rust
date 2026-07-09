//! MP `bg_public.h` field descriptor.
//!
//! Type definition source: `oracle/codemp/game/bg_public.h:1263-1269`

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

use super::fieldtype::fieldtype_t;

/// Raven `BG_field_t`.
///
/// Type definition source: `oracle/codemp/game/bg_public.h:1263-1269`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BG_field_t {
    pub name: *mut c_char,
    pub ofs: c_int,
    pub r#type: fieldtype_t,
    pub flags: c_int,
}

// `name` is always initialized from a `'static` string literal (Raven's
// spawn-field tables are `static const fieldDescriptor_t` arrays of C-string
// literals cast to `char *`) and never mutated, so sharing a `BG_field_t`
// table across threads is sound despite the raw pointer.
unsafe impl Sync for BG_field_t {}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<BG_field_t>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, ofs) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, r#type) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(BG_field_t, flags) == 16);
