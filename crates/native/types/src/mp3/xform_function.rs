#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_void;

/// Raven `XFORM_FUNCTION` — MP3 transform callback.
///
/// Type definition source: `oracle/codemp/client/../mp3code/mp3struct.h:14-14`
/// Type definition source: `oracle/code/client/../mp3code/mp3struct.h:14-14`
pub type XFORM_FUNCTION = extern "C" fn(*mut c_void, i32) -> ();
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<XFORM_FUNCTION>() == 8);
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<XFORM_FUNCTION>() == 4);
