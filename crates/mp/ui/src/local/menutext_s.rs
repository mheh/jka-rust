#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::menucommon_s::menucommon_s;

/// Raven `menutext_s` — a static text menu item.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:244-250`
#[repr(C)]
pub struct menutext_s {
    pub generic: menucommon_s,
    pub string: *mut c_char,
    pub style: i32,
    pub color: *mut f32,
}

const _: () = assert!(core::mem::size_of::<menutext_s>() == 112);
const _: () = assert!(core::mem::offset_of!(menutext_s, generic) == 0);
const _: () = assert!(core::mem::offset_of!(menutext_s, string) == 88);
const _: () = assert!(core::mem::offset_of!(menutext_s, style) == 96);
const _: () = assert!(core::mem::offset_of!(menutext_s, color) == 104);
