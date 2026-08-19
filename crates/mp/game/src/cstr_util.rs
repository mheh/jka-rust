//! C ABI string helpers for game code.
//!
//! The shared set (`atoi`, `cstr`, `cstr_to_str`, `write_cstr_field`) lives in
//! `mp_bg::cstr_util`, and this module re-exports it so game importers use one path.
//! Only `cstr_from_chars` stays here, because `mp_bg` has no consumer for it.
//! Every function in this file has a pointer-facing shape.
//! These functions retire when the trap-wrapper migration to owned `String` is done.
//! Do not add new functions here. Value logic lives in `native_string`.

use core::ffi::c_char;
use std::ffi::CStr;

pub use mp_bg::cstr_util::{atoi, cstr, cstr_to_str, write_cstr_field};

/// Borrow a Rust-owned `[c_char]` buffer (a fixed `char[N]` struct field or a
/// stack local) as a `&CStr`, reading up to the first NUL.
///
/// This replaces `unsafe { CStr::from_ptr(buf.as_ptr()) }` at sites where `buf` is a
/// Rust-owned array reachable by safe field access.
/// The scan is bounded by the slice length instead of C's unbounded `strlen`.
/// A well-terminated buffer yields the identical bytes, with no `unsafe` at the call site.
///
/// Porting-rules §19 applies here.
/// An unterminated buffer causes UB in Raven, a `strlen` scan that runs past the end.
/// The defined behavior picked here is a panic, not a read out of bounds.
/// Raven's game buffers are always terminated by `Com_sprintf` or `Q_strncpyz`, so this never fires on real data.
pub fn cstr_from_chars(a: &[c_char]) -> &CStr {
    // Sound: `c_char` and `u8` are both 1 byte, with identical alignment.
    // Every bit pattern is valid for each type, so the slice reinterpret is a pure type pun over the same bytes and length.
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
        // A `char[8]` holds `abc\0` plus stale bytes past the terminator.
        // A real fixed field looks like this after a shorter string overwrote a longer one.
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
