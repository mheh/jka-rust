#![allow(non_camel_case_types, non_snake_case)]

/// Raven `SBT_FUNCTION` — an mp3 sample synthesis function pointer.
///
/// Type definition source: `oracle/code/mp3code/mp3struct.h:13-13`
pub type SBT_FUNCTION = extern "C" fn(*mut f32, *mut i16, i32);
