//! `q_shared.c` string primitives — the shared-tier home for engine-island
//! callers (`mp_game` carries its own module-island copies in `q_shared.rs`).

use core::ffi::{c_char, c_int, CStr};

use native_string::Q_stricmpBytes;

use crate::shared::limits::MAX_TOKEN_CHARS;
use crate::shared::q_format::{c_vsprintf, FmtArg};
use crate::shared::string_id_table::stringID_table_t;
use crate::shared::MAX_QPATH;

// va() rotating-buffer statics (2-slot rotating return buffer).
static mut VA_STRING: [[c_char; 32000]; 2] = [[0; 32000]; 2];
static mut VA_INDEX: usize = 0;

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

/// Borrowed `&str` view of a NUL-terminated `[c_char; N]` field — the read
/// accessor for ABI-frozen inline char arrays (the DEC-33 structs' `_str()`
/// methods route here). A missing NUL or non-UTF-8 bytes decode as `""`.
// Sound: `c_char` and `u8` are both 1-byte with every bit pattern valid — a
// pure type pun over the same bytes and length.
pub fn chars_str(a: &[c_char]) -> &str {
    let bytes = unsafe { core::slice::from_raw_parts(a.as_ptr() as *const u8, a.len()) };
    core::ffi::CStr::from_bytes_until_nul(bytes)
        .ok()
        .and_then(|c| c.to_str().ok())
        .unwrap_or("")
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

/// Raven `COM_Parse` / `COM_ParseExt` — the native `&str` tokenizer.
///
/// Reshape: returns `(token, remaining)` over the borrow instead of Raven's
/// `char **data` / `static char com_token[]` pointer channel. `allowLineBreaks`
/// selects `COM_ParseExt(data, allowLineBreaks)`; Raven's `COM_Parse(data)` is
/// the `qtrue` wrapper. Pure byte cursor — no `static mut`, no `CString`.
///
/// Signed-`char` fidelity: retail win32 `char` is signed, so Raven's
/// `SkipWhitespace` (`(c = *data) <= ' '`) and the word terminator (`c > 32`)
/// sign-extend each byte — every byte `>= 0x80` is negative, i.e. counts as
/// whitespace. Bytes are cast through `i8` here to reproduce that exactly: a
/// high byte (a UTF-8 unit of a Latin-1 glyph) is whitespace-skipped and never
/// enters an unquoted token, matching the oracle byte-for-byte. Quoted strings
/// keep Raven's rule of storing every byte except `"`/NUL.
///
/// Diverges (§19): Raven's `com_token[len] = 0` writes at exactly
/// `MAX_TOKEN_CHARS` (a one-past overrun on the quoted path) are not
/// reproduced — the token is length-bounded — while the word path's defined
/// `if (len == MAX_TOKEN_CHARS) len = 0` whole-token discard is kept. An
/// embedded NUL terminates parsing (Raven's C-string end), unlike the retired
/// `CString` wrapper which emptied the whole input.
/// Source: `oracle/codemp/game/q_shared.c:295-298,336-351,421-526`
pub fn COM_Parse(data: &str, allowLineBreaks: bool) -> (String, &str) {
    let bytes = data.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    let mut has_new_lines = false;

    // COM_ParseExt's leading loop: skip whitespace (SkipWhitespace) and
    // `//` + `/* */` comments, re-running the whitespace skip after each.
    'skip: loop {
        loop {
            if i >= n {
                return (String::new(), "");
            }
            let b = bytes[i];
            if (b as i8 as i32) > b' ' as i32 {
                break;
            }
            if b == 0 {
                return (String::new(), "");
            }
            if b == b'\n' {
                has_new_lines = true;
            }
            i += 1;
        }
        if has_new_lines && !allowLineBreaks {
            return (String::new(), &data[i..]);
        }
        if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            i += 2;
            while i < n && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            i += 2;
            while i < n && !(bytes[i] == b'*' && i + 1 < n && bytes[i + 1] == b'/') {
                i += 1;
            }
            if i < n {
                i += 2;
            }
        } else {
            break 'skip;
        }
    }

    let mut token: Vec<u8> = Vec::new();

    // handle quoted strings
    if bytes[i] == b'"' {
        i += 1;
        loop {
            if i >= n {
                return (String::from_utf8_lossy(&token).into_owned(), "");
            }
            let ch = bytes[i];
            i += 1;
            if ch == b'"' || ch == 0 {
                return (String::from_utf8_lossy(&token).into_owned(), &data[i..]);
            }
            if token.len() < MAX_TOKEN_CHARS as usize {
                token.push(ch);
            }
        }
    }

    // parse a regular word
    loop {
        if token.len() < MAX_TOKEN_CHARS as usize {
            token.push(bytes[i]);
        }
        i += 1;
        if i >= n || (bytes[i] as i8 as i32) <= b' ' as i32 {
            break;
        }
    }
    if token.len() == MAX_TOKEN_CHARS as usize {
        token.clear();
    }
    (String::from_utf8_lossy(&token).into_owned(), &data[i..])
}

/// Raven `SkipBracedSection` — native `&str` form: consume a balanced
/// `{ ... }` block, returning the cursor past it.
///
/// Reshape: threads the `&str` cursor and returns the remainder instead of
/// Raven's `char **program`. Raven's `*program != NULL` guard (loop-exhaustion
/// safety for unbalanced input) maps to "the cursor has no further content";
/// a `""` empty-token iteration changes neither `depth` nor the final position,
/// so an empty remainder is a faithful stand-in.
/// Source: `oracle/codemp/game/q_shared.c:685-701`
pub fn SkipBracedSection(program: &str) -> &str {
    let mut depth: c_int = 0;
    let mut cursor = program;
    loop {
        let (token, rest) = COM_Parse(cursor, true);
        cursor = rest;
        if token.len() == 1 {
            match token.as_bytes()[0] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
        }
        if depth == 0 || cursor.is_empty() {
            break;
        }
    }
    cursor
}

/// Raven `SkipRestOfLine` — native `&str` form: consume through the next
/// newline (inclusive), or to end of input / an embedded NUL.
///
/// Source: `oracle/codemp/game/q_shared.c:708-721`
pub fn SkipRestOfLine(data: &str) -> &str {
    let bytes = data.as_bytes();
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        let c = bytes[i];
        i += 1;
        if c == b'\n' || c == 0 {
            break;
        }
    }
    &data[i..]
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
