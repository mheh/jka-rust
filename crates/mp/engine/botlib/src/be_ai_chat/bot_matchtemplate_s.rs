#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use super::bot_matchpiece_s::bot_matchpiece_t;

/// Raven `bot_matchtemplate_t` — a match template keyed by context bitmask.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:124-131`
#[repr(C)]
pub struct bot_matchtemplate_t {
    pub context: c_ulong,
    pub r#type: i32,
    pub subtype: i32,
    pub first: *mut bot_matchpiece_t,
    pub next: *mut bot_matchtemplate_t,
}

pub type bot_matchtemplate_s = bot_matchtemplate_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_matchtemplate_t>() == 32);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, context) == 0);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, r#type) == 8);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, subtype) == 12);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, first) == 16);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, next) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_matchtemplate_t>() == 20);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, context) == 0);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, r#type) == 4);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, subtype) == 8);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, first) == 12);
    assert!(core::mem::offset_of!(bot_matchtemplate_t, next) == 16);
};
