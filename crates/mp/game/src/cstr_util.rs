//! Seam string helpers for the C ABI boundary.
//!
//! The shared set (`atoi`, `cstr`, `cstr_to_str`, `write_cstr_field`) lives in
//! `mp_bg::cstr_util` — it moved down with the Stage-5 bg split and is
//! re-exported here so game importers keep one canonical path. Only
//! `cstr_from_chars` (no bg consumer) stays local. Every fn is a
//! pointer-facing shape that retires with the trap-wrapper `String`
//! migration — do not add to it (value logic lives in `native_string`).

use core::ffi::c_char;
use std::ffi::CStr;

pub use mp_bg::cstr_util::{atoi, cstr, cstr_to_str, write_cstr_field};

/// Borrow a Rust-owned `[c_char]` buffer (a fixed `char[N]` struct field or a
/// stack local) as a `&CStr`, reading up to the first NUL.
///
/// Replaces `unsafe { CStr::from_ptr(buf.as_ptr()) }` at sites where `buf` is a
/// Rust-owned array reachable by safe field access: the scan is bounded by the
/// slice length instead of C's unbounded `strlen`, so a well-terminated buffer
/// yields the identical bytes with no `unsafe` at the call site.
///
/// Porting-rules §19: an unterminated buffer is a `strlen`-past-the-end UB in
/// Raven; the one defined behavior picked here is to panic rather than read out
/// of bounds. Raven's game buffers are always `Com_sprintf`/`Q_strncpyz`-
/// terminated, so this never fires on real data.
pub fn cstr_from_chars(a: &[c_char]) -> &CStr {
    // Sound: `c_char` and `u8` are both 1-byte with identical alignment and
    // every bit pattern is valid for each, so the slice reinterpret is a pure
    // type pun over the same bytes and length.
    let bytes = unsafe { core::slice::from_raw_parts(a.as_ptr() as *const u8, a.len()) };
    CStr::from_bytes_until_nul(bytes)
        .expect("cstr_from_chars: Rust-owned char buffer is not NUL-terminated")
}

#[cfg(test)]
mod cstr_from_chars_tests {
    use super::cstr_from_chars;
    use core::ffi::c_char;

    #[test]
    fn reads_up_to_first_nul_ignoring_trailing_garbage() {
        // A `char[8]` holding "abc\0" plus stale bytes past the terminator, as a
        // real fixed field would after a shorter string was written over it.
        let buf: [c_char; 8] = [
            b'a' as c_char,
            b'b' as c_char,
            b'c' as c_char,
            0,
            b'X' as c_char,
            b'Y' as c_char,
            b'Z' as c_char,
            b'!' as c_char,
        ];
        assert_eq!(cstr_from_chars(&buf).to_bytes(), b"abc");
    }

    #[test]
    fn empty_string_buffer() {
        let buf: [c_char; 4] = [0; 4];
        assert_eq!(cstr_from_chars(&buf).to_bytes(), b"");
    }

    #[test]
    #[should_panic(expected = "not NUL-terminated")]
    fn unterminated_buffer_panics() {
        let buf: [c_char; 3] = [b'a' as c_char, b'b' as c_char, b'c' as c_char];
        let _ = cstr_from_chars(&buf);
    }
}
