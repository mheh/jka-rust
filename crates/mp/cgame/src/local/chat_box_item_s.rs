#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `MAX_SAY_TEXT`.
///
/// Source: `oracle/oracle/codemp/game/q_shared.h:402`
pub const MAX_SAY_TEXT: usize = 150;

/// Raven `chatBoxItem_t` — a single line stored in the cgame chat box history.
///
/// Type definition source: `oracle/oracle/codemp/cgame/cg_local.h:748-753`
#[repr(C)]
pub struct chatBoxItem_t {
	pub string: [c_char; MAX_SAY_TEXT],
	pub time: i32,
	pub lines: i32,
}

const _: () = assert!(core::mem::size_of::<chatBoxItem_t>() == 160);
const _: () = assert!(core::mem::offset_of!(chatBoxItem_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(chatBoxItem_t, time) == 152);
const _: () = assert!(core::mem::offset_of!(chatBoxItem_t, lines) == 156);
