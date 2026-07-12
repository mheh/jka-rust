#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_ulong;

use super::bot_synonym_s::bot_synonym_t;

/// Raven `bot_synonymlist_t` — a list with synonyms, keyed by context bitmask.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:100-106`
#[repr(C)]
pub struct bot_synonymlist_t {
    pub context: c_ulong,
    pub totalweight: f32,
    pub firstsynonym: *mut bot_synonym_t,
    pub next: *mut bot_synonymlist_t,
}

pub type bot_synonymlist_s = bot_synonymlist_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_synonymlist_t>() == 32);
    assert!(core::mem::offset_of!(bot_synonymlist_t, context) == 0);
    assert!(core::mem::offset_of!(bot_synonymlist_t, totalweight) == 8);
    assert!(core::mem::offset_of!(bot_synonymlist_t, firstsynonym) == 16);
    assert!(core::mem::offset_of!(bot_synonymlist_t, next) == 24);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_synonymlist_t>() == 16);
    assert!(core::mem::offset_of!(bot_synonymlist_t, context) == 0);
    assert!(core::mem::offset_of!(bot_synonymlist_t, totalweight) == 4);
    assert!(core::mem::offset_of!(bot_synonymlist_t, firstsynonym) == 8);
    assert!(core::mem::offset_of!(bot_synonymlist_t, next) == 12);
};
