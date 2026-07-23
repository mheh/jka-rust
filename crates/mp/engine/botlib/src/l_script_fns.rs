#![allow(non_camel_case_types, non_snake_case)]
//! `l_script.cpp` — the botlib script tokenizer (function bodies).
//!
//! One Rust module per oracle source file (`l_script.cpp`); the stem collides
//! with the `l_script/` type directory, so this lands as `l_script_fns.rs`.
//!
//! Idiomatic redesign (porting-rules §F17): the tokenizer runs on the owned
//! `Script` (a `Vec<u8>` buffer plus `usize` cursors) and `Token` (a `String`)
//! shapes, not Raven's malloc'd `script_t`/`token_t`. A few translation
//! conventions:
//!
//! - **Signed-`char` fidelity.** Raven reads each byte into a (platform-signed)
//!   `char`; the `*script_p <= ' '` / `> ' '` tests therefore treat bytes
//!   `>= 0x80` as whitespace. Every such read casts `buffer[i] as c_char` so
//!   the signed comparison matches the oracle exactly.
//! - **`String` token bodies.** Readers build `token.string` by pushing one
//!   source byte per Raven `char`, tracking `len` explicitly so the `MAX_TOKEN`
//!   truncation fires at the same spot. Bytes are ASCII in practice (config
//!   files, chat templates); a source byte `>= 0x80` becomes the same Unicode
//!   scalar value.
//! - **Errors carry text, not a C format.** `ScriptError`/`ScriptWarning` take a
//!   pre-formatted `&str` and hand the finished `"file …, line …: …\n"` line to
//!   `botimport.Print` as a single `%s` argument, reproducing the oracle text.
//! - **`PS_CreatePunctuationTable` is dropped** — the `PUNCTABLE` first-char
//!   bucket table is never built here; `PS_ReadPunctuation` scans the
//!   length-ordered slice linearly (matching the oracle's non-`PUNCTABLE`
//!   arm).
//!
//! Source: `oracle/codemp/botlib/l_script.cpp`

use core::ffi::{c_char, c_int, c_long, c_void};
use std::ffi::{CStr, CString};

use crate::l_script::consts::{
    MAX_TOKEN, SCFL_NOERRORS, SCFL_NOSTRINGESCAPECHARS, SCFL_NOSTRINGWHITESPACES, SCFL_NOWARNINGS,
    SCFL_PRIMITIVE, TT_BINARY, TT_DECIMAL, TT_FLOAT, TT_HEX, TT_INTEGER, TT_LITERAL, TT_LONG,
    TT_NAME, TT_NUMBER, TT_OCTAL, TT_PUNCTUATION, TT_STRING, TT_UNSIGNED,
};
use crate::l_script::punctuation_s::Punctuation;
use crate::l_script::script_s::Script;
use crate::l_script::token_s::Token;
use crate::{BotLib, DEFAULT_PUNCTUATIONS};

use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_WARNING};
use mp_qshared::shared::q_string::{COM_Compress, Q_strncpyz};
use mp_qshared::shared::{fileHandle_t, FS_READ, MAX_QPATH};

/// Raven `PunctuationFromNum` — look up a punctuation's text by its number.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:200-209`
pub fn PunctuationFromNum(script: &Script, num: c_int) -> &'static str {
    for punc in script.punctuations {
        if punc.n == num {
            return punc.p;
        }
    }
    "unkown punctuation"
}

/// Raven `ScriptError` — print a tokenizer error tagged with the current
/// script file and line.
///
/// The oracle `vsprintf`s the message, then `Print`s `"file %s, line %d: %s\n"`;
/// the pre-formatted `text` is composed here and handed to `Print` as one `%s`
/// argument, yielding byte-identical output.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:216-235`
pub fn ScriptError(bot: &mut BotLib, script: &Script, text: &str) {
    if script.flags & SCFL_NOERRORS != 0 {
        return;
    }
    // #ifdef BOTLIB (defined)
    let msg = CString::new(format!(
        "file {}, line {}: {}\n",
        script.filename, script.line, text
    ))
    .unwrap_or_default();
    unsafe {
        (bot.botimport.Print.unwrap())(PRT_ERROR, c"%s".as_ptr() as *mut c_char, msg.as_ptr());
    }
}

/// Raven `ScriptWarning` — print a tokenizer warning tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:242-261`
pub fn ScriptWarning(bot: &mut BotLib, script: &Script, text: &str) {
    if script.flags & SCFL_NOWARNINGS != 0 {
        return;
    }
    // #ifdef BOTLIB (defined)
    let msg = CString::new(format!(
        "file {}, line {}: {}\n",
        script.filename, script.line, text
    ))
    .unwrap_or_default();
    unsafe {
        (bot.botimport.Print.unwrap())(PRT_WARNING, c"%s".as_ptr() as *mut c_char, msg.as_ptr());
    }
}

/// Raven `SetScriptPunctuations` — install a script's punctuation set
/// (falling back to `default_punctuations`).
///
/// The `#ifdef PUNCTABLE` `PS_CreatePunctuationTable` calls are dropped (the
/// bucket table is never built here); the linear-scan `PS_ReadPunctuation` is
/// the only lookup path.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:268-276`
pub fn SetScriptPunctuations(bot: &BotLib, script: &mut Script, p: Option<&'static [Punctuation]>) {
    match p {
        Some(p) => script.punctuations = p,
        None => script.punctuations = bot.default_punctuations,
    }
}

/// Raven `PS_ReadWhiteSpace` — skip whitespace and `//`/`/* */` comments.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:285-335`
pub fn PS_ReadWhiteSpace(script: &mut Script) -> c_int {
    loop {
        // skip white space
        while (script.buffer[script.script_p] as c_char) <= b' ' as c_char {
            if script.buffer[script.script_p] == 0 {
                return 0;
            }
            if script.buffer[script.script_p] == b'\n' {
                script.line += 1;
            }
            script.script_p += 1;
        }
        // skip comments
        if script.buffer[script.script_p] == b'/' {
            // comments //
            if script.buffer[script.script_p + 1] == b'/' {
                script.script_p += 1;
                loop {
                    script.script_p += 1;
                    if script.buffer[script.script_p] == 0 {
                        return 0;
                    }
                    if script.buffer[script.script_p] == b'\n' {
                        break;
                    }
                }
                script.line += 1;
                script.script_p += 1;
                if script.buffer[script.script_p] == 0 {
                    return 0;
                }
                continue;
            }
            // comments /* */
            else if script.buffer[script.script_p + 1] == b'*' {
                script.script_p += 1;
                loop {
                    script.script_p += 1;
                    if script.buffer[script.script_p] == 0 {
                        return 0;
                    }
                    if script.buffer[script.script_p] == b'\n' {
                        script.line += 1;
                    }
                    if script.buffer[script.script_p] == b'*'
                        && script.buffer[script.script_p + 1] == b'/'
                    {
                        break;
                    }
                }
                script.script_p += 1;
                if script.buffer[script.script_p] == 0 {
                    return 0;
                }
                script.script_p += 1;
                if script.buffer[script.script_p] == 0 {
                    return 0;
                }
                continue;
            }
        }
        break;
    }
    1
}

/// Raven `PS_ReadEscapeCharacter` — decode a `\X` escape at the cursor.
///
/// Out-param `char *ch` (§C7) becomes the return value; Raven always returns
/// success (`1`), so the `None` arm is unreachable but preserved for structural
/// parity with the callers' failure checks.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:344-411`
pub fn PS_ReadEscapeCharacter(bot: &mut BotLib, script: &mut Script) -> Option<c_char> {
    // step over the leading '\\'
    script.script_p += 1;
    // determine the escape character
    let c: c_int = match script.buffer[script.script_p] as char {
        '\\' => b'\\' as c_int,
        'n' => b'\n' as c_int,
        'r' => b'\r' as c_int,
        't' => b'\t' as c_int,
        'v' => 0x0b,
        'b' => 0x08,
        'f' => 0x0c,
        'a' => 0x07,
        '\'' => b'\'' as c_int,
        '\"' => b'\"' as c_int,
        '?' => b'?' as c_int,
        'x' => {
            script.script_p += 1;
            let mut val: c_int = 0;
            loop {
                let mut cc = script.buffer[script.script_p] as c_char as c_int;
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
                script.script_p += 1;
            }
            script.script_p -= 1;
            if val > 0xFF {
                ScriptWarning(bot, script, "too large value in escape character");
                val = 0xFF;
            }
            val
        }
        // NOTE: decimal ASCII code, NOT octal
        _ => {
            let ch0 = script.buffer[script.script_p] as c_char;
            if ch0 < b'0' as c_char || ch0 > b'9' as c_char {
                ScriptError(bot, script, "unknown escape char");
            }
            let mut val: c_int = 0;
            loop {
                let cc = script.buffer[script.script_p] as c_char as c_int;
                if cc >= b'0' as c_int && cc <= b'9' as c_int {
                    val = val * 10 + (cc - b'0' as c_int);
                } else {
                    break;
                }
                script.script_p += 1;
            }
            script.script_p -= 1;
            if val > 0xFF {
                ScriptWarning(bot, script, "too large value in escape character");
                val = 0xFF;
            }
            val
        }
    };
    // step over the escape character or the last digit of the number
    script.script_p += 1;
    // succesfully read escape character
    Some(c as c_char)
}

/// Raven `PS_ReadString` — read a quoted string/literal token, honoring the
/// script's escape-char and whitespace-between-strings flags.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:422-504`
pub fn PS_ReadString(bot: &mut BotLib, script: &mut Script, token: &mut Token, quote: c_int) -> c_int {
    if quote == b'\"' as c_int {
        token.type_ = TT_STRING;
    } else {
        token.type_ = TT_LITERAL;
    }

    let mut len: usize = 0;
    // leading quote
    token.string.push(script.buffer[script.script_p] as char);
    script.script_p += 1;
    len += 1;
    loop {
        // minus 2 because trailing double quote and zero have to be appended
        if len >= MAX_TOKEN - 2 {
            ScriptError(
                bot,
                script,
                &format!("string longer than MAX_TOKEN = {}", MAX_TOKEN),
            );
            return 0;
        }
        // if there is an escape character and
        // if escape characters inside a string are allowed
        if script.buffer[script.script_p] == b'\\' && script.flags & SCFL_NOSTRINGESCAPECHARS == 0 {
            match PS_ReadEscapeCharacter(bot, script) {
                Some(ch) => {
                    token.string.push(ch as u8 as char);
                    len += 1;
                }
                None => return 0,
            }
        }
        // if a trailing quote
        else if script.buffer[script.script_p] == quote as u8 {
            // step over the double quote
            script.script_p += 1;
            // if white spaces in a string are not allowed
            if script.flags & SCFL_NOSTRINGWHITESPACES != 0 {
                break;
            }
            let tmpscript_p = script.script_p;
            let tmpline = script.line;
            // read unusefull stuff between possible two following strings
            if PS_ReadWhiteSpace(script) == 0 {
                script.script_p = tmpscript_p;
                script.line = tmpline;
                break;
            }
            // if there's no leading double qoute
            if script.buffer[script.script_p] != quote as u8 {
                script.script_p = tmpscript_p;
                script.line = tmpline;
                break;
            }
            // step over the new leading double quote
            script.script_p += 1;
        } else {
            if script.buffer[script.script_p] == 0 {
                ScriptError(bot, script, "missing trailing quote");
                return 0;
            }
            if script.buffer[script.script_p] == b'\n' {
                ScriptError(
                    bot,
                    script,
                    &format!("newline inside string {}", token.string),
                );
                return 0;
            }
            token.string.push(script.buffer[script.script_p] as char);
            script.script_p += 1;
            len += 1;
        }
    }
    // trailing quote
    token.string.push(quote as u8 as char);
    len += 1;
    // the sub type is the length of the string
    token.subtype = len as c_int;
    1
}

/// Raven `PS_ReadName` — read an identifier token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:511-534`
pub fn PS_ReadName(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    let mut len: usize = 0;
    token.type_ = TT_NAME;
    loop {
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
        if len >= MAX_TOKEN {
            ScriptError(
                bot,
                script,
                &format!("name longer than MAX_TOKEN = {}", MAX_TOKEN),
            );
            return 0;
        }
        let c = script.buffer[script.script_p] as c_char;
        if !((c >= b'a' as c_char && c <= b'z' as c_char)
            || (c >= b'A' as c_char && c <= b'Z' as c_char)
            || (c >= b'0' as c_char && c <= b'9' as c_char)
            || c == b'_' as c_char)
        {
            break;
        }
    }
    // the sub type is the length of the name
    token.subtype = len as c_int;
    1
}

/// Raven `NumberValue` — parse a decoded token's decimal/int/float value out
/// of its string form.
///
/// Out-params `intvalue`/`floatvalue` (§C7) become the returned tuple.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:541-606`
fn NumberValue(string: &str, subtype: c_int) -> (u64, f64) {
    let bytes = string.as_bytes();
    let mut intvalue: u64 = 0;
    let mut floatvalue: f64 = 0.0;
    let mut dotfound: u64 = 0;

    // floating point number
    if subtype & TT_FLOAT != 0 {
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'.' {
                if dotfound != 0 {
                    return (intvalue, floatvalue);
                }
                dotfound = 10;
                i += 1;
            }
            // Raven reads `*string` unconditionally after the `.` step; a
            // trailing `.` would run it past the terminator (UB). Guard it
            // (porting-rules §F19) — a real number token ("0.7", "800", …)
            // never ends on a dot.
            if i >= bytes.len() {
                break;
            }
            // Raven promotes `char` to `int` in these subtractions (a second
            // `.` yields a negative contribution); cast to `c_int` to match,
            // which also avoids a `u8` underflow panic.
            if dotfound != 0 {
                floatvalue += (bytes[i] as c_int - b'0' as c_int) as f64 / dotfound as f64;
                dotfound *= 10;
            } else {
                floatvalue = floatvalue * 10.0 + (bytes[i] as c_int - b'0' as c_int) as f64;
            }
            i += 1;
        }
        intvalue = floatvalue as u64;
    } else if subtype & TT_DECIMAL != 0 {
        for &b in bytes {
            intvalue = intvalue * 10 + (b as c_int - b'0' as c_int) as u64;
        }
        floatvalue = intvalue as f64;
    } else if subtype & TT_HEX != 0 {
        // step over the leading 0x or 0X
        let mut i = 2;
        while i < bytes.len() {
            intvalue <<= 4;
            let b = bytes[i];
            if b >= b'a' && b <= b'f' {
                intvalue += (b as c_int - b'a' as c_int + 10) as u64;
            } else if b >= b'A' && b <= b'F' {
                intvalue += (b as c_int - b'A' as c_int + 10) as u64;
            } else {
                intvalue += (b as c_int - b'0' as c_int) as u64;
            }
            i += 1;
        }
        floatvalue = intvalue as f64;
    } else if subtype & TT_OCTAL != 0 {
        // step over the first zero
        for &b in &bytes[1..] {
            intvalue = (intvalue << 3) + (b as c_int - b'0' as c_int) as u64;
        }
        floatvalue = intvalue as f64;
    } else if subtype & TT_BINARY != 0 {
        // step over the leading 0b or 0B
        for &b in &bytes[2..] {
            intvalue = (intvalue << 1) + (b as c_int - b'0' as c_int) as u64;
        }
        floatvalue = intvalue as f64;
    }
    (intvalue, floatvalue)
}

/// Raven `PS_ReadNumber` — read a number token (hex/decimal/octal/float).
///
/// Source: `oracle/codemp/botlib/l_script.cpp:613-714`
pub fn PS_ReadNumber(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    let mut len: usize = 0;

    token.type_ = TT_NUMBER;
    // check for a hexadecimal number
    if script.buffer[script.script_p] == b'0'
        && (script.buffer[script.script_p + 1] == b'x' || script.buffer[script.script_p + 1] == b'X')
    {
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
        let mut c = script.buffer[script.script_p] as c_char;
        // hexadecimal
        while (c >= b'0' as c_char && c <= b'9' as c_char)
            || (c >= b'a' as c_char && c <= b'f' as c_char)
            || (c >= b'A' as c_char && c <= b'A' as c_char)
        {
            token.string.push(script.buffer[script.script_p] as char);
            script.script_p += 1;
            len += 1;
            if len >= MAX_TOKEN {
                ScriptError(
                    bot,
                    script,
                    &format!("hexadecimal number longer than MAX_TOKEN = {}", MAX_TOKEN),
                );
                return 0;
            }
            c = script.buffer[script.script_p] as c_char;
        }
        token.subtype |= TT_HEX;
    }
    // check for a binary number
    else if script.buffer[script.script_p] == b'0'
        && (script.buffer[script.script_p + 1] == b'b' || script.buffer[script.script_p + 1] == b'B')
    {
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
        let mut c = script.buffer[script.script_p] as c_char;
        // binary
        while c == b'0' as c_char || c == b'1' as c_char {
            token.string.push(script.buffer[script.script_p] as char);
            script.script_p += 1;
            len += 1;
            if len >= MAX_TOKEN {
                ScriptError(
                    bot,
                    script,
                    &format!("binary number longer than MAX_TOKEN = {}", MAX_TOKEN),
                );
                return 0;
            }
            c = script.buffer[script.script_p] as c_char;
        }
        token.subtype |= TT_BINARY;
    }
    // decimal or octal integer or floating point number
    else {
        let mut octal = script.buffer[script.script_p] == b'0';
        let mut dot = false;
        loop {
            let c = script.buffer[script.script_p] as c_char;
            if c == b'.' as c_char {
                dot = true;
            } else if c == b'8' as c_char || c == b'9' as c_char {
                octal = false;
            } else if c < b'0' as c_char || c > b'9' as c_char {
                break;
            }
            token.string.push(script.buffer[script.script_p] as char);
            script.script_p += 1;
            len += 1;
            if len >= MAX_TOKEN - 1 {
                ScriptError(
                    bot,
                    script,
                    &format!("number longer than MAX_TOKEN = {}", MAX_TOKEN),
                );
                return 0;
            }
        }
        if octal {
            token.subtype |= TT_OCTAL;
        } else {
            token.subtype |= TT_DECIMAL;
        }
        if dot {
            token.subtype |= TT_FLOAT;
        }
    }
    for _ in 0..2 {
        let c = script.buffer[script.script_p] as c_char;
        // check for a LONG number
        if (c == b'l' as c_char || c == b'L' as c_char) && (token.subtype & TT_LONG) == 0 {
            script.script_p += 1;
            token.subtype |= TT_LONG;
        }
        // check for an UNSIGNED number
        else if (c == b'u' as c_char || c == b'U' as c_char)
            && (token.subtype & (TT_UNSIGNED | TT_FLOAT)) == 0
        {
            script.script_p += 1;
            token.subtype |= TT_UNSIGNED;
        }
    }
    let (intvalue, floatvalue) = NumberValue(&token.string, token.subtype);
    token.intvalue = intvalue;
    token.floatvalue = floatvalue;
    if token.subtype & TT_FLOAT == 0 {
        token.subtype |= TT_INTEGER;
    }
    1
}

/// Raven `PS_ReadLiteral` — read a `'x'` character-literal token.
///
/// Dead in this build (`PS_ReadToken` routes literals through `PS_ReadString`,
/// preserving the oracle's commented-out call), but ported for parity.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:721-761`
pub fn PS_ReadLiteral(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    token.type_ = TT_LITERAL;
    // first quote
    token.string.push(script.buffer[script.script_p] as char);
    script.script_p += 1;
    // check for end of file
    if script.buffer[script.script_p] == 0 {
        ScriptError(bot, script, "end of file before trailing '");
        return 0;
    }
    // if it is an escape character
    let lit: c_char;
    if script.buffer[script.script_p] == b'\\' {
        match PS_ReadEscapeCharacter(bot, script) {
            Some(ch) => {
                lit = ch;
                token.string.push(ch as u8 as char);
            }
            None => return 0,
        }
    } else {
        lit = script.buffer[script.script_p] as c_char;
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
    }
    // check for trailing quote
    if script.buffer[script.script_p] != b'\'' {
        ScriptWarning(bot, script, "too many characters in literal, ignored");
        while script.buffer[script.script_p] != 0
            && script.buffer[script.script_p] != b'\''
            && script.buffer[script.script_p] != b'\n'
        {
            script.script_p += 1;
        }
        if script.buffer[script.script_p] == b'\'' {
            script.script_p += 1;
        }
    }
    // store the trailing quote
    token.string.push(script.buffer[script.script_p] as char);
    script.script_p += 1;
    // the sub type is the integer literal value
    token.subtype = lit as c_int;
    1
}

/// Raven `PS_ReadPunctuation` — match the longest punctuation at the cursor.
///
/// `PUNCTABLE` is defined in the oracle build, but its first-char-bucketed
/// lookup returns the same longest match as this linear scan over the
/// length-ordered `script.punctuations`, so the scan is transcribed.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:768-802`
pub fn PS_ReadPunctuation(script: &mut Script, token: &mut Token) -> c_int {
    let punctuations = script.punctuations;
    for punc in punctuations {
        let p = punc.p;
        let len = p.len();
        // if the script contains at least as much characters as the punctuation
        if script.script_p + len <= script.end_p {
            // if the script contains the punctuation
            if &script.buffer[script.script_p..script.script_p + len] == p.as_bytes() {
                token.string = p.to_string();
                script.script_p += len;
                token.type_ = TT_PUNCTUATION;
                // sub type is the number of the punctuation
                token.subtype = punc.n;
                return 1;
            }
        }
    }
    0
}

/// Raven `PS_ReadPrimitive` — read a whitespace/`;`-delimited primitive token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:809-828`
pub fn PS_ReadPrimitive(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    let mut len: usize = 0;
    while (script.buffer[script.script_p] as c_char) > b' ' as c_char
        && script.buffer[script.script_p] != b';'
    {
        if len >= MAX_TOKEN {
            ScriptError(
                bot,
                script,
                &format!("primitive token longer than MAX_TOKEN = {}", MAX_TOKEN),
            );
            return 0;
        }
        token.string.push(script.buffer[script.script_p] as char);
        script.script_p += 1;
        len += 1;
    }
    // copy the token into the script structure
    script.token = token.clone();
    // primitive reading successfull
    1
}

/// Raven `PS_ReadToken` — read the next token, dispatching by lookahead
/// character to string/number/name/primitive/punctuation readers.
///
/// Raven's own commented-out `PS_ReadLiteral(script, token)` call
/// (l_script.cpp:870) is preserved as a comment; the live call is
/// `PS_ReadString(script, token, '\'')`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:835-902`
pub fn PS_ReadToken(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    // if there is a token available (from UnreadToken)
    if script.tokenavailable != 0 {
        script.tokenavailable = 0;
        *token = script.token.clone();
        return 1;
    }
    // save script pointer
    script.lastscript_p = script.script_p;
    // save line counter
    script.lastline = script.line;
    // clear the token stuff
    *token = Token::default();
    // start of the white space
    script.whitespace_p = script.script_p;
    // read unusefull stuff
    if PS_ReadWhiteSpace(script) == 0 {
        return 0;
    }
    // end of the white space
    script.endwhitespace_p = script.script_p;
    token.whitespace_span = Some((script.whitespace_p, script.endwhitespace_p));
    // line the token is on
    token.line = script.line;
    // number of lines crossed before token
    token.linescrossed = script.line - script.lastline;
    let c = script.buffer[script.script_p] as c_char;
    // if there is a leading double quote
    if c == b'\"' as c_char {
        if PS_ReadString(bot, script, token, b'\"' as c_int) == 0 {
            return 0;
        }
    }
    // if an literal
    else if c == b'\'' as c_char {
        // if (!PS_ReadLiteral(script, token)) return 0;
        if PS_ReadString(bot, script, token, b'\'' as c_int) == 0 {
            return 0;
        }
    }
    // if there is a number
    else if (c >= b'0' as c_char && c <= b'9' as c_char)
        || (c == b'.' as c_char
            && (script.buffer[script.script_p + 1] as c_char >= b'0' as c_char
                && script.buffer[script.script_p + 1] as c_char <= b'9' as c_char))
    {
        if PS_ReadNumber(bot, script, token) == 0 {
            return 0;
        }
    }
    // if this is a primitive script
    else if script.flags & SCFL_PRIMITIVE != 0 {
        return PS_ReadPrimitive(bot, script, token);
    }
    // if there is a name
    else if (c >= b'a' as c_char && c <= b'z' as c_char)
        || (c >= b'A' as c_char && c <= b'Z' as c_char)
        || c == b'_' as c_char
        || c == b'@' as c_char
    {
        if PS_ReadName(bot, script, token) == 0 {
            return 0;
        }
    }
    // check for punctuations
    else if PS_ReadPunctuation(script, token) == 0 {
        ScriptError(bot, script, "can't read token");
        return 0;
    }
    // copy the token into the script structure
    script.token = token.clone();
    // succesfully read a token
    1
}

/// Raven `PS_ExpectTokenString` — read a token and require it to equal a
/// literal string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:909-925`
pub fn PS_ExpectTokenString(bot: &mut BotLib, script: &mut Script, string: &str) -> c_int {
    let mut token = Token::default();

    if PS_ReadToken(bot, script, &mut token) == 0 {
        ScriptError(bot, script, &format!("couldn't find expected {}", string));
        return 0;
    }

    if token.string != string {
        ScriptError(
            bot,
            script,
            &format!("expected {}, found {}", string, token.string),
        );
        return 0;
    }
    1
}

/// Raven `PS_ExpectTokenType` — read a token and require it to match a
/// type/subtype.
///
/// The punctuation-mismatch message uses `PunctuationFromNum` (§F19): Raven's
/// `script->punctuations[subtype]` passes a `punctuation_t` struct where `%s`
/// expects a `char *` (UB); the sensible defined behavior is the punctuation's
/// text.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:932-983`
pub fn PS_ExpectTokenType(
    bot: &mut BotLib,
    script: &mut Script,
    type_: c_int,
    subtype: c_int,
    token: &mut Token,
) -> c_int {
    let mut str = String::new();

    if PS_ReadToken(bot, script, token) == 0 {
        ScriptError(bot, script, "couldn't read expected token");
        return 0;
    }

    if token.type_ != type_ {
        if type_ == TT_STRING {
            str = "string".to_string();
        }
        if type_ == TT_LITERAL {
            str = "literal".to_string();
        }
        if type_ == TT_NUMBER {
            str = "number".to_string();
        }
        if type_ == TT_NAME {
            str = "name".to_string();
        }
        if type_ == TT_PUNCTUATION {
            str = "punctuation".to_string();
        }
        ScriptError(
            bot,
            script,
            &format!("expected a {}, found {}", str, token.string),
        );
        return 0;
    }
    if token.type_ == TT_NUMBER {
        if (token.subtype & subtype) != subtype {
            if subtype & TT_DECIMAL != 0 {
                str = "decimal".to_string();
            }
            if subtype & TT_HEX != 0 {
                str = "hex".to_string();
            }
            if subtype & TT_OCTAL != 0 {
                str = "octal".to_string();
            }
            if subtype & TT_BINARY != 0 {
                str = "binary".to_string();
            }
            if subtype & TT_LONG != 0 {
                str.push_str(" long");
            }
            if subtype & TT_UNSIGNED != 0 {
                str.push_str(" unsigned");
            }
            if subtype & TT_FLOAT != 0 {
                str.push_str(" float");
            }
            if subtype & TT_INTEGER != 0 {
                str.push_str(" integer");
            }
            ScriptError(
                bot,
                script,
                &format!("expected {}, found {}", str, token.string),
            );
            return 0;
        }
    } else if token.type_ == TT_PUNCTUATION {
        if subtype < 0 {
            ScriptError(bot, script, "BUG: wrong punctuation subtype");
            return 0;
        }
        if token.subtype != subtype {
            let punc = PunctuationFromNum(script, subtype);
            ScriptError(
                bot,
                script,
                &format!("expected {}, found {}", punc, token.string),
            );
            return 0;
        }
    }
    1
}

/// Raven `PS_ExpectAnyToken` — read a token, requiring only that one exists.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:990-1001`
pub fn PS_ExpectAnyToken(bot: &mut BotLib, script: &mut Script, token: &mut Token) -> c_int {
    if PS_ReadToken(bot, script, token) == 0 {
        ScriptError(bot, script, "couldn't read expected token");
        0
    } else {
        1
    }
}

/// Raven `PS_CheckTokenString` — peek a token, consuming it only if it
/// matches a literal string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1008-1018`
pub fn PS_CheckTokenString(bot: &mut BotLib, script: &mut Script, string: &str) -> c_int {
    let mut tok = Token::default();

    if PS_ReadToken(bot, script, &mut tok) == 0 {
        return 0;
    }
    // if the token is available
    if tok.string == string {
        return 1;
    }
    // token not available
    script.script_p = script.lastscript_p;
    0
}

/// Raven `PS_CheckTokenType` — peek a token, consuming it only if it matches
/// a type/subtype.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1025-1040`
pub fn PS_CheckTokenType(
    bot: &mut BotLib,
    script: &mut Script,
    type_: c_int,
    subtype: c_int,
    token: &mut Token,
) -> c_int {
    let mut tok = Token::default();

    if PS_ReadToken(bot, script, &mut tok) == 0 {
        return 0;
    }
    // if the type matches
    if tok.type_ == type_ && (tok.subtype & subtype) == subtype {
        *token = tok;
        return 1;
    }
    // token is not available
    script.script_p = script.lastscript_p;
    0
}

/// Raven `PS_SkipUntilString` — read tokens until one matches a literal
/// string.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1047-1056`
pub fn PS_SkipUntilString(bot: &mut BotLib, script: &mut Script, string: &str) -> c_int {
    let mut token = Token::default();
    while PS_ReadToken(bot, script, &mut token) != 0 {
        if token.string == string {
            return 1;
        }
    }
    0
}

/// Raven `PS_UnreadLastToken` — mark the last-read token available again.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1063-1066`
pub fn PS_UnreadLastToken(script: &mut Script) {
    script.tokenavailable = 1;
}

/// Raven `PS_UnreadToken` — push a token back onto the script for the next
/// `PS_ReadToken`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1073-1077`
pub fn PS_UnreadToken(script: &mut Script, token: &Token) {
    script.token = token.clone();
    script.tokenavailable = 1;
}

/// Raven `PS_NextWhiteSpaceChar` — step through the last token's saved
/// whitespace span one character at a time.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1085-1095`
pub fn PS_NextWhiteSpaceChar(script: &mut Script) -> c_char {
    if script.whitespace_p != script.endwhitespace_p {
        let c = script.buffer[script.whitespace_p] as c_char;
        script.whitespace_p += 1;
        c
    } else {
        0
    }
}

/// Raven `StripDoubleQuotes` — strip a single leading/trailing `"` pair.
///
/// Raven removes the leading quote, then tests `string[strlen-1]` on the
/// shortened string — which underruns for a lone `"` (§F19); this returns `""`.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1102-1112`
pub fn StripDoubleQuotes(string: &mut String) {
    if string.starts_with('\"') {
        string.remove(0);
    }
    if string.ends_with('\"') {
        string.pop();
    }
}

/// Raven `StripSingleQuotes` — strip a single leading/trailing `'` pair.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1119-1129`
pub fn StripSingleQuotes(string: &mut String) {
    if string.starts_with('\'') {
        string.remove(0);
    }
    if string.ends_with('\'') {
        string.pop();
    }
}

/// Raven `ReadSignedFloat` — read an optionally `-`-signed float value token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1136-1152`
pub fn ReadSignedFloat(bot: &mut BotLib, script: &mut Script) -> f64 {
    let mut token = Token::default();
    let mut sign: f64 = 1.0;

    PS_ExpectAnyToken(bot, script, &mut token);
    if token.string == "-" {
        sign = -1.0;
        PS_ExpectTokenType(bot, script, TT_NUMBER, 0, &mut token);
    } else if token.type_ != TT_NUMBER {
        ScriptError(
            bot,
            script,
            &format!("expected float value, found {}\n", token.string),
        );
    }
    sign * token.floatvalue
}

/// Raven `ReadSignedInt` — read an optionally `-`-signed integer value token.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1159-1175`
pub fn ReadSignedInt(bot: &mut BotLib, script: &mut Script) -> c_long {
    let mut token = Token::default();
    let mut sign: c_long = 1;

    PS_ExpectAnyToken(bot, script, &mut token);
    if token.string == "-" {
        sign = -1;
        PS_ExpectTokenType(bot, script, TT_NUMBER, TT_INTEGER, &mut token);
    } else if token.type_ != TT_NUMBER || token.subtype == TT_FLOAT {
        ScriptError(
            bot,
            script,
            &format!("expected integer value, found {}\n", token.string),
        );
    }
    sign * token.intvalue as c_long
}

/// Raven `SetScriptFlags` — replace a script's flag word.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1182-1185`
pub fn SetScriptFlags(script: &mut Script, flags: c_int) {
    script.flags = flags;
}

/// Raven `GetScriptFlags` — read a script's flag word.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1192-1195`
pub fn GetScriptFlags(script: &Script) -> c_int {
    script.flags
}

/// Raven `ResetScript` — rewind a script's lexing cursor to the start.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1202-1219`
pub fn ResetScript(script: &mut Script) {
    // pointer in script buffer
    script.script_p = 0;
    // pointer in script buffer before reading token
    script.lastscript_p = 0;
    // begin of white space (Raven NULL → index 0; the span stays empty)
    script.whitespace_p = 0;
    // end of white space
    script.endwhitespace_p = 0;
    // set if there's a token available in script.token
    script.tokenavailable = 0;
    script.line = 1;
    script.lastline = 1;
    // clear the saved token
    script.token = Token::default();
}

/// Raven `EndOfScript` — true once the cursor has reached the buffer end.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1227-1230`
pub fn EndOfScript(script: &Script) -> c_int {
    (script.script_p >= script.end_p) as c_int
}

/// Raven `NumLinesCrossed` — lines crossed since the last saved line count.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1237-1240`
pub fn NumLinesCrossed(script: &Script) -> c_int {
    script.line - script.lastline
}

/// Raven `ScriptSkipTo` — skip forward until a literal string is found.
///
/// The `strncmp` slice is bounds-checked against the buffer end (§F19): Raven's
/// raw `strncmp` could overread near the buffer tail.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1247-1266`
pub fn ScriptSkipTo(script: &mut Script, value: &str) -> c_int {
    let firstchar = value.as_bytes().first().copied().unwrap_or(0);
    let len = value.len();
    loop {
        if PS_ReadWhiteSpace(script) == 0 {
            return 0;
        }
        if script.buffer[script.script_p] == firstchar
            && script.script_p + len <= script.buffer.len()
            && &script.buffer[script.script_p..script.script_p + len] == value.as_bytes()
        {
            return 1;
        }
        script.script_p += 1;
    }
}

/// Raven `LoadScriptFile` — load a script file from disk via `botimport.FS_*`.
///
/// `BOTLIB` is defined in this build, so the `botimport.FS_*` arm is
/// transcribed; the standalone-`fopen` `#else` arm is dropped per §C10. The
/// malloc'd `script_t`+buffer allocation becomes an owned `Script` with a
/// `Vec<u8>` buffer.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1293-1355`
pub fn LoadScriptFile(bot: &mut BotLib, filename: &str) -> Option<Script> {
    let mut pathname = [0 as c_char; MAX_QPATH];
    let mut fp: fileHandle_t = 0;

    if bot.basefolder[0] != 0 {
        let basefolder = unsafe { CStr::from_ptr(bot.basefolder.as_ptr()) }.to_string_lossy();
        let pathname_c = CString::new(format!("{}/{}", basefolder, filename)).unwrap_or_default();
        Q_strncpyz(pathname.as_mut_ptr(), pathname_c.as_ptr(), MAX_QPATH as c_int);
    } else {
        let pathname_c = CString::new(filename).unwrap_or_default();
        Q_strncpyz(pathname.as_mut_ptr(), pathname_c.as_ptr(), MAX_QPATH as c_int);
    }
    let length = unsafe { (bot.botimport.FS_FOpenFile.unwrap())(pathname.as_ptr(), &mut fp, FS_READ) };
    if fp == 0 {
        return None;
    }

    let mut buffer = vec![0u8; length as usize + 1];
    unsafe {
        (bot.botimport.FS_Read.unwrap())(buffer.as_mut_ptr() as *mut c_void, length, fp);
        (bot.botimport.FS_FCloseFile.unwrap())(fp);
    }
    buffer[length as usize] = 0;

    let mut script = Script {
        filename: filename.to_owned(),
        buffer,
        script_p: 0,
        // pointer to end of script buffer (the *original* length; the compressed
        // NUL below stops the lexer earlier — Raven leaves `end_p` here too)
        end_p: length as usize,
        // pointer in script buffer before reading token
        lastscript_p: 0,
        whitespace_p: 0,
        endwhitespace_p: 0,
        length,
        line: 1,
        lastline: 1,
        tokenavailable: 0,
        flags: 0,
        // SetScriptPunctuations(bot, script, None) — the default C/C++ set
        punctuations: DEFAULT_PUNCTUATIONS,
        token: Token::default(),
    };

    script.length = COM_Compress(script.buffer.as_mut_ptr() as *mut c_char);

    Some(script)
}

/// Raven `LoadScriptMemory` — load a script from an in-memory buffer.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1362-1391`
pub fn LoadScriptMemory(ptr: &[u8], length: c_int, name: &str) -> Script {
    let mut buffer = vec![0u8; length as usize + 1];
    buffer[..length as usize].copy_from_slice(&ptr[..length as usize]);
    buffer[length as usize] = 0;

    Script {
        filename: name.to_owned(),
        buffer,
        script_p: 0,
        end_p: length as usize,
        lastscript_p: 0,
        whitespace_p: 0,
        endwhitespace_p: 0,
        length,
        line: 1,
        lastline: 1,
        tokenavailable: 0,
        flags: 0,
        // SetScriptPunctuations(bot, script, None) — the default C/C++ set
        punctuations: DEFAULT_PUNCTUATIONS,
        token: Token::default(),
    }
}

/// Raven `FreeScript` — free a loaded script and its punctuation table.
///
/// The `#ifdef PUNCTABLE` free arm is dropped (no table is built), and Raven's
/// `FreeMemory(script)` becomes an ownership drop: consuming the owned `Script`
/// reclaims its `Vec<u8>` buffer.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1398-1404`
pub fn FreeScript(script: Script) {
    drop(script);
}

/// Raven `PS_SetBaseFolder` — set the tokenizer's base folder for
/// `LoadScriptFile`.
///
/// `BSPC` is not defined in this build; the `#ifdef BSPC` `sprintf` arm is
/// dropped per §C10.
///
/// Source: `oracle/codemp/botlib/l_script.cpp:1411-1418`
pub fn PS_SetBaseFolder(bot: &mut BotLib, path: &str) {
    let path_c = CString::new(path).unwrap_or_default();
    Q_strncpyz(bot.basefolder.as_mut_ptr(), path_c.as_ptr(), MAX_QPATH as c_int);
}
