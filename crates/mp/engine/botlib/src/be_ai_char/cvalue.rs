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

const _: () = assert!(core::mem::size_of::<cvalue>() == 8);
