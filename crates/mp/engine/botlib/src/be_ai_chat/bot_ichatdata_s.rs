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

const _: () = assert!(core::mem::size_of::<bot_ichatdata_t>() == 136);
const _: () = assert!(core::mem::offset_of!(bot_ichatdata_t, chat) == 0);
const _: () = assert!(core::mem::offset_of!(bot_ichatdata_t, filename) == 8);
const _: () = assert!(core::mem::offset_of!(bot_ichatdata_t, chatname) == 72);
