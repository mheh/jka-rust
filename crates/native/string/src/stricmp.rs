//! C `stricmp` matching the **native libc** `_stricmp` linked by the oracle
//! DLL (msvcrt, C locale) — the raw difference-returning form, distinct from
//! Raven's clamped `Q_stricmp` (-1/0/1). ICARUS `Q3_Evaluate` returns the raw
//! value to the interpreter (`oracle/codemp/icarus/Q3_Interface.cpp:595`), so
//! the magnitude is observable.

use crate::ctype::tolower_byte;

/// C `stricmp`: case-insensitive compare returning the difference of the first
/// mismatching pair of `tolower`ed bytes (`0` when equal; a missing byte reads
/// as NUL, so a proper prefix compares less).
pub fn stricmp(a: &str, b: &str) -> i32 {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let n = ab.len().max(bb.len());
    for i in 0..n {
        let x = ab.get(i).map(|&c| tolower_byte(c)).unwrap_or(0);
        let y = bb.get(i).map(|&c| tolower_byte(c)).unwrap_or(0);
        if x != y {
            return x as i32 - y as i32;
        }
    }
    0
}

#[cfg(test)]
mod stricmp_tests {
    use super::stricmp;

    #[test]
    fn is_case_insensitive() {
        assert_eq!(stricmp("Hello", "hello"), 0);
        assert_eq!(stricmp("ABC", "abc"), 0);
    }

    #[test]
    fn returns_lowercased_byte_difference() {
        assert_eq!(stricmp("abc", "abd"), b'c' as i32 - b'd' as i32);
        assert_eq!(stricmp("abd", "abc"), b'd' as i32 - b'c' as i32);
    }

    #[test]
    fn prefix_compares_less() {
        assert!(stricmp("ab", "abc") < 0);
        assert!(stricmp("abc", "ab") > 0);
    }

    #[test]
    fn bytes_above_ascii_compare_raw() {
        assert_eq!(stricmp("\u{e9}", "\u{e9}"), 0);
        assert!(stricmp("a", "\u{e9}") < 0);
    }
}
