//! `q_shared.c` string primitives — the shared-tier home for engine-island
//! callers (`mp_game` carries its own module-island copies in `q_shared.rs`).

use core::ffi::{c_char, c_int, CStr};

use native_string::Q_stricmpBytes;

use crate::shared::limits::{BIG_INFO_STRING, MAX_TOKEN_CHARS};
use crate::shared::q_format::{c_vsprintf, FmtArg};
use crate::shared::string_id_table::stringID_table_t;
use crate::shared::{qboolean, qfalse, qtrue, MAX_QPATH};

// Parse-session state (mirrors Raven's file-static globals in q_shared.c);
// module-island duplicate of the same statics in `mp_game`'s `q_shared.rs`.
static mut COM_LINES: c_int = 0;
static mut COM_TOKEN: [c_char; 1024] = [0; 1024]; // MAX_TOKEN_CHARS

// va() rotating-buffer statics (2-slot rotating return buffer).
static mut VA_STRING: [[c_char; 32000]; 2] = [[0; 32000]; 2];
static mut VA_INDEX: usize = 0;

// Info_ValueForKey rotating-buffer statics (same rotating idiom as va()).
static mut INFO_VALUE: [[c_char; 8192]; 2] = [[0; 8192]; 2]; // BIG_INFO_VALUE
static mut INFO_VALUEINDEX: c_int = 0;

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

// Raven `Com_Printf` for the `Info_*` diagnostics. At this (shared) tier there is
// no engine console; the fixed diagnostic strings go to stderr. Divergence:
// engine output routes through `Com_Printf`/`PlatformHost` (ruling 10) post-parity.
unsafe fn com_printf_lit(msg: &str) {
    eprint!("{msg}");
}

/// Raven `Q_stricmpn`.
///
/// Source: `oracle/codemp/game/q_shared.c:855-879`
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

/// Raven `SkipWhitespace`.
///
/// Source: `oracle/codemp/game/q_shared.c:336-351`
pub fn SkipWhitespace(data: *const c_char, hasNewLines: *mut qboolean) -> *const c_char {
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
                COM_LINES += 1;
                *hasNewLines = qtrue;
            }
            p = p.offset(1);
        }

        p
    }
}

/// Raven `COM_ParseExt`.
///
/// Source: `oracle/codemp/game/q_shared.c:421-526`
pub fn COM_ParseExt(data_p: *mut *const c_char, allowLineBreaks: qboolean) -> *mut c_char {
    unsafe {
        let mut c: c_int;
        let mut len: c_int;
        let mut hasNewLines = qfalse;
        let mut data = *data_p;

        len = 0;
        COM_TOKEN[0] = 0;

        // make sure incoming data is valid
        if data.is_null() {
            *data_p = std::ptr::null();
            return (&raw mut COM_TOKEN).cast::<c_char>();
        }

        loop {
            // skip whitespace
            data = SkipWhitespace(data, &mut hasNewLines);
            if data.is_null() {
                *data_p = std::ptr::null();
                return (&raw mut COM_TOKEN).cast::<c_char>();
            }
            if hasNewLines == qtrue && allowLineBreaks == qfalse {
                *data_p = data;
                return (&raw mut COM_TOKEN).cast::<c_char>();
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
                    COM_TOKEN[len.min(MAX_TOKEN_CHARS as c_int - 1) as usize] = 0;
                    *data_p = data as *const c_char;
                    return (&raw mut COM_TOKEN).cast::<c_char>();
                }
                if len < MAX_TOKEN_CHARS as c_int {
                    COM_TOKEN[len as usize] = c as c_char;
                    len += 1;
                }
            }
        }

        // parse a regular word
        loop {
            if len < MAX_TOKEN_CHARS as c_int {
                COM_TOKEN[len as usize] = c as c_char;
                len += 1;
            }
            data = data.offset(1);
            c = *data as c_int;
            if c == b'\n' as c_int {
                COM_LINES += 1;
            }
            if !(c > b' ' as c_int) {
                break;
            }
        }

        if len == MAX_TOKEN_CHARS as c_int {
            len = 0;
        }
        COM_TOKEN[len as usize] = 0;

        *data_p = data as *const c_char;
        (&raw mut COM_TOKEN).cast::<c_char>()
    }
}

/// Raven `COM_Parse` / `COM_ParseExt`.
///
/// Engine-island reshape: engine parsers walk a Rust `&str` cursor, so this
/// returns `(token, remaining)` over the borrow rather than Raven's `char **data`
/// / `static char com_token[]` pointer channel. `allowLineBreaks` selects
/// `COM_ParseExt(data, allowLineBreaks)` — Raven's `COM_Parse(data)` is the
/// `allowLineBreaks == qtrue` wrapper. The tokenizer is reused verbatim by
/// copying the cursor into a NUL-terminated buffer and mapping the consumed byte
/// count back onto the input slice.
/// Source: `oracle/codemp/game/q_shared.c:295-298,421-526`
pub fn COM_Parse(data: &str, allowLineBreaks: bool) -> (String, &str) {
    let c = std::ffi::CString::new(data).unwrap_or_default();
    let start = c.as_ptr();
    let mut p: *const c_char = start;
    let token = COM_ParseExt(&mut p, if allowLineBreaks { qtrue } else { qfalse });
    let token_str = unsafe { std::ffi::CStr::from_ptr(token) }
        .to_string_lossy()
        .into_owned();
    let rest = if p.is_null() {
        ""
    } else {
        let consumed = (p as usize) - (start as usize);
        data.get(consumed..).unwrap_or("")
    };
    (token_str, rest)
}

/// Raven `SkipBracedSection`.
///
/// Source: `oracle/codemp/game/q_shared.c:685-701`
pub fn SkipBracedSection(program: *mut *const c_char) {
    unsafe {
        let mut depth: c_int = 0;
        loop {
            let token = COM_ParseExt(program, qtrue);
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
pub fn SkipRestOfLine(data: *mut *const c_char) {
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
                COM_LINES += 1;
                break;
            }
        }

        *data = p;
    }
}

/// Raven `COM_StripExtension`.
///
/// Engine-island reshape: callers hand a Rust `&str` and want an owned result,
/// so this returns the stripped `String` rather than writing Raven's `out`
/// buffer. Behavior is identical — copy up to the first `.` (Raven's faithful
/// first-dot truncation).
/// Source: `oracle/codemp/game/q_shared.c:99-104`
pub fn COM_StripExtension(r#in: &str) -> String {
    match r#in.find('.') {
        Some(i) => r#in[..i].to_string(),
        None => r#in.to_string(),
    }
}

/// Raven `COM_DefaultExtension`.
///
/// Uses a fixed `"%s%s"` pattern (`oldPath`, `extension`) — inlined directly
/// rather than routed through a generic `Com_sprintf` seam (a statically-known
/// format with known args is a mechanical identity, not a design choice).
/// Source: `oracle/codemp/game/q_shared.c:112-131`
pub fn COM_DefaultExtension(path: *mut c_char, maxSize: c_int, extension: *const c_char) {
    unsafe {
        let len = c_strlen(path);
        let _ = len;
        let mut src = path.offset(c_strlen(path) as isize - 1);
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

/// Raven `Com_sprintf`.
///
/// Engine-island reshape: engine callers pre-render the format through Rust
/// `format!`, so this takes the already-rendered `&str` (mirroring the `FmtArg`
/// reshape rationale of `mp_game`'s copy) and reproduces only the buffer
/// semantics — the 32000-byte bigbuffer `ERR_FATAL` (→ panic, frozen Group A),
/// the `len >= size` overflow warning, and the closing `Q_strncpyz`.
/// Source: `oracle/codemp/game/q_shared.c:985-1005`
pub fn Com_sprintf(dest: *mut c_char, size: c_int, s: &str) {
    unsafe {
        let bytes = s.as_bytes();
        let len = bytes.len();
        if len >= 32000 {
            // Com_Error(ERR_FATAL, "Com_sprintf: overflowed bigbuffer") -> panic.
            panic!("Com_sprintf: overflowed bigbuffer");
        }
        if len as c_int >= size {
            com_printf_lit(&format!("Com_sprintf: overflow of {len} in {size}\n"));
        }
        // Q_strncpyz needs a NUL-terminated source.
        let mut cbig: Vec<c_char> = bytes.iter().map(|&b| b as c_char).collect();
        cbig.push(0);
        Q_strncpyz(dest, cbig.as_ptr() as *const c_char, size);
    }
}

/// Raven `va` — does a varargs printf into a temp buffer, so I don't need to
/// have varargs versions of all text functions.
///
/// Diverges: a `>= 32000`-byte result truncates instead of overrunning (Raven UB).
/// Source: `oracle/codemp/game/q_shared.c:1017-1031`
pub fn va(format: *const c_char, args: &[FmtArg]) -> *mut c_char {
    unsafe {
        let buf = VA_STRING[VA_INDEX & 1].as_mut_ptr();
        VA_INDEX += 1;

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
pub fn Info_ValueForKey(s: *const c_char, key: *const c_char) -> *mut c_char {
    unsafe {
        let mut pkey: [c_char; 8192] = [0; 8192]; // BIG_INFO_KEY
        let mut o: *mut c_char;

        if s.is_null() || key.is_null() {
            return c"".as_ptr() as *mut c_char;
        }

        if c_strlen(s) >= BIG_INFO_STRING {
            // Raven guards on `BIG_INFO_STRING` (8192), not `MAX_INFO_STRING`.
            // Com_Error(ERR_DROP, ...) -> panic.
            panic!("Info_ValueForKey: oversize infostring");
        }

        INFO_VALUEINDEX ^= 1;
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

            o = INFO_VALUE[INFO_VALUEINDEX as usize].as_mut_ptr();

            while *p != b'\\' as c_char && *p != 0 {
                if o.offset_from(INFO_VALUE[INFO_VALUEINDEX as usize].as_ptr()) < 8191 {
                    *o = *p;
                    o = o.offset(1);
                }
                p = p.offset(1);
            }
            *o = 0;

            // Raven matches the key case-INSENSITIVELY (`!Q_stricmp(key,pkey)`).
            if Q_stricmp(key, pkey.as_ptr() as *const c_char) == 0 {
                return INFO_VALUE[INFO_VALUEINDEX as usize].as_mut_ptr();
            }

            if *p == 0 {
                break;
            }
            p = p.offset(1);
        }

        c"".as_ptr() as *mut c_char
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
            // Q_PrintStrlen); inlined here rather than invoking an unresolved
            // symbol.
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

// S5-5: the following pure string helpers moved down from `mp_game`
// (`crates/mp/game/src/q_shared.rs`, byte-identical bodies) so the 11 bg files
// can retarget off `crate::q_shared`; imports/paths adjusted for this tier.

/// Raven `GetIDForString`.
///
/// Source: `oracle/codemp/game/q_shared.c:13-27`
pub fn GetIDForString(table: *mut stringID_table_t, string: &str) -> c_int {
    unsafe {
        let mut index: isize = 0;
        loop {
            let entry = *table.offset(index);
            if entry.name.is_null() || *entry.name == 0 {
                break;
            }
            if Q_stricmpBytes(CStr::from_ptr(entry.name).to_bytes(), string.as_bytes()) == 0 {
                return entry.id;
            }
            index += 1;
        }
        -1
    }
}

/// C standard-library `strcmp` (case-sensitive), as called bare (not via a
/// `Q_*` wrapper) at various `q_shared.c` sites. Housed alongside the other
/// `q_shared.c` string helpers per the file's existing string-fn family.
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
