//! Seam string helpers for the C ABI boundary — the `*const c_char`/`CString`
//! wrappers a port needs wherever it calls a `trap_*` syscall or crosses
//! `va`/`Com_sprintf` string territory. The value logic lives in
//! `native_string` (DEC-32); every fn here is a pointer-facing shape that
//! retires with the trap-wrapper `String` migration — do not add to it.

use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use native_string::atoi::atoi_bytes;

/// `atoi()` over a raw C string — [`native_string::atoi`] semantics (libc
/// `(int)strtol`: skip `isspace`, optional sign, digit prefix, i64 clamp then
/// truncating `(int)` cast).
///
/// Raven callers never pass a NULL `char*` here; libc `atoi(NULL)` is UB, so
/// the NULL case returns 0 defensively (porting-rules §19 pick).
pub fn atoi(string: *const c_char) -> c_int {
    if string.is_null() {
        return 0;
    }
    atoi_bytes(unsafe { CStr::from_ptr(string) }.to_bytes())
}

/// Own a NUL-terminated `CString` for a `trap_*` syscall argument, where
/// Raven passes a plain string literal / an owned `String` built from
/// `va`/`Com_sprintf` where the original C code took a `char*`.
///
/// Source: moved from `g_cmds.rs` (and a duplicate private copy in
/// `g_active.rs`); `oracle/codemp/game/g_cmds.c`.
#[inline]
pub fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap()
}

/// Read a NUL-terminated `*const c_char` into an owned `String` (lossy).
///
/// Source: pattern used throughout `oracle/codemp/game/` for string
/// conversion; moved from `bg_misc.rs`.
#[inline]
pub unsafe fn cstr_to_str(p: *const c_char) -> String {
    std::ffi::CStr::from_ptr(p).to_string_lossy().into_owned()
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

#[cfg(test)]
mod atoi_seam_tests {
    use super::atoi;
    use std::ffi::CString;

    // Value semantics are pinned by `native_string::atoi`'s own tests; this
    // covers only the pointer seam.
    #[test]
    fn null_returns_zero() {
        assert_eq!(atoi(std::ptr::null()), 0);
    }

    #[test]
    fn cstring_round_trip() {
        let c = CString::new(" \t-42abc").unwrap();
        assert_eq!(atoi(c.as_ptr()), -42);
    }
}
