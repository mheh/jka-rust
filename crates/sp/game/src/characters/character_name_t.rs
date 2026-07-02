#![allow(non_camel_case_types, non_snake_case)]

use std::os::raw::c_char;

use sp_qshared::shared::sfxHandle_t;

/// Raven `characterName_t` — a named sound reference paired with its resolved handle.
///
/// Type definition source: `oracle/oracle/code/game/characters.h:47-52`
#[repr(C)]
pub struct characterName_t {
    pub name: *mut c_char,
    pub sound: *mut c_char,
    pub soundIndex: sfxHandle_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<characterName_t>() == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(characterName_t, name) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(characterName_t, sound) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(characterName_t, soundIndex) == 16);
