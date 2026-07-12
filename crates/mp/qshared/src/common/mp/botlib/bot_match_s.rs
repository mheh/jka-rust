#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_char;

use super::bot_matchvariable_s::bot_matchvariable_t;

/// `MAX_MESSAGE_SIZE`.
///
/// Type definition source: `oracle/codemp/game/be_ai_chat.h:16`
pub const MAX_MESSAGE_SIZE: usize = 256;

/// `MAX_MATCHVARIABLES`.
///
/// Type definition source: `oracle/codemp/game/be_ai_chat.h:18`
pub const MAX_MATCHVARIABLES: usize = 8;

/// Raven `bot_match_t` — a matched bot chat message, with type/subtype and
/// extracted variables.
///
/// Type definition source: `oracle/codemp/game/be_ai_chat.h:45-51`
#[repr(C)]
pub struct bot_match_t {
    pub string: [c_char; MAX_MESSAGE_SIZE],
    pub r#type: i32,
    pub subtype: i32,
    pub variables: [bot_matchvariable_t; MAX_MATCHVARIABLES],
}

pub type bot_match_s = bot_match_t;

const _: () = assert!(core::mem::size_of::<bot_match_t>() == 328);
const _: () = assert!(core::mem::offset_of!(bot_match_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(bot_match_t, r#type) == 256);
const _: () = assert!(core::mem::offset_of!(bot_match_t, subtype) == 260);
const _: () = assert!(core::mem::offset_of!(bot_match_t, variables) == 264);
