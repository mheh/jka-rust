#![allow(non_camel_case_types, non_snake_case)]

use std::ffi::c_void;

/// Raven `XFORM_FUNCTION` — MP3 transform callback.
///
/// Type definition source: `oracle/codemp/client/../mp3code/mp3struct.h:14-14`
pub type XFORM_FUNCTION = extern "C" fn(*mut c_void, i32) -> ();
const _: () = assert!(core::mem::size_of::<XFORM_FUNCTION>() == 8);
