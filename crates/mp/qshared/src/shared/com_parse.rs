//! `q_shared.c` `COM_Parse*` family threaded through `QSharedScratch`.
//!
//! S5-5 canonical move (user ruling): the safe-state Stage-3 `QSharedScratch`
//! type and the `&mut QSharedScratch`-threaded parser bg consumes live here in
//! `mp_qshared` (below the bg tier), so bg's saber/vehicle loaders can retarget
//! off `crate::q_shared`, and the engine island can later migrate off its
//! `static mut` twins in `q_string.rs` (which stay untouched for existing
//! engine callers). Bodies are byte-identical to the `mp_game` copies
//! (`crates/mp/game/src/q_shared.rs`); imports/paths adjusted for this tier.
//!
//! Source: `oracle/codemp/game/q_shared.c` (parse-session functions + statics).

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::shared::limits::MAX_TOKEN_CHARS;
use crate::shared::q_string::Q_strncpyz;
use crate::shared::{qboolean, qfalse, qtrue};

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

/// Raven `COM_BeginParseSession`.
///
/// PORT-NOTE(variadic-c-abi): Raven calls Com_sprintf with "%s" format; since Com_sprintf
/// cannot accept varargs in Rust, this implementation directly copies the name via Q_strncpyz.
/// Source: `oracle/codemp/game/q_shared.c:284-288`
pub fn COM_BeginParseSession(qs: &mut QSharedScratch, name: *const c_char) {
    // (Redundant `unsafe` wrapper dropped vs. the game copy — `Q_strncpyz` is a
    // safe fn; behavior identical. The game file masked the `unused_unsafe`
    // warning via its blanket `#![allow(unused)]`.)
    qs.com_lines = 0;
    Q_strncpyz(
        qs.com_parsename.as_mut_ptr(),
        name,
        MAX_TOKEN_CHARS as c_int,
    );
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
    COM_ParseExt(qs, data_p, qtrue)
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

/// Raven `COM_ParseExt`.
///
/// Source: `oracle/codemp/game/q_shared.c:421-526`
pub fn COM_ParseExt(
    qs: &mut QSharedScratch,
    data_p: *mut *const c_char,
    allowLineBreaks: qboolean,
) -> *mut c_char {
    unsafe {
        // (Dead `= 0` initializer dropped vs. the game copy — `c` is
        // unconditionally assigned in the loop below before any read; behavior
        // identical. The game file masked the `unused_assignments` warning.)
        let mut c: c_int;
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
            data = SkipWhitespace(qs, data, &mut hasNewLines);
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

/// Raven `SkipBracedSection`.
///
/// Source: `oracle/codemp/game/q_shared.c:685-701`
pub fn SkipBracedSection(qs: &mut QSharedScratch, program: *mut *const c_char) {
    unsafe {
        let mut depth: c_int = 0;
        loop {
            let token = COM_ParseExt(qs, program, qtrue);
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
