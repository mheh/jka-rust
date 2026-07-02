#![allow(non_camel_case_types, non_snake_case)]

use super::punctuation_s::punctuation_t;
use super::token_s::token_t;

/// Raven `script_t` — a tokenizer script buffer and its lexing cursor state.
///
/// Raven: file name of the script / buffer containing the script / current
/// pointer in the script / pointer to the end of the script / script pointer
/// before reading token / begin of the white space / end of the white space /
/// length of the script in bytes / current line in script / line before
/// reading token / set by UnreadLastToken / several script flags / the
/// punctuations used in the script / available token / next script in a
/// chain.
/// Type definition source: `oracle/oracle/codemp/botlib/l_script.h:158-176`
#[repr(C)]
pub struct script_t {
    /// file name of the script
    pub filename: [core::ffi::c_char; 1024],
    /// buffer containing the script
    pub buffer: *mut core::ffi::c_char,
    /// current pointer in the script
    pub script_p: *mut core::ffi::c_char,
    /// pointer to the end of the script
    pub end_p: *mut core::ffi::c_char,
    /// script pointer before reading token
    pub lastscript_p: *mut core::ffi::c_char,
    /// begin of the white space
    pub whitespace_p: *mut core::ffi::c_char,
    /// end of the white space
    pub endwhitespace_p: *mut core::ffi::c_char,
    /// length of the script in bytes
    pub length: core::ffi::c_int,
    /// current line in script
    pub line: core::ffi::c_int,
    /// line before reading token
    pub lastline: core::ffi::c_int,
    /// set by UnreadLastToken
    pub tokenavailable: core::ffi::c_int,
    /// several script flags
    pub flags: core::ffi::c_int,
    /// the punctuations used in the script
    pub punctuations: *mut punctuation_t,
    pub punctuationtable: *mut *mut punctuation_t,
    /// available token
    pub token: token_t,
    /// next script in a chain
    pub next: *mut script_t,
}

pub type script_s = script_t;

const _: () = assert!(core::mem::size_of::<script_t>() == 2200);
const _: () = assert!(core::mem::offset_of!(script_t, filename) == 0);
const _: () = assert!(core::mem::offset_of!(script_t, buffer) == 1024);
const _: () = assert!(core::mem::offset_of!(script_t, script_p) == 1032);
const _: () = assert!(core::mem::offset_of!(script_t, end_p) == 1040);
const _: () = assert!(core::mem::offset_of!(script_t, lastscript_p) == 1048);
const _: () = assert!(core::mem::offset_of!(script_t, whitespace_p) == 1056);
const _: () = assert!(core::mem::offset_of!(script_t, endwhitespace_p) == 1064);
const _: () = assert!(core::mem::offset_of!(script_t, length) == 1072);
const _: () = assert!(core::mem::offset_of!(script_t, line) == 1076);
const _: () = assert!(core::mem::offset_of!(script_t, lastline) == 1080);
const _: () = assert!(core::mem::offset_of!(script_t, tokenavailable) == 1084);
const _: () = assert!(core::mem::offset_of!(script_t, flags) == 1088);
const _: () = assert!(core::mem::offset_of!(script_t, punctuations) == 1096);
const _: () = assert!(core::mem::offset_of!(script_t, punctuationtable) == 1104);
const _: () = assert!(core::mem::offset_of!(script_t, token) == 1112);
const _: () = assert!(core::mem::offset_of!(script_t, next) == 2192);
