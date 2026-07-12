#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]
//! `l_script.cpp` — the botlib script tokenizer (function bodies).
//!
//! One Rust module per oracle source file (`l_script.cpp`); the stem collides
//! with the `l_script/` type directory, so this lands as `l_script_fns.rs`.
//!
//! Source: `oracle/codemp/botlib/l_script.cpp`
//!
//! PORT-NOTE(BotLib): `BotLib` is the synthesized botlib aggregate (per
//! `_PREAMBLE.md`'s state-receiver table) — not yet defined anywhere in the
//! tree, matching every sibling `*_fns.rs` file in this crate that already
//! references `bot: &mut BotLib` / `bot.botimport` / `bot.basefolder` ahead
//! of its landing. Reported in missing_symbols.
//!
//! PORT-NOTE(variadic): `ScriptError`/`ScriptWarning` use Raven's
//! `va_start`/`vsprintf`/`va_end` C-variadic seam; a plain Rust fn cannot read
//! `...`, so the `va_list` plumbing is resolved at integration (same seam as
//! `SourceError`/`SourceWarning` in `l_precomp_fns.rs`). The body is
//! transcribed line-for-line against that seam.

use core::ffi::{c_char, c_int, c_long, c_ulong, c_void};

use libc::sprintf;

use crate::l_script::consts::{
    MAX_TOKEN, SCFL_NOERRORS, SCFL_NOSTRINGESCAPECHARS, SCFL_NOSTRINGWHITESPACES, SCFL_NOWARNINGS,
    SCFL_PRIMITIVE, TT_BINARY, TT_DECIMAL, TT_FLOAT, TT_HEX, TT_INTEGER, TT_LITERAL, TT_LONG,
    TT_NAME, TT_NUMBER, TT_OCTAL, TT_PUNCTUATION, TT_STRING, TT_UNSIGNED,
};
use crate::l_script::punctuation_s::punctuation_t;
use crate::l_script::script_s::script_t;
use crate::l_script::token_s::token_t;
use crate::BotLib;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_WARNING};
use mp_qshared::shared::{fileHandle_t, FS_READ, MAX_QPATH};

use crate::l_memory_fns::{FreeMemory, GetClearedMemory, GetMemory};
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};
use mp_qshared::shared::q_string::{COM_Compress, Q_strncpyz};

// Raven's `long double` has no Rust equivalent; `f64` matches every existing
// use here (parsed/stored as a plain float, never relying on 80-bit extended
// precision).
// Source: `oracle/codemp/botlib/l_script.cpp` (multiple `long double` locals)
#[allow(non_camel_case_types)]
type long_double = f64;

/// Raven `PunctuationFromNum` — look up a punctuation's text by its number.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:200-209`
pub fn PunctuationFromNum(script: *mut script_t, num: c_int) -> *mut c_char {
    unsafe {
        let mut i = 0isize;
        loop {
            let p = (*script).punctuations.offset(i);
            if (*p).p.is_null() {
                break;
            }
            if (*p).n == num {
                return (*p).p;
            }
            i += 1;
        }
        c"unkown punctuation".as_ptr() as *mut c_char
    }
}

/// Raven `ScriptError` — print a tokenizer error tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:216-235`
///
/// PORT-NOTE(variadic): Raven's `va_start`/`vsprintf`/`va_end` C-variadic seam
/// cannot be a non-extern Rust fn `...`. Resolved at integration (same seam as
/// `SourceError`/`SourceWarning` in `l_precomp_fns.rs`): the fn now takes an
/// already-rendered message; the `script_error!` macro below reproduces the
/// original `vsprintf`-into-buffer step at each call site.
pub fn ScriptError(bot: &mut BotLib, script: *mut script_t, text: *const c_char) {
    unsafe {
        if (*script).flags & SCFL_NOERRORS != 0 {
            return;
        }

        // #ifdef BOTLIB (defined)
        (bot.botimport.Print.unwrap())(
            PRT_ERROR,
            c"file %s, line %d: %s\n".as_ptr() as *mut c_char,
            (*script).filename.as_ptr(),
            (*script).line,
            text,
        );
    }
}

/// Raven `ScriptWarning` — print a tokenizer warning tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:242-261`
///
/// PORT-NOTE(variadic): see `ScriptError` — same `va_list` seam.
pub fn ScriptWarning(bot: &mut BotLib, script: *mut script_t, text: *const c_char) {
    unsafe {
        if (*script).flags & SCFL_NOWARNINGS != 0 {
            return;
        }

        // #ifdef BOTLIB (defined)
        (bot.botimport.Print.unwrap())(
            PRT_WARNING,
            c"file %s, line %d: %s\n".as_ptr() as *mut c_char,
            (*script).filename.as_ptr(),
            (*script).line,
            text,
        );
    }
}

// PORT-NOTE(variadic): reproduces Raven's `vsprintf(text, str, ap)` step at
// each `ScriptError`/`ScriptWarning` call site (the C variadic seam resolved
// above), then forwards the rendered buffer.
macro_rules! script_error {
    ($bot:expr, $script:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __se_text = [0 as c_char; 1024];
        sprintf(__se_text.as_mut_ptr(), $fmt $(, $arg)*);
        ScriptError($bot, $script, __se_text.as_ptr())
    }};
}
macro_rules! script_warning {
    ($bot:expr, $script:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __sw_text = [0 as c_char; 1024];
        sprintf(__sw_text.as_mut_ptr(), $fmt $(, $arg)*);
        ScriptWarning($bot, $script, __sw_text.as_ptr())
    }};
}

/// Raven `PS_ReadWhiteSpace` — skip whitespace and `//`/`/* */` comments.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:285-335`
pub fn PS_ReadWhiteSpace(script: *mut script_t) -> c_int {
    unsafe {
        loop {
            // skip white space
            while *(*script).script_p <= b' ' as c_char {
                if *(*script).script_p == 0 {
                    return 0;
                }
                if *(*script).script_p == b'\n' as c_char {
                    (*script).line += 1;
                }
                (*script).script_p = (*script).script_p.offset(1);
            }
            // skip comments
            if *(*script).script_p == b'/' as c_char {
                // comments //
                if *(*script).script_p.offset(1) == b'/' as c_char {
                    (*script).script_p = (*script).script_p.offset(1);
                    loop {
                        (*script).script_p = (*script).script_p.offset(1);
                        if *(*script).script_p == 0 {
                            return 0;
                        }
                        if *(*script).script_p == b'\n' as c_char {
                            break;
                        }
                    }
                    (*script).line += 1;
                    (*script).script_p = (*script).script_p.offset(1);
                    if *(*script).script_p == 0 {
                        return 0;
                    }
                    continue;
                }
                // comments /* */
                else if *(*script).script_p.offset(1) == b'*' as c_char {
                    (*script).script_p = (*script).script_p.offset(1);
                    loop {
                        (*script).script_p = (*script).script_p.offset(1);
                        if *(*script).script_p == 0 {
                            return 0;
                        }
                        if *(*script).script_p == b'\n' as c_char {
                            (*script).line += 1;
                        }
                        if *(*script).script_p == b'*' as c_char
                            && *(*script).script_p.offset(1) == b'/' as c_char
                        {
                            break;
                        }
                    }
                    (*script).script_p = (*script).script_p.offset(1);
                    if *(*script).script_p == 0 {
                        return 0;
                    }
                    (*script).script_p = (*script).script_p.offset(1);
                    if *(*script).script_p == 0 {
                        return 0;
                    }
                    continue;
                }
            }
            break;
        }
        1
    }
}

/// Raven `NumberValue` — parse a decoded token's decimal/int/float value out
/// of its string form.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:541-606`
///
/// PORT-NOTE(long_double): Raven's `long double *floatvalue` param has no
/// rosetta entry (`long double` is unresolved — see missing_symbols); written
/// verbatim per the packet's resolved signature.
pub fn NumberValue(
    mut string: *mut c_char,
    subtype: c_int,
    intvalue: *mut c_ulong,
    floatvalue: *mut long_double,
) {
    unsafe {
        let mut dotfound: c_ulong = 0;

        *intvalue = 0;
        *floatvalue = 0.0;
        // floating point number
        if subtype & TT_FLOAT != 0 {
            while *string != 0 {
                if *string == b'.' as c_char {
                    if dotfound != 0 {
                        return;
                    }
                    dotfound = 10;
                    string = string.offset(1);
                }
                if dotfound != 0 {
                    *floatvalue +=
                        (*string as c_int - b'0' as c_int) as long_double / dotfound as long_double;
                    dotfound *= 10;
                } else {
                    *floatvalue =
                        *floatvalue * 10.0 + (*string as c_int - b'0' as c_int) as long_double;
                }
                string = string.offset(1);
            }
            *intvalue = *floatvalue as c_ulong;
        } else if subtype & TT_DECIMAL != 0 {
            while *string != 0 {
                *intvalue = *intvalue * 10 + (*string as c_int - b'0' as c_int) as c_ulong;
                string = string.offset(1);
            }
            *floatvalue = *intvalue as long_double;
        } else if subtype & TT_HEX != 0 {
            // step over the leading 0x or 0X
            string = string.offset(2);
            while *string != 0 {
                *intvalue <<= 4;
                let c = *string;
                if c >= b'a' as c_char && c <= b'f' as c_char {
                    *intvalue += (c as c_int - b'a' as c_int + 10) as c_ulong;
                } else if c >= b'A' as c_char && c <= b'F' as c_char {
                    *intvalue += (c as c_int - b'A' as c_int + 10) as c_ulong;
                } else {
                    *intvalue += (c as c_int - b'0' as c_int) as c_ulong;
                }
                string = string.offset(1);
            }
            *floatvalue = *intvalue as long_double;
        } else if subtype & TT_OCTAL != 0 {
            // step over the first zero
            string = string.offset(1);
            while *string != 0 {
                *intvalue = (*intvalue << 3) + (*string as c_int - b'0' as c_int) as c_ulong;
                string = string.offset(1);
            }
            *floatvalue = *intvalue as long_double;
        } else if subtype & TT_BINARY != 0 {
            // step over the leading 0b or 0B
            string = string.offset(2);
            while *string != 0 {
                *intvalue = (*intvalue << 1) + (*string as c_int - b'0' as c_int) as c_ulong;
                string = string.offset(1);
            }
            *floatvalue = *intvalue as long_double;
        }
    }
}

/// Raven `PS_ReadPunctuation` — match the longest punctuation at the cursor.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:768-802`
///
/// PORT-NOTE(PUNCTABLE): Raven's `#ifdef PUNCTABLE` hashed-table lookup is
/// dropped per porting-rules §C10 (control-flow behavior, not shape) — the
/// linear `#else` scan over `script->punctuations[]` is transcribed, matching
/// the `PUNCTABLE`-undefined build already assumed elsewhere in this file.
pub fn PS_ReadPunctuation(script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut i = 0isize;
        loop {
            let punc = (*script).punctuations.offset(i);
            if (*punc).p.is_null() {
                break;
            }
            let p = (*punc).p;
            let len = libc::strlen(p);
            // if the script contains at least as much characters as the punctuation
            if (*script).script_p.offset(len as isize) <= (*script).end_p {
                // if the script contains the punctuation
                if libc::strncmp((*script).script_p, p, len) == 0 {
                    libc::strncpy((*token).string.as_mut_ptr(), p, MAX_TOKEN);
                    (*script).script_p = (*script).script_p.offset(len as isize);
                    (*token).r#type = TT_PUNCTUATION;
                    // sub type is the number of the punctuation
                    (*token).subtype = (*punc).n;
                    return 1;
                }
            }
            i += 1;
        }
        0
    }
}

/// Raven `PS_UnreadLastToken` (comment says `UnreadLastToken`) — mark the
/// last-read token available again.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1063-1066`
pub fn PS_UnreadLastToken(script: *mut script_t) {
    unsafe {
        (*script).tokenavailable = 1;
    }
}

/// Raven `PS_NextWhiteSpaceChar` — step through the last token's saved
/// whitespace span one character at a time.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1085-1095`
pub fn PS_NextWhiteSpaceChar(script: *mut script_t) -> c_char {
    unsafe {
        if (*script).whitespace_p != (*script).endwhitespace_p {
            let c = *(*script).whitespace_p;
            (*script).whitespace_p = (*script).whitespace_p.offset(1);
            c
        } else {
            0
        }
    }
}

/// Raven `StripDoubleQuotes` — strip a single leading/trailing `"` pair.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1102-1112`
pub fn StripDoubleQuotes(string: *mut c_char) {
    unsafe {
        if *string == b'\"' as c_char {
            libc::strcpy(string, string.offset(1));
        }
        let len = libc::strlen(string);
        if *string.offset(len as isize - 1) == b'\"' as c_char {
            *string.offset(len as isize - 1) = 0;
        }
    }
}

/// Raven `StripSingleQuotes` — strip a single leading/trailing `'` pair.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1119-1129`
pub fn StripSingleQuotes(string: *mut c_char) {
    unsafe {
        if *string == b'\'' as c_char {
            libc::strcpy(string, string.offset(1));
        }
        let len = libc::strlen(string);
        if *string.offset(len as isize - 1) == b'\'' as c_char {
            *string.offset(len as isize - 1) = 0;
        }
    }
}

/// Raven `SetScriptFlags` — replace a script's flag word.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1182-1185`
pub fn SetScriptFlags(script: *mut script_t, flags: c_int) {
    unsafe {
        (*script).flags = flags;
    }
}

/// Raven `GetScriptFlags` — read a script's flag word.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1192-1195`
pub fn GetScriptFlags(script: *mut script_t) -> c_int {
    unsafe { (*script).flags }
}

/// Raven `EndOfScript` — true once the cursor has reached the buffer end.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1227-1230`
pub fn EndOfScript(script: *mut script_t) -> c_int {
    unsafe { ((*script).script_p >= (*script).end_p) as c_int }
}

/// Raven `NumLinesCrossed` — lines crossed since the last saved line count.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1237-1240`
pub fn NumLinesCrossed(script: *mut script_t) -> c_int {
    unsafe { (*script).line - (*script).lastline }
}

/// Raven `PS_SetBaseFolder` — set the tokenizer's base folder for
/// `LoadScriptFile`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1411-1418`
///
/// PORT-NOTE(BSPC): the `#ifdef BSPC` `sprintf` arm is dropped per §C10 —
/// `BSPC` is not defined in this tree, matching `PC_SetBaseFolder`'s existing
/// call site (`l_precomp_fns.rs`) which already forwards here.
pub fn PS_SetBaseFolder(bot: &mut BotLib, path: *mut c_char) {
    unsafe {
        let path_str = core::ffi::CStr::from_ptr(path).to_string_lossy();
        let __s = std::ffi::CString::new(path_str.to_string()).unwrap_or_default();
        Q_strncpyz(
            bot.basefolder.as_mut_ptr(),
            __s.as_ptr(),
            MAX_QPATH as c_int,
        );
    }
}

/// Raven `PS_CreatePunctuationTable` — build the per-first-character
/// punctuation lookup table, longest match first.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:160-193`
pub fn PS_CreatePunctuationTable(
    bot: &mut BotLib,
    script: *mut script_t,
    punctuations: *mut punctuation_t,
) {
    unsafe {
        // get memory for the table
        if (*script).punctuationtable.is_null() {
            (*script).punctuationtable = GetMemory(
                bot,
                (256 * core::mem::size_of::<*mut punctuation_t>()) as c_ulong,
            ) as *mut *mut punctuation_t;
        }
        Com_Memset(
            (*script).punctuationtable as *mut (),
            0,
            256 * core::mem::size_of::<*mut punctuation_t>(),
        );
        // add the punctuations in the list to the punctuation table
        let mut i = 0isize;
        loop {
            let newp = punctuations.offset(i);
            if (*newp).p.is_null() {
                break;
            }
            let mut lastp: *mut punctuation_t = core::ptr::null_mut();
            let idx = *(*newp).p as u8 as usize;
            // sort the punctuations in this table entry on length (longer punctuations first)
            let mut p = *(*script).punctuationtable.add(idx);
            let mut found = false;
            while !p.is_null() {
                if libc::strlen((*p).p) < libc::strlen((*newp).p) {
                    (*newp).next = p;
                    if !lastp.is_null() {
                        (*lastp).next = newp;
                    } else {
                        *(*script).punctuationtable.add(idx) = newp;
                    }
                    found = true;
                    break;
                }
                lastp = p;
                p = (*p).next;
            }
            if !found {
                (*newp).next = core::ptr::null_mut();
                if !lastp.is_null() {
                    (*lastp).next = newp;
                } else {
                    *(*script).punctuationtable.add(idx) = newp;
                }
            }
            i += 1;
        }
    }
}

/// Raven `PS_ReadEscapeCharacter` — decode a `\X` escape at the cursor.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:344-411`
pub fn PS_ReadEscapeCharacter(bot: &mut BotLib, script: *mut script_t, ch: *mut c_char) -> c_int {
    unsafe {
        // step over the leading '\\'
        (*script).script_p = (*script).script_p.offset(1);
        let c: c_int;
        // determine the escape character
        match *(*script).script_p as u8 as char {
            '\\' => c = b'\\' as c_int,
            'n' => c = b'\n' as c_int,
            'r' => c = b'\r' as c_int,
            't' => c = b'\t' as c_int,
            'v' => c = 0x0b,
            'b' => c = 0x08,
            'f' => c = 0x0c,
            'a' => c = 0x07,
            '\'' => c = b'\'' as c_int,
            '\"' => c = b'\"' as c_int,
            '?' => c = b'?' as c_int,
            'x' => {
                (*script).script_p = (*script).script_p.offset(1);
                let mut val: c_int = 0;
                loop {
                    let mut cc = *(*script).script_p as c_int;
                    if cc >= b'0' as c_int && cc <= b'9' as c_int {
                        cc -= b'0' as c_int;
                    } else if cc >= b'A' as c_int && cc <= b'Z' as c_int {
                        cc = cc - b'A' as c_int + 10;
                    } else if cc >= b'a' as c_int && cc <= b'z' as c_int {
                        cc = cc - b'a' as c_int + 10;
                    } else {
                        break;
                    }
                    val = (val << 4) + cc;
                    (*script).script_p = (*script).script_p.offset(1);
                }
                (*script).script_p = (*script).script_p.offset(-1);
                if val > 0xFF {
                    script_warning!(
                        bot,
                        script,
                        c"too large value in escape character".as_ptr() as *mut c_char,
                    );
                    val = 0xFF;
                }
                c = val;
            }
            // NOTE: decimal ASCII code, NOT octal
            _ => {
                if *(*script).script_p < b'0' as c_char || *(*script).script_p > b'9' as c_char {
                    script_error!(bot, script, c"unknown escape char".as_ptr() as *mut c_char);
                }
                let mut val: c_int = 0;
                loop {
                    let cc = *(*script).script_p as c_int;
                    if cc >= b'0' as c_int && cc <= b'9' as c_int {
                        val = val * 10 + (cc - b'0' as c_int);
                    } else {
                        break;
                    }
                    (*script).script_p = (*script).script_p.offset(1);
                }
                (*script).script_p = (*script).script_p.offset(-1);
                if val > 0xFF {
                    script_warning!(
                        bot,
                        script,
                        c"too large value in escape character".as_ptr() as *mut c_char,
                    );
                    val = 0xFF;
                }
                c = val;
            }
        }
        // step over the escape character or the last digit of the number
        (*script).script_p = (*script).script_p.offset(1);
        // store the escape character
        *ch = c as c_char;
        // succesfully read escape character
        1
    }
}

/// Raven `PS_ReadName` — read an identifier token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:511-534`
pub fn PS_ReadName(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut len: usize = 0;
        (*token).r#type = TT_NAME;
        loop {
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
            if len >= MAX_TOKEN {
                script_error!(
                    bot,
                    script,
                    c"name longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                    MAX_TOKEN,
                );
                return 0;
            }
            let c = *(*script).script_p;
            if !((c >= b'a' as c_char && c <= b'z' as c_char)
                || (c >= b'A' as c_char && c <= b'Z' as c_char)
                || (c >= b'0' as c_char && c <= b'9' as c_char)
                || c == b'_' as c_char)
            {
                break;
            }
        }
        (*token).string[len] = 0;
        // the sub type is the length of the name
        (*token).subtype = len as c_int;
        1
    }
}

/// Raven `PS_ReadNumber` — read a number token (hex/decimal/octal/float).
///
/// Source: `oracle/codemp/botlib/l_script.cpp:613-714`
///
/// PORT-NOTE(BINARYNUMBERS/NUMBERVALUE): both guards are unconditionally
/// defined in this tree (`l_script/consts.rs`'s `BINARYNUMBERS`/`NUMBERVALUE`
/// doc comments), so both `#ifdef` arms are live per §C10.
pub fn PS_ReadNumber(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut len: usize = 0;
        let octal;
        let dot;

        (*token).r#type = TT_NUMBER;
        // check for a hexadecimal number
        if *(*script).script_p == b'0' as c_char
            && (*(*script).script_p.offset(1) == b'x' as c_char
                || *(*script).script_p.offset(1) == b'X' as c_char)
        {
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
            let mut c = *(*script).script_p;
            // hexadecimal
            while (c >= b'0' as c_char && c <= b'9' as c_char)
                || (c >= b'a' as c_char && c <= b'f' as c_char)
                || (c >= b'A' as c_char && c <= b'A' as c_char)
            {
                (*token).string[len] = *(*script).script_p;
                (*script).script_p = (*script).script_p.offset(1);
                len += 1;
                if len >= MAX_TOKEN {
                    script_error!(
                        bot,
                        script,
                        c"hexadecimal number longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                        MAX_TOKEN,
                    );
                    return 0;
                }
                c = *(*script).script_p;
            }
            (*token).subtype |= TT_HEX;
        }
        // check for a binary number
        else if *(*script).script_p == b'0' as c_char
            && (*(*script).script_p.offset(1) == b'b' as c_char
                || *(*script).script_p.offset(1) == b'B' as c_char)
        {
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
            let mut c = *(*script).script_p;
            // binary
            while c == b'0' as c_char || c == b'1' as c_char {
                (*token).string[len] = *(*script).script_p;
                (*script).script_p = (*script).script_p.offset(1);
                len += 1;
                if len >= MAX_TOKEN {
                    script_error!(
                        bot,
                        script,
                        c"binary number longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                        MAX_TOKEN,
                    );
                    return 0;
                }
                c = *(*script).script_p;
            }
            (*token).subtype |= TT_BINARY;
        }
        // decimal or octal integer or floating point number
        else {
            octal = *(*script).script_p == b'0' as c_char;
            let mut octal = octal;
            dot = false;
            let mut dot = dot;
            loop {
                let c = *(*script).script_p;
                if c == b'.' as c_char {
                    dot = true;
                } else if c == b'8' as c_char || c == b'9' as c_char {
                    octal = false;
                } else if c < b'0' as c_char || c > b'9' as c_char {
                    break;
                }
                (*token).string[len] = *(*script).script_p;
                (*script).script_p = (*script).script_p.offset(1);
                len += 1;
                if len >= MAX_TOKEN - 1 {
                    script_error!(
                        bot,
                        script,
                        c"number longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                        MAX_TOKEN,
                    );
                    return 0;
                }
            }
            if octal {
                (*token).subtype |= TT_OCTAL;
            } else {
                (*token).subtype |= TT_DECIMAL;
            }
            if dot {
                (*token).subtype |= TT_FLOAT;
            }
        }
        for _ in 0..2 {
            let c = *(*script).script_p;
            // check for a LONG number
            if (c == b'l' as c_char || c == b'L' as c_char) && ((*token).subtype & TT_LONG) == 0 {
                (*script).script_p = (*script).script_p.offset(1);
                (*token).subtype |= TT_LONG;
            }
            // check for an UNSIGNED number
            else if (c == b'u' as c_char || c == b'U' as c_char)
                && ((*token).subtype & (TT_UNSIGNED | TT_FLOAT)) == 0
            {
                (*script).script_p = (*script).script_p.offset(1);
                (*token).subtype |= TT_UNSIGNED;
            }
        }
        (*token).string[len] = 0;
        // PORT-NOTE(long_double-mismatch): `NumberValue`'s resolved signature
        // takes `*mut long_double` (unresolved rosetta type — see
        // missing_symbols); `token_t::floatvalue` is `f64` here. Cast through
        // the pointer per the LAW callee signature; flagged in
        // shape_mismatches.
        NumberValue(
            (*token).string.as_mut_ptr(),
            (*token).subtype,
            &mut (*token).intvalue as *mut u64 as *mut c_ulong,
            &mut (*token).floatvalue as *mut f64 as *mut long_double,
        );
        if (*token).subtype & TT_FLOAT == 0 {
            (*token).subtype |= TT_INTEGER;
        }
        1
    }
}

/// Raven `PS_ReadPrimitive` — read a whitespace/`;`-delimited primitive token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:809-828`
pub fn PS_ReadPrimitive(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut len: usize = 0;
        while *(*script).script_p > b' ' as c_char && *(*script).script_p != b';' as c_char {
            if len >= MAX_TOKEN {
                script_error!(
                    bot,
                    script,
                    c"primitive token longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                    MAX_TOKEN,
                );
                return 0;
            }
            (*token).string[len] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
            len += 1;
        }
        (*token).string[len] = 0;
        // copy the token into the script structure
        Com_Memcpy(
            &mut (*script).token as *mut token_t as *mut (),
            token as *const (),
            core::mem::size_of::<token_t>(),
        );
        // primitive reading successfull
        1
    }
}

/// Raven `PS_UnreadToken` (comment says `UnreadToken`) — push a token back
/// onto the script for the next `PS_ReadToken`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1073-1077`
pub fn PS_UnreadToken(script: *mut script_t, token: *mut token_t) {
    unsafe {
        Com_Memcpy(
            &mut (*script).token as *mut token_t as *mut (),
            token as *const (),
            core::mem::size_of::<token_t>(),
        );
        (*script).tokenavailable = 1;
    }
}

/// Raven `ResetScript` — rewind a script's lexing cursor to the start.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1202-1219`
pub fn ResetScript(script: *mut script_t) {
    unsafe {
        // pointer in script buffer
        (*script).script_p = (*script).buffer;
        // pointer in script buffer before reading token
        (*script).lastscript_p = (*script).buffer;
        // begin of white space
        (*script).whitespace_p = core::ptr::null_mut();
        // end of white space
        (*script).endwhitespace_p = core::ptr::null_mut();
        // set if there's a token available in script->token
        (*script).tokenavailable = 0;
        (*script).line = 1;
        (*script).lastline = 1;
        // clear the saved token
        Com_Memset(
            &mut (*script).token as *mut token_t as *mut (),
            0,
            core::mem::size_of::<token_t>(),
        );
    }
}

/// Raven `ScriptSkipTo` — skip forward until a literal string is found.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1247-1266`
pub fn ScriptSkipTo(script: *mut script_t, value: *mut c_char) -> c_int {
    unsafe {
        let firstchar = *value;
        let len = libc::strlen(value);
        loop {
            if PS_ReadWhiteSpace(script) == 0 {
                return 0;
            }
            if *(*script).script_p == firstchar
                && libc::strncmp((*script).script_p, value, len) == 0
            {
                return 1;
            }
            (*script).script_p = (*script).script_p.offset(1);
        }
    }
}

/// Raven `FreeScript` — free a loaded script and its punctuation table.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1398-1404`
///
/// PORT-NOTE(PUNCTABLE): `#ifdef PUNCTABLE` is not defined in this tree (see
/// `PS_ReadPunctuation`'s note); the punctuationtable free arm is dropped
/// per §C10.
pub fn FreeScript(bot: &mut BotLib, script: *mut script_t) {
    FreeMemory(bot, script as *mut ());
}

/// Raven `SetScriptPunctuations` — install a script's punctuation set
/// (falling back to `default_punctuations`).
///
/// Source: `oracle/codemp/botlib/l_script.cpp:268-276`
///
/// PORT-NOTE(PUNCTABLE): `#ifdef PUNCTABLE` is not defined in this tree (see
/// `PS_ReadPunctuation`'s note) — the `PS_CreatePunctuationTable` calls are
/// dropped per §C10, matching the linear-scan `PS_ReadPunctuation`.
///
/// PORT-NOTE(default_punctuations): `bot.default_punctuations` is the
/// `Engine`-threaded home for the file-scope `default_punctuations[]` table
/// (ruling 2); referenced per the state-receiver convention ahead of its
/// field landing.
pub fn SetScriptPunctuations(bot: &mut BotLib, script: *mut script_t, p: *mut punctuation_t) {
    unsafe {
        if !p.is_null() {
            (*script).punctuations = p;
        } else {
            (*script).punctuations = bot.default_punctuations.as_mut_ptr();
        }
    }
}

/// Raven `PS_ReadString` — read a quoted string/literal token, honoring the
/// script's escape-char and whitespace-between-strings flags.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:422-504`
pub fn PS_ReadString(
    bot: &mut BotLib,
    script: *mut script_t,
    token: *mut token_t,
    quote: c_int,
) -> c_int {
    unsafe {
        if quote == b'\"' as c_int {
            (*token).r#type = TT_STRING;
        } else {
            (*token).r#type = TT_LITERAL;
        }

        let mut len: usize = 0;
        // leading quote
        (*token).string[len] = *(*script).script_p;
        (*script).script_p = (*script).script_p.offset(1);
        len += 1;
        loop {
            // minus 2 because trailing double quote and zero have to be appended
            if len >= MAX_TOKEN - 2 {
                script_error!(
                    bot,
                    script,
                    c"string longer than MAX_TOKEN = %d".as_ptr() as *mut c_char,
                    MAX_TOKEN,
                );
                return 0;
            }
            // if there is an escape character and
            // if escape characters inside a string are allowed
            if *(*script).script_p == b'\\' as c_char
                && (*script).flags & SCFL_NOSTRINGESCAPECHARS == 0
            {
                if PS_ReadEscapeCharacter(bot, script, &mut (*token).string[len]) == 0 {
                    (*token).string[len] = 0;
                    return 0;
                }
                len += 1;
            }
            // if a trailing quote
            else if *(*script).script_p == quote as c_char {
                // step over the double quote
                (*script).script_p = (*script).script_p.offset(1);
                // if white spaces in a string are not allowed
                if (*script).flags & SCFL_NOSTRINGWHITESPACES != 0 {
                    break;
                }
                let tmpscript_p = (*script).script_p;
                let tmpline = (*script).line;
                // read unusefull stuff between possible two following strings
                if PS_ReadWhiteSpace(script) == 0 {
                    (*script).script_p = tmpscript_p;
                    (*script).line = tmpline;
                    break;
                }
                // if there's no leading double qoute
                if *(*script).script_p != quote as c_char {
                    (*script).script_p = tmpscript_p;
                    (*script).line = tmpline;
                    break;
                }
                // step over the new leading double quote
                (*script).script_p = (*script).script_p.offset(1);
            } else {
                if *(*script).script_p == 0 {
                    (*token).string[len] = 0;
                    script_error!(
                        bot,
                        script,
                        c"missing trailing quote".as_ptr() as *mut c_char,
                    );
                    return 0;
                }
                if *(*script).script_p == b'\n' as c_char {
                    (*token).string[len] = 0;
                    script_error!(
                        bot,
                        script,
                        c"newline inside string %s".as_ptr() as *mut c_char,
                        (*token).string.as_ptr(),
                    );
                    return 0;
                }
                (*token).string[len] = *(*script).script_p;
                (*script).script_p = (*script).script_p.offset(1);
                len += 1;
            }
        }
        // trailing quote
        (*token).string[len] = quote as c_char;
        len += 1;
        // end string with a zero
        (*token).string[len] = 0;
        // the sub type is the length of the string
        (*token).subtype = len as c_int;
        1
    }
}

/// Raven `PS_ReadLiteral` — read a `'x'` character-literal token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:721-761`
pub fn PS_ReadLiteral(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        (*token).r#type = TT_LITERAL;
        // first quote
        (*token).string[0] = *(*script).script_p;
        (*script).script_p = (*script).script_p.offset(1);
        // check for end of file
        if *(*script).script_p == 0 {
            script_error!(
                bot,
                script,
                c"end of file before trailing '".as_ptr() as *mut c_char,
            );
            return 0;
        }
        // if it is an escape character
        if *(*script).script_p == b'\\' as c_char {
            if PS_ReadEscapeCharacter(bot, script, &mut (*token).string[1]) == 0 {
                return 0;
            }
        } else {
            (*token).string[1] = *(*script).script_p;
            (*script).script_p = (*script).script_p.offset(1);
        }
        // check for trailing quote
        if *(*script).script_p != b'\'' as c_char {
            script_warning!(
                bot,
                script,
                c"too many characters in literal, ignored".as_ptr() as *mut c_char,
            );
            while *(*script).script_p != 0
                && *(*script).script_p != b'\'' as c_char
                && *(*script).script_p != b'\n' as c_char
            {
                (*script).script_p = (*script).script_p.offset(1);
            }
            if *(*script).script_p == b'\'' as c_char {
                (*script).script_p = (*script).script_p.offset(1);
            }
        }
        // store the trailing quote
        (*token).string[2] = *(*script).script_p;
        (*script).script_p = (*script).script_p.offset(1);
        // store trailing zero to end the string
        (*token).string[3] = 0;
        // the sub type is the integer literal value
        (*token).subtype = (*token).string[1] as c_int;
        1
    }
}

/// Raven `PS_ReadToken` — read the next token, dispatching by lookahead
/// character to string/number/name/primitive/punctuation readers.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:835-902`
///
/// PORT-NOTE(PS_ReadLiteral): Raven's own commented-out
/// `PS_ReadLiteral(script, token)` call (l_script.cpp:870) is preserved as a
/// comment; the live call is `PS_ReadString(script, token, '\'')`.
pub fn PS_ReadToken(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    unsafe {
        // if there is a token available (from UnreadToken)
        if (*script).tokenavailable != 0 {
            (*script).tokenavailable = 0;
            Com_Memcpy(
                token as *mut (),
                &(*script).token as *const token_t as *const (),
                core::mem::size_of::<token_t>(),
            );
            return 1;
        }
        // save script pointer
        (*script).lastscript_p = (*script).script_p;
        // save line counter
        (*script).lastline = (*script).line;
        // clear the token stuff
        Com_Memset(token as *mut (), 0, core::mem::size_of::<token_t>());
        // start of the white space
        (*script).whitespace_p = (*script).script_p;
        (*token).whitespace_p = (*script).script_p;
        // read unusefull stuff
        if PS_ReadWhiteSpace(script) == 0 {
            return 0;
        }
        // end of the white space
        (*script).endwhitespace_p = (*script).script_p;
        (*token).endwhitespace_p = (*script).script_p;
        // line the token is on
        (*token).line = (*script).line;
        // number of lines crossed before token
        (*token).linescrossed = (*script).line - (*script).lastline;
        // if there is a leading double quote
        if *(*script).script_p == b'\"' as c_char {
            if PS_ReadString(bot, script, token, b'\"' as c_int) == 0 {
                return 0;
            }
        }
        // if an literal
        else if *(*script).script_p == b'\'' as c_char {
            // if (!PS_ReadLiteral(script, token)) return 0;
            if PS_ReadString(bot, script, token, b'\'' as c_int) == 0 {
                return 0;
            }
        }
        // if there is a number
        else if (*(*script).script_p >= b'0' as c_char && *(*script).script_p <= b'9' as c_char)
            || (*(*script).script_p == b'.' as c_char
                && (*(*script).script_p.offset(1) >= b'0' as c_char
                    && *(*script).script_p.offset(1) <= b'9' as c_char))
        {
            if PS_ReadNumber(bot, script, token) == 0 {
                return 0;
            }
        }
        // if this is a primitive script
        else if (*script).flags & SCFL_PRIMITIVE != 0 {
            return PS_ReadPrimitive(bot, script, token);
        }
        // if there is a name
        else if (*(*script).script_p >= b'a' as c_char && *(*script).script_p <= b'z' as c_char)
            || (*(*script).script_p >= b'A' as c_char && *(*script).script_p <= b'Z' as c_char)
            || *(*script).script_p == b'_' as c_char
            || *(*script).script_p == b'@' as c_char
        {
            if PS_ReadName(bot, script, token) == 0 {
                return 0;
            }
        }
        // check for punctuations
        else if PS_ReadPunctuation(script, token) == 0 {
            script_error!(bot, script, c"can't read token".as_ptr() as *mut c_char);
            return 0;
        }
        // copy the token into the script structure
        Com_Memcpy(
            &mut (*script).token as *mut token_t as *mut (),
            token as *const (),
            core::mem::size_of::<token_t>(),
        );
        // succesfully read a token
        1
    }
}

/// Raven `LoadScriptFile` — load a script file from disk via `botimport.FS_*`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1293-1355`
///
/// PORT-NOTE(BOTLIB): `#ifdef BOTLIB` is defined in this tree (see
/// `PC_SetBaseFolder`'s precedent) — the `botimport.FS_*` arm is transcribed,
/// the standalone-`fopen` `#else` arm is dropped per §C10.
pub fn LoadScriptFile(bot: &mut BotLib, filename: *const c_char) -> *mut script_t {
    unsafe {
        let mut pathname = [0 as c_char; MAX_QPATH];
        let mut fp: fileHandle_t = 0;
        let length: c_int;

        let filename_str = core::ffi::CStr::from_ptr(filename).to_string_lossy();
        if libc::strlen(bot.basefolder.as_ptr()) != 0 {
            let basefolder_str =
                core::ffi::CStr::from_ptr(bot.basefolder.as_ptr()).to_string_lossy();
            let __s = std::ffi::CString::new(format!("{}/{}", basefolder_str, filename_str))
                .unwrap_or_default();
            Q_strncpyz(pathname.as_mut_ptr(), __s.as_ptr(), MAX_QPATH as c_int);
        } else {
            let __s = std::ffi::CString::new(filename_str.to_string()).unwrap_or_default();
            Q_strncpyz(pathname.as_mut_ptr(), __s.as_ptr(), MAX_QPATH as c_int);
        }
        length = (bot.botimport.FS_FOpenFile.unwrap())(pathname.as_ptr(), &mut fp, FS_READ);
        if fp == 0 {
            return core::ptr::null_mut();
        }

        let buffer = GetClearedMemory(
            bot,
            (core::mem::size_of::<script_t>() + length as usize + 1) as c_ulong,
        );
        let script = buffer as *mut script_t;
        Com_Memset(script as *mut (), 0, core::mem::size_of::<script_t>());
        libc::strcpy((*script).filename.as_mut_ptr(), filename);
        (*script).buffer = (buffer as *mut c_char).add(core::mem::size_of::<script_t>());
        *(*script).buffer.offset(length as isize) = 0;
        (*script).length = length;
        // pointer in script buffer
        (*script).script_p = (*script).buffer;
        // pointer in script buffer before reading token
        (*script).lastscript_p = (*script).buffer;
        // pointer to end of script buffer
        (*script).end_p = (*script).buffer.offset(length as isize);
        // set if there's a token available in script->token
        (*script).tokenavailable = 0;
        (*script).line = 1;
        (*script).lastline = 1;
        SetScriptPunctuations(bot, script, core::ptr::null_mut());

        (bot.botimport.FS_Read.unwrap())((*script).buffer as *mut c_void, length, fp);
        (bot.botimport.FS_FCloseFile.unwrap())(fp);

        (*script).length = COM_Compress((*script).buffer);

        script
    }
}

/// Raven `LoadScriptMemory` — load a script from an in-memory buffer.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1362-1391`
pub fn LoadScriptMemory(
    bot: &mut BotLib,
    ptr: *mut c_char,
    length: c_int,
    name: *mut c_char,
) -> *mut script_t {
    unsafe {
        let buffer = GetClearedMemory(
            bot,
            (core::mem::size_of::<script_t>() + length as usize + 1) as c_ulong,
        );
        let script = buffer as *mut script_t;
        Com_Memset(script as *mut (), 0, core::mem::size_of::<script_t>());
        libc::strcpy((*script).filename.as_mut_ptr(), name);
        (*script).buffer = (buffer as *mut c_char).add(core::mem::size_of::<script_t>());
        *(*script).buffer.offset(length as isize) = 0;
        (*script).length = length;
        // pointer in script buffer
        (*script).script_p = (*script).buffer;
        // pointer in script buffer before reading token
        (*script).lastscript_p = (*script).buffer;
        // pointer to end of script buffer
        (*script).end_p = (*script).buffer.offset(length as isize);
        // set if there's a token available in script->token
        (*script).tokenavailable = 0;
        (*script).line = 1;
        (*script).lastline = 1;
        SetScriptPunctuations(bot, script, core::ptr::null_mut());
        Com_Memcpy(
            (*script).buffer as *mut (),
            ptr as *const (),
            length as usize,
        );
        script
    }
}

/// Raven `PS_ExpectTokenString` — read a token and require it to equal a
/// literal string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:909-925`
pub fn PS_ExpectTokenString(bot: &mut BotLib, script: *mut script_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut token = core::mem::zeroed::<token_t>();

        if PS_ReadToken(bot, script, &mut token) == 0 {
            script_error!(
                bot,
                script,
                c"couldn't find expected %s".as_ptr() as *mut c_char,
                string,
            );
            return 0;
        }

        if libc::strcmp(token.string.as_ptr(), string) != 0 {
            script_error!(
                bot,
                script,
                c"expected %s, found %s".as_ptr() as *mut c_char,
                string,
                token.string.as_ptr(),
            );
            return 0;
        }
        1
    }
}

/// Raven `PS_ExpectTokenType` (comment says `PS_ExpectToken`) — read a token
/// and require it to match a type/subtype.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:932-983`
pub fn PS_ExpectTokenType(
    bot: &mut BotLib,
    script: *mut script_t,
    r#type: c_int,
    subtype: c_int,
    token: *mut token_t,
) -> c_int {
    unsafe {
        let mut str = [0 as c_char; MAX_TOKEN];

        if PS_ReadToken(bot, script, token) == 0 {
            script_error!(
                bot,
                script,
                c"couldn't read expected token".as_ptr() as *mut c_char,
            );
            return 0;
        }

        if (*token).r#type != r#type {
            if r#type == TT_STRING {
                libc::strcpy(str.as_mut_ptr(), c"string".as_ptr());
            }
            if r#type == TT_LITERAL {
                libc::strcpy(str.as_mut_ptr(), c"literal".as_ptr());
            }
            if r#type == TT_NUMBER {
                libc::strcpy(str.as_mut_ptr(), c"number".as_ptr());
            }
            if r#type == TT_NAME {
                libc::strcpy(str.as_mut_ptr(), c"name".as_ptr());
            }
            if r#type == TT_PUNCTUATION {
                libc::strcpy(str.as_mut_ptr(), c"punctuation".as_ptr());
            }
            script_error!(
                bot,
                script,
                c"expected a %s, found %s".as_ptr() as *mut c_char,
                str.as_ptr(),
                (*token).string.as_ptr(),
            );
            return 0;
        }
        if (*token).r#type == TT_NUMBER {
            if ((*token).subtype & subtype) != subtype {
                if subtype & TT_DECIMAL != 0 {
                    libc::strcpy(str.as_mut_ptr(), c"decimal".as_ptr());
                }
                if subtype & TT_HEX != 0 {
                    libc::strcpy(str.as_mut_ptr(), c"hex".as_ptr());
                }
                if subtype & TT_OCTAL != 0 {
                    libc::strcpy(str.as_mut_ptr(), c"octal".as_ptr());
                }
                if subtype & TT_BINARY != 0 {
                    libc::strcpy(str.as_mut_ptr(), c"binary".as_ptr());
                }
                if subtype & TT_LONG != 0 {
                    libc::strcat(str.as_mut_ptr(), c" long".as_ptr());
                }
                if subtype & TT_UNSIGNED != 0 {
                    libc::strcat(str.as_mut_ptr(), c" unsigned".as_ptr());
                }
                if subtype & TT_FLOAT != 0 {
                    libc::strcat(str.as_mut_ptr(), c" float".as_ptr());
                }
                if subtype & TT_INTEGER != 0 {
                    libc::strcat(str.as_mut_ptr(), c" integer".as_ptr());
                }
                script_error!(
                    bot,
                    script,
                    c"expected %s, found %s".as_ptr() as *mut c_char,
                    str.as_ptr(),
                    (*token).string.as_ptr(),
                );
                return 0;
            }
        } else if (*token).r#type == TT_PUNCTUATION {
            if subtype < 0 {
                script_error!(
                    bot,
                    script,
                    c"BUG: wrong punctuation subtype".as_ptr() as *mut c_char,
                );
                return 0;
            }
            if (*token).subtype != subtype {
                script_error!(
                    bot,
                    script,
                    c"expected %s, found %s".as_ptr() as *mut c_char,
                    PunctuationFromNum(script, subtype),
                    (*token).string.as_ptr(),
                );
                return 0;
            }
        }
        1
    }
}

/// Raven `PS_ExpectAnyToken` — read a token, requiring only that one exists.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:990-1001`
pub fn PS_ExpectAnyToken(bot: &mut BotLib, script: *mut script_t, token: *mut token_t) -> c_int {
    if PS_ReadToken(bot, script, token) == 0 {
        unsafe {
            script_error!(
                bot,
                script,
                c"couldn't read expected token".as_ptr() as *mut c_char,
            )
        };
        0
    } else {
        1
    }
}

/// Raven `PS_CheckTokenString` — peek a token, consuming it only if it
/// matches a literal string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1008-1018`
pub fn PS_CheckTokenString(bot: &mut BotLib, script: *mut script_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut tok = core::mem::zeroed::<token_t>();

        if PS_ReadToken(bot, script, &mut tok) == 0 {
            return 0;
        }
        // if the token is available
        if libc::strcmp(tok.string.as_ptr(), string) == 0 {
            return 1;
        }
        // token not available
        (*script).script_p = (*script).lastscript_p;
        0
    }
}

/// Raven `PS_CheckTokenType` — peek a token, consuming it only if it matches
/// a type/subtype.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1025-1040`
pub fn PS_CheckTokenType(
    bot: &mut BotLib,
    script: *mut script_t,
    r#type: c_int,
    subtype: c_int,
    token: *mut token_t,
) -> c_int {
    unsafe {
        let mut tok = core::mem::zeroed::<token_t>();

        if PS_ReadToken(bot, script, &mut tok) == 0 {
            return 0;
        }
        // if the type matches
        if tok.r#type == r#type && (tok.subtype & subtype) == subtype {
            Com_Memcpy(
                token as *mut (),
                &tok as *const token_t as *const (),
                core::mem::size_of::<token_t>(),
            );
            return 1;
        }
        // token is not available
        (*script).script_p = (*script).lastscript_p;
        0
    }
}

/// Raven `PS_SkipUntilString` — read tokens until one matches a literal
/// string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1047-1056`
pub fn PS_SkipUntilString(bot: &mut BotLib, script: *mut script_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut token = core::mem::zeroed::<token_t>();
        while PS_ReadToken(bot, script, &mut token) != 0 {
            if libc::strcmp(token.string.as_ptr(), string) == 0 {
                return 1;
            }
        }
        0
    }
}

/// Raven `ReadSignedFloat` — read an optionally `-`-signed float value token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1136-1152`
///
/// PORT-NOTE(long_double): return type is Raven's unresolved `long double`
/// (see missing_symbols); `token.floatvalue` is `f64` here, cast at the
/// return per the LAW resolved signature.
pub fn ReadSignedFloat(bot: &mut BotLib, script: *mut script_t) -> long_double {
    unsafe {
        let mut token = core::mem::zeroed::<token_t>();
        let mut sign: long_double = 1.0;

        PS_ExpectAnyToken(bot, script, &mut token);
        if libc::strcmp(token.string.as_ptr(), c"-".as_ptr()) == 0 {
            sign = -1.0;
            PS_ExpectTokenType(bot, script, TT_NUMBER, 0, &mut token);
        } else if token.r#type != TT_NUMBER {
            script_error!(
                bot,
                script,
                c"expected float value, found %s\n".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
        }
        sign * token.floatvalue as long_double
    }
}

/// Raven `ReadSignedInt` — read an optionally `-`-signed integer value token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1159-1175`
pub fn ReadSignedInt(bot: &mut BotLib, script: *mut script_t) -> c_long {
    unsafe {
        let mut token = core::mem::zeroed::<token_t>();
        let mut sign: c_long = 1;

        PS_ExpectAnyToken(bot, script, &mut token);
        if libc::strcmp(token.string.as_ptr(), c"-".as_ptr()) == 0 {
            sign = -1;
            PS_ExpectTokenType(bot, script, TT_NUMBER, TT_INTEGER, &mut token);
        } else if token.r#type != TT_NUMBER || token.subtype == TT_FLOAT {
            script_error!(
                bot,
                script,
                c"expected integer value, found %s\n".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
        }
        sign * token.intvalue as c_long
    }
}
