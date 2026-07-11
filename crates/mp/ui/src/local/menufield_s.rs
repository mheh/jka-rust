#![allow(non_camel_case_types, non_snake_case)]

use super::menucommon_s::menucommon_s;
use super::mfield_t::mfield_t;

/// Raven `menufield_s` — a menu item widget wrapping an editable text field.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:187-191`
#[repr(C)]
pub struct menufield_s {
    pub generic: menucommon_s,
    pub field: mfield_t,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<menufield_s>() == 360);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menufield_s, generic) == 0);
#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::offset_of!(menufield_s, field) == 88);
