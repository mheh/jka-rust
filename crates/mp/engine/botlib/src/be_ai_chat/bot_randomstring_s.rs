#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_randomstring_t` — a random string list node.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:78-82`
#[repr(C)]
pub struct bot_randomstring_t {
    pub string: *mut c_char,
    pub next: *mut bot_randomstring_t,
}

pub type bot_randomstring_s = bot_randomstring_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_randomstring_t>() == 16);
    assert!(core::mem::offset_of!(bot_randomstring_t, string) == 0);
    assert!(core::mem::offset_of!(bot_randomstring_t, next) == 8);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_randomstring_t>() == 8);
    assert!(core::mem::offset_of!(bot_randomstring_t, string) == 0);
    assert!(core::mem::offset_of!(bot_randomstring_t, next) == 4);
};
