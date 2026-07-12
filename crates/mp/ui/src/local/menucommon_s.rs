#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use super::menuframework_s::menuframework_s;

/// Raven `menucommon_s` — base fields shared by every menu item widget.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:160-177`
#[repr(C)]
pub struct menucommon_s {
    pub r#type: c_int,
    pub name: *const c_char,
    pub id: c_int,
    pub x: c_int,
    pub y: c_int,
    pub left: c_int,
    pub top: c_int,
    pub right: c_int,
    pub bottom: c_int,
    pub parent: *mut menuframework_s,
    pub menuPosition: c_int,
    pub flags: u32,

    pub callback: Option<unsafe extern "C" fn(self_: *mut c_void, event: c_int)>,
    pub statusbar: Option<unsafe extern "C" fn(self_: *mut c_void)>,
    pub ownerdraw: Option<unsafe extern "C" fn(self_: *mut c_void)>,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<menucommon_s>() == 88);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, r#type) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, name) == 8);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, id) == 16);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, x) == 20);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, y) == 24);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, left) == 28);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, top) == 32);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, right) == 36);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, bottom) == 40);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, parent) == 48);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, menuPosition) == 56);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, flags) == 60);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, callback) == 64);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, statusbar) == 72);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menucommon_s, ownerdraw) == 80);
