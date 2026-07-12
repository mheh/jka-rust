#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::qboolean;

/// Raven `libvar_s` — a bot library variable (cvar-like linked list node).
///
/// Type definition source: `oracle/codemp/botlib/l_libvar.h:16-24`
#[repr(C)]
pub struct libvar_t {
    pub name: *mut c_char,
    pub string: *mut c_char,
    pub flags: i32,
    /// set each time the cvar is changed
    pub modified: qboolean,
    pub value: f32,
    pub next: *mut libvar_t,
}

pub type libvar_s = libvar_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<libvar_t>() == 40);
    assert!(core::mem::offset_of!(libvar_t, name) == 0);
    assert!(core::mem::offset_of!(libvar_t, string) == 8);
    assert!(core::mem::offset_of!(libvar_t, flags) == 16);
    assert!(core::mem::offset_of!(libvar_t, modified) == 20);
    assert!(core::mem::offset_of!(libvar_t, value) == 24);
    assert!(core::mem::offset_of!(libvar_t, next) == 32);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<libvar_t>() == 24);
    assert!(core::mem::offset_of!(libvar_t, name) == 0);
    assert!(core::mem::offset_of!(libvar_t, string) == 4);
    assert!(core::mem::offset_of!(libvar_t, flags) == 8);
    assert!(core::mem::offset_of!(libvar_t, modified) == 12);
    assert!(core::mem::offset_of!(libvar_t, value) == 16);
    assert!(core::mem::offset_of!(libvar_t, next) == 20);
};
