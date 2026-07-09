#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

/// Raven `DECODE_FUNCTION` — an mp3 decode function pointer.
///
/// Type definition source: `oracle/code/mp3code/mp3struct.h:15-15`
pub type DECODE_FUNCTION = extern "C" fn(*mut c_void, *mut c_void) -> super::in_out::IN_OUT;
