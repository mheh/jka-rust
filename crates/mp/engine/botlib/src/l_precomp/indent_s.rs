#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use crate::l_script::script_s::script_t;

/// Raven `indent_t` — a preprocessor `#if`/`#ifdef` indent stack entry.
///
/// Raven: (none).
/// Type definition source: `oracle/codemp/botlib/l_precomp.h:71-77`
#[repr(C)]
pub struct indent_t {
    /// indent type
    pub r#type: c_int,
    /// true if skipping current indent
    pub skip: c_int,
    /// script the indent was in
    pub script: *mut script_t,
    /// next indent on the indent stack
    pub next: *mut indent_t,
}

pub type indent_s = indent_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<indent_t>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(indent_t, r#type) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(indent_t, skip) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(indent_t, script) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(indent_t, next) == 16);
