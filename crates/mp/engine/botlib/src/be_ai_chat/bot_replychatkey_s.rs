#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::bot_matchpiece_s::bot_matchpiece_t;

/// Raven `bot_replychatkey_t` — a reply-chat key.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:134-140`
#[repr(C)]
pub struct bot_replychatkey_t {
    pub flags: i32,
    pub string: *mut c_char,
    pub r#match: *mut bot_matchpiece_t,
    pub next: *mut bot_replychatkey_t,
}

pub type bot_replychatkey_s = bot_replychatkey_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_replychatkey_t>() == 32);
    assert!(core::mem::offset_of!(bot_replychatkey_t, flags) == 0);
    assert!(core::mem::offset_of!(bot_replychatkey_t, string) == 8);
    assert!(core::mem::offset_of!(bot_replychatkey_t, r#match) == 16);
    assert!(core::mem::offset_of!(bot_replychatkey_t, next) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_replychatkey_t>() == 16);
    assert!(core::mem::offset_of!(bot_replychatkey_t, flags) == 0);
    assert!(core::mem::offset_of!(bot_replychatkey_t, string) == 4);
    assert!(core::mem::offset_of!(bot_replychatkey_t, r#match) == 8);
    assert!(core::mem::offset_of!(bot_replychatkey_t, next) == 12);
};
