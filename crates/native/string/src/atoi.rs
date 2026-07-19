//! `atoi()` matching the **native libc** `atoi` linked by the oracle DLL
//! (`nm` shows `_atoi` U) — not `bg_lib::atoi`, which is the QVM-only
//! bytecode port of `oracle/codemp/game/bg_lib.c:914-1318`
//! (`#if defined(Q3_VM)`), never compiled into the native game DLL build.

use crate::ctype::isspace_byte;

/// macOS libc `atoi` is `(int)strtol(s, NULL, 10)`: skip libc `isspace`
/// whitespace, optional single sign, decimal-digit prefix (stopping at the
/// first non-digit — `"12abc"` -> 12, `""`/`"abc"` -> 0), with the `strtol`
/// accumulation done in `long` (i64 here) and clamped to `i64::MAX`/`MIN` on
/// overflow before the final truncating `(int)` cast — this differs from
/// `bg_lib::atoi`'s whitespace class (`<= ' '`, so it skips all C0 controls,
/// not just libc's six) and its overflow behavior (wraps in `i32` rather
/// than clamping in `i64` first).
pub fn atoi(text: &str) -> i32 {
    atoi_bytes(text.as_bytes())
}

/// [`atoi`] over raw bytes, for seam callers that hold unconverted C data.
pub fn atoi_bytes(bytes: &[u8]) -> i32 {
    let mut i = 0usize;
    while i < bytes.len() && isspace_byte(bytes[i]) {
        i += 1;
    }
    let neg = match bytes.get(i) {
        Some(b'+') => {
            i += 1;
            false
        }
        Some(b'-') => {
            i += 1;
            true
        }
        _ => false,
    };

    let mut acc: i64 = 0;
    let mut overflowed = false;
    let mut has_digits = false;
    while let Some(&b) = bytes.get(i) {
        if !b.is_ascii_digit() {
            break;
        }
        has_digits = true;
        let d = (b - b'0') as i64;
        if !overflowed {
            match acc.checked_mul(10).and_then(|v| v.checked_add(d)) {
                Some(v) => acc = v,
                // strtol clamps the instant accumulation would overflow the
                // `long` (i64) accumulator, then keeps consuming the
                // remaining digit characters without updating the value.
                None => overflowed = true,
            }
        }
        i += 1;
    }
    if !has_digits {
        return 0;
    }

    let magnitude = if overflowed { i64::MAX } else { acc };
    let value: i64 = if neg {
        if overflowed {
            i64::MIN
        } else {
            -magnitude
        }
    } else {
        magnitude
    };
    // Bit-truncating cast, matching C's `(int)` on the `long` strtol result.
    value as i32
}

#[cfg(test)]
mod atoi_tests {
    use super::atoi;

    #[test]
    fn trailing_garbage_stops_at_first_non_digit() {
        assert_eq!(atoi("12abc"), 12);
    }

    #[test]
    fn empty_and_non_numeric() {
        assert_eq!(atoi(""), 0);
        assert_eq!(atoi("abc"), 0);
    }

    #[test]
    fn leading_whitespace_and_sign() {
        assert_eq!(atoi(" \t-42"), -42);
    }

    #[test]
    fn vertical_tab_is_libc_isspace() {
        assert_eq!(atoi("\x0b5"), 5);
    }

    #[test]
    fn leading_plus() {
        assert_eq!(atoi("+7"), 7);
    }

    #[test]
    fn overflow_clamps_in_i64_then_truncates() {
        assert_eq!(atoi("2147483648"), -2147483648);
        assert_eq!(atoi("-2147483649"), 2147483647);
        assert_eq!(atoi("99999999999999999999"), -1);
        assert_eq!(atoi("-99999999999999999999"), 0);
    }
}
