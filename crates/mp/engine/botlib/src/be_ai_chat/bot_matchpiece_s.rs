#![allow(non_camel_case_types, non_snake_case)]

use super::bot_matchstring_s::bot_matchstring_t;

/// Raven `bot_matchpiece_t` — one piece of a match template.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:116-122`
#[repr(C)]
pub struct bot_matchpiece_t {
    pub r#type: i32,
    pub firststring: *mut bot_matchstring_t,
    pub variable: i32,
    pub next: *mut bot_matchpiece_t,
}

pub type bot_matchpiece_s = bot_matchpiece_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_matchpiece_t>() == 32);
    assert!(core::mem::offset_of!(bot_matchpiece_t, r#type) == 0);
    assert!(core::mem::offset_of!(bot_matchpiece_t, firststring) == 8);
    assert!(core::mem::offset_of!(bot_matchpiece_t, variable) == 16);
    assert!(core::mem::offset_of!(bot_matchpiece_t, next) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_matchpiece_t>() == 16);
    assert!(core::mem::offset_of!(bot_matchpiece_t, r#type) == 0);
    assert!(core::mem::offset_of!(bot_matchpiece_t, firststring) == 4);
    assert!(core::mem::offset_of!(bot_matchpiece_t, variable) == 8);
    assert!(core::mem::offset_of!(bot_matchpiece_t, next) == 12);
};
