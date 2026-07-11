#![allow(non_camel_case_types, non_snake_case)]

/// Raven `mfield_t` — an editable text field on a menu.
///
/// Type definition source: `oracle/codemp/ui/ui_local.h:179-185`
#[repr(C)]
pub struct mfield_t {
    pub cursor: i32,
    pub scroll: i32,
    pub widthInChars: i32,
    // Raven's `#define MAX_EDIT_LINE 256` (oracle/codemp/ui/ui_local.h:98).
    pub buffer: [core::ffi::c_char; 256],
    pub maxchars: i32,
}

const _: () = assert!(core::mem::size_of::<mfield_t>() == 272);
const _: () = assert!(core::mem::offset_of!(mfield_t, cursor) == 0);
const _: () = assert!(core::mem::offset_of!(mfield_t, scroll) == 4);
const _: () = assert!(core::mem::offset_of!(mfield_t, widthInChars) == 8);
const _: () = assert!(core::mem::offset_of!(mfield_t, buffer) == 12);
const _: () = assert!(core::mem::offset_of!(mfield_t, maxchars) == 268);
