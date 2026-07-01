//! Raven parser token shared across PC parser syscall surfaces.

#![allow(non_camel_case_types)]

use core::ffi::{c_char, c_int};

/// Raven `MAX_TOKENLENGTH`.
///
/// Definition source: `oracle/oracle/codemp/game/q_shared.h:1649`
/// Definition source: `oracle/oracle/codemp/botlib/l_precomp.h:148`
pub const MAX_TOKENLENGTH: usize = 1024;

/// Raven `pc_token_t` parser token.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1657-1668`
/// Type definition source: `oracle/oracle/codemp/botlib/l_precomp.h:149-156`
/// Type definition source: `oracle/oracle/code/ui/ui_shared.h:25-32`
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct pc_token_t {
    pub type_: c_int,
    pub subtype: c_int,
    pub intvalue: c_int,
    pub floatvalue: f32,
    pub string: [c_char; MAX_TOKENLENGTH],
}

const _: () = assert!(core::mem::size_of::<pc_token_t>() == 1040);
const _: () = assert!(core::mem::offset_of!(pc_token_t, type_) == 0);
const _: () = assert!(core::mem::offset_of!(pc_token_t, subtype) == 4);
const _: () = assert!(core::mem::offset_of!(pc_token_t, intvalue) == 8);
const _: () = assert!(core::mem::offset_of!(pc_token_t, floatvalue) == 12);
const _: () = assert!(core::mem::offset_of!(pc_token_t, string) == 16);
