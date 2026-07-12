#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `union cvalue` — a bot characteristic value.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_char.cpp:39-44`
#[repr(C)]
pub union cvalue {
    pub integer: i32,
    pub _float: f32,
    pub string: *mut c_char,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<cvalue>() == 8);
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<cvalue>() == 4);
