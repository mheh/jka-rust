#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::l_script::token_s::token_t;

/// Raven `define_t` — a preprocessor `#define` macro entry.
///
/// Raven: (none).
/// Type definition source: `oracle/oracle/codemp/botlib/l_precomp.h:55-66`
#[repr(C)]
pub struct define_t {
    /// define name
    pub name: *mut c_char,
    /// define flags
    pub flags: c_int,
    /// > 0 if builtin define
    pub builtin: c_int,
    /// number of define parameters
    pub numparms: c_int,
    /// define parameters
    pub parms: *mut token_t,
    /// macro tokens (possibly containing parm tokens)
    pub tokens: *mut token_t,
    /// next defined macro in a list
    pub next: *mut define_t,
    /// next define in the hash chain
    pub hashnext: *mut define_t,
    /// used to link up the globald defines
    pub globalnext: *mut define_t,
}

pub type define_s = define_t;

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<define_t>() == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, flags) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, builtin) == 12);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, numparms) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, parms) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, tokens) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, next) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, hashnext) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(define_t, globalnext) == 56);
