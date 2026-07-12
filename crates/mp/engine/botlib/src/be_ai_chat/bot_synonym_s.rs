#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

/// Raven `bot_synonym_t` — a synonym list node.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:93-98`
#[repr(C)]
pub struct bot_synonym_t {
    pub string: *mut c_char,
    pub weight: f32,
    pub next: *mut bot_synonym_t,
}

pub type bot_synonym_s = bot_synonym_t;

const _: () = assert!(core::mem::size_of::<bot_synonym_t>() == 24);
const _: () = assert!(core::mem::offset_of!(bot_synonym_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(bot_synonym_t, weight) == 8);
const _: () = assert!(core::mem::offset_of!(bot_synonym_t, next) == 16);
