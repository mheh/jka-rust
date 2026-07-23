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
//! Ported per the engine C-track packets (`botlib__0502`..`botlib__2456`), then
//! redesigned onto owned shapes (porting-rules §F17; botlib is statically
//! linked, so layout is free): the `bot_character_t*` handle table is an owned
//! slab `Vec<Option<BotCharacter>>` (§B5), the trailing characteristic array is
//! a `Vec<Characteristic>`, and the `type`-tagged `cvalue` union is the
//! `Characteristic` sum type. Raven's malloc'd characteristic strings become
//! owned `String`s (freed on drop), retiring `BotFreeCharacterStrings` and the
//! `GetMemory`/`FreeMemory` calls.
//!
//! Source: `oracle/codemp/botlib/be_ai_char.cpp`.
//!
//! DESTINATION NOTE: the packet order named
//! `crates/mp/engine/botlib/src/be_ai_char.rs`, but `be_ai_char` already
//! exists as a directory module (`be_ai_char/mod.rs`, types-only) — `_fns`
//! escape per `_PREAMBLE.md`'s destination rule.

use core::ffi::{c_char, c_double, c_int};
use std::ffi::{CStr, CString};

use libc::strncpy;
use native_types::{qfalse, qtrue};

use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::limits::MAX_CLIENTS;

use crate::be_ai_char::bot_character_s::BotCharacter;
use crate::be_ai_char::bot_characteristic_s::Characteristic;
use crate::be_ai_char::consts::{DEFAULT_CHARACTER, MAX_CHARACTERISTICS};
use crate::l_libvar_fns::LibVarGetValue;
use crate::l_log_fns::Log_Write;
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_ExpectAnyToken, PC_ExpectTokenString, PC_ExpectTokenType,
    PC_ReadToken, PC_SetBaseFolder, SourceError,
};
use crate::l_script::consts::{TT_FLOAT, TT_INTEGER, TT_NUMBER, TT_STRING};
use crate::l_script::token_s::Token;
use crate::l_script_fns::StripDoubleQuotes;
use crate::BotLib;

/// Raven `BotCharacterFromHandle` — validate that `handle` names a loaded
/// character. Raven returned the `bot_character_t*`; with the owned slab (§B5)
/// callers index `bot.botcharacters[handle]` directly, so this returns whether
/// the handle is valid, keeping Raven's range/null error prints.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:68-81`
pub fn BotCharacterFromHandle(bot: &mut BotLib, handle: c_int) -> bool {
    unsafe {
        if handle <= 0 || handle > MAX_CLIENTS as c_int {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"character handle %d out of range\n".as_ptr() as *mut c_char,
                handle,
            );
            return false;
        } //end if
        if bot.botcharacters[handle as usize].is_none() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid character %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return false;
        } //end if
        true
    }
}

/// Raven `BotFindCachedCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:345-359`
pub fn BotFindCachedCharacter(bot: &mut BotLib, charfile: &str, skill: f32) -> c_int {
    for handle in 1..=MAX_CLIENTS as c_int {
        let ch = match &bot.botcharacters[handle as usize] {
            Some(ch) => ch,
            None => continue,
        }; //end if
        if ch.filename == charfile && (skill < 0.0 || (ch.skill - skill).abs() < 0.01) {
            return handle;
        } //end if
    } //end for
    0
}

/// Raven `BotDumpCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:88-105`
pub fn BotDumpCharacter(bot: &mut BotLib, handle: c_int) {
    let __m =
        CString::new(bot.botcharacters[handle as usize].as_ref().unwrap().filename.clone())
            .unwrap_or_default();
    Log_Write(bot, __m.as_ptr() as *mut c_char);
    // Raven's own format string uses `%d` for the (float) `skill` field —
    // an oracle bug, kept faithful.
    let __m = CString::new(format!(
        "skill {}\n",
        bot.botcharacters[handle as usize].as_ref().unwrap().skill as c_int
    ))
    .unwrap_or_default();
    Log_Write(bot, __m.as_ptr() as *mut c_char);
    Log_Write(bot, c"{\n".as_ptr() as *mut c_char);
    for i in 0..MAX_CHARACTERISTICS as usize {
        let __m = match &bot.botcharacters[handle as usize].as_ref().unwrap().c[i] {
            Characteristic::Integer(v) => {
                Some(CString::new(format!(" {:4} {}\n", i, v)).unwrap_or_default())
            }
            Characteristic::Float(v) => {
                Some(CString::new(format!(" {:4} {}\n", i, *v as f64)).unwrap_or_default())
            }
            Characteristic::Str(s) => {
                Some(CString::new(format!(" {:4} {}\n", i, s)).unwrap_or_default())
            }
            Characteristic::None => None,
        }; //end case
        if let Some(__m) = __m {
            Log_Write(bot, __m.as_ptr() as *mut c_char);
        } //end if
    } //end for
    Log_Write(bot, c"}\n".as_ptr() as *mut c_char);
}

/// Raven `BotDefaultCharacteristics` — fill `ch`'s uninitialized characteristics
/// from `defaultch`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:163-188`
pub fn BotDefaultCharacteristics(bot: &mut BotLib, ch: c_int, defaultch: c_int) {
    for i in 0..MAX_CHARACTERISTICS as usize {
        if !matches!(
            bot.botcharacters[ch as usize].as_ref().unwrap().c[i],
            Characteristic::None
        ) {
            continue;
        } //end if
          // Copy the default's characteristic; an owned `String` clone replaces
          // Raven's `GetMemory`+`strcpy` for the `CT_STRING` case. A `None`
          // default clones to `None`, matching Raven leaving `ch` untouched.
        let dc = bot.botcharacters[defaultch as usize].as_ref().unwrap().c[i].clone();
        bot.botcharacters[ch as usize].as_mut().unwrap().c[i] = dc;
    } //end for
}

/// Raven `CheckCharacteristicIndex`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:600-617`
pub fn CheckCharacteristicIndex(bot: &mut BotLib, character: c_int, index: c_int) -> c_int {
    unsafe {
        if !BotCharacterFromHandle(bot, character) {
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
        if matches!(
            bot.botcharacters[character as usize].as_ref().unwrap().c[index as usize],
            Characteristic::None
        ) {
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
        if bot.botcharacters[handle as usize].is_none() {
            bot.botimport.Print.unwrap()(
                PRT_FATAL,
                c"invalid character %d\n".as_ptr() as *mut c_char,
                handle,
            );
            return;
        } //end if
          // Dropping the owned character frees its characteristic strings (retiring
          // Raven's `BotFreeCharacterStrings`) and the struct (retiring `FreeMemory`).
        bot.botcharacters[handle as usize] = None;
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
    // Raven fetches both handles (each may print) before the null check.
    let ok1 = BotCharacterFromHandle(bot, handle1);
    let ok2 = BotCharacterFromHandle(bot, handle2);
    if !ok1 || !ok2 {
        return 0;
    } //end if
      //find a free spot for a character
    let mut handle: c_int = 1;
    while handle <= MAX_CLIENTS as c_int {
        if bot.botcharacters[handle as usize].is_none() {
            break;
        } //end if
        handle += 1;
    } //end for
    if handle > MAX_CLIENTS as c_int {
        return 0;
    } //end if

    let ch1_skill = bot.botcharacters[handle1 as usize].as_ref().unwrap().skill;
    let ch2_skill = bot.botcharacters[handle2 as usize].as_ref().unwrap().skill;
    let filename = bot.botcharacters[handle1 as usize].as_ref().unwrap().filename.clone();
    let scale = (desiredskill - ch1_skill) / (ch2_skill - ch1_skill);
    let mut c: Vec<Characteristic> = vec![Characteristic::None; MAX_CHARACTERISTICS as usize + 1];
    for i in 0..MAX_CHARACTERISTICS as usize {
        let c1 = &bot.botcharacters[handle1 as usize].as_ref().unwrap().c[i];
        let c2 = &bot.botcharacters[handle2 as usize].as_ref().unwrap().c[i];
        c[i] = match (c1, c2) {
            (Characteristic::Float(f1), Characteristic::Float(f2)) => {
                Characteristic::Float(*f1 + (*f2 - *f1) * scale)
            }
            (Characteristic::Integer(v), _) => Characteristic::Integer(*v),
            (Characteristic::Str(s), _) => Characteristic::Str(s.clone()),
            _ => Characteristic::None,
        }; //end else if
    } //end for
    bot.botcharacters[handle as usize] = Some(BotCharacter {
        filename,
        skill: desiredskill,
        c,
    });
    handle
}

/// Raven `Characteristic_Float`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:624-649`
pub fn Characteristic_Float(bot: &mut BotLib, character: c_int, index: c_int) -> f32 {
    if !BotCharacterFromHandle(bot, character) {
        return 0.0;
    } //end if
      //check if the index is in range
    if CheckCharacteristicIndex(bot, character, index) == 0 {
        return 0.0;
    } //end if
    match &bot.botcharacters[character as usize].as_ref().unwrap().c[index as usize] {
        //an integer will be converted to a float
        Characteristic::Integer(v) => *v as f32,
        //floats are just returned
        Characteristic::Float(v) => *v,
        //cannot convert a string pointer to a float
        _ => {
            unsafe {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"characteristic %d is not a float\n".as_ptr() as *mut c_char,
                    index,
                );
            }
            0.0
        } //end else if
    }
}

/// Raven `Characteristic_Integer`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:679-703`
pub fn Characteristic_Integer(bot: &mut BotLib, character: c_int, index: c_int) -> c_int {
    if !BotCharacterFromHandle(bot, character) {
        return 0;
    } //end if
      //check if the index is in range
    if CheckCharacteristicIndex(bot, character, index) == 0 {
        return 0;
    } //end if
    match &bot.botcharacters[character as usize].as_ref().unwrap().c[index as usize] {
        //an integer will just be returned
        Characteristic::Integer(v) => *v,
        //floats are casted to integers
        Characteristic::Float(v) => *v as c_int,
        _ => {
            unsafe {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"characteristic %d is not a integer\n".as_ptr() as *mut c_char,
                    index,
                );
            }
            0
        } //end else if
    }
}

/// Raven `Characteristic_String` — frozen-signature seam export; the owned
/// `String` is copied out with one bounded `strncpy` (batch-② adapter
/// precedent), preserving Raven's truncation + forced trailing NUL.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:733-754`
pub fn Characteristic_String(
    bot: &mut BotLib,
    character: c_int,
    index: c_int,
    buf: *mut c_char,
    size: c_int,
) {
    if !BotCharacterFromHandle(bot, character) {
        return;
    } //end if
      //check if the index is in range
    if CheckCharacteristicIndex(bot, character, index) == 0 {
        return;
    } //end if
    match &bot.botcharacters[character as usize].as_ref().unwrap().c[index as usize] {
        Characteristic::Str(s) => {
            let s_c = CString::new(s.as_str()).unwrap_or_default();
            unsafe {
                strncpy(buf, s_c.as_ptr(), (size - 1) as usize);
                *buf.offset((size - 1) as isize) = 0;
            }
        }
        _ => unsafe {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"characteristic %d is not a string\n".as_ptr() as *mut c_char,
                index,
            );
        }, //end else if
    }
}

/// Raven `BotFreeCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:152-156`
pub fn BotFreeCharacter(bot: &mut BotLib, handle: c_int) {
    if LibVarGetValue(bot, "bot_reloadcharacters") == 0.0 {
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
    if !BotCharacterFromHandle(bot, character) {
        return 0.0;
    } //end if
    if min > max {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"cannot bound characteristic %d between %f and %f\n".as_ptr() as *mut c_char,
                index,
                min as c_double,
                max as c_double,
            );
        }
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
    if !BotCharacterFromHandle(bot, character) {
        return 0;
    } //end if
    if min > max {
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_ERROR,
                c"cannot bound characteristic %d between %d and %d\n".as_ptr() as *mut c_char,
                index,
                min,
                max,
            );
        }
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

/// Raven `BotShutdownCharacters`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:761-772`
pub fn BotShutdownCharacters(bot: &mut BotLib) {
    for handle in 1..=MAX_CLIENTS as c_int {
        if bot.botcharacters[handle as usize].is_some() {
            BotFreeCharacter2(bot, handle);
        } //end if
    } //end for
}

/// Raven `BotLoadCharacterFromFile` — parse a character file into an owned
/// `BotCharacter`; on any error the local is simply dropped (freeing its
/// characteristic strings), retiring Raven's `BotFreeCharacterStrings`+`FreeMemory`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:195-338`
pub fn BotLoadCharacterFromFile(bot: &mut BotLib, charfile: &str, skill: c_int) -> Option<BotCharacter> {
    let mut index: c_int;
    let mut foundcharacter = false;
    //a bot character is parsed in two phases
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, charfile) {
        Some(s) => s,
        None => {
            let charfile_c = CString::new(charfile).unwrap_or_default();
            unsafe {
                bot.botimport.Print.unwrap()(
                    PRT_ERROR,
                    c"counldn't load %s\n".as_ptr() as *mut c_char,
                    charfile_c.as_ptr() as *mut c_char,
                );
            }
            return None;
        }
    }; //end if
    let mut ch = BotCharacter {
        filename: charfile.to_string(),
        skill: 0.0,
        c: vec![Characteristic::None; MAX_CHARACTERISTICS as usize + 1],
    };
    // §19: `token` is Raven's C stack local, first written by `PC_ReadToken`
    // below before any read — zero-init to give it a defined start value.
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.string == "skill" {
            if PC_ExpectTokenType(bot, &mut source, TT_NUMBER, 0, &mut token) == 0 {
                FreeSource(source);
                return None;
            } //end if
            if PC_ExpectTokenString(bot, &mut source, "{") == 0 {
                FreeSource(source);
                return None;
            } //end if
              //if it's the correct skill
            if skill < 0 || token.intvalue as c_int == skill {
                foundcharacter = true;
                ch.skill = token.intvalue as f32;
                while PC_ExpectAnyToken(bot, &mut source, &mut token) != 0 {
                    if token.string == "}" {
                        break;
                    } //end if
                    if token.type_ != TT_NUMBER || (token.subtype & TT_INTEGER) == 0 {
                        SourceError(
                            bot,
                            &source,
                            &format!("expected integer index, found {}\n", token.string),
                        );
                        FreeSource(source);
                        return None;
                    } //end if
                    index = token.intvalue as c_int;
                    if index < 0 || index > MAX_CHARACTERISTICS {
                        SourceError(
                            bot,
                            &source,
                            &format!(
                                "characteristic index out of range [0, {}]\n",
                                MAX_CHARACTERISTICS
                            ),
                        );
                        FreeSource(source);
                        return None;
                    } //end if
                    if !matches!(ch.c[index as usize], Characteristic::None) {
                        SourceError(
                            bot,
                            &source,
                            &format!("characteristic {} already initialized\n", index),
                        );
                        FreeSource(source);
                        return None;
                    } //end if
                    if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                        FreeSource(source);
                        return None;
                    } //end if
                    if token.type_ == TT_NUMBER {
                        if (token.subtype & TT_FLOAT) != 0 {
                            ch.c[index as usize] = Characteristic::Float(token.floatvalue as f32);
                        } else {
                            ch.c[index as usize] = Characteristic::Integer(token.intvalue as i32);
                        } //end else
                    } else if token.type_ == TT_STRING {
                        StripDoubleQuotes(&mut token.string);
                        ch.c[index as usize] = Characteristic::Str(token.string.clone());
                    } else {
                        SourceError(
                            bot,
                            &source,
                            &format!("expected integer, float or string, found {}\n", token.string),
                        );
                        FreeSource(source);
                        return None;
                    } //end else
                } //end if
                break;
            }
            //end if
            else {
                let mut indent: c_int = 1;
                while indent != 0 {
                    if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                        FreeSource(source);
                        return None;
                    } //end if
                    if token.string == "{" {
                        indent += 1;
                    } else if token.string == "}" {
                        indent -= 1;
                    } //end else if
                } //end while
            } //end else
        }
        //end if
        else {
            SourceError(bot, &source, &format!("unknown definition {}\n", token.string));
            FreeSource(source);
            return None;
        } //end else
    } //end while
    FreeSource(source);
    //
    if !foundcharacter {
        return None;
    } //end if
    Some(ch)
}

/// Raven `BotLoadCachedCharacter`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:366-472`
///
/// The `#ifdef DEBUG` timing print (`Sys_MilliSeconds`/`bot_developer`) is
/// dropped; `DEBUG` is not defined in this retail build.
pub fn BotLoadCachedCharacter(bot: &mut BotLib, charfile: &str, skill: f32, reload: c_int) -> c_int {
    let charfile_c = CString::new(charfile).unwrap_or_default();
    unsafe {
        //find a free spot for a character
        let mut handle: c_int = 1;
        while handle <= MAX_CLIENTS as c_int {
            if bot.botcharacters[handle as usize].is_none() {
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
                    skill as c_double,
                    charfile_c.as_ptr() as *mut c_char,
                );
                return cachedhandle;
            } //end if
        } //end else
          //
        let intskill = (skill + 0.5) as c_int;
        //try to load the character with the given skill
        if let Some(ch) = BotLoadCharacterFromFile(bot, charfile, intskill) {
            bot.botcharacters[handle as usize] = Some(ch);
            //
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded skill %d from %s\n".as_ptr() as *mut c_char,
                intskill,
                charfile_c.as_ptr() as *mut c_char,
            );
            return handle;
        } //end if
          //
        bot.botimport.Print.unwrap()(
            PRT_WARNING,
            c"couldn't find skill %d in %s\n".as_ptr() as *mut c_char,
            intskill,
            charfile_c.as_ptr() as *mut c_char,
        );
        //
        if reload == 0 {
            //try to load a cached default character with the given skill
            let cachedhandle = BotFindCachedCharacter(bot, DEFAULT_CHARACTER, skill);
            if cachedhandle != 0 {
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached default skill %d from %s\n".as_ptr() as *mut c_char,
                    intskill,
                    charfile_c.as_ptr() as *mut c_char,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load the default character with the given skill
        if let Some(ch) = BotLoadCharacterFromFile(bot, DEFAULT_CHARACTER, intskill) {
            bot.botcharacters[handle as usize] = Some(ch);
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded default skill %d from %s\n".as_ptr() as *mut c_char,
                intskill,
                charfile_c.as_ptr() as *mut c_char,
            );
            return handle;
        } //end if
          //
        if reload == 0 {
            //try to load a cached character with any skill
            let cachedhandle = BotFindCachedCharacter(bot, charfile, -1.0);
            if cachedhandle != 0 {
                let skill_v = bot.botcharacters[cachedhandle as usize].as_ref().unwrap().skill;
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached skill %f from %s\n".as_ptr() as *mut c_char,
                    skill_v as c_double,
                    charfile_c.as_ptr() as *mut c_char,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load a character with any skill
        if let Some(ch) = BotLoadCharacterFromFile(bot, charfile, -1) {
            bot.botcharacters[handle as usize] = Some(ch);
            let skill_v = bot.botcharacters[handle as usize].as_ref().unwrap().skill;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded skill %f from %s\n".as_ptr() as *mut c_char,
                skill_v as c_double,
                charfile_c.as_ptr() as *mut c_char,
            );
            return handle;
        } //end if
          //
        if reload == 0 {
            //try to load a cached character with any skill
            let cachedhandle = BotFindCachedCharacter(bot, DEFAULT_CHARACTER, -1.0);
            if cachedhandle != 0 {
                let skill_v = bot.botcharacters[cachedhandle as usize].as_ref().unwrap().skill;
                bot.botimport.Print.unwrap()(
                    PRT_MESSAGE,
                    c"loaded cached default skill %f from %s\n".as_ptr() as *mut c_char,
                    skill_v as c_double,
                    charfile_c.as_ptr() as *mut c_char,
                );
                return cachedhandle;
            } //end if
        } //end if
          //try to load a character with any skill
        if let Some(ch) = BotLoadCharacterFromFile(bot, DEFAULT_CHARACTER, -1) {
            bot.botcharacters[handle as usize] = Some(ch);
            let skill_v = bot.botcharacters[handle as usize].as_ref().unwrap().skill;
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded default skill %f from %s\n".as_ptr() as *mut c_char,
                skill_v as c_double,
                charfile_c.as_ptr() as *mut c_char,
            );
            return handle;
        } //end if
          //
        bot.botimport.Print.unwrap()(
            PRT_WARNING,
            c"couldn't load any skill from %s\n".as_ptr() as *mut c_char,
            charfile_c.as_ptr() as *mut c_char,
        );
        //couldn't load any character
        0
    }
}

/// Raven `BotLoadCharacterSkill`.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:479-492`
pub fn BotLoadCharacterSkill(bot: &mut BotLib, charfile: &str, skill: f32) -> c_int {
    let defaultch = BotLoadCachedCharacter(bot, DEFAULT_CHARACTER, skill, qfalse as c_int);
    let reload = LibVarGetValue(bot, "bot_reloadcharacters") as c_int;
    let ch = BotLoadCachedCharacter(bot, charfile, skill, reload);

    if defaultch != 0 && ch != 0 {
        BotDefaultCharacteristics(bot, ch, defaultch);
    } //end if

    ch
}

/// Raven `BotLoadCharacter` — frozen-signature seam export; `charfile` is
/// decoded to `&str` once here, then threaded through the internals.
///
/// Source: `oracle/codemp/botlib/be_ai_char.cpp:551-593`
pub fn BotLoadCharacter(bot: &mut BotLib, charfile: *mut c_char, skill: f32) -> c_int {
    let charfile_owned = unsafe { CStr::from_ptr(charfile).to_string_lossy().into_owned() };
    let charfile = charfile_owned.as_str();
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
        let charfile_c = CString::new(charfile).unwrap_or_default();
        unsafe {
            bot.botimport.Print.unwrap()(
                PRT_MESSAGE,
                c"loaded cached skill %f from %s\n".as_ptr() as *mut c_char,
                skill as c_double,
                charfile_c.as_ptr() as *mut c_char,
            );
        }
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
    BotDumpCharacter(bot, handle);
    //
    handle
}
