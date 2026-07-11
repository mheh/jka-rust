//! `q_shared.c` string primitives — the shared-tier home for engine-island
//! callers (`mp_game` carries its own module-island copies in `q_shared.rs`).

use core::ffi::{c_char, c_int};

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
