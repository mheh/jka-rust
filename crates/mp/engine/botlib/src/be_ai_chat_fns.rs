//! MP botlib `be_ai_chat.cpp` chat-AI functions.
//!
//! Source: `oracle/codemp/botlib/be_ai_chat.cpp`
//!
//! Redesigned (porting-rules §F17, ruling 2B): the malloc'd pointer-linked
//! parse trees (synonyms, random strings, match templates, reply chats, chat
//! files) become owned `Vec`/`String` shapes; chat files live in the
//! `BotLib.botchats` arena reached by `BotChatHandle` (shared between a bot's
//! chat state and the `ichatdata` cache). The `bot_consolemessage_t` pool +
//! freelist is retained raw: that struct is seam-visible (copied out by
//! `BotNextConsoleMessage`), so its shape and the `firstmessage`/`lastmessage`
//! queue pointers stay. The frozen exports (`BotInitialChat`, `BotReplyChat`,
//! `BotGetChatMessage`, `BotMatchVariable`, `BotFindMatch`, `BotLoadChatFile`,
//! …) keep their C signatures and decode the inbound `*mut c_char` / var slots
//! at the boundary; `bot_match_t` is a frozen qshared seam type, so the match
//! machinery still writes its fixed `string`/`variables` buffer raw.

#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int, c_ulong, c_void};
use std::ffi::{CStr, CString};

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;
use mp_qshared::common::mp::botlib::bot_match_s::{bot_match_t, MAX_MATCHVARIABLES};
use mp_qshared::common::mp::botlib::bot_matchvariable_s::bot_matchvariable_t;
use mp_qshared::common::mp::botlib::botlib_error::{BLERR_CANNOTLOADICHAT, BLERR_NOERROR};
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::limits::MAX_CLIENTS;
use mp_qshared::shared::MAX_QPATH;

use crate::be_ai_chat::bot_chat_s::{BotChat, BotChatHandle};
use crate::be_ai_chat::bot_chatmessage_s::BotChatMessage;
use crate::be_ai_chat::bot_chatstate_s::{BotChatState, MAX_MESSAGE_SIZE};
use crate::be_ai_chat::bot_chattype_s::BotChatType;
use crate::be_ai_chat::bot_ichatdata_s::BotIChatData;
use crate::be_ai_chat::bot_matchpiece_s::BotMatchPiece;
use crate::be_ai_chat::bot_matchtemplate_s::BotMatchTemplate;
use crate::be_ai_chat::bot_randomlist_s::BotRandomList;
use crate::be_ai_chat::bot_replychat_s::BotReplyChat as BotReplyChatRule;
use crate::be_ai_chat::bot_replychatkey_s::BotReplyChatKey;
use crate::be_ai_chat::bot_synonym_s::BotSynonym;
use crate::be_ai_chat::bot_synonymlist_s::BotSynonymList;
use crate::be_ai_chat::chat_consts::{
    CHAT_GENDERFEMALE, CHAT_GENDERLESS, CHAT_GENDERMALE, CHAT_TEAM, CHAT_TELL, MAX_CHATTYPE_NAME,
};
use crate::be_ai_chat::chat_cpp_consts::{
    CHATMESSAGE_RECENTTIME, ESCAPE_CHAR, MT_STRING, MT_VARIABLE, RCKFL_AND, RCKFL_BOTNAMES,
    RCKFL_GENDERFEMALE, RCKFL_GENDERLESS, RCKFL_GENDERMALE, RCKFL_NAME, RCKFL_NOT, RCKFL_STRING,
    RCKFL_VARIABLES,
};
use crate::be_interface::botimport_print;
use crate::l_precomp::source_s::Source;
use crate::l_script::consts::{TT_INTEGER, TT_NAME, TT_NUMBER, TT_PUNCTUATION, TT_STRING};
use crate::l_script::token_s::Token;
use crate::BotLib;

use crate::be_aas_main::AAS_Time;
use crate::be_ea_fns::EA_Command;
use crate::l_libvar_fns::{LibVarGetValue, LibVarString, LibVarValue};
use crate::l_log_fns::{Log_FilePointer, Log_Write};
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory};
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_CheckTokenString, PC_ExpectAnyToken, PC_ExpectTokenString,
    PC_ExpectTokenType, PC_ReadToken, PC_SetBaseFolder, PC_UnreadLastToken, SourceError,
    SourceWarning,
};
use crate::l_script_fns::StripDoubleQuotes;
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};

/// Raven `BotChatStateFromHandle` — validate a chat-state handle and return its
/// slab index (Raven returned the `bot_chatstate_t *`; the arena equivalent is
/// the index). Prints the fatal diagnostics and returns `None` on a bad handle.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:202-215`
pub fn BotChatStateFromHandle(bot: &mut BotLib, handle: c_int) -> Option<usize> {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe { botimport_print(bot, PRT_FATAL, "chat state handle %d out of range\n") };
        return None;
    }
    if bot.botchatstates[handle as usize].is_none() {
        unsafe { botimport_print(bot, PRT_FATAL, "invalid chat state %d\n") };
        return None;
    }
    Some(handle as usize)
}

/// Raven `AllocConsoleMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:251-258`
pub fn AllocConsoleMessage(bot: &mut BotLib) -> *mut bot_consolemessage_t {
    let message = bot.freeconsolemessages;
    unsafe {
        if !bot.freeconsolemessages.is_null() {
            bot.freeconsolemessages = (*bot.freeconsolemessages).next;
        }
        if !bot.freeconsolemessages.is_null() {
            (*bot.freeconsolemessages).prev = core::ptr::null_mut();
        }
    }
    message
}

/// Raven `FreeConsoleMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:266-272`
pub fn FreeConsoleMessage(bot: &mut BotLib, message: *mut bot_consolemessage_t) {
    unsafe {
        if !bot.freeconsolemessages.is_null() {
            (*bot.freeconsolemessages).prev = message;
        }
        (*message).prev = core::ptr::null_mut();
        (*message).next = bot.freeconsolemessages;
        bot.freeconsolemessages = message;
    }
}

/// Raven `IsWhiteSpace`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:384-397`
pub fn IsWhiteSpace(c: c_char) -> c_int {
    let c = c as u8 as char;
    if c.is_ascii_alphanumeric()
        || matches!(
            c,
            '(' | ')' | '?' | ':' | '\'' | '/' | ',' | '.' | '[' | ']' | '-' | '_' | '+' | '='
        )
    {
        0
    } else {
        1
    }
}

/// Raven `BotRemoveTildes` — delete every `~` from a chat message. Redesigned to
/// operate on the owned `String` (Raven shifted the raw buffer left over each
/// `~`); `retain` drops the same characters.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:404-416`
pub fn BotRemoveTildes(message: &mut String) {
    message.retain(|c| c != '~');
}

/// Raven `StringContains`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:447-471`
pub fn StringContains(str1: *mut c_char, str2: *mut c_char, casesensitive: c_int) -> c_int {
    if str1.is_null() || str2.is_null() {
        return -1;
    }
    unsafe {
        let len = libc::strlen(str1) as isize - libc::strlen(str2) as isize;
        let mut index = 0;
        let mut i = 0isize;
        while i <= len {
            let s1 = str1.offset(i);
            let mut j = 0isize;
            loop {
                let c2 = *str2.offset(j);
                if c2 == 0 {
                    break;
                }
                let c1 = *s1.offset(j);
                let matched = if casesensitive != 0 {
                    c1 == c2
                } else {
                    libc::toupper(c1 as c_int) == libc::toupper(c2 as c_int)
                };
                if !matched {
                    break;
                }
                j += 1;
            }
            if *str2.offset(j) == 0 {
                return index;
            }
            i += 1;
            index += 1;
        }
    }
    -1
}

/// Raven `StringContainsWord`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:478-513`
pub fn StringContainsWord(
    str1: *mut c_char,
    str2: *mut c_char,
    casesensitive: c_int,
) -> *mut c_char {
    unsafe {
        let len = libc::strlen(str1) as isize - libc::strlen(str2) as isize;
        let mut i = 0isize;
        while i <= len {
            let mut s1 = str1.offset(i);
            if i != 0 {
                while *s1 != 0
                    && *s1 != b' ' as c_char
                    && *s1 != b'.' as c_char
                    && *s1 != b',' as c_char
                    && *s1 != b'!' as c_char
                {
                    s1 = s1.offset(1);
                }
                if *s1 == 0 {
                    break;
                }
                s1 = s1.offset(1);
            }
            let mut j = 0isize;
            loop {
                let c2 = *str2.offset(j);
                if c2 == 0 {
                    break;
                }
                let c1 = *s1.offset(j);
                let matched = if casesensitive != 0 {
                    c1 == c2
                } else {
                    libc::toupper(c1 as c_int) == libc::toupper(c2 as c_int)
                };
                if !matched {
                    break;
                }
                j += 1;
            }
            if *str2.offset(j) == 0 {
                let sj = *s1.offset(j);
                if sj == 0
                    || sj == b' ' as c_char
                    || sj == b'.' as c_char
                    || sj == b',' as c_char
                    || sj == b'!' as c_char
                {
                    return s1;
                }
            }
            i += 1;
        }
    }
    core::ptr::null_mut()
}

/// Raven `RandomString` — pick a random string from the named random list.
/// Redesigned to return an owned `String` (Raven returned a pointer into the
/// parse tree). The `irand(0, numstrings-1)` call runs on every name match to
/// keep the RNG stream identical (even when the list is empty: `irand(0, -1)`
/// consumes one value and the walk yields nothing, exactly as Raven's).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1031-1053`
pub fn RandomString(common: &mut Common, bot: &mut BotLib, name: &str) -> Option<String> {
    for random in &bot.randomstrings {
        if random.string == name {
            let mut i = common.qrand.irand(0, random.strings.len() as c_int - 1);
            for rs in &random.strings {
                i -= 1;
                if i < 0 {
                    return Some(rs.clone());
                }
            }
        }
    }
    None
}

/// Raven `BotMatchVariable` — frozen-signature seam export; reads one matched
/// variable out of the (frozen) `bot_match_t` into the caller's buffer.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1451-1473`
pub fn BotMatchVariable(
    bot: &mut BotLib,
    r#match: *mut bot_match_t,
    variable: c_int,
    buf: *mut c_char,
    mut size: c_int,
) {
    unsafe {
        if variable < 0 || variable >= MAX_MATCHVARIABLES as c_int {
            botimport_print(bot, PRT_FATAL, "BotMatchVariable: variable out of range\n");
            libc::strcpy(buf, c"".as_ptr());
            return;
        }
        let var = &(*r#match).variables[variable as usize];
        if var.offset >= 0 {
            if var.length < size {
                size = var.length + 1;
            }
            libc::strncpy(
                buf,
                (*r#match).string.as_ptr().offset(var.offset as isize),
                (size - 1) as usize,
            );
            *buf.offset((size - 1) as isize) = 0;
        } else {
            libc::strcpy(buf, c"".as_ptr());
        }
    }
}

/// Raven `BotFindStringInList` — membership test over the integrity-check string
/// list (redesigned from a `bot_stringlist_t` chain to a `&[String]`).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1480-1489`
pub fn BotFindStringInList(list: &[String], string: &str) -> bool {
    list.iter().any(|s| s == string)
}

/// Raven `BotPrintReplyChatKeys`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2556-2590`
pub fn BotPrintReplyChatKeys(bot: &mut BotLib, replychat: &BotReplyChatRule) {
    unsafe {
        botimport_print(bot, PRT_MESSAGE, "[");
        let nkeys = replychat.keys.len();
        for (ki, key) in replychat.keys.iter().enumerate() {
            let flags = key.flags;
            if flags & RCKFL_AND != 0 {
                botimport_print(bot, PRT_MESSAGE, "&");
            } else if flags & RCKFL_NOT != 0 {
                botimport_print(bot, PRT_MESSAGE, "!");
            }
            if flags & RCKFL_NAME != 0 {
                botimport_print(bot, PRT_MESSAGE, "name");
            } else if flags & RCKFL_GENDERFEMALE != 0 {
                botimport_print(bot, PRT_MESSAGE, "female");
            } else if flags & RCKFL_GENDERMALE != 0 {
                botimport_print(bot, PRT_MESSAGE, "male");
            } else if flags & RCKFL_GENDERLESS != 0 {
                botimport_print(bot, PRT_MESSAGE, "it");
            } else if flags & RCKFL_VARIABLES != 0 {
                botimport_print(bot, PRT_MESSAGE, "(");
                let npieces = key.match_.len();
                for (mi, mp) in key.match_.iter().enumerate() {
                    if mp.type_ == MT_STRING {
                        botimport_print(bot, PRT_MESSAGE, "string");
                    } else {
                        botimport_print(bot, PRT_MESSAGE, "var");
                    }
                    if mi + 1 < npieces {
                        botimport_print(bot, PRT_MESSAGE, ", ");
                    }
                }
                botimport_print(bot, PRT_MESSAGE, ")");
            } else if flags & RCKFL_STRING != 0 {
                botimport_print(bot, PRT_MESSAGE, "string");
            }
            if ki + 1 < nkeys {
                botimport_print(bot, PRT_MESSAGE, ", ");
            } else {
                botimport_print(bot, PRT_MESSAGE, "] = ...\n");
            }
        }
        botimport_print(bot, PRT_MESSAGE, "{\n");
    }
}

/// Raven `BotResetChatAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2859-2871`
pub fn BotResetChatAI(bot: &mut BotLib) {
    for rchat in &mut bot.replychats {
        for m in &mut rchat.chatmessages {
            m.time = 0.0;
        }
    }
}

/// Raven `BotRemoveConsoleMessage` — unlink a message from the console queue and
/// return it to the freelist. The chat state is reached by a raw
/// `*mut BotChatState` so `FreeConsoleMessage` can borrow `bot`; the queue and
/// its `bot_consolemessage_t` pool are the retained-raw part of the subsystem.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:279-302`
pub fn BotRemoveConsoleMessage(bot: &mut BotLib, chatstate: c_int, handle: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let cs = bot.botchatstates[idx].as_mut().unwrap() as *mut BotChatState;
    unsafe {
        let mut m = (*cs).firstmessage;
        while !m.is_null() {
            let nextm = (*m).next;
            if (*m).handle == handle {
                if !(*m).next.is_null() {
                    (*(*m).next).prev = (*m).prev;
                } else {
                    (*cs).lastmessage = (*m).prev;
                }
                if !(*m).prev.is_null() {
                    (*(*m).prev).next = (*m).next;
                } else {
                    (*cs).firstmessage = (*m).next;
                }
                FreeConsoleMessage(bot, m);
                (*cs).numconsolemessages -= 1;
                break;
            }
            m = nextm;
        }
    }
}

/// Raven `BotQueueConsoleMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:309-343`
pub fn BotQueueConsoleMessage(
    bot: &mut BotLib,
    chatstate: c_int,
    r#type: c_int,
    message: *mut c_char,
) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let m = AllocConsoleMessage(bot);
    if m.is_null() {
        unsafe { botimport_print(bot, PRT_ERROR, "empty console message heap\n") };
        return;
    }
    let time = AAS_Time(bot);
    let cs = bot.botchatstates[idx].as_mut().unwrap();
    unsafe {
        cs.handle += 1;
        if cs.handle <= 0 || cs.handle > 8192 {
            cs.handle = 1;
        }
        (*m).handle = cs.handle;
        (*m).time = time;
        (*m).r#type = r#type;
        libc::strncpy((*m).message.as_mut_ptr(), message, 256);
        (*m).next = core::ptr::null_mut();
        if !cs.lastmessage.is_null() {
            (*cs.lastmessage).next = m;
            (*m).prev = cs.lastmessage;
            cs.lastmessage = m;
        } else {
            cs.lastmessage = m;
            cs.firstmessage = m;
            (*m).prev = core::ptr::null_mut();
        }
        cs.numconsolemessages += 1;
    }
}

/// Raven `BotNextConsoleMessage` — copy the first queued message out to the
/// caller's (seam-provided) `bot_consolemessage_t`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:350-363`
pub fn BotNextConsoleMessage(
    bot: &mut BotLib,
    chatstate: c_int,
    cm: *mut bot_consolemessage_t,
) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return 0,
    };
    let cs = bot.botchatstates[idx].as_ref().unwrap();
    unsafe {
        if !cs.firstmessage.is_null() {
            Com_Memcpy(
                cm as *mut (),
                cs.firstmessage as *const (),
                core::mem::size_of::<bot_consolemessage_t>(),
            );
            (*cm).next = core::ptr::null_mut();
            (*cm).prev = core::ptr::null_mut();
            return (*cm).handle;
        }
    }
    0
}

/// Raven `BotNumConsoleMessages`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:370-377`
pub fn BotNumConsoleMessages(bot: &mut BotLib, chatstate: c_int) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return 0,
    };
    bot.botchatstates[idx].as_ref().unwrap().numconsolemessages
}

/// Raven `UnifyWhiteSpaces`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:423-440`
pub fn UnifyWhiteSpaces(string: *mut c_char) {
    unsafe {
        let mut ptr = string;
        let mut oldptr;
        while *ptr != 0 {
            oldptr = ptr;
            while *ptr != 0 && IsWhiteSpace(*ptr) != 0 {
                ptr = ptr.offset(1);
            }
            if ptr > oldptr {
                if oldptr > string && *ptr != 0 {
                    *oldptr = b' ' as c_char;
                    oldptr = oldptr.offset(1);
                }
                if ptr > oldptr {
                    let len = libc::strlen(ptr);
                    libc::memmove(oldptr as *mut c_void, ptr as *const c_void, len + 1);
                    ptr = oldptr;
                }
            }
            while *ptr != 0 && IsWhiteSpace(*ptr) == 0 {
                ptr = ptr.offset(1);
            }
        }
    }
}

/// Raven `StringReplaceWords`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:520-546`
pub fn StringReplaceWords(string: *mut c_char, synonym: *mut c_char, replacement: *mut c_char) {
    unsafe {
        let mut str_ = StringContainsWord(string, synonym, 0);
        while !str_.is_null() {
            let mut str2 = StringContainsWord(string, replacement, 0);
            while !str2.is_null() {
                if str2 <= str_ && str_ < str2.offset(libc::strlen(replacement) as isize) {
                    break;
                }
                str2 = StringContainsWord(str2.offset(1), replacement, 0);
            }
            if str2.is_null() {
                let syn_len = libc::strlen(synonym) as isize;
                let rep_len = libc::strlen(replacement) as isize;
                let tail = str_.offset(syn_len);
                let tail_len = libc::strlen(tail);
                libc::memmove(
                    str_.offset(rep_len) as *mut c_void,
                    tail as *const c_void,
                    tail_len + 1,
                );
                Com_Memcpy(str_ as *mut (), replacement as *const (), rep_len as usize);
            }
            str_ = StringContainsWord(str_.offset(libc::strlen(replacement) as isize), synonym, 0);
        }
    }
}

/// Raven `BotDumpSynonymList`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:553-571`
pub fn BotDumpSynonymList(bot: &mut BotLib, synlist: &[BotSynonymList]) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        for syn in synlist {
            libc::fprintf(fp, c"%ld : [".as_ptr(), syn.context);
            let nsyn = syn.synonyms.len();
            for (i, synonym) in syn.synonyms.iter().enumerate() {
                let string_c = CString::new(synonym.string.as_str()).unwrap_or_default();
                libc::fprintf(
                    fp,
                    c"(\"%s\", %1.2f)".as_ptr(),
                    string_c.as_ptr(),
                    synonym.weight as f64,
                );
                if i + 1 < nsyn {
                    libc::fprintf(fp, c", ".as_ptr());
                }
            }
            libc::fprintf(fp, c"]\n".as_ptr());
        }
    }
}

/// Raven `BotReplaceSynonyms` — frozen-signature seam export; replaces every
/// non-canonical synonym in `string` (edited in place) with the first synonym
/// of each matching list.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:744-757`
pub fn BotReplaceSynonyms(bot: &mut BotLib, string: *mut c_char, context: c_ulong) {
    for syn in &bot.synonyms {
        if syn.context & context != 0 {
            if let Some(first) = syn.synonyms.first() {
                let replacement_c = CString::new(first.string.as_str()).unwrap_or_default();
                for synonym in syn.synonyms.iter().skip(1) {
                    let synonym_c = CString::new(synonym.string.as_str()).unwrap_or_default();
                    StringReplaceWords(
                        string,
                        synonym_c.as_ptr() as *mut c_char,
                        replacement_c.as_ptr() as *mut c_char,
                    );
                }
            }
        }
    }
}

/// Raven `BotReplaceWeightedSynonyms`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:764-790`
pub fn BotReplaceWeightedSynonyms(
    common: &mut Common,
    bot: &mut BotLib,
    string: *mut c_char,
    context: c_ulong,
) {
    for syn in &bot.synonyms {
        if syn.context & context != 0 {
            let weight = common.qrand.flrand(0.0, 1.0) * syn.totalweight;
            if weight != 0.0 {
                let mut curweight = 0.0f32;
                let mut replacement: Option<usize> = None;
                for (i, r) in syn.synonyms.iter().enumerate() {
                    curweight += r.weight;
                    if weight < curweight {
                        replacement = Some(i);
                        break;
                    }
                }
                if let Some(ri) = replacement {
                    let repl_c = CString::new(syn.synonyms[ri].string.as_str()).unwrap_or_default();
                    for (i, synonym) in syn.synonyms.iter().enumerate() {
                        if i != ri {
                            let syn_c = CString::new(synonym.string.as_str()).unwrap_or_default();
                            StringReplaceWords(
                                string,
                                syn_c.as_ptr() as *mut c_char,
                                repl_c.as_ptr() as *mut c_char,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Raven `BotReplaceReplySynonyms` — like `BotReplaceSynonyms` but scans word by
/// word and replaces only a leading non-canonical synonym per word.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:797-838`
pub fn BotReplaceReplySynonyms(bot: &mut BotLib, string: *mut c_char, context: c_ulong) {
    unsafe {
        let mut str1 = string;
        loop {
            while *str1 != 0 && *str1 <= b' ' as c_char {
                str1 = str1.offset(1);
            }
            if *str1 == 0 {
                break;
            }
            'syn: for syn in &bot.synonyms {
                if syn.context & context == 0 {
                    continue;
                }
                let first = match syn.synonyms.first() {
                    Some(f) => f,
                    None => continue,
                };
                let replacement_c = CString::new(first.string.as_str()).unwrap_or_default();
                for synonym in syn.synonyms.iter().skip(1) {
                    let synonym_c = CString::new(synonym.string.as_str()).unwrap_or_default();
                    let str2 = StringContainsWord(str1, synonym_c.as_ptr() as *mut c_char, 0);
                    if str2.is_null() || str2 != str1 {
                        continue;
                    }
                    let str2b = StringContainsWord(str1, replacement_c.as_ptr() as *mut c_char, 0);
                    if !str2b.is_null() && str2b == str1 {
                        continue;
                    }
                    let syn_len = libc::strlen(synonym_c.as_ptr()) as isize;
                    let rep_len = libc::strlen(replacement_c.as_ptr()) as isize;
                    let tail = str1.offset(syn_len);
                    let tail_len = libc::strlen(tail);
                    libc::memmove(
                        str1.offset(rep_len) as *mut c_void,
                        tail as *const c_void,
                        tail_len + 1,
                    );
                    Com_Memcpy(
                        str1 as *mut (),
                        replacement_c.as_ptr() as *const (),
                        rep_len as usize,
                    );
                    break 'syn;
                }
            }
            while *str1 != 0 && *str1 > b' ' as c_char {
                str1 = str1.offset(1);
            }
            if *str1 == 0 {
                break;
            }
        }
    }
}

/// Raven `BotDumpRandomStringList`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:904-922`
pub fn BotDumpRandomStringList(bot: &mut BotLib, randomlist: &[BotRandomList]) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        for random in randomlist {
            let name_c = CString::new(random.string.as_str()).unwrap_or_default();
            libc::fprintf(fp, c"%s = {".as_ptr(), name_c.as_ptr());
            let nstrings = random.strings.len();
            for (i, rs) in random.strings.iter().enumerate() {
                let rs_c = CString::new(rs.as_str()).unwrap_or_default();
                libc::fprintf(fp, c"\"%s\"".as_ptr(), rs_c.as_ptr());
                if i + 1 < nstrings {
                    libc::fprintf(fp, c", ".as_ptr());
                } else {
                    libc::fprintf(fp, c"}\n".as_ptr());
                }
            }
        }
    }
}

/// Raven `BotDumpMatchTemplates`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1060-1090`
pub fn BotDumpMatchTemplates(bot: &mut BotLib, matches: &[BotMatchTemplate]) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        for mt in matches {
            libc::fprintf(fp, c"{ ".as_ptr());
            let npieces = mt.first.len();
            for (pi, mp) in mt.first.iter().enumerate() {
                if mp.type_ == MT_STRING {
                    let nstrings = mp.strings.len();
                    for (si, ms) in mp.strings.iter().enumerate() {
                        let ms_c = CString::new(ms.as_str()).unwrap_or_default();
                        libc::fprintf(fp, c"\"%s\"".as_ptr(), ms_c.as_ptr());
                        if si + 1 < nstrings {
                            libc::fprintf(fp, c"|".as_ptr());
                        }
                    }
                } else if mp.type_ == MT_VARIABLE {
                    libc::fprintf(fp, c"%d".as_ptr(), mp.variable);
                }
                if pi + 1 < npieces {
                    libc::fprintf(fp, c", ".as_ptr());
                }
            }
            libc::fprintf(fp, c" = (%d, %d);}\n".as_ptr(), mt.type_, mt.subtype);
        }
    }
}

/// Raven `StringsMatch` — match a `bot_match_t` (frozen seam type; its fixed
/// `string`/`variables` buffer is written raw) against a `&[BotMatchPiece]`
/// template, recording matched-variable offsets/lengths. The `firststring`
/// chain is now a `Vec<String>`; the string arithmetic is otherwise Raven's.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1340-1410`
pub fn StringsMatch(pieces: &[BotMatchPiece], r#match: *mut bot_match_t) -> c_int {
    unsafe {
        let base = (*r#match).string.as_ptr();
        let mut lastvariable: isize = -1;
        let mut strptr = (*r#match).string.as_mut_ptr();
        for mp in pieces {
            if mp.type_ == MT_STRING {
                let mut newstrptr: *mut c_char = core::ptr::null_mut();
                let mut matched_len: usize = 0;
                for ms in &mp.strings {
                    if ms.is_empty() {
                        newstrptr = strptr;
                        matched_len = 0;
                        break;
                    }
                    let ms_c = CString::new(ms.as_str()).unwrap_or_default();
                    let index = StringContains(strptr, ms_c.as_ptr() as *mut c_char, 0);
                    if index >= 0 {
                        newstrptr = strptr.offset(index as isize);
                        if lastvariable >= 0 {
                            (*r#match).variables[lastvariable as usize].length =
                                ((newstrptr as isize) - (base as isize)) as i32
                                    - (*r#match).variables[lastvariable as usize].offset as i32;
                            lastvariable = -1;
                            matched_len = ms.len();
                            break;
                        } else if index == 0 {
                            matched_len = ms.len();
                            break;
                        }
                        newstrptr = core::ptr::null_mut();
                    }
                }
                if newstrptr.is_null() {
                    return 0;
                }
                strptr = newstrptr.offset(matched_len as isize);
            } else if mp.type_ == MT_VARIABLE {
                let var = mp.variable as usize;
                (*r#match).variables[var].offset =
                    ((strptr as isize) - (base as isize)) as c_char;
                lastvariable = var as isize;
            }
        }
        if lastvariable >= 0 || libc::strlen(strptr) == 0 {
            if lastvariable >= 0 {
                let off = (*r#match).variables[lastvariable as usize].offset;
                (*r#match).variables[lastvariable as usize].length =
                    libc::strlen(base.offset(off as isize)) as i32;
            }
            return 1;
        }
    }
    0
}

/// Raven `BotDumpReplyChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1619-1667`
pub fn BotDumpReplyChat(bot: &mut BotLib, replychat: &[BotReplyChatRule]) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        libc::fprintf(fp, c"BotDumpReplyChat:\n".as_ptr());
        for rp in replychat {
            libc::fprintf(fp, c"[".as_ptr());
            let nkeys = rp.keys.len();
            for (ki, key) in rp.keys.iter().enumerate() {
                let flags = key.flags;
                if flags & RCKFL_AND != 0 {
                    libc::fprintf(fp, c"&".as_ptr());
                } else if flags & RCKFL_NOT != 0 {
                    libc::fprintf(fp, c"!".as_ptr());
                }
                if flags & RCKFL_NAME != 0 {
                    libc::fprintf(fp, c"name".as_ptr());
                } else if flags & RCKFL_GENDERFEMALE != 0 {
                    libc::fprintf(fp, c"female".as_ptr());
                } else if flags & RCKFL_GENDERMALE != 0 {
                    libc::fprintf(fp, c"male".as_ptr());
                } else if flags & RCKFL_GENDERLESS != 0 {
                    libc::fprintf(fp, c"it".as_ptr());
                } else if flags & RCKFL_VARIABLES != 0 {
                    libc::fprintf(fp, c"(".as_ptr());
                    let npieces = key.match_.len();
                    for (mi, mp) in key.match_.iter().enumerate() {
                        if mp.type_ == MT_STRING {
                            let ms_c = CString::new(
                                mp.strings.first().map(|s| s.as_str()).unwrap_or(""),
                            )
                            .unwrap_or_default();
                            libc::fprintf(fp, c"\"%s\"".as_ptr(), ms_c.as_ptr());
                        } else {
                            libc::fprintf(fp, c"%d".as_ptr(), mp.variable);
                        }
                        if mi + 1 < npieces {
                            libc::fprintf(fp, c", ".as_ptr());
                        }
                    }
                    libc::fprintf(fp, c")".as_ptr());
                } else if flags & RCKFL_STRING != 0 {
                    let key_c = CString::new(key.string.as_str()).unwrap_or_default();
                    libc::fprintf(fp, c"\"%s\"".as_ptr(), key_c.as_ptr());
                }
                if ki + 1 < nkeys {
                    libc::fprintf(fp, c", ".as_ptr());
                } else {
                    libc::fprintf(fp, c"] = %1.0f\n".as_ptr(), rp.priority as f64);
                }
            }
            libc::fprintf(fp, c"{\n".as_ptr());
            for cm in &rp.chatmessages {
                let cm_c = CString::new(cm.chatmessage.as_str()).unwrap_or_default();
                libc::fprintf(fp, c"\t\"%s\";\n".as_ptr(), cm_c.as_ptr());
            }
            libc::fprintf(fp, c"}\n".as_ptr());
        }
    }
}

/// Raven `BotCheckValidReplyChatKeySet` — emit warnings for suspect reply-chat
/// key sets. Reworked over the owned `&[BotReplyChatKey]` (each key's `string`
/// is a `String`, its `match` a `Vec<BotMatchPiece>`); the diagnostic text and
/// conditions match Raven.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1704-1804`
pub fn BotCheckValidReplyChatKeySet(bot: &mut BotLib, source: &mut Source, keys: &[BotReplyChatKey]) {
    let mut allprefixed = true;
    let mut hasvariableskey = false;
    let mut hasstringkey = false;
    for (i, key) in keys.iter().enumerate() {
        let flags = key.flags;
        if flags & (RCKFL_AND | RCKFL_NOT) == 0 {
            allprefixed = false;
            if flags & RCKFL_VARIABLES != 0 {
                for m in &key.match_ {
                    if m.type_ == MT_VARIABLE {
                        hasvariableskey = true;
                    }
                }
            } else if flags & RCKFL_STRING != 0 {
                hasstringkey = true;
            }
        } else if (flags & RCKFL_AND != 0) && (flags & RCKFL_STRING != 0) {
            for (j, key2) in keys.iter().enumerate() {
                if j == i {
                    continue;
                }
                if key2.flags & RCKFL_NOT != 0 {
                    continue;
                }
                if key2.flags & RCKFL_VARIABLES != 0 {
                    let mut found = false;
                    for m in &key2.match_ {
                        if m.type_ == MT_STRING {
                            let mut hit = false;
                            for ms in &m.strings {
                                let ms_c = CString::new(ms.as_str()).unwrap_or_default();
                                let key_c = CString::new(key.string.as_str()).unwrap_or_default();
                                if StringContains(
                                    ms_c.as_ptr() as *mut c_char,
                                    key_c.as_ptr() as *mut c_char,
                                    0,
                                ) != -1
                                {
                                    hit = true;
                                    break;
                                }
                            }
                            if hit {
                                found = true;
                                break;
                            }
                        } else if m.type_ == MT_VARIABLE {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        SourceWarning(
                            bot,
                            source,
                            &format!(
                                "one of the match templates does not leave space for the key {} with the & prefix",
                                key.string
                            ),
                        );
                    }
                }
            }
        }
        if (flags & RCKFL_NOT != 0) && (flags & RCKFL_STRING != 0) {
            for (j, key2) in keys.iter().enumerate() {
                if j == i {
                    continue;
                }
                if key2.flags & RCKFL_NOT != 0 {
                    continue;
                }
                if key2.flags & RCKFL_STRING != 0 {
                    let key2_c = CString::new(key2.string.as_str()).unwrap_or_default();
                    let key_c = CString::new(key.string.as_str()).unwrap_or_default();
                    if StringContains(
                        key2_c.as_ptr() as *mut c_char,
                        key_c.as_ptr() as *mut c_char,
                        0,
                    ) != -1
                    {
                        SourceWarning(
                            bot,
                            source,
                            &format!(
                                "the key {} with prefix ! is inside the key {}",
                                key.string, key2.string
                            ),
                        );
                    }
                } else if key2.flags & RCKFL_VARIABLES != 0 {
                    for m in &key2.match_ {
                        if m.type_ == MT_STRING {
                            for ms in &m.strings {
                                let ms_c = CString::new(ms.as_str()).unwrap_or_default();
                                let key_c = CString::new(key.string.as_str()).unwrap_or_default();
                                if StringContains(
                                    ms_c.as_ptr() as *mut c_char,
                                    key_c.as_ptr() as *mut c_char,
                                    0,
                                ) != -1
                                {
                                    SourceWarning(
                                        bot,
                                        source,
                                        &format!(
                                            "the key {} with prefix ! is inside the match template string {}",
                                            key.string, ms
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if allprefixed {
        SourceWarning(bot, source, "all keys have a & or ! prefix");
    }
    if hasvariableskey && hasstringkey {
        SourceWarning(
            bot,
            source,
            "variables from the match template(s) could be invalid when outputting one of the chat messages",
        );
    }
}

/// Raven `BotDumpInitialChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1971-1989`
pub fn BotDumpInitialChat(bot: &mut BotLib, chat: &BotChat) {
    Log_Write(bot, c"{".as_ptr() as *mut c_char);
    for t in &chat.types {
        Log_Write(bot, c" type".as_ptr() as *mut c_char);
        Log_Write(bot, c" {".as_ptr() as *mut c_char);
        Log_Write(bot, c"  numchatmessages".as_ptr() as *mut c_char);
        for _m in &t.chatmessages {
            Log_Write(bot, c"  chatmessage".as_ptr() as *mut c_char);
        }
        Log_Write(bot, c" }".as_ptr() as *mut c_char);
    }
    Log_Write(bot, c"}".as_ptr() as *mut c_char);
}

/// Raven `BotFreeChatFile` — drop a chat state's loaded chat (arena slot).
///
/// In the cached path (`bot_reloadcharacters == 0`) the freed handle may also
/// be held by an `ichatdata` entry (Raven's shared pointer); Raven only calls
/// this on the first load (when `chat` is null) or in the non-cached path, so
/// the alias is never double-freed live. If a chat state ever reloaded a
/// different cached chat, Raven would leave `ichatdata` dangling (UB); here the
/// stale handle would panic on a later resolve — kept out of the live path.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2168-2176`
pub fn BotFreeChatFile(bot: &mut BotLib, chatstate: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    if let Some(h) = bot.botchatstates[idx].as_ref().unwrap().chat {
        bot.botchats[h.0] = None;
    }
    bot.botchatstates[idx].as_mut().unwrap().chat = None;
}

/// Raven `BotChooseInitialChatMessage` — pick a chat message of the given type
/// from the (arena) chat, marking the chosen one recent. Returns an owned
/// `String` (Raven returned a pointer into the chat). `AAS_Time` is constant
/// within the call, so it is captured once.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2390-2441`
pub fn BotChooseInitialChatMessage(
    common: &mut Common,
    bot: &mut BotLib,
    chat_handle: BotChatHandle,
    r#type: &str,
) -> Option<String> {
    let time = AAS_Time(bot);
    let recent = CHATMESSAGE_RECENTTIME as f32;
    let chat = bot.botchat_mut(chat_handle);
    for t in &mut chat.types {
        if t.name.eq_ignore_ascii_case(r#type) {
            let numchatmessages = t.chatmessages.iter().filter(|m| m.time <= time).count() as c_int;
            if numchatmessages <= 0 {
                let mut besttime = 0.0f32;
                let mut best: Option<usize> = None;
                for (i, m) in t.chatmessages.iter().enumerate() {
                    if besttime == 0.0 || m.time < besttime {
                        best = Some(i);
                        besttime = m.time;
                    }
                }
                return best.map(|i| t.chatmessages[i].chatmessage.clone());
            } else {
                let mut n = common.qrand.irand(0, numchatmessages - 1);
                for m in t.chatmessages.iter_mut() {
                    if m.time <= time {
                        n -= 1;
                        if n < 0 {
                            m.time = time + recent;
                            return Some(m.chatmessage.clone());
                        }
                    }
                }
                return None;
            }
        }
    }
    None
}

/// Raven `BotChatLength`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2754-2761`
pub fn BotChatLength(bot: &mut BotLib, chatstate: c_int) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return 0,
    };
    bot.botchatstates[idx].as_ref().unwrap().chatmessage.len() as c_int
}

/// Raven `BotGetChatMessage` — frozen-signature seam export; strips tildes and
/// copies the (owned) chat message out into the caller's buffer with Raven's
/// `size`-bounded truncation, then clears it.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2804-2816`
pub fn BotGetChatMessage(bot: &mut BotLib, chatstate: c_int, buf: *mut c_char, size: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let cs = bot.botchatstates[idx].as_mut().unwrap();
    BotRemoveTildes(&mut cs.chatmessage);
    let msg_c = CString::new(cs.chatmessage.as_str()).unwrap_or_default();
    unsafe {
        libc::strncpy(buf, msg_c.as_ptr(), (size - 1) as usize);
        *buf.offset((size - 1) as isize) = 0;
    }
    cs.chatmessage.clear();
}

/// Raven `BotSetChatGender`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2823-2835`
pub fn BotSetChatGender(bot: &mut BotLib, chatstate: c_int, gender: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let cs = bot.botchatstates[idx].as_mut().unwrap();
    cs.gender = match gender {
        g if g == CHAT_GENDERFEMALE => CHAT_GENDERFEMALE,
        g if g == CHAT_GENDERMALE => CHAT_GENDERMALE,
        _ => CHAT_GENDERLESS,
    };
}

/// Raven `BotSetChatName` — frozen-signature seam export; decodes the inbound
/// name and stores it truncated to 31 bytes (Raven `strncpy(name, ,32);
/// name[31]=0`). Chat names are ASCII, so `chars().take(31)` matches the byte
/// truncation.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2842-2852`
pub fn BotSetChatName(bot: &mut BotLib, chatstate: c_int, name: *mut c_char, client: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let name_str = unsafe { CStr::from_ptr(name).to_string_lossy().into_owned() };
    let cs = bot.botchatstates[idx].as_mut().unwrap();
    cs.client = client;
    cs.name = name_str.chars().take(31).collect();
}

/// Raven `BotFreeMatchTemplates` / `BotFreeMatchPieces` / `BotFreeReplyChat` —
/// dissolved: the owned `Vec`/`String` trees free by `Drop`. Loaders that hit
/// an error return early and the partially-built local `Vec` drops.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1097-1115,1224-1234,1674-1697`
// (No function body — see the note above.)

/// Raven `BotFindMatch` — frozen-signature seam export; fills the (frozen)
/// `bot_match_t` from the first matching template.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1417-1444`
pub fn BotFindMatch(
    bot: &mut BotLib,
    str: *mut c_char,
    r#match: *mut bot_match_t,
    context: c_ulong,
) -> c_int {
    unsafe {
        libc::strncpy((*r#match).string.as_mut_ptr(), str, MAX_MESSAGE_SIZE - 1);
        let mut len = libc::strlen((*r#match).string.as_ptr());
        while len > 0 && *(*r#match).string.as_ptr().add(len - 1) == b'\n' as c_char {
            *(*r#match).string.as_mut_ptr().add(len - 1) = 0;
            len -= 1;
        }
        for ms in &bot.matchtemplates {
            if ms.context & context != 0 {
                for i in 0..MAX_MATCHVARIABLES {
                    (*r#match).variables[i].offset = -1;
                }
                if StringsMatch(&ms.first, r#match) != 0 {
                    (*r#match).r#type = ms.type_;
                    (*r#match).subtype = ms.subtype;
                    return 1;
                }
            }
        }
    }
    0
}

/// Raven `BotCheckChatMessageIntegrety` — scan a message's `\x01r` random refs;
/// unknown ones are logged and accumulated into `stringlist` (a `Vec<String>`
/// set, replacing Raven's `bot_stringlist_t` chain).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1496-1560`
pub fn BotCheckChatMessageIntegrety(
    common: &mut Common,
    bot: &mut BotLib,
    message: &str,
    stringlist: &mut Vec<String>,
) {
    let msg = message.as_bytes();
    let mut i = 0;
    while i < msg.len() {
        if msg[i] == ESCAPE_CHAR {
            i += 1;
            match msg.get(i).copied() {
                Some(b'v') => {
                    i += 1;
                    while i < msg.len() && msg[i] != ESCAPE_CHAR {
                        i += 1;
                    }
                    if i < msg.len() {
                        i += 1;
                    }
                }
                Some(b'r') => {
                    i += 1;
                    let start = i;
                    while i < msg.len() && msg[i] != ESCAPE_CHAR {
                        i += 1;
                    }
                    let name = String::from_utf8_lossy(&msg[start..i]).into_owned();
                    if i < msg.len() {
                        i += 1;
                    }
                    if RandomString(common, bot, &name).is_none()
                        && !BotFindStringInList(stringlist, &name)
                    {
                        Log_Write(bot, c"MISSING RANDOM".as_ptr() as *mut c_char);
                        stringlist.push(name);
                    }
                }
                _ => unsafe {
                    botimport_print(
                        bot,
                        PRT_FATAL,
                        "BotCheckChatMessageIntegrety: invalid escape char\n",
                    );
                },
            }
        } else {
            i += 1;
        }
    }
}

/// Raven `BotNumInitialChats` — frozen-signature seam export; count the messages
/// of a chat type. Raven derefs `cs->chat` without a null check (UB when no
/// chat is loaded); here a `None` handle returns `0` (defined choice, §F19).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2448-2468`
pub fn BotNumInitialChats(bot: &mut BotLib, chatstate: c_int, r#type: *mut c_char) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return 0,
    };
    let chat_handle = match bot.botchatstates[idx].as_ref().unwrap().chat {
        Some(h) => h,
        None => return 0,
    };
    let type_str = unsafe { CStr::from_ptr(r#type).to_string_lossy().into_owned() };
    let mut result: Option<c_int> = None;
    for t in &bot.botchat(chat_handle).types {
        if t.name.eq_ignore_ascii_case(&type_str) {
            result = Some(t.chatmessages.len() as c_int);
            break;
        }
    }
    if let Some(n) = result {
        if LibVarGetValue(bot, "bot_testichat") != 0.0 {
            unsafe {
                botimport_print(bot, PRT_MESSAGE, "chat lines\n");
                botimport_print(bot, PRT_MESSAGE, "-------------------\n");
            }
        }
        return n;
    }
    0
}

/// Raven `BotEnterChat` — frozen-signature seam export; emits the pending chat
/// message via `EA_Command`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2768-2797`
pub fn BotEnterChat(bot: &mut BotLib, chatstate: c_int, clientto: c_int, sendto: c_int) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    if bot.botchatstates[idx].as_ref().unwrap().chatmessage.is_empty() {
        return;
    }
    BotRemoveTildes(&mut bot.botchatstates[idx].as_mut().unwrap().chatmessage);
    if LibVarGetValue(bot, "bot_testichat") != 0.0 {
        unsafe { botimport_print(bot, PRT_MESSAGE, "chatmessage\n") };
    } else {
        let msg = bot.botchatstates[idx].as_ref().unwrap().chatmessage.clone();
        let client = bot.botchatstates[idx].as_ref().unwrap().client;
        let cmdline = match sendto {
            t if t == CHAT_TEAM => format!("say_team {msg}"),
            t if t == CHAT_TELL => format!("tell {clientto} {msg}"),
            _ => format!("say {msg}"),
        };
        let mut cstring = CString::new(cmdline)
            .unwrap_or_default()
            .into_bytes_with_nul();
        EA_Command(bot, client, cstring.as_mut_ptr() as *mut c_char);
    }
    bot.botchatstates[idx].as_mut().unwrap().chatmessage.clear();
}

/// Raven `BotAllocChatState` — take the first free chat-state slab slot.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2878-2891`
pub fn BotAllocChatState(bot: &mut BotLib) -> c_int {
    for i in 1..=MAX_CLIENTS {
        if bot.botchatstates[i].is_none() {
            bot.botchatstates[i] = Some(Default::default());
            return i as c_int;
        }
    }
    0
}

/// Raven `BotFreeChatState`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2898-2927`
pub fn BotFreeChatState(bot: &mut BotLib, handle: c_int) {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe { botimport_print(bot, PRT_FATAL, "chat state handle out of range\n") };
        return;
    }
    if bot.botchatstates[handle as usize].is_none() {
        unsafe { botimport_print(bot, PRT_FATAL, "invalid chat state\n") };
        return;
    }
    if LibVarGetValue(bot, "bot_reloadcharacters") != 0.0 {
        BotFreeChatFile(bot, handle);
    }
    let mut m = bot_consolemessage_t {
        handle: 0,
        r#type: 0,
        time: 0.0,
        message: [0; 256],
        prev: core::ptr::null_mut(),
        next: core::ptr::null_mut(),
    };
    let mut h = BotNextConsoleMessage(bot, handle, &mut m);
    while h != 0 {
        BotRemoveConsoleMessage(bot, handle, h);
        h = BotNextConsoleMessage(bot, handle, &mut m);
    }
    bot.botchatstates[handle as usize] = None;
}

/// Raven `BotCheckInitialChatIntegrety`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1567-1586`
pub fn BotCheckInitialChatIntegrety(common: &mut Common, bot: &mut BotLib, chat: &BotChat) {
    let mut stringlist: Vec<String> = Vec::new();
    for t in &chat.types {
        for cm in &t.chatmessages {
            BotCheckChatMessageIntegrety(common, bot, &cm.chatmessage, &mut stringlist);
        }
    }
}

/// Raven `BotCheckReplyChatIntegrety`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1593-1612`
pub fn BotCheckReplyChatIntegrety(common: &mut Common, bot: &mut BotLib, replychat: &[BotReplyChatRule]) {
    let mut stringlist: Vec<String> = Vec::new();
    for rp in replychat {
        for cm in &rp.chatmessages {
            BotCheckChatMessageIntegrety(common, bot, &cm.chatmessage, &mut stringlist);
        }
    }
}

/// Raven `BotExpandChatMessage` — one expansion pass over `message` into
/// `outmessage` (both raw 256-byte buffers managed by `BotConstructChatMessage`,
/// so the match-variable / random substitution and the `MAX_MESSAGE_SIZE`
/// truncation stay byte-for-byte Raven). Only the `\x01r` random lookup changed:
/// `RandomString` now returns an owned `String`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2241-2355`
#[allow(clippy::too_many_arguments)]
pub fn BotExpandChatMessage(
    common: &mut Common,
    bot: &mut BotLib,
    outmessage: *mut c_char,
    message: *mut c_char,
    mcontext: c_ulong,
    r#match: *mut bot_match_t,
    vcontext: c_ulong,
    reply: c_int,
) -> c_int {
    unsafe {
        let mut expansion = 0;
        let mut msgptr = message;
        let outputbuf = outmessage;
        let mut len: isize = 0;
        let mut temp = [0 as c_char; MAX_MESSAGE_SIZE];
        while *msgptr != 0 {
            if *msgptr == ESCAPE_CHAR as c_char {
                msgptr = msgptr.offset(1);
                match *msgptr as u8 as char {
                    'v' => {
                        msgptr = msgptr.offset(1);
                        let mut num: isize = 0;
                        while *msgptr != 0 && *msgptr != ESCAPE_CHAR as c_char {
                            num = num * 10 + (*msgptr as isize) - ('0' as isize);
                            msgptr = msgptr.offset(1);
                        }
                        if *msgptr != 0 {
                            msgptr = msgptr.offset(1);
                        }
                        if num as usize > MAX_MATCHVARIABLES {
                            botimport_print(bot, PRT_ERROR, "BotConstructChat: variable out of range\n");
                            return 0;
                        }
                        let var = &(*r#match).variables[num as usize];
                        if var.offset >= 0 {
                            let ptr = (*r#match).string.as_ptr().offset(var.offset as isize);
                            for i in 0..var.length as isize {
                                temp[i as usize] = *ptr.offset(i);
                            }
                            temp[var.length as usize] = 0;
                            if reply != 0 {
                                BotReplaceReplySynonyms(bot, temp.as_mut_ptr(), vcontext);
                            } else {
                                BotReplaceSynonyms(bot, temp.as_mut_ptr(), vcontext);
                            }
                            let tlen = libc::strlen(temp.as_ptr()) as isize;
                            if len + tlen >= MAX_MESSAGE_SIZE as isize {
                                botimport_print(bot, PRT_ERROR, "BotConstructChat: message too long\n");
                                return 0;
                            }
                            libc::strcpy(outputbuf.offset(len), temp.as_ptr());
                            len += tlen;
                        }
                    }
                    'r' => {
                        msgptr = msgptr.offset(1);
                        let mut i = 0isize;
                        while *msgptr != 0 && *msgptr != ESCAPE_CHAR as c_char {
                            temp[i as usize] = *msgptr;
                            msgptr = msgptr.offset(1);
                            i += 1;
                        }
                        temp[i as usize] = 0;
                        if *msgptr != 0 {
                            msgptr = msgptr.offset(1);
                        }
                        let name = CStr::from_ptr(temp.as_ptr()).to_string_lossy().into_owned();
                        let s = match RandomString(common, bot, &name) {
                            Some(s) => s,
                            None => {
                                botimport_print(bot, PRT_ERROR, "BotConstructChat: unknown random string\n");
                                return 0;
                            }
                        };
                        let plen = s.len() as isize;
                        if len + plen >= MAX_MESSAGE_SIZE as isize {
                            botimport_print(bot, PRT_ERROR, "BotConstructChat: message too long\n");
                            return 0;
                        }
                        let s_c = CString::new(s.as_str()).unwrap_or_default();
                        libc::strcpy(outputbuf.offset(len), s_c.as_ptr());
                        len += plen;
                        expansion = 1;
                    }
                    _ => {
                        botimport_print(bot, PRT_FATAL, "BotConstructChat: invalid escape char\n");
                    }
                }
            } else {
                *outputbuf.offset(len) = *msgptr;
                len += 1;
                msgptr = msgptr.offset(1);
                if len as usize >= MAX_MESSAGE_SIZE {
                    botimport_print(bot, PRT_ERROR, "BotConstructChat: message too long\n");
                    break;
                }
            }
        }
        *outputbuf.offset(len) = 0;
        BotReplaceWeightedSynonyms(common, bot, outputbuf, mcontext);
        expansion
    }
}

/// Raven `BotShutdownChatAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2968-3000`
pub fn BotShutdownChatAI(bot: &mut BotLib) {
    for i in 0..MAX_CLIENTS {
        if bot.botchatstates[i].is_some() {
            BotFreeChatState(bot, i as c_int);
        }
    }
    for i in 0..MAX_CLIENTS {
        if let Some(ic) = bot.ichatdata[i].take() {
            if let Some(h) = ic.chat {
                bot.botchats[h.0] = None;
            }
        }
    }
    if !bot.consolemessageheap.is_null() {
        FreeMemory(bot, bot.consolemessageheap as *mut ());
    }
    bot.consolemessageheap = core::ptr::null_mut();
    bot.matchtemplates.clear();
    bot.randomstrings.clear();
    bot.synonyms.clear();
    bot.replychats.clear();
}

/// Raven `InitConsoleMessageHeap` — allocate the raw `bot_consolemessage_t`
/// pool and thread its freelist. Retained unchanged (seam-visible pool).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:223-243`
pub fn InitConsoleMessageHeap(bot: &mut BotLib) {
    unsafe {
        if !bot.consolemessageheap.is_null() {
            FreeMemory(bot, bot.consolemessageheap as *mut ());
        }
        let max_messages = LibVarValue(bot, "max_messages", "1024") as isize;
        bot.consolemessageheap = GetClearedHunkMemory(
            bot,
            (max_messages as usize * core::mem::size_of::<bot_consolemessage_t>()) as c_ulong,
        ) as *mut bot_consolemessage_t;
        (*bot.consolemessageheap.offset(0)).prev = core::ptr::null_mut();
        (*bot.consolemessageheap.offset(0)).next = bot.consolemessageheap.offset(1);
        for i in 1..max_messages - 1 {
            (*bot.consolemessageheap.offset(i)).prev = bot.consolemessageheap.offset(i - 1);
            (*bot.consolemessageheap.offset(i)).next = bot.consolemessageheap.offset(i + 1);
        }
        (*bot.consolemessageheap.offset(max_messages - 1)).prev =
            bot.consolemessageheap.offset(max_messages - 2);
        (*bot.consolemessageheap.offset(max_messages - 1)).next = core::ptr::null_mut();
        bot.freeconsolemessages = bot.consolemessageheap;
    }
}

/// Raven `BotConstructChatMessage` — repeatedly expand `message` (up to 10
/// passes) into the chat state's owned `chatmessage`. The expansion runs over
/// raw 256-byte scratch buffers (`BotExpandChatMessage`) to preserve Raven's
/// truncation; the final result is written into the `String` field.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2362-2382`
#[allow(clippy::too_many_arguments)]
pub fn BotConstructChatMessage(
    common: &mut Common,
    bot: &mut BotLib,
    cs_idx: usize,
    message: &str,
    mcontext: c_ulong,
    r#match: *mut bot_match_t,
    vcontext: c_ulong,
    reply: c_int,
) {
    unsafe {
        let mut srcmessage = [0 as c_char; MAX_MESSAGE_SIZE];
        let mut outbuf = [0 as c_char; MAX_MESSAGE_SIZE];
        let message_c = CString::new(message).unwrap_or_default();
        libc::strcpy(srcmessage.as_mut_ptr(), message_c.as_ptr());
        let mut i = 0;
        while i < 10 {
            if BotExpandChatMessage(
                common,
                bot,
                outbuf.as_mut_ptr(),
                srcmessage.as_mut_ptr(),
                mcontext,
                r#match,
                vcontext,
                reply,
            ) == 0
            {
                break;
            }
            libc::strcpy(srcmessage.as_mut_ptr(), outbuf.as_ptr());
            i += 1;
        }
        if i >= 10 {
            botimport_print(bot, PRT_WARNING, "too many expansions in chat message\n");
            botimport_print(bot, PRT_WARNING, "chatmessage\n");
        }
        let out = CStr::from_ptr(outbuf.as_ptr()).to_string_lossy().into_owned();
        bot.botchatstates[cs_idx].as_mut().unwrap().chatmessage = out;
    }
}

/// Raven `BotInitialChat` — frozen-signature seam export; the eight var slots
/// are decoded at the boundary and concatenated into the (frozen) `bot_match_t`
/// string, then the chosen chat message is expanded.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2475-2549`
#[allow(clippy::too_many_arguments)]
pub fn BotInitialChat(
    common: &mut Common,
    bot: &mut BotLib,
    chatstate: c_int,
    r#type: *mut c_char,
    mcontext: c_int,
    var0: *mut c_char,
    var1: *mut c_char,
    var2: *mut c_char,
    var3: *mut c_char,
    var4: *mut c_char,
    var5: *mut c_char,
    var6: *mut c_char,
    var7: *mut c_char,
) {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return,
    };
    let chat_handle = match bot.botchatstates[idx].as_ref().unwrap().chat {
        Some(h) => h,
        None => return,
    };
    unsafe {
        let type_str = CStr::from_ptr(r#type).to_string_lossy().into_owned();
        let message = match BotChooseInitialChatMessage(common, bot, chat_handle, &type_str) {
            Some(m) => m,
            None => return,
        };
        let mut r#match = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| bot_matchvariable_t { offset: 0, length: 0 }),
        };
        Com_Memset(
            &mut r#match as *mut bot_match_t as *mut (),
            0,
            core::mem::size_of::<bot_match_t>(),
        );
        let mut index: isize = 0;
        for (i, var) in [var0, var1, var2, var3, var4, var5, var6, var7]
            .into_iter()
            .enumerate()
        {
            if !var.is_null() {
                libc::strcat(r#match.string.as_mut_ptr(), var);
                r#match.variables[i].offset = index as c_char;
                let vlen = libc::strlen(var) as i32;
                r#match.variables[i].length = vlen;
                index += vlen as isize;
            }
        }
        BotConstructChatMessage(common, bot, idx, &message, mcontext as c_ulong, &mut r#match, 0, 0);
    }
}

/// Raven `BotReplyChat` — frozen-signature seam export; find the best-priority
/// matching reply chat and expand one of its messages. `AAS_Time` is captured
/// once; the reply-chat / key / message iteration order (Raven's prepended,
/// reverse-file order, preserved by the loader) drives the RNG stream and the
/// tie-break, so it is faithful.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2597-2747`
#[allow(clippy::too_many_arguments)]
pub fn BotReplyChat(
    common: &mut Common,
    bot: &mut BotLib,
    chatstate: c_int,
    message: *mut c_char,
    mcontext: c_int,
    vcontext: c_int,
    var0: *mut c_char,
    var1: *mut c_char,
    var2: *mut c_char,
    var3: *mut c_char,
    var4: *mut c_char,
    var5: *mut c_char,
    var6: *mut c_char,
    var7: *mut c_char,
) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return 0,
    };
    unsafe {
        let cs_name = bot.botchatstates[idx].as_ref().unwrap().name.clone();
        let cs_gender = bot.botchatstates[idx].as_ref().unwrap().gender;
        let cs_name_c = CString::new(cs_name.as_str()).unwrap_or_default();
        let time = AAS_Time(bot);
        let mut r#match = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| bot_matchvariable_t { offset: 0, length: 0 }),
        };
        Com_Memset(
            &mut r#match as *mut bot_match_t as *mut (),
            0,
            core::mem::size_of::<bot_match_t>(),
        );
        libc::strcpy(r#match.string.as_mut_ptr(), message);
        let mut bestpriority = -1.0f32;
        let mut best: Option<(usize, usize)> = None;
        let mut bestmatch = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| bot_matchvariable_t { offset: 0, length: 0 }),
        };
        for (ri, rchat) in bot.replychats.iter().enumerate() {
            let mut found = false;
            for key in &rchat.keys {
                let flags = key.flags;
                let res = if flags & RCKFL_NAME != 0 {
                    StringContains(message, cs_name_c.as_ptr() as *mut c_char, 0) != -1
                } else if flags & RCKFL_BOTNAMES != 0 {
                    let ks = CString::new(key.string.as_str()).unwrap_or_default();
                    StringContains(
                        ks.as_ptr() as *mut c_char,
                        cs_name_c.as_ptr() as *mut c_char,
                        0,
                    ) != -1
                } else if flags & RCKFL_GENDERFEMALE != 0 {
                    cs_gender == CHAT_GENDERFEMALE
                } else if flags & RCKFL_GENDERMALE != 0 {
                    cs_gender == CHAT_GENDERMALE
                } else if flags & RCKFL_GENDERLESS != 0 {
                    cs_gender == CHAT_GENDERLESS
                } else if flags & RCKFL_VARIABLES != 0 {
                    StringsMatch(&key.match_, &mut r#match) != 0
                } else if flags & RCKFL_STRING != 0 {
                    let ks = CString::new(key.string.as_str()).unwrap_or_default();
                    !StringContainsWord(message, ks.as_ptr() as *mut c_char, 0).is_null()
                } else {
                    false
                };
                if flags & RCKFL_AND != 0 {
                    if !res {
                        found = false;
                        break;
                    }
                } else if flags & RCKFL_NOT != 0 {
                    if res {
                        found = false;
                        break;
                    }
                } else if res {
                    found = true;
                }
            }
            if found && rchat.priority > bestpriority {
                let numchatmessages =
                    rchat.chatmessages.iter().filter(|m| m.time <= time).count() as c_int;
                let mut num = common.qrand.irand(0, numchatmessages.max(1) - 1);
                let mut chosen: Option<usize> = None;
                for (mi, _m) in rchat.chatmessages.iter().enumerate() {
                    num -= 1;
                    if num < 0 {
                        chosen = Some(mi);
                        break;
                    }
                }
                if let Some(mi) = chosen {
                    Com_Memcpy(
                        &mut bestmatch as *mut bot_match_t as *mut (),
                        &r#match as *const bot_match_t as *const (),
                        core::mem::size_of::<bot_match_t>(),
                    );
                    best = Some((ri, mi));
                    bestpriority = rchat.priority;
                }
            }
        }
        if let Some((ri, mi)) = best {
            let mut index = libc::strlen(bestmatch.string.as_ptr()) as isize;
            for (i, var) in [var0, var1, var2, var3, var4, var5, var6, var7]
                .into_iter()
                .enumerate()
            {
                if !var.is_null() {
                    libc::strcat(bestmatch.string.as_mut_ptr(), var);
                    bestmatch.variables[i].offset = index as c_char;
                    let vlen = libc::strlen(var) as i32;
                    bestmatch.variables[i].length = vlen;
                    index += vlen as isize;
                }
            }
            if LibVarGetValue(bot, "bot_testrchat") != 0.0 {
                let msgs: Vec<String> = bot.replychats[ri]
                    .chatmessages
                    .iter()
                    .map(|m| m.chatmessage.clone())
                    .collect();
                for msg in &msgs {
                    BotConstructChatMessage(
                        common,
                        bot,
                        idx,
                        msg,
                        mcontext as c_ulong,
                        &mut bestmatch,
                        vcontext as c_ulong,
                        1,
                    );
                    BotRemoveTildes(&mut bot.botchatstates[idx].as_mut().unwrap().chatmessage);
                    botimport_print(bot, PRT_MESSAGE, "chatmessage\n");
                }
            } else {
                bot.replychats[ri].chatmessages[mi].time = time + CHATMESSAGE_RECENTTIME as f32;
                let msg = bot.replychats[ri].chatmessages[mi].chatmessage.clone();
                BotConstructChatMessage(
                    common,
                    bot,
                    idx,
                    &msg,
                    mcontext as c_ulong,
                    &mut bestmatch,
                    vcontext as c_ulong,
                    1,
                );
            }
            return 1;
        }
    }
    0
}

/// Raven `BotLoadSynonyms` — single-pass build of the owned `Vec<BotSynonymList>`
/// (Raven's two-pass malloc-sizing pass is unnecessary for owned collections;
/// the parse acceptance, error text, ordering and RNG-relevant weights are
/// preserved).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:578-736`
pub fn BotLoadSynonyms(bot: &mut BotLib, filename: &str) -> Vec<BotSynonymList> {
    let mut synlist: Vec<BotSynonymList> = Vec::new();
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, filename) {
        Some(s) => s,
        None => {
            unsafe { botimport_print(bot, PRT_ERROR, "counldn't load file\n") };
            return Vec::new();
        }
    };
    let mut context: c_ulong = 0;
    let mut contextlevel: isize = 0;
    let mut contextstack = [0 as c_ulong; 32];
    let mut lastsyn: Option<BotSynonymList> = None;
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.type_ == TT_NUMBER {
            context |= token.intvalue as c_ulong;
            contextstack[contextlevel as usize] = token.intvalue as c_ulong;
            contextlevel += 1;
            if contextlevel >= 32 {
                SourceError(bot, &source, "more than 32 context levels");
                FreeSource(source);
                return Vec::new();
            }
            if PC_ExpectTokenString(bot, &mut source, "{") == 0 {
                FreeSource(source);
                return Vec::new();
            }
        } else if token.type_ == TT_PUNCTUATION {
            if token.string == "}" {
                contextlevel -= 1;
                if contextlevel < 0 {
                    SourceError(bot, &source, "too many }");
                    FreeSource(source);
                    return Vec::new();
                }
                context &= !contextstack[contextlevel as usize];
            } else if token.string == "[" {
                let mut syn = BotSynonymList {
                    context,
                    totalweight: 0.0,
                    synonyms: Vec::new(),
                };
                let mut numsynonyms = 0;
                loop {
                    if PC_ExpectTokenString(bot, &mut source, "(") == 0
                        || PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0
                    {
                        FreeSource(source);
                        return Vec::new();
                    }
                    StripDoubleQuotes(&mut token.string);
                    if token.string.is_empty() {
                        SourceError(bot, &source, "empty string");
                        FreeSource(source);
                        return Vec::new();
                    }
                    let syn_string = token.string.clone();
                    numsynonyms += 1;
                    if PC_ExpectTokenString(bot, &mut source, ",") == 0
                        || PC_ExpectTokenType(bot, &mut source, TT_NUMBER, 0, &mut token) == 0
                        || PC_ExpectTokenString(bot, &mut source, ")") == 0
                    {
                        FreeSource(source);
                        return Vec::new();
                    }
                    let weight = token.floatvalue as f32;
                    syn.totalweight += weight;
                    syn.synonyms.push(BotSynonym {
                        string: syn_string,
                        weight,
                    });
                    if PC_CheckTokenString(bot, &mut source, "]") != 0 {
                        break;
                    }
                    if PC_ExpectTokenString(bot, &mut source, ",") == 0 {
                        FreeSource(source);
                        return Vec::new();
                    }
                }
                if numsynonyms < 2 {
                    SourceError(bot, &source, "synonym must have at least two entries\n");
                    FreeSource(source);
                    return Vec::new();
                }
                if let Some(prev) = lastsyn.take() {
                    synlist.push(prev);
                }
                lastsyn = Some(syn);
            } else {
                SourceError(bot, &source, &format!("unexpected {}", token.string));
                FreeSource(source);
                return Vec::new();
            }
        }
    }
    if let Some(prev) = lastsyn.take() {
        synlist.push(prev);
    }
    // Raven frees `source` before checking `contextlevel`, so its "missing }"
    // path reads an already-freed source (UB). `FreeSource` consumes the value,
    // so we check first, then free — same message, same source state, freed
    // exactly once either way.
    if contextlevel > 0 {
        SourceError(bot, &source, "missing }");
        FreeSource(source);
        return Vec::new();
    }
    FreeSource(source);
    unsafe { botimport_print(bot, PRT_MESSAGE, "loaded synonyms\n") };
    synlist
}

/// Raven `BotLoadChatMessage` — parse one chat message (string literals,
/// `\x01v<int>\x01` for numbers, `\x01r<name>\x01` for names) into an owned
/// `String`, preserving the `MAX_MESSAGE_SIZE` length checks.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:845-897`
pub fn BotLoadChatMessage(bot: &mut BotLib, source: &mut Source) -> Option<String> {
    let mut out = String::new();
    let mut token = Token::default();
    loop {
        if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
            return None;
        }
        if token.type_ == TT_STRING {
            StripDoubleQuotes(&mut token.string);
            if out.len() + token.string.len() + 1 > MAX_MESSAGE_SIZE {
                SourceError(bot, source, "chat message too long\n");
                return None;
            }
            out.push_str(&token.string);
        } else if token.type_ == TT_NUMBER && (token.subtype & TT_INTEGER) != 0 {
            if out.len() + 7 > MAX_MESSAGE_SIZE {
                SourceError(bot, source, "chat message too long\n");
                return None;
            }
            out.push(ESCAPE_CHAR as char);
            out.push('v');
            out.push_str(&token.intvalue.to_string());
            out.push(ESCAPE_CHAR as char);
        } else if token.type_ == TT_NAME {
            if out.len() + 7 > MAX_MESSAGE_SIZE {
                SourceError(bot, source, "chat message too long\n");
                return None;
            }
            out.push(ESCAPE_CHAR as char);
            out.push('r');
            out.push_str(&token.string);
            out.push(ESCAPE_CHAR as char);
        } else {
            SourceError(
                bot,
                source,
                &format!("unknown message component {}\n", token.string),
            );
            return None;
        }
        if PC_CheckTokenString(bot, source, ";") != 0 {
            break;
        }
        if PC_ExpectTokenString(bot, source, ",") == 0 {
            return None;
        }
    }
    Some(out)
}

/// Raven `BotLoadMatchPieces` — parse a match-piece list into an owned
/// `Vec<BotMatchPiece>`. On error returns `None` (the caller frees `source`);
/// the partially-built local `Vec` drops (Raven's `BotFreeMatchPieces`).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1122-1217`
pub fn BotLoadMatchPieces(
    bot: &mut BotLib,
    source: &mut Source,
    endtoken: &str,
) -> Option<Vec<BotMatchPiece>> {
    let mut firstpiece: Vec<BotMatchPiece> = Vec::new();
    let mut lastwasvariable = false;
    let mut token = Token::default();
    while PC_ReadToken(bot, source, &mut token) != 0 {
        if token.type_ == TT_NUMBER && (token.subtype & TT_INTEGER) != 0 {
            if token.intvalue as usize >= MAX_MATCHVARIABLES {
                SourceError(
                    bot,
                    source,
                    &format!("can't have more than {} match variables\n", MAX_MATCHVARIABLES),
                );
                return None;
            }
            if lastwasvariable {
                SourceError(bot, source, "not allowed to have adjacent variables\n");
                return None;
            }
            lastwasvariable = true;
            firstpiece.push(BotMatchPiece {
                type_: MT_VARIABLE,
                strings: Vec::new(),
                variable: token.intvalue as c_int,
            });
        } else if token.type_ == TT_STRING {
            let mut piece = BotMatchPiece {
                type_: MT_STRING,
                strings: Vec::new(),
                variable: 0,
            };
            let mut emptystring = false;
            loop {
                if !piece.strings.is_empty()
                    && PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0
                {
                    return None;
                }
                StripDoubleQuotes(&mut token.string);
                if token.string.is_empty() {
                    emptystring = true;
                }
                piece.strings.push(token.string.clone());
                if PC_CheckTokenString(bot, source, "|") == 0 {
                    break;
                }
            }
            if !emptystring {
                lastwasvariable = false;
            }
            firstpiece.push(piece);
        } else {
            SourceError(bot, source, &format!("invalid token {}\n", token.string));
            return None;
        }
        if PC_CheckTokenString(bot, source, endtoken) != 0 {
            break;
        }
        if PC_ExpectTokenString(bot, source, ",") == 0 {
            return None;
        }
    }
    Some(firstpiece)
}

/// Raven `BotLoadRandomStrings` — single-pass build of `Vec<BotRandomList>`.
/// Raven prepends each random string, so `strings` is built with `insert(0, ..)`
/// to keep `RandomString`'s index → string mapping byte-identical.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:929-1024`
pub fn BotLoadRandomStrings(bot: &mut BotLib, filename: &str) -> Vec<BotRandomList> {
    let mut randomlist: Vec<BotRandomList> = Vec::new();
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, filename) {
        Some(s) => s,
        None => {
            unsafe { botimport_print(bot, PRT_ERROR, "counldn't load file\n") };
            return Vec::new();
        }
    };
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.type_ != TT_NAME {
            SourceError(bot, &source, &format!("unknown random {}", token.string));
            FreeSource(source);
            return Vec::new();
        }
        let mut random = BotRandomList {
            string: token.string.clone(),
            strings: Vec::new(),
        };
        if PC_ExpectTokenString(bot, &mut source, "=") == 0
            || PC_ExpectTokenString(bot, &mut source, "{") == 0
        {
            FreeSource(source);
            return Vec::new();
        }
        while PC_CheckTokenString(bot, &mut source, "}") == 0 {
            let msg = match BotLoadChatMessage(bot, &mut source) {
                Some(m) => m,
                None => {
                    FreeSource(source);
                    return Vec::new();
                }
            };
            random.strings.insert(0, msg);
        }
        randomlist.push(random);
    }
    FreeSource(source);
    unsafe { botimport_print(bot, PRT_MESSAGE, "loaded random strings\n") };
    randomlist
}

/// Raven `BotLoadMatchTemplates` — single-pass build of `Vec<BotMatchTemplate>`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1241-1333`
pub fn BotLoadMatchTemplates(bot: &mut BotLib, matchfile: &str) -> Vec<BotMatchTemplate> {
    let mut matches: Vec<BotMatchTemplate> = Vec::new();
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, matchfile) {
        Some(s) => s,
        None => {
            unsafe { botimport_print(bot, PRT_ERROR, "counldn't load file\n") };
            return Vec::new();
        }
    };
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.type_ != TT_NUMBER || (token.subtype & TT_INTEGER) == 0 {
            SourceError(
                bot,
                &source,
                &format!("expected integer, found {}\n", token.string),
            );
            FreeSource(source);
            return Vec::new();
        }
        let context = token.intvalue as c_ulong;
        if PC_ExpectTokenString(bot, &mut source, "{") == 0 {
            FreeSource(source);
            return Vec::new();
        }
        while PC_ReadToken(bot, &mut source, &mut token) != 0 {
            if token.string == "}" {
                break;
            }
            PC_UnreadLastToken(&mut source);
            let first = match BotLoadMatchPieces(bot, &mut source, "=") {
                Some(v) => v,
                None => {
                    FreeSource(source);
                    return Vec::new();
                }
            };
            let mut matchtemplate = BotMatchTemplate {
                context,
                type_: 0,
                subtype: 0,
                first,
            };
            if PC_ExpectTokenString(bot, &mut source, "(") == 0
                || PC_ExpectTokenType(bot, &mut source, TT_NUMBER, TT_INTEGER, &mut token) == 0
            {
                FreeSource(source);
                return Vec::new();
            }
            matchtemplate.type_ = token.intvalue as i32;
            if PC_ExpectTokenString(bot, &mut source, ",") == 0
                || PC_ExpectTokenType(bot, &mut source, TT_NUMBER, TT_INTEGER, &mut token) == 0
            {
                FreeSource(source);
                return Vec::new();
            }
            matchtemplate.subtype = token.intvalue as i32;
            if PC_ExpectTokenString(bot, &mut source, ")") == 0
                || PC_ExpectTokenString(bot, &mut source, ";") == 0
            {
                FreeSource(source);
                return Vec::new();
            }
            matches.push(matchtemplate);
        }
    }
    FreeSource(source);
    unsafe { botimport_print(bot, PRT_MESSAGE, "loaded match templates\n") };
    matches
}

/// Raven `BotLoadReplyChat` — single-pass build of `Vec<BotReplyChat>`. Raven
/// prepends reply chats, keys and messages, so each is built with `insert(0, ..)`
/// to keep the reply RNG stream and priority tie-break byte-identical.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1811-1964`
pub fn BotLoadReplyChat(common: &mut Common, bot: &mut BotLib, filename: &str) -> Vec<BotReplyChatRule> {
    let mut replychatlist: Vec<BotReplyChatRule> = Vec::new();
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, filename) {
        Some(s) => s,
        None => {
            unsafe { botimport_print(bot, PRT_ERROR, "counldn't load file\n") };
            return Vec::new();
        }
    };
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.string != "[" {
            SourceError(bot, &source, &format!("expected [, found {}", token.string));
            FreeSource(source);
            return Vec::new();
        }
        let mut replychat = BotReplyChatRule {
            keys: Vec::new(),
            priority: 0.0,
            chatmessages: Vec::new(),
        };
        loop {
            let mut key = BotReplyChatKey {
                flags: 0,
                string: String::new(),
                match_: Vec::new(),
            };
            if PC_CheckTokenString(bot, &mut source, "&") != 0 {
                key.flags |= RCKFL_AND;
            } else if PC_CheckTokenString(bot, &mut source, "!") != 0 {
                key.flags |= RCKFL_NOT;
            }
            if PC_CheckTokenString(bot, &mut source, "name") != 0 {
                key.flags |= RCKFL_NAME;
            } else if PC_CheckTokenString(bot, &mut source, "female") != 0 {
                key.flags |= RCKFL_GENDERFEMALE;
            } else if PC_CheckTokenString(bot, &mut source, "male") != 0 {
                key.flags |= RCKFL_GENDERMALE;
            } else if PC_CheckTokenString(bot, &mut source, "it") != 0 {
                key.flags |= RCKFL_GENDERLESS;
            } else if PC_CheckTokenString(bot, &mut source, "(") != 0 {
                key.flags |= RCKFL_VARIABLES;
                key.match_ = match BotLoadMatchPieces(bot, &mut source, ")") {
                    Some(v) => v,
                    None => {
                        FreeSource(source);
                        return Vec::new();
                    }
                };
            } else if PC_CheckTokenString(bot, &mut source, "<") != 0 {
                key.flags |= RCKFL_BOTNAMES;
                let mut namebuffer = String::new();
                loop {
                    if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0 {
                        FreeSource(source);
                        return Vec::new();
                    }
                    StripDoubleQuotes(&mut token.string);
                    if !namebuffer.is_empty() {
                        namebuffer.push('\\');
                    }
                    namebuffer.push_str(&token.string);
                    if PC_CheckTokenString(bot, &mut source, ",") == 0 {
                        break;
                    }
                }
                if PC_ExpectTokenString(bot, &mut source, ">") == 0 {
                    FreeSource(source);
                    return Vec::new();
                }
                key.string = namebuffer;
            } else {
                key.flags |= RCKFL_STRING;
                if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0 {
                    FreeSource(source);
                    return Vec::new();
                }
                StripDoubleQuotes(&mut token.string);
                key.string = token.string.clone();
            }
            replychat.keys.insert(0, key);
            PC_CheckTokenString(bot, &mut source, ",");
            if PC_CheckTokenString(bot, &mut source, "]") != 0 {
                break;
            }
        }
        BotCheckValidReplyChatKeySet(bot, &mut source, &replychat.keys);
        if PC_ExpectTokenString(bot, &mut source, "=") == 0
            || PC_ExpectTokenType(bot, &mut source, TT_NUMBER, 0, &mut token) == 0
        {
            FreeSource(source);
            return Vec::new();
        }
        replychat.priority = token.floatvalue as f32;
        if PC_ExpectTokenString(bot, &mut source, "{") == 0 {
            FreeSource(source);
            return Vec::new();
        }
        while PC_CheckTokenString(bot, &mut source, "}") == 0 {
            let msg = match BotLoadChatMessage(bot, &mut source) {
                Some(m) => m,
                None => {
                    FreeSource(source);
                    return Vec::new();
                }
            };
            replychat.chatmessages.insert(
                0,
                BotChatMessage {
                    chatmessage: msg,
                    time: -2.0 * CHATMESSAGE_RECENTTIME as f32,
                },
            );
        }
        replychatlist.insert(0, replychat);
    }
    FreeSource(source);
    unsafe { botimport_print(bot, PRT_MESSAGE, "loaded reply chat\n") };
    if bot.bot_developer != 0 {
        BotCheckReplyChatIntegrety(common, bot, &replychatlist);
    }
    if replychatlist.is_empty() {
        unsafe { botimport_print(bot, PRT_MESSAGE, "no rchats\n") };
    }
    replychatlist
}

/// Raven `BotLoadInitialChat` — single-pass build of the owned `BotChat` for the
/// named chat in the file. Chat types and messages are prepended by Raven, so
/// they are built with `insert(0, ..)` (message-pick RNG order preserved).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1996-2161`
pub fn BotLoadInitialChat(
    common: &mut Common,
    bot: &mut BotLib,
    chatfile: &str,
    chatname: &str,
) -> Option<BotChat> {
    let mut chat = BotChat { types: Vec::new() };
    let mut foundchat = false;
    PC_SetBaseFolder(bot, BOTFILESBASEFOLDER);
    let mut source = match LoadSourceFile(bot, chatfile) {
        Some(s) => s,
        None => {
            unsafe { botimport_print(bot, PRT_ERROR, "counldn't load file\n") };
            return None;
        }
    };
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {
        if token.string == "chat" {
            if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0 {
                FreeSource(source);
                return None;
            }
            StripDoubleQuotes(&mut token.string);
            let this_chatname = token.string.clone();
            if PC_ExpectTokenString(bot, &mut source, "{") == 0 {
                FreeSource(source);
                return None;
            }
            if this_chatname.eq_ignore_ascii_case(chatname) {
                foundchat = true;
                loop {
                    if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                        FreeSource(source);
                        return None;
                    }
                    if token.string == "}" {
                        break;
                    }
                    if token.string != "type" {
                        SourceError(bot, &source, &format!("expected type found {}\n", token.string));
                        FreeSource(source);
                        return None;
                    }
                    if PC_ExpectTokenType(bot, &mut source, TT_STRING, 0, &mut token) == 0
                        || PC_ExpectTokenString(bot, &mut source, "{") == 0
                    {
                        FreeSource(source);
                        return None;
                    }
                    StripDoubleQuotes(&mut token.string);
                    // Q_strncpyz(name, token, MAX_CHATTYPE_NAME): truncate to
                    // MAX_CHATTYPE_NAME - 1 bytes; chat type names are ASCII, so
                    // char-take matches the byte truncation.
                    let mut chattype = BotChatType {
                        name: token.string.chars().take(MAX_CHATTYPE_NAME - 1).collect(),
                        chatmessages: Vec::new(),
                    };
                    while PC_CheckTokenString(bot, &mut source, "}") == 0 {
                        let msg = match BotLoadChatMessage(bot, &mut source) {
                            Some(m) => m,
                            None => {
                                FreeSource(source);
                                return None;
                            }
                        };
                        chattype.chatmessages.insert(
                            0,
                            BotChatMessage {
                                chatmessage: msg,
                                time: -2.0 * CHATMESSAGE_RECENTTIME as f32,
                            },
                        );
                    }
                    chat.types.insert(0, chattype);
                }
            } else {
                let mut indent = 1;
                while indent != 0 {
                    if PC_ExpectAnyToken(bot, &mut source, &mut token) == 0 {
                        FreeSource(source);
                        return None;
                    }
                    if token.string == "{" {
                        indent += 1;
                    } else if token.string == "}" {
                        indent -= 1;
                    }
                }
            }
        } else {
            SourceError(bot, &source, &format!("unknown definition {}\n", token.string));
            FreeSource(source);
            return None;
        }
    }
    FreeSource(source);
    if !foundchat {
        unsafe { botimport_print(bot, PRT_ERROR, "couldn't find chat\n") };
        return None;
    }
    unsafe { botimport_print(bot, PRT_MESSAGE, "loaded chat\n") };
    if bot.bot_developer != 0 {
        BotCheckInitialChatIntegrety(common, bot, &chat);
    }
    Some(chat)
}

/// Raven `BotLoadChatFile` — frozen-signature seam export; decode the file/chat
/// names at the boundary, reuse a cached chat (shared arena handle) or load a
/// new one into the arena.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2183-2234`
pub fn BotLoadChatFile(
    common: &mut Common,
    bot: &mut BotLib,
    chatstate: c_int,
    chatfile: *mut c_char,
    chatname: *mut c_char,
) -> c_int {
    let idx = match BotChatStateFromHandle(bot, chatstate) {
        Some(i) => i,
        None => return BLERR_CANNOTLOADICHAT,
    };
    BotFreeChatFile(bot, chatstate);
    let chatfile_str = unsafe { CStr::from_ptr(chatfile).to_string_lossy().into_owned() };
    let chatname_str = unsafe { CStr::from_ptr(chatname).to_string_lossy().into_owned() };
    let reload = LibVarGetValue(bot, "bot_reloadcharacters");
    let mut avail: isize = 0;
    if reload == 0.0 {
        avail = -1;
        for n in 0..MAX_CLIENTS {
            match &bot.ichatdata[n] {
                None => {
                    if avail == -1 {
                        avail = n as isize;
                    }
                    continue;
                }
                Some(ic) => {
                    if ic.filename != chatfile_str {
                        continue;
                    }
                    if ic.chatname != chatname_str {
                        continue;
                    }
                    bot.botchatstates[idx].as_mut().unwrap().chat = ic.chat;
                    return BLERR_NOERROR;
                }
            }
        }
        if avail == -1 {
            unsafe { botimport_print(bot, PRT_FATAL, "ichatdata table full\n") };
            return BLERR_CANNOTLOADICHAT;
        }
    }
    let chat = match BotLoadInitialChat(common, bot, &chatfile_str, &chatname_str) {
        Some(c) => c,
        None => {
            unsafe { botimport_print(bot, PRT_FATAL, "couldn't load chat\n") };
            return BLERR_CANNOTLOADICHAT;
        }
    };
    let handle = bot.alloc_botchat(chat);
    bot.botchatstates[idx].as_mut().unwrap().chat = Some(handle);
    if reload == 0.0 {
        // Q_strncpyz to MAX_QPATH: names are ASCII paths; char-take matches.
        bot.ichatdata[avail as usize] = Some(BotIChatData {
            chat: Some(handle),
            chatname: chatname_str.chars().take(MAX_QPATH - 1).collect(),
            filename: chatfile_str.chars().take(MAX_QPATH - 1).collect(),
        });
    }
    BLERR_NOERROR
}

/// Raven `BotSetupChatAI` — load the synonym / random / match / reply-chat data
/// and init the console-message heap. The filename cvars are read as `String`s
/// and passed straight to the loaders (the ③-era `CString` bridges dissolve).
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2934-2961`
pub fn BotSetupChatAI(common: &mut Common, bot: &mut BotLib) -> c_int {
    let file = LibVarString(bot, "synfile", "syn.c");
    bot.synonyms = BotLoadSynonyms(bot, &file);
    let file = LibVarString(bot, "rndfile", "rnd.c");
    bot.randomstrings = BotLoadRandomStrings(bot, &file);
    let file = LibVarString(bot, "matchfile", "match.c");
    bot.matchtemplates = BotLoadMatchTemplates(bot, &file);
    if LibVarValue(bot, "nochat", "0") == 0.0 {
        let file = LibVarString(bot, "rchatfile", "rchat.c");
        bot.replychats = BotLoadReplyChat(common, bot, &file);
    }
    InitConsoleMessageHeap(bot);
    BLERR_NOERROR
}
