#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

use super::punctuation_s::Punctuation;
use super::token_s::Token;

/// Raven `script_t` — a tokenizer script buffer and its lexing cursor state.
///
/// Idiomatic redesign (porting-rules §F17): the malloc'd `char *buffer` (packed
/// after the struct) becomes an owned `Vec<u8>` (NUL-terminated at `length`),
/// and the six raw `char *` cursors into it — `script_p`/`end_p`/`lastscript_p`/
/// `whitespace_p`/`endwhitespace_p` (all rewound and compared, never aliased) —
/// become `usize` byte indices into `buffer`. `filename` owns its bytes as a
/// `String`. The `punctuations`/`punctuationtable` pair collapses to a single
/// `&'static [Punctuation]` reference (the `PUNCTABLE` bucket table is never
/// built here); `SetScriptPunctuations` sets it, defaulting to the crate's
/// `DEFAULT_PUNCTUATIONS`. The `script_s *next` chain link dissolves — the
/// script stack now lives in `Source::scriptstack` as a `Vec<Script>`.
///
/// Type definition source: `oracle/codemp/botlib/l_script.h:158-176`
pub struct Script {
    /// file name of the script
    pub filename: String,
    /// buffer containing the script (owns its bytes; NUL-terminated at `length`)
    pub buffer: Vec<u8>,
    /// current byte index in the script
    pub script_p: usize,
    /// byte index of the end of the script
    pub end_p: usize,
    /// script index before reading token
    pub lastscript_p: usize,
    /// begin of the white space
    pub whitespace_p: usize,
    /// end of the white space
    pub endwhitespace_p: usize,
    /// length of the script in bytes
    pub length: c_int,
    /// current line in script
    pub line: c_int,
    /// line before reading token
    pub lastline: c_int,
    /// set by UnreadLastToken
    pub tokenavailable: c_int,
    /// several script flags
    pub flags: c_int,
    /// the punctuations used in the script
    pub punctuations: &'static [Punctuation],
    /// available token
    pub token: Token,
}
