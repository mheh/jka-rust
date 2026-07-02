#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `gameTypeInfo`.
///
/// Type definition source: `oracle/oracle/codemp/ui/ui_local.h:624-627`
#[repr(C)]
pub struct gameTypeInfo {
	pub gameType: *const c_char,
	pub gtEnum: i32,
}

const _: () = assert!(core::mem::size_of::<gameTypeInfo>() == 16);
const _: () = assert!(core::mem::offset_of!(gameTypeInfo, gameType) == 0);
const _: () = assert!(core::mem::offset_of!(gameTypeInfo, gtEnum) == 8);
