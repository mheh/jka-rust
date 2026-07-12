#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `ammoData_t` — per-ammo-type data.
///
/// Type definition source: `oracle/code/game/weapons.h:142-146`
#[repr(C)]
pub struct ammoData_t {
    pub icon: [c_char; 32], // Name of ammo icon file
    pub max: i32,           // Max amount player can hold of ammo
}

const _: () = assert!(core::mem::size_of::<ammoData_t>() == 36);
const _: () = assert!(core::mem::offset_of!(ammoData_t, icon) == 0);
const _: () = assert!(core::mem::offset_of!(ammoData_t, max) == 32);
