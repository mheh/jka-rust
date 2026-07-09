#![allow(non_camel_case_types, non_snake_case)]

/// Raven `uifield_t` — an editable text field on a menu, with label/color styling.
///
/// Type definition source: `oracle/code/ui/ui_local.h:21-31`
#[repr(C)]
pub struct uifield_t {
	pub cursor: i32,
	pub scroll: i32,
	pub widthInChars: i32,
	// Raven's `#define MAX_EDIT_LINE 256` (oracle/code/ui/ui_local.h:19).
	pub buffer: [core::ffi::c_char; 256],
	pub maxchars: i32,
	pub style: i32,
	pub textEnum: i32,   // Label
	pub textcolor: i32,  // Normal color
	pub textcolor2: i32, // Highlight color
}

const _: () = assert!(core::mem::size_of::<uifield_t>() == 288);
const _: () = assert!(core::mem::offset_of!(uifield_t, cursor) == 0);
const _: () = assert!(core::mem::offset_of!(uifield_t, scroll) == 4);
const _: () = assert!(core::mem::offset_of!(uifield_t, widthInChars) == 8);
const _: () = assert!(core::mem::offset_of!(uifield_t, buffer) == 12);
const _: () = assert!(core::mem::offset_of!(uifield_t, maxchars) == 268);
const _: () = assert!(core::mem::offset_of!(uifield_t, style) == 272);
const _: () = assert!(core::mem::offset_of!(uifield_t, textEnum) == 276);
const _: () = assert!(core::mem::offset_of!(uifield_t, textcolor) == 280);
const _: () = assert!(core::mem::offset_of!(uifield_t, textcolor2) == 284);
