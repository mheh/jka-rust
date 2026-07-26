#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]
//! `l_struct.cpp` — the botlib generic struct read/write helpers (find a
//! field by name, write/read a `structdef_t`-described struct to/from a
//! text file).
//!
//! One Rust module per oracle source file (`l_struct.cpp`); the stem collides
//! with the `l_struct/` type directory, so this lands as `l_struct_fns.rs`
//! per `_PREAMBLE.md`'s destination rule.
//!
//! Source: `oracle/codemp/botlib/l_struct.cpp`

use core::ffi::{c_char, c_int};
use std::ffi::CString;

use libc::FILE;

use mp_qshared::shared::q_string::Q_strncpyz;
use mp_qshared::shared::{qfalse, qtrue};
use native_string::string_to_latin1;
use native_types::qboolean;

use crate::be_ai_goal::iteminfo_s::MAX_STRINGFIELD;
use crate::l_precomp::source_s::Source;
use crate::l_precomp_fns::{
    PC_CheckTokenString, PC_ExpectAnyToken, PC_ExpectTokenString, PC_ExpectTokenType,
    PC_UnreadLastToken, SourceError,
};
use crate::l_script::consts::{TT_FLOAT, TT_LITERAL, TT_NUMBER, TT_PUNCTUATION, TT_STRING};
use crate::l_script::token_s::Token;
use crate::l_script_fns::{StripDoubleQuotes, StripSingleQuotes};
use crate::l_struct::fielddef_s::fielddef_t;
use crate::l_struct::l_struct_consts::{
    FT_ARRAY, FT_BOUNDED, FT_CHAR, FT_FLOAT, FT_INT, FT_STRING, FT_STRUCT, FT_TYPE, FT_UNSIGNED,
};
use crate::l_struct::structdef_s::structdef_t;
use crate::BotLib;

/// Raven `FindField` — linear-search a `fielddef_t[]` by name.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:43-52`
pub fn FindField(defs: *mut fielddef_t, name: *mut c_char) -> *mut fielddef_t {
    unsafe {
        let mut i: isize = 0;
        while !(*defs.offset(i)).name.is_null() {
            if libc::strcmp((*defs.offset(i)).name, name) == 0 {
                return defs.offset(i);
            }
            i += 1;
        }
        core::ptr::null_mut()
    }
}

/// Raven `WriteIndent` — write `indent` tab characters to `fp`.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:313-320`
pub fn WriteIndent(fp: *mut FILE, indent: c_int) -> c_int {
    unsafe {
        let mut indent = indent;
        while {
            let cur = indent;
            indent -= 1;
            cur > 0
        } {
            if libc::fprintf(fp, c"\t".as_ptr()) < 0 {
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `WriteFloat` — write a float to `fp`, stripping trailing zeros
/// (and a bare trailing `.`).
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:327-348`
pub fn WriteFloat(fp: *mut FILE, value: f32) -> c_int {
    unsafe {
        let mut buf = [0 as c_char; 128];
        libc::sprintf(
            buf.as_mut_ptr(),
            c"%f".as_ptr(),
            value as core::ffi::c_double,
        );
        let mut l = libc::strlen(buf.as_ptr()) as isize;
        //strip any trailing zeros
        while {
            l -= 1;
            l > 0
        } {
            let ch = *buf.as_ptr().offset(l) as u8 as char;
            if ch != '0' && ch != '.' {
                break;
            }
            if ch == '.' {
                *buf.as_mut_ptr().offset(l) = 0;
                break;
            }
            *buf.as_mut_ptr().offset(l) = 0;
        }
        //write the float to file
        if libc::fprintf(fp, c"%s".as_ptr(), buf.as_ptr()) < 0 {
            return 0;
        }
        1
    }
}

/// Raven `WriteStructWithIndent` — write `structure` (described by `def`)
/// to `fp` as an indented text block, recursing into `FT_STRUCT` fields.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:355-434`
pub fn WriteStructWithIndent(
    fp: *mut FILE,
    def: *mut structdef_t,
    structure: *mut c_char,
    indent: c_int,
) -> c_int {
    unsafe {
        let mut indent = indent;

        if WriteIndent(fp, indent) == 0 {
            return qfalse;
        }
        if libc::fprintf(fp, c"{\r\n".as_ptr()) < 0 {
            return qfalse;
        }

        indent += 1;
        let mut i: isize = 0;
        while !(*(*def).fields.offset(i)).name.is_null() {
            let fd: *mut fielddef_t = (*def).fields.offset(i);
            if WriteIndent(fp, indent) == 0 {
                return qfalse;
            }
            if libc::fprintf(fp, c"%s\t".as_ptr(), (*fd).name) < 0 {
                return qfalse;
            }
            let mut p: *mut u8 = (structure as *mut u8).offset((*fd).offset as isize);
            let mut num: c_int;
            if (*fd).r#type & FT_ARRAY != 0 {
                num = (*fd).maxarray;
                if libc::fprintf(fp, c"{".as_ptr()) < 0 {
                    return qfalse;
                }
            } else {
                num = 1;
            }
            while {
                let cur = num;
                num -= 1;
                cur > 0
            } {
                match (*fd).r#type & FT_TYPE {
                    x if x == FT_CHAR => {
                        if libc::fprintf(fp, c"%d".as_ptr(), *(p as *const c_char) as c_int) < 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<c_char>());
                    }
                    x if x == FT_INT => {
                        if libc::fprintf(fp, c"%d".as_ptr(), *(p as *const c_int)) < 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<c_int>());
                    }
                    x if x == FT_FLOAT => {
                        if WriteFloat(fp, *(p as *const f32)) == 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<f32>());
                    }
                    x if x == FT_STRING => {
                        if libc::fprintf(fp, c"\"%s\"".as_ptr(), p as *const c_char) < 0 {
                            return qfalse;
                        }
                        p = p.add(crate::be_ai_goal::iteminfo_s::MAX_STRINGFIELD);
                    }
                    x if x == FT_STRUCT => {
                        if WriteStructWithIndent(fp, (*fd).substruct, structure, indent) == 0 {
                            return qfalse;
                        }
                        p = p.add((*(*fd).substruct).size as usize);
                    }
                    _ => {}
                }
                if (*fd).r#type & FT_ARRAY != 0 {
                    if num > 0 {
                        if libc::fprintf(fp, c",".as_ptr()) < 0 {
                            return qfalse;
                        }
                    } else if libc::fprintf(fp, c"}".as_ptr()) < 0 {
                        return qfalse;
                    }
                }
            }
            if libc::fprintf(fp, c"\r\n".as_ptr()) < 0 {
                return qfalse;
            }
            i += 1;
        }
        indent -= 1;

        if WriteIndent(fp, indent) == 0 {
            return qfalse;
        }
        if libc::fprintf(fp, c"}\r\n".as_ptr()) < 0 {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `WriteStructure` — write a struct to `fp` at indent 0.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:441-444`
pub fn WriteStructure(fp: *mut FILE, def: *mut structdef_t, structure: *mut c_char) -> c_int {
    WriteStructWithIndent(fp, def, structure, 0)
}

/// Raven `ReadNumber` — read (and range-/bound-check) a number token into
/// the field-typed value at `p`.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:59-167`
pub fn ReadNumber(
    bot: &mut BotLib,
    source: &mut Source,
    fd: *mut fielddef_t,
    p: *mut (),
) -> qboolean {
    unsafe {
        let mut token = Token::default();
        let mut negative: c_int = qfalse;
        let mut intval: i64;
        let mut intmin: i64 = 0;
        let mut intmax: i64 = 0;
        let floatval: f64;

        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return 0;
        }

        //check for minus sign
        if token.type_ == TT_PUNCTUATION {
            if (*fd).r#type & FT_UNSIGNED != 0 {
                SourceError(
                    bot,
                    source,
                    &format!("expected unsigned value, found {}", token.string),
                );
                return 0;
            }
            //if not a minus sign
            if token.string != "-" {
                SourceError(
                    bot,
                    source,
                    &format!("unexpected punctuation {}", token.string),
                );
                return 0;
            }
            negative = qtrue;
            //read the number
            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                return 0;
            }
        }
        //check if it is a number
        if token.type_ != TT_NUMBER {
            SourceError(
                bot,
                source,
                &format!("expected number, found {}", token.string),
            );
            return 0;
        }
        //check for a float value
        if token.subtype & TT_FLOAT != 0 {
            if ((*fd).r#type & FT_TYPE) != FT_FLOAT {
                SourceError(bot, source, "unexpected float");
                return 0;
            }
            floatval = if negative != 0 {
                -token.floatvalue
            } else {
                token.floatvalue
            };
            if (*fd).r#type & FT_BOUNDED != 0 {
                if floatval < (*fd).floatmin as f64 || floatval > (*fd).floatmax as f64 {
                    SourceError(
                        bot,
                        source,
                        &format!(
                            "float out of range [{:.6}, {:.6}]",
                            (*fd).floatmin as core::ffi::c_double,
                            (*fd).floatmax as core::ffi::c_double
                        ),
                    );
                    return 0;
                }
            }
            *(p as *mut f32) = floatval as f32;
            return 1;
        }
        //
        intval = token.intvalue as i64;
        if negative != 0 {
            intval = -intval;
        }
        //check bounds
        if ((*fd).r#type & FT_TYPE) == FT_CHAR {
            if (*fd).r#type & FT_UNSIGNED != 0 {
                intmin = 0;
                intmax = 255;
            } else {
                intmin = -128;
                intmax = 127;
            }
        }
        if ((*fd).r#type & FT_TYPE) == FT_INT {
            if (*fd).r#type & FT_UNSIGNED != 0 {
                intmin = 0;
                intmax = 65535;
            } else {
                intmin = -32768;
                intmax = 32767;
            }
        }
        if ((*fd).r#type & FT_TYPE) == FT_CHAR || ((*fd).r#type & FT_TYPE) == FT_INT {
            if (*fd).r#type & FT_BOUNDED != 0 {
                // Raven `Maximum`/`Minimum` macros — plain clamp comparisons.
                let fmin = (*fd).floatmin as i64;
                let fmax = (*fd).floatmax as i64;
                intmin = if intmin > fmin { intmin } else { fmin };
                intmax = if intmax < fmax { intmax } else { fmax };
            }
            if intval < intmin || intval > intmax {
                SourceError(
                    bot,
                    source,
                    &format!(
                        "value {} out of range [{}, {}]",
                        intval as c_int, intmin as c_int, intmax as c_int
                    ),
                );
                return 0;
            }
        } else if ((*fd).r#type & FT_TYPE) == FT_FLOAT {
            if (*fd).r#type & FT_BOUNDED != 0
                && ((intval as f32) < (*fd).floatmin || (intval as f32) > (*fd).floatmax)
            {
                SourceError(
                    bot,
                    source,
                    &format!(
                        "value {} out of range [{:.6}, {:.6}]",
                        intval as c_int,
                        (*fd).floatmin as core::ffi::c_double,
                        (*fd).floatmax as core::ffi::c_double
                    ),
                );
                return 0;
            }
        }
        //store the value
        if ((*fd).r#type & FT_TYPE) == FT_CHAR {
            if (*fd).r#type & FT_UNSIGNED != 0 {
                *(p as *mut u8) = intval as u8;
            } else {
                *(p as *mut c_char) = intval as c_char;
            }
        } else if ((*fd).r#type & FT_TYPE) == FT_INT {
            if (*fd).r#type & FT_UNSIGNED != 0 {
                *(p as *mut u32) = intval as u32;
            } else {
                *(p as *mut i32) = intval as i32;
            }
        } else if ((*fd).r#type & FT_TYPE) == FT_FLOAT {
            *(p as *mut f32) = intval as f32;
        }
        1
    }
}

/// Raven `ReadString` — read a quoted string token into the fixed-size
/// `MAX_STRINGFIELD` buffer at `p`.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:199-212`
pub fn ReadString(bot: &mut BotLib, source: &mut Source, fd: *mut fielddef_t, p: *mut ()) -> c_int {
    let mut token = Token::default();

    if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0 {
        return 0;
    }
    //remove the double quotes
    StripDoubleQuotes(&mut token.string);
    //copy the string (bounded into the field's fixed MAX_STRINGFIELD buffer,
    //preserving Raven's strncpy truncation + forced NUL terminator)
    let token_string_c = CString::new(token.string.as_str()).unwrap_or_default();
    Q_strncpyz(
        p as *mut c_char,
        token_string_c.as_ptr(),
        MAX_STRINGFIELD as c_int,
    );
    //
    1
}

/// Raven `ReadChar` — read a char token: single-quoted literal, or fall
/// through to `ReadNumber`.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:174-192`
pub fn ReadChar(
    bot: &mut BotLib,
    source: &mut Source,
    fd: *mut fielddef_t,
    p: *mut (),
) -> qboolean {
    unsafe {
        let mut token = Token::default();

        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return 0;
        }

        //take literals into account
        if token.type_ == TT_LITERAL {
            StripSingleQuotes(&mut token.string);
            // Raven reads token.string[0]; an emptied literal ('') leaves the NUL
            // terminator there, i.e. 0.
            *(p as *mut c_char) = string_to_latin1(&token.string)
                .first()
                .copied()
                .unwrap_or(0) as c_char;
        } else {
            PC_UnreadLastToken(source);
            if ReadNumber(bot, source, fd, p) == 0 {
                return 0;
            }
        }
        1
    }
}

/// Raven `ReadStructure` — read a `{ field value ... }` block from `source`
/// into `structure`, described by `def`.
///
/// Source: `oracle/codemp/botlib/l_struct.cpp:219-306`
pub fn ReadStructure(
    bot: &mut BotLib,
    source: &mut Source,
    def: *mut structdef_t,
    structure: *mut c_char,
) -> c_int {
    unsafe {
        let mut token = Token::default();
        let mut fd: *mut fielddef_t;
        let mut p: *mut u8;
        let mut num: c_int;

        if PC_ExpectTokenString(bot, source, "{") == 0 {
            return 0;
        }
        loop {
            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                return qfalse;
            }
            //if end of structure
            if token.string == "}" {
                break;
            }
            //find the field with the name
            let token_string_c = CString::new(token.string.as_str()).unwrap_or_default();
            fd = FindField((*def).fields, token_string_c.as_ptr() as *mut c_char);
            if fd.is_null() {
                SourceError(
                    bot,
                    source,
                    &format!("unknown structure field {}", token.string),
                );
                return qfalse;
            }
            if (*fd).r#type & FT_ARRAY != 0 {
                num = (*fd).maxarray;
                if PC_ExpectTokenString(bot, source, "{") == 0 {
                    return qfalse;
                }
            } else {
                num = 1;
            }
            p = (structure as *mut u8).offset((*fd).offset as isize);
            while {
                let cur = num;
                num -= 1;
                cur > 0
            } {
                if (*fd).r#type & FT_ARRAY != 0 && PC_CheckTokenString(bot, source, "}") != 0 {
                    break;
                }
                match (*fd).r#type & FT_TYPE {
                    x if x == FT_CHAR => {
                        if ReadChar(bot, source, fd, p as *mut ()) == 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<c_char>());
                    }
                    x if x == FT_INT => {
                        if ReadNumber(bot, source, fd, p as *mut ()) == 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<c_int>());
                    }
                    x if x == FT_FLOAT => {
                        if ReadNumber(bot, source, fd, p as *mut ()) == 0 {
                            return qfalse;
                        }
                        p = p.add(core::mem::size_of::<f32>());
                    }
                    x if x == FT_STRING => {
                        if ReadString(bot, source, fd, p as *mut ()) == 0 {
                            return qfalse;
                        }
                        p = p.add(crate::be_ai_goal::iteminfo_s::MAX_STRINGFIELD);
                    }
                    x if x == FT_STRUCT => {
                        if (*fd).substruct.is_null() {
                            SourceError(bot, source, "BUG: no sub structure defined");
                            return qfalse;
                        }
                        // Raven ignores this recursive call's return value.
                        ReadStructure(bot, source, (*fd).substruct, p as *mut c_char);
                        p = p.add((*(*fd).substruct).size as usize);
                    }
                    _ => {}
                }
                if (*fd).r#type & FT_ARRAY != 0 {
                    if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                        return qfalse;
                    }
                    if token.string == "}" {
                        break;
                    }
                    if token.string != "," {
                        SourceError(
                            bot,
                            source,
                            &format!("expected a comma, found {}", token.string),
                        );
                        return qfalse;
                    }
                }
            }
        }
        qtrue
    }
}
