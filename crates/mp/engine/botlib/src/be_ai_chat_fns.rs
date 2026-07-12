//! MP botlib `be_ai_chat.cpp` chat-AI functions.
//!
//! Source: `oracle/codemp/botlib/be_ai_chat.cpp`

#![allow(non_camel_case_types, non_snake_case, clippy::missing_safety_doc)]

use core::ffi::{c_char, c_int, c_ulong, c_void};

use mp_engine_qcommon::common::Common;
use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;
use mp_qshared::common::mp::botlib::bot_match_s::{bot_match_t, MAX_MATCHVARIABLES};
use mp_qshared::common::mp::botlib::botlib_error::{BLERR_CANNOTLOADICHAT, BLERR_NOERROR};
use mp_qshared::common::mp::botlib::botlib_misc::BOTFILESBASEFOLDER;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_FATAL, PRT_MESSAGE, PRT_WARNING};
use mp_qshared::shared::limits::MAX_CLIENTS;

use crate::be_ai_chat::bot_chat_s::bot_chat_t;
use crate::be_ai_chat::bot_chatmessage_s::bot_chatmessage_t;
use crate::be_ai_chat::bot_chatstate_s::{bot_chatstate_t, MAX_MESSAGE_SIZE};
use crate::be_ai_chat::bot_chattype_s::bot_chattype_t;
use crate::be_ai_chat::bot_matchpiece_s::bot_matchpiece_t;
use crate::be_ai_chat::bot_matchstring_s::bot_matchstring_t;
use crate::be_ai_chat::bot_matchtemplate_s::bot_matchtemplate_t;
use crate::be_ai_chat::bot_randomlist_s::bot_randomlist_t;
use crate::be_ai_chat::bot_randomstring_s::bot_randomstring_t;
use crate::be_ai_chat::bot_replychat_s::bot_replychat_t;
use crate::be_ai_chat::bot_replychatkey_s::bot_replychatkey_t;
use crate::be_ai_chat::bot_stringlist_s::bot_stringlist_t;
use crate::be_ai_chat::bot_synonym_s::bot_synonym_t;
use crate::be_ai_chat::bot_synonymlist_s::bot_synonymlist_t;
use crate::be_ai_chat::chat_consts::{
    CHAT_ALL, CHAT_GENDERFEMALE, CHAT_GENDERLESS, CHAT_GENDERMALE, CHAT_TEAM, CHAT_TELL,
    MAX_CHATTYPE_NAME,
};
use crate::be_ai_chat::chat_cpp_consts::{
    CHATMESSAGE_RECENTTIME, ESCAPE_CHAR, MT_STRING, MT_VARIABLE, RCKFL_AND, RCKFL_BOTNAMES,
    RCKFL_GENDERFEMALE, RCKFL_GENDERLESS, RCKFL_GENDERMALE, RCKFL_NAME, RCKFL_NOT, RCKFL_STRING,
    RCKFL_VARIABLES,
};
use crate::l_precomp::source_s::source_t;
use crate::l_script::consts::{TT_INTEGER, TT_NAME, TT_NUMBER, TT_PUNCTUATION, TT_STRING};
use crate::l_script::token_s::token_t;
use crate::BotLib;

use crate::be_aas_main::AAS_Time;
use crate::be_ea_fns::EA_Command;
use crate::l_libvar_fns::{LibVarGetValue, LibVarString, LibVarValue};
use crate::l_log_fns::{Log_FilePointer, Log_Write};
use crate::l_memory_fns::{FreeMemory, GetClearedHunkMemory, GetClearedMemory};
use crate::l_precomp_fns::{
    FreeSource, LoadSourceFile, PC_CheckTokenString, PC_ExpectAnyToken, PC_ExpectTokenString,
    PC_ExpectTokenType, PC_ReadToken, PC_SetBaseFolder, PC_UnreadLastToken, SourceError,
    SourceWarning,
};
use crate::l_script_fns::StripDoubleQuotes;
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};

/// Raven `BotChatStateFromHandle`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:202-215`
pub fn BotChatStateFromHandle(bot: &mut BotLib, handle: c_int) -> *mut bot_chatstate_t {
    if handle <= 0 || handle > MAX_CLIENTS as c_int {
        unsafe {
            crate::be_interface::botimport_print(
                bot,
                PRT_FATAL,
                "chat state handle %d out of range\n",
            )
        };
        return core::ptr::null_mut();
    }
    let cs = bot.botchatstates[handle as usize];
    if cs.is_null() {
        unsafe { crate::be_interface::botimport_print(bot, PRT_FATAL, "invalid chat state %d\n") };
        return core::ptr::null_mut();
    }
    cs
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

/// Raven `BotRemoveTildes`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:404-416`
pub fn BotRemoveTildes(message: *mut c_char) {
    unsafe {
        let mut i = 0isize;
        while *message.offset(i) != 0 {
            if *message.offset(i) == b'~' as c_char {
                let src = message.offset(i + 1);
                let len = libc::strlen(src);
                libc::memmove(
                    message.offset(i) as *mut c_void,
                    src as *const c_void,
                    len + 1,
                );
            } else {
                i += 1;
            }
        }
    }
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

/// Raven `RandomString`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1031-1053`
pub fn RandomString(common: &mut Common, bot: &mut BotLib, name: *mut c_char) -> *mut c_char {
    // PORT-NOTE(QRand): `random()` routes through the engine LCG on `common`
    // (ruling 21); the exact field name lands with the `QRand` type.
    let _ = &common;
    unsafe {
        let mut random = bot.randomstrings;
        while !random.is_null() {
            if libc::strcmp((*random).string, name) == 0 {
                let mut i = (common.qrand.irand(0, (*random).numstrings - 1)) as c_int;
                let mut rs = (*random).firstrandomstring;
                while !rs.is_null() {
                    i -= 1;
                    if i < 0 {
                        break;
                    }
                    rs = (*rs).next;
                }
                if !rs.is_null() {
                    return (*rs).string;
                }
            }
            random = (*random).next;
        }
    }
    core::ptr::null_mut()
}

/// Raven `BotMatchVariable`.
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
            crate::be_interface::botimport_print(
                bot,
                PRT_FATAL,
                "BotMatchVariable: variable out of range\n",
            );
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

/// Raven `BotFindStringInList`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1480-1489`
pub fn BotFindStringInList(
    list: *mut bot_stringlist_t,
    string: *mut c_char,
) -> *mut bot_stringlist_t {
    unsafe {
        let mut s = list;
        while !s.is_null() {
            if libc::strcmp((*s).string, string) == 0 {
                return s;
            }
            s = (*s).next;
        }
    }
    core::ptr::null_mut()
}

/// Raven `BotPrintReplyChatKeys`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2556-2590`
pub fn BotPrintReplyChatKeys(bot: &mut BotLib, replychat: *mut bot_replychat_t) {
    unsafe {
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "[");
        let mut key = (*replychat).keys;
        while !key.is_null() {
            let flags = (*key).flags;
            if flags & RCKFL_AND != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "&");
            } else if flags & RCKFL_NOT != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "!");
            }
            if flags & RCKFL_NAME != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "name");
            } else if flags & RCKFL_GENDERFEMALE != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "female");
            } else if flags & RCKFL_GENDERMALE != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "male");
            } else if flags & RCKFL_GENDERLESS != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "it");
            } else if flags & RCKFL_VARIABLES != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "(");
                let mut mp = (*key).r#match;
                while !mp.is_null() {
                    if (*mp).r#type == MT_STRING {
                        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "string");
                    } else {
                        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "var");
                    }
                    if !(*mp).next.is_null() {
                        crate::be_interface::botimport_print(bot, PRT_MESSAGE, ", ");
                    }
                    mp = (*mp).next;
                }
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, ")");
            } else if flags & RCKFL_STRING != 0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "string");
            }
            if !(*key).next.is_null() {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, ", ");
            } else {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "] = ...\n");
            }
            key = (*key).next;
        }
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "{\n");
    }
}

/// Raven `BotResetChatAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2859-2871`
pub fn BotResetChatAI(bot: &mut BotLib) {
    unsafe {
        let mut rchat = bot.replychats;
        while !rchat.is_null() {
            let mut m = (*rchat).firstchatmessage;
            while !m.is_null() {
                (*m).time = 0.0;
                m = (*m).next;
            }
            rchat = (*rchat).next;
        }
    }
}

/// Raven `BotRemoveConsoleMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:279-302`
pub fn BotRemoveConsoleMessage(bot: &mut BotLib, chatstate: c_int, handle: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
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
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    let m = AllocConsoleMessage(bot);
    if m.is_null() {
        unsafe {
            crate::be_interface::botimport_print(bot, PRT_ERROR, "empty console message heap\n")
        };
        return;
    }
    unsafe {
        (*cs).handle += 1;
        if (*cs).handle <= 0 || (*cs).handle > 8192 {
            (*cs).handle = 1;
        }
        (*m).handle = (*cs).handle;
        (*m).time = AAS_Time(bot);
        (*m).r#type = r#type;
        libc::strncpy((*m).message.as_mut_ptr(), message, 256);
        (*m).next = core::ptr::null_mut();
        if !(*cs).lastmessage.is_null() {
            (*(*cs).lastmessage).next = m;
            (*m).prev = (*cs).lastmessage;
            (*cs).lastmessage = m;
        } else {
            (*cs).lastmessage = m;
            (*cs).firstmessage = m;
            (*m).prev = core::ptr::null_mut();
        }
        (*cs).numconsolemessages += 1;
    }
}

/// Raven `BotNextConsoleMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:350-363`
pub fn BotNextConsoleMessage(
    bot: &mut BotLib,
    chatstate: c_int,
    cm: *mut bot_consolemessage_t,
) -> c_int {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return 0;
    }
    unsafe {
        if !(*cs).firstmessage.is_null() {
            Com_Memcpy(
                cm as *mut (),
                (*cs).firstmessage as *const (),
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
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return 0;
    }
    unsafe { (*cs).numconsolemessages }
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
pub fn BotDumpSynonymList(bot: &mut BotLib, synlist: *mut bot_synonymlist_t) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        let mut syn = synlist;
        while !syn.is_null() {
            libc::fprintf(fp, c"%ld : [".as_ptr(), (*syn).context);
            let mut synonym = (*syn).firstsynonym;
            while !synonym.is_null() {
                libc::fprintf(
                    fp,
                    c"(\"%s\", %1.2f)".as_ptr(),
                    (*synonym).string,
                    (*synonym).weight as f64,
                );
                if !(*synonym).next.is_null() {
                    libc::fprintf(fp, c", ".as_ptr());
                }
                synonym = (*synonym).next;
            }
            libc::fprintf(fp, c"]\n".as_ptr());
            syn = (*syn).next;
        }
    }
}

/// Raven `BotReplaceReplySynonyms`.
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
            let mut syn = bot.synonyms;
            let mut replaced = false;
            while !syn.is_null() {
                if (*syn).context & context == 0 {
                    syn = (*syn).next;
                    continue;
                }
                let mut synonym = (*(*syn).firstsynonym).next;
                while !synonym.is_null() {
                    let str2 = StringContainsWord(str1, (*synonym).string, 0);
                    if str2.is_null() || str2 != str1 {
                        synonym = (*synonym).next;
                        continue;
                    }
                    let replacement = (*(*syn).firstsynonym).string;
                    let str2b = StringContainsWord(str1, replacement, 0);
                    if !str2b.is_null() && str2b == str1 {
                        synonym = (*synonym).next;
                        continue;
                    }
                    let syn_len = libc::strlen((*synonym).string) as isize;
                    let rep_len = libc::strlen(replacement) as isize;
                    let tail = str1.offset(syn_len);
                    let tail_len = libc::strlen(tail);
                    libc::memmove(
                        str1.offset(rep_len) as *mut c_void,
                        tail as *const c_void,
                        tail_len + 1,
                    );
                    Com_Memcpy(str1 as *mut (), replacement as *const (), rep_len as usize);
                    break;
                }
                if !synonym.is_null() {
                    replaced = true;
                    break;
                }
                syn = (*syn).next;
            }
            let _ = replaced;
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
pub fn BotDumpRandomStringList(bot: &mut BotLib, randomlist: *mut bot_randomlist_t) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        let mut random = randomlist;
        while !random.is_null() {
            libc::fprintf(fp, c"%s = {".as_ptr(), (*random).string);
            let mut rs = (*random).firstrandomstring;
            while !rs.is_null() {
                libc::fprintf(fp, c"\"%s\"".as_ptr(), (*rs).string);
                if !(*rs).next.is_null() {
                    libc::fprintf(fp, c", ".as_ptr());
                } else {
                    libc::fprintf(fp, c"}\n".as_ptr());
                }
                rs = (*rs).next;
            }
            random = (*random).next;
        }
    }
}

/// Raven `BotDumpMatchTemplates`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1060-1090`
pub fn BotDumpMatchTemplates(bot: &mut BotLib, matches: *mut bot_matchtemplate_t) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        let mut mt = matches;
        while !mt.is_null() {
            libc::fprintf(fp, c"{ ".as_ptr());
            let mut mp = (*mt).first;
            while !mp.is_null() {
                if (*mp).r#type == MT_STRING {
                    let mut ms = (*mp).firststring;
                    while !ms.is_null() {
                        libc::fprintf(fp, c"\"%s\"".as_ptr(), (*ms).string);
                        if !(*ms).next.is_null() {
                            libc::fprintf(fp, c"|".as_ptr());
                        }
                        ms = (*ms).next;
                    }
                } else if (*mp).r#type == MT_VARIABLE {
                    libc::fprintf(fp, c"%d".as_ptr(), (*mp).variable);
                }
                if !(*mp).next.is_null() {
                    libc::fprintf(fp, c", ".as_ptr());
                }
                mp = (*mp).next;
            }
            libc::fprintf(fp, c" = (%d, %d);}\n".as_ptr(), (*mt).r#type, (*mt).subtype);
            mt = (*mt).next;
        }
    }
}

/// Raven `BotFreeMatchPieces`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1097-1115`
pub fn BotFreeMatchPieces(bot: &mut BotLib, matchpieces: *mut bot_matchpiece_t) {
    unsafe {
        let mut mp = matchpieces;
        while !mp.is_null() {
            let nextmp = (*mp).next;
            if (*mp).r#type == MT_STRING {
                let mut ms = (*mp).firststring;
                while !ms.is_null() {
                    let nextms = (*ms).next;
                    FreeMemory(bot, ms as *mut ());
                    ms = nextms;
                }
            }
            FreeMemory(bot, mp as *mut ());
            mp = nextmp;
        }
    }
}

/// Raven `StringsMatch`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1340-1410`
pub fn StringsMatch(pieces: *mut bot_matchpiece_t, r#match: *mut bot_match_t) -> c_int {
    unsafe {
        let mut lastvariable: isize = -1;
        let mut strptr = (*r#match).string.as_mut_ptr();
        let mut mp = pieces;
        while !mp.is_null() {
            if (*mp).r#type == MT_STRING {
                let mut newstrptr: *mut c_char = core::ptr::null_mut();
                let mut ms = (*mp).firststring;
                let mut lastms = ms;
                while !ms.is_null() {
                    if libc::strlen((*ms).string) == 0 {
                        newstrptr = strptr;
                        lastms = ms;
                        break;
                    }
                    let index = StringContains(strptr, (*ms).string, 0);
                    if index >= 0 {
                        newstrptr = strptr.offset(index as isize);
                        if lastvariable >= 0 {
                            (*r#match).variables[lastvariable as usize].length =
                                ((newstrptr as isize) - ((*r#match).string.as_ptr() as isize))
                                    as i32
                                    - (*r#match).variables[lastvariable as usize].offset as i32;
                            lastvariable = -1;
                            lastms = ms;
                            break;
                        } else if index == 0 {
                            lastms = ms;
                            break;
                        }
                        newstrptr = core::ptr::null_mut();
                    }
                    lastms = ms;
                    ms = (*ms).next;
                }
                if newstrptr.is_null() {
                    return 0;
                }
                strptr = newstrptr.offset(libc::strlen((*lastms).string) as isize);
            } else if (*mp).r#type == MT_VARIABLE {
                let var = (*mp).variable as usize;
                (*r#match).variables[var].offset =
                    ((strptr as isize) - ((*r#match).string.as_ptr() as isize)) as c_char;
                lastvariable = var as isize;
            }
            mp = (*mp).next;
        }
        if mp.is_null() && (lastvariable >= 0 || libc::strlen(strptr) == 0) {
            if lastvariable >= 0 {
                let off = (*r#match).variables[lastvariable as usize].offset;
                (*r#match).variables[lastvariable as usize].length =
                    libc::strlen((*r#match).string.as_ptr().offset(off as isize)) as i32;
            }
            return 1;
        }
    }
    0
}

/// Raven `BotDumpReplyChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1619-1667`
pub fn BotDumpReplyChat(bot: &mut BotLib, replychat: *mut bot_replychat_t) {
    unsafe {
        let fp = Log_FilePointer(bot);
        if fp.is_null() {
            return;
        }
        libc::fprintf(fp, c"BotDumpReplyChat:\n".as_ptr());
        let mut rp = replychat;
        while !rp.is_null() {
            libc::fprintf(fp, c"[".as_ptr());
            let mut key = (*rp).keys;
            while !key.is_null() {
                let flags = (*key).flags;
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
                    let mut mp = (*key).r#match;
                    while !mp.is_null() {
                        if (*mp).r#type == MT_STRING {
                            libc::fprintf(fp, c"\"%s\"".as_ptr(), (*(*mp).firststring).string);
                        } else {
                            libc::fprintf(fp, c"%d".as_ptr(), (*mp).variable);
                        }
                        if !(*mp).next.is_null() {
                            libc::fprintf(fp, c", ".as_ptr());
                        }
                        mp = (*mp).next;
                    }
                    libc::fprintf(fp, c")".as_ptr());
                } else if flags & RCKFL_STRING != 0 {
                    libc::fprintf(fp, c"\"%s\"".as_ptr(), (*key).string);
                }
                if !(*key).next.is_null() {
                    libc::fprintf(fp, c", ".as_ptr());
                } else {
                    libc::fprintf(fp, c"] = %1.0f\n".as_ptr(), (*rp).priority as f64);
                }
                key = (*key).next;
            }
            libc::fprintf(fp, c"{\n".as_ptr());
            let mut cm = (*rp).firstchatmessage;
            while !cm.is_null() {
                libc::fprintf(fp, c"\t\"%s\";\n".as_ptr(), (*cm).chatmessage);
                cm = (*cm).next;
            }
            libc::fprintf(fp, c"}\n".as_ptr());
            rp = (*rp).next;
        }
    }
}

/// Raven `BotCheckValidReplyChatKeySet`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1704-1804`
pub fn BotCheckValidReplyChatKeySet(
    bot: &mut BotLib,
    source: *mut source_t,
    keys: *mut bot_replychatkey_t,
) {
    unsafe {
        let mut allprefixed = true;
        let mut hasvariableskey = false;
        let mut hasstringkey = false;
        let mut key = keys;
        while !key.is_null() {
            let flags = (*key).flags;
            if flags & (RCKFL_AND | RCKFL_NOT) == 0 {
                allprefixed = false;
                if flags & RCKFL_VARIABLES != 0 {
                    let mut m = (*key).r#match;
                    while !m.is_null() {
                        if (*m).r#type == MT_VARIABLE {
                            hasvariableskey = true;
                        }
                        m = (*m).next;
                    }
                } else if flags & RCKFL_STRING != 0 {
                    hasstringkey = true;
                }
            } else if (flags & RCKFL_AND != 0) && (flags & RCKFL_STRING != 0) {
                let mut key2 = keys;
                while !key2.is_null() {
                    if key2 == key {
                        key2 = (*key2).next;
                        continue;
                    }
                    if (*key2).flags & RCKFL_NOT != 0 {
                        key2 = (*key2).next;
                        continue;
                    }
                    if (*key2).flags & RCKFL_VARIABLES != 0 {
                        let mut m = (*key2).r#match;
                        let mut found = false;
                        while !m.is_null() {
                            if (*m).r#type == MT_STRING {
                                let mut ms = (*m).firststring;
                                let mut hit = false;
                                while !ms.is_null() {
                                    if StringContains((*ms).string, (*key).string, 0) != -1 {
                                        hit = true;
                                        break;
                                    }
                                    ms = (*ms).next;
                                }
                                if hit {
                                    found = true;
                                    break;
                                }
                            } else if (*m).r#type == MT_VARIABLE {
                                found = true;
                                break;
                            }
                            m = (*m).next;
                        }
                        if !found {
                            SourceWarning(bot, source, c"one of the match templates does not leave space for the key with the & prefix".as_ptr());
                        }
                    }
                    key2 = (*key2).next;
                }
            }
            if (flags & RCKFL_NOT != 0) && (flags & RCKFL_STRING != 0) {
                let mut key2 = keys;
                while !key2.is_null() {
                    if key2 == key {
                        key2 = (*key2).next;
                        continue;
                    }
                    if (*key2).flags & RCKFL_NOT != 0 {
                        key2 = (*key2).next;
                        continue;
                    }
                    if (*key2).flags & RCKFL_STRING != 0 {
                        if StringContains((*key2).string, (*key).string, 0) != -1 {
                            SourceWarning(
                                bot,
                                source,
                                c"the key with prefix ! is inside another key".as_ptr(),
                            );
                        }
                    } else if (*key2).flags & RCKFL_VARIABLES != 0 {
                        let mut m = (*key2).r#match;
                        while !m.is_null() {
                            if (*m).r#type == MT_STRING {
                                let mut ms = (*m).firststring;
                                while !ms.is_null() {
                                    if StringContains((*ms).string, (*key).string, 0) != -1 {
                                        SourceWarning(bot, source, c"the key with prefix ! is inside the match template string".as_ptr());
                                    }
                                    ms = (*ms).next;
                                }
                            }
                            m = (*m).next;
                        }
                    }
                    key2 = (*key2).next;
                }
            }
            key = (*key).next;
        }
        if allprefixed {
            SourceWarning(bot, source, c"all keys have a & or ! prefix".as_ptr());
        }
        if hasvariableskey && hasstringkey {
            SourceWarning(bot, source, c"variables from the match template(s) could be invalid when outputting one of the chat messages".as_ptr());
        }
    }
}

/// Raven `BotDumpInitialChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1971-1989`
pub fn BotDumpInitialChat(bot: &mut BotLib, chat: *mut bot_chat_t) {
    unsafe {
        Log_Write(bot, c"{".as_ptr() as *mut c_char);
        let mut t = (*chat).types;
        while !t.is_null() {
            Log_Write(bot, c" type".as_ptr() as *mut c_char);
            Log_Write(bot, c" {".as_ptr() as *mut c_char);
            Log_Write(bot, c"  numchatmessages".as_ptr() as *mut c_char);
            let mut m = (*t).firstchatmessage;
            while !m.is_null() {
                Log_Write(bot, c"  chatmessage".as_ptr() as *mut c_char);
                m = (*m).next;
            }
            Log_Write(bot, c" }".as_ptr() as *mut c_char);
            t = (*t).next;
        }
        Log_Write(bot, c"}".as_ptr() as *mut c_char);
    }
}

/// Raven `BotFreeChatFile`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2168-2176`
pub fn BotFreeChatFile(bot: &mut BotLib, chatstate: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        if !(*cs).chat.is_null() {
            FreeMemory(bot, (*cs).chat as *mut ());
        }
        (*cs).chat = core::ptr::null_mut();
    }
}

/// Raven `BotChooseInitialChatMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2390-2441`
pub fn BotChooseInitialChatMessage(
    common: &mut Common,
    bot: &mut BotLib,
    cs: *mut bot_chatstate_t,
    r#type: *mut c_char,
) -> *mut c_char {
    unsafe {
        let chat = (*cs).chat;
        let mut t = (*chat).types;
        while !t.is_null() {
            if libc::strcasecmp((*t).name.as_ptr(), r#type) == 0 {
                let mut numchatmessages = 0;
                let mut m = (*t).firstchatmessage;
                while !m.is_null() {
                    if (*m).time <= AAS_Time(bot) {
                        numchatmessages += 1;
                    }
                    m = (*m).next;
                }
                if numchatmessages <= 0 {
                    let mut besttime = 0.0f32;
                    let mut bestchatmessage: *mut bot_chatmessage_t = core::ptr::null_mut();
                    let mut m = (*t).firstchatmessage;
                    while !m.is_null() {
                        if besttime == 0.0 || (*m).time < besttime {
                            bestchatmessage = m;
                            besttime = (*m).time;
                        }
                        m = (*m).next;
                    }
                    if !bestchatmessage.is_null() {
                        return (*bestchatmessage).chatmessage;
                    }
                } else {
                    let mut n = (common.qrand.irand(0, numchatmessages - 1)) as c_int;
                    let mut m = (*t).firstchatmessage;
                    while !m.is_null() {
                        if (*m).time <= AAS_Time(bot) {
                            n -= 1;
                            if n < 0 {
                                (*m).time = AAS_Time(bot) + CHATMESSAGE_RECENTTIME as f32;
                                return (*m).chatmessage;
                            }
                        }
                        m = (*m).next;
                    }
                }
                return core::ptr::null_mut();
            }
            t = (*t).next;
        }
    }
    core::ptr::null_mut()
}

/// Raven `BotChatLength`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2754-2761`
pub fn BotChatLength(bot: &mut BotLib, chatstate: c_int) -> c_int {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return 0;
    }
    unsafe { libc::strlen((*cs).chatmessage.as_ptr()) as c_int }
}

/// Raven `BotGetChatMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2804-2816`
pub fn BotGetChatMessage(bot: &mut BotLib, chatstate: c_int, buf: *mut c_char, size: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        BotRemoveTildes((*cs).chatmessage.as_mut_ptr());
        libc::strncpy(buf, (*cs).chatmessage.as_ptr(), (size - 1) as usize);
        *buf.offset((size - 1) as isize) = 0;
        libc::strcpy((*cs).chatmessage.as_mut_ptr(), c"".as_ptr());
    }
}

/// Raven `BotSetChatGender`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2823-2835`
pub fn BotSetChatGender(bot: &mut BotLib, chatstate: c_int, gender: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        (*cs).gender = match gender {
            g if g == CHAT_GENDERFEMALE => CHAT_GENDERFEMALE,
            g if g == CHAT_GENDERMALE => CHAT_GENDERMALE,
            _ => CHAT_GENDERLESS,
        };
    }
}

/// Raven `BotSetChatName`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2842-2852`
pub fn BotSetChatName(bot: &mut BotLib, chatstate: c_int, name: *mut c_char, client: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        (*cs).client = client;
        Com_Memset(
            (*cs).name.as_mut_ptr() as *mut (),
            0,
            core::mem::size_of_val(&(*cs).name),
        );
        libc::strncpy(
            (*cs).name.as_mut_ptr(),
            name,
            core::mem::size_of_val(&(*cs).name),
        );
        *(*cs).name.as_mut_ptr().offset(31) = 0;
    }
}

/// Raven `BotReplaceSynonyms`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:744-757`
pub fn BotReplaceSynonyms(bot: &mut BotLib, string: *mut c_char, context: c_ulong) {
    unsafe {
        let mut syn = bot.synonyms;
        while !syn.is_null() {
            if (*syn).context & context != 0 {
                let mut synonym = (*(*syn).firstsynonym).next;
                while !synonym.is_null() {
                    StringReplaceWords(string, (*synonym).string, (*(*syn).firstsynonym).string);
                    synonym = (*synonym).next;
                }
            }
            syn = (*syn).next;
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
    unsafe {
        let mut syn = bot.synonyms;
        while !syn.is_null() {
            if (*syn).context & context != 0 {
                let weight = common.qrand.flrand(0.0, 1.0) * (*syn).totalweight;
                if weight != 0.0 {
                    let mut curweight = 0.0f32;
                    let mut replacement = (*syn).firstsynonym;
                    while !replacement.is_null() {
                        curweight += (*replacement).weight;
                        if weight < curweight {
                            break;
                        }
                        replacement = (*replacement).next;
                    }
                    if !replacement.is_null() {
                        let mut synonym = (*syn).firstsynonym;
                        while !synonym.is_null() {
                            if synonym != replacement {
                                StringReplaceWords(
                                    string,
                                    (*synonym).string,
                                    (*replacement).string,
                                );
                            }
                            synonym = (*synonym).next;
                        }
                    }
                }
            }
            syn = (*syn).next;
        }
    }
}

/// Raven `BotFreeMatchTemplates`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1224-1234`
pub fn BotFreeMatchTemplates(bot: &mut BotLib, mt: *mut bot_matchtemplate_t) {
    unsafe {
        let mut mt = mt;
        while !mt.is_null() {
            let nextmt = (*mt).next;
            BotFreeMatchPieces(bot, (*mt).first);
            FreeMemory(bot, mt as *mut ());
            mt = nextmt;
        }
    }
}

/// Raven `BotFindMatch`.
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
        let mut ms = bot.matchtemplates;
        while !ms.is_null() {
            if (*ms).context & context != 0 {
                for i in 0..MAX_MATCHVARIABLES {
                    (*r#match).variables[i].offset = -1;
                }
                if StringsMatch((*ms).first, r#match) != 0 {
                    (*r#match).r#type = (*ms).r#type;
                    (*r#match).subtype = (*ms).subtype;
                    return 1;
                }
            }
            ms = (*ms).next;
        }
    }
    0
}

/// Raven `BotCheckChatMessageIntegrety`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1496-1560`
pub fn BotCheckChatMessageIntegrety(
    common: &mut Common,
    bot: &mut BotLib,
    message: *mut c_char,
    stringlist: *mut bot_stringlist_t,
) -> *mut bot_stringlist_t {
    let mut stringlist = stringlist;
    unsafe {
        let mut msgptr = message;
        let mut temp = [0 as c_char; MAX_MESSAGE_SIZE];
        while *msgptr != 0 {
            if *msgptr == ESCAPE_CHAR as c_char {
                msgptr = msgptr.offset(1);
                match *msgptr as u8 as char {
                    'v' => {
                        msgptr = msgptr.offset(1);
                        while *msgptr != 0 && *msgptr != ESCAPE_CHAR as c_char {
                            msgptr = msgptr.offset(1);
                        }
                        if *msgptr != 0 {
                            msgptr = msgptr.offset(1);
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
                        if RandomString(common, bot, temp.as_mut_ptr()).is_null()
                            && BotFindStringInList(stringlist, temp.as_mut_ptr()).is_null()
                        {
                            Log_Write(bot, c"MISSING RANDOM".as_ptr() as *mut c_char);
                            let tlen = libc::strlen(temp.as_ptr());
                            let s = GetClearedMemory(
                                bot,
                                (core::mem::size_of::<bot_stringlist_t>() + tlen + 1) as u64,
                            ) as *mut bot_stringlist_t;
                            (*s).string =
                                (s as *mut c_char).add(core::mem::size_of::<bot_stringlist_t>());
                            libc::strcpy((*s).string, temp.as_ptr());
                            (*s).next = stringlist;
                            stringlist = s;
                        }
                    }
                    _ => {
                        crate::be_interface::botimport_print(
                            bot,
                            PRT_FATAL,
                            "BotCheckChatMessageIntegrety: invalid escape char\n",
                        );
                    }
                }
            } else {
                msgptr = msgptr.offset(1);
            }
        }
    }
    stringlist
}

/// Raven `BotFreeReplyChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1674-1697`
pub fn BotFreeReplyChat(bot: &mut BotLib, replychat: *mut bot_replychat_t) {
    unsafe {
        let mut rp = replychat;
        while !rp.is_null() {
            let nextrp = (*rp).next;
            let mut key = (*rp).keys;
            while !key.is_null() {
                let nextkey = (*key).next;
                if !(*key).r#match.is_null() {
                    BotFreeMatchPieces(bot, (*key).r#match);
                }
                if !(*key).string.is_null() {
                    FreeMemory(bot, (*key).string as *mut ());
                }
                FreeMemory(bot, key as *mut ());
                key = nextkey;
            }
            let mut cm = (*rp).firstchatmessage;
            while !cm.is_null() {
                let nextcm = (*cm).next;
                FreeMemory(bot, cm as *mut ());
                cm = nextcm;
            }
            FreeMemory(bot, rp as *mut ());
            rp = nextrp;
        }
    }
}

/// Raven `BotNumInitialChats`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2448-2468`
pub fn BotNumInitialChats(bot: &mut BotLib, chatstate: c_int, r#type: *mut c_char) -> c_int {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return 0;
    }
    unsafe {
        let mut t = (*(*cs).chat).types;
        while !t.is_null() {
            if libc::strcasecmp((*t).name.as_ptr(), r#type) == 0 {
                if LibVarGetValue(bot, c"bot_testichat".as_ptr() as *mut c_char) != 0.0 {
                    crate::be_interface::botimport_print(bot, PRT_MESSAGE, "chat lines\n");
                    crate::be_interface::botimport_print(bot, PRT_MESSAGE, "-------------------\n");
                }
                return (*t).numchatmessages;
            }
            t = (*t).next;
        }
    }
    0
}

/// Raven `BotEnterChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2768-2797`
pub fn BotEnterChat(bot: &mut BotLib, chatstate: c_int, clientto: c_int, sendto: c_int) {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        if libc::strlen((*cs).chatmessage.as_ptr()) != 0 {
            BotRemoveTildes((*cs).chatmessage.as_mut_ptr());
            if LibVarGetValue(bot, c"bot_testichat".as_ptr() as *mut c_char) != 0.0 {
                crate::be_interface::botimport_print(bot, PRT_MESSAGE, "chatmessage\n");
            } else {
                let msg = std::ffi::CStr::from_ptr((*cs).chatmessage.as_ptr())
                    .to_string_lossy()
                    .into_owned();
                let cmdline = match sendto {
                    t if t == CHAT_TEAM => format!("say_team {msg}"),
                    t if t == CHAT_TELL => format!("tell {clientto} {msg}"),
                    _ => format!("say {msg}"),
                };
                let _ = CHAT_ALL;
                let mut cstring = std::ffi::CString::new(cmdline)
                    .unwrap_or_default()
                    .into_bytes_with_nul();
                EA_Command(bot, (*cs).client, cstring.as_mut_ptr() as *mut c_char);
            }
            libc::strcpy((*cs).chatmessage.as_mut_ptr(), c"".as_ptr());
        }
    }
}

/// Raven `BotAllocChatState`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2878-2891`
pub fn BotAllocChatState(bot: &mut BotLib) -> c_int {
    for i in 1..=MAX_CLIENTS {
        if bot.botchatstates[i].is_null() {
            bot.botchatstates[i] = {
                GetClearedMemory(bot, (core::mem::size_of::<bot_chatstate_t>()) as u64)
                    as *mut bot_chatstate_t
            };
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
        unsafe {
            crate::be_interface::botimport_print(bot, PRT_FATAL, "chat state handle out of range\n")
        };
        return;
    }
    if bot.botchatstates[handle as usize].is_null() {
        unsafe { crate::be_interface::botimport_print(bot, PRT_FATAL, "invalid chat state\n") };
        return;
    }
    if LibVarGetValue(bot, c"bot_reloadcharacters".as_ptr() as *mut c_char) != 0.0 {
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
    FreeMemory(bot, bot.botchatstates[handle as usize] as *mut ());
    bot.botchatstates[handle as usize] = core::ptr::null_mut();
}

/// Raven `BotCheckInitialChatIntegrety`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1567-1586`
pub fn BotCheckInitialChatIntegrety(common: &mut Common, bot: &mut BotLib, chat: *mut bot_chat_t) {
    let mut stringlist: *mut bot_stringlist_t = core::ptr::null_mut();
    unsafe {
        let mut t = (*chat).types;
        while !t.is_null() {
            let mut cm = (*t).firstchatmessage;
            while !cm.is_null() {
                stringlist =
                    BotCheckChatMessageIntegrety(common, bot, (*cm).chatmessage, stringlist);
                cm = (*cm).next;
            }
            t = (*t).next;
        }
        let mut s = stringlist;
        while !s.is_null() {
            let nexts = (*s).next;
            FreeMemory(bot, s as *mut ());
            s = nexts;
        }
    }
}

/// Raven `BotCheckReplyChatIntegrety`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1593-1612`
pub fn BotCheckReplyChatIntegrety(
    common: &mut Common,
    bot: &mut BotLib,
    replychat: *mut bot_replychat_t,
) {
    let mut stringlist: *mut bot_stringlist_t = core::ptr::null_mut();
    unsafe {
        let mut rp = replychat;
        while !rp.is_null() {
            let mut cm = (*rp).firstchatmessage;
            while !cm.is_null() {
                stringlist =
                    BotCheckChatMessageIntegrety(common, bot, (*cm).chatmessage, stringlist);
                cm = (*cm).next;
            }
            rp = (*rp).next;
        }
        let mut s = stringlist;
        while !s.is_null() {
            let nexts = (*s).next;
            FreeMemory(bot, s as *mut ());
            s = nexts;
        }
    }
}

/// Raven `BotExpandChatMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2241-2355`
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
                            crate::be_interface::botimport_print(
                                bot,
                                PRT_ERROR,
                                "BotConstructChat: variable out of range\n",
                            );
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
                                crate::be_interface::botimport_print(
                                    bot,
                                    PRT_ERROR,
                                    "BotConstructChat: message too long\n",
                                );
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
                        let ptr = RandomString(common, bot, temp.as_mut_ptr());
                        if ptr.is_null() {
                            crate::be_interface::botimport_print(
                                bot,
                                PRT_ERROR,
                                "BotConstructChat: unknown random string\n",
                            );
                            return 0;
                        }
                        let plen = libc::strlen(ptr) as isize;
                        if len + plen >= MAX_MESSAGE_SIZE as isize {
                            crate::be_interface::botimport_print(
                                bot,
                                PRT_ERROR,
                                "BotConstructChat: message too long\n",
                            );
                            return 0;
                        }
                        libc::strcpy(outputbuf.offset(len), ptr);
                        len += plen;
                        expansion = 1;
                    }
                    _ => {
                        crate::be_interface::botimport_print(
                            bot,
                            PRT_FATAL,
                            "BotConstructChat: invalid escape char\n",
                        );
                    }
                }
            } else {
                *outputbuf.offset(len) = *msgptr;
                len += 1;
                msgptr = msgptr.offset(1);
                if len as usize >= MAX_MESSAGE_SIZE {
                    crate::be_interface::botimport_print(
                        bot,
                        PRT_ERROR,
                        "BotConstructChat: message too long\n",
                    );
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
        if !bot.botchatstates[i].is_null() {
            BotFreeChatState(bot, i as c_int);
        }
    }
    for i in 0..MAX_CLIENTS {
        unsafe {
            if !bot.ichatdata[i].is_null() {
                FreeMemory(bot, (*bot.ichatdata[i]).chat as *mut ());
                FreeMemory(bot, bot.ichatdata[i] as *mut ());
                bot.ichatdata[i] = core::ptr::null_mut();
            }
        }
    }
    if !bot.consolemessageheap.is_null() {
        FreeMemory(bot, bot.consolemessageheap as *mut ());
    }
    bot.consolemessageheap = core::ptr::null_mut();
    if !bot.matchtemplates.is_null() {
        BotFreeMatchTemplates(bot, bot.matchtemplates);
    }
    bot.matchtemplates = core::ptr::null_mut();
    if !bot.randomstrings.is_null() {
        FreeMemory(bot, bot.randomstrings as *mut ());
    }
    bot.randomstrings = core::ptr::null_mut();
    if !bot.synonyms.is_null() {
        FreeMemory(bot, bot.synonyms as *mut ());
    }
    bot.synonyms = core::ptr::null_mut();
    if !bot.replychats.is_null() {
        BotFreeReplyChat(bot, bot.replychats);
    }
    bot.replychats = core::ptr::null_mut();
}

/// Raven `InitConsoleMessageHeap`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:223-243`
pub fn InitConsoleMessageHeap(bot: &mut BotLib) {
    unsafe {
        if !bot.consolemessageheap.is_null() {
            FreeMemory(bot, bot.consolemessageheap as *mut ());
        }
        let max_messages = LibVarValue(
            bot,
            c"max_messages".as_ptr() as *mut c_char,
            c"1024".as_ptr() as *mut c_char,
        ) as isize;
        bot.consolemessageheap = GetClearedHunkMemory(
            bot,
            (max_messages as usize * core::mem::size_of::<bot_consolemessage_t>()) as u64,
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

/// Raven `BotConstructChatMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2362-2382`
pub fn BotConstructChatMessage(
    common: &mut Common,
    bot: &mut BotLib,
    chatstate: *mut bot_chatstate_t,
    message: *mut c_char,
    mcontext: c_ulong,
    r#match: *mut bot_match_t,
    vcontext: c_ulong,
    reply: c_int,
) {
    unsafe {
        let mut srcmessage = [0 as c_char; MAX_MESSAGE_SIZE];
        libc::strcpy(srcmessage.as_mut_ptr(), message);
        let mut i = 0;
        while i < 10 {
            if BotExpandChatMessage(
                common,
                bot,
                (*chatstate).chatmessage.as_mut_ptr(),
                srcmessage.as_mut_ptr(),
                mcontext,
                r#match,
                vcontext,
                reply,
            ) == 0
            {
                break;
            }
            libc::strcpy(srcmessage.as_mut_ptr(), (*chatstate).chatmessage.as_ptr());
            i += 1;
        }
        if i >= 10 {
            crate::be_interface::botimport_print(
                bot,
                PRT_WARNING,
                "too many expansions in chat message\n",
            );
            crate::be_interface::botimport_print(bot, PRT_WARNING, "chatmessage\n");
        }
    }
}

/// Raven `BotInitialChat`.
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
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return;
    }
    unsafe {
        if (*cs).chat.is_null() {
            return;
        }
        let message = BotChooseInitialChatMessage(common, bot, cs, r#type);
        if message.is_null() {
            return;
        }
        let mut r#match = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| {
                mp_qshared::common::mp::botlib::bot_matchvariable_s::bot_matchvariable_t {
                    offset: 0,
                    length: 0,
                }
            }),
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
        BotConstructChatMessage(
            common,
            bot,
            cs,
            message,
            mcontext as c_ulong,
            &mut r#match,
            0,
            0,
        );
    }
}

/// Raven `BotReplyChat`.
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
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return 0;
    }
    unsafe {
        let mut r#match = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| {
                mp_qshared::common::mp::botlib::bot_matchvariable_s::bot_matchvariable_t {
                    offset: 0,
                    length: 0,
                }
            }),
        };
        Com_Memset(
            &mut r#match as *mut bot_match_t as *mut (),
            0,
            core::mem::size_of::<bot_match_t>(),
        );
        libc::strcpy(r#match.string.as_mut_ptr(), message);
        let mut bestpriority = -1.0f32;
        let mut bestchatmessage: *mut bot_chatmessage_t = core::ptr::null_mut();
        let mut bestrchat: *mut bot_replychat_t = core::ptr::null_mut();
        let mut bestmatch = bot_match_t {
            string: [0; MAX_MESSAGE_SIZE],
            r#type: 0,
            subtype: 0,
            variables: core::array::from_fn(|_| {
                mp_qshared::common::mp::botlib::bot_matchvariable_s::bot_matchvariable_t {
                    offset: 0,
                    length: 0,
                }
            }),
        };
        let mut rchat = bot.replychats;
        while !rchat.is_null() {
            let mut found = false;
            let mut key = (*rchat).keys;
            let mut broke = false;
            while !key.is_null() {
                let flags = (*key).flags;
                let res = if flags & RCKFL_NAME != 0 {
                    StringContains(message, (*cs).name.as_mut_ptr(), 0) != -1
                } else if flags & RCKFL_BOTNAMES != 0 {
                    StringContains((*key).string, (*cs).name.as_mut_ptr(), 0) != -1
                } else if flags & RCKFL_GENDERFEMALE != 0 {
                    (*cs).gender == CHAT_GENDERFEMALE
                } else if flags & RCKFL_GENDERMALE != 0 {
                    (*cs).gender == CHAT_GENDERMALE
                } else if flags & RCKFL_GENDERLESS != 0 {
                    (*cs).gender == CHAT_GENDERLESS
                } else if flags & RCKFL_VARIABLES != 0 {
                    StringsMatch((*key).r#match, &mut r#match) != 0
                } else if flags & RCKFL_STRING != 0 {
                    !StringContainsWord(message, (*key).string, 0).is_null()
                } else {
                    false
                };
                if flags & RCKFL_AND != 0 {
                    if !res {
                        found = false;
                        broke = true;
                        break;
                    }
                } else if flags & RCKFL_NOT != 0 {
                    if res {
                        found = false;
                        broke = true;
                        break;
                    }
                } else if res {
                    found = true;
                }
                key = (*key).next;
            }
            let _ = broke;
            if found && (*rchat).priority > bestpriority {
                let mut numchatmessages = 0;
                let mut m = (*rchat).firstchatmessage;
                while !m.is_null() {
                    if (*m).time <= AAS_Time(bot) {
                        numchatmessages += 1;
                    }
                    m = (*m).next;
                }
                let mut num = (common.qrand.irand(0, numchatmessages.max(1) - 1)) as c_int;
                let mut m = (*rchat).firstchatmessage;
                while !m.is_null() {
                    num -= 1;
                    if num < 0 {
                        break;
                    }
                    m = (*m).next;
                }
                if !m.is_null() {
                    Com_Memcpy(
                        &mut bestmatch as *mut bot_match_t as *mut (),
                        &r#match as *const bot_match_t as *const (),
                        core::mem::size_of::<bot_match_t>(),
                    );
                    bestchatmessage = m;
                    bestrchat = rchat;
                    bestpriority = (*rchat).priority;
                }
            }
            rchat = (*rchat).next;
        }
        if !bestchatmessage.is_null() {
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
            if LibVarGetValue(bot, c"bot_testrchat".as_ptr() as *mut c_char) != 0.0 {
                let mut m = (*bestrchat).firstchatmessage;
                while !m.is_null() {
                    BotConstructChatMessage(
                        common,
                        bot,
                        cs,
                        (*m).chatmessage,
                        mcontext as c_ulong,
                        &mut bestmatch,
                        vcontext as c_ulong,
                        1,
                    );
                    BotRemoveTildes((*cs).chatmessage.as_mut_ptr());
                    crate::be_interface::botimport_print(bot, PRT_MESSAGE, "chatmessage\n");
                    m = (*m).next;
                }
            } else {
                (*bestchatmessage).time = AAS_Time(bot) + CHATMESSAGE_RECENTTIME as f32;
                BotConstructChatMessage(
                    common,
                    bot,
                    cs,
                    (*bestchatmessage).chatmessage,
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

/// Raven `BotLoadSynonyms`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:578-736`
pub fn BotLoadSynonyms(bot: &mut BotLib, filename: *mut c_char) -> *mut bot_synonymlist_t {
    unsafe {
        let mut size: usize = 0;
        let mut synlist: *mut bot_synonymlist_t = core::ptr::null_mut();
        let mut ptr: *mut c_char = core::ptr::null_mut();
        for pass in 0..2 {
            if pass != 0 && size != 0 {
                ptr = GetClearedHunkMemory(bot, (size) as u64) as *mut c_char;
            }
            PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
            let source = LoadSourceFile(bot, filename);
            if source.is_null() {
                crate::be_interface::botimport_print(bot, PRT_ERROR, "counldn't load file\n");
                return core::ptr::null_mut();
            }
            let mut context: c_ulong = 0;
            let mut contextlevel: isize = 0;
            let mut contextstack = [0 as c_ulong; 32];
            synlist = core::ptr::null_mut();
            let mut lastsyn: *mut bot_synonymlist_t = core::ptr::null_mut();
            let mut token = core::mem::zeroed::<token_t>();
            while PC_ReadToken(bot, source, &mut token) != 0 {
                if token.r#type == TT_NUMBER {
                    context |= token.intvalue as c_ulong;
                    contextstack[contextlevel as usize] = token.intvalue as c_ulong;
                    contextlevel += 1;
                    if contextlevel >= 32 {
                        SourceError(bot, source, c"more than 32 context levels".as_ptr());
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    if PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0 {
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                } else if token.r#type == TT_PUNCTUATION {
                    if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                        contextlevel -= 1;
                        if contextlevel < 0 {
                            SourceError(bot, source, c"too many }".as_ptr());
                            FreeSource(bot, source);
                            return core::ptr::null_mut();
                        }
                        context &= !contextstack[contextlevel as usize];
                    } else if libc::strcmp(token.string.as_ptr(), c"[".as_ptr()) == 0 {
                        size += core::mem::size_of::<bot_synonymlist_t>();
                        let mut syn: *mut bot_synonymlist_t = core::ptr::null_mut();
                        if pass != 0 {
                            syn = ptr as *mut bot_synonymlist_t;
                            ptr = ptr.add(core::mem::size_of::<bot_synonymlist_t>());
                            (*syn).context = context;
                            (*syn).firstsynonym = core::ptr::null_mut();
                            (*syn).next = core::ptr::null_mut();
                            if !lastsyn.is_null() {
                                (*lastsyn).next = syn;
                            } else {
                                synlist = syn;
                            }
                            lastsyn = syn;
                        }
                        let mut numsynonyms = 0;
                        let mut lastsynonym: *mut bot_synonym_t = core::ptr::null_mut();
                        loop {
                            if PC_ExpectTokenString(bot, source, c"(".as_ptr() as *mut c_char) == 0
                                || PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0
                            {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            StripDoubleQuotes(token.string.as_mut_ptr());
                            if libc::strlen(token.string.as_ptr()) == 0 {
                                SourceError(bot, source, c"empty string".as_ptr());
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            size += core::mem::size_of::<bot_synonym_t>()
                                + libc::strlen(token.string.as_ptr())
                                + 1;
                            let mut synonym: *mut bot_synonym_t = core::ptr::null_mut();
                            if pass != 0 {
                                synonym = ptr as *mut bot_synonym_t;
                                ptr = ptr.add(core::mem::size_of::<bot_synonym_t>());
                                (*synonym).string = ptr;
                                ptr = ptr.add(libc::strlen(token.string.as_ptr()) + 1);
                                libc::strcpy((*synonym).string, token.string.as_ptr());
                                if !lastsynonym.is_null() {
                                    (*lastsynonym).next = synonym;
                                } else {
                                    (*syn).firstsynonym = synonym;
                                }
                                lastsynonym = synonym;
                            }
                            numsynonyms += 1;
                            if PC_ExpectTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0
                                || PC_ExpectTokenType(bot, source, TT_NUMBER, 0, &mut token) == 0
                                || PC_ExpectTokenString(bot, source, c")".as_ptr() as *mut c_char)
                                    == 0
                            {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            if pass != 0 {
                                (*synonym).weight = token.floatvalue as f32;
                                (*syn).totalweight += (*synonym).weight;
                            }
                            if PC_CheckTokenString(bot, source, c"]".as_ptr() as *mut c_char) != 0 {
                                break;
                            }
                            if PC_ExpectTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0
                            {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                        }
                        if numsynonyms < 2 {
                            SourceError(
                                bot,
                                source,
                                c"synonym must have at least two entries\n".as_ptr(),
                            );
                            FreeSource(bot, source);
                            return core::ptr::null_mut();
                        }
                    } else {
                        SourceError(bot, source, c"unexpected token".as_ptr());
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                }
            }
            FreeSource(bot, source);
            if contextlevel > 0 {
                SourceError(bot, source, c"missing }".as_ptr());
                return core::ptr::null_mut();
            }
        }
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "loaded synonyms\n");
        synlist
    }
}

/// Raven `BotLoadChatMessage`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:845-897`
pub fn BotLoadChatMessage(
    bot: &mut BotLib,
    source: *mut source_t,
    chatmessagestring: *mut c_char,
) -> c_int {
    unsafe {
        let ptr = chatmessagestring;
        *ptr = 0;
        let mut token = core::mem::zeroed::<token_t>();
        loop {
            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                return 0;
            }
            if token.r#type == TT_STRING {
                StripDoubleQuotes(token.string.as_mut_ptr());
                if libc::strlen(ptr) + libc::strlen(token.string.as_ptr()) + 1 > MAX_MESSAGE_SIZE {
                    SourceError(bot, source, c"chat message too long\n".as_ptr());
                    return 0;
                }
                libc::strcat(ptr, token.string.as_ptr());
            } else if token.r#type == TT_NUMBER && (token.subtype & TT_INTEGER) != 0 {
                if libc::strlen(ptr) + 7 > MAX_MESSAGE_SIZE {
                    SourceError(bot, source, c"chat message too long\n".as_ptr());
                    return 0;
                }
                let fmt = format!(
                    "{}v{}{}",
                    ESCAPE_CHAR as char, token.intvalue, ESCAPE_CHAR as char
                );
                let cs = std::ffi::CString::new(fmt).unwrap_or_default();
                libc::strcat(ptr, cs.as_ptr());
            } else if token.r#type == TT_NAME {
                if libc::strlen(ptr) + 7 > MAX_MESSAGE_SIZE {
                    SourceError(bot, source, c"chat message too long\n".as_ptr());
                    return 0;
                }
                let name = std::ffi::CStr::from_ptr(token.string.as_ptr()).to_string_lossy();
                let fmt = format!("{}r{}{}", ESCAPE_CHAR as char, name, ESCAPE_CHAR as char);
                let cs = std::ffi::CString::new(fmt).unwrap_or_default();
                libc::strcat(ptr, cs.as_ptr());
            } else {
                SourceError(bot, source, c"unknown message component\n".as_ptr());
                return 0;
            }
            if PC_CheckTokenString(bot, source, c";".as_ptr() as *mut c_char) != 0 {
                break;
            }
            if PC_ExpectTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0 {
                return 0;
            }
        }
        1
    }
}

/// Raven `BotLoadMatchPieces`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1122-1217`
pub fn BotLoadMatchPieces(
    bot: &mut BotLib,
    source: *mut source_t,
    endtoken: *mut c_char,
) -> *mut bot_matchpiece_t {
    unsafe {
        let mut firstpiece: *mut bot_matchpiece_t = core::ptr::null_mut();
        let mut lastpiece: *mut bot_matchpiece_t = core::ptr::null_mut();
        let mut lastwasvariable = false;
        let mut token = core::mem::zeroed::<token_t>();
        while PC_ReadToken(bot, source, &mut token) != 0 {
            if token.r#type == TT_NUMBER && (token.subtype & TT_INTEGER) != 0 {
                if token.intvalue as usize >= MAX_MATCHVARIABLES {
                    SourceError(
                        bot,
                        source,
                        c"can't have more than max match variables\n".as_ptr(),
                    );
                    FreeSource(bot, source);
                    BotFreeMatchPieces(bot, firstpiece);
                    return core::ptr::null_mut();
                }
                if lastwasvariable {
                    SourceError(
                        bot,
                        source,
                        c"not allowed to have adjacent variables\n".as_ptr(),
                    );
                    FreeSource(bot, source);
                    BotFreeMatchPieces(bot, firstpiece);
                    return core::ptr::null_mut();
                }
                lastwasvariable = true;
                let matchpiece =
                    GetClearedHunkMemory(bot, (core::mem::size_of::<bot_matchpiece_t>()) as u64)
                        as *mut bot_matchpiece_t;
                (*matchpiece).r#type = MT_VARIABLE;
                (*matchpiece).variable = token.intvalue as c_int;
                (*matchpiece).next = core::ptr::null_mut();
                if !lastpiece.is_null() {
                    (*lastpiece).next = matchpiece;
                } else {
                    firstpiece = matchpiece;
                }
                lastpiece = matchpiece;
            } else if token.r#type == TT_STRING {
                let matchpiece =
                    GetClearedHunkMemory(bot, (core::mem::size_of::<bot_matchpiece_t>()) as u64)
                        as *mut bot_matchpiece_t;
                (*matchpiece).firststring = core::ptr::null_mut();
                (*matchpiece).r#type = MT_STRING;
                (*matchpiece).variable = 0;
                (*matchpiece).next = core::ptr::null_mut();
                if !lastpiece.is_null() {
                    (*lastpiece).next = matchpiece;
                } else {
                    firstpiece = matchpiece;
                }
                lastpiece = matchpiece;
                let mut lastmatchstring: *mut bot_matchstring_t = core::ptr::null_mut();
                let mut emptystring = false;
                loop {
                    if !(*matchpiece).firststring.is_null()
                        && PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0
                    {
                        FreeSource(bot, source);
                        BotFreeMatchPieces(bot, firstpiece);
                        return core::ptr::null_mut();
                    }
                    StripDoubleQuotes(token.string.as_mut_ptr());
                    let tlen = libc::strlen(token.string.as_ptr());
                    let matchstring = GetClearedHunkMemory(
                        bot,
                        (core::mem::size_of::<bot_matchstring_t>() + tlen + 1) as u64,
                    ) as *mut bot_matchstring_t;
                    (*matchstring).string =
                        (matchstring as *mut c_char).add(core::mem::size_of::<bot_matchstring_t>());
                    libc::strcpy((*matchstring).string, token.string.as_ptr());
                    if tlen == 0 {
                        emptystring = true;
                    }
                    (*matchstring).next = core::ptr::null_mut();
                    if !lastmatchstring.is_null() {
                        (*lastmatchstring).next = matchstring;
                    } else {
                        (*matchpiece).firststring = matchstring;
                    }
                    lastmatchstring = matchstring;
                    if PC_CheckTokenString(bot, source, c"|".as_ptr() as *mut c_char) == 0 {
                        break;
                    }
                }
                if !emptystring {
                    lastwasvariable = false;
                }
            } else {
                SourceError(bot, source, c"invalid token\n".as_ptr());
                FreeSource(bot, source);
                BotFreeMatchPieces(bot, firstpiece);
                return core::ptr::null_mut();
            }
            if PC_CheckTokenString(bot, source, endtoken) != 0 {
                break;
            }
            if PC_ExpectTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0 {
                FreeSource(bot, source);
                BotFreeMatchPieces(bot, firstpiece);
                return core::ptr::null_mut();
            }
        }
        firstpiece
    }
}

/// Raven `BotLoadRandomStrings`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:929-1024`
pub fn BotLoadRandomStrings(bot: &mut BotLib, filename: *mut c_char) -> *mut bot_randomlist_t {
    unsafe {
        let mut size: usize = 0;
        let mut ptr: *mut c_char = core::ptr::null_mut();
        let mut randomlist: *mut bot_randomlist_t = core::ptr::null_mut();
        for pass in 0..2 {
            if pass != 0 && size != 0 {
                ptr = GetClearedHunkMemory(bot, (size) as u64) as *mut c_char;
            }
            PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
            let source = LoadSourceFile(bot, filename);
            if source.is_null() {
                crate::be_interface::botimport_print(bot, PRT_ERROR, "counldn't load file\n");
                return core::ptr::null_mut();
            }
            randomlist = core::ptr::null_mut();
            let mut lastrandom: *mut bot_randomlist_t = core::ptr::null_mut();
            let mut token = core::mem::zeroed::<token_t>();
            let mut chatmessagestring = [0 as c_char; MAX_MESSAGE_SIZE];
            while PC_ReadToken(bot, source, &mut token) != 0 {
                if token.r#type != TT_NAME {
                    SourceError(bot, source, c"unknown random".as_ptr());
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                size += core::mem::size_of::<bot_randomlist_t>()
                    + libc::strlen(token.string.as_ptr())
                    + 1;
                let mut random: *mut bot_randomlist_t = core::ptr::null_mut();
                if pass != 0 {
                    random = ptr as *mut bot_randomlist_t;
                    ptr = ptr.add(core::mem::size_of::<bot_randomlist_t>());
                    (*random).string = ptr;
                    ptr = ptr.add(libc::strlen(token.string.as_ptr()) + 1);
                    libc::strcpy((*random).string, token.string.as_ptr());
                    (*random).firstrandomstring = core::ptr::null_mut();
                    (*random).numstrings = 0;
                    if !lastrandom.is_null() {
                        (*lastrandom).next = random;
                    } else {
                        randomlist = random;
                    }
                    lastrandom = random;
                }
                if PC_ExpectTokenString(bot, source, c"=".as_ptr() as *mut c_char) == 0
                    || PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0
                {
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                while PC_CheckTokenString(bot, source, c"}".as_ptr() as *mut c_char) == 0 {
                    if BotLoadChatMessage(bot, source, chatmessagestring.as_mut_ptr()) == 0 {
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    size += core::mem::size_of::<bot_randomstring_t>()
                        + libc::strlen(chatmessagestring.as_ptr())
                        + 1;
                    if pass != 0 {
                        let randomstring = ptr as *mut bot_randomstring_t;
                        ptr = ptr.add(core::mem::size_of::<bot_randomstring_t>());
                        (*randomstring).string = ptr;
                        ptr = ptr.add(libc::strlen(chatmessagestring.as_ptr()) + 1);
                        libc::strcpy((*randomstring).string, chatmessagestring.as_ptr());
                        (*random).numstrings += 1;
                        (*randomstring).next = (*random).firstrandomstring;
                        (*random).firstrandomstring = randomstring;
                    }
                }
            }
            FreeSource(bot, source);
        }
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "loaded random strings\n");
        randomlist
    }
}

/// Raven `BotLoadMatchTemplates`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1241-1333`
pub fn BotLoadMatchTemplates(bot: &mut BotLib, matchfile: *mut c_char) -> *mut bot_matchtemplate_t {
    unsafe {
        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
        let source = LoadSourceFile(bot, matchfile);
        if source.is_null() {
            crate::be_interface::botimport_print(bot, PRT_ERROR, "counldn't load file\n");
            return core::ptr::null_mut();
        }
        let mut matches: *mut bot_matchtemplate_t = core::ptr::null_mut();
        let mut lastmatch: *mut bot_matchtemplate_t = core::ptr::null_mut();
        let mut token = core::mem::zeroed::<token_t>();
        while PC_ReadToken(bot, source, &mut token) != 0 {
            if token.r#type != TT_NUMBER || (token.subtype & TT_INTEGER) == 0 {
                SourceError(bot, source, c"expected integer\n".as_ptr());
                BotFreeMatchTemplates(bot, matches);
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
            let context = token.intvalue as c_ulong;
            if PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0 {
                BotFreeMatchTemplates(bot, matches);
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
            while PC_ReadToken(bot, source, &mut token) != 0 {
                if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                    break;
                }
                PC_UnreadLastToken(bot, source);
                let matchtemplate =
                    GetClearedHunkMemory(bot, (core::mem::size_of::<bot_matchtemplate_t>()) as u64)
                        as *mut bot_matchtemplate_t;
                (*matchtemplate).context = context;
                (*matchtemplate).next = core::ptr::null_mut();
                if !lastmatch.is_null() {
                    (*lastmatch).next = matchtemplate;
                } else {
                    matches = matchtemplate;
                }
                lastmatch = matchtemplate;
                (*matchtemplate).first =
                    BotLoadMatchPieces(bot, source, c"=".as_ptr() as *mut c_char);
                if (*matchtemplate).first.is_null() {
                    BotFreeMatchTemplates(bot, matches);
                    return core::ptr::null_mut();
                }
                if PC_ExpectTokenString(bot, source, c"(".as_ptr() as *mut c_char) == 0
                    || PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0
                {
                    BotFreeMatchTemplates(bot, matches);
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                (*matchtemplate).r#type = token.intvalue as i32;
                if PC_ExpectTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0
                    || PC_ExpectTokenType(bot, source, TT_NUMBER, TT_INTEGER, &mut token) == 0
                {
                    BotFreeMatchTemplates(bot, matches);
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                (*matchtemplate).subtype = token.intvalue as i32;
                if PC_ExpectTokenString(bot, source, c")".as_ptr() as *mut c_char) == 0
                    || PC_ExpectTokenString(bot, source, c";".as_ptr() as *mut c_char) == 0
                {
                    BotFreeMatchTemplates(bot, matches);
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
            }
        }
        FreeSource(bot, source);
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "loaded match templates\n");
        matches
    }
}

/// Raven `BotLoadReplyChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1811-1964`
pub fn BotLoadReplyChat(
    common: &mut Common,
    bot: &mut BotLib,
    filename: *mut c_char,
) -> *mut bot_replychat_t {
    unsafe {
        let mut chatmessagestring = [0 as c_char; MAX_MESSAGE_SIZE];
        let mut namebuffer = [0 as c_char; MAX_MESSAGE_SIZE];
        PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
        let source = LoadSourceFile(bot, filename);
        if source.is_null() {
            crate::be_interface::botimport_print(bot, PRT_ERROR, "counldn't load file\n");
            return core::ptr::null_mut();
        }
        let mut replychatlist: *mut bot_replychat_t = core::ptr::null_mut();
        let mut token = core::mem::zeroed::<token_t>();
        while PC_ReadToken(bot, source, &mut token) != 0 {
            if libc::strcmp(token.string.as_ptr(), c"[".as_ptr()) != 0 {
                SourceError(bot, source, c"expected [".as_ptr());
                BotFreeReplyChat(bot, replychatlist);
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
            let replychat =
                GetClearedHunkMemory(bot, (core::mem::size_of::<bot_replychat_t>()) as u64)
                    as *mut bot_replychat_t;
            (*replychat).keys = core::ptr::null_mut();
            (*replychat).next = replychatlist;
            replychatlist = replychat;
            loop {
                let key =
                    GetClearedHunkMemory(bot, (core::mem::size_of::<bot_replychatkey_t>()) as u64)
                        as *mut bot_replychatkey_t;
                (*key).flags = 0;
                (*key).string = core::ptr::null_mut();
                (*key).r#match = core::ptr::null_mut();
                (*key).next = (*replychat).keys;
                (*replychat).keys = key;
                if PC_CheckTokenString(bot, source, c"&".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_AND;
                } else if PC_CheckTokenString(bot, source, c"!".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_NOT;
                }
                if PC_CheckTokenString(bot, source, c"name".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_NAME;
                } else if PC_CheckTokenString(bot, source, c"female".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_GENDERFEMALE;
                } else if PC_CheckTokenString(bot, source, c"male".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_GENDERMALE;
                } else if PC_CheckTokenString(bot, source, c"it".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_GENDERLESS;
                } else if PC_CheckTokenString(bot, source, c"(".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_VARIABLES;
                    (*key).r#match = BotLoadMatchPieces(bot, source, c")".as_ptr() as *mut c_char);
                    if (*key).r#match.is_null() {
                        BotFreeReplyChat(bot, replychatlist);
                        return core::ptr::null_mut();
                    }
                } else if PC_CheckTokenString(bot, source, c"<".as_ptr() as *mut c_char) != 0 {
                    (*key).flags |= RCKFL_BOTNAMES;
                    libc::strcpy(namebuffer.as_mut_ptr(), c"".as_ptr());
                    loop {
                        if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0 {
                            BotFreeReplyChat(bot, replychatlist);
                            FreeSource(bot, source);
                            return core::ptr::null_mut();
                        }
                        StripDoubleQuotes(token.string.as_mut_ptr());
                        if libc::strlen(namebuffer.as_ptr()) != 0 {
                            libc::strcat(namebuffer.as_mut_ptr(), c"\\".as_ptr());
                        }
                        libc::strcat(namebuffer.as_mut_ptr(), token.string.as_ptr());
                        if PC_CheckTokenString(bot, source, c",".as_ptr() as *mut c_char) == 0 {
                            break;
                        }
                    }
                    if PC_ExpectTokenString(bot, source, c">".as_ptr() as *mut c_char) == 0 {
                        BotFreeReplyChat(bot, replychatlist);
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    let nlen = libc::strlen(namebuffer.as_ptr());
                    (*key).string = GetClearedHunkMemory(bot, (nlen + 1) as u64) as *mut c_char;
                    libc::strcpy((*key).string, namebuffer.as_ptr());
                } else {
                    (*key).flags |= RCKFL_STRING;
                    if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0 {
                        BotFreeReplyChat(bot, replychatlist);
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    StripDoubleQuotes(token.string.as_mut_ptr());
                    let tlen = libc::strlen(token.string.as_ptr());
                    (*key).string = GetClearedHunkMemory(bot, (tlen + 1) as u64) as *mut c_char;
                    libc::strcpy((*key).string, token.string.as_ptr());
                }
                PC_CheckTokenString(bot, source, c",".as_ptr() as *mut c_char);
                if PC_CheckTokenString(bot, source, c"]".as_ptr() as *mut c_char) != 0 {
                    break;
                }
            }
            BotCheckValidReplyChatKeySet(bot, source, (*replychat).keys);
            if PC_ExpectTokenString(bot, source, c"=".as_ptr() as *mut c_char) == 0
                || PC_ExpectTokenType(bot, source, TT_NUMBER, 0, &mut token) == 0
            {
                BotFreeReplyChat(bot, replychatlist);
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
            (*replychat).priority = token.floatvalue as f32;
            if PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0 {
                BotFreeReplyChat(bot, replychatlist);
                FreeSource(bot, source);
                return core::ptr::null_mut();
            }
            (*replychat).numchatmessages = 0;
            while PC_CheckTokenString(bot, source, c"}".as_ptr() as *mut c_char) == 0 {
                if BotLoadChatMessage(bot, source, chatmessagestring.as_mut_ptr()) == 0 {
                    BotFreeReplyChat(bot, replychatlist);
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
                let clen = libc::strlen(chatmessagestring.as_ptr());
                let chatmessage = GetClearedHunkMemory(
                    bot,
                    (core::mem::size_of::<bot_chatmessage_t>() + clen + 1) as u64,
                ) as *mut bot_chatmessage_t;
                (*chatmessage).chatmessage =
                    (chatmessage as *mut c_char).add(core::mem::size_of::<bot_chatmessage_t>());
                libc::strcpy((*chatmessage).chatmessage, chatmessagestring.as_ptr());
                (*chatmessage).time = -2.0 * CHATMESSAGE_RECENTTIME as f32;
                (*chatmessage).next = (*replychat).firstchatmessage;
                (*replychat).firstchatmessage = chatmessage;
                (*replychat).numchatmessages += 1;
            }
        }
        FreeSource(bot, source);
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "loaded reply chat\n");
        if bot.bot_developer != 0 {
            BotCheckReplyChatIntegrety(common, bot, replychatlist);
        }
        if replychatlist.is_null() {
            crate::be_interface::botimport_print(bot, PRT_MESSAGE, "no rchats\n");
        }
        replychatlist
    }
}

/// Raven `BotLoadInitialChat`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:1996-2161`
pub fn BotLoadInitialChat(
    common: &mut Common,
    bot: &mut BotLib,
    chatfile: *mut c_char,
    chatname: *mut c_char,
) -> *mut bot_chat_t {
    unsafe {
        let mut size: usize;
        let mut ptr: *mut c_char = core::ptr::null_mut();
        let mut chat: *mut bot_chat_t = core::ptr::null_mut();
        let mut foundchat = false;
        let mut chatmessagestring = [0 as c_char; MAX_MESSAGE_SIZE];
        size = 0;
        for pass in 0..2 {
            if pass != 0 && size != 0 {
                ptr = GetClearedMemory(bot, (size) as u64) as *mut c_char;
            }
            PC_SetBaseFolder(bot, BOTFILESBASEFOLDER.as_ptr() as *mut c_char);
            let source = LoadSourceFile(bot, chatfile);
            if source.is_null() {
                crate::be_interface::botimport_print(bot, PRT_ERROR, "counldn't load file\n");
                return core::ptr::null_mut();
            }
            if pass != 0 {
                chat = ptr as *mut bot_chat_t;
                ptr = ptr.add(core::mem::size_of::<bot_chat_t>());
            }
            size = core::mem::size_of::<bot_chat_t>();
            let mut token = core::mem::zeroed::<token_t>();
            while PC_ReadToken(bot, source, &mut token) != 0 {
                if libc::strcmp(token.string.as_ptr(), c"chat".as_ptr()) == 0 {
                    if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0 {
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    StripDoubleQuotes(token.string.as_mut_ptr());
                    if PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char) == 0 {
                        FreeSource(bot, source);
                        return core::ptr::null_mut();
                    }
                    if libc::strcasecmp(token.string.as_ptr(), chatname) == 0 {
                        foundchat = true;
                        loop {
                            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                                break;
                            }
                            if libc::strcmp(token.string.as_ptr(), c"type".as_ptr()) != 0 {
                                SourceError(bot, source, c"expected type\n".as_ptr());
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            if PC_ExpectTokenType(bot, source, TT_STRING, 0, &mut token) == 0
                                || PC_ExpectTokenString(bot, source, c"{".as_ptr() as *mut c_char)
                                    == 0
                            {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            StripDoubleQuotes(token.string.as_mut_ptr());
                            let mut chattype: *mut bot_chattype_t = core::ptr::null_mut();
                            if pass != 0 {
                                chattype = ptr as *mut bot_chattype_t;
                                libc::strncpy(
                                    (*chattype).name.as_mut_ptr(),
                                    token.string.as_ptr(),
                                    MAX_CHATTYPE_NAME as usize,
                                );
                                (*chattype).firstchatmessage = core::ptr::null_mut();
                                (*chattype).next = (*chat).types;
                                (*chat).types = chattype;
                                ptr = ptr.add(core::mem::size_of::<bot_chattype_t>());
                            }
                            size += core::mem::size_of::<bot_chattype_t>();
                            while PC_CheckTokenString(bot, source, c"}".as_ptr() as *mut c_char)
                                == 0
                            {
                                if BotLoadChatMessage(bot, source, chatmessagestring.as_mut_ptr())
                                    == 0
                                {
                                    FreeSource(bot, source);
                                    return core::ptr::null_mut();
                                }
                                if pass != 0 {
                                    let chatmessage = ptr as *mut bot_chatmessage_t;
                                    (*chatmessage).time = -2.0 * CHATMESSAGE_RECENTTIME as f32;
                                    (*chatmessage).next = (*chattype).firstchatmessage;
                                    (*chattype).firstchatmessage = chatmessage;
                                    ptr = ptr.add(core::mem::size_of::<bot_chatmessage_t>());
                                    (*chatmessage).chatmessage = ptr;
                                    libc::strcpy(
                                        (*chatmessage).chatmessage,
                                        chatmessagestring.as_ptr(),
                                    );
                                    ptr = ptr.add(libc::strlen(chatmessagestring.as_ptr()) + 1);
                                    (*chattype).numchatmessages += 1;
                                }
                                size += core::mem::size_of::<bot_chatmessage_t>()
                                    + libc::strlen(chatmessagestring.as_ptr())
                                    + 1;
                            }
                        }
                    } else {
                        let mut indent = 1;
                        while indent != 0 {
                            if PC_ExpectAnyToken(bot, source, &mut token) == 0 {
                                FreeSource(bot, source);
                                return core::ptr::null_mut();
                            }
                            if libc::strcmp(token.string.as_ptr(), c"{".as_ptr()) == 0 {
                                indent += 1;
                            } else if libc::strcmp(token.string.as_ptr(), c"}".as_ptr()) == 0 {
                                indent -= 1;
                            }
                        }
                    }
                } else {
                    SourceError(bot, source, c"unknown definition\n".as_ptr());
                    FreeSource(bot, source);
                    return core::ptr::null_mut();
                }
            }
            FreeSource(bot, source);
            if !foundchat {
                crate::be_interface::botimport_print(bot, PRT_ERROR, "couldn't find chat\n");
                return core::ptr::null_mut();
            }
        }
        crate::be_interface::botimport_print(bot, PRT_MESSAGE, "loaded chat\n");
        if bot.bot_developer != 0 {
            BotCheckInitialChatIntegrety(common, bot, chat);
        }
        chat
    }
}

/// Raven `BotLoadChatFile`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2183-2234`
pub fn BotLoadChatFile(
    common: &mut Common,
    bot: &mut BotLib,
    chatstate: c_int,
    chatfile: *mut c_char,
    chatname: *mut c_char,
) -> c_int {
    let cs = BotChatStateFromHandle(bot, chatstate);
    if cs.is_null() {
        return BLERR_CANNOTLOADICHAT;
    }
    BotFreeChatFile(bot, chatstate);
    unsafe {
        let mut avail: isize = 0;
        if LibVarGetValue(bot, c"bot_reloadcharacters".as_ptr() as *mut c_char) == 0.0 {
            avail = -1;
            for n in 0..MAX_CLIENTS {
                if bot.ichatdata[n].is_null() {
                    if avail == -1 {
                        avail = n as isize;
                    }
                    continue;
                }
                if libc::strcmp(chatfile, (*bot.ichatdata[n]).filename.as_ptr()) != 0 {
                    continue;
                }
                if libc::strcmp(chatname, (*bot.ichatdata[n]).chatname.as_ptr()) != 0 {
                    continue;
                }
                (*cs).chat = (*bot.ichatdata[n]).chat;
                return BLERR_NOERROR;
            }
            if avail == -1 {
                crate::be_interface::botimport_print(bot, PRT_FATAL, "ichatdata table full\n");
                return BLERR_CANNOTLOADICHAT;
            }
        }
        (*cs).chat = BotLoadInitialChat(common, bot, chatfile, chatname);
        if (*cs).chat.is_null() {
            crate::be_interface::botimport_print(bot, PRT_FATAL, "couldn't load chat\n");
            return BLERR_CANNOTLOADICHAT;
        }
        if LibVarGetValue(bot, c"bot_reloadcharacters".as_ptr() as *mut c_char) == 0.0 {
            // PORT-NOTE(bot_ichatdata_t): not in this shard's TYPE ROSETTA (no
            // rosetta row yet); referenced at the plausible sibling-type path
            // per naming convention. Reported in missing_symbols.
            bot.ichatdata[avail as usize] = GetClearedMemory(
                bot,
                core::mem::size_of::<crate::be_ai_chat::bot_ichatdata_s::bot_ichatdata_t>() as u64,
            )
                as *mut crate::be_ai_chat::bot_ichatdata_s::bot_ichatdata_t;
            (*bot.ichatdata[avail as usize]).chat = (*cs).chat;
            libc::strncpy(
                (*bot.ichatdata[avail as usize]).chatname.as_mut_ptr(),
                chatname,
                core::mem::size_of_val(&(*bot.ichatdata[avail as usize]).chatname),
            );
            libc::strncpy(
                (*bot.ichatdata[avail as usize]).filename.as_mut_ptr(),
                chatfile,
                core::mem::size_of_val(&(*bot.ichatdata[avail as usize]).filename),
            );
        }
    }
    BLERR_NOERROR
}

/// Raven `BotSetupChatAI`.
///
/// Source: `oracle/codemp/botlib/be_ai_chat.cpp:2934-2961`
pub fn BotSetupChatAI(common: &mut Common, bot: &mut BotLib) -> c_int {
    let file = LibVarString(
        bot,
        c"synfile".as_ptr() as *mut c_char,
        c"syn.c".as_ptr() as *mut c_char,
    );
    bot.synonyms = BotLoadSynonyms(bot, file);
    let file = LibVarString(
        bot,
        c"rndfile".as_ptr() as *mut c_char,
        c"rnd.c".as_ptr() as *mut c_char,
    );
    bot.randomstrings = BotLoadRandomStrings(bot, file);
    let file = LibVarString(
        bot,
        c"matchfile".as_ptr() as *mut c_char,
        c"match.c".as_ptr() as *mut c_char,
    );
    bot.matchtemplates = BotLoadMatchTemplates(bot, file);
    if LibVarValue(
        bot,
        c"nochat".as_ptr() as *mut c_char,
        c"0".as_ptr() as *mut c_char,
    ) == 0.0
    {
        let file = LibVarString(
            bot,
            c"rchatfile".as_ptr() as *mut c_char,
            c"rchat.c".as_ptr() as *mut c_char,
        );
        bot.replychats = BotLoadReplyChat(common, bot, file);
    }
    InitConsoleMessageHeap(bot);
    BLERR_NOERROR
}
