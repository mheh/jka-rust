#![allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_assignments,
    unused_mut
)]

//! Function bodies for Raven's `be_ai_char.cpp` (bot character loading:
//! parse/cache/interpolate/free character files, characteristic accessors).
//!
//! Ported per the engine C-track packets (`botlib__0502`..`botlib__2456`).
//! Source: `oracle/codemp/botlib/be_ai_char.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_ai_char.rs`, but `be_ai_char` already
//! exists as a directory module (`be_ai_char/mod.rs`, types-only) — `_fns`
//! escape per `_PREAMBLE.md`'s destination rule.
//!
//! PORT-NOTE(varsize): `bot_character_t::c` is Raven's variable-sized
//! trailing characteristic array, emulated in the type port as
//! `[bot_characteristic_t; 1]`; bodies here index past that bound via raw
//! pointer arithmetic (`.c.as_mut_ptr().add(i)`) exactly as Raven's C array
//! indexing does, matching the type's own doc comment.

use core::ffi::{c_char, c_int, c_ulong};

use libc::{strcmp, strcpy, strlen, strncpy};
use native_types::{qfalse, qtrue};

use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::limits::MAX_CLIENTS;

use crate::be_ai_char::bot_character_s::bot_character_t;
use crate::be_ai_char::bot_characteristic_s::bot_characteristic_t;
use crate::be_ai_char::consts::{
    CT_FLOAT, CT_INTEGER, CT_STRING, DEFAULT_CHARACTER, MAX_CHARACTERISTICS,
};
use crate::l_libvar_fns::LibVarGetValue;
use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory, GetMemory};
use crate::l_precomp::source_s::source_t;
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_ExpectAnyToken, PC_ExpectTokenString, PC_ExpectTokenType,
    PC_ReadToken, PC_SetBaseFolder, SourceError,
};
use crate::l_script::consts::{TT_FLOAT, TT_INTEGER, TT_NUMBER, TT_STRING};
use crate::l_script::token_s::token_t;
use crate::l_script_fns::StripDoubleQuotes;
use crate::BotLib;

/// Raven `BotCharacterFromHandle`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:68-81`
pub fn BotCharacterFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_character_t {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"character handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
            return core::ptr::null_mut();
        } //end if
        if bot.botcharacters[handle as usize].is_null() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid character %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return core::ptr::null_mut();
        } //end if
        bot.botcharacters[handle as usize]
    }
}

/// Raven `BotFindCachedCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:345-359`
pub fn BotFindCachedCharacter(bot: &mut BotLib, charfile: *mut c_char, skill: f32) -> c_int {
    unsafe {
        for handle in 1..=MAX_CLIENTS as c_int {
            let ch = bot.botcharacters[handle as usize];
            if ch.is_null() {
                continue;
            } //end if
            if strcmp((*ch).filename.as_ptr(), charfile) == 0
                && (skill < 0.0 || ((*ch).skill - skill).abs() < 0.01)
            {
                return handle;
            } //end if
        } //end for
        0
    }
}

/// Raven `BotDumpCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:88-105`
pub fn BotDumpCharacter(bot: &mut BotLib, ch: *mut bot_character_t) {
    unsafe {
        let __m = std::ffi::CString::new(format!(
            "{}",
            std::ffi::CStr::from_ptr((*ch).filename.as_ptr()).to_string_lossy()
        ))
        .unwrap_or_default();
        Log_Write(bot, __m.as_ptr() as *mut c_char);
        // Raven's own format string uses `%d` for the (float) `skill` field —
        // an oracle bug, kept faithful.
        let __m = std::ffi::CString::new(format!("skill {}\n", (*ch).skill as c_int))
            .unwrap_or_default();
        Log_Write(bot, __m.as_ptr() as *mut c_char);
        Log_Write(bot, c"{\n".as_ptr() as *mut c_char);
        for i in 0..MAX_CHARACTERISTICS {
            let c = (*ch).c.as_mut_ptr().add(i as usize);
            let t = (*c).r#type as c_int;
            if t == CT_INTEGER as c_int {
                let __m = std::ffi::CString::new(format!(" {:4} {}\n", i, (*c).value.integer))
                    .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            } else if t == CT_FLOAT as c_int {
                let __m =
                    std::ffi::CString::new(format!(" {:4} {}\n", i, (*c).value._float as f64))
                        .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            } else if t == CT_STRING as c_int {
                let __m = std::ffi::CString::new(format!(
                    " {:4} {}\n",
                    i,
                    std::ffi::CStr::from_ptr((*c).value.string).to_string_lossy()
                ))
                .unwrap_or_default();
                Log_Write(bot, __m.as_ptr() as *mut c_char);
            } //end case
        } //end for
        Log_Write(bot, c"}\n".as_ptr() as *mut c_char);
    }
}

/// Raven `BotFreeCharacterStrings`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:112-123`
pub fn BotFreeCharacterStrings(bot: &mut BotLib, ch: *mut bot_character_t) {
    unsafe {
        for i in 0..MAX_CHARACTERISTICS {
            let c = (*ch).c.as_mut_ptr().add(i as usize);
            if (*c).r#type as c_int == CT_STRING as c_int {
                FreeMemory(bot, (*c).value.string as *mut ());
            } //end if
        } //end for
    }
}

/// Raven `BotDefaultCharacteristics`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:163-188`
pub fn BotDefaultCharacteristics(
    bot: &mut BotLib,
    ch: *mut bot_character_t,
    defaultch: *mut bot_character_t,
) {
    unsafe {
        for i in 0..MAX_CHARACTERISTICS {
            let cc = (*ch).c.as_mut_ptr().add(i as usize);
            if (*cc).r#type != 0 {
                continue;
            } //end if
              //
            let dc = (*defaultch).c.as_mut_ptr().add(i as usize);
            if (*dc).r#type as c_int == CT_FLOAT as c_int {
                (*cc).r#type = CT_FLOAT;
                (*cc).value._float = (*dc).value._float;
            } else if (*dc).r#type as c_int == CT_INTEGER as c_int {
                (*cc).r#type = CT_INTEGER;
                (*cc).value.integer = (*dc).value.integer;
            } else if (*dc).r#type as c_int == CT_STRING as c_int {
                (*cc).r#type = CT_STRING;
                let len = strlen((*dc).value.string);
                (*cc).value.string = GetMemory(bot, (len + 1) as c_ulong) as *mut c_char;
                strcpy((*cc).value.string, (*dc).value.string);
            } //end else if
        } //end for
    }
}

/// Raven `CheckCharacteristicIndex`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:600-617`
pub fn CheckCharacteristicIndex(bot: &mut BotLib, character: c_int, index: c_int) -> c_int {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return qfalse as c_int;
        } //end if
        if index < 0 || index >= MAX_CHARACTERISTICS {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d does not exist\n".as_ptr() as *mut c_char,
                index,
            );
            return qfalse as c_int;
        } //end if
        let c = (*ch).c.as_mut_ptr().add(index as usize);
        if (*c).r#type == 0 {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d is not initialized\n".as_ptr() as *mut c_char,
                index,
            );
            return qfalse as c_int;
        } //end if
        qtrue as c_int
    }
}

/// Raven `BotFreeCharacter2`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:130-145`
pub fn BotFreeCharacter2(bot: &mut BotLib, handle: c_int) {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"character handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
            return;
        } //end if
        if bot.botcharacters[handle as usize].is_null() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid character %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return;
        } //end if
        BotFreeCharacterStrings(bot, bot.botcharacters[handle as usize]);
        FreeMemory(bot, bot.botcharacters[handle as usize] as *mut ());
        bot.botcharacters[handle as usize] = core::ptr::null_mut();
    }
}

/// Raven `BotInterpolateCharacters`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:499-544`
pub fn BotInterpolateCharacters(
    bot: &mut BotLib,
    handle1: c_int,
    handle2: c_int,
    desiredskill: f32,
) -> c_int {
    unsafe {
        let ch1 = BotCharacterFromHandle(bot, handle1);
        let ch2 = BotCharacterFromHandle(bot, handle2);
        if ch1.is_null() || ch2.is_null() {
            return 0;
        } //end if
          //find a free spot for a character
        let mut handle: c_int = 1;
        while handle <= MAX_CLIENTS as c_int {
            if bot.botcharacters[handle as usize].is_null() {
                break;
            } //end if
            handle += 1;
        } //end for
        if handle > MAX_CLIENTS as c_int {
            return 0;
        } //end if
        let out = GetClearedMemory(
            bot,
            (core::mem::size_of::<bot_character_t>()
                + MAX_CHARACTERISTICS as usize * core::mem::size_of::<bot_characteristic_t>())
                as c_ulong,
        ) as *mut bot_character_t;
        (*out).skill = desiredskill;
        strcpy((*out).filename.as_mut_ptr(), (*ch1).filename.as_ptr());
        bot.botcharacters[handle as usize] = out;

        let scale = (desiredskill - (*ch1).skill) / ((*ch2).skill - (*ch1).skill);
        for i in 0..MAX_CHARACTERISTICS {
            //
            let c1 = (*ch1).c.as_mut_ptr().add(i as usize);
            let c2 = (*ch2).c.as_mut_ptr().add(i as usize);
            let co = (*out).c.as_mut_ptr().add(i as usize);
            if (*c1).r#type as c_int == CT_FLOAT as c_int
                && (*c2).r#type as c_int == CT_FLOAT as c_int
            {
                (*co).r#type = CT_FLOAT;
                (*co).value._float =
                    (*c1).value._float + ((*c2).value._float - (*c1).value._float) * scale;
            } else if (*c1).r#type as c_int == CT_INTEGER as c_int {
                (*co).r#type = CT_INTEGER;
                (*co).value.integer = (*c1).value.integer;
            } else if (*c1).r#type as c_int == CT_STRING as c_int {
                (*co).r#type = CT_STRING;
                let len = strlen((*c1).value.string);
                (*co).value.string = GetMemory(bot, (len + 1) as c_ulong) as *mut c_char;
                strcpy((*co).value.string, (*c1).value.string);
            } //end else if
        } //end for
        handle
    }
}

/// Raven `Characteristic_Float`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:624-649`
pub fn Characteristic_Float(bot: &mut BotLib, character: c_int, index: c_int) -> f32 {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return 0.0;
        } //end if
          //check if the index is in range
        if CheckCharacteristicIndex(bot, character, index) == 0 {
            return 0.0;
        } //end if
        let c = (*ch).c.as_mut_ptr().add(index as usize);
        //an integer will be converted to a float
        if (*c).r#type as c_int == CT_INTEGER as c_int {
            (*c).value.integer as f32
        }
        //floats are just returned
        else if (*c).r#type as c_int == CT_FLOAT as c_int {
            (*c).value._float
        }
        //cannot convert a string pointer to a float
        else {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d is not a float\n".as_ptr() as *mut c_char,
                index,
            );
            0.0
        } //end else if
    }
}

/// Raven `Characteristic_Integer`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:679-703`
pub fn Characteristic_Integer(bot: &mut BotLib, character: c_int, index: c_int) -> c_int {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return 0;
        } //end if
          //check if the index is in range
        if CheckCharacteristicIndex(bot, character, index) == 0 {
            return 0;
        } //end if
        let c = (*ch).c.as_mut_ptr().add(index as usize);
        //an integer will just be returned
        if (*c).r#type as c_int == CT_INTEGER as c_int {
            (*c).value.integer
        }
        //floats are casted to integers
        else if (*c).r#type as c_int == CT_FLOAT as c_int {
            (*c).value._float as c_int
        } else {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d is not a integer\n".as_ptr() as *mut c_char,
                index,
            );
            0
        } //end else if
    }
}

/// Raven `Characteristic_String`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:733-754`
pub fn Characteristic_String(
    bot: &mut BotLib,
    character: c_int,
    index: c_int,
    buf: *mut c_char,
    size: c_int,
) {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return;
        } //end if
          //check if the index is in range
        if CheckCharacteristicIndex(bot, character, index) == 0 {
            return;
        } //end if
        let c = (*ch).c.as_mut_ptr().add(index as usize);
        //an integer will be converted to a float
        if (*c).r#type as c_int == CT_STRING as c_int {
            strncpy(buf, (*c).value.string, (size - 1) as usize);
            *buf.offset((size - 1) as isize) = 0;
            return;
        } else {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d is not a string\n".as_ptr() as *mut c_char,
                index,
            );
            return;
        } //end else if
    }
}

/// Raven `BotFreeCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:152-156`
pub fn BotFreeCharacter(bot: &mut BotLib, handle: c_int) {
    if LibVarGetValue(bot, c"bot_reloadcharacters".as_ptr() as *mut c_char) == 0.0 {
        return;
    } //end if
    BotFreeCharacter2(bot, handle);
}

/// Raven `Characteristic_BFloat`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:656-672`
pub fn Characteristic_BFloat(
    bot: &mut BotLib,
    character: c_int,
    index: c_int,
    min: f32,
    max: f32,
) -> f32 {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return 0.0;
        } //end if
        if min > max {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"cannot bound characteristic %d between %f and %f\n".as_ptr() as *mut c_char,
                index,
                min as core::ffi::c_double,
                max as core::ffi::c_double,
            );
            return 0.0;
        } //end if
        let value = Characteristic_Float(bot, character, index);
        if value < min {
            return min;
        } //end if
        if value > max {
            return max;
        } //end if
        value
    }
}

/// Raven `Characteristic_BInteger`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:710-726`
pub fn Characteristic_BInteger(
    bot: &mut BotLib,
    character: c_int,
    index: c_int,
    min: c_int,
    max: c_int,
) -> c_int {
    unsafe {
        let ch = BotCharacterFromHandle(bot, character);
        if ch.is_null() {
            return 0;
        } //end if
        if min > max {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"cannot bound characteristic %d between %d and %d\n".as_ptr() as *mut c_char,
                index,
                min,
                max,
            );
            return 0;
        } //end if
        let value = Characteristic_Integer(bot, character, index);
        if value < min {
            return min;
        } //end if
        if value > max {
            return max;
        } //end if
        value
    }
}

/// Raven `BotShutdownCharacters`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:761-772`
pub fn BotShutdownCharacters(bot: &mut BotLib) {
    for handle in 1..=MAX_CLIENTS as c_int {
        if !bot.botcharacters[handle as usize].is_null() {
            BotFreeCharacter2(bot, handle);
        } //end if
    } //end for
}

/// Raven `BotLoadCharacterFromFile`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:195-338`
pub fn BotLoadCharacterFromFile(
    bot: &mut BotLib,
    charfile: *mut c_char,
    skill: c_int,
) -> *mut bot_character_t {
    unsafe {
        let mut index: c_int;
        let mut foundcharacter = qfalse as c_int;
        //a bot character is parsed in two phases
        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
        let source: *mut source_t = LoadSourceFile(bot, charfile);
        if source.is_null() {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"counldn't load %s\n".as_ptr() as *mut c_char,
                charfile,
            );
            return core::ptr::null_mut();
        } //end if
        let ch: *mut bot_character_t = GetClearedMemory(
            bot,
            (core::mem::size_of::<bot_character_t>()
                + MAX_CHARACTERISTICS as usize * core::mem::size_of::<bot_characteristic_t>())
                as c_ulong,
        ) as *mut bot_character_t;
        strcpy((*ch).filename.as_mut_ptr(), charfile);
        // §19: `token` is Raven's C stack local, first written by `PC_ReadToken`
        // below before any read — zero-init to give it a defined start value.
        let mut token: token_t = core::mem::zeroed();
        while PC_ReadToken(bot, source, &mut token) != 0 {
            if strcmp(token.string.as_ptr(), c"skill".as_ptr()) == 0 {
                if PC_ExpectTokenType(bot, source, TT_NUMBER, 0, &mut token) == 0 {
                    FreeSource(bot, source);
                    BotFreeCharacterStrings(bot, ch);
                    FreeMemory(bot, ch as *mut ());
                    return core::ptr::null_mut();
                } //end if
                if PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0 {
                    FreeSource(bot, source);
                    BotFreeCharacterStrings(bot, ch);
                    FreeMemory(bot, ch as *mut ());
                    return core::ptr::null_mut();
                } //end if
                  //if it's the correct skill
                if skill < 0 || token.intvalue as c_int == skill {
                    foundcharacter = qtrue as c_int;
                    (*ch).skill = token.intvalue as f32;
                    while PC_ExpectAnyToken(bot, source, &mut token) != 0 {
                        if strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                            break;
                        } //end if
                        if token.r#type != TT_NUMBER || (token.subtype & TT_INTEGER) == 0 {
                            let __m = std::ffi::CString::new(format!(
                                "expected integer index, found {}\n",
                                core::ffi::CStr::from_ptr(token.string.as_ptr()).to_string_lossy()
                            ))
                            .unwrap_or_default();
                            SourceError(bot, source, __m.as_ptr());
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end if
                        index = token.intvalue as c_int;
                        if index < 0 || index > MAX_CHARACTERISTICS {
                            let __m = std::ffi::CString::new(format!(
                                "characteristic index out of range [0, {}]\n",
                                MAX_CHARACTERISTICS
                            ))
                            .unwrap_or_default();
                            SourceError(bot, source, __m.as_ptr());
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end if
                        let c = (*ch).c.as_mut_ptr().add(index as usize);
                        if (*c).r#type != 0 {
                            let __m = std::ffi::CString::new(format!(
                                "characteristic {} already initialized\n",
                                index
                            ))
                            .unwrap_or_default();
                            SourceError(bot, source, __m.as_ptr());
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end if
                        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end if
                        if token.r#type == TT_NUMBER {
                            if (token.subtype & TT_FLOAT) != 0 {
                                (*c).value._float = token.floatvalue as f32;
                                (*c).r#type = CT_FLOAT;
                            } else {
                                (*c).value.integer = token.intvalue as i32;
                                (*c).r#type = CT_INTEGER;
                            } //end else
                        } else if token.r#type == TT_STRING {
                            StripDoubleQuotes(token.string.as_mut_ptr());
                            let len = strlen(token.string.as_ptr());
                            (*c).value.string = GetMemory(bot, (len + 1) as c_ulong) as *mut c_char;
                            strcpy((*c).value.string, token.string.as_ptr());
                            (*c).r#type = CT_STRING;
                        } else {
                            let __m = std::ffi::CString::new(format!(
                                "expected integer, float or string, found {}\n",
                                core::ffi::CStr::from_ptr(token.string.as_ptr()).to_string_lossy()
                            ))
                            .unwrap_or_default();
                            SourceError(bot, source, __m.as_ptr());
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end else
                    } //end if
                    break;
                }
                //end if
                else {
                    let mut indent: c_int = 1;
                    while indent != 0 {
                        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                            FreeSource(bot, source);
                            BotFreeCharacterStrings(bot, ch);
                            FreeMemory(bot, ch as *mut ());
                            return core::ptr::null_mut();
                        } //end if
                        if strcmp(token.string.as_ptr(), c"{".as_ptr()) == 0 {
                            indent += 1;
                        } else if strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                            indent -= 1;
                        } //end else if
                    } //end while
                } //end else
            }
            //end if
            else {
                let __m = std::ffi::CString::new(format!(
                    "unknown definition {}\n",
                    core::ffi::CStr::from_ptr(token.string.as_ptr()).to_string_lossy()
                ))
                .unwrap_or_default();
                SourceError(bot, source, __m.as_ptr());
                FreeSource(bot, source);
                BotFreeCharacterStrings(bot, ch);
                FreeMemory(bot, ch as *mut ());
                return core::ptr::null_mut();
            } //end else
        } //end while
        FreeSource(bot, source);
        //
        if foundcharacter == 0 {
            BotFreeCharacterStrings(bot, ch);
            FreeMemory(bot, ch as *mut ());
            return core::ptr::null_mut();
        } //end if
        ch
    }
}

/// Raven `BotLoadCachedCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:366-472`
///
/// PORT-NOTE(DEBUG): the `#ifdef DEBUG` timing print (`Sys_MilliSeconds`/
/// `bot_developer`) is dropped — retail/NDEBUG build; `DEBUG` has no rosetta
/// row (escalated, not guessed).
pub fn BotLoadCachedCharacter(
    bot: &mut BotLib,
    charfile: *mut c_char,
    skill: f32,
    reload: c_int,
) -> c_int {
    unsafe {
        //find a free spot for a character
        let mut handle: c_int = 1;
        while handle <= MAX_CLIENTS as c_int {
            if bot.botcharacters[handle as usize].is_null() {
                break;
            } //end if
            handle += 1;
        } //end for
        if handle > MAX_CLIENTS as c_int {
            return 0;
        } //end if
          //try to load a cached character with the given skill
        if reload == 0 {
            let cachedhandle = BotFindCachedCharacter(bot, charfile, skill);
            if cachedhandle != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached skill %f from %s\n".as_ptr() as *mut c_char,
                    skill as core::ffi::c_double,
                    charfile,
                );
                return cachedhandle;
            } //end if
        } //end else
          //
        let intskill = (skill + 0.5) as c_int;
        //try to load the character with the given skill
        let mut ch = BotLoadCharacterFromFile(bot, charfile, intskill);
        if !ch.is_null() {
            bot.botcharacters[handle as usize] = ch;
            //
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded skill %d from %s\n".as_ptr() as *mut c_char,
                intskill,
                charfile,
            );
            return handle;
        } //end if
          //
        bot.botimport.Print.unwrap()(
            PRT_WARNING,
            c"couldn't find skill %d in %s\n".as_ptr() as *mut c_char,
            intskill,
            charfile,
        );
        //
        if reload == 0 {
            //try to load a cached default character with the given skill
            let cachedhandle =
                BotFindCachedCharacter(bot, DEFAULT_CHARACTER.as_ptr() as *mut c_char, skill);
            if cachedhandle != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached default skill %d from %s\n".as_ptr() as *mut c_char,
                    intskill,
                    charfile,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load the default character with the given skill
        ch = BotLoadCharacterFromFile(bot, DEFAULT_CHARACTER.as_ptr() as *mut c_char, intskill);
        if !ch.is_null() {
            bot.botcharacters[handle as usize] = ch;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded default skill %d from %s\n".as_ptr() as *mut c_char,
                intskill,
                charfile,
            );
            return handle;
        } //end if
          //
        if reload == 0 {
            //try to load a cached character with any skill
            let cachedhandle = BotFindCachedCharacter(bot, charfile, -1.0);
            if cachedhandle != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached skill %f from %s\n".as_ptr() as *mut c_char,
                    (*bot.botcharacters[cachedhandle as usize]).skill as core::ffi::c_double,
                    charfile,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load a character with any skill
        ch = BotLoadCharacterFromFile(bot, charfile, -1);
        if !ch.is_null() {
            bot.botcharacters[handle as usize] = ch;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded skill %f from %s\n".as_ptr() as *mut c_char,
                (*ch).skill as core::ffi::c_double,
                charfile,
            );
            return handle;
        } //end if
          //
        if reload == 0 {
            //try to load a cached character with any skill
            let cachedhandle =
                BotFindCachedCharacter(bot, DEFAULT_CHARACTER.as_ptr() as *mut c_char, -1.0);
            if cachedhandle != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached default skill %f from %s\n".as_ptr() as *mut c_char,
                    (*bot.botcharacters[cachedhandle as usize]).skill as core::ffi::c_double,
                    charfile,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load a character with any skill
        ch = BotLoadCharacterFromFile(bot, DEFAULT_CHARACTER.as_ptr() as *mut c_char, -1);
        if !ch.is_null() {
            bot.botcharacters[handle as usize] = ch;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded default skill %f from %s\n".as_ptr() as *mut c_char,
                (*ch).skill as core::ffi::c_double,
                charfile,
            );
            return handle;
        } //end if
          //
        bot.botimport.Print.unwrap()(
            PRT_WARNING,
            c"couldn't load any skill from %s\n".as_ptr() as *mut c_char,
            charfile,
        );
        //couldn't load any character
        0
    }
}

/// Raven `BotLoadCharacterSkill`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:479-492`
pub fn BotLoadCharacterSkill(bot: &mut BotLib, charfile: *mut c_char, skill: f32) -> c_int {
    let defaultch = BotLoadCachedCharacter(
        bot,
        DEFAULT_CHARACTER.as_ptr() as *mut c_char,
        skill,
        qfalse as c_int,
    );
    let reload =
        LibVarGetValue(bot, c"bot_reloadcharacters".as_ptr() as *mut c_char) as c_int;
    let ch = BotLoadCachedCharacter(bot, charfile, skill, reload);

    if defaultch != 0 && ch != 0 {
        BotDefaultCharacteristics(
            bot,
            bot.botcharacters[ch as usize],
            bot.botcharacters[defaultch as usize],
        );
    } //end if

    ch
}

/// Raven `BotLoadCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:551-593`
pub fn BotLoadCharacter(bot: &mut BotLib, charfile: *mut c_char, skill: f32) -> c_int {
    unsafe {
        let mut skill = skill;
        //make sure the skill is in the valid range
        if skill < 1.0 {
            skill = 1.0;
        } else if skill > 5.0 {
            skill = 5.0;
        } //end else if
          //skill 1, 4 and 5 should be available in the character files
        if skill == 1.0 || skill == 4.0 || skill == 5.0 {
            return BotLoadCharacterSkill(bot, charfile, skill);
        } //end if
          //check if there's a cached skill
        let handle = BotFindCachedCharacter(bot, charfile, skill);
        if handle != 0 {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded cached skill %f from %s\n".as_ptr() as *mut c_char,
                skill as core::ffi::c_double,
                charfile,
            );
            return handle;
        } //end if
        let firstskill: c_int;
        let secondskill: c_int;
        if skill < 4.0 {
            //load skill 1 and 4
            firstskill = BotLoadCharacterSkill(bot, charfile, 1.0);
            if firstskill == 0 {
                return 0;
            } //end if
            secondskill = BotLoadCharacterSkill(bot, charfile, 4.0);
            if secondskill == 0 {
                return firstskill;
            } //end if
        }
        //end if
        else {
            //load skill 4 and 5
            firstskill = BotLoadCharacterSkill(bot, charfile, 4.0);
            if firstskill == 0 {
                return 0;
            } //end if
            secondskill = BotLoadCharacterSkill(bot, charfile, 5.0);
            if secondskill == 0 {
                return firstskill;
            } //end if
        } //end else
          //interpolate between the two skills
        let handle = BotInterpolateCharacters(bot, firstskill, secondskill, skill);
        if handle == 0 {
            return 0;
        } //end if
          //write the character to the log file
        BotDumpCharacter(bot, bot.botcharacters[handle as usize]);
        //
        handle
    }
}
