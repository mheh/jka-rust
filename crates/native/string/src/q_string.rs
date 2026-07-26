//! Raven `q_shared.c` string family — safe canonical forms (string-data
//! migration, DEC-32): `&str`/slice surfaces, byte-walking bodies in `Bytes`
//! siblings for seam callers holding unconverted C data. Comparisons keep
//! Raven's signed-`char` widening (high bytes order negative).
//!
//! Source: `oracle/codemp/game/q_shared.c:855-937`

use core::ffi::{c_char, c_int};

use crate::cstr::{latin1_to_string, string_to_latin1};

/// One C-string byte as Raven's widened `int`: in-bounds bytes sign-extend
/// (`char` is signed), the end of the slice reads as the NUL.
fn c_at(s: &[u8], i: usize) -> c_int {
    s.get(i).map_or(0, |&b| b as i8 as c_int)
}

/// Raven `Q_stricmpn` over raw bytes.
///
/// Source: `oracle/codemp/game/q_shared.c:855-879`
pub fn Q_stricmpnBytes(s1: &[u8], s2: &[u8], n: usize) -> c_int {
    let mut n = n;
    let mut i = 0usize;
    loop {
        let mut c1 = c_at(s1, i);
        let mut c2 = c_at(s2, i);
        i += 1;

        if n == 0 {
            return 0;
        }
        n -= 1;

        if c1 != c2 {
            if (b'a' as c_int..=b'z' as c_int).contains(&c1) {
                c1 -= b'a' as c_int - b'A' as c_int;
            }
            if (b'a' as c_int..=b'z' as c_int).contains(&c2) {
                c2 -= b'a' as c_int - b'A' as c_int;
            }
            if c1 != c2 {
                return if c1 < c2 { -1 } else { 1 };
            }
        }
        if c1 == 0 {
            return 0;
        }
    }
}

/// Raven `Q_stricmpn`.
pub fn Q_stricmpn(s1: &str, s2: &str, n: usize) -> c_int {
    Q_stricmpnBytes(s1.as_bytes(), s2.as_bytes(), n)
}

/// Raven `Q_stricmp` over raw bytes (Raven's literal 99999-char cap).
///
/// Source: `oracle/codemp/game/q_shared.c:900-902`
pub fn Q_stricmpBytes(s1: &[u8], s2: &[u8]) -> c_int {
    Q_stricmpnBytes(s1, s2, 99999)
}

/// Raven `Q_stricmp` (the C null-argument arms are unrepresentable here; the
/// seam twin keeps them).
pub fn Q_stricmp(s1: &str, s2: &str) -> c_int {
    Q_stricmpBytes(s1.as_bytes(), s2.as_bytes())
}

/// Raven `Q_CleanStr` — strip color codes and non-printable bytes, returning
/// an owned `String` (the migrated-field twin of the pointer `Q_CleanStr`, for
/// callers whose source became a `String`).
///
/// `Q_IsColorString(s)` = `^` followed by a byte in `'0'..='7'` that is not a
/// second `^`; both the `^` and the digit are dropped. A `^` with no valid
/// digit is kept (it is itself printable). Bytes outside `0x20..=0x7E` drop.
/// Byte-positional, so the output matches Raven's in-place compaction exactly.
///
/// Source: `oracle/codemp/game/q_shared.c:963-982`
pub fn Q_CleanStr(string: &str) -> String {
    let bytes = string.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let n = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        if c == b'^' && n != 0 && n != b'^' && (b'0'..=b'7').contains(&n) {
            // Q_IsColorString(s): skip the `^` here (Raven's inner `s++`); the
            // digit is skipped by the trailing `i += 1` on this same pass.
            i += 1;
        } else if (0x20..=0x7E).contains(&c) {
            out.push(c);
        }
        i += 1;
    }
    latin1_to_string(&out)
}

/// Raven `Q_strncmp` over raw bytes.
///
/// Source: `oracle/codemp/game/q_shared.c:881-898`
pub fn Q_strncmpBytes(s1: &[u8], s2: &[u8], n: usize) -> c_int {
    let mut n = n;
    let mut i = 0usize;
    loop {
        let c1 = c_at(s1, i);
        let c2 = c_at(s2, i);
        i += 1;

        if n == 0 {
            return 0;
        }
        n -= 1;

        if c1 != c2 {
            return if c1 < c2 { -1 } else { 1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
}

/// Raven `Q_strncmp`.
pub fn Q_strncmp(s1: &str, s2: &str, n: usize) -> c_int {
    Q_strncmpBytes(s1.as_bytes(), s2.as_bytes(), n)
}

/// Raven's bare `strcmp` call sites (no `Q_*` wrapper in C) over raw bytes.
///
/// Source: `oracle/codemp/game/q_shared.c` (bare `strcmp` call sites).
pub fn Q_strcmpBytes(s1: &[u8], s2: &[u8]) -> c_int {
    let mut i = 0usize;
    loop {
        let c1 = c_at(s1, i);
        let c2 = c_at(s2, i);
        i += 1;

        if c1 != c2 {
            return if c1 < c2 { -1 } else { 1 };
        }
        if c1 == 0 {
            return 0;
        }
    }
}

/// Raven's bare `strcmp`.
pub fn Q_strcmp(s1: &str, s2: &str) -> c_int {
    Q_strcmpBytes(s1.as_bytes(), s2.as_bytes())
}

/// Raven `Q_strlwr` — ASCII-lowercase a C `char` buffer in place, stopping at
/// the NUL (owned `String`s use std's `make_ascii_lowercase` directly).
///
/// Source: `oracle/codemp/game/q_shared.c:905-914`
pub fn Q_strlwr(s: &mut [c_char]) {
    for c in s.iter_mut() {
        if *c == 0 {
            break;
        }
        *c = (*c as u8).to_ascii_lowercase() as c_char;
    }
}

/// Raven `Q_strcat` — bounded append of `src` after `dest`'s NUL; panics where
/// Raven `Com_Error(ERR_FATAL, "Q_strcat: already overflowed")` fires.
///
/// Source: `oracle/codemp/game/q_shared.c:929-937`
pub fn Q_strcat(dest: &mut [c_char], size: usize, src: &str) {
    let l1 = dest.iter().position(|&c| c == 0).unwrap_or(dest.len());
    if l1 >= size {
        panic!("Q_strcat: already overflowed");
    }
    crate::q_strncpyz::Q_strncpyz(&mut dest[l1..], src, size - l1);
}

/// Raven `Q_strcat` into an owned `String` — the migrated-field twin of
/// [`Q_strcat`] for locals that became `String`. Appends `src` after `dest`,
/// keeping at most `size - 1` total bytes (Raven's `Q_strncpyz(dest+l1, src,
/// size-l1)` bound); panics where Raven `Com_Error(ERR_FATAL, "Q_strcat: already
/// overflowed")` fires.
///
/// Lengths and the truncation are taken in the Latin-1 byte domain (one stored
/// char = one C byte), so `strlen(dest)` and the copy bound match Raven's on a
/// non-ASCII payload and the cut never lands inside a UTF-8 sequence.
///
/// Source: `oracle/codemp/game/q_shared.c:929-937`
pub fn strcat_string(dest: &mut String, size: usize, src: &str) {
    let l1 = dest.chars().count();
    if l1 >= size {
        panic!("Q_strcat: already overflowed");
    }
    let src_bytes = string_to_latin1(src);
    let n = src_bytes.len().min(size - l1 - 1);
    dest.push_str(&latin1_to_string(&src_bytes[..n]));
}

#[cfg(test)]
mod q_string_tests {
    use super::*;

    #[test]
    fn stricmp_folds_case_and_orders() {
        assert_eq!(Q_stricmp("maps/MP/duel1", "MAPS/mp/DUEL1"), 0);
        assert_eq!(Q_stricmp("abc", "abd"), -1);
        assert_eq!(Q_stricmp("abd", "abc"), 1);
    }

    #[test]
    fn stricmpn_stops_at_n() {
        assert_eq!(Q_stricmpn("abcX", "abcY", 3), 0);
        assert_eq!(Q_stricmpn("abcX", "abcY", 4), -1);
    }

    #[test]
    fn strncmp_is_case_sensitive() {
        assert_eq!(Q_strncmp("abc", "ABC", 3), 1);
        assert_eq!(Q_strncmp("abc", "abc", 8), 0);
    }

    #[test]
    fn high_bytes_order_as_signed_chars() {
        // 0x80 widens negative, so it sorts below 'A' exactly as C's signed
        // char comparison does.
        assert_eq!(Q_strcmpBytes(&[0x80, 0], &[b'A', 0]), -1);
    }

    #[test]
    fn cleanstr_strips_color_codes_and_control_bytes() {
        // `^`+digit('0'..='7') drops both bytes; a printable char survives.
        assert_eq!(Q_CleanStr("^1red^7white"), "redwhite");
        // A `^` with no valid digit is itself printable and is kept; `^8`/`^^`
        // are not color codes (matches Q_PrintStrlen's guard).
        assert_eq!(Q_CleanStr("a^b^8c^^d"), "a^b^8c^^d");
        // Bytes outside 0x20..=0x7E drop (here a control byte and a high byte).
        assert_eq!(Q_CleanStr("a\u{1}b\u{7f}"), "ab");
        // A trailing lone `^` is kept (no follower digit).
        assert_eq!(Q_CleanStr("hi^"), "hi^");
    }

    #[test]
    fn strlwr_stops_at_nul() {
        let mut buf = [b'A' as c_char, b'B' as c_char, 0, b'C' as c_char];
        Q_strlwr(&mut buf);
        assert_eq!(buf, [b'a' as c_char, b'b' as c_char, 0, b'C' as c_char]);
    }

    #[test]
    fn strcat_appends_after_nul() {
        let mut buf = [0 as c_char; 8];
        crate::q_strncpyz::Q_strncpyz(&mut buf, "ab", 8);
        Q_strcat(&mut buf, 8, "cd");
        assert_eq!(
            &buf[..5],
            &[
                b'a' as c_char,
                b'b' as c_char,
                b'c' as c_char,
                b'd' as c_char,
                0
            ]
        );
    }
}
