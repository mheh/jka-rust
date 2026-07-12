#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::bot_randomstring_s::bot_randomstring_t;

/// Raven `bot_randomlist_t` — a list with random strings.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:84-90`
#[repr(C)]
pub struct bot_randomlist_t {
    pub string: *mut c_char,
    pub numstrings: i32,
    pub firstrandomstring: *mut bot_randomstring_t,
    pub next: *mut bot_randomlist_t,
}

pub type bot_randomlist_s = bot_randomlist_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_randomlist_t>() == 32);
    assert!(core::mem::offset_of!(bot_randomlist_t, string) == 0);
    assert!(core::mem::offset_of!(bot_randomlist_t, numstrings) == 8);
    assert!(core::mem::offset_of!(bot_randomlist_t, firstrandomstring) == 16);
    assert!(core::mem::offset_of!(bot_randomlist_t, next) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_randomlist_t>() == 16);
    assert!(core::mem::offset_of!(bot_randomlist_t, string) == 0);
    assert!(core::mem::offset_of!(bot_randomlist_t, numstrings) == 4);
    assert!(core::mem::offset_of!(bot_randomlist_t, firstrandomstring) == 8);
    assert!(core::mem::offset_of!(bot_randomlist_t, next) == 12);
};
