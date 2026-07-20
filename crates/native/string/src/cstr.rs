//! C-string boundary conversions — the one home for crossing between Rust
//! strings and NUL-terminated byte buffers (DEC-32). Callers at the ABI seam
//! cast pointers themselves; these fns stay pointer-free.

use std::ffi::CString;

/// Decode a NUL-terminated (or full-length) byte buffer into an owned
/// `String` (lossy) — the read twin of [`crate::q_strncpyz::Q_strncpyzBytes`].
pub fn buf_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).into_owned()
}

/// NUL-terminated copy of `s` for a `const char *` argument.
pub fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[cfg(test)]
mod cstr_tests {
    use super::{buf_to_string, cstr};

    #[test]
    fn stops_at_nul() {
        assert_eq!(buf_to_string(b"abc\0def"), "abc");
    }

    #[test]
    fn no_nul_takes_whole_buffer() {
        assert_eq!(buf_to_string(b"abc"), "abc");
    }

    #[test]
    fn cstr_round_trip() {
        assert_eq!(cstr("hello").to_bytes(), b"hello");
    }
}
