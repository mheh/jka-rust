#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong};

/// Raven `token_t` — a single lexer token, plus its surrounding whitespace and
/// chain link.
///
/// Raven: available token / last read token type / last read token sub type /
/// integer value / floating point value / start of white space before token /
/// start of white space before token / line the token was on / lines crossed
/// in white space / next token in chain.
/// Type definition source: `oracle/codemp/botlib/l_script.h:141-155`
#[repr(C)]
pub struct token_t {
    /// available token
    pub string: [c_char; 1024], // MAX_TOKEN
    /// last read token type
    pub r#type: c_int,
    /// last read token sub type
    pub subtype: c_int,
    // Raven `unsigned long int` — platform-width, 4 bytes on ILP32.
    /// integer value
    pub intvalue: c_ulong,
    // Raven's `long double` is 8 bytes wide on the platform this struct was
    // asserted against (matches the 8-byte gap before `whitespace_p`); `f64`
    // is the closest Rust representation.
    /// floating point value
    pub floatvalue: f64,
    /// start of white space before token
    pub whitespace_p: *mut c_char,
    /// start of white space before token
    pub endwhitespace_p: *mut c_char,
    /// line the token was on
    pub line: c_int,
    /// lines crossed in white space
    pub linescrossed: c_int,
    /// next token in chain
    pub next: *mut token_t,
}

pub type token_s = token_t;

const _: () = assert!(core::mem::offset_of!(token_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(token_t, r#type) == 1024);
const _: () = assert!(core::mem::offset_of!(token_t, subtype) == 1028);
const _: () = assert!(core::mem::offset_of!(token_t, intvalue) == 1032);
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::size_of::<token_t>() == 1080);
    assert!(core::mem::offset_of!(token_t, floatvalue) == 1040);
    assert!(core::mem::offset_of!(token_t, whitespace_p) == 1048);
    assert!(core::mem::offset_of!(token_t, endwhitespace_p) == 1056);
    assert!(core::mem::offset_of!(token_t, line) == 1064);
    assert!(core::mem::offset_of!(token_t, linescrossed) == 1068);
    assert!(core::mem::offset_of!(token_t, next) == 1072);
};
// ILP32 twin. Diverges from clang i386 from `floatvalue` on: SysV i386
// `long double` is 12 bytes, Rust's `f64` stand-in is 8 (engine-internal type,
// never ABI-crossing).
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::size_of::<token_t>() == 1064);
    assert!(core::mem::offset_of!(token_t, floatvalue) == 1036);
    assert!(core::mem::offset_of!(token_t, whitespace_p) == 1044);
    assert!(core::mem::offset_of!(token_t, endwhitespace_p) == 1048);
    assert!(core::mem::offset_of!(token_t, line) == 1052);
    assert!(core::mem::offset_of!(token_t, linescrossed) == 1056);
    assert!(core::mem::offset_of!(token_t, next) == 1060);
};
