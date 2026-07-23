//! `q_shared.c` `COM_Parse*` family threaded through `QSharedScratch`.
//!
//! S5-5 canonical move (user ruling): the safe-state Stage-3 `QSharedScratch`
//! type and the session-threaded parser bg consumes live here in `mp_qshared`
//! (below the bg tier), so bg's saber/vehicle loaders can retarget off
//! `crate::q_shared`.
//!
//! Phase-5b: the raw-pointer / `static char com_token[]` machinery is retired.
//! The tokenizer is now a pure byte cursor — no `unsafe`, no `static`, no
//! `com_token` buffer — reproducing `COM_ParseExt`'s exact byte semantics
//! (signed-`char` whitespace: bytes `>= 0x80` sign-extend negative and count as
//! whitespace, so they terminate/skip like retail win32; `//`/`/* */` comment
//! skipping; quoted strings; `MAX_TOKEN_CHARS` bound and the word path's
//! `len == MAX` whole-token discard; embedded NUL = C-string end, §19).
//!
//! Cursor type: `Option<&[u8]>` rather than `&str`. Raven's cursor is a nullable
//! `const char *`; `None` reproduces `*data_p = NULL` (SkipWhitespace hitting
//! EOF), `Some(&[])` reproduces a non-null pointer parked on the terminator.
//! The slice is bytes, not `&str`, because the tokenizer is defined over raw
//! bytes with signed-`char` math: the qshared golden fixture deliberately
//! carries invalid-UTF-8 high bytes (`0x80`/`0xa0`/…) to pin exactly that rule,
//! which a `&str` cannot hold without UB, and sanitizing them would silently
//! gut the coverage. Parsed **tokens** come back as owned `String` (high bytes
//! never enter a word token — they are whitespace — so the lossy decode is a
//! no-op in practice), which is where consumers actually want text.
//!
//! Line counting is IDENTICAL to the pointer version (`COM_GetCurrentParseLine`
//! numbers unchanged): `com_lines` increments only for `\n` skipped as leading
//! whitespace and the single terminating `\n` of a word — never for newlines
//! inside `/* */` comments or quoted strings — which is why 5a's line-agnostic
//! `q_string::COM_Parse` cannot simply be delegated to (its consumed-span newline
//! total is not this quantity); the state machine is reproduced with the
//! increments injected at Raven's exact points.
//!
//! Source: `oracle/codemp/game/q_shared.c` (parse-session functions + statics).

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use crate::shared::limits::MAX_TOKEN_CHARS;
use native_string::Q_strncpyzBytes;

/// Raven's `q_shared.c` file-static parse/format state, moved into
/// `BgState.qs` (safe-state Stage 3 — no `static mut`, rule B3). Rotation
/// index semantics (`va`/`Info_ValueForKey` two-slot rings) preserved.
/// Source: `oracle/codemp/game/q_shared.c` file statics.
pub struct QSharedScratch {
    /// Raven `static int com_lines`.
    pub com_lines: c_int,
    /// Raven `static char com_parsename[MAX_TOKEN_CHARS]` — parse-error text.
    pub com_parsename: [c_char; 1024],
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
            va_string: Box::new([[0; 32000]; 2]),
            va_index: 0,
            info_value: Box::new([[0; 8192]; 2]),
            info_valueindex: 0,
        }
    }
}

/// Raven `COM_BeginParseSession`.
///
/// Raven calls `Com_sprintf` with a `"%s"` format; since Rust has no varargs,
/// this copies the name directly via the byte-bounded `Q_strncpyzBytes` (same
/// result). `name` is `&str`; the `com_parsename` buffer stays a fixed
/// `[c_char; MAX_TOKEN_CHARS]` (parse-error text only).
/// Source: `oracle/codemp/game/q_shared.c:284-288`
pub fn COM_BeginParseSession(qs: &mut QSharedScratch, name: &str) {
    qs.com_lines = 0;
    Q_strncpyzBytes(&mut qs.com_parsename, name.as_bytes(), MAX_TOKEN_CHARS);
}

/// Raven `COM_GetCurrentParseLine`.
///
/// Source: `oracle/codemp/game/q_shared.c:290-293`
pub fn COM_GetCurrentParseLine(qs: &QSharedScratch) -> c_int {
    qs.com_lines
}

/// Raven `COM_Parse` — `COM_ParseExt(data, qtrue)`.
///
/// Source: `oracle/codemp/game/q_shared.c:295-298`
pub fn COM_Parse<'a>(qs: &mut QSharedScratch, data: Option<&'a [u8]>) -> (String, Option<&'a [u8]>) {
    COM_ParseExt(qs, data, true)
}

/// Raven `COM_ParseExt` — the native byte-cursor tokenizer.
///
/// Returns `(token, remaining)`: `token` is the parsed word/quoted string
/// (owned, `MAX_TOKEN_CHARS`-bounded), `remaining` is the cursor past it
/// (`None` = Raven's `*data_p = NULL`). `allowLineBreaks == false` returns an
/// empty token + the cursor parked at the line break, exactly as Raven.
/// Source: `oracle/codemp/game/q_shared.c:421-526` (SkipWhitespace inlined,
/// `oracle/codemp/game/q_shared.c:336-351`).
pub fn COM_ParseExt<'a>(
    qs: &mut QSharedScratch,
    data: Option<&'a [u8]>,
    allowLineBreaks: bool,
) -> (String, Option<&'a [u8]>) {
    // make sure incoming data is valid (`if (!data) { *data_p = NULL; ... }`)
    let bytes = match data {
        Some(b) => b,
        None => return (String::new(), None),
    };
    let n = bytes.len();
    let mut i = 0usize;
    let mut hasNewLines = false;

    // Skip whitespace and `//` / `/* */` comments (COM_ParseExt's leading loop,
    // re-running SkipWhitespace after each comment). Breaks with `i` at the
    // first token byte.
    loop {
        // SkipWhitespace — counts `com_lines` for each `\n` consumed.
        loop {
            if i >= n {
                // `*data` past end (or embedded NUL below) => SkipWhitespace
                // returned NULL.
                return (String::new(), None);
            }
            let c = bytes[i];
            if (c as i8 as c_int) > b' ' as c_int {
                break;
            }
            if c == 0 {
                return (String::new(), None);
            }
            if c == b'\n' {
                qs.com_lines += 1;
                hasNewLines = true;
            }
            i += 1;
        }
        if hasNewLines && !allowLineBreaks {
            return (String::new(), Some(&bytes[i..]));
        }

        let c = bytes[i];
        if c == b'/' && i + 1 < n && bytes[i + 1] == b'/' {
            // skip `//` comments (stops at `\n`/end, not past it)
            i += 2;
            while i < n && bytes[i] != 0 && bytes[i] != b'\n' {
                i += 1;
            }
        } else if c == b'/' && i + 1 < n && bytes[i + 1] == b'*' {
            // skip `/* */` comments (newlines inside are NOT counted)
            i += 2;
            while i < n && bytes[i] != 0 && !(bytes[i] == b'*' && i + 1 < n && bytes[i + 1] == b'/')
            {
                i += 1;
            }
            if i < n && bytes[i] != 0 {
                i += 2;
            }
        } else {
            break;
        }
    }

    let mut token: Vec<u8> = Vec::new();

    // handle quoted strings
    if bytes[i] == b'"' {
        i += 1;
        loop {
            if i >= n {
                // `c = *data++` reads the terminating NUL => close.
                return (String::from_utf8_lossy(&token).into_owned(), Some(&bytes[n..]));
            }
            let c = bytes[i];
            i += 1;
            if c == b'"' || c == 0 {
                return (String::from_utf8_lossy(&token).into_owned(), Some(&bytes[i..]));
            }
            if token.len() < MAX_TOKEN_CHARS as usize {
                token.push(c);
            }
        }
    }

    // parse a regular word
    loop {
        if token.len() < MAX_TOKEN_CHARS as usize {
            token.push(bytes[i]);
        }
        i += 1;
        let c = if i < n { bytes[i] } else { 0 };
        if c == b'\n' {
            qs.com_lines += 1;
        }
        if (c as i8 as c_int) <= b' ' as c_int {
            break;
        }
    }

    if token.len() == MAX_TOKEN_CHARS as usize {
        // Raven's `if (len == MAX_TOKEN_CHARS) len = 0;` — the whole token is
        // discarded (com_token[0] = 0).
        token.clear();
    }

    (String::from_utf8_lossy(&token).into_owned(), Some(&bytes[i..]))
}

/// Raven `SkipBracedSection` — consume a balanced `{ ... }` block, returning the
/// cursor past it (`None` at EOF, Raven's `*program == NULL`).
///
/// `token[1] == 0` (single-char token) maps to `token.len() == 1`; the only
/// divergence would be an empty token, which occurs solely at EOF where the
/// `do..while (depth && *program)` guard exits on the null cursor regardless, so
/// the observable (final cursor) is identical.
/// Source: `oracle/codemp/game/q_shared.c:685-701`
pub fn SkipBracedSection<'a>(
    qs: &mut QSharedScratch,
    program: Option<&'a [u8]>,
) -> Option<&'a [u8]> {
    let mut depth: c_int = 0;
    let mut cursor = program;
    loop {
        let (token, rest) = COM_ParseExt(qs, cursor, true);
        cursor = rest;
        if token.len() == 1 {
            match token.as_bytes()[0] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
        }
        if !(depth != 0 && cursor.is_some()) {
            break;
        }
    }
    cursor
}

/// Raven `SkipRestOfLine` — consume through the next `\n` (inclusive) or the
/// terminating NUL/end, counting the `\n` in `com_lines`.
///
/// Raven advances one past the terminator (`c = *p++`), so a NUL-terminated
/// buffer with no `\n` leaves the cursor one past the NUL — preserved here by
/// consuming the trailing `0` byte when present (callers feed NUL-terminated
/// buffers).
/// Source: `oracle/codemp/game/q_shared.c:708-721`
pub fn SkipRestOfLine<'a>(qs: &mut QSharedScratch, data: Option<&'a [u8]>) -> Option<&'a [u8]> {
    let bytes = match data {
        Some(b) => b,
        None => return None,
    };
    let n = bytes.len();
    let mut i = 0usize;
    while i < n {
        let c = bytes[i];
        i += 1;
        if c == 0 {
            break;
        }
        if c == b'\n' {
            qs.com_lines += 1;
            break;
        }
    }
    Some(&bytes[i..])
}
