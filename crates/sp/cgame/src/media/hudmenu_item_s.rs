#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;
use core::ffi::c_int;

use sp_qshared::shared::{qhandle_t, vec4_t};

/// Raven `HUDMenuItem_t` — an on-screen HUD menu item (position, size, color, background).
///
/// Type definition source: `oracle/oracle/code/cgame/cg_media.h:43-53`
#[repr(C)]
pub struct HUDMenuItem_t {
    pub menuName: *mut c_char,
    pub itemName: *mut c_char,
    pub xPos: c_int,
    pub yPos: c_int,
    pub width: c_int,
    pub height: c_int,
    pub color: vec4_t,
    pub background: qhandle_t,
}

const _: () = assert!(core::mem::size_of::<HUDMenuItem_t>() == 56);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, menuName) == 0);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, itemName) == 8);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, xPos) == 16);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, yPos) == 20);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, width) == 24);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, height) == 28);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, color) == 32);
const _: () = assert!(core::mem::offset_of!(HUDMenuItem_t, background) == 48);
