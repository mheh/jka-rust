#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::qhandle_t;

use super::menucommon_s::menucommon_s;

/// Raven `menubitmap_s`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:232-242`
#[repr(C)]
pub struct menubitmap_s {
    pub generic: menucommon_s,
    pub focuspic: *mut c_char,
    pub errorpic: *mut c_char,
    pub shader: qhandle_t,
    pub focusshader: qhandle_t,
    pub width: i32,
    pub height: i32,
    pub focuscolor: *mut f32,
}

const _: () = assert!(core::mem::size_of::<menubitmap_s>() == 128);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, generic) == 0);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, focuspic) == 88);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, errorpic) == 96);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, shader) == 104);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, focusshader) == 108);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, width) == 112);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, height) == 116);
const _: () = assert!(core::mem::offset_of!(menubitmap_s, focuscolor) == 120);
