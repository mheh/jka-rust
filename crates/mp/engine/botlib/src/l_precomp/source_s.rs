#![allow(non_camel_case_types, non_snake_case)]

use super::define_s::define_t;
use super::indent_s::indent_t;
use crate::l_script::punctuation_s::punctuation_t;
use crate::l_script::script_s::script_t;
use crate::l_script::token_s::token_t;

/// Raven `source_t` — an open preprocessor source: its script stack, macro
/// definitions, punctuation table, and pending tokens.
///
/// Raven: file name of the script / path to include files / punctuations to
/// use / stack with scripts of the source / tokens to read first / list with
/// macro definitions / hash chain with defines / stack with indents / > 0 if
/// skipping conditional code / last read token.
/// Type definition source: `oracle/oracle/codemp/botlib/l_precomp.h:80-92`
#[repr(C)]
pub struct source_t {
    /// file name of the script
    pub filename: [core::ffi::c_char; 1024],
    /// path to include files
    pub includepath: [core::ffi::c_char; 1024],
    /// punctuations to use
    pub punctuations: *mut punctuation_t,
    /// stack with scripts of the source
    pub scriptstack: *mut script_t,
    /// tokens to read first
    pub tokens: *mut token_t,
    /// list with macro definitions
    pub defines: *mut define_t,
    /// hash chain with defines
    pub definehash: *mut *mut define_t,
    /// stack with indents
    pub indentstack: *mut indent_t,
    /// > 0 if skipping conditional code
    pub skip: core::ffi::c_int,
    /// last read token
    pub token: token_t,
}

pub type source_s = source_t;

const _: () = assert!(core::mem::size_of::<source_t>() == 3184);
const _: () = assert!(core::mem::offset_of!(source_t, filename) == 0);
const _: () = assert!(core::mem::offset_of!(source_t, includepath) == 1024);
const _: () = assert!(core::mem::offset_of!(source_t, punctuations) == 2048);
const _: () = assert!(core::mem::offset_of!(source_t, scriptstack) == 2056);
const _: () = assert!(core::mem::offset_of!(source_t, tokens) == 2064);
const _: () = assert!(core::mem::offset_of!(source_t, defines) == 2072);
const _: () = assert!(core::mem::offset_of!(source_t, definehash) == 2080);
const _: () = assert!(core::mem::offset_of!(source_t, indentstack) == 2088);
const _: () = assert!(core::mem::offset_of!(source_t, skip) == 2096);
const _: () = assert!(core::mem::offset_of!(source_t, token) == 2104);
