#![allow(non_camel_case_types, non_snake_case)]

use super::menucommon_s::menucommon_s;

/// Raven `menuslider_s` — a slider menu item.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:193-202`
#[repr(C)]
pub struct menuslider_s {
    pub generic: menucommon_s,

    pub minvalue: f32,
    pub maxvalue: f32,
    pub curvalue: f32,

    pub range: f32,
}

const _: () = assert!(core::mem::size_of::<menuslider_s>() == 104);
const _: () = assert!(core::mem::offset_of!(menuslider_s, generic) == 0);
const _: () = assert!(core::mem::offset_of!(menuslider_s, minvalue) == 88);
const _: () = assert!(core::mem::offset_of!(menuslider_s, maxvalue) == 92);
const _: () = assert!(core::mem::offset_of!(menuslider_s, curvalue) == 96);
const _: () = assert!(core::mem::offset_of!(menuslider_s, range) == 100);
