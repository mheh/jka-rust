#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use mp_qshared::shared::MAX_QPATH;

use super::bot_chat_s::bot_chat_t;

/// Raven `bot_ichatdata_t` — a cached initial-chat file entry (one per client
/// slot in `ichatdata[]`), keyed by `filename`/`chatname`.
///
/// Type definition source: `oracle/codemp/botlib/be_ai_chat.cpp:175-179`
#[repr(C)]
pub struct bot_ichatdata_t {
    pub chat: *mut bot_chat_t,
    pub filename: [c_char; MAX_QPATH as usize],
    pub chatname: [c_char; MAX_QPATH as usize],
}

pub type bot_ichatdata_s = bot_ichatdata_t;

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<bot_ichatdata_t>() == 136);
    assert!(core::mem::offset_of!(bot_ichatdata_t, chat) == 0);
    assert!(core::mem::offset_of!(bot_ichatdata_t, filename) == 8);
    assert!(core::mem::offset_of!(bot_ichatdata_t, chatname) == 72);
};
// ILP32 twin: clang i386 ground truth (msvc and linux-gnu agree).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<bot_ichatdata_t>() == 132);
    assert!(core::mem::offset_of!(bot_ichatdata_t, chat) == 0);
    assert!(core::mem::offset_of!(bot_ichatdata_t, filename) == 4);
    assert!(core::mem::offset_of!(bot_ichatdata_t, chatname) == 68);
};
