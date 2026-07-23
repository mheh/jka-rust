#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::c_int;

/// Raven `token_t` — a single lexer token.
///
/// Idiomatic redesign (porting-rules §F17): Raven's `char string[MAX_TOKEN]`
/// fixed buffer becomes an owned `String`; the `char *whitespace_p` /
/// `char *endwhitespace_p` pair (only ever consumed as a *length*, by
/// `PC_WhiteSpaceBeforeToken`'s `endwhitespace_p - whitespace_p > 0`, never
/// dereferenced) becomes `whitespace_span`, a byte range into the script
/// buffer that produced the token; and the `token_s *next` chain link dissolves
/// (token lists are now their owners' `Vec<Token>` / `VecDeque<Token>`). Raven
/// builds `string` one `char` at a time and bounds it at `MAX_TOKEN` inside the
/// `PS_Read*` readers; that truncation is preserved at those sites.
///
/// Type definition source: `oracle/codemp/botlib/l_script.h:141-155`
#[derive(Clone, Default)]
pub struct Token {
    /// available token
    pub string: String,
    /// last read token type
    pub type_: c_int,
    /// last read token sub type
    pub subtype: c_int,
    /// integer value (Raven `unsigned long int`)
    pub intvalue: u64,
    /// floating point value (Raven `long double`; `f64` is the closest Rust
    /// representation — the lexer never relies on 80-bit extended precision)
    pub floatvalue: f64,
    /// byte span `(begin, end)` of the white space preceding this token, into
    /// the script buffer it was read from. `None` when unset. Only the span
    /// *length* is ever observed (`PC_WhiteSpaceBeforeToken`); the bytes are
    /// never re-read, so the range need not outlive the buffer.
    pub whitespace_span: Option<(usize, usize)>,
    /// line the token was on
    pub line: c_int,
    /// lines crossed in white space
    pub linescrossed: c_int,
}
