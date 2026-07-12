#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

// Raven's `#define MAX_ATTACHMENT_NAME 64`.
// Source: `oracle/codemp/game/ai_main.h:18`
pub const MAX_ATTACHMENT_NAME: usize = 64;

/// Raven `botattachment_t` — a named bot attachment level.
///
/// Type definition source: `oracle/codemp/game/ai_main.h:109-113`
#[repr(C)]
pub struct botattachment_t {
    pub level: c_int,
    pub name: [u8; MAX_ATTACHMENT_NAME],
}

const _: () = assert!(core::mem::size_of::<botattachment_t>() == 68);
const _: () = assert!(core::mem::offset_of!(botattachment_t, level) == 0);
const _: () = assert!(core::mem::offset_of!(botattachment_t, name) == 4);
