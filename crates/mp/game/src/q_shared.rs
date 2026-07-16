// PORT-COMPLETE: q_shared.c 46/10
//! FAITHFUL port of `oracle/codemp/game/q_shared.c`.
//!
//! Filled by the jampgame mega-pass.
//!
//! `Com_Error(ERR_DROP/ERR_FATAL, ...)` call sites resolve directly to
//! `panic!` (frozen Group A: "Com_Error→panic + EntityId + GameContext"),
//! matching the bless-appendix ruling in the file packet — they do not
//! route through the still-parked `crate::g_main::Com_Error` variadic
//! stub. `Com_Printf` call sites route through `crate::g_main::Com_Printf`
//! directly (same precedent as `bg_saberLoad.rs`), with any interpolated
//! text pre-formatted into a single `CString` (the multi-arg variadic
//! entry point itself is not invoked).
#![allow(non_snake_case, unused, clippy::all)]

use crate::c_format::{c_vsprintf, FmtArg};
use crate::prelude::*;
use mp_qshared::shared::{BIG_INFO_STRING, MAX_INFO_STRING};

// Parse-session state (cross-frame state -> GameWorld fields, pending full threading).
/// Raven's `q_shared.c` file-static parse/format state, moved into
/// `BgState.qs` (safe-state Stage 3 — no `static mut`, rule B3). Rotation
/// index semantics (`va`/`Info_ValueForKey` two-slot rings) preserved.
/// Source: `oracle/codemp/game/q_shared.c` file statics.
pub struct QSharedScratch {
    /// Raven `static int com_lines`.
    pub com_lines: c_int,
    /// Raven `static char com_parsename[MAX_TOKEN_CHARS]`.
    pub com_parsename: [c_char; 1024],
    /// Raven `static char com_token[MAX_TOKEN_CHARS]` — `COM_Parse*` return
    /// pointers point into this buffer.
    pub com_token: [c_char; 1024],
    /// Raven `static char string[2][32000]` (va's rotating pair) + index.
    pub va_string: Box<[[c_char; 32000]; 2]>,
    pub va_index: usize,
    /// Raven `Info_ValueForKey`'s rotating pair + index.
    pub info_value: Box<[[c_char; 8192]; 2]>,
    pub info_valueindex: c_int,
}

impl QSharedScratch {
    pub fn zeroed() -> Self {
        Self {
            com_lines: 0,
            com_parsename: [0; 1024],
            com_token: [0; 1024],
            va_string: Box::new([[0; 32000]; 2]),
            va_index: 0,
            info_value: Box::new([[0; 8192]; 2]),
            info_valueindex: 0,
        }
    }
}

/// Raven `FOFS(targetname)` — `#define FOFS(x) ((int)&(((gentity_t *)0)->x))`,
/// specialized to the `targetname` field for `G_Find` call sites.
///
/// Source: `oracle/codemp/game/g_local.h:1511`
pub const FOFS_targetname: c_int = core::mem::offset_of!(gentity_t, targetname) as c_int;

// ---------------------------------------------------------------------
// Local helpers mirroring libc, faithful to the unchecked C semantics used
// throughout this file (`strlen`/`strchr`/`strcmp`/`tolower`/`toupper`/
// `atoi` — house rule: libc/other symbols use the Rust std equivalent, no
// resolved signature needed).
// ---------------------------------------------------------------------

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

/// Prints a fixed (no-interpolation) message via `Com_Printf`.
unsafe fn com_printf_lit(msg: &str) {
    let c = std::ffi::CString::new(msg).unwrap();
    crate::g_main::Com_Printf(c.as_ptr());
}

/// Raven `GetIDForString`.
///
/// Source: `oracle/codemp/game/q_shared.c:13-27`
pub fn GetIDForString(table: *mut stringID_table_t, string: *const c_char) -> c_int {
    unsafe {
        let mut index: isize = 0;
        loop {
            let entry = *table.offset(index);
            if entry.name.is_null() || *entry.name == 0 {
                break;
            }
            if crate::q_shared::Q_stricmp(entry.name as *const c_char, string) == 0 {
                return entry.id;
            }
            index += 1;
        }
        -1
    }
}

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

/// Raven `Com_Clampi`.
///
/// Source: `oracle/codemp/game/q_shared.c:51-62`
pub fn Com_Clampi(min: c_int, max: c_int, value: c_int) -> c_int {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

/// Raven `Com_Clamp`.
///
/// Source: `oracle/codemp/game/q_shared.c:64-72`
pub fn Com_Clamp(min: f32, max: f32, value: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
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
/// Uses a fixed `"%s%s"` pattern (`oldPath`, `extension`) — inlined directly
/// rather than routed through the still-parked generic `Com_sprintf` seam
/// (porting-rules §A2: no invented behavior, but a statically-known format
/// with known args is a mechanical identity, not a design choice).
/// Source: `oracle/codemp/game/q_shared.c:112-131`
pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    unsafe {
        let len = c_strlen(path);
        if len == 0 {
            // src = path - 1 with src != path never true; faithful edge case
            // not reachable in practice (Raven never calls this with an
            // empty path). No special-case needed beyond the loop below.
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

/// Raven `COM_BeginParseSession`.
///
/// PORT-NOTE(variadic-c-abi): Raven calls Com_sprintf with "%s" format; since Com_sprintf
/// cannot accept varargs in Rust, this implementation directly copies the name via Q_strncpyz.
/// Source: `oracle/codemp/game/q_shared.c:284-288`
pub fn COM_BeginParseSession(qs: &mut QSharedScratch, name: *const c_char) {
    unsafe {
        qs.com_lines = 0;
        crate::q_shared::Q_strncpyz(
            qs.com_parsename.as_mut_ptr(),
            name,
            MAX_TOKEN_CHARS as c_int,
        );
    }
}

/// Raven `COM_GetCurrentParseLine`.
///
/// Source: `oracle/codemp/game/q_shared.c:290-293`
pub fn COM_GetCurrentParseLine(qs: &QSharedScratch) -> c_int {
    qs.com_lines
}

/// Raven `COM_Parse`.
///
/// Source: `oracle/codemp/game/q_shared.c:295-298`
pub fn COM_Parse(qs: &mut QSharedScratch, data_p: *mut *const c_char) -> *mut c_char {
    crate::q_shared::COM_ParseExt(qs, data_p, qtrue)
}

/// Raven `COM_ParseError`.
///
/// PORT-NOTE(variadic-c-abi): Rust cannot express C varargs without external C FFI or macros.
/// The Raven implementation uses va_start/va_end/vsprintf. This implementation formats
/// the available data (format string as placeholder) via Com_Printf; true format-arg expansion
/// requires a seam decision (vsprintf FFI wrapper or pre-formatted String caller convention).
/// Source: `oracle/codemp/game/q_shared.c:300-310`
pub fn COM_ParseError(qs: &QSharedScratch, format: *mut c_char) {
    unsafe {
        let fmt_str = std::ffi::CStr::from_ptr(format as *const c_char).to_string_lossy();
        let parsename_str = cstr_from_chars(&qs.com_parsename).to_string_lossy();
        let com_lines = qs.com_lines;
        let msg = format!("ERROR: {}, line {}: {}", parsename_str, com_lines, fmt_str);
        let c_msg = std::ffi::CString::new(msg).unwrap();
        crate::g_main::Com_Printf(c_msg.as_ptr());
    }
}

/// Raven `COM_ParseWarning`.
///
/// PORT-NOTE(variadic-c-abi): same as COM_ParseError — Rust cannot express varargs without
/// external C FFI. The format string and parse-session globals are available; actual arg
/// formatting requires a seam decision.
/// Source: `oracle/codemp/game/q_shared.c:312-322`
pub fn COM_ParseWarning(qs: &QSharedScratch, format: *mut c_char) {
    unsafe {
        let fmt_str = std::ffi::CStr::from_ptr(format as *const c_char).to_string_lossy();
        let parsename_str = cstr_from_chars(&qs.com_parsename).to_string_lossy();
        let com_lines = qs.com_lines;
        let msg = format!(
            "WARNING: {}, line {}: {}",
            parsename_str, com_lines, fmt_str
        );
        let c_msg = std::ffi::CString::new(msg).unwrap();
        crate::g_main::Com_Printf(c_msg.as_ptr());
    }
}

/// Raven `SkipWhitespace`.
///
/// Source: `oracle/codemp/game/q_shared.c:336-351`
pub fn SkipWhitespace(
    qs: &mut QSharedScratch,
    data: *const c_char,
    hasNewLines: *mut qboolean,
) -> *const c_char {
    unsafe {
        let mut c: c_int;
        let mut p = data;

        loop {
            c = *p as c_int;
            if c > b' ' as c_int {
                break;
            }
            if c == 0 {
                return std::ptr::null();
            }
            if c == b'\n' as c_int {
                qs.com_lines += 1;
                *hasNewLines = qtrue;
            }
            p = p.offset(1);
        }

        p
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

/// Raven `COM_ParseExt`.
///
/// Source: `oracle/codemp/game/q_shared.c:421-526`
pub fn COM_ParseExt(
    qs: &mut QSharedScratch,
    data_p: *mut *const c_char,
    allowLineBreaks: qboolean,
) -> *mut c_char {
    unsafe {
        let mut c: c_int = 0;
        let mut len: c_int;
        let mut hasNewLines = qfalse;
        let mut data = *data_p;

        len = 0;
        qs.com_token[0] = 0;

        // make sure incoming data is valid
        if data.is_null() {
            *data_p = std::ptr::null();
            return qs.com_token.as_mut_ptr();
        }

        loop {
            // skip whitespace
            data = crate::q_shared::SkipWhitespace(qs, data, &mut hasNewLines);
            if data.is_null() {
                *data_p = std::ptr::null();
                return qs.com_token.as_mut_ptr();
            }
            if hasNewLines == qtrue && allowLineBreaks == qfalse {
                *data_p = data;
                return qs.com_token.as_mut_ptr();
            }

            c = *data as c_int;

            // skip double slash comments
            if c == b'/' as c_int && *data.offset(1) == b'/' as c_char {
                data = data.offset(2);
                while *data != 0 && *data != b'\n' as c_char {
                    data = data.offset(1);
                }
            } else if c == b'/' as c_int && *data.offset(1) == b'*' as c_char {
                data = data.offset(2);
                while *data != 0 && !(*data == b'*' as c_char && *data.offset(1) == b'/' as c_char)
                {
                    data = data.offset(1);
                }
                if *data != 0 {
                    data = data.offset(2);
                }
            } else {
                break;
            }
        }

        // handle quoted strings
        if c == b'"' as c_int {
            data = data.offset(1);
            loop {
                c = *data as c_int;
                data = data.offset(1);
                if c == b'"' as c_int || c == 0 {
                    // Raven's quoted path omits the `len == MAX_TOKEN_CHARS`
                    // reset the word path below applies, so a buffer-filling
                    // token writes the terminator one past `com_token`. Clamp
                    // to the last slot rather than reproduce that overrun.
                    qs.com_token[len.min(MAX_TOKEN_CHARS as c_int - 1) as usize] = 0;
                    *data_p = data as *const c_char;
                    return qs.com_token.as_mut_ptr();
                }
                if len < MAX_TOKEN_CHARS as c_int {
                    qs.com_token[len as usize] = c as c_char;
                    len += 1;
                }
            }
        }

        // parse a regular word
        loop {
            if len < MAX_TOKEN_CHARS as c_int {
                qs.com_token[len as usize] = c as c_char;
                len += 1;
            }
            data = data.offset(1);
            c = *data as c_int;
            if c == b'\n' as c_int {
                qs.com_lines += 1;
            }
            if !(c > b' ' as c_int) {
                break;
            }
        }

        if len == MAX_TOKEN_CHARS as c_int {
            len = 0;
        }
        qs.com_token[len as usize] = 0;

        *data_p = data as *const c_char;
        qs.com_token.as_mut_ptr()
    }
}

/// Raven `COM_ParseString`.
///
/// Source: `oracle/codemp/game/q_shared.c:588-598`
pub fn COM_ParseString(
    qs: &mut QSharedScratch,
    data: *mut *const c_char,
    s: *mut *const c_char,
) -> qboolean {
    unsafe {
        let token = crate::q_shared::COM_ParseExt(qs, data, qfalse);
        *s = token as *const c_char;
        // Raven's guard is literally `if ( s[0] == 0 )` — `s` is `const
        // char **`, so `s[0]` is the token pointer itself, not `*token`.
        // That's always non-zero here (COM_ParseExt never returns NULL),
        // so the oracle's check is dead in practice; preserved faithfully
        // as a null-pointer check rather than silently "fixed" to `*token`.
        if (*s).is_null() {
            com_printf_lit("unexpected EOF\n");
            return qtrue;
        }
        qfalse
    }
}

/// Raven `COM_ParseInt`.
///
/// Source: `oracle/codemp/game/q_shared.c:605-618`
pub fn COM_ParseInt(qs: &mut QSharedScratch, data: *mut *const c_char, i: *mut c_int) -> qboolean {
    unsafe {
        let token = crate::q_shared::COM_ParseExt(qs, data, qfalse);
        if *token == 0 {
            com_printf_lit("unexpected EOF\n");
            return qtrue;
        }
        *i = atoi(token as *const c_char);
        qfalse
    }
}

/// Raven `COM_ParseFloat`.
///
/// Source: `oracle/codemp/game/q_shared.c:625-638`
pub fn COM_ParseFloat(qs: &mut QSharedScratch, data: *mut *const c_char, f: *mut f32) -> qboolean {
    unsafe {
        let token = crate::q_shared::COM_ParseExt(qs, data, qfalse);
        if *token == 0 {
            com_printf_lit("unexpected EOF\n");
            return qtrue;
        }
        *f = crate::bg_lib::atof(token as *const c_char) as f32;
        qfalse
    }
}

/// Raven `COM_ParseVec4`.
///
/// Source: `oracle/codemp/game/q_shared.c:645-659`
pub fn COM_ParseVec4(
    qs: &mut QSharedScratch,
    buffer: *mut *const c_char,
    c: *mut vec4_t,
) -> qboolean {
    unsafe {
        for i in 0..4usize {
            let mut f = 0.0f32;
            if crate::q_shared::COM_ParseFloat(qs, buffer, &mut f) == qtrue {
                return qtrue;
            }
            (*c)[i] = f;
        }
        qfalse
    }
}

/// Raven `COM_MatchToken`.
///
/// Source: `oracle/codemp/game/q_shared.c:666-673`
pub fn COM_MatchToken(qs: &mut QSharedScratch, buf_p: *mut *const c_char, r#match: *mut c_char) {
    unsafe {
        let token = crate::q_shared::COM_Parse(qs, buf_p);
        if c_strcmp(token as *const c_char, r#match as *const c_char) != 0 {
            let t = std::ffi::CStr::from_ptr(token).to_string_lossy();
            let m = std::ffi::CStr::from_ptr(r#match).to_string_lossy();
            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
            panic!("MatchToken: {t} != {m}");
        }
    }
}

/// Raven `SkipBracedSection`.
///
/// Source: `oracle/codemp/game/q_shared.c:685-701`
pub fn SkipBracedSection(qs: &mut QSharedScratch, program: *mut *const c_char) {
    unsafe {
        let mut depth: c_int = 0;
        loop {
            let token = crate::q_shared::COM_ParseExt(qs, program, qtrue);
            if *token.offset(1) == 0 {
                if *token == b'{' as c_char {
                    depth += 1;
                } else if *token == b'}' as c_char {
                    depth -= 1;
                }
            }
            if !(depth != 0 && !(*program).is_null()) {
                break;
            }
        }
    }
}

/// Raven `SkipRestOfLine`.
///
/// Source: `oracle/codemp/game/q_shared.c:708-721`
pub fn SkipRestOfLine(qs: &mut QSharedScratch, data: *mut *const c_char) {
    unsafe {
        let mut p = *data;
        let mut c: c_int;

        loop {
            c = *p as c_int;
            p = p.offset(1);
            if c == 0 {
                break;
            }
            if c == b'\n' as c_int {
                qs.com_lines += 1;
                break;
            }
        }

        *data = p;
    }
}

/// Raven `Parse1DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:724-736`
pub fn Parse1DMatrix(qs: &mut QSharedScratch, buf_p: *mut *const c_char, x: c_int, m: *mut f32) {
    unsafe {
        crate::q_shared::COM_MatchToken(qs, buf_p, c"(".as_ptr() as *mut c_char);
        for i in 0..x {
            let token = crate::q_shared::COM_Parse(qs, buf_p);
            *m.offset(i as isize) = crate::bg_lib::atof(token as *const c_char) as f32;
        }
        crate::q_shared::COM_MatchToken(qs, buf_p, c")".as_ptr() as *mut c_char);
    }
}

/// Raven `Parse2DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:738-748`
pub fn Parse2DMatrix(
    qs: &mut QSharedScratch,
    buf_p: *mut *const c_char,
    y: c_int,
    x: c_int,
    m: *mut f32,
) {
    unsafe {
        crate::q_shared::COM_MatchToken(qs, buf_p, c"(".as_ptr() as *mut c_char);
        for i in 0..y {
            crate::q_shared::Parse1DMatrix(qs, buf_p, x, m.offset((i * x) as isize));
        }
        crate::q_shared::COM_MatchToken(qs, buf_p, c")".as_ptr() as *mut c_char);
    }
}

/// Raven `Parse3DMatrix`.
///
/// Source: `oracle/codemp/game/q_shared.c:750-760`
pub fn Parse3DMatrix(
    qs: &mut QSharedScratch,
    buf_p: *mut *const c_char,
    z: c_int,
    y: c_int,
    x: c_int,
    m: *mut f32,
) {
    unsafe {
        crate::q_shared::COM_MatchToken(qs, buf_p, c"(".as_ptr() as *mut c_char);
        for i in 0..z {
            crate::q_shared::Parse2DMatrix(qs, buf_p, y, x, m.offset((i * x * y) as isize));
        }
        crate::q_shared::COM_MatchToken(qs, buf_p, c")".as_ptr() as *mut c_char);
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

/// C standard-library `strcmp` (case-sensitive), as called bare (not via a
/// `Q_*` wrapper) at various `q_shared.c` sites (e.g. lines 548, 565, 670,
/// 1185, 1240). Housed alongside the other `q_shared.c` string helpers per
/// the file's existing string-fn family.
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

/// C standard-library `strchr` (first-occurrence character search), as
/// called bare (not via a `Q_*` wrapper) at various `q_shared.c` sites (e.g.
/// lines 1157, 1212, 1264, 1267, 1287, 1293, 1299, 1335, 1341, 1347). Housed
/// alongside the other `q_shared.c` string helpers per the file's existing
/// string-fn family.
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

/// Raven bare `strstr` call sites (e.g. `g_client.c` GLA-name matching).
///
/// House-authored wrapper, not a named Raven `Q_` fn: mirrors `Q_strchr`
/// above — ported to this canonical string-fn home since call sites need a
/// C-string `strstr` and none existed yet.
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
        crate::q_shared::Q_stricmpn(s1, s2, 99999)
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
        crate::q_shared::Q_strncpyz(dest.offset(l1 as isize), src, size - l1);
    }
}

/// Raven `Q_PrintStrlen`.
///
/// Note: Raven's `Q_IsColorString` is a `q_shared.h` inline predicate; no
/// resolved cross-file signature was provided for this file's packet
/// (0 header-inline helpers listed), so the color-escape skip (`^` + digit,
/// 2-byte stride) is inlined directly here rather than invoking an
/// unresolved symbol.
/// Source: `oracle/codemp/game/q_shared.c:940-960`
pub fn Q_PrintStrlen(string: *const c_char) -> c_int {
    unsafe {
        if string.is_null() {
            return 0;
        }
        let mut len: c_int = 0;
        let mut p = string;
        while *p != 0 {
            // `Q_IsColorString(p)` = `^` followed by a digit '0'..='7' (and not
            // '^'); the port previously accepted any non-NUL follower, which
            // over-counted `^^`/`^8...` escapes (caught by the oracle slice).
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
            // `Q_IsColorString(s)` = `^` followed by a digit '0'..='7' (see
            // Q_PrintStrlen); the port previously skipped on any non-NUL
            // follower, wrongly stripping `^^`/`^8...` (caught by the slice).
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
/// PORT-NOTE(variadic-c-abi): Raven's `...` becomes an explicit `&[FmtArg]`
/// channel formatted by `c_format::c_vsprintf` (native-libc `vsprintf` parity);
/// see `c_format` for the seam rationale. The 32000-byte `bigbuffer`, the
/// `ERR_FATAL` on `len >= sizeof(bigbuffer)` (→ panic, frozen Group A), the
/// `Com_Printf` overflow-of warning on `len >= size`, and the closing
/// `Q_strncpyz(dest, bigbuffer, size)` are reproduced exactly.
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
            crate::g_main::Com_Printf(cstr(&msg).as_ptr());
        }
        // Q_strncpyz needs a NUL-terminated source; `bigbuffer` has no interior
        // NUL (the formatter never emits one), so append the terminator here.
        let mut cbig = bigbuffer;
        cbig.push(0);
        crate::q_shared::Q_strncpyz(dest, cbig.as_ptr() as *const c_char, size);
    }
}

/// Raven `va`.
///
/// PORT-NOTE(variadic-c-abi): Raven's `...` becomes an explicit `&[FmtArg]`
/// channel formatted by `c_format::c_vsprintf` (native-libc `vsprintf` parity).
/// The 2-slot rotating `static char string[2][32000]` return buffer and the
/// `index & 1` alternation are reproduced by the module statics. Raven's own
/// `// FIXME: make this buffer size safe someday` means a `>= 32000`-byte result
/// overruns in C; the port instead truncates into the 31999-usable-byte slot.
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

/// Raven `Info_ValueForKey`.
///
/// Source: `oracle/codemp/game/q_shared.c:1051-1098`
pub fn Info_ValueForKey(
    qs: &mut QSharedScratch,
    s: *const c_char,
    key: *const c_char,
) -> *mut c_char {
    unsafe {
        let mut pkey: [c_char; 8192] = [0; 8192]; // BIG_INFO_KEY
        let mut o: *mut c_char;

        if s.is_null() || key.is_null() {
            return c"".as_ptr() as *mut c_char;
        }

        if c_strlen(s) >= BIG_INFO_STRING {
            // Raven guards on `BIG_INFO_STRING` (8192), not `MAX_INFO_STRING`;
            // the port previously hard-coded 1024 (a divergence caught by the
            // oracle slice's big-infostring case). Com_Error(ERR_DROP, ...) -> panic.
            panic!("Info_ValueForKey: oversize infostring");
        }

        qs.info_valueindex ^= 1;
        let mut p = s;
        if *p == b'\\' as c_char {
            p = p.offset(1);
        }

        loop {
            o = pkey.as_mut_ptr();
            while *p != b'\\' as c_char {
                if *p == 0 {
                    return c"".as_ptr() as *mut c_char;
                }
                if o.offset_from(pkey.as_ptr()) < 8191 {
                    *o = *p;
                    o = o.offset(1);
                }
                p = p.offset(1);
            }
            *o = 0;
            p = p.offset(1);

            o = qs.info_value[qs.info_valueindex as usize].as_mut_ptr();

            while *p != b'\\' as c_char && *p != 0 {
                if o.offset_from(qs.info_value[qs.info_valueindex as usize].as_ptr()) < 8191 {
                    *o = *p;
                    o = o.offset(1);
                }
                p = p.offset(1);
            }
            *o = 0;

            // Raven matches the key case-INSENSITIVELY (`!Q_stricmp(key,pkey)`);
            // the port previously used case-sensitive strcmp (a divergence
            // caught by the oracle slice's "Name" vs "name" probe).
            if crate::q_shared::Q_stricmp(key, pkey.as_ptr() as *const c_char) == 0 {
                return qs.info_value[qs.info_valueindex as usize].as_mut_ptr();
            }

            if *p == 0 {
                break;
            }
            p = p.offset(1);
        }

        c"".as_ptr() as *mut c_char
    }
}

/// Raven `Info_NextPair`.
///
/// Source: `oracle/codemp/game/q_shared.c:1108-1139`
pub fn Info_NextPair(head: *mut *const c_char, key: *mut c_char, value: *mut c_char) {
    unsafe {
        let mut s = *head;

        if *s == b'\\' as c_char {
            s = s.offset(1);
        }
        *key = 0;
        *value = 0;

        let mut o = key;
        loop {
            if *s == b'\\' as c_char {
                break;
            }
            if *s == 0 {
                *o = 0;
                *head = s;
                return;
            }
            *o = *s;
            o = o.offset(1);
            s = s.offset(1);
        }
        *o = 0;
        s = s.offset(1);

        let mut o = value;
        while *s != b'\\' as c_char && *s != 0 {
            *o = *s;
            o = o.offset(1);
            s = s.offset(1);
        }
        *o = 0;

        *head = s;
    }
}

/// Raven `Info_RemoveKey`.
///
/// Source: `oracle/codemp/game/q_shared.c:1147-1195`
pub fn Info_RemoveKey(mut s: *mut c_char, key: *const c_char) {
    unsafe {
        if c_strlen(s as *const c_char) >= MAX_INFO_STRING {
            // Com_Error(ERR_DROP, ...) -> panic (frozen Group A).
            panic!("Info_RemoveKey: oversize infostring");
        }

        if !c_strchr(key, b'\\' as c_char).is_null() {
            return;
        }

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
                c_strcpy(start, p); // remove this part
                return;
            }

            if *p == 0 {
                return;
            }
            // advance `s` to just past this pair for the next iteration.
            s = p as *mut c_char;
        }
    }
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

/// Raven `Info_Validate`.
///
/// Source: `oracle/codemp/game/q_shared.c:1263-1271`
pub fn Info_Validate(s: *const c_char) -> qboolean {
    unsafe {
        if !c_strchr(s, b'"' as c_char).is_null() {
            return qfalse;
        }
        if !c_strchr(s, b';' as c_char).is_null() {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `Info_SetValueForKey`.
///
/// The `Com_sprintf(newi, sizeof(newi), "\\%s\\%s", key, value)` call has a
/// statically-known 2-arg format, so it's inlined directly (mechanical
/// identity) rather than routed through the parked variadic `Com_sprintf`.
/// Source: `oracle/codemp/game/q_shared.c:1280-1319`
pub fn Info_SetValueForKey(s: *mut c_char, key: *const c_char, value: *const c_char) {
    unsafe {
        if c_strlen(s as *const c_char) >= MAX_INFO_STRING {
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

        crate::q_shared::Info_RemoveKey(s, key);
        if value.is_null() || c_strlen(value) == 0 {
            return;
        }

        let key_s = std::ffi::CStr::from_ptr(key).to_string_lossy();
        let value_s = std::ffi::CStr::from_ptr(value).to_string_lossy();
        let newi = format!("\\{key_s}\\{value_s}");
        let s_s = std::ffi::CStr::from_ptr(s).to_string_lossy();

        if newi.len() + s_s.len() > MAX_INFO_STRING {
            com_printf_lit("Info string length exceeded\n");
            return;
        }

        // strcat(newi, s); strcpy(s, newi);
        let full = format!("{newi}{s_s}");
        let cstr = std::ffi::CString::new(full).unwrap();
        c_strcpy(s, cstr.as_ptr());
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

        crate::q_shared::Info_RemoveKey_Big(s, key);
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
