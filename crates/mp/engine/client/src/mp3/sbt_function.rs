#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SBT_FUNCTION` — MP3 synthesis back-transform callback.
///
/// Type definition source: `oracle/oracle/codemp/client/../mp3code/mp3struct.h:13-13`
pub type SBT_FUNCTION = extern "C" fn(*mut f32, *mut i16, i32) -> ();
const _: () = assert!(core::mem::size_of::<SBT_FUNCTION>() == 8);
