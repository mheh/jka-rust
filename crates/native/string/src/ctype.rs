//! C-locale `<ctype.h>` classes and case maps — what the oracle's libc calls
//! (`atoi`'s `isspace`, `Com_Filter`'s `toupper`, the `Q_is*` family) resolve
//! to in the "C" locale. `char`-fronted, cascading to `_byte` siblings for
//! parsers that walk raw bytes. All classes are pure ASCII; non-ASCII is
//! never a member and case maps leave it unchanged.

/// libc `isspace` in the "C" locale — exactly the six chars
/// `' ' '\t' '\n' '\x0b' '\x0c' '\r'` (narrower than Rust's
/// `char::is_whitespace`, wider than a bare `== ' '`).
pub fn isspace(c: char) -> bool {
    c.is_ascii() && isspace_byte(c as u8)
}

/// [`isspace`] over a raw byte.
pub fn isspace_byte(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// libc `isdigit` in the "C" locale.
pub fn isdigit(c: char) -> bool {
    c.is_ascii() && isdigit_byte(c as u8)
}

/// [`isdigit`] over a raw byte.
pub fn isdigit_byte(b: u8) -> bool {
    b.is_ascii_digit()
}

/// libc `isupper` in the "C" locale.
pub fn isupper(c: char) -> bool {
    c.is_ascii() && isupper_byte(c as u8)
}

/// [`isupper`] over a raw byte.
pub fn isupper_byte(b: u8) -> bool {
    b.is_ascii_uppercase()
}

/// libc `islower` in the "C" locale.
pub fn islower(c: char) -> bool {
    c.is_ascii() && islower_byte(c as u8)
}

/// [`islower`] over a raw byte.
pub fn islower_byte(b: u8) -> bool {
    b.is_ascii_lowercase()
}

/// libc `toupper` in the "C" locale — ASCII-only, other chars unchanged.
pub fn toupper(c: char) -> char {
    if c.is_ascii() {
        toupper_byte(c as u8) as char
    } else {
        c
    }
}

/// [`toupper`] over a raw byte — bytes >= 0x80 unchanged.
pub fn toupper_byte(b: u8) -> u8 {
    b.to_ascii_uppercase()
}

/// libc `tolower` in the "C" locale — ASCII-only, other chars unchanged.
pub fn tolower(c: char) -> char {
    if c.is_ascii() {
        tolower_byte(c as u8) as char
    } else {
        c
    }
}

/// [`tolower`] over a raw byte — bytes >= 0x80 unchanged.
pub fn tolower_byte(b: u8) -> u8 {
    b.to_ascii_lowercase()
}
