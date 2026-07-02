#![allow(non_camel_case_types, non_snake_case)]

use super::menucommon_s::menucommon_s;

/// Raven `menuradiobutton_s` — a radio-button menu item.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:226-230`
#[repr(C)]
pub struct menuradiobutton_s {
	pub generic: menucommon_s,
	pub curvalue: i32,
}

const _: () = assert!(core::mem::size_of::<menuradiobutton_s>() == 96);
const _: () = assert!(core::mem::offset_of!(menuradiobutton_s, generic) == 0);
const _: () = assert!(core::mem::offset_of!(menuradiobutton_s, curvalue) == 88);
