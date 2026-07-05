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
