//! `q_shared.c` string primitives — the shared-tier home for engine-island
//! callers (`mp_game` carries its own module-island copies in `q_shared.rs`).

use core::ffi::{c_char, c_int};

use crate::shared::limits::BIG_INFO_STRING;

/// Raven `Q_strncpyz`.
///
/// Source: `oracle/codemp/game/q_shared.c:826-840`
pub fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int) {
    unsafe {
        if dest.is_null() {
            panic!("Q_strncpyz: NULL dest"); // Com_Error(ERR_FATAL, ...) -> panic (frozen Group A).
        }
        if src.is_null() {
            panic!("Q_strncpyz: NULL src"); // Com_Error(ERR_FATAL, ...) -> panic (frozen Group A).
        }
        if destsize < 1 {
            panic!("Q_strncpyz: destsize < 1"); // Com_Error(ERR_FATAL, ...) -> panic (frozen Group A).
        }

        // strncpy(dest, src, destsize-1) + trailing NUL.
        let n = (destsize - 1) as usize;
        let mut i = 0usize;
        while i < n {
            let c = *src.offset(i as isize);
            *dest.offset(i as isize) = c;
            if c == 0 {
                // strncpy pads the remainder with NULs; faithful behavior
                // (destsize-1 bytes total, all writes below are zero anyway).
                i += 1;
                while i < n {
                    *dest.offset(i as isize) = 0;
                    i += 1;
                }
                break;
            }
            i += 1;
        }
        *dest.offset(destsize as isize - 1) = 0;
    }
}

/// Raven `Q_strncmp`.
///
/// Source: `oracle/codemp/game/q_shared.c:881-898`
pub fn Q_strncmp(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    unsafe {
        let mut n = n;
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let c1 = *p1 as c_int;
            let c2 = *p2 as c_int;
            p1 = p1.offset(1);
            p2 = p2.offset(1);

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
}

// Local helpers mirroring libc, faithful to the unchecked C semantics of the
// `Info_*` bodies below (`strlen`/`strchr`/`strcmp`/`strcpy`). Transcribed with
// the `Info_*` functions from the verified `mp_game` copies
// (`crates/mp/game/src/q_shared.rs`); house rule: libc symbols use the Rust
// equivalent, no resolved signature needed.

unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut n = 0isize;
    while *s.offset(n) != 0 {
        n += 1;
    }
    n as usize
}

unsafe fn c_strchr(s: *const c_char, c: c_char) -> *mut c_char {
    let mut p = s;
    loop {
        if *p == c {
            return p as *mut c_char;
        }
        if *p == 0 {
            return std::ptr::null_mut();
        }
        p = p.offset(1);
    }
}

unsafe fn c_strcmp(a: *const c_char, b: *const c_char) -> c_int {
    let mut i: isize = 0;
    loop {
        let ca = *a.offset(i);
        let cb = *b.offset(i);
        if ca != cb {
            return (ca as c_int) - (cb as c_int);
        }
        if ca == 0 {
            return 0;
        }
        i += 1;
    }
}

unsafe fn c_strcpy(dst: *mut c_char, src: *const c_char) {
    let mut i: isize = 0;
    loop {
        let c = *src.offset(i);
        *dst.offset(i) = c;
        if c == 0 {
            break;
        }
        i += 1;
    }
}

// Raven `Com_Printf` for the `Info_*` diagnostics. At this (shared) tier there is
// no engine console; the fixed diagnostic strings go to stderr. Divergence:
// engine output routes through `Com_Printf`/`PlatformHost` (ruling 10) post-parity.
unsafe fn com_printf_lit(msg: &str) {
    eprint!("{msg}");
}

/// Raven `Info_RemoveKey_Big`.
///
/// Same shape as `Info_RemoveKey`; distinct oracle function (`BIG_INFO_*`
/// bounds), kept as its own body per porting-rules §20 (duplicate, don't
/// unify).
/// Source: `oracle/codemp/game/q_shared.c:1202-1250`
pub fn Info_RemoveKey_Big(s: *mut c_char, key: *const c_char) {
    unsafe {
        if c_strlen(s as *const c_char) >= BIG_INFO_STRING {
            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
            panic!("Info_RemoveKey_Big: oversize infostring");
        }

        if !c_strchr(key, b'\\' as c_char).is_null() {
            return;
        }

        let mut s = s;
        loop {
            let start = s;
            let mut p = start as *const c_char;
            if *p == b'\\' as c_char {
                p = p.offset(1);
            }
            let mut pkey: Vec<c_char> = Vec::new();
            loop {
                if *p == b'\\' as c_char {
                    break;
                }
                if *p == 0 {
                    return;
                }
                pkey.push(*p);
                p = p.offset(1);
            }
            pkey.push(0);
            p = p.offset(1);

            let mut value: Vec<c_char> = Vec::new();
            loop {
                if *p == b'\\' as c_char || *p == 0 {
                    break;
                }
                if *p == 0 {
                    return;
                }
                value.push(*p);
                p = p.offset(1);
            }
            value.push(0);

            if c_strcmp(key, pkey.as_ptr()) == 0 {
                c_strcpy(start, p);
                return;
            }

            if *p == 0 {
                return;
            }
            s = p as *mut c_char;
        }
    }
}

/// Raven `Info_SetValueForKey_Big`.
///
/// Same shape as `Info_SetValueForKey` against `Info_RemoveKey_Big`/
/// `BIG_INFO_STRING`; kept as its own body per porting-rules §20.
/// Source: `oracle/codemp/game/q_shared.c:1328-1366`
pub fn Info_SetValueForKey_Big(s: *mut c_char, key: *const c_char, value: *const c_char) {
    unsafe {
        if c_strlen(s as *const c_char) >= BIG_INFO_STRING {
            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
            panic!("Info_SetValueForKey: oversize infostring");
        }

        if !c_strchr(key, b'\\' as c_char).is_null() || !c_strchr(value, b'\\' as c_char).is_null()
        {
            com_printf_lit("Can't use keys or values with a \\\n");
            return;
        }

        if !c_strchr(key, b';' as c_char).is_null() || !c_strchr(value, b';' as c_char).is_null() {
            com_printf_lit("Can't use keys or values with a semicolon\n");
            return;
        }

        if !c_strchr(key, b'"' as c_char).is_null() || !c_strchr(value, b'"' as c_char).is_null() {
            com_printf_lit("Can't use keys or values with a \"\n");
            return;
        }

        Info_RemoveKey_Big(s, key);
        if value.is_null() || c_strlen(value) == 0 {
            return;
        }

        let key_s = std::ffi::CStr::from_ptr(key).to_string_lossy();
        let value_s = std::ffi::CStr::from_ptr(value).to_string_lossy();
        let newi = format!("\\{key_s}\\{value_s}");
        let s_s = std::ffi::CStr::from_ptr(s).to_string_lossy();

        if newi.len() + s_s.len() > BIG_INFO_STRING {
            com_printf_lit("BIG Info string length exceeded\n");
            return;
        }

        // strcat(s, newi) — appends newi onto the end of s (note: reversed
        // order vs. Info_SetValueForKey's strcat(newi, s)/strcpy(s, newi)).
        let full = format!("{s_s}{newi}");
        let cstr = std::ffi::CString::new(full).unwrap();
        c_strcpy(s, cstr.as_ptr());
    }
}
