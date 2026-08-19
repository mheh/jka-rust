//! Port of `oracle/codemp/game/q_shared.c`.
//!
//! `Com_Error(ERR_DROP/ERR_FATAL, ...)` call sites resolve directly to `panic!` (frozen Group A).
//! They do not route through the `crate::g_main::Com_Error` variadic stub.
//! `Com_Printf` call sites route through `crate::g_main::Com_Printf` directly, the same precedent as `bg_saberLoad.rs`.
//! Any interpolated text is pre-formatted into a single `CString`.
//! The multi-arg variadic entry point itself is not invoked.
#![allow(non_snake_case, unused, clippy::all)]

use core::ffi::CStr;

use crate::c_format::{c_vsprintf, FmtArg};
use crate::prelude::*;
use native_string::atof_bytes;
use native_string::atoi_bytes;
use native_string::latin1_to_string;
use native_string::InfoSetResult;

// The `QSharedScratch` type and the `QSharedScratch`-threaded `COM_Parse*` family are canonical in `mp_qshared` now.
// This sits below the bg tier so bg can retarget.
// This file re-exports them so its ~44 game-side importers, and the game parse helpers below
// (`COM_ParseString`/`Int`/`Float`, `COM_MatchToken`, `Parse{1,2,3}DMatrix`), keep compiling unchanged.
pub use mp_qshared::shared::com_parse::{
    COM_BeginParseSession, COM_GetCurrentParseLine, COM_Parse, COM_ParseExt, QSharedScratch,
    SkipBracedSection, SkipRestOfLine,
};

// ---------------------------------------------------------------------
// These are local helpers that mirror libc, with the unchecked C semantics used throughout this file.
// This covers `strlen`/`strchr`/`strcmp`/`tolower`/`toupper`/`atoi`.
// House rule: libc and other symbols use the Rust std equivalent, with no resolved signature needed.
// ---------------------------------------------------------------------

unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut n = 0isize;
    while *s.offset(n) != 0 {
        n += 1;
    }
    n as usize
}

/// Prints a fixed (no-interpolation) message via `Com_Printf`.
unsafe fn com_printf_lit(msg: &str) {
    crate::g_main::Com_Printf(msg);
}

// `GetIDForString` is canonical in `mp_qshared` (the shared-tier `q_string.rs`).
// It is re-exported here so this file's importers keep resolving `GetIDForString`.
pub use mp_qshared::shared::q_string::GetIDForString;

/// Raven `GetStringForID`.
///
/// Source: `oracle/codemp/game/q_shared.c:35-49`
pub fn GetStringForID(table: *mut stringID_table_t, id: c_int) -> *const c_char {
    unsafe {
        let mut index: isize = 0;
        loop {
            let entry = *table.offset(index);
            if entry.name.is_null() || *entry.name == 0 {
                break;
            }
            if entry.id == id {
                return entry.name as *const c_char;
            }
            index += 1;
        }
        std::ptr::null()
    }
}

/// Raven `COM_SkipPath`.
///
/// Source: `oracle/codemp/game/q_shared.c:80-92`
pub fn COM_SkipPath(pathname: *mut c_char) -> *mut c_char {
    unsafe {
        let mut last = pathname;
        let mut p = pathname;
        while *p != 0 {
            if *p == b'/' as c_char {
                last = p.offset(1);
            }
            p = p.offset(1);
        }
        last
    }
}

/// Raven `COM_StripExtension`.
///
/// Source: `oracle/codemp/game/q_shared.c:99-104`
pub fn COM_StripExtension(r#in: *const c_char, out: *mut c_char) {
    unsafe {
        let mut i = r#in;
        let mut o = out;
        while *i != 0 && *i != b'.' as c_char {
            *o = *i;
            o = o.offset(1);
            i = i.offset(1);
        }
        *o = 0;
    }
}

/// Raven `COM_DefaultExtension`.
///
/// Uses a fixed `"%s%s"` pattern (`oldPath`, `extension`), inlined directly rather than routed through the
/// generic `Com_sprintf` seam.
/// A statically known format with known args is a mechanical identity, not a design choice (porting-rules §A2).
/// Source: `oracle/codemp/game/q_shared.c:112-131`
pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    unsafe {
        let len = c_strlen(path);
        if len == 0 {
            // src = path - 1 with src != path is never true here.
            // This edge case is not reachable in practice, because Raven never calls this with an empty path.
            // No special case is needed beyond the loop below.
        }
        let mut src = path.offset(len as isize - 1);
        while *src != b'/' as c_char && src != path {
            if *src == b'.' as c_char {
                return; // it has an extension
            }
            src = src.offset(-1);
        }

        // Raven copies `path` into `oldPath[MAX_QPATH]` via Q_strncpyz first,
        // truncating to MAX_QPATH-1 bytes before the "%s%s" concatenation.
        let path_bytes = std::ffi::CStr::from_ptr(path).to_bytes();
        let ext_bytes = std::ffi::CStr::from_ptr(extension).to_bytes();
        let old_len = path_bytes.len().min(MAX_QPATH as usize - 1);
        let mut combined: Vec<c_char> = Vec::with_capacity(old_len + ext_bytes.len() + 1);
        combined.extend(path_bytes[..old_len].iter().map(|&b| b as c_char));
        combined.extend(ext_bytes.iter().map(|&b| b as c_char));
        combined.push(0);
        // Com_sprintf(path, maxSize, ...) truncates to maxSize-1 + NUL.
        let cap = maxSize.max(1) as usize;
        let n = combined.len().min(cap);
        std::ptr::copy_nonoverlapping(combined.as_ptr(), path, n - 1);
        *path.offset(n as isize - 1) = 0;
    }
}

/// Raven `ShortSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:162-170`
pub fn ShortSwap(l: c_short) -> c_short {
    let b1 = (l & 255) as u16;
    let b2 = ((l >> 8) & 255) as u16;
    ((b1 << 8) + b2) as c_short
}

/// Raven `ShortNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:172-175`
pub fn ShortNoSwap(l: c_short) -> c_short {
    l
}

/// Raven `LongSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:177-187`
pub fn LongSwap(l: c_int) -> c_int {
    let b1 = (l & 255) as u32;
    let b2 = ((l >> 8) & 255) as u32;
    let b3 = ((l >> 16) & 255) as u32;
    let b4 = ((l >> 24) & 255) as u32;
    ((b1 << 24) + (b2 << 16) + (b3 << 8) + b4) as c_int
}

/// Raven `LongNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:189-192`
pub fn LongNoSwap(l: c_int) -> c_int {
    l
}

/// Raven `Long64Swap`.
///
/// Source: `oracle/codemp/game/q_shared.c:194-208`
pub fn Long64Swap(ll: qint64) -> qint64 {
    qint64 {
        b0: ll.b7,
        b1: ll.b6,
        b2: ll.b5,
        b3: ll.b4,
        b4: ll.b3,
        b5: ll.b2,
        b6: ll.b1,
        b7: ll.b0,
    }
}

/// Raven `Long64NoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:210-213`
pub fn Long64NoSwap(ll: qint64) -> qint64 {
    ll
}

/// Raven `FloatSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:220-228`
pub fn FloatSwap(f: *const f32) -> f32 {
    unsafe {
        let i = (*f).to_bits() as c_int;
        f32::from_bits(LongSwap(i) as u32)
    }
}

/// Raven `FloatNoSwap`.
///
/// Source: `oracle/codemp/game/q_shared.c:230-233`
pub fn FloatNoSwap(f: *const f32) -> f32 {
    unsafe { *f }
}

/// Raven `COM_ParseError`.
///
// Varargs are not threaded, so this prints the format string as-is.
// This oracle function has zero callers in the entire MP codebase, so no call site ever supplies args to expand.
/// Source: `oracle/codemp/game/q_shared.c:300-310`
pub fn COM_ParseError(qs: &QSharedScratch, format: *mut c_char) {
    unsafe {
        let fmt_str = latin1_to_string(CStr::from_ptr(format as *const c_char).to_bytes());
        let parsename_str = latin1_to_string(cstr_from_chars(&qs.com_parsename).to_bytes());
        let com_lines = qs.com_lines;
        let msg = format!("ERROR: {}, line {}: {}", parsename_str, com_lines, fmt_str);
        crate::g_main::Com_Printf(&msg);
    }
}

/// Raven `COM_ParseWarning`.
///
// Varargs are not threaded.
// This has zero oracle callers, the same as `COM_ParseError`.
/// Source: `oracle/codemp/game/q_shared.c:312-322`
pub fn COM_ParseWarning(qs: &QSharedScratch, format: *mut c_char) {
    unsafe {
        let fmt_str = latin1_to_string(CStr::from_ptr(format as *const c_char).to_bytes());
        let parsename_str = latin1_to_string(cstr_from_chars(&qs.com_parsename).to_bytes());
        let com_lines = qs.com_lines;
        let msg = format!(
            "WARNING: {}, line {}: {}",
            parsename_str, com_lines, fmt_str
        );
        crate::g_main::Com_Printf(&msg);
    }
}

/// Raven `COM_Compress`.
///
/// Source: `oracle/codemp/game/q_shared.c:353-419`
pub fn COM_Compress(data_p: *mut c_char) -> c_int {
    unsafe {
        if data_p.is_null() {
            return 0;
        }
        let mut newline = false;
        let mut whitespace = false;
        let mut r#in = data_p;
        let mut out = data_p;

        loop {
            let c = *r#in;
            if c == 0 {
                break;
            }
            if c == b'/' as c_char && *r#in.offset(1) == b'/' as c_char {
                while *r#in != 0 && *r#in != b'\n' as c_char {
                    r#in = r#in.offset(1);
                }
            } else if c == b'/' as c_char && *r#in.offset(1) == b'*' as c_char {
                while *r#in != 0 && !(*r#in == b'*' as c_char && *r#in.offset(1) == b'/' as c_char)
                {
                    r#in = r#in.offset(1);
                }
                if *r#in != 0 {
                    r#in = r#in.offset(2);
                }
            } else if c == b'\n' as c_char || c == b'\r' as c_char {
                newline = true;
                r#in = r#in.offset(1);
            } else if c == b' ' as c_char || c == b'\t' as c_char {
                whitespace = true;
                r#in = r#in.offset(1);
            } else {
                if newline {
                    *out = b'\n' as c_char;
                    out = out.offset(1);
                    newline = false;
                    whitespace = false;
                }
                if whitespace {
                    *out = b' ' as c_char;
                    out = out.offset(1);
                    whitespace = false;
                }

                if c == b'"' as c_char {
                    *out = c;
                    out = out.offset(1);
                    r#in = r#in.offset(1);
                    loop {
                        let cc = *r#in;
                        if cc != 0 && cc != b'"' as c_char {
                            *out = cc;
                            out = out.offset(1);
                            r#in = r#in.offset(1);
                        } else {
                            break;
                        }
                    }
                    if *r#in == b'"' as c_char {
                        *out = *r#in;
                        out = out.offset(1);
                        r#in = r#in.offset(1);
                    }
                } else {
                    *out = c;
                    out = out.offset(1);
                    r#in = r#in.offset(1);
                }
            }
        }
        *out = 0;
        out.offset_from(data_p) as c_int
    }
}

/// Raven `COM_ParseString`.
///
/// Raven's guard is `if ( s[0] == 0 )` where `s` is `const char **`, so `s[0]` is the (always non-NULL) `com_token`
/// pointer, not the first token byte.
/// The EOF branch is dead, and `COM_ParseString` never returns `qtrue`.
/// Source: `oracle/codemp/game/q_shared.c:588-598`
pub fn COM_ParseString(
    qs: &mut QSharedScratch,
    data: &mut Option<&[u8]>,
    s: &mut String,
) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    *s = token;
    qfalse
}

/// Raven `COM_ParseInt`.
///
/// Source: `oracle/codemp/game/q_shared.c:605-618`
pub fn COM_ParseInt(qs: &mut QSharedScratch, data: &mut Option<&[u8]>, i: &mut c_int) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    if token.is_empty() {
        unsafe { com_printf_lit("unexpected EOF\n") };
        return qtrue;
    }
    *i = atoi_bytes(token.as_bytes());
    qfalse
}

/// Raven `COM_ParseFloat`.
///
/// Source: `oracle/codemp/game/q_shared.c:625-638`
pub fn COM_ParseFloat(qs: &mut QSharedScratch, data: &mut Option<&[u8]>, f: &mut f32) -> qboolean {
    let (token, rest) = COM_ParseExt(qs, *data, false);
    *data = rest;
    if token.is_empty() {
        unsafe { com_printf_lit("unexpected EOF\n") };
        return qtrue;
    }
    *f = atof_bytes(token.as_bytes()) as f32;
    qfalse
}

/// Raven `COM_ParseVec4`.
///
/// Source: `oracle/codemp/game/q_shared.c:645-659`
pub fn COM_ParseVec4(
    qs: &mut QSharedScratch,
    buffer: &mut Option<&[u8]>,
    c: &mut vec4_t,
) -> qboolean {
    for i in 0..4usize {
        let mut f = 0.0f32;
        if COM_ParseFloat(qs, buffer, &mut f) == qtrue {
            return qtrue;
        }
        c[i] = f;
    }
    qfalse
}

/// Raven `COM_MatchToken`.
///
/// Source: `oracle/codemp/game/q_shared.c:666-673`
pub fn COM_MatchToken(qs: &mut QSharedScratch, buf_p: &mut Option<&[u8]>, r#match: &str) {
    let (token, rest) = COM_Parse(qs, *buf_p);
    *buf_p = rest;
    if token != r#match {
        // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
        panic!("MatchToken: {token} != {}", r#match);
    }
}

/// Raven `Parse1DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:724-736`
pub fn Parse1DMatrix(qs: &mut QSharedScratch, buf_p: &mut Option<&[u8]>, x: c_int, m: *mut f32) {
    unsafe {
        COM_MatchToken(qs, buf_p, "(");
        for i in 0..x {
            let (token, rest) = COM_Parse(qs, *buf_p);
            *buf_p = rest;
            *m.offset(i as isize) = atof_bytes(token.as_bytes()) as f32;
        }
        COM_MatchToken(qs, buf_p, ")");
    }
}

/// Raven `Parse2DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:738-748`
pub fn Parse2DMatrix(
    qs: &mut QSharedScratch,
    buf_p: &mut Option<&[u8]>,
    y: c_int,
    x: c_int,
    m: *mut f32,
) {
    unsafe {
        COM_MatchToken(qs, buf_p, "(");
        for i in 0..y {
            Parse1DMatrix(qs, buf_p, x, m.offset((i * x) as isize));
        }
        COM_MatchToken(qs, buf_p, ")");
    }
}

/// Raven `Parse3DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:750-760`
pub fn Parse3DMatrix(
    qs: &mut QSharedScratch,
    buf_p: &mut Option<&[u8]>,
    z: c_int,
    y: c_int,
    x: c_int,
    m: *mut f32,
) {
    unsafe {
        COM_MatchToken(qs, buf_p, "(");
        for i in 0..z {
            Parse2DMatrix(qs, buf_p, y, x, m.offset((i * x * y) as isize));
        }
        COM_MatchToken(qs, buf_p, ")");
    }
}

/// Raven `Q_isprint`.
///
/// Source: `oracle/codemp/game/q_shared.c:771-776`
pub fn Q_isprint(c: c_int) -> c_int {
    if c >= 0x20 && c <= 0x7E {
        1
    } else {
        0
    }
}

/// Raven `Q_islower`.
///
/// Source: `oracle/codemp/game/q_shared.c:778-783`
pub fn Q_islower(c: c_int) -> c_int {
    if c >= b'a' as c_int && c <= b'z' as c_int {
        1
    } else {
        0
    }
}

/// Raven `Q_isupper`.
///
/// Source: `oracle/codemp/game/q_shared.c:785-790`
pub fn Q_isupper(c: c_int) -> c_int {
    if c >= b'A' as c_int && c <= b'Z' as c_int {
        1
    } else {
        0
    }
}

/// Raven `Q_isalpha`.
///
/// Source: `oracle/codemp/game/q_shared.c:792-797`
pub fn Q_isalpha(c: c_int) -> c_int {
    if (c >= b'a' as c_int && c <= b'z' as c_int) || (c >= b'A' as c_int && c <= b'Z' as c_int) {
        1
    } else {
        0
    }
}

/// Raven `Q_strrchr`.
///
/// Source: `oracle/codemp/game/q_shared.c:799-817`
pub fn Q_strrchr(string: *const c_char, c: c_int) -> *mut c_char {
    unsafe {
        let cc = c as c_char;
        let mut s = string;
        let mut sp: *mut c_char = std::ptr::null_mut();
        while *s != 0 {
            if *s == cc {
                sp = s as *mut c_char;
            }
            s = s.offset(1);
        }
        if cc == 0 {
            sp = s as *mut c_char;
        }
        sp
    }
}

/// C standard-library `strcmp` (case-sensitive), as called bare (not through a `Q_*` wrapper) at various `q_shared.c`
/// sites, for example lines 548, 565, 670, 1185, 1240.
/// This is housed alongside the other `q_shared.c` string helpers, per the file's existing string-fn family.
///
/// Source: `oracle/codemp/game/q_shared.c` (bare `strcmp` call sites).
pub fn Q_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    unsafe {
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let c1 = *p1 as c_int;
            let c2 = *p2 as c_int;
            p1 = p1.offset(1);
            p2 = p2.offset(1);

            if c1 != c2 {
                return if c1 < c2 { -1 } else { 1 };
            }
            if c1 == 0 {
                return 0;
            }
        }
    }
}

/// C standard-library `strchr` (first-occurrence character search), as called bare (not through a `Q_*` wrapper) at
/// various `q_shared.c` sites, for example lines 1157, 1212, 1264, 1267, 1287, 1293, 1299, 1335, 1341, 1347.
/// This is housed alongside the other `q_shared.c` string helpers, per the file's existing string-fn family.
///
/// Source: `oracle/codemp/game/q_shared.c` (bare `strchr` call sites).
pub fn Q_strchr(string: *const c_char, c: c_int) -> *mut c_char {
    unsafe {
        let cc = c as c_char;
        let mut s = string;
        loop {
            if *s == cc {
                return s as *mut c_char;
            }
            if *s == 0 {
                return std::ptr::null_mut();
            }
            s = s.offset(1);
        }
    }
}

/// Raven bare `strstr` call sites, for example `g_client.c` GLA-name matching.
///
/// This is a house-authored wrapper, not a named Raven `Q_` fn.
/// It mirrors `Q_strchr` above, ported to this canonical string-fn home since call sites needed a C-string `strstr`
/// and none existed yet.
///
/// Source: `oracle/codemp/game/g_client.c` (bare `strstr` call sites).
pub fn Q_strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    unsafe {
        if *needle == 0 {
            return haystack as *mut c_char;
        }
        let mut h = haystack;
        loop {
            if *h == 0 {
                return std::ptr::null_mut();
            }
            let mut hh = h;
            let mut nn = needle;
            while *hh != 0 && *nn != 0 && *hh == *nn {
                hh = hh.offset(1);
                nn = nn.offset(1);
            }
            if *nn == 0 {
                return h as *mut c_char;
            }
            h = h.offset(1);
        }
    }
}

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
                // `strncpy` pads the remainder with NULs.
                // This is `destsize-1` bytes total, and all writes below are zero anyway.
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

/// Raven `Q_stricmpn`.
///
/// Source: `oracle/codemp/game/q_shared.c:842-879`
pub fn Q_stricmpn(s1: *const c_char, s2: *const c_char, n: c_int) -> c_int {
    unsafe {
        if s1.is_null() {
            return if s2.is_null() { 0 } else { -1 };
        } else if s2.is_null() {
            return 1;
        }

        let mut n = n;
        let mut p1 = s1;
        let mut p2 = s2;
        loop {
            let mut c1 = *p1 as c_int;
            let mut c2 = *p2 as c_int;
            p1 = p1.offset(1);
            p2 = p2.offset(1);

            if n == 0 {
                return 0;
            }
            n -= 1;

            if c1 != c2 {
                if c1 >= b'a' as c_int && c1 <= b'z' as c_int {
                    c1 -= b'a' as c_int - b'A' as c_int;
                }
                if c2 >= b'a' as c_int && c2 <= b'z' as c_int {
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

/// C standard-library `strlen`, as called bare (not a Raven `Q_*` wrapper) at
/// the `NPC_VehiclePrecache` GLA-name/animation.cfg path-splice site. Housed
/// alongside the other `q_shared.c` string helpers per the file's existing
/// string-fn family.
///
/// Source: `oracle/codemp/game/NPC_spawn.c` (`NPC_VehiclePrecache`,
/// literal `strlen("/animation.cfg")` call).
pub fn Q_strlen(string: *const c_char) -> usize {
    unsafe { std::ffi::CStr::from_ptr(string).to_bytes().len() }
}

/// Raven `Q_stricmp`.
///
/// Source: `oracle/codemp/game/q_shared.c:900-902`
pub fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int {
    if !s1.is_null() && !s2.is_null() {
        Q_stricmpn(s1, s2, 99999)
    } else {
        -1
    }
}

/// Raven `Q_strlwr`.
///
/// Source: `oracle/codemp/game/q_shared.c:905-914`
pub fn Q_strlwr(s1: *mut c_char) -> *mut c_char {
    unsafe {
        let mut s = s1;
        while *s != 0 {
            *s = (*s as u8 as char).to_ascii_lowercase() as c_char;
            s = s.offset(1);
        }
        s1
    }
}

/// Raven `Q_strupr`.
///
/// Source: `oracle/codemp/game/q_shared.c:916-925`
pub fn Q_strupr(s1: *mut c_char) -> *mut c_char {
    unsafe {
        let mut s = s1;
        while *s != 0 {
            *s = (*s as u8 as char).to_ascii_uppercase() as c_char;
            s = s.offset(1);
        }
        s1
    }
}

/// Raven `Q_strcat`.
///
/// Source: `oracle/codemp/game/q_shared.c:929-937`
pub fn Q_strcat(dest: *mut c_char, size: c_int, src: *const c_char) {
    unsafe {
        let l1 = c_strlen(dest) as c_int;
        if l1 >= size {
            panic!("Q_strcat: already overflowed"); // Com_Error(ERR_FATAL, ...) -> panic (frozen Group A).
        }
        Q_strncpyz(dest.offset(l1 as isize), src, size - l1);
    }
}

/// Raven `Q_PrintStrlen`.
///
/// Note: Raven's `Q_IsColorString` is a `q_shared.h` inline predicate.
/// No resolved cross-file signature was provided for this file (0 header-inline helpers listed).
/// So the color-escape skip (`^` + digit, 2-byte stride) is inlined directly here rather than invoking an unresolved
/// symbol.
/// Source: `oracle/codemp/game/q_shared.c:940-960`
pub fn Q_PrintStrlen(string: *const c_char) -> c_int {
    unsafe {
        if string.is_null() {
            return 0;
        }
        let mut len: c_int = 0;
        let mut p = string;
        while *p != 0 {
            // `Q_IsColorString(p)` equals `^` followed by a digit '0'..='7' and not `^`.
            // The port previously accepted any non-NUL follower, which over-counted `^^`/`^8...` escapes.
            // The oracle slice caught this.
            let n = *p.offset(1);
            if *p == b'^' as c_char
                && n != 0
                && n != b'^' as c_char
                && n >= b'0' as c_char
                && n <= b'7' as c_char
            {
                p = p.offset(2);
                continue;
            }
            p = p.offset(1);
            len += 1;
        }
        len
    }
}

/// Raven `Q_CleanStr`.
///
/// Source: `oracle/codemp/game/q_shared.c:963-982`
pub fn Q_CleanStr(string: *mut c_char) -> *mut c_char {
    unsafe {
        let mut s = string as *const c_char;
        let mut d = string;
        loop {
            let c = *s;
            if c == 0 {
                break;
            }
            // `Q_IsColorString(s)` equals `^` followed by a digit '0'..='7', see `Q_PrintStrlen`.
            // The port previously skipped on any non-NUL follower, wrongly stripping `^^`/`^8...`.
            // The slice caught this.
            let n = *s.offset(1);
            if c == b'^' as c_char
                && n != 0
                && n != b'^' as c_char
                && n >= b'0' as c_char
                && n <= b'7' as c_char
            {
                s = s.offset(1);
            } else if c >= 0x20 && c <= 0x7E {
                *d = c;
                d = d.offset(1);
            }
            s = s.offset(1);
        }
        *d = 0;
        string
    }
}

/// Raven `Com_sprintf`.
///
/// Raven's `...` is an explicit `&[FmtArg]` channel formatted by `c_format::c_vsprintf`.
/// This reproduces the `ERR_FATAL` bigbuffer overflow exactly as a panic (frozen Group A).
/// Source: `oracle/codemp/game/q_shared.c:985-1005`
pub fn Com_sprintf(dest: *mut c_char, size: c_int, fmt: *const c_char, args: &[FmtArg]) {
    unsafe {
        let fmt_bytes = std::ffi::CStr::from_ptr(fmt).to_bytes();
        let bigbuffer = c_vsprintf(fmt_bytes, args);
        let len = bigbuffer.len();
        if len >= 32000 {
            // Com_Error(ERR_FATAL, "Com_sprintf: overflowed bigbuffer") -> panic.
            panic!("Com_sprintf: overflowed bigbuffer");
        }
        if len as c_int >= size {
            let msg = format!("Com_sprintf: overflow of {} in {}\n", len, size);
            crate::g_main::Com_Printf(&msg);
        }
        // `Q_strncpyz` needs a NUL-terminated source.
        // `bigbuffer` has no interior NUL, because the formatter never emits one, so this appends the terminator here.
        let mut cbig = bigbuffer;
        cbig.push(0);
        Q_strncpyz(dest, cbig.as_ptr() as *const c_char, size);
    }
}

/// Raven `va`.
///
/// Raven's `...` is an explicit `&[FmtArg]` channel (`c_format::c_vsprintf`).
/// The 2-slot rotating buffer is reproduced.
/// Raven overruns past 32000 bytes, its own FIXME, and the port truncates instead (§19).
/// Source: `oracle/codemp/game/q_shared.c:1017-1031`
pub fn va(qs: &mut QSharedScratch, format: *const c_char, args: &[FmtArg]) -> *mut c_char {
    unsafe {
        let buf = qs.va_string[qs.va_index & 1].as_mut_ptr();
        qs.va_index += 1;

        let fmt_bytes = std::ffi::CStr::from_ptr(format).to_bytes();
        let formatted = c_vsprintf(fmt_bytes, args);
        let copy_len = formatted.len().min(32000 - 1);
        std::ptr::copy_nonoverlapping(formatted.as_ptr() as *const c_char, buf, copy_len);
        *buf.offset(copy_len as isize) = 0;

        buf
    }
}

/// Raven `Info_SetValueForKey`.
///
/// The value logic lives in [`native_string::info`].
/// This shim reproduces the `Com_Printf` a rejected set emits.
///
/// Source: `oracle/codemp/game/q_shared.c:1280-1319`
pub fn Info_SetValueForKey(s: &mut String, key: &str, value: &str) {
    let result = native_string::info::Info_SetValueForKey(s, key, value);
    print_info_set_result(result, "Info string length exceeded\n");
}

/// Raven `Info_SetValueForKey_Big`.
///
/// The value logic lives in [`native_string::info`], and it appends where the non-Big form prepends.
/// The `Com_Printf` shim is the same as above.
///
/// Source: `oracle/codemp/game/q_shared.c:1328-1366`
pub fn Info_SetValueForKey_Big(s: &mut String, key: &str, value: &str) {
    let result = native_string::info::Info_SetValueForKey_Big(s, key, value);
    print_info_set_result(result, "BIG Info string length exceeded\n");
}

/// These are the `Com_Printf` messages Raven prints on a rejected `Info_SetValueForKey`.
/// Only the length-exceeded text differs between the Big and non-Big forms.
fn print_info_set_result(result: InfoSetResult, exceeded_msg: &str) {
    unsafe {
        match result {
            InfoSetResult::Set => {}
            InfoSetResult::ContainsBackslash => {
                com_printf_lit("Can't use keys or values with a \\\n");
            }
            InfoSetResult::ContainsSemicolon => {
                com_printf_lit("Can't use keys or values with a semicolon\n");
            }
            InfoSetResult::ContainsQuote => {
                com_printf_lit("Can't use keys or values with a \"\n");
            }
            InfoSetResult::LengthExceeded => com_printf_lit(exceeded_msg),
        }
    }
}
