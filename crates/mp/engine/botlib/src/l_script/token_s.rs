#![allow(non_camel_case_types, non_snake_case)]

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
    pub string: [core::ffi::c_char; 1024], // MAX_TOKEN
    /// last read token type
    pub r#type: core::ffi::c_int,
    /// last read token sub type
    pub subtype: core::ffi::c_int,
    // Raven's `unsigned long int` is 8 bytes under the LP64 layout this struct
    // was asserted against.
    /// integer value
    pub intvalue: u64,
    // Raven's `long double` is 8 bytes wide on the platform this struct was
    // asserted against (matches the 8-byte gap before `whitespace_p`); `f64`
    // is the closest Rust representation.
    /// floating point value
    pub floatvalue: f64,
    /// start of white space before token
    pub whitespace_p: *mut core::ffi::c_char,
    /// start of white space before token
    pub endwhitespace_p: *mut core::ffi::c_char,
    /// line the token was on
    pub line: core::ffi::c_int,
    /// lines crossed in white space
    pub linescrossed: core::ffi::c_int,
    /// next token in chain
    pub next: *mut token_t,
}

pub type token_s = token_t;

const _: () = assert!(core::mem::size_of::<token_t>() == 1080);
const _: () = assert!(core::mem::offset_of!(token_t, string) == 0);
const _: () = assert!(core::mem::offset_of!(token_t, r#type) == 1024);
const _: () = assert!(core::mem::offset_of!(token_t, subtype) == 1028);
const _: () = assert!(core::mem::offset_of!(token_t, intvalue) == 1032);
const _: () = assert!(core::mem::offset_of!(token_t, floatvalue) == 1040);
const _: () = assert!(core::mem::offset_of!(token_t, whitespace_p) == 1048);
const _: () = assert!(core::mem::offset_of!(token_t, endwhitespace_p) == 1056);
const _: () = assert!(core::mem::offset_of!(token_t, line) == 1064);
const _: () = assert!(core::mem::offset_of!(token_t, linescrossed) == 1068);
const _: () = assert!(core::mem::offset_of!(token_t, next) == 1072);
