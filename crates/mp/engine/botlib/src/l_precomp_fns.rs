#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]
//! `l_precomp.cpp` — the botlib preprocessor (function bodies).
//!
//! One Rust module per oracle source file (`l_precomp.cpp`); the stem collides
//! with the `l_precomp/` type directory, so this lands as `l_precomp_fns.rs`.
//!
//! Source: `oracle/codemp/botlib/l_precomp.cpp`
//!
//! `DEFINEHASHING` (`l_precomp.cpp:83`) and `BOTLIB` are compile-time-defined
//! in this build; `MEQCC`/`BSPC`/`QUAKE`/`QUAKEC`/`SCREWUP`/`NUMBERVALUE`/
//! `DEBUG_EVAL` are not — the corresponding dead `#if`/`#else` arms are
//! dropped per §C10.

use core::ffi::{c_char, c_int, c_long, c_ulong};

use libc::{abs, sprintf, strcat, strcmp, strcpy, strlen, strncat, strncpy, time};

/// `ctime(&t)` in asctime layout ("Www Mmm dd hh:mm:ss yyyy\n", 26 bytes incl.
/// NUL) from `localtime` fields — libc's linux bindings omit `ctime`, and the
/// oracle's `free(curtime)` on ctime's static buffer is dropped (§19: freeing
/// a non-heap pointer; the port's buffer is a stack local).
fn ctime_buf(t: libc::time_t) -> [c_char; 26] {
    const WDAY: [&[u8; 3]; 7] = [b"Sun", b"Mon", b"Tue", b"Wed", b"Thu", b"Fri", b"Sat"];
    const MON: [&[u8; 3]; 12] = [
        b"Jan", b"Feb", b"Mar", b"Apr", b"May", b"Jun", b"Jul", b"Aug", b"Sep", b"Oct", b"Nov",
        b"Dec",
    ];
    let mut out = [0 as c_char; 26];
    let tm = unsafe { libc::localtime(&t) };
    if tm.is_null() {
        return out;
    }
    let tm = unsafe { &*tm };
    let text = format!(
        "{} {} {:2} {:02}:{:02}:{:02} {}\n",
        core::str::from_utf8(WDAY[tm.tm_wday.rem_euclid(7) as usize]).unwrap(),
        core::str::from_utf8(MON[tm.tm_mon.rem_euclid(12) as usize]).unwrap(),
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec,
        1900 + tm.tm_year
    );
    for (i, b) in text.bytes().take(25).enumerate() {
        out[i] = b as c_char;
    }
    out
}

use crate::l_log_fns::Log_Write;
use crate::l_memory_fns::{FreeMemory, GetClearedMemory, GetMemory};
use crate::l_script_fns::{
    EndOfScript, FreeScript, LoadScriptFile, LoadScriptMemory, PS_ReadToken, PS_SetBaseFolder,
    StripDoubleQuotes,
};
use mp_engine_qcommon::common_fns::{Com_Memcpy, Com_Memset};

use crate::l_precomp::builtin_defines::{
    BUILTIN_DATE, BUILTIN_FILE, BUILTIN_LINE, BUILTIN_STDC, BUILTIN_TIME,
};
use crate::l_precomp::define_flags::{DEFINE_FIXED, DEFINE_GLOBAL};
use crate::l_precomp::define_s::define_t;
use crate::l_precomp::directive_s::directive_t;
use crate::l_precomp::indent_s::indent_t;
use crate::l_precomp::indent_type::{
    INDENT_ELIF, INDENT_ELSE, INDENT_IF, INDENT_IFDEF, INDENT_IFNDEF,
};
use crate::l_precomp::operator_s::operator_t;
use crate::l_precomp::path_seperator_consts::{PATHSEPERATOR_CHAR, PATHSEPERATOR_STR};
use crate::l_precomp::precomp_consts::{
    DEFINEHASHSIZE, MAX_DEFINEPARMS, MAX_OPERATORS, MAX_PATH, MAX_SOURCEFILES, MAX_VALUES,
};
use crate::l_precomp::source_s::source_t;
use crate::l_precomp::value_s::value_t;
use crate::l_script::consts::{
    MAX_TOKEN, P_ADD, P_BIN_AND, P_BIN_NOT, P_BIN_OR, P_BIN_XOR, P_COLON, P_DEC, P_DIV, P_INC,
    P_LOGIC_AND, P_LOGIC_EQ, P_LOGIC_GEQ, P_LOGIC_GREATER, P_LOGIC_LEQ, P_LOGIC_LESS, P_LOGIC_NOT,
    P_LOGIC_OR, P_LOGIC_UNEQ, P_LSHIFT, P_MOD, P_MUL, P_PARENTHESESCLOSE, P_PARENTHESESOPEN,
    P_QUESTIONMARK, P_RSHIFT, P_SUB, TT_BINARY, TT_DECIMAL, TT_FLOAT, TT_HEX, TT_INTEGER,
    TT_LITERAL, TT_LONG, TT_NAME, TT_NUMBER, TT_OCTAL, TT_PUNCTUATION, TT_STRING, TT_UNSIGNED,
};
use crate::l_script::punctuation_s::punctuation_t;
use crate::l_script::script_s::script_t;
use crate::l_script::token_s::token_t;
use crate::BotLib;
use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_WARNING};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::pc_token_t;
use mp_qshared::shared::{qfalse, qtrue};

use mp_qshared::shared::q_string::Q_stricmp;

/// Raven `SourceError` — print a preprocessor error tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:117-134`
pub fn SourceError(bot: &mut BotLib, source: *mut source_t, text: *const c_char) {
    unsafe {
        // #ifdef BOTLIB (defined)
        (bot.botimport.Print.unwrap())(
            PRT_ERROR,
            c"file %s, line %d: %s\n".as_ptr() as *mut c_char,
            (*(*source).scriptstack).filename.as_ptr(),
            (*(*source).scriptstack).line,
            text,
        );
    }
}

/// Raven `SourceWarning` — print a preprocessor warning tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:141-158`
pub fn SourceWarning(bot: &mut BotLib, source: *mut source_t, text: *const c_char) {
    unsafe {
        // #ifdef BOTLIB (defined)
        (bot.botimport.Print.unwrap())(
            PRT_WARNING,
            c"file %s, line %d: %s\n".as_ptr() as *mut c_char,
            (*(*source).scriptstack).filename.as_ptr(),
            (*(*source).scriptstack).line,
            text,
        );
    }
}

// Raven's `va_start`/`vsprintf`/`va_end` C-variadic seam has no stable-Rust
// equivalent; this macro reproduces the `vsprintf(text, str, ap)` step at
// each `SourceError`/`SourceWarning` call site, then forwards the buffer.
macro_rules! source_error {
    ($bot:expr, $source:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __se_text = [0 as ::core::ffi::c_char; 1024];
        ::libc::sprintf(__se_text.as_mut_ptr(), $fmt $(, $arg)*);
        $crate::l_precomp_fns::SourceError($bot, $source, __se_text.as_ptr())
    }};
}
macro_rules! source_warning {
    ($bot:expr, $source:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {{
        let mut __sw_text = [0 as ::core::ffi::c_char; 1024];
        ::libc::sprintf(__sw_text.as_mut_ptr(), $fmt $(, $arg)*);
        $crate::l_precomp_fns::SourceWarning($bot, $source, __sw_text.as_ptr())
    }};
}
pub(crate) use source_error;

/// Raven `PC_InitTokenHeap` — the static token heap is entirely commented out in
/// Raven; the body is a no-op.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:230-244`
pub fn PC_InitTokenHeap() {
    // Raven's body is fully commented out (the `TOKEN_HEAP_SIZE` freelist is
    // dead); nothing to do.
}

/// Raven `PC_PushIndent` — push an `#if`/`#ifdef` indent onto the source's
/// indent stack.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:165-176`
pub fn PC_PushIndent(bot: &mut BotLib, source: *mut source_t, r#type: c_int, skip: c_int) {
    unsafe {
        let indent: *mut indent_t =
            GetMemory(bot, core::mem::size_of::<indent_t>() as c_ulong) as *mut indent_t;
        (*indent).r#type = r#type;
        (*indent).script = (*source).scriptstack;
        (*indent).skip = (skip != 0) as c_int;
        (*source).skip += (*indent).skip;
        (*indent).next = (*source).indentstack;
        (*source).indentstack = indent;
    }
}

/// Raven `PC_PopIndent` — pop the top indent of the current script, reporting
/// its type and skip flag.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:183-201`
pub fn PC_PopIndent(bot: &mut BotLib, source: *mut source_t, r#type: *mut c_int, skip: *mut c_int) {
    unsafe {
        *r#type = 0;
        *skip = 0;

        let indent: *mut indent_t = (*source).indentstack;
        if indent.is_null() {
            return;
        }

        // must be an indent from the current script
        if (*(*source).indentstack).script != (*source).scriptstack {
            return;
        }

        *r#type = (*indent).r#type;
        *skip = (*indent).skip;
        (*source).indentstack = (*(*source).indentstack).next;
        (*source).skip -= (*indent).skip;
        FreeMemory(bot, indent as *mut _);
    }
}

/// Raven `PC_PushScript` — push a script onto the source's script stack,
/// erroring on recursive inclusion.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:208-223`
pub fn PC_PushScript(bot: &mut BotLib, source: *mut source_t, script: *mut script_t) {
    unsafe {
        let mut s: *mut script_t = (*source).scriptstack;
        while !s.is_null() {
            if Q_stricmp((*s).filename.as_ptr(), (*script).filename.as_ptr()) == 0 {
                source_error!(
                    bot,
                    source,
                    c"%s recursively included".as_ptr() as *mut c_char,
                    (*script).filename.as_ptr(),
                );
                return;
            }
            s = (*s).next;
        }
        // push the script on the script stack
        (*script).next = (*source).scriptstack;
        (*source).scriptstack = script;
    }
}

/// Raven `PC_FreeToken` — free a token and decrement the live-token count.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:279-286`
pub fn PC_FreeToken(bot: &mut BotLib, token: *mut token_t) {
    FreeMemory(bot, token as *mut _);
    bot.numtokens -= 1;
}

/// Raven `PC_CopyToken` — allocate a duplicate of a token, aborting on OOM.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:251-272`
pub fn PC_CopyToken(bot: &mut BotLib, token: *mut token_t) -> *mut token_t {
    unsafe {
        let t: *mut token_t =
            GetMemory(bot, core::mem::size_of::<token_t>() as c_ulong) as *mut token_t;
        if t.is_null() {
            // #ifdef BSPC not defined -> Com_Error branch (ruling 1: a longjmp/panic)
            let _ = errorParm_t::ERR_FATAL;
            panic!("out of token space");
        }
        Com_Memcpy(t.cast(), token.cast(), core::mem::size_of::<token_t>());
        (*t).next = core::ptr::null_mut();
        bot.numtokens += 1;
        t
    }
}

/// Raven `PC_StringizeTokens` — build a `"..."` string token from a token list.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:450-465`
pub fn PC_StringizeTokens(tokens: *mut token_t, token: *mut token_t) -> c_int {
    unsafe {
        (*token).r#type = TT_STRING;
        (*token).whitespace_p = core::ptr::null_mut();
        (*token).endwhitespace_p = core::ptr::null_mut();
        (*token).string[0] = b'\0' as c_char;
        strcat((*token).string.as_mut_ptr(), c"\"".as_ptr());
        let mut t: *mut token_t = tokens;
        while !t.is_null() {
            strncat(
                (*token).string.as_mut_ptr(),
                (*t).string.as_ptr(),
                MAX_TOKEN - strlen((*token).string.as_ptr()),
            );
            t = (*t).next;
        }
        strncat(
            (*token).string.as_mut_ptr(),
            c"\"".as_ptr(),
            MAX_TOKEN - strlen((*token).string.as_ptr()),
        );
        qtrue
    }
}

/// Raven `PC_MergeTokens` — merge `t2` into `t1` for name/number/string pairs.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:472-491`
pub fn PC_MergeTokens(t1: *mut token_t, t2: *mut token_t) -> c_int {
    unsafe {
        // merging of a name with a name or number
        if (*t1).r#type == TT_NAME && ((*t2).r#type == TT_NAME || (*t2).r#type == TT_NUMBER) {
            strcat((*t1).string.as_mut_ptr(), (*t2).string.as_ptr());
            return qtrue;
        }
        // merging of two strings
        if (*t1).r#type == TT_STRING && (*t2).r#type == TT_STRING {
            // remove trailing double quote
            let end = strlen((*t1).string.as_ptr()) - 1;
            (*t1).string[end] = b'\0' as c_char;
            // concat without leading double quote
            strcat((*t1).string.as_mut_ptr(), (*t2).string.as_ptr().add(1));
            return qtrue;
        }
        // Raven note: merging of two numbers of the same sub type is unhandled.
        qfalse
    }
}

/// Raven `PC_NameHash` — hash a define name into the define hash-chain table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:539-552`
pub fn PC_NameHash(name: *mut c_char) -> c_int {
    unsafe {
        let mut hash: c_int = 0;
        let mut i: c_int = 0;
        while *name.add(i as usize) != b'\0' as c_char {
            hash += (*name.add(i as usize) as c_int) * (119 + i);
            i += 1;
        }
        hash = (hash ^ (hash >> 10) ^ (hash >> 20)) & (DEFINEHASHSIZE as c_int - 1);
        hash
    }
}

/// Raven `PC_FindDefine` — linear-scan a define list by name.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:604-613`
pub fn PC_FindDefine(defines: *mut define_t, name: *mut c_char) -> *mut define_t {
    unsafe {
        let mut d: *mut define_t = defines;
        while !d.is_null() {
            if strcmp((*d).name, name) == 0 {
                return d;
            }
            d = (*d).next;
        }
        core::ptr::null_mut()
    }
}

/// Raven `PC_FindDefineParm` — index of a define parameter by name, or -1.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:621-633`
pub fn PC_FindDefineParm(define: *mut define_t, name: *mut c_char) -> c_int {
    unsafe {
        let mut i: c_int = 0;
        let mut p: *mut token_t = (*define).parms;
        while !p.is_null() {
            if strcmp((*p).string.as_ptr(), name) == 0 {
                return i;
            }
            i += 1;
            p = (*p).next;
        }
        -1
    }
}

/// Raven `PC_AddDefineToHash` — link a define into a hash-chain table (or into
/// the global table when collecting global defines).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:559-578`
pub fn PC_AddDefineToHash(bot: &mut BotLib, define: *mut define_t, definehash: *mut *mut define_t) {
    unsafe {
        let mut definehash = definehash;

        if bot.addGlobalDefine == qtrue {
            definehash = bot.globaldefines;
            (*define).flags |= DEFINE_GLOBAL;
        }

        let hash = PC_NameHash((*define).name);
        (*define).hashnext = *definehash.add(hash as usize);
        *definehash.add(hash as usize) = define;

        if bot.addGlobalDefine == qtrue {
            (*define).globalnext = (*define).hashnext;
        }
    }
}

/// Raven `PC_FindHashedDefine` — look up a define by name in a hash-chain table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:585-596`
pub fn PC_FindHashedDefine(definehash: *mut *mut define_t, name: *mut c_char) -> *mut define_t {
    unsafe {
        let hash = PC_NameHash(name);
        let mut d: *mut define_t = *definehash.add(hash as usize);
        while !d.is_null() {
            if strcmp((*d).name, name) == 0 {
                return d;
            }
            d = (*d).hashnext;
        }
        core::ptr::null_mut()
    }
}

/// Raven `PC_PrintDefineHashTable` — dump the define hash table to the log.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:516-530`
pub fn PC_PrintDefineHashTable(bot: &mut BotLib, definehash: *mut *mut define_t) {
    unsafe {
        for i in 0..DEFINEHASHSIZE {
            let mut buf = [0 as c_char; 64];
            sprintf(
                buf.as_mut_ptr(),
                c"%4d:".as_ptr() as *mut c_char,
                i as c_int,
            );
            Log_Write(bot, buf.as_mut_ptr());
            let mut d: *mut define_t = *definehash.add(i);
            while !d.is_null() {
                let mut buf = [0 as c_char; 128];
                sprintf(buf.as_mut_ptr(), c" %s".as_ptr() as *mut c_char, (*d).name);
                Log_Write(bot, buf.as_mut_ptr());
                d = (*d).hashnext;
            }
            Log_Write(bot, c"\n".as_ptr() as *mut c_char);
        }
    }
}

/// Raven `PC_AddGlobalDefine` — add a define string to the global list. With
/// `DEFINEHASHING` (=1) live, the body reduces to a success return.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1407-1418`
pub fn PC_AddGlobalDefine(string: *mut c_char) -> c_int {
    // Whole body is `#if !DEFINEHASHING` (dead); DEFINEHASHING=1.
    qtrue
}

/// Raven `PC_RemoveGlobalDefine` — remove a global define by name. With
/// `DEFINEHASHING` (=1) live, the body reduces to a failure return.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1426-1439`
pub fn PC_RemoveGlobalDefine(name: *mut c_char) -> c_int {
    // Whole body is `#if !DEFINEHASHING` (dead); DEFINEHASHING=1.
    qfalse
}

/// Raven `PC_RemoveAllGlobalDefines` — free every define in the global hash
/// table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1447-1472`
pub fn PC_RemoveAllGlobalDefines(bot: &mut BotLib) {
    unsafe {
        // #if DEFINEHASHING (live)
        if !bot.globaldefines.is_null() {
            for i in 0..DEFINEHASHSIZE {
                while !(*bot.globaldefines.add(i)).is_null() {
                    let define: *mut define_t = *bot.globaldefines.add(i);
                    *bot.globaldefines.add(i) = (*define).globalnext;
                    PC_FreeDefine(bot, define);
                }
            }
        }
    }
}

/// Raven `PC_FreeDefine` — free a define and its parameter/macro tokens.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:640-658`
pub fn PC_FreeDefine(bot: &mut BotLib, define: *mut define_t) {
    unsafe {
        // free the define parameters
        let mut t: *mut token_t = (*define).parms;
        while !t.is_null() {
            let next = (*t).next;
            PC_FreeToken(bot, t);
            t = next;
        }
        // free the define tokens
        t = (*define).tokens;
        while !t.is_null() {
            let next = (*t).next;
            PC_FreeToken(bot, t);
            t = next;
        }
        // free the define
        FreeMemory(bot, define as *mut _);
    }
}

/// Raven `PC_AddBuiltinDefines` — register `__LINE__`/`__FILE__`/`__DATE__`/
/// `__TIME__` builtins on a source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:665-698`
pub fn PC_AddBuiltinDefines(bot: &mut BotLib, source: *mut source_t) {
    unsafe {
        // Raven's local `struct builtin { char *string; int mBuiltin; }` table.
        let builtin: [(*const c_char, c_int); 5] = [
            (c"__LINE__".as_ptr(), BUILTIN_LINE),
            (c"__FILE__".as_ptr(), BUILTIN_FILE),
            (c"__DATE__".as_ptr(), BUILTIN_DATE),
            (c"__TIME__".as_ptr(), BUILTIN_TIME),
            (core::ptr::null(), 0),
        ];

        let mut i = 0usize;
        while !builtin[i].0.is_null() {
            let define: *mut define_t = GetMemory(
                bot,
                (core::mem::size_of::<define_t>() + strlen(builtin[i].0) + 1) as c_ulong,
            ) as *mut define_t;
            Com_Memset(define.cast(), 0, core::mem::size_of::<define_t>());
            (*define).name = (define as *mut c_char).add(core::mem::size_of::<define_t>());
            strcpy((*define).name, builtin[i].0);
            (*define).flags |= DEFINE_FIXED;
            (*define).builtin = builtin[i].1;
            // add the define to the source (#if DEFINEHASHING, live)
            PC_AddDefineToHash(bot, define, (*source).definehash);
            i += 1;
        }
        let _ = BUILTIN_STDC;
    }
}

/// Raven `PC_ExpandBuiltinDefine` — expand a builtin macro token into fresh
/// tokens.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:705-775`
pub fn PC_ExpandBuiltinDefine(
    bot: &mut BotLib,
    source: *mut source_t,
    deftoken: *mut token_t,
    define: *mut define_t,
    firsttoken: *mut *mut token_t,
    lasttoken: *mut *mut token_t,
) -> c_int {
    unsafe {
        let token: *mut token_t = PC_CopyToken(bot, deftoken);
        let t: c_ulong; // time_t t; (LCC warning workaround)
        let curtime: *const c_char;
        match (*define).builtin {
            BUILTIN_LINE => {
                sprintf(
                    (*token).string.as_mut_ptr(),
                    c"%d".as_ptr(),
                    (*deftoken).line,
                );
                (*token).r#type = TT_NUMBER;
                (*token).subtype = TT_DECIMAL | TT_INTEGER;
                *firsttoken = token;
                *lasttoken = token;
            }
            BUILTIN_FILE => {
                strcpy(
                    (*token).string.as_mut_ptr(),
                    (*(*source).scriptstack).filename.as_ptr(),
                );
                (*token).r#type = TT_NAME;
                (*token).subtype = strlen((*token).string.as_ptr()) as c_int;
                *firsttoken = token;
                *lasttoken = token;
            }
            BUILTIN_DATE => {
                t = time(core::ptr::null_mut()) as c_ulong;
                let curtime_buf = ctime_buf(t as libc::time_t);
                curtime = curtime_buf.as_ptr();
                strcpy((*token).string.as_mut_ptr(), c"\"".as_ptr());
                strncat((*token).string.as_mut_ptr(), curtime.add(4), 7);
                strncat((*token).string.as_mut_ptr().add(7), curtime.add(20), 4);
                strcat((*token).string.as_mut_ptr(), c"\"".as_ptr());
                (*token).r#type = TT_NAME;
                (*token).subtype = strlen((*token).string.as_ptr()) as c_int;
                *firsttoken = token;
                *lasttoken = token;
            }
            BUILTIN_TIME => {
                t = time(core::ptr::null_mut()) as c_ulong;
                let curtime_buf = ctime_buf(t as libc::time_t);
                curtime = curtime_buf.as_ptr();
                strcpy((*token).string.as_mut_ptr(), c"\"".as_ptr());
                strncat((*token).string.as_mut_ptr(), curtime.add(11), 8);
                strcat((*token).string.as_mut_ptr(), c"\"".as_ptr());
                (*token).r#type = TT_NAME;
                (*token).subtype = strlen((*token).string.as_ptr()) as c_int;
                *firsttoken = token;
                *lasttoken = token;
            }
            // BUILTIN_STDC and default
            _ => {
                *firsttoken = core::ptr::null_mut();
                *lasttoken = core::ptr::null_mut();
                let _ = BUILTIN_STDC;
            }
        }
        qtrue
    }
}

/// Raven `PC_CopyDefine` — deep-copy a define (name, flags, parms, tokens),
/// unlinked.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1479-1515`
pub fn PC_CopyDefine(
    bot: &mut BotLib,
    source: *mut source_t,
    define: *mut define_t,
) -> *mut define_t {
    unsafe {
        let newdefine: *mut define_t = GetMemory(
            bot,
            (core::mem::size_of::<define_t>() + strlen((*define).name) + 1) as c_ulong,
        ) as *mut define_t;
        // copy the define name
        (*newdefine).name = (newdefine as *mut c_char).add(core::mem::size_of::<define_t>());
        strcpy((*newdefine).name, (*define).name);
        (*newdefine).flags = (*define).flags;
        (*newdefine).builtin = (*define).builtin;
        (*newdefine).numparms = (*define).numparms;
        // the define is not linked
        (*newdefine).next = core::ptr::null_mut();
        (*newdefine).hashnext = core::ptr::null_mut();
        // copy the define tokens
        (*newdefine).tokens = core::ptr::null_mut();
        let mut lasttoken: *mut token_t = core::ptr::null_mut();
        let mut token: *mut token_t = (*define).tokens;
        while !token.is_null() {
            let newtoken = PC_CopyToken(bot, token);
            (*newtoken).next = core::ptr::null_mut();
            if !lasttoken.is_null() {
                (*lasttoken).next = newtoken;
            } else {
                (*newdefine).tokens = newtoken;
            }
            lasttoken = newtoken;
            token = (*token).next;
        }
        // copy the define parameters
        (*newdefine).parms = core::ptr::null_mut();
        lasttoken = core::ptr::null_mut();
        token = (*define).parms;
        while !token.is_null() {
            let newtoken = PC_CopyToken(bot, token);
            (*newtoken).next = core::ptr::null_mut();
            if !lasttoken.is_null() {
                (*lasttoken).next = newtoken;
            } else {
                (*newdefine).parms = newtoken;
            }
            lasttoken = newtoken;
            token = (*token).next;
        }
        newdefine
    }
}

/// Raven `PC_AddGlobalDefinesToSource` — copy every global define into a source's
/// hash table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1522-1551`
pub fn PC_AddGlobalDefinesToSource(bot: &mut BotLib, source: *mut source_t) {
    unsafe {
        // #if DEFINEHASHING (live)
        for i in 0..DEFINEHASHSIZE {
            let mut define: *mut define_t = *bot.globaldefines.add(i);
            while !define.is_null() {
                (*define).hashnext = core::ptr::null_mut();
                PC_AddDefineToHash(bot, define, (*source).definehash);

                define = (*define).globalnext;
            }
        }
    }
}

/// Raven `PC_ConvertPath` — collapse doubled separators and normalize to the
/// OS path separator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:940-963`
pub fn PC_ConvertPath(path: *mut c_char) {
    unsafe {
        // remove double path seperators
        let mut ptr: *mut c_char = path;
        while *ptr != 0 {
            if (*ptr == b'\\' as c_char || *ptr == b'/' as c_char)
                && (*ptr.add(1) == b'\\' as c_char || *ptr.add(1) == b'/' as c_char)
            {
                strcpy(ptr, ptr.add(1));
            } else {
                ptr = ptr.add(1);
            }
        }
        // set OS dependent path seperators
        ptr = path;
        while *ptr != 0 {
            if *ptr == b'/' as c_char || *ptr == b'\\' as c_char {
                *ptr = PATHSEPERATOR_CHAR as c_char;
            }
            ptr = ptr.add(1);
        }
    }
}

/// Raven `PC_WhiteSpaceBeforeToken` — true if the token has leading whitespace.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1086-1089`
pub fn PC_WhiteSpaceBeforeToken(token: *mut token_t) -> c_int {
    unsafe { ((*token).endwhitespace_p as isize - (*token).whitespace_p as isize > 0) as c_int }
}

/// Raven `PC_ClearTokenWhiteSpace` — zero a token's whitespace bookkeeping.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1096-1101`
pub fn PC_ClearTokenWhiteSpace(token: *mut token_t) {
    unsafe {
        (*token).whitespace_p = core::ptr::null_mut();
        (*token).endwhitespace_p = core::ptr::null_mut();
        (*token).linescrossed = 0;
    }
}

/// Raven `PC_OperatorPriority` — precedence of a `#if`/`#elif` operator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1668-1701`
pub fn PC_OperatorPriority(op: c_int) -> c_int {
    match op {
        P_MUL => 15,
        P_DIV => 15,
        P_MOD => 15,
        P_ADD => 14,
        P_SUB => 14,

        P_LOGIC_AND => 7,
        P_LOGIC_OR => 6,
        P_LOGIC_GEQ => 12,
        P_LOGIC_LEQ => 12,
        P_LOGIC_EQ => 11,
        P_LOGIC_UNEQ => 11,

        P_LOGIC_NOT => 16,
        P_LOGIC_GREATER => 12,
        P_LOGIC_LESS => 12,

        P_RSHIFT => 13,
        P_LSHIFT => 13,

        P_BIN_AND => 10,
        P_BIN_OR => 8,
        P_BIN_XOR => 9,
        P_BIN_NOT => 16,

        P_COLON => 5,
        P_QUESTIONMARK => 5,
        _ => qfalse,
    }
}

/// Raven `PC_EvaluateTokens` — evaluate a `#if`/`#elif` token expression to an
/// integer/float value.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1730-2177`
///
/// The C `switch(t->type)`/`switch(t->subtype)` fallthrough is transcribed
/// with labeled blocks (`'sw`/`'subsw`) so a C `break` out of a case maps to
/// `break '<label>`.
pub fn PC_EvaluateTokens(
    bot: &mut BotLib,
    source: *mut source_t,
    tokens: *mut token_t,
    intvalue: *mut c_long,
    floatvalue: *mut f64,
    integer: c_int,
) -> c_int {
    unsafe {
        let mut firstoperator: *mut operator_t;
        let mut lastoperator: *mut operator_t;
        let mut firstvalue: *mut value_t;
        let mut lastvalue: *mut value_t;
        let mut v1: *mut value_t;
        let mut v2: *mut value_t;
        let mut brace: c_int = 0;
        let mut parentheses: c_int = 0;
        let mut error: c_int = 0;
        let mut lastwasvalue: c_int = 0;
        let mut negativevalue: c_int = 0;
        let mut questmarkintvalue: c_int = 0;
        let mut questmarkfloatvalue: f64 = 0.0;
        let mut gotquestmarkvalue: c_int = qfalse;
        let mut lastoperatortype: c_int = 0;
        //
        // §19: Raven reads these heaps only after AllocValue/AllocOperator writes
        // the slot; zero-init to keep it defined.
        let mut operator_heap: [operator_t; MAX_OPERATORS] = core::mem::zeroed();
        let mut numoperators: c_int = 0;
        let mut value_heap: [value_t; MAX_VALUES] = core::mem::zeroed();
        let mut numvalues: c_int = 0;

        firstoperator = core::ptr::null_mut();
        lastoperator = core::ptr::null_mut();
        firstvalue = core::ptr::null_mut();
        lastvalue = core::ptr::null_mut();
        if !intvalue.is_null() {
            *intvalue = 0;
        }
        if !floatvalue.is_null() {
            *floatvalue = 0.0;
        }
        let mut t: *mut token_t = tokens;
        'tokens: while !t.is_null() {
            'sw: {
                if (*t).r#type == TT_NAME {
                    if lastwasvalue != 0 || negativevalue != 0 {
                        source_error!(
                            bot,
                            source,
                            c"syntax error in #if/#elif".as_ptr() as *mut c_char
                        );
                        error = 1;
                        break 'sw;
                    }
                    if strcmp((*t).string.as_ptr(), c"defined".as_ptr()) != 0 {
                        source_error!(
                            bot,
                            source,
                            c"undefined name %s in #if/#elif".as_ptr() as *mut c_char,
                            (*t).string.as_ptr(),
                        );
                        error = 1;
                        break 'sw;
                    }
                    t = (*t).next;
                    if strcmp((*t).string.as_ptr(), c"(".as_ptr()) == 0 {
                        brace = qtrue;
                        t = (*t).next;
                    }
                    if t.is_null() || (*t).r#type != TT_NAME {
                        source_error!(
                            bot,
                            source,
                            c"defined without name in #if/#elif".as_ptr() as *mut c_char,
                        );
                        error = 1;
                        break 'sw;
                    }
                    // AllocValue(v)
                    if numvalues >= MAX_VALUES as c_int {
                        source_error!(bot, source, c"out of value space\n".as_ptr() as *mut c_char);
                        error = 1;
                        break 'sw;
                    }
                    let v: *mut value_t = &mut value_heap[numvalues as usize];
                    numvalues += 1;
                    // #if DEFINEHASHING (live)
                    if !PC_FindHashedDefine(
                        (*source).definehash,
                        (*t).string.as_ptr() as *mut c_char,
                    )
                    .is_null()
                    {
                        (*v).intvalue = 1;
                        (*v).floatvalue = 1.0;
                    } else {
                        (*v).intvalue = 0;
                        (*v).floatvalue = 0.0;
                    }
                    (*v).parentheses = parentheses;
                    (*v).next = core::ptr::null_mut();
                    (*v).prev = lastvalue;
                    if !lastvalue.is_null() {
                        (*lastvalue).next = v;
                    } else {
                        firstvalue = v;
                    }
                    lastvalue = v;
                    if brace != 0 {
                        t = (*t).next;
                        if t.is_null() || strcmp((*t).string.as_ptr(), c")".as_ptr()) != 0 {
                            source_error!(
                                bot,
                                source,
                                c"defined without ) in #if/#elif".as_ptr() as *mut c_char,
                            );
                            error = 1;
                            break 'sw;
                        }
                    }
                    brace = qfalse;
                    // defined() creates a value
                    lastwasvalue = 1;
                } else if (*t).r#type == TT_NUMBER {
                    if lastwasvalue != 0 {
                        source_error!(
                            bot,
                            source,
                            c"syntax error in #if/#elif".as_ptr() as *mut c_char
                        );
                        error = 1;
                        break 'sw;
                    }
                    // AllocValue(v)
                    if numvalues >= MAX_VALUES as c_int {
                        source_error!(bot, source, c"out of value space\n".as_ptr() as *mut c_char);
                        error = 1;
                        break 'sw;
                    }
                    let v: *mut value_t = &mut value_heap[numvalues as usize];
                    numvalues += 1;
                    if negativevalue != 0 {
                        (*v).intvalue = -((*t).intvalue as c_int) as c_long;
                        (*v).floatvalue = -(*t).floatvalue;
                    } else {
                        (*v).intvalue = (*t).intvalue as c_long;
                        (*v).floatvalue = (*t).floatvalue;
                    }
                    (*v).parentheses = parentheses;
                    (*v).next = core::ptr::null_mut();
                    (*v).prev = lastvalue;
                    if !lastvalue.is_null() {
                        (*lastvalue).next = v;
                    } else {
                        firstvalue = v;
                    }
                    lastvalue = v;
                    // last token was a value
                    lastwasvalue = 1;
                    //
                    negativevalue = 0;
                } else if (*t).r#type == TT_PUNCTUATION {
                    if negativevalue != 0 {
                        source_error!(
                            bot,
                            source,
                            c"misplaced minus sign in #if/#elif".as_ptr() as *mut c_char,
                        );
                        error = 1;
                        break 'sw;
                    }
                    if (*t).subtype == P_PARENTHESESOPEN {
                        parentheses += 1;
                        break 'sw;
                    } else if (*t).subtype == P_PARENTHESESCLOSE {
                        parentheses -= 1;
                        if parentheses < 0 {
                            source_error!(
                                bot,
                                source,
                                c"too many ) in #if/#elsif".as_ptr() as *mut c_char,
                            );
                            error = 1;
                        }
                        break 'sw;
                    }
                    // check for invalid operators on floating point values
                    if integer == 0
                        && ((*t).subtype == P_BIN_NOT
                            || (*t).subtype == P_MOD
                            || (*t).subtype == P_RSHIFT
                            || (*t).subtype == P_LSHIFT
                            || (*t).subtype == P_BIN_AND
                            || (*t).subtype == P_BIN_OR
                            || (*t).subtype == P_BIN_XOR)
                    {
                        source_error!(
                            bot,
                            source,
                            c"illigal operator %s on floating point operands\n".as_ptr()
                                as *mut c_char,
                            (*t).string.as_ptr(),
                        );
                        error = 1;
                        break 'sw;
                    }
                    'subsw: {
                        let st = (*t).subtype;
                        if st == P_LOGIC_NOT || st == P_BIN_NOT {
                            if lastwasvalue != 0 {
                                source_error!(
                                    bot,
                                    source,
                                    c"! or ~ after value in #if/#elif".as_ptr() as *mut c_char,
                                );
                                error = 1;
                                break 'subsw;
                            }
                        } else if st == P_INC || st == P_DEC {
                            source_error!(
                                bot,
                                source,
                                c"++ or -- used in #if/#elif".as_ptr() as *mut c_char,
                            );
                        } else if st == P_SUB && lastwasvalue == 0 {
                            // P_SUB with no preceding value: unary minus
                            negativevalue = 1;
                        } else if st == P_SUB
                            || st == P_MUL
                            || st == P_DIV
                            || st == P_MOD
                            || st == P_ADD
                            || st == P_LOGIC_AND
                            || st == P_LOGIC_OR
                            || st == P_LOGIC_GEQ
                            || st == P_LOGIC_LEQ
                            || st == P_LOGIC_EQ
                            || st == P_LOGIC_UNEQ
                            || st == P_LOGIC_GREATER
                            || st == P_LOGIC_LESS
                            || st == P_RSHIFT
                            || st == P_LSHIFT
                            || st == P_BIN_AND
                            || st == P_BIN_OR
                            || st == P_BIN_XOR
                            || st == P_COLON
                            || st == P_QUESTIONMARK
                        {
                            if lastwasvalue == 0 {
                                source_error!(
                                    bot,
                                    source,
                                    c"operator %s after operator in #if/#elif".as_ptr()
                                        as *mut c_char,
                                    (*t).string.as_ptr(),
                                );
                                error = 1;
                                break 'subsw;
                            }
                        } else {
                            source_error!(
                                bot,
                                source,
                                c"invalid operator %s in #if/#elif".as_ptr() as *mut c_char,
                                (*t).string.as_ptr(),
                            );
                            error = 1;
                            break 'subsw;
                        }
                    }
                    if error == 0 && negativevalue == 0 {
                        // AllocOperator(o)
                        if numoperators >= MAX_OPERATORS as c_int {
                            source_error!(
                                bot,
                                source,
                                c"out of operator space\n".as_ptr() as *mut c_char,
                            );
                            error = 1;
                            break 'sw;
                        }
                        let o: *mut operator_t = &mut operator_heap[numoperators as usize];
                        numoperators += 1;
                        (*o).mOperator = (*t).subtype;
                        (*o).priority = PC_OperatorPriority((*t).subtype);
                        (*o).parentheses = parentheses;
                        (*o).next = core::ptr::null_mut();
                        (*o).prev = lastoperator;
                        if !lastoperator.is_null() {
                            (*lastoperator).next = o;
                        } else {
                            firstoperator = o;
                        }
                        lastoperator = o;
                        lastwasvalue = 0;
                    }
                } else {
                    source_error!(
                        bot,
                        source,
                        c"unknown %s in #if/#elif".as_ptr() as *mut c_char,
                        (*t).string.as_ptr(),
                    );
                    error = 1;
                }
            }
            if error != 0 {
                break 'tokens;
            }
            t = (*t).next;
        }
        if error == 0 {
            if lastwasvalue == 0 {
                source_error!(
                    bot,
                    source,
                    c"trailing operator in #if/#elif".as_ptr() as *mut c_char
                );
                error = 1;
            } else if parentheses != 0 {
                source_error!(
                    bot,
                    source,
                    c"too many ( in #if/#elif".as_ptr() as *mut c_char
                );
                error = 1;
            }
        }
        //
        gotquestmarkvalue = qfalse;
        questmarkintvalue = 0;
        questmarkfloatvalue = 0.0;
        // while there are operators
        while error == 0 && !firstoperator.is_null() {
            let mut v: *mut value_t = firstvalue;
            let mut o: *mut operator_t = firstoperator;
            while !(*o).next.is_null() {
                // if the current operator is nested deeper in parentheses
                // than the next operator
                if (*o).parentheses > (*(*o).next).parentheses {
                    break;
                }
                // if the current and next operator are nested equally deep
                if (*o).parentheses == (*(*o).next).parentheses {
                    // if the priority is equal or higher than the next
                    if (*o).priority >= (*(*o).next).priority {
                        break;
                    }
                }
                // if the arity of the operator isn't equal to 1
                if (*o).mOperator != P_LOGIC_NOT && (*o).mOperator != P_BIN_NOT {
                    v = (*v).next;
                }
                // if there's no value or no next value
                if v.is_null() {
                    source_error!(
                        bot,
                        source,
                        c"mising values in #if/#elif".as_ptr() as *mut c_char
                    );
                    error = 1;
                    break;
                }
                o = (*o).next;
            }
            if error != 0 {
                break;
            }
            v1 = v;
            v2 = (*v).next;
            match (*o).mOperator {
                P_LOGIC_NOT => {
                    (*v1).intvalue = ((*v1).intvalue == 0) as c_long;
                    (*v1).floatvalue = if (*v1).floatvalue == 0.0 { 1.0 } else { 0.0 };
                }
                P_BIN_NOT => {
                    (*v1).intvalue = !(*v1).intvalue;
                }
                P_MUL => {
                    (*v1).intvalue *= (*v2).intvalue;
                    (*v1).floatvalue *= (*v2).floatvalue;
                }
                P_DIV => {
                    if (*v2).intvalue == 0 || (*v2).floatvalue == 0.0 {
                        source_error!(
                            bot,
                            source,
                            c"divide by zero in #if/#elif\n".as_ptr() as *mut c_char,
                        );
                        error = 1;
                    } else {
                        (*v1).intvalue /= (*v2).intvalue;
                        (*v1).floatvalue /= (*v2).floatvalue;
                    }
                }
                P_MOD => {
                    if (*v2).intvalue == 0 {
                        source_error!(
                            bot,
                            source,
                            c"divide by zero in #if/#elif\n".as_ptr() as *mut c_char,
                        );
                        error = 1;
                    } else {
                        (*v1).intvalue %= (*v2).intvalue;
                    }
                }
                P_ADD => {
                    (*v1).intvalue += (*v2).intvalue;
                    (*v1).floatvalue += (*v2).floatvalue;
                }
                P_SUB => {
                    (*v1).intvalue -= (*v2).intvalue;
                    (*v1).floatvalue -= (*v2).floatvalue;
                }
                P_LOGIC_AND => {
                    (*v1).intvalue = ((*v1).intvalue != 0 && (*v2).intvalue != 0) as c_long;
                    (*v1).floatvalue =
                        (((*v1).floatvalue != 0.0) && ((*v2).floatvalue != 0.0)) as i32 as f64;
                }
                P_LOGIC_OR => {
                    (*v1).intvalue = ((*v1).intvalue != 0 || (*v2).intvalue != 0) as c_long;
                    (*v1).floatvalue =
                        (((*v1).floatvalue != 0.0) || ((*v2).floatvalue != 0.0)) as i32 as f64;
                }
                P_LOGIC_GEQ => {
                    (*v1).intvalue = ((*v1).intvalue >= (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue >= (*v2).floatvalue) as i32 as f64;
                }
                P_LOGIC_LEQ => {
                    (*v1).intvalue = ((*v1).intvalue <= (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue <= (*v2).floatvalue) as i32 as f64;
                }
                P_LOGIC_EQ => {
                    (*v1).intvalue = ((*v1).intvalue == (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue == (*v2).floatvalue) as i32 as f64;
                }
                P_LOGIC_UNEQ => {
                    (*v1).intvalue = ((*v1).intvalue != (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue != (*v2).floatvalue) as i32 as f64;
                }
                P_LOGIC_GREATER => {
                    (*v1).intvalue = ((*v1).intvalue > (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue > (*v2).floatvalue) as i32 as f64;
                }
                P_LOGIC_LESS => {
                    (*v1).intvalue = ((*v1).intvalue < (*v2).intvalue) as c_long;
                    (*v1).floatvalue = ((*v1).floatvalue < (*v2).floatvalue) as i32 as f64;
                }
                P_RSHIFT => {
                    (*v1).intvalue >>= (*v2).intvalue;
                }
                P_LSHIFT => {
                    (*v1).intvalue <<= (*v2).intvalue;
                }
                P_BIN_AND => {
                    (*v1).intvalue &= (*v2).intvalue;
                }
                P_BIN_OR => {
                    (*v1).intvalue |= (*v2).intvalue;
                }
                P_BIN_XOR => {
                    (*v1).intvalue ^= (*v2).intvalue;
                }
                P_COLON => {
                    if gotquestmarkvalue == 0 {
                        source_error!(
                            bot,
                            source,
                            c": without ? in #if/#elif".as_ptr() as *mut c_char
                        );
                        error = 1;
                    } else {
                        if integer != 0 {
                            if questmarkintvalue == 0 {
                                (*v1).intvalue = (*v2).intvalue;
                            }
                        } else if questmarkfloatvalue == 0.0 {
                            (*v1).floatvalue = (*v2).floatvalue;
                        }
                        gotquestmarkvalue = qfalse;
                    }
                }
                P_QUESTIONMARK => {
                    if gotquestmarkvalue != 0 {
                        source_error!(
                            bot,
                            source,
                            c"? after ? in #if/#elif".as_ptr() as *mut c_char
                        );
                        error = 1;
                    } else {
                        questmarkintvalue = (*v1).intvalue as c_int;
                        questmarkfloatvalue = (*v1).floatvalue;
                        gotquestmarkvalue = qtrue;
                    }
                }
                _ => {}
            }
            if error != 0 {
                break;
            }
            lastoperatortype = (*o).mOperator;
            // if not an operator with arity 1
            if (*o).mOperator != P_LOGIC_NOT && (*o).mOperator != P_BIN_NOT {
                // remove the second value if not question mark operator
                if (*o).mOperator != P_QUESTIONMARK {
                    v = (*v).next;
                }
                //
                if !(*v).prev.is_null() {
                    (*(*v).prev).next = (*v).next;
                } else {
                    firstvalue = (*v).next;
                }
                if !(*v).next.is_null() {
                    (*(*v).next).prev = (*v).prev;
                } else {
                    lastvalue = (*v).prev;
                }
                // FreeValue(v) — no-op macro
            }
            // remove the operator
            if !(*o).prev.is_null() {
                (*(*o).prev).next = (*o).next;
            } else {
                firstoperator = (*o).next;
            }
            if !(*o).next.is_null() {
                (*(*o).next).prev = (*o).prev;
            } else {
                lastoperator = (*o).prev;
            }
            // FreeOperator(o) — no-op macro
        }
        if !firstvalue.is_null() {
            if !intvalue.is_null() {
                *intvalue = (*firstvalue).intvalue;
            }
            if !floatvalue.is_null() {
                *floatvalue = (*firstvalue).floatvalue;
            }
        }
        let mut o: *mut operator_t = firstoperator;
        while !o.is_null() {
            lastoperator = (*o).next;
            // FreeOperator(o) — no-op macro
            o = lastoperator;
        }
        let mut v: *mut value_t = firstvalue;
        while !v.is_null() {
            lastvalue = (*v).next;
            // FreeValue(v) — no-op macro
            v = lastvalue;
        }
        let _ = lastoperatortype;
        if error == 0 {
            return qtrue;
        }
        if !intvalue.is_null() {
            *intvalue = 0;
        }
        if !floatvalue.is_null() {
            *floatvalue = 0.0;
        }
        qfalse
    }
}

/// Raven `FreeSource` — free a source and every script/token/define/indent it
/// owns.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3113-3177`
pub fn FreeSource(bot: &mut BotLib, source: *mut source_t) {
    unsafe {
        // free all the scripts
        while !(*source).scriptstack.is_null() {
            let script = (*source).scriptstack;
            (*source).scriptstack = (*(*source).scriptstack).next;
            FreeScript(bot, script);
        }
        // free all the tokens
        while !(*source).tokens.is_null() {
            let token = (*source).tokens;
            (*source).tokens = (*(*source).tokens).next;
            PC_FreeToken(bot, token);
        }
        // #if DEFINEHASHING (live)
        for i in 0..DEFINEHASHSIZE {
            let mut define: *mut define_t = *(*source).definehash.add(i);
            while !define.is_null() {
                let nextdefine = (*define).hashnext;

                if ((*define).flags & DEFINE_GLOBAL) == 0 {
                    PC_FreeDefine(bot, define);
                }

                define = nextdefine;
            }

            *(*source).definehash.add(i) = core::ptr::null_mut();
        }
        // free all indents
        while !(*source).indentstack.is_null() {
            let indent = (*source).indentstack;
            (*source).indentstack = (*(*source).indentstack).next;
            FreeMemory(bot, indent as *mut _);
        }
        // #if DEFINEHASHING (live)
        if !(*source).definehash.is_null() {
            FreeMemory(bot, (*source).definehash as *mut _);
        }
        // free the source itself
        FreeMemory(bot, source as *mut _);
    }
}

/// Raven `PC_ReadSourceToken` — read the next token from the source, popping
/// finished scripts.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:293-329`
pub fn PC_ReadSourceToken(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut r#type: c_int = 0;
        let mut skip: c_int = 0;
        // if there's no token already available
        while (*source).tokens.is_null() {
            // if there's a token to read from the script
            if PS_ReadToken(bot, (*source).scriptstack, token) != 0 {
                return qtrue;
            }
            // if at the end of the script
            if EndOfScript((*source).scriptstack) != 0 {
                // remove all indents of the script
                while !(*source).indentstack.is_null()
                    && (*(*source).indentstack).script == (*source).scriptstack
                {
                    source_warning!(bot, source, c"missing #endif".as_ptr() as *mut c_char);
                    PC_PopIndent(bot, source, &mut r#type, &mut skip);
                }
            }
            // if this was the initial script
            if (*(*source).scriptstack).next.is_null() {
                return qfalse;
            }
            // remove the script and return to the last one
            let script = (*source).scriptstack;
            (*source).scriptstack = (*(*source).scriptstack).next;
            FreeScript(bot, script);
        }
        // copy the already available token
        Com_Memcpy(
            token.cast(),
            (*source).tokens.cast(),
            core::mem::size_of::<token_t>(),
        );
        // free the read token
        let t = (*source).tokens;
        (*source).tokens = (*(*source).tokens).next;
        PC_FreeToken(bot, t);
        qtrue
    }
}

/// Raven `PC_UnreadSourceToken` — push a copy of a token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:336-344`
pub fn PC_UnreadSourceToken(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) -> c_int {
    unsafe {
        let t: *mut token_t = PC_CopyToken(bot, token);
        (*t).next = (*source).tokens;
        (*source).tokens = t;
        qtrue
    }
}

/// Raven `PC_ReadDefineParms` — read the actual parameters of a macro
/// invocation into `parms`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:351-443`
pub fn PC_ReadDefineParms(
    bot: &mut BotLib,
    source: *mut source_t,
    define: *mut define_t,
    parms: *mut *mut token_t,
    maxparms: c_int,
) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();
        let mut t: *mut token_t;
        let mut last: *mut token_t;
        let mut done: c_int;
        let mut lastcomma: c_int;
        let mut numparms: c_int;
        let mut indent: c_int;

        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(
                bot,
                source,
                c"define %s missing parms".as_ptr() as *mut c_char,
                (*define).name,
            );
            return qfalse;
        }
        //
        if (*define).numparms > maxparms {
            source_error!(
                bot,
                source,
                c"define with more than %d parameters".as_ptr() as *mut c_char,
                maxparms,
            );
            return qfalse;
        }
        //
        for i in 0..(*define).numparms {
            *parms.add(i as usize) = core::ptr::null_mut();
        }
        // if no leading "("
        if strcmp(token.string.as_ptr(), c"(".as_ptr()) != 0 {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"define %s missing parms".as_ptr() as *mut c_char,
                (*define).name,
            );
            return qfalse;
        }
        // read the define parameters
        done = 0;
        numparms = 0;
        indent = 0;
        while done == 0 {
            if numparms >= maxparms {
                source_error!(
                    bot,
                    source,
                    c"define %s with too many parms".as_ptr() as *mut c_char,
                    (*define).name,
                );
                return qfalse;
            }
            if numparms >= (*define).numparms {
                source_warning!(
                    bot,
                    source,
                    c"define %s has too many parms".as_ptr() as *mut c_char,
                    (*define).name,
                );
                return qfalse;
            }
            *parms.add(numparms as usize) = core::ptr::null_mut();
            lastcomma = 1;
            last = core::ptr::null_mut();
            while done == 0 {
                //
                if PC_ReadSourceToken(bot, source, &mut token) == 0 {
                    source_error!(
                        bot,
                        source,
                        c"define %s incomplete".as_ptr() as *mut c_char,
                        (*define).name,
                    );
                    return qfalse;
                }
                //
                if strcmp(token.string.as_ptr(), c",".as_ptr()) == 0 && indent <= 0 {
                    if lastcomma != 0 {
                        source_warning!(bot, source, c"too many comma's".as_ptr() as *mut c_char);
                    }
                    lastcomma = 1;
                    break;
                }
                lastcomma = 0;
                //
                if strcmp(token.string.as_ptr(), c"(".as_ptr()) == 0 {
                    indent += 1;
                    continue;
                } else if strcmp(token.string.as_ptr(), c")".as_ptr()) == 0 {
                    indent -= 1;
                    if indent <= 0 {
                        if (*parms.add(((*define).numparms - 1) as usize)).is_null() {
                            source_warning!(
                                bot,
                                source,
                                c"too few define parms".as_ptr() as *mut c_char
                            );
                        }
                        done = 1;
                        break;
                    }
                }
                //
                if numparms < (*define).numparms {
                    //
                    t = PC_CopyToken(bot, &mut token);
                    (*t).next = core::ptr::null_mut();
                    if !last.is_null() {
                        (*last).next = t;
                    } else {
                        *parms.add(numparms as usize) = t;
                    }
                    last = t;
                }
            }
            numparms += 1;
        }
        qtrue
    }
}

/// Raven `PC_Directive_include` — handle `#include "file"` / `#include <file>`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:970-1053`
pub fn PC_Directive_include(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut script: *mut script_t;
        let mut token: token_t = core::mem::zeroed();
        let mut path = [0 as c_char; MAX_PATH];

        if (*source).skip > 0 {
            return qtrue;
        }
        //
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(
                bot,
                source,
                c"#include without file name".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        if token.linescrossed > 0 {
            source_error!(
                bot,
                source,
                c"#include without file name".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        if token.r#type == TT_STRING {
            StripDoubleQuotes(token.string.as_mut_ptr());
            PC_ConvertPath(token.string.as_mut_ptr());
            script = LoadScriptFile(bot, token.string.as_ptr());
            if script.is_null() {
                strcpy(path.as_mut_ptr(), (*source).includepath.as_ptr());
                strcat(path.as_mut_ptr(), token.string.as_ptr());
                script = LoadScriptFile(bot, path.as_ptr());
            }
        } else if token.r#type == TT_PUNCTUATION && token.string[0] == b'<' as c_char {
            strcpy(path.as_mut_ptr(), (*source).includepath.as_ptr());
            while PC_ReadSourceToken(bot, source, &mut token) != 0 {
                if token.linescrossed > 0 {
                    PC_UnreadSourceToken(bot, source, &mut token);
                    break;
                }
                if token.r#type == TT_PUNCTUATION && token.string[0] == b'>' as c_char {
                    break;
                }
                strncat(path.as_mut_ptr(), token.string.as_ptr(), MAX_PATH);
            }
            if token.string[0] != b'>' as c_char {
                source_warning!(
                    bot,
                    source,
                    c"#include missing trailing >".as_ptr() as *mut c_char
                );
            }
            if strlen(path.as_ptr()) == 0 {
                source_error!(
                    bot,
                    source,
                    c"#include without file name between < >".as_ptr() as *mut c_char,
                );
                return qfalse;
            }
            PC_ConvertPath(path.as_mut_ptr());
            script = LoadScriptFile(bot, path.as_ptr());
        } else {
            source_error!(
                bot,
                source,
                c"#include without file name".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        // #ifdef QUAKE (not defined) omitted.
        if script.is_null() {
            // #ifdef SCREWUP (not defined) -> SourceError branch
            source_error!(
                bot,
                source,
                c"file %s not found".as_ptr() as *mut c_char,
                path.as_ptr(),
            );
            return qfalse;
        }
        PC_PushScript(bot, source, script);
        qtrue
    }
}

/// Raven `PC_ReadLine` — read a token on the current logical line, honoring
/// line continuations.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1062-1079`
pub fn PC_ReadLine(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut crossline: c_int = 0;
        loop {
            if PC_ReadSourceToken(bot, source, token) == 0 {
                return qfalse;
            }

            if (*token).linescrossed > crossline {
                PC_UnreadSourceToken(bot, source, token);
                return qfalse;
            }
            crossline = 1;
            if strcmp((*token).string.as_ptr(), c"\\".as_ptr()) != 0 {
                break;
            }
        }
        qtrue
    }
}

/// Raven `PC_Directive_undef` — handle `#undef name`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1108-1173`
pub fn PC_Directive_undef(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();
        let mut define: *mut define_t;
        let mut lastdefine: *mut define_t;

        if (*source).skip > 0 {
            return qtrue;
        }
        //
        if PC_ReadLine(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"undef without name".as_ptr() as *mut c_char);
            return qfalse;
        }
        if token.r#type != TT_NAME {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"expected name, found %s".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
            return qfalse;
        }
        // #if DEFINEHASHING (live)
        let hash = PC_NameHash(token.string.as_mut_ptr());
        lastdefine = core::ptr::null_mut();
        define = *(*source).definehash.add(hash as usize);
        while !define.is_null() {
            if strcmp((*define).name, token.string.as_ptr()) == 0 {
                if (*define).flags & DEFINE_FIXED != 0 {
                    source_warning!(
                        bot,
                        source,
                        c"can't undef %s".as_ptr() as *mut c_char,
                        token.string.as_ptr(),
                    );
                } else {
                    if !lastdefine.is_null() {
                        (*lastdefine).hashnext = (*define).hashnext;
                    } else {
                        *(*source).definehash.add(hash as usize) = (*define).hashnext;
                    }

                    if ((*define).flags & DEFINE_GLOBAL) == 0 {
                        PC_FreeDefine(bot, define);
                    }
                }
                break;
            }
            lastdefine = define;
            define = (*define).hashnext;
        }
        qtrue
    }
}

/// Raven `PC_Directive_define` — handle `#define name[(...)] tokens`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1180-1316`
pub fn PC_Directive_define(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();
        let mut t: *mut token_t;
        let mut last: *mut token_t;
        let mut define: *mut define_t;

        if (*source).skip > 0 {
            return qtrue;
        }
        //
        if PC_ReadLine(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"#define without name".as_ptr() as *mut c_char);
            return qfalse;
        }
        if token.r#type != TT_NAME {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"expected name after #define, found %s".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
            return qfalse;
        }
        // check if the define already exists (#if DEFINEHASHING, live)
        define = PC_FindHashedDefine((*source).definehash, token.string.as_mut_ptr());
        if !define.is_null() {
            if (*define).flags & DEFINE_FIXED != 0 {
                source_error!(
                    bot,
                    source,
                    c"can't redefine %s".as_ptr() as *mut c_char,
                    token.string.as_ptr(),
                );
                return qfalse;
            }
            source_warning!(
                bot,
                source,
                c"redefinition of %s".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
            // unread the define name before executing the #undef directive
            PC_UnreadSourceToken(bot, source, &mut token);
            if PC_Directive_undef(bot, source) == 0 {
                return qfalse;
            }
            // if the define was not removed (define->flags & DEFINE_FIXED)
            define = PC_FindHashedDefine((*source).definehash, token.string.as_mut_ptr());
        }
        // allocate define
        define = GetMemory(
            bot,
            (core::mem::size_of::<define_t>() + strlen(token.string.as_ptr()) + 1) as c_ulong,
        ) as *mut define_t;
        Com_Memset(define.cast(), 0, core::mem::size_of::<define_t>());
        (*define).name = (define as *mut c_char).add(core::mem::size_of::<define_t>());
        strcpy((*define).name, token.string.as_ptr());
        // add the define to the source (#if DEFINEHASHING, live)
        PC_AddDefineToHash(bot, define, (*source).definehash);
        // if nothing is defined, just return
        if PC_ReadLine(bot, source, &mut token) == 0 {
            return qtrue;
        }
        // if it is a define with parameters
        if PC_WhiteSpaceBeforeToken(&mut token) == 0
            && strcmp(token.string.as_ptr(), c"(".as_ptr()) == 0
        {
            // read the define parameters
            last = core::ptr::null_mut();
            if PC_CheckTokenString(bot, source, c")".as_ptr() as *mut c_char) == 0 {
                loop {
                    if PC_ReadLine(bot, source, &mut token) == 0 {
                        source_error!(
                            bot,
                            source,
                            c"expected define parameter".as_ptr() as *mut c_char
                        );
                        return qfalse;
                    }
                    // if it isn't a name
                    if token.r#type != TT_NAME {
                        source_error!(
                            bot,
                            source,
                            c"invalid define parameter".as_ptr() as *mut c_char
                        );
                        return qfalse;
                    }
                    //
                    if PC_FindDefineParm(define, token.string.as_mut_ptr()) >= 0 {
                        source_error!(
                            bot,
                            source,
                            c"two the same define parameters".as_ptr() as *mut c_char,
                        );
                        return qfalse;
                    }
                    // add the define parm
                    t = PC_CopyToken(bot, &mut token);
                    PC_ClearTokenWhiteSpace(t);
                    (*t).next = core::ptr::null_mut();
                    if !last.is_null() {
                        (*last).next = t;
                    } else {
                        (*define).parms = t;
                    }
                    last = t;
                    (*define).numparms += 1;
                    // read next token
                    if PC_ReadLine(bot, source, &mut token) == 0 {
                        source_error!(
                            bot,
                            source,
                            c"define parameters not terminated".as_ptr() as *mut c_char,
                        );
                        return qfalse;
                    }
                    //
                    if strcmp(token.string.as_ptr(), c")".as_ptr()) == 0 {
                        break;
                    }
                    // then it must be a comma
                    if strcmp(token.string.as_ptr(), c",".as_ptr()) != 0 {
                        source_error!(
                            bot,
                            source,
                            c"define not terminated".as_ptr() as *mut c_char
                        );
                        return qfalse;
                    }
                }
            }
            if PC_ReadLine(bot, source, &mut token) == 0 {
                return qtrue;
            }
        }
        // read the defined stuff
        last = core::ptr::null_mut();
        loop {
            t = PC_CopyToken(bot, &mut token);
            if (*t).r#type == TT_NAME && strcmp((*t).string.as_ptr(), (*define).name) == 0 {
                source_error!(
                    bot,
                    source,
                    c"recursive define (removed recursion)".as_ptr() as *mut c_char,
                );
                if PC_ReadLine(bot, source, &mut token) == 0 {
                    break;
                }
                continue;
            }
            PC_ClearTokenWhiteSpace(t);
            (*t).next = core::ptr::null_mut();
            if !last.is_null() {
                (*last).next = t;
            } else {
                (*define).tokens = t;
            }
            last = t;
            if PC_ReadLine(bot, source, &mut token) == 0 {
                break;
            }
        }
        //
        if !last.is_null() {
            // check for merge operators at the beginning or end
            if strcmp((*(*define).tokens).string.as_ptr(), c"##".as_ptr()) == 0
                || strcmp((*last).string.as_ptr(), c"##".as_ptr()) == 0
            {
                source_error!(
                    bot,
                    source,
                    c"define with misplaced ##".as_ptr() as *mut c_char
                );
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `PC_DefineFromString` — build a define from a `name value` string.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1323-1374`
pub fn PC_DefineFromString(bot: &mut BotLib, string: *mut c_char) -> *mut define_t {
    unsafe {
        let mut src: source_t = core::mem::zeroed();
        let mut def: *mut define_t;

        PC_InitTokenHeap();

        let script: *mut script_t = LoadScriptMemory(
            bot,
            string,
            strlen(string) as c_int,
            c"*extern".as_ptr() as *mut c_char,
        );
        // create a new source
        Com_Memset(
            (&mut src as *mut source_t).cast(),
            0,
            core::mem::size_of::<source_t>(),
        );
        strncpy(src.filename.as_mut_ptr(), c"*extern".as_ptr(), MAX_PATH);
        src.scriptstack = script;
        // #if DEFINEHASHING (live)
        src.definehash = GetClearedMemory(
            bot,
            (DEFINEHASHSIZE * core::mem::size_of::<*mut define_t>()) as c_ulong,
        ) as *mut *mut define_t;
        // create a define from the source
        let res = PC_Directive_define(bot, &mut src);
        // free any tokens if left
        let mut t: *mut token_t = src.tokens;
        while !t.is_null() {
            src.tokens = (*src.tokens).next;
            PC_FreeToken(bot, t);
            t = src.tokens;
        }
        // #ifdef DEFINEHASHING (live)
        def = core::ptr::null_mut();
        for i in 0..DEFINEHASHSIZE {
            if !(*src.definehash.add(i)).is_null() {
                def = *src.definehash.add(i);
                break;
            }
        }
        //
        // #if DEFINEHASHING (live)
        FreeMemory(bot, src.definehash as *mut _);
        //
        FreeScript(bot, script);
        // if the define was created succesfully
        if res > 0 {
            return def;
        }
        // free the define is created
        if !src.defines.is_null() {
            PC_FreeDefine(bot, def);
        }
        //
        core::ptr::null_mut()
    }
}

/// Raven `PC_AddDefine` — add a `name value` define to a source (or globally).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1381-1399`
pub fn PC_AddDefine(bot: &mut BotLib, source: *mut source_t, string: *mut c_char) -> c_int {
    unsafe {
        if bot.addGlobalDefine == qtrue {
            return PC_AddGlobalDefine(string);
        }

        let define: *mut define_t = PC_DefineFromString(bot, string);
        if define.is_null() {
            return qfalse;
        }
        // #if DEFINEHASHING (live)
        PC_AddDefineToHash(bot, define, (*source).definehash);
        qtrue
    }
}

/// Raven `PC_Directive_if_def` — shared body of `#ifdef`/`#ifndef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1558-1583`
pub fn PC_Directive_if_def(bot: &mut BotLib, source: *mut source_t, r#type: c_int) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();
        let d: *mut define_t;
        let skip: c_int;

        if PC_ReadLine(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"#ifdef without name".as_ptr() as *mut c_char);
            return qfalse;
        }
        if token.r#type != TT_NAME {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"expected name after #ifdef, found %s".as_ptr() as *mut c_char,
                token.string.as_ptr(),
            );
            return qfalse;
        }
        // #if DEFINEHASHING (live)
        d = PC_FindHashedDefine((*source).definehash, token.string.as_mut_ptr());
        skip = ((r#type == INDENT_IFDEF) == d.is_null()) as c_int;
        PC_PushIndent(bot, source, r#type, skip);
        qtrue
    }
}

/// Raven `PC_Directive_ifdef` — handle `#ifdef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1590-1593`
pub fn PC_Directive_ifdef(bot: &mut BotLib, source: *mut source_t) -> c_int {
    PC_Directive_if_def(bot, source, INDENT_IFDEF)
}

/// Raven `PC_Directive_ifndef` — handle `#ifndef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1600-1603`
pub fn PC_Directive_ifndef(bot: &mut BotLib, source: *mut source_t) -> c_int {
    PC_Directive_if_def(bot, source, INDENT_IFNDEF)
}

/// Raven `PC_Directive_else` — handle `#else`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1610-1627`
pub fn PC_Directive_else(bot: &mut BotLib, source: *mut source_t) -> c_int {
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(bot, source, &mut r#type, &mut skip);
    if r#type == 0 {
        unsafe { source_error!(bot, source, c"misplaced #else".as_ptr() as *mut c_char) };
        return qfalse;
    }
    if r#type == INDENT_ELSE {
        unsafe { source_error!(bot, source, c"#else after #else".as_ptr() as *mut c_char) };
        return qfalse;
    }
    PC_PushIndent(bot, source, INDENT_ELSE, (skip == 0) as c_int);
    qtrue
}

/// Raven `PC_Directive_endif` — handle `#endif`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1634-1645`
pub fn PC_Directive_endif(bot: &mut BotLib, source: *mut source_t) -> c_int {
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(bot, source, &mut r#type, &mut skip);
    if r#type == 0 {
        unsafe { source_error!(bot, source, c"misplaced #endif".as_ptr() as *mut c_char) };
        return qfalse;
    }
    qtrue
}

/// Raven `PC_ExpandDefine` — expand a macro invocation into a token list.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:782-913`
pub fn PC_ExpandDefine(
    bot: &mut BotLib,
    source: *mut source_t,
    deftoken: *mut token_t,
    define: *mut define_t,
    firsttoken: *mut *mut token_t,
    lasttoken: *mut *mut token_t,
) -> c_int {
    unsafe {
        let mut parms = [core::ptr::null_mut::<token_t>(); MAX_DEFINEPARMS];
        let mut dt: *mut token_t;
        let mut pt: *mut token_t;
        let mut t: *mut token_t;
        let mut t1: *mut token_t;
        let mut t2: *mut token_t;
        let mut first: *mut token_t;
        let mut last: *mut token_t;
        let mut nextpt: *mut token_t;
        let mut token: token_t = core::mem::zeroed();
        let mut parmnum: c_int;

        // if it is a builtin define
        if (*define).builtin != 0 {
            return PC_ExpandBuiltinDefine(bot, source, deftoken, define, firsttoken, lasttoken);
        }
        // if the define has parameters
        if (*define).numparms != 0 {
            if PC_ReadDefineParms(
                bot,
                source,
                define,
                parms.as_mut_ptr(),
                MAX_DEFINEPARMS as c_int,
            ) == 0
            {
                return qfalse;
            }
        }
        // empty list at first
        first = core::ptr::null_mut();
        last = core::ptr::null_mut();
        // create a list with tokens of the expanded define
        dt = (*define).tokens;
        while !dt.is_null() {
            parmnum = -1;
            // if the token is a name, it could be a define parameter
            if (*dt).r#type == TT_NAME {
                parmnum = PC_FindDefineParm(define, (*dt).string.as_mut_ptr());
            }
            // if it is a define parameter
            if parmnum >= 0 {
                pt = parms[parmnum as usize];
                while !pt.is_null() {
                    t = PC_CopyToken(bot, pt);
                    // add the token to the list
                    (*t).next = core::ptr::null_mut();
                    if !last.is_null() {
                        (*last).next = t;
                    } else {
                        first = t;
                    }
                    last = t;
                    pt = (*pt).next;
                }
            } else {
                // if stringizing operator
                if (*dt).string[0] == b'#' as c_char && (*dt).string[1] == b'\0' as c_char {
                    // the stringizing operator must be followed by a define parameter
                    if !(*dt).next.is_null() {
                        parmnum = PC_FindDefineParm(define, (*(*dt).next).string.as_mut_ptr());
                    } else {
                        parmnum = -1;
                    }
                    //
                    if parmnum >= 0 {
                        // step over the stringizing operator
                        dt = (*dt).next;
                        // stringize the define parameter tokens
                        if PC_StringizeTokens(parms[parmnum as usize], &mut token) == 0 {
                            source_error!(
                                bot,
                                source,
                                c"can't stringize tokens".as_ptr() as *mut c_char
                            );
                            return qfalse;
                        }
                        t = PC_CopyToken(bot, &mut token);
                    } else {
                        source_warning!(
                            bot,
                            source,
                            c"stringizing operator without define parameter".as_ptr()
                                as *mut c_char,
                        );
                        dt = (*dt).next;
                        continue;
                    }
                } else {
                    t = PC_CopyToken(bot, dt);
                }
                // add the token to the list
                (*t).next = core::ptr::null_mut();
                if !last.is_null() {
                    (*last).next = t;
                } else {
                    first = t;
                }
                last = t;
            }
            dt = (*dt).next;
        }
        // check for the merging operator
        t = first;
        while !t.is_null() {
            if !(*t).next.is_null() {
                // if the merging operator
                if (*(*t).next).string[0] == b'#' as c_char
                    && (*(*t).next).string[1] == b'#' as c_char
                {
                    t1 = t;
                    t2 = (*(*t).next).next;
                    if !t2.is_null() {
                        if PC_MergeTokens(t1, t2) == 0 {
                            source_error!(
                                bot,
                                source,
                                c"can't merge %s with %s".as_ptr() as *mut c_char,
                                (*t1).string.as_ptr(),
                                (*t2).string.as_ptr(),
                            );
                            return qfalse;
                        }
                        PC_FreeToken(bot, (*t1).next);
                        (*t1).next = (*t2).next;
                        if t2 == last {
                            last = t1;
                        }
                        PC_FreeToken(bot, t2);
                        continue;
                    }
                }
            }
            t = (*t).next;
        }
        // store the first and last token of the list
        *firsttoken = first;
        *lasttoken = last;
        // free all the parameter tokens
        for i in 0..(*define).numparms {
            pt = parms[i as usize];
            while !pt.is_null() {
                nextpt = (*pt).next;
                PC_FreeToken(bot, pt);
                pt = nextpt;
            }
        }
        //
        qtrue
    }
}

/// Raven `PC_ExpandDefineIntoSource` — expand a macro and push its tokens back
/// onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:920-933`
pub fn PC_ExpandDefineIntoSource(
    bot: &mut BotLib,
    source: *mut source_t,
    deftoken: *mut token_t,
    define: *mut define_t,
) -> c_int {
    unsafe {
        let mut firsttoken: *mut token_t = core::ptr::null_mut();
        let mut lasttoken: *mut token_t = core::ptr::null_mut();

        if PC_ExpandDefine(
            bot,
            source,
            deftoken,
            define,
            &mut firsttoken,
            &mut lasttoken,
        ) == 0
        {
            return qfalse;
        }

        if !firsttoken.is_null() && !lasttoken.is_null() {
            (*lasttoken).next = (*source).tokens;
            (*source).tokens = firsttoken;
            return qtrue;
        }
        qfalse
    }
}

/// Raven `PC_Directive_line` — `#line` is not supported.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2429-2433`
pub fn PC_Directive_line(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        source_error!(
            bot,
            source,
            c"#line directive not supported".as_ptr() as *mut c_char
        )
    };
    qfalse
}

/// Raven `PC_Directive_error` — `#error directive: <text>`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2440-2448`
pub fn PC_Directive_error(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        strcpy(token.string.as_mut_ptr(), c"".as_ptr());
        PC_ReadSourceToken(bot, source, &mut token);
        source_error!(
            bot,
            source,
            c"#error directive: %s".as_ptr() as *mut c_char,
            token.string.as_ptr(),
        );
        qfalse
    }
}

/// Raven `PC_Directive_pragma` — `#pragma` is not supported (skip the line).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2455-2462`
pub fn PC_Directive_pragma(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        source_warning!(
            bot,
            source,
            c"#pragma directive not supported".as_ptr() as *mut c_char
        );
        while PC_ReadLine(bot, source, &mut token) != 0 {}
        qtrue
    }
}

/// Raven `UnreadSignToken` — push a synthesized `-` token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2469-2481`
pub fn UnreadSignToken(bot: &mut BotLib, source: *mut source_t) {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        token.line = (*(*source).scriptstack).line;
        token.whitespace_p = (*(*source).scriptstack).script_p;
        token.endwhitespace_p = (*(*source).scriptstack).script_p;
        token.linescrossed = 0;
        strcpy(token.string.as_mut_ptr(), c"-".as_ptr());
        token.r#type = TT_PUNCTUATION;
        token.subtype = P_SUB;
        PC_UnreadSourceToken(bot, source, &mut token);
    }
}

/// Raven `PC_Directive_eval` — `#eval expr` pushes the integer result back.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2488-2505`
pub fn PC_Directive_eval(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut value: c_long = 0;
        let mut token: token_t = core::mem::zeroed();

        if PC_Evaluate(bot, source, &mut value, core::ptr::null_mut(), qtrue) == 0 {
            return qfalse;
        }
        //
        token.line = (*(*source).scriptstack).line;
        token.whitespace_p = (*(*source).scriptstack).script_p;
        token.endwhitespace_p = (*(*source).scriptstack).script_p;
        token.linescrossed = 0;
        sprintf(
            token.string.as_mut_ptr(),
            c"%d".as_ptr(),
            abs(value as c_int),
        );
        token.r#type = TT_NUMBER;
        token.subtype = TT_INTEGER | TT_LONG | TT_DECIMAL;
        PC_UnreadSourceToken(bot, source, &mut token);
        if value < 0 {
            UnreadSignToken(bot, source);
        }
        qtrue
    }
}

/// Raven `PC_Directive_evalfloat` — `#evalfloat expr` pushes the float result.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2512-2528`
pub fn PC_Directive_evalfloat(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut value: f64 = 0.0;
        let mut token: token_t = core::mem::zeroed();

        if PC_Evaluate(bot, source, core::ptr::null_mut(), &mut value, qfalse) == 0 {
            return qfalse;
        }
        token.line = (*(*source).scriptstack).line;
        token.whitespace_p = (*(*source).scriptstack).script_p;
        token.endwhitespace_p = (*(*source).scriptstack).script_p;
        token.linescrossed = 0;
        sprintf(token.string.as_mut_ptr(), c"%1.2f".as_ptr(), value.abs());
        token.r#type = TT_NUMBER;
        token.subtype = TT_FLOAT | TT_LONG | TT_DECIMAL;
        PC_UnreadSourceToken(bot, source, &mut token);
        if value < 0.0 {
            UnreadSignToken(bot, source);
        }
        qtrue
    }
}

/// Sentinel handler for the `{NULL, NULL}` terminator row of the directive
/// dispatch tables. Never invoked — the walk stops on the null `name` before
/// `func` is read — but a Rust `fn` pointer cannot be null, so the terminator
/// row carries this no-op in place of Raven's `NULL`.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2550,2652` (`{NULL, NULL}`).
fn PC_Directive_sentinel(_bot: &mut BotLib, _source: *mut source_t) -> c_int {
    0
}

/// Raven `directives[]` — file-scope `#`-directive dispatch table (ruling 5:
/// const `fn`-item table). `name` is compared with `strcmp`; the null-`name`
/// terminator row ends the walk.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2535-2551`.
const directives: [directive_t; 15] = [
    directive_t {
        name: c"if".as_ptr(),
        func: PC_Directive_if,
    },
    directive_t {
        name: c"ifdef".as_ptr(),
        func: PC_Directive_ifdef,
    },
    directive_t {
        name: c"ifndef".as_ptr(),
        func: PC_Directive_ifndef,
    },
    directive_t {
        name: c"elif".as_ptr(),
        func: PC_Directive_elif,
    },
    directive_t {
        name: c"else".as_ptr(),
        func: PC_Directive_else,
    },
    directive_t {
        name: c"endif".as_ptr(),
        func: PC_Directive_endif,
    },
    directive_t {
        name: c"include".as_ptr(),
        func: PC_Directive_include,
    },
    directive_t {
        name: c"define".as_ptr(),
        func: PC_Directive_define,
    },
    directive_t {
        name: c"undef".as_ptr(),
        func: PC_Directive_undef,
    },
    directive_t {
        name: c"line".as_ptr(),
        func: PC_Directive_line,
    },
    directive_t {
        name: c"error".as_ptr(),
        func: PC_Directive_error,
    },
    directive_t {
        name: c"pragma".as_ptr(),
        func: PC_Directive_pragma,
    },
    directive_t {
        name: c"eval".as_ptr(),
        func: PC_Directive_eval,
    },
    directive_t {
        name: c"evalfloat".as_ptr(),
        func: PC_Directive_evalfloat,
    },
    directive_t {
        name: core::ptr::null(),
        func: PC_Directive_sentinel,
    },
];

/// Raven `dollardirectives[]` — file-scope `$`-directive dispatch table.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2648-2653`.
const dollardirectives: [directive_t; 3] = [
    directive_t {
        name: c"evalint".as_ptr(),
        func: PC_DollarDirective_evalint,
    },
    directive_t {
        name: c"evalfloat".as_ptr(),
        func: PC_DollarDirective_evalfloat,
    },
    directive_t {
        name: core::ptr::null(),
        func: PC_Directive_sentinel,
    },
];

/// Raven `PC_ReadDirective` — dispatch a `#`-directive to its handler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2554-2586`
pub fn PC_ReadDirective(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        // read the directive name
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"found # without name".as_ptr() as *mut c_char);
            return qfalse;
        }
        // directive name must be on the same line
        if token.linescrossed > 0 {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"found # at end of line".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        // if if is a name
        if token.r#type == TT_NAME {
            // find the precompiler directive
            let mut i = 0usize;
            while !directives[i].name.is_null() {
                if strcmp(directives[i].name, token.string.as_ptr()) == 0 {
                    return (directives[i].func)(bot, source);
                }
                i += 1;
            }
        }
        source_error!(
            bot,
            source,
            c"unknown precompiler directive %s".as_ptr() as *mut c_char,
            token.string.as_ptr(),
        );
        qfalse
    }
}

/// Raven `PC_ReadDollarDirective` — dispatch a `$`-directive to its handler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2655-2688`
pub fn PC_ReadDollarDirective(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        // read the directive name
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"found $ without name".as_ptr() as *mut c_char);
            return qfalse;
        }
        // directive name must be on the same line
        if token.linescrossed > 0 {
            PC_UnreadSourceToken(bot, source, &mut token);
            source_error!(
                bot,
                source,
                c"found $ at end of line".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        // if if is a name
        if token.r#type == TT_NAME {
            // find the precompiler directive
            let mut i = 0usize;
            while !dollardirectives[i].name.is_null() {
                if strcmp(dollardirectives[i].name, token.string.as_ptr()) == 0 {
                    return (dollardirectives[i].func)(bot, source);
                }
                i += 1;
            }
        }
        PC_UnreadSourceToken(bot, source, &mut token);
        source_error!(
            bot,
            source,
            c"unknown precompiler directive %s".as_ptr() as *mut c_char,
            token.string.as_ptr(),
        );
        qfalse
    }
}

/// Raven `PC_DollarDirective_evalint` — `$evalint(expr)`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2593-2614`
pub fn PC_DollarDirective_evalint(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut value: c_long = 0;
        let mut token: token_t = core::mem::zeroed();

        if PC_DollarEvaluate(bot, source, &mut value, core::ptr::null_mut(), qtrue) == 0 {
            return qfalse;
        }
        //
        token.line = (*(*source).scriptstack).line;
        token.whitespace_p = (*(*source).scriptstack).script_p;
        token.endwhitespace_p = (*(*source).scriptstack).script_p;
        token.linescrossed = 0;
        sprintf(
            token.string.as_mut_ptr(),
            c"%d".as_ptr(),
            abs(value as c_int),
        );
        token.r#type = TT_NUMBER;
        token.subtype = TT_INTEGER | TT_LONG | TT_DECIMAL;
        // #ifdef NUMBERVALUE (not defined) omitted.
        PC_UnreadSourceToken(bot, source, &mut token);
        if value < 0 {
            UnreadSignToken(bot, source);
        }
        qtrue
    }
}

/// Raven `PC_DollarDirective_evalfloat` — `$evalfloat(expr)`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2621-2641`
pub fn PC_DollarDirective_evalfloat(bot: &mut BotLib, source: *mut source_t) -> c_int {
    unsafe {
        let mut value: f64 = 0.0;
        let mut token: token_t = core::mem::zeroed();

        if PC_DollarEvaluate(bot, source, core::ptr::null_mut(), &mut value, qfalse) == 0 {
            return qfalse;
        }
        token.line = (*(*source).scriptstack).line;
        token.whitespace_p = (*(*source).scriptstack).script_p;
        token.endwhitespace_p = (*(*source).scriptstack).script_p;
        token.linescrossed = 0;
        sprintf(token.string.as_mut_ptr(), c"%1.2f".as_ptr(), value.abs());
        token.r#type = TT_NUMBER;
        token.subtype = TT_FLOAT | TT_LONG | TT_DECIMAL;
        // #ifdef NUMBERVALUE (not defined) omitted.
        PC_UnreadSourceToken(bot, source, &mut token);
        if value < 0.0 {
            UnreadSignToken(bot, source);
        }
        qtrue
    }
}

/// Raven `PC_Evaluate` — evaluate a `#if`/`#elif` expression line.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2184-2276`
pub fn PC_Evaluate(
    bot: &mut BotLib,
    source: *mut source_t,
    intvalue: *mut c_long,
    floatvalue: *mut f64,
    integer: c_int,
) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();
        let mut firsttoken: *mut token_t;
        let mut lasttoken: *mut token_t;
        let mut t: *mut token_t;
        let mut nexttoken: *mut token_t;
        let mut define: *mut define_t;
        let mut defined: c_int = qfalse;

        if !intvalue.is_null() {
            *intvalue = 0;
        }
        if !floatvalue.is_null() {
            *floatvalue = 0.0;
        }
        //
        if PC_ReadLine(bot, source, &mut token) == 0 {
            source_error!(
                bot,
                source,
                c"no value after #if/#elif".as_ptr() as *mut c_char
            );
            return qfalse;
        }
        firsttoken = core::ptr::null_mut();
        lasttoken = core::ptr::null_mut();
        loop {
            // if the token is a name
            if token.r#type == TT_NAME {
                if defined != 0 {
                    defined = qfalse;
                    t = PC_CopyToken(bot, &mut token);
                    (*t).next = core::ptr::null_mut();
                    if !lasttoken.is_null() {
                        (*lasttoken).next = t;
                    } else {
                        firsttoken = t;
                    }
                    lasttoken = t;
                } else if strcmp(token.string.as_ptr(), c"defined".as_ptr()) == 0 {
                    defined = qtrue;
                    t = PC_CopyToken(bot, &mut token);
                    (*t).next = core::ptr::null_mut();
                    if !lasttoken.is_null() {
                        (*lasttoken).next = t;
                    } else {
                        firsttoken = t;
                    }
                    lasttoken = t;
                } else {
                    // then it must be a define (#if DEFINEHASHING, live)
                    define = PC_FindHashedDefine((*source).definehash, token.string.as_mut_ptr());
                    if define.is_null() {
                        source_error!(
                            bot,
                            source,
                            c"can't evaluate %s, not defined".as_ptr() as *mut c_char,
                            token.string.as_ptr(),
                        );
                        return qfalse;
                    }
                    if PC_ExpandDefineIntoSource(bot, source, &mut token, define) == 0 {
                        return qfalse;
                    }
                }
            }
            // if the token is a number or a punctuation
            else if token.r#type == TT_NUMBER || token.r#type == TT_PUNCTUATION {
                t = PC_CopyToken(bot, &mut token);
                (*t).next = core::ptr::null_mut();
                if !lasttoken.is_null() {
                    (*lasttoken).next = t;
                } else {
                    firsttoken = t;
                }
                lasttoken = t;
            } else {
                source_error!(
                    bot,
                    source,
                    c"can't evaluate %s".as_ptr() as *mut c_char,
                    token.string.as_ptr(),
                );
                return qfalse;
            }
            if PC_ReadLine(bot, source, &mut token) == 0 {
                break;
            }
        }
        //
        if PC_EvaluateTokens(bot, source, firsttoken, intvalue, floatvalue, integer) == 0 {
            return qfalse;
        }
        //
        t = firsttoken;
        while !t.is_null() {
            nexttoken = (*t).next;
            PC_FreeToken(bot, t);
            t = nexttoken;
        }
        //
        qtrue
    }
}

/// Raven `PC_DollarEvaluate` — evaluate a `$evalint`/`$evalfloat(expr)` body.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2283-2384`
pub fn PC_DollarEvaluate(
    bot: &mut BotLib,
    source: *mut source_t,
    intvalue: *mut c_long,
    floatvalue: *mut f64,
    integer: c_int,
) -> c_int {
    unsafe {
        let mut indent: c_int;
        let mut defined: c_int = qfalse;
        let mut token: token_t = core::mem::zeroed();
        let mut firsttoken: *mut token_t;
        let mut lasttoken: *mut token_t;
        let mut t: *mut token_t;
        let mut nexttoken: *mut token_t;
        let mut define: *mut define_t;

        if !intvalue.is_null() {
            *intvalue = 0;
        }
        if !floatvalue.is_null() {
            *floatvalue = 0.0;
        }
        //
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(
                bot,
                source,
                c"no leading ( after $evalint/$evalfloat".as_ptr() as *mut c_char,
            );
            return qfalse;
        }
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            source_error!(bot, source, c"nothing to evaluate".as_ptr() as *mut c_char);
            return qfalse;
        }
        indent = 1;
        firsttoken = core::ptr::null_mut();
        lasttoken = core::ptr::null_mut();
        loop {
            // if the token is a name
            if token.r#type == TT_NAME {
                if defined != 0 {
                    defined = qfalse;
                    t = PC_CopyToken(bot, &mut token);
                    (*t).next = core::ptr::null_mut();
                    if !lasttoken.is_null() {
                        (*lasttoken).next = t;
                    } else {
                        firsttoken = t;
                    }
                    lasttoken = t;
                } else if strcmp(token.string.as_ptr(), c"defined".as_ptr()) == 0 {
                    defined = qtrue;
                    t = PC_CopyToken(bot, &mut token);
                    (*t).next = core::ptr::null_mut();
                    if !lasttoken.is_null() {
                        (*lasttoken).next = t;
                    } else {
                        firsttoken = t;
                    }
                    lasttoken = t;
                } else {
                    // then it must be a define (#if DEFINEHASHING, live)
                    define = PC_FindHashedDefine((*source).definehash, token.string.as_mut_ptr());
                    if define.is_null() {
                        source_error!(
                            bot,
                            source,
                            c"can't evaluate %s, not defined".as_ptr() as *mut c_char,
                            token.string.as_ptr(),
                        );
                        return qfalse;
                    }
                    if PC_ExpandDefineIntoSource(bot, source, &mut token, define) == 0 {
                        return qfalse;
                    }
                }
            }
            // if the token is a number or a punctuation
            else if token.r#type == TT_NUMBER || token.r#type == TT_PUNCTUATION {
                if token.string[0] == b'(' as c_char {
                    indent += 1;
                } else if token.string[0] == b')' as c_char {
                    indent -= 1;
                }
                if indent <= 0 {
                    break;
                }
                t = PC_CopyToken(bot, &mut token);
                (*t).next = core::ptr::null_mut();
                if !lasttoken.is_null() {
                    (*lasttoken).next = t;
                } else {
                    firsttoken = t;
                }
                lasttoken = t;
            } else {
                source_error!(
                    bot,
                    source,
                    c"can't evaluate %s".as_ptr() as *mut c_char,
                    token.string.as_ptr(),
                );
                return qfalse;
            }
            if PC_ReadSourceToken(bot, source, &mut token) == 0 {
                break;
            }
        }
        //
        if PC_EvaluateTokens(bot, source, firsttoken, intvalue, floatvalue, integer) == 0 {
            return qfalse;
        }
        //
        t = firsttoken;
        while !t.is_null() {
            nexttoken = (*t).next;
            PC_FreeToken(bot, t);
            t = nexttoken;
        }
        //
        qtrue
    }
}

/// Raven `PC_ReadToken` — read a fully-resolved token (directives expanded,
/// defines applied, adjacent strings concatenated).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2749-2837`
pub fn PC_ReadToken(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) -> c_int {
    unsafe {
        let mut define: *mut define_t;

        loop {
            if PC_ReadSourceToken(bot, source, token) == 0 {
                return qfalse;
            }
            // check for precompiler directives
            if (*token).r#type == TT_PUNCTUATION && (*token).string[0] == b'@' as c_char {
                // It is a StringEd key
                let mut holdString2 = [0 as c_char; MAX_TOKEN];

                PC_ReadSourceToken(bot, source, token);
                let holdString: *mut c_char = (*token).string.as_mut_ptr().add(1);
                Com_Memcpy(
                    holdString2.as_mut_ptr().cast(),
                    (*token).string.as_ptr().cast(),
                    core::mem::size_of_val(&(*token).string),
                );
                Com_Memcpy(
                    holdString.cast(),
                    holdString2.as_ptr().cast(),
                    core::mem::size_of_val(&(*token).string),
                );
                (*token).string[0] = b'@' as c_char;
                return qtrue;
            }

            if (*token).r#type == TT_PUNCTUATION && (*token).string[0] == b'#' as c_char {
                // #ifdef QUAKEC (not defined) -> block always runs
                // read the precompiler directive
                if PC_ReadDirective(bot, source) == 0 {
                    return qfalse;
                }
                continue;
            }
            if (*token).r#type == TT_PUNCTUATION && (*token).string[0] == b'$' as c_char {
                // #ifdef QUAKEC (not defined) -> block always runs
                // read the precompiler directive
                if PC_ReadDollarDirective(bot, source) == 0 {
                    return qfalse;
                }
                continue;
            }
            // recursively concatenate strings that are behind each other still resolving defines
            if (*token).r#type == TT_STRING {
                let mut newtoken: token_t = core::mem::zeroed();
                if PC_ReadToken(bot, source, &mut newtoken) != 0 {
                    if newtoken.r#type == TT_STRING {
                        let end = strlen((*token).string.as_ptr()) - 1;
                        (*token).string[end] = b'\0' as c_char;
                        if strlen((*token).string.as_ptr())
                            + strlen(newtoken.string.as_ptr().add(1))
                            + 1
                            >= MAX_TOKEN
                        {
                            source_error!(
                                bot,
                                source,
                                c"string longer than MAX_TOKEN %d\n".as_ptr() as *mut c_char,
                                MAX_TOKEN as c_int,
                            );
                            return qfalse;
                        }
                        strcat(
                            (*token).string.as_mut_ptr(),
                            newtoken.string.as_ptr().add(1),
                        );
                    } else {
                        PC_UnreadToken(bot, source, &mut newtoken);
                    }
                }
            }
            // if skipping source because of conditional compilation
            if (*source).skip != 0 {
                continue;
            }
            // if the token is a name
            if (*token).r#type == TT_NAME {
                // check if the name is a define macro (#if DEFINEHASHING, live)
                define = PC_FindHashedDefine((*source).definehash, (*token).string.as_mut_ptr());
                // if it is a define macro
                if !define.is_null() {
                    // expand the defined macro
                    if PC_ExpandDefineIntoSource(bot, source, token, define) == 0 {
                        return qfalse;
                    }
                    continue;
                }
            }
            // copy token for unreading
            Com_Memcpy(
                (&mut (*source).token as *mut token_t).cast(),
                token.cast(),
                core::mem::size_of::<token_t>(),
            );
            // found a token
            return qtrue;
        }
    }
}

/// Raven `PC_Directive_elif` — handle `#elif`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2391-2406`
pub fn PC_Directive_elif(bot: &mut BotLib, source: *mut source_t) -> c_int {
    let mut value: c_long = 0;
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(bot, source, &mut r#type, &mut skip);
    if r#type == 0 || r#type == INDENT_ELSE {
        unsafe { source_error!(bot, source, c"misplaced #elif".as_ptr() as *mut c_char) };
        return qfalse;
    }
    if PC_Evaluate(bot, source, &mut value, core::ptr::null_mut(), qtrue) == 0 {
        return qfalse;
    }
    skip = (value == 0) as c_int;
    PC_PushIndent(bot, source, INDENT_ELIF, skip);
    qtrue
}

/// Raven `PC_Directive_if` — handle `#if`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2413-2422`
pub fn PC_Directive_if(bot: &mut BotLib, source: *mut source_t) -> c_int {
    let mut value: c_long = 0;
    let skip: c_int;

    if PC_Evaluate(bot, source, &mut value, core::ptr::null_mut(), qtrue) == 0 {
        return qfalse;
    }
    skip = (value == 0) as c_int;
    PC_PushIndent(bot, source, INDENT_IF, skip);
    qtrue
}

/// Raven `PC_ExpectTokenString` — read the next token and require it to equal
/// `string`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2844-2860`
pub fn PC_ExpectTokenString(bot: &mut BotLib, source: *mut source_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        if PC_ReadToken(bot, source, &mut token) == 0 {
            source_error!(
                bot,
                source,
                c"couldn't find expected %s".as_ptr() as *mut c_char,
                string,
            );
            return qfalse;
        }

        if strcmp(token.string.as_ptr(), string) != 0 {
            source_error!(
                bot,
                source,
                c"expected %s, found %s".as_ptr() as *mut c_char,
                string,
                token.string.as_ptr(),
            );
            return qfalse;
        }
        qtrue
    }
}

/// Raven `PC_ExpectTokenType` — read the next token and require a matching
/// type/subtype.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2867-2913`
pub fn PC_ExpectTokenType(
    bot: &mut BotLib,
    source: *mut source_t,
    r#type: c_int,
    subtype: c_int,
    token: *mut token_t,
) -> c_int {
    unsafe {
        let mut str = [0 as c_char; MAX_TOKEN];

        if PC_ReadToken(bot, source, token) == 0 {
            source_error!(
                bot,
                source,
                c"couldn't read expected token".as_ptr() as *mut c_char
            );
            return qfalse;
        }

        if (*token).r#type != r#type {
            strcpy(str.as_mut_ptr(), c"".as_ptr());
            if r#type == TT_STRING {
                strcpy(str.as_mut_ptr(), c"string".as_ptr());
            }
            if r#type == TT_LITERAL {
                strcpy(str.as_mut_ptr(), c"literal".as_ptr());
            }
            if r#type == TT_NUMBER {
                strcpy(str.as_mut_ptr(), c"number".as_ptr());
            }
            if r#type == TT_NAME {
                strcpy(str.as_mut_ptr(), c"name".as_ptr());
            }
            if r#type == TT_PUNCTUATION {
                strcpy(str.as_mut_ptr(), c"punctuation".as_ptr());
            }
            source_error!(
                bot,
                source,
                c"expected a %s, found %s".as_ptr() as *mut c_char,
                str.as_ptr(),
                (*token).string.as_ptr(),
            );
            return qfalse;
        }
        if (*token).r#type == TT_NUMBER {
            if ((*token).subtype & subtype) != subtype {
                if subtype & TT_DECIMAL != 0 {
                    strcpy(str.as_mut_ptr(), c"decimal".as_ptr());
                }
                if subtype & TT_HEX != 0 {
                    strcpy(str.as_mut_ptr(), c"hex".as_ptr());
                }
                if subtype & TT_OCTAL != 0 {
                    strcpy(str.as_mut_ptr(), c"octal".as_ptr());
                }
                if subtype & TT_BINARY != 0 {
                    strcpy(str.as_mut_ptr(), c"binary".as_ptr());
                }
                if subtype & TT_LONG != 0 {
                    strcat(str.as_mut_ptr(), c" long".as_ptr());
                }
                if subtype & TT_UNSIGNED != 0 {
                    strcat(str.as_mut_ptr(), c" unsigned".as_ptr());
                }
                if subtype & TT_FLOAT != 0 {
                    strcat(str.as_mut_ptr(), c" float".as_ptr());
                }
                if subtype & TT_INTEGER != 0 {
                    strcat(str.as_mut_ptr(), c" integer".as_ptr());
                }
                source_error!(
                    bot,
                    source,
                    c"expected %s, found %s".as_ptr() as *mut c_char,
                    str.as_ptr(),
                    (*token).string.as_ptr(),
                );
                return qfalse;
            }
        } else if (*token).r#type == TT_PUNCTUATION {
            if (*token).subtype != subtype {
                source_error!(
                    bot,
                    source,
                    c"found %s".as_ptr() as *mut c_char,
                    (*token).string.as_ptr(),
                );
                return qfalse;
            }
        }
        qtrue
    }
}

/// Raven `PC_ExpectAnyToken` — read the next token, erroring only at EOF.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2920-2931`
pub fn PC_ExpectAnyToken(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) -> c_int {
    if PC_ReadToken(bot, source, token) == 0 {
        unsafe {
            source_error!(
                bot,
                source,
                c"couldn't read expected token".as_ptr() as *mut c_char
            )
        };
        qfalse
    } else {
        qtrue
    }
}

/// Raven `PC_CheckTokenString` — read the next token; if it equals `string`
/// consume it, else unread and fail.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2938-2948`
pub fn PC_CheckTokenString(bot: &mut BotLib, source: *mut source_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut tok: token_t = core::mem::zeroed();

        if PC_ReadToken(bot, source, &mut tok) == 0 {
            return qfalse;
        }
        // if the token is available
        if strcmp(tok.string.as_ptr(), string) == 0 {
            return qtrue;
        }
        //
        PC_UnreadSourceToken(bot, source, &mut tok);
        qfalse
    }
}

/// Raven `PC_CheckTokenType` — read the next token; if type/subtype match copy
/// it out, else unread and fail.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2955-2970`
pub fn PC_CheckTokenType(
    bot: &mut BotLib,
    source: *mut source_t,
    r#type: c_int,
    subtype: c_int,
    token: *mut token_t,
) -> c_int {
    unsafe {
        let mut tok: token_t = core::mem::zeroed();

        if PC_ReadToken(bot, source, &mut tok) == 0 {
            return qfalse;
        }
        // if the type matches
        if tok.r#type == r#type && (tok.subtype & subtype) == subtype {
            Com_Memcpy(
                token.cast(),
                (&tok as *const token_t).cast(),
                core::mem::size_of::<token_t>(),
            );
            return qtrue;
        }
        //
        PC_UnreadSourceToken(bot, source, &mut tok);
        qfalse
    }
}

/// Raven `PC_SkipUntilString` — read tokens until `string` is found.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2977-2986`
pub fn PC_SkipUntilString(bot: &mut BotLib, source: *mut source_t, string: *mut c_char) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        while PC_ReadToken(bot, source, &mut token) != 0 {
            if strcmp(token.string.as_ptr(), string) == 0 {
                return qtrue;
            }
        }
        qfalse
    }
}

/// Raven `PC_UnreadLastToken` — push the source's last-read token back.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2993-2996`
pub fn PC_UnreadLastToken(bot: &mut BotLib, source: *mut source_t) {
    unsafe {
        PC_UnreadSourceToken(bot, source, &mut (*source).token);
    }
}

/// Raven `PC_UnreadToken` — push a token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3003-3006`
pub fn PC_UnreadToken(bot: &mut BotLib, source: *mut source_t, token: *mut token_t) {
    PC_UnreadSourceToken(bot, source, token);
}

/// Raven `PC_SetIncludePath` — set a source's include path, ensuring a trailing
/// separator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3013-3022`
pub fn PC_SetIncludePath(source: *mut source_t, path: *mut c_char) {
    unsafe {
        strncpy((*source).includepath.as_mut_ptr(), path, MAX_PATH);
        // add trailing path seperator
        let n = strlen((*source).includepath.as_ptr());
        if (*source).includepath[n - 1] != b'\\' as c_char
            && (*source).includepath[n - 1] != b'/' as c_char
        {
            strcat(
                (*source).includepath.as_mut_ptr(),
                PATHSEPERATOR_STR.as_ptr() as *const c_char,
            );
        }
    }
}

/// Raven `PC_SetPunctuations` — install a punctuation table on a source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3029-3032`
pub fn PC_SetPunctuations(source: *mut source_t, p: *mut punctuation_t) {
    unsafe {
        (*source).punctuations = p;
    }
}

/// Raven `LoadSourceFile` — open a preprocessor source over a script file.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3039-3073`
pub fn LoadSourceFile(bot: &mut BotLib, filename: *const c_char) -> *mut source_t {
    unsafe {
        PC_InitTokenHeap();

        // #if DEFINEHASHING (live)
        if bot.globaldefines.is_null() {
            bot.globaldefines = GetClearedMemory(
                bot,
                (DEFINEHASHSIZE * core::mem::size_of::<*mut define_t>()) as c_ulong,
            ) as *mut *mut define_t;
        }

        let script: *mut script_t = LoadScriptFile(bot, filename);
        if script.is_null() {
            return core::ptr::null_mut();
        }

        (*script).next = core::ptr::null_mut();

        let source: *mut source_t =
            GetMemory(bot, core::mem::size_of::<source_t>() as c_ulong) as *mut source_t;
        Com_Memset(source.cast(), 0, core::mem::size_of::<source_t>());

        strncpy((*source).filename.as_mut_ptr(), filename, MAX_PATH);
        (*source).scriptstack = script;
        (*source).tokens = core::ptr::null_mut();
        (*source).defines = core::ptr::null_mut();
        (*source).indentstack = core::ptr::null_mut();
        (*source).skip = 0;

        // #if DEFINEHASHING (live)
        (*source).definehash = GetClearedMemory(
            bot,
            (DEFINEHASHSIZE * core::mem::size_of::<*mut define_t>()) as c_ulong,
        ) as *mut *mut define_t;
        PC_AddGlobalDefinesToSource(bot, source);
        source
    }
}

/// Raven `LoadSourceMemory` — open a preprocessor source over an in-memory
/// script.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3080-3106`
pub fn LoadSourceMemory(
    bot: &mut BotLib,
    ptr: *mut c_char,
    length: c_int,
    name: *mut c_char,
) -> *mut source_t {
    unsafe {
        PC_InitTokenHeap();

        let script: *mut script_t = LoadScriptMemory(bot, ptr, length, name);
        if script.is_null() {
            return core::ptr::null_mut();
        }
        (*script).next = core::ptr::null_mut();

        let source: *mut source_t =
            GetMemory(bot, core::mem::size_of::<source_t>() as c_ulong) as *mut source_t;
        Com_Memset(source.cast(), 0, core::mem::size_of::<source_t>());

        strncpy((*source).filename.as_mut_ptr(), name, MAX_PATH);
        (*source).scriptstack = script;
        (*source).tokens = core::ptr::null_mut();
        (*source).defines = core::ptr::null_mut();
        (*source).indentstack = core::ptr::null_mut();
        (*source).skip = 0;

        // #if DEFINEHASHING (live)
        (*source).definehash = GetClearedMemory(
            bot,
            (DEFINEHASHSIZE * core::mem::size_of::<*mut define_t>()) as c_ulong,
        ) as *mut *mut define_t;
        PC_AddGlobalDefinesToSource(bot, source);
        source
    }
}

/// Raven `PC_FreeSourceHandle` — free the source at `handle` in the handle
/// table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3214-3224`
pub fn PC_FreeSourceHandle(bot: &mut BotLib, handle: c_int) -> c_int {
    if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
        return qfalse;
    }
    if bot.sourceFiles[handle as usize].is_null() {
        return qfalse;
    }

    FreeSource(bot, bot.sourceFiles[handle as usize]);
    bot.sourceFiles[handle as usize] = core::ptr::null_mut();
    qtrue
}

/// Raven `PC_LoadSourceHandle` — load a source file and store it in the handle
/// table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3189-3207`
pub fn PC_LoadSourceHandle(bot: &mut BotLib, filename: *const c_char) -> c_int {
    let mut i: c_int = 1;
    while i < MAX_SOURCEFILES as c_int {
        if bot.sourceFiles[i as usize].is_null() {
            break;
        }
        i += 1;
    }
    if i >= MAX_SOURCEFILES as c_int {
        return 0;
    }
    PS_SetBaseFolder(bot, c"".as_ptr() as *mut c_char);
    let source: *mut source_t = LoadSourceFile(bot, filename);
    if source.is_null() {
        return 0;
    }
    bot.sourceFiles[i as usize] = source;
    i
}

/// Raven `PC_SetBaseFolder` — forward to the script tokenizer's base folder.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3300-3303`
pub fn PC_SetBaseFolder(bot: &mut BotLib, path: *mut c_char) {
    PS_SetBaseFolder(bot, path);
}

/// Raven `PC_SourceFileAndLine` — report the file/line of an open source
/// handle.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3280-3293`
pub fn PC_SourceFileAndLine(
    bot: &mut BotLib,
    handle: c_int,
    filename: *mut c_char,
    line: *mut c_int,
) -> c_int {
    unsafe {
        if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
            return qfalse;
        }
        if bot.sourceFiles[handle as usize].is_null() {
            return qfalse;
        }

        strcpy(
            filename,
            (*bot.sourceFiles[handle as usize]).filename.as_ptr(),
        );
        if !(*bot.sourceFiles[handle as usize]).scriptstack.is_null() {
            *line = (*(*bot.sourceFiles[handle as usize]).scriptstack).line;
        } else {
            *line = 0;
        }
        qtrue
    }
}

/// Raven `PC_CheckOpenSourceHandles` — warn about any source left open in the
/// precompiler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3310-3323`
pub fn PC_CheckOpenSourceHandles(bot: &mut BotLib) {
    unsafe {
        for i in 1..MAX_SOURCEFILES {
            if !bot.sourceFiles[i].is_null() {
                // #ifdef BOTLIB (defined)
                (bot.botimport.Print.unwrap())(
                    PRT_ERROR,
                    c"file %s still open in precompiler\n".as_ptr() as *mut c_char,
                    (*(*bot.sourceFiles[i]).scriptstack).filename.as_ptr(),
                );
            }
        }
    }
}

/// Raven `PC_LoadGlobalDefines` — load a file purely to register its defines
/// globally.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3226-3245`
pub fn PC_LoadGlobalDefines(bot: &mut BotLib, filename: *const c_char) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        let handle = PC_LoadSourceHandle(bot, filename);
        if handle < 1 {
            return qfalse;
        }

        bot.addGlobalDefine = qtrue;

        // Read all the token files which will add the defines globally
        while PC_ReadToken(bot, bot.sourceFiles[handle as usize], &mut token) != 0 {}

        bot.addGlobalDefine = qfalse;

        PC_FreeSourceHandle(bot, handle);

        qtrue
    }
}

/// Raven `PC_ReadTokenHandle` — read a token from an open source handle into a
/// public `pc_token_t`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3253-3273`
pub fn PC_ReadTokenHandle(bot: &mut BotLib, handle: c_int, pc_token: *mut pc_token_t) -> c_int {
    unsafe {
        let mut token: token_t = core::mem::zeroed();

        if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
            return 0;
        }
        if bot.sourceFiles[handle as usize].is_null() {
            return 0;
        }

        let ret = PC_ReadToken(bot, bot.sourceFiles[handle as usize], &mut token);
        strcpy((*pc_token).string.as_mut_ptr(), token.string.as_ptr());
        (*pc_token).type_ = token.r#type;
        (*pc_token).subtype = token.subtype;
        (*pc_token).intvalue = token.intvalue as c_int;
        (*pc_token).floatvalue = token.floatvalue as f32;
        if (*pc_token).type_ == TT_STRING && (*pc_token).string[0] != b'@' as c_char {
            StripDoubleQuotes((*pc_token).string.as_mut_ptr());
        }

        ret
    }
}
