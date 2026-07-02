#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_int, c_void};

use mp_qshared::shared::{qboolean, sfxHandle_t};
use mp_uishared::shared::menu_def_t::MAX_MENUITEMS;

/// Raven `menuframework_s` — base menu framework shared by all MP UI menus.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:144-158`
#[repr(C)]
#[derive(Clone, Copy)]
pub struct menuframework_s {
    pub cursor: c_int,
    pub cursor_prev: c_int,

    pub nitems: c_int,
    // Raven declares `void *items` (ui_local.h:150); elements point at
    // `menucommon_s`-headed widgets, but the faithful field type is `void *`.
    pub items: [*mut c_void; MAX_MENUITEMS],

    pub draw: Option<unsafe extern "C" fn()>,
    pub key: Option<unsafe extern "C" fn(key: c_int) -> sfxHandle_t>,

    pub wrapAround: qboolean,
    pub fullscreen: qboolean,
    pub showlogo: qboolean,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<menuframework_s>() == 2096);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, cursor) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, cursor_prev) == 4);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, nitems) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, items) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, draw) == 2064);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, key) == 2072);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, wrapAround) == 2080);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, fullscreen) == 2084);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menuframework_s, showlogo) == 2088);
