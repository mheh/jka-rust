//! Seam string helpers for the C ABI boundary.
//!
//! These are internal utilities, not ported Raven items — they exist purely
//! to bridge the `*const c_char`/`CString` seam wherever a jampgame port
//! calls a `trap_*` syscall or crosses `va`/`Com_sprintf` string territory
//! (bless-the-rule appendix: "va/Com_sprintf → owned String"). Consolidated
//! here from three duplicated copies (`g_cmds.rs`, a private copy in
//! `g_active.rs`, and `bg_misc.rs`) so pass-3 porters have one canonical
//! import path. The pass-3 packet primer (`tools/closure-prototype/packets3.py`)
//! cites this module by path for its va/printf mapping table.

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

/// `sscanf(..., "%f", ...)` scanner matching the **native libc** `sscanf`
/// linked by the oracle DLL (`nm` shows `_sscanf` U) — not the QVM-only
/// bytecode `sscanf`/`_atof` in `oracle/oracle/codemp/game/bg_lib.c`, which
/// is never linked into the game DLL build. One shared implementation for
/// the ~12 `sscanf(s, "%f %f %f", ...)`-shaped call sites across
/// `g_spawn.rs`/`bg_vehicleLoad.rs`/`bg_misc.rs`/`g_ICARUScb.rs`.
///
/// For each of `outs`, skips libc `isspace` whitespace (`' ' '\t' '\n' \x0b
/// \x0c '\r'`), then parses the longest valid float prefix — optional sign,
/// digits, optional `.` + digits, optional exponent (`e`/`E`, optional sign,
/// >=1 digit; an `e` not followed by a valid exponent is not consumed) — and
/// converts it with `str::parse::<f32>` (correctly rounded, same class as
/// `strtod`). Stops at the first directive with no valid prefix and returns
/// the count of directives matched so far, mirroring libc `sscanf`'s
/// stop-at-first-failed-conversion behavior (a failed token is never skipped
/// so later values never shift into earlier slots, unlike a naive
/// `split_whitespace().filter_map(...)`).
///
/// Unmatched `outs` slots are left UNTOUCHED. Oracle note (porting-rules
/// §19): the C callers' destinations are stack locals or struct fields the
/// caller passes uninitialized/pre-zeroed; unmatched components are then
/// genuine stack garbage (UB) in Raven. The one defined behavior picked here
/// is "leave the Rust-side destination unmodified".
///
/// C99 hex-float (`0x1.8p3`) and `inf`/`nan` prefixes are not handled — no
/// oracle input path (`.veh`/`.vwp` text, ICARUS scripts, spawn-var strings)
/// emits them.
pub fn sscanf_f32s(text: &str, outs: &mut [f32]) -> usize {
    let bytes = text.as_bytes();
    let mut pos = 0usize;
    let mut count = 0usize;
    for out in outs.iter_mut() {
        while pos < bytes.len() && is_libc_isspace(bytes[pos]) {
            pos += 1;
        }
        let start = pos;
        let mut p = pos;
        if p < bytes.len() && (bytes[p] == b'+' || bytes[p] == b'-') {
            p += 1;
        }
        let mut has_digits = false;
        while p < bytes.len() && bytes[p].is_ascii_digit() {
            p += 1;
            has_digits = true;
        }
        if p < bytes.len() && bytes[p] == b'.' {
            p += 1;
            while p < bytes.len() && bytes[p].is_ascii_digit() {
                p += 1;
                has_digits = true;
            }
        }
        if !has_digits {
            break;
        }
        // Optional exponent — only consumed if followed by >=1 digit.
        let mut end = p;
        if p < bytes.len() && (bytes[p] == b'e' || bytes[p] == b'E') {
            let mut q = p + 1;
            if q < bytes.len() && (bytes[q] == b'+' || bytes[q] == b'-') {
                q += 1;
            }
            let exp_digits_start = q;
            while q < bytes.len() && bytes[q].is_ascii_digit() {
                q += 1;
            }
            if q > exp_digits_start {
                end = q;
            }
        }
        // The matched prefix is pure ASCII (sign/digit/'.'/'e'/'E'), so these
        // byte offsets are always valid `str` char boundaries even if `text`
        // contains multi-byte UTF-8 elsewhere (e.g. from `to_string_lossy`).
        let token = std::str::from_utf8(&bytes[start..end]).unwrap();
        match token.parse::<f32>() {
            Ok(v) => {
                *out = v;
                pos = end;
                count += 1;
            }
            Err(_) => break,
        }
    }
    count
}

fn is_libc_isspace(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

#[cfg(test)]
mod sscanf_f32s_tests {
    use super::sscanf_f32s;

    const SENTINEL: f32 = -12345.0;

    #[test]
    fn well_formed() {
        let mut outs = [SENTINEL; 3];
        let n = sscanf_f32s("1 2 3", &mut outs);
        assert_eq!(n, 3);
        assert_eq!(outs, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn stop_at_first_failure() {
        let mut outs = [SENTINEL; 3];
        let n = sscanf_f32s("abc 5 6", &mut outs);
        assert_eq!(n, 0);
        assert_eq!(outs, [SENTINEL; 3]);
    }

    #[test]
    fn prefix_parse_then_next_directive_fails() {
        let mut outs = [SENTINEL; 2];
        let n = sscanf_f32s("12.5abc 3 4", &mut outs);
        assert_eq!(n, 1);
        assert_eq!(outs[0], 12.5);
        assert_eq!(outs[1], SENTINEL);
    }

    #[test]
    fn exponent_accepted() {
        let mut outs = [SENTINEL; 2];
        let n = sscanf_f32s("1e2 5", &mut outs);
        assert_eq!(n, 2);
        assert_eq!(outs, [100.0, 5.0]);
    }

    #[test]
    fn trailing_e_without_exponent_digits_not_consumed() {
        // "1e" — the 'e' is not followed by a valid exponent, so only "1" is
        // consumed and the next directive starts right at 'e'.
        let mut outs = [SENTINEL; 2];
        let n = sscanf_f32s("1e", &mut outs);
        assert_eq!(n, 1);
        assert_eq!(outs[0], 1.0);
        assert_eq!(outs[1], SENTINEL);
    }
}

/// `atoi()` matching the **native libc** `atoi` linked by the oracle DLL
/// (`nm` shows `_atoi` U) — not `bg_lib::atoi`, which is the QVM-only
/// bytecode port of `oracle/oracle/codemp/game/bg_lib.c:914-1318`
/// (`#if defined(Q3_VM)`), never compiled into the native game DLL build.
/// macOS libc `atoi` is `(int)strtol(s, NULL, 10)`: skip libc `isspace`
/// whitespace, optional single sign, decimal-digit prefix (stopping at the
/// first non-digit — `"12abc"` -> 12, `""`/`"abc"` -> 0), with the `strtol`
/// accumulation done in `long` (i64 here) and clamped to `i64::MAX`/`MIN` on
/// overflow before the final truncating `(int)` cast — this differs from
/// `bg_lib::atoi`'s whitespace class (`<= ' '`, so it skips all C0 controls,
/// not just libc's six) and its overflow behavior (wraps in `i32` rather
/// than clamping in `i64` first).
///
/// Raven callers never pass a NULL `char*` here; libc `atoi(NULL)` is UB, so
/// the NULL case returns 0 defensively (porting-rules §19 pick).
pub fn atoi(string: *const c_char) -> c_int {
    if string.is_null() {
        return 0;
    }
    atoi_bytes(unsafe { CStr::from_ptr(string) }.to_bytes())
}

/// `atoi_str()` — same libc semantics as [`atoi`] over an owned `&str`, for
/// call sites that already hold a Rust string rather than a raw C pointer.
pub fn atoi_str(text: &str) -> c_int {
    atoi_bytes(text.as_bytes())
}

fn atoi_bytes(bytes: &[u8]) -> c_int {
    let mut i = 0usize;
    while i < bytes.len() && is_libc_isspace(bytes[i]) {
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
    use super::{atoi, atoi_str};
    use std::ffi::CString;

    fn atoi_c(s: &str) -> i32 {
        let c = CString::new(s).unwrap();
        atoi(c.as_ptr())
    }

    #[test]
    fn trailing_garbage_stops_at_first_non_digit() {
        assert_eq!(atoi_c("12abc"), 12);
        assert_eq!(atoi_str("12abc"), 12);
    }

    #[test]
    fn empty_and_non_numeric() {
        assert_eq!(atoi_c(""), 0);
        assert_eq!(atoi_c("abc"), 0);
    }

    #[test]
    fn leading_whitespace_and_sign() {
        assert_eq!(atoi_c(" \t-42"), -42);
    }

    #[test]
    fn vertical_tab_is_libc_isspace() {
        assert_eq!(atoi_c("\x0b5"), 5);
    }

    #[test]
    fn leading_plus() {
        assert_eq!(atoi_c("+7"), 7);
    }

    #[test]
    fn overflow_clamps_in_i64_then_truncates() {
        assert_eq!(atoi_c("2147483648"), -2147483648);
        assert_eq!(atoi_c("-2147483649"), 2147483647);
        assert_eq!(atoi_c("99999999999999999999"), -1);
        assert_eq!(atoi_c("-99999999999999999999"), 0);
    }
}

/// Own a NUL-terminated `CString` for a `trap_*` syscall argument, where
/// Raven passes a plain string literal / an owned `String` built from
/// `va`/`Com_sprintf` where the original C code took a `char*`.
///
/// Source: moved from `g_cmds.rs` (and a duplicate private copy in
/// `g_active.rs`); `oracle/oracle/codemp/game/g_cmds.c`.
#[inline]
pub fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap()
}

/// Read a NUL-terminated `*const c_char` into an owned `String` (lossy).
///
/// Source: pattern used throughout `oracle/oracle/codemp/game/` for string
/// conversion; moved from `bg_misc.rs`.
#[inline]
pub unsafe fn cstr_to_str(p: *const c_char) -> String {
    std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
}

/// Alias of [`cstr_to_str`], kept for call sites written against the
/// original private `bg_misc.rs` name.
#[inline]
pub unsafe fn cstr_to_string(p: *const c_char) -> String {
    cstr_to_str(p)
}

/// Write `src` into the caller's fixed C buffer `dest`, NUL-terminated and
/// truncated to fit — mirrors `Com_sprintf`'s truncate-and-terminate
/// contract for a `char[N]` struct field. No-op if `dest` is empty (no room
/// for the terminator).
pub fn write_cstr_field(dest: &mut [c_char], src: &str) {
    if dest.is_empty() {
        return;
    }
    let bytes = src.as_bytes();
    let n = bytes.len().min(dest.len() - 1);
    for (i, &b) in bytes[..n].iter().enumerate() {
        dest[i] = b as c_char;
    }
    dest[n] = 0;
}
