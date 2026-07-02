#![allow(non_camel_case_types, non_snake_case)]

/// Raven `DECODE_FUNCTION` — MP3 bitstream decoder callback.
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:15-15`
pub type DECODE_FUNCTION = extern "C" fn(*mut u8, *mut u8) -> ();
const _: () = assert!(core::mem::size_of::<DECODE_FUNCTION>() == 8);
