#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `punctuation_t` — one punctuation operator entry.
///
/// Idiomatic redesign (porting-rules §F17): the malloc-free `default_punctuations[]`
/// table is a `&'static [Punctuation]` const (in the crate root), so `char *p`
/// becomes a `&'static str` and the `punctuation_s *next` link — used only to
/// thread the never-built `PUNCTABLE` first-character buckets — dissolves. The
/// live lexer path (`PS_ReadPunctuation`) scans the length-ordered slice
/// linearly.
///
/// Type definition source: `oracle/codemp/botlib/l_script.h:133-138`
#[derive(Clone, Copy)]
pub struct Punctuation {
    /// punctuation character(s)
    pub p: &'static str,
    /// punctuation indication
    pub n: c_int,
}
