//! C-string boundary conversions — the one home for crossing between Rust
//! strings and NUL-terminated byte buffers (DEC-32). Callers at the ABI seam
//! cast pointers themselves; these fns stay pointer-free.

use std::ffi::CString;

/// Decode a NUL-terminated (or full-length) byte buffer into an owned
/// `String` — the read twin of [`crate::q_strncpyz::Q_strncpyzBytes`]. Decodes
/// Latin-1 via [`latin1_to_string`], so every byte survives (C string data is
/// bytes, not UTF-8); ASCII is unaffected.
pub fn buf_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    latin1_to_string(&buf[..end])
}

/// NUL-terminated copy of `s` for a `const char *` argument.
pub fn cstr(s: &str) -> CString {
    CString::new(s).unwrap()
}

/// Bijective Latin-1 decode of a wire/byte string into a `String`: every byte
/// `b` becomes `char::from(b)` (U+0000..U+00FF), so all 256 byte values survive
/// losslessly. This is the decode for every game-domain byte string (chat/
/// userinfo/configstrings, file tokens, enumerated filenames) — retail carries
/// non-ASCII bytes verbatim, and a lossy UTF-8 decode would fold them all onto
/// U+FFFD. Reserve `String::from_utf8*` for bytes that are genuinely UTF-8 text.
pub fn latin1_to_string(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// Bijective Latin-1 encode of a `String`/`&str` back to wire bytes: every char
/// `c <= U+00FF` maps to its single byte `c as u8`, inverting [`latin1_to_string`].
/// Chars above U+00FF are unreachable from our own Latin-1 decodes (they only
/// arise from hand-written string literals with multi-byte codepoints); such a
/// char maps to `b'.'` so the output stays one byte per char and never emits a
/// truncated multi-byte UTF-8 sequence onto the wire.
pub fn string_to_latin1(s: &str) -> Vec<u8> {
    s.chars()
        .map(|c| if (c as u32) <= 0xFF { c as u8 } else { b'.' })
        .collect()
}

/// Byte-truncating `Q_strncpyz` into an owned `String` — the migrated-field
/// twin of [`crate::q_strncpyz::Q_strncpyz`] for struct fields that became
/// `String`. Takes `src` up to its first NUL, keeps at most `destsize - 1`
/// bytes (Raven's `sizeof(dest)` bound), and Latin-1-decodes the result. A zero
/// `destsize` yields the empty string (Raven's `destsize < 1` is a `Com_Error`).
pub fn strncpyz_string(src: &[u8], destsize: usize) -> String {
    if destsize == 0 {
        return String::new();
    }
    let end = src.iter().position(|&b| b == 0).unwrap_or(src.len());
    let n = end.min(destsize - 1);
    latin1_to_string(&src[..n])
}

#[cfg(test)]
mod cstr_tests {
    use super::{buf_to_string, cstr, latin1_to_string, string_to_latin1};

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

    #[test]
    fn latin1_all_256_bytes_round_trip() {
        for b in 0u8..=255 {
            let s = latin1_to_string(&[b]);
            assert_eq!(s.chars().count(), 1, "byte {b:#04x} must be exactly one char");
            let back = string_to_latin1(&s);
            assert_eq!(back, vec![b], "byte {b:#04x} must round-trip identically");
        }
    }

    #[test]
    fn latin1_mixed_string_round_trips() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let s = latin1_to_string(&bytes);
        assert_eq!(string_to_latin1(&s), bytes);
    }

    #[test]
    fn string_to_latin1_defangs_wide_char() {
        // A codepoint above U+00FF cannot come from our own decodes; it maps to '.'.
        assert_eq!(string_to_latin1("\u{100}"), vec![b'.']);
    }
}
