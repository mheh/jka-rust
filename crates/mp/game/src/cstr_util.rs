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

use core::ffi::c_char;

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
