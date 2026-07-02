#![allow(non_camel_case_types, non_snake_case)]

/// Raven `field_t` — an editable text field (cursor/scroll/width + buffer).
///
/// Type definition source: `oracle/oracle/codemp/client/keys.h:12-17`
#[repr(C)]
pub struct field_t {
	pub cursor: i32,
	pub scroll: i32,
	pub widthInChars: i32,
	// Raven's `#define MAX_EDIT_LINE 256` (oracle/oracle/codemp/client/keys.h:9).
	pub buffer: [core::ffi::c_char; 256],
}

const _: () = assert!(core::mem::size_of::<field_t>() == 268);
const _: () = assert!(core::mem::offset_of!(field_t, cursor) == 0);
const _: () = assert!(core::mem::offset_of!(field_t, scroll) == 4);
const _: () = assert!(core::mem::offset_of!(field_t, widthInChars) == 8);
const _: () = assert!(core::mem::offset_of!(field_t, buffer) == 12);
