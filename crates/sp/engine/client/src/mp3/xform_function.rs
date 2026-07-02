#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

/// Raven `XFORM_FUNCTION` — an mp3 transform function pointer.
///
/// Type definition source: `oracle/oracle/code/mp3code/mp3struct.h:14-14`
pub type XFORM_FUNCTION = extern "C" fn(*mut c_void, i32);
