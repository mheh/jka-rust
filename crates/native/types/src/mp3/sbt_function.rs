#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SBT_FUNCTION` — MP3 synthesis back-transform callback.
///
/// Type definition source: `oracle/codemp/client/../mp3code/mp3struct.h:13-13`
/// Type definition source: `oracle/code/client/../mp3code/mp3struct.h:13-13`
pub type SBT_FUNCTION = extern "C" fn(*mut f32, *mut i16, i32) -> ();
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<SBT_FUNCTION>() == 8);
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<SBT_FUNCTION>() == 4);
