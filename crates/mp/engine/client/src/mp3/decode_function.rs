#![allow(non_camel_case_types, non_snake_case)]

/// Raven `DECODE_FUNCTION` — MP3 bitstream decoder callback.
///
/// Type definition source: `oracle/codemp/client/../mp3code/mp3struct.h:15-15`
pub type DECODE_FUNCTION = extern "C" fn(*mut u8, *mut u8) -> ();
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<DECODE_FUNCTION>() == 8);
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<DECODE_FUNCTION>() == 4);
