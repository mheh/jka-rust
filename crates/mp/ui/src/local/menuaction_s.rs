#![allow(non_camel_case_types, non_snake_case)]

use super::menucommon_s::menucommon_s;

/// Raven `menuaction_s`.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:221-224`
#[repr(C)]
pub struct menuaction_s {
	pub generic: menucommon_s,
}

const _: () = assert!(core::mem::size_of::<menuaction_s>() == 88);
const _: () = assert!(core::mem::offset_of!(menuaction_s, generic) == 0);
