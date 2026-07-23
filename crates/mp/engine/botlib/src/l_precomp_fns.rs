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
//! Idiomatic redesign (porting-rules §F17): the preprocessor runs on the owned
//! `Source` (a `Vec<Script>` script stack, a `VecDeque<Token>` unread-stack, a
//! `Vec<Define>` arena indexed by `definehash: Vec<Vec<usize>>` prepend-buckets)
//! and the owned `Script`/`Token` shapes, not Raven's malloc'd
//! `source_t`/`define_t`/`token_t`. Threading conventions:
//!
//! - **Defines are an arena + index buckets.** `PC_AddDefineToHash` prepends an
//!   arena index into `definehash[hash]`; `PC_FindHashedDefine` walks the bucket
//!   front-to-back, first name match wins (the RULED faithful hash-chain shape —
//!   duplicate-named globals coexist, so lookup stays chain-order-dependent).
//!   Removing (`#undef`) unlinks the index from its bucket; the arena slot is
//!   left dead (never compacted, so indices stay stable).
//! - **Globals are a flat arena on `BotLib`.** `globaldefines: Vec<Define>` is
//!   append-only and only ever iterated (never hash-looked-up), so it needs no
//!   bucket table. `PC_AddGlobalDefinesToSource` copies globals into a source by
//!   iterating that arena in reverse and prepending each copy — reproducing
//!   Raven's per-bucket newest-first walk + prepend (double reversal → the
//!   first-defined global wins in the source), byte-for-byte on lookups.
//! - **The live source is threaded, never reached.** Every `PC_*` takes
//!   `bot: &mut BotLib` plus `source: &mut Source` (or `&Source`) as disjoint
//!   borrows: consumers own the `Source` as a local; the seam adapters take it
//!   out of `bot.sourceFiles[handle]` for the call and put it back.
//! - **Errors carry text, not a C format.** `SourceError`/`SourceWarning` take a
//!   pre-formatted `&str` (mirroring `ScriptError`), reproducing the oracle's
//!   `"file …, line …: …\n"` line byte-for-byte.
//! - **Token lists are `Vec<Token>`.** `PC_CopyToken`/`PC_FreeToken` and the
//!   `numtokens` accounting dissolve into `.clone()` + drop; `PC_InitTokenHeap`
//!   (a no-op in Raven) is dropped.
//!
//! Source: `oracle/codemp/botlib/l_precomp.cpp`
//!
//! `DEFINEHASHING` (`l_precomp.cpp:83`) and `BOTLIB` are compile-time-defined in
//! this build; `MEQCC`/`BSPC`/`QUAKE`/`QUAKEC`/`SCREWUP`/`NUMBERVALUE`/
//! `DEBUG_EVAL` are not — the corresponding dead `#if`/`#else` arms are dropped
//! per §C10.

use core::ffi::{c_char, c_int, c_long};
use std::ffi::{CStr, CString};

use libc::time;

use crate::l_precomp::builtin_defines::{
    BUILTIN_DATE, BUILTIN_FILE, BUILTIN_LINE, BUILTIN_STDC, BUILTIN_TIME,
};
use crate::l_precomp::define_flags::{DEFINE_FIXED, DEFINE_GLOBAL};
use crate::l_precomp::define_s::Define;
use crate::l_precomp::directive_s::Directive;
use crate::l_precomp::indent_s::Indent;
use crate::l_precomp::indent_type::{
    INDENT_ELIF, INDENT_ELSE, INDENT_IF, INDENT_IFDEF, INDENT_IFNDEF,
};
use crate::l_precomp::operator_s::operator_t;
use crate::l_precomp::path_seperator_consts::{PATHSEPERATOR_CHAR, PATHSEPERATOR_STR};
use crate::l_precomp::precomp_consts::{
    DEFINEHASHSIZE, MAX_DEFINEPARMS, MAX_OPERATORS, MAX_SOURCEFILES, MAX_VALUES,
};
use crate::l_precomp::source_s::Source;
use crate::l_precomp::value_s::value_t;
use crate::l_script::consts::{
    MAX_TOKEN, P_ADD, P_BIN_AND, P_BIN_NOT, P_BIN_OR, P_BIN_XOR, P_COLON, P_DEC, P_DIV, P_INC,
    P_LOGIC_AND, P_LOGIC_EQ, P_LOGIC_GEQ, P_LOGIC_GREATER, P_LOGIC_LEQ, P_LOGIC_LESS, P_LOGIC_NOT,
    P_LOGIC_OR, P_LOGIC_UNEQ, P_LSHIFT, P_MOD, P_MUL, P_PARENTHESESCLOSE, P_PARENTHESESOPEN,
    P_QUESTIONMARK, P_RSHIFT, P_SUB, TT_BINARY, TT_DECIMAL, TT_FLOAT, TT_HEX, TT_INTEGER,
    TT_LITERAL, TT_LONG, TT_NAME, TT_NUMBER, TT_OCTAL, TT_PUNCTUATION, TT_STRING, TT_UNSIGNED,
};
use crate::l_script::script_s::Script;
use crate::l_script::token_s::Token;
use crate::l_script_fns::{
    EndOfScript, LoadScriptFile, LoadScriptMemory, PS_ReadToken, PS_SetBaseFolder, StripDoubleQuotes,
};
use crate::BotLib;

use mp_qshared::common::mp::botlib::print_type::{PRT_ERROR, PRT_WARNING};
use mp_qshared::shared::{pc_token_t, qfalse, qtrue, MAX_TOKENLENGTH};

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

/// Raven `SourceError` — print a preprocessor error tagged with the current
/// script file and line.
///
/// The oracle `vsprintf`s the message, then `Print`s `"file %s, line %d: %s\n"`;
/// the pre-formatted `text` is composed at the call site and handed to `Print`
/// as one `%s` argument, yielding byte-identical output. File/line come from the
/// current (top) script; an empty script stack (never reached while a source is
/// live — Raven would deref NULL, §F19) falls back to `("", 0)`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:117-134`
pub fn SourceError(bot: &mut BotLib, source: &Source, text: &str) {
    // #ifdef BOTLIB (defined)
    let (filename, line) = match source.scriptstack.last() {
        Some(s) => (s.filename.as_str(), s.line),
        None => ("", 0),
    };
    let msg = CString::new(format!("file {}, line {}: {}\n", filename, line, text)).unwrap_or_default();
    unsafe {
        (bot.botimport.Print.unwrap())(PRT_ERROR, c"%s".as_ptr() as *mut c_char, msg.as_ptr());
    }
}

/// Raven `SourceWarning` — print a preprocessor warning tagged with the current
/// script file and line.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:141-158`
pub fn SourceWarning(bot: &mut BotLib, source: &Source, text: &str) {
    // #ifdef BOTLIB (defined)
    let (filename, line) = match source.scriptstack.last() {
        Some(s) => (s.filename.as_str(), s.line),
        None => ("", 0),
    };
    let msg = CString::new(format!("file {}, line {}: {}\n", filename, line, text)).unwrap_or_default();
    unsafe {
        (bot.botimport.Print.unwrap())(PRT_WARNING, c"%s".as_ptr() as *mut c_char, msg.as_ptr());
    }
}

/// Raven `PC_PushIndent` — push an `#if`/`#ifdef` indent onto the source's
/// indent stack.
///
/// The malloc'd `indent_t` becomes a pushed `Indent`; `indent->script`
/// (Raven's `script_t*` back-pointer) becomes the current top-of-stack index.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:165-176`
pub fn PC_PushIndent(source: &mut Source, r#type: c_int, skip: c_int) {
    let indent = Indent {
        type_: r#type,
        script: source.scriptstack.len() - 1,
        skip: (skip != 0) as c_int,
    };
    source.skip += indent.skip;
    source.indentstack.push(indent);
}

/// Raven `PC_PopIndent` — pop the top indent of the current script, reporting
/// its type and skip flag.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:183-201`
pub fn PC_PopIndent(source: &mut Source, r#type: &mut c_int, skip: &mut c_int) {
    *r#type = 0;
    *skip = 0;

    let indent = match source.indentstack.last().copied() {
        Some(i) => i,
        None => return,
    };
    // must be an indent from the current script
    if indent.script != source.scriptstack.len() - 1 {
        return;
    }
    *r#type = indent.type_;
    *skip = indent.skip;
    source.indentstack.pop();
    source.skip -= indent.skip;
}

/// Raven `PC_PushScript` — push a script onto the source's script stack,
/// erroring on recursive inclusion.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:208-223`
pub fn PC_PushScript(bot: &mut BotLib, source: &mut Source, script: Script) {
    for s in &source.scriptstack {
        if s.filename.eq_ignore_ascii_case(&script.filename) {
            SourceError(
                bot,
                source,
                &format!("{} recursively included", script.filename),
            );
            // Raven leaks the script here; the owned `script` is dropped.
            return;
        }
    }
    // push the script on the script stack
    source.scriptstack.push(script);
}

/// Raven `PC_StringizeTokens` — build a `"..."` string token from a token list.
///
/// Raven's `PC_StringizeTokens` always returns success (`qtrue`), so the built
/// `Token` is returned directly and the callers' dead error arm is dropped. The
/// per-`strncat` `MAX_TOKEN` bound becomes a final truncation (§F19; the
/// stringizing operator is effectively never exercised by the bot files).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:450-465`
fn PC_StringizeTokens(tokens: &[Token]) -> Token {
    let mut token = Token::default();
    token.type_ = TT_STRING;
    token.string.push('"');
    for t in tokens {
        token.string.push_str(&t.string);
    }
    token.string.push('"');
    if token.string.len() >= MAX_TOKEN {
        token.string.truncate(MAX_TOKEN - 1);
    }
    token
}

/// Raven `PC_MergeTokens` — merge `t2` into `t1` for name/number/string pairs.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:472-491`
fn PC_MergeTokens(t1: &mut Token, t2: &Token) -> c_int {
    // merging of a name with a name or number
    if t1.type_ == TT_NAME && (t2.type_ == TT_NAME || t2.type_ == TT_NUMBER) {
        t1.string.push_str(&t2.string);
        return qtrue;
    }
    // merging of two strings
    if t1.type_ == TT_STRING && t2.type_ == TT_STRING {
        // remove trailing double quote
        t1.string.pop();
        // concat without leading double quote
        if !t2.string.is_empty() {
            t1.string.push_str(&t2.string[1..]);
        }
        return qtrue;
    }
    // Raven note: merging of two numbers of the same sub type is unhandled.
    qfalse
}

/// Raven `PC_NameHash` — hash a define name into the define hash-chain table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:539-552`
pub fn PC_NameHash(name: &str) -> c_int {
    let mut hash: c_int = 0;
    for (i, b) in name.bytes().enumerate() {
        // Raven promotes each (platform-signed) `char` to `int`; define names
        // are ASCII, so the `as c_char` cast is a formality that matches width.
        hash = hash.wrapping_add((b as c_char as c_int).wrapping_mul(119 + i as c_int));
    }
    (hash ^ (hash >> 10) ^ (hash >> 20)) & (DEFINEHASHSIZE as c_int - 1)
}

/// Raven `PC_FindDefineParm` — index of a define parameter by name, or -1.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:621-633`
fn PC_FindDefineParm(define: &Define, name: &str) -> c_int {
    for (i, p) in define.parms.iter().enumerate() {
        if p.string == name {
            return i as c_int;
        }
    }
    -1
}

/// Raven `PC_AddDefineToHash` — insert a fully-built define into its owning
/// arena and hash chain.
///
/// Redesigned (porting-rules §F17): the finished `Define` is moved into either
/// the source's arena (prepending its index into `definehash[hash]`) or — when
/// `addGlobalDefine` is set — `bot.globaldefines` (flagged `DEFINE_GLOBAL`). The
/// global arena needs no hash table: it is only ever iterated.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:559-578`
fn PC_AddDefineToHash(bot: &mut BotLib, source: &mut Source, mut define: Define) {
    if bot.addGlobalDefine == qtrue {
        define.flags |= DEFINE_GLOBAL;
        bot.globaldefines.push(define);
    } else {
        let hash = PC_NameHash(&define.name) as usize;
        let idx = source.defines.len();
        source.defines.push(define);
        // prepend index (newest first) — first-match-wins on lookup
        source.definehash[hash].insert(0, idx);
    }
}

/// Raven `PC_FindHashedDefine` — look up a define by name in a source's hash
/// chains, returning its arena index.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:585-596`
fn PC_FindHashedDefine(source: &Source, name: &str) -> Option<usize> {
    let hash = PC_NameHash(name) as usize;
    for &idx in &source.definehash[hash] {
        if source.defines[idx].name == name {
            return Some(idx);
        }
    }
    None
}

/// Raven `PC_AddGlobalDefine` — add a define string to the global list. With
/// `DEFINEHASHING` (=1) live, the body reduces to a success return.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1407-1418`
pub fn PC_AddGlobalDefine(string: *mut c_char) -> c_int {
    // Whole body is `#if !DEFINEHASHING` (dead); DEFINEHASHING=1.
    let _ = string;
    qtrue
}

/// Raven `PC_RemoveAllGlobalDefines` — free every global define.
///
/// Redesigned: `bot.globaldefines` is an owned `Vec<Define>`; clearing it drops
/// every global define (Raven walked the hash chains freeing each).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1447-1472`
pub fn PC_RemoveAllGlobalDefines(bot: &mut BotLib) {
    // #if DEFINEHASHING (live)
    bot.globaldefines.clear();
}

/// Raven `PC_AddGlobalDefinesToSource` — copy every global define into a source's
/// hash table.
///
/// Redesigned: iterate `bot.globaldefines` in reverse and prepend a clone of
/// each into the source's arena/buckets. Raven walks each hash bucket
/// newest-first (via `globalnext`) and prepends into the source, so the source
/// bucket ends up oldest-first and the first-defined global wins; reverse
/// iteration + prepend reproduces that ordering exactly (cross-bucket order is
/// irrelevant to lookups). The copies keep `DEFINE_GLOBAL`; dropping the source
/// drops them without touching the shared global arena.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1522-1551`
fn PC_AddGlobalDefinesToSource(bot: &mut BotLib, source: &mut Source) {
    // #if DEFINEHASHING (live)
    for define in bot.globaldefines.iter().rev() {
        let copy = define.clone();
        let hash = PC_NameHash(&copy.name) as usize;
        let idx = source.defines.len();
        source.defines.push(copy);
        source.definehash[hash].insert(0, idx);
    }
}

/// Raven `PC_AddBuiltinDefines` — register `__LINE__`/`__FILE__`/`__DATE__`/
/// `__TIME__` builtins on a source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:665-698`
pub fn PC_AddBuiltinDefines(bot: &mut BotLib, source: &mut Source) {
    // Raven's local `struct builtin { char *string; int mBuiltin; }` table.
    let builtins: [(&str, c_int); 4] = [
        ("__LINE__", BUILTIN_LINE),
        ("__FILE__", BUILTIN_FILE),
        ("__DATE__", BUILTIN_DATE),
        ("__TIME__", BUILTIN_TIME),
    ];
    let _ = BUILTIN_STDC;

    for (name, builtin) in builtins {
        let mut define = Define::default();
        define.name = name.to_string();
        define.flags |= DEFINE_FIXED;
        define.builtin = builtin;
        // add the define to the source (#if DEFINEHASHING, live)
        PC_AddDefineToHash(bot, source, define);
    }
}

/// Raven `PC_ExpandBuiltinDefine` — expand a builtin macro token into fresh
/// tokens.
///
/// Returns the expanded token list (empty for `BUILTIN_STDC`/default, matching
/// Raven's `firsttoken = lasttoken = NULL`); Raven always reports success.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:705-775`
fn PC_ExpandBuiltinDefine(
    bot: &mut BotLib,
    source: &mut Source,
    deftoken: &Token,
    define: &Define,
) -> Vec<Token> {
    let mut token = deftoken.clone();
    match define.builtin {
        BUILTIN_LINE => {
            token.string = format!("{}", deftoken.line);
            token.type_ = TT_NUMBER;
            token.subtype = TT_DECIMAL | TT_INTEGER;
            vec![token]
        }
        BUILTIN_FILE => {
            token.string = source
                .scriptstack
                .last()
                .map(|s| s.filename.clone())
                .unwrap_or_default();
            token.type_ = TT_NAME;
            token.subtype = token.string.len() as c_int;
            vec![token]
        }
        BUILTIN_DATE => {
            let curtime = ctime_buf(unsafe { time(core::ptr::null_mut()) });
            let bytes: Vec<u8> = curtime.iter().map(|&c| c as u8).collect();
            let mut s = String::from("\"");
            for &b in &bytes[4..11] {
                s.push(b as char);
            }
            for &b in &bytes[20..24] {
                s.push(b as char);
            }
            s.push('"');
            token.string = s;
            token.type_ = TT_NAME;
            token.subtype = token.string.len() as c_int;
            vec![token]
        }
        BUILTIN_TIME => {
            let curtime = ctime_buf(unsafe { time(core::ptr::null_mut()) });
            let bytes: Vec<u8> = curtime.iter().map(|&c| c as u8).collect();
            let mut s = String::from("\"");
            for &b in &bytes[11..19] {
                s.push(b as char);
            }
            s.push('"');
            token.string = s;
            token.type_ = TT_NAME;
            token.subtype = token.string.len() as c_int;
            vec![token]
        }
        // BUILTIN_STDC and default
        _ => Vec::new(),
    }
}

/// Raven `PC_ConvertPath` — collapse doubled separators and normalize to the
/// OS path separator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:940-963`
fn PC_ConvertPath(path: &mut String) {
    let bytes = path.as_bytes();
    // remove double path seperators
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if (c == b'\\' || c == b'/')
            && i + 1 < bytes.len()
            && (bytes[i + 1] == b'\\' || bytes[i + 1] == b'/')
        {
            // collapse: drop this separator, stay on the run
            i += 1;
            continue;
        }
        out.push(c);
        i += 1;
    }
    // set OS dependent path seperators
    for b in out.iter_mut() {
        if *b == b'/' || *b == b'\\' {
            *b = PATHSEPERATOR_CHAR;
        }
    }
    *path = String::from_utf8_lossy(&out).into_owned();
}

/// Raven `PC_WhiteSpaceBeforeToken` — true if the token has leading whitespace.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1086-1089`
fn PC_WhiteSpaceBeforeToken(token: &Token) -> c_int {
    matches!(token.whitespace_span, Some((b, e)) if e - b > 0) as c_int
}

/// Raven `PC_ClearTokenWhiteSpace` — zero a token's whitespace bookkeeping.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1096-1101`
fn PC_ClearTokenWhiteSpace(token: &mut Token) {
    token.whitespace_span = None;
    token.linescrossed = 0;
}

/// Raven `PC_OperatorPriority` — precedence of a `#if`/`#elif` operator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1668-1701`
fn PC_OperatorPriority(op: c_int) -> c_int {
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
/// The numeric evaluator core (the `operator_t`/`value_t` fixed heaps and their
/// intrusive prev/next lists) is kept byte-faithful — it is `String`-free, so
/// its two `zeroed()` stack arrays stay. Only the token *input* changes: Raven's
/// `token_t *` linked list becomes a `&[Token]` walked by index.
///
/// The C `switch(t->type)`/`switch(t->subtype)` fallthrough is transcribed with
/// labeled blocks (`'sw`/`'subsw`) so a C `break` out of a case maps to
/// `break '<label>`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1730-2177`
fn PC_EvaluateTokens(
    bot: &mut BotLib,
    source: &Source,
    tokens: &[Token],
    mut intvalue: Option<&mut c_long>,
    mut floatvalue: Option<&mut f64>,
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
        if let Some(v) = intvalue.as_deref_mut() {
            *v = 0;
        }
        if let Some(v) = floatvalue.as_deref_mut() {
            *v = 0.0;
        }
        let mut ti: usize = 0;
        'tokens: while ti < tokens.len() {
            'sw: {
                if tokens[ti].type_ == TT_NAME {
                    if lastwasvalue != 0 || negativevalue != 0 {
                        SourceError(bot, source, "syntax error in #if/#elif");
                        error = 1;
                        break 'sw;
                    }
                    if tokens[ti].string != "defined" {
                        SourceError(
                            bot,
                            source,
                            &format!("undefined name {} in #if/#elif", tokens[ti].string),
                        );
                        error = 1;
                        break 'sw;
                    }
                    ti += 1;
                    // Raven derefs `t->string` here without a null check (UB if
                    // `defined` was the last token, §F19); guard it.
                    if tokens.get(ti).map(|t| t.string == "(").unwrap_or(false) {
                        brace = qtrue;
                        ti += 1;
                    }
                    if ti >= tokens.len() || tokens[ti].type_ != TT_NAME {
                        SourceError(bot, source, "defined without name in #if/#elif");
                        error = 1;
                        break 'sw;
                    }
                    // AllocValue(v)
                    if numvalues >= MAX_VALUES as c_int {
                        SourceError(bot, source, "out of value space\n");
                        error = 1;
                        break 'sw;
                    }
                    let v: *mut value_t = &mut value_heap[numvalues as usize];
                    numvalues += 1;
                    // #if DEFINEHASHING (live)
                    if PC_FindHashedDefine(source, &tokens[ti].string).is_some() {
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
                        ti += 1;
                        if ti >= tokens.len() || tokens[ti].string != ")" {
                            SourceError(bot, source, "defined without ) in #if/#elif");
                            error = 1;
                            break 'sw;
                        }
                    }
                    brace = qfalse;
                    // defined() creates a value
                    lastwasvalue = 1;
                } else if tokens[ti].type_ == TT_NUMBER {
                    if lastwasvalue != 0 {
                        SourceError(bot, source, "syntax error in #if/#elif");
                        error = 1;
                        break 'sw;
                    }
                    // AllocValue(v)
                    if numvalues >= MAX_VALUES as c_int {
                        SourceError(bot, source, "out of value space\n");
                        error = 1;
                        break 'sw;
                    }
                    let v: *mut value_t = &mut value_heap[numvalues as usize];
                    numvalues += 1;
                    if negativevalue != 0 {
                        (*v).intvalue = -(tokens[ti].intvalue as c_int) as c_long;
                        (*v).floatvalue = -tokens[ti].floatvalue;
                    } else {
                        (*v).intvalue = tokens[ti].intvalue as c_long;
                        (*v).floatvalue = tokens[ti].floatvalue;
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
                } else if tokens[ti].type_ == TT_PUNCTUATION {
                    if negativevalue != 0 {
                        SourceError(bot, source, "misplaced minus sign in #if/#elif");
                        error = 1;
                        break 'sw;
                    }
                    if tokens[ti].subtype == P_PARENTHESESOPEN {
                        parentheses += 1;
                        break 'sw;
                    } else if tokens[ti].subtype == P_PARENTHESESCLOSE {
                        parentheses -= 1;
                        if parentheses < 0 {
                            SourceError(bot, source, "too many ) in #if/#elsif");
                            error = 1;
                        }
                        break 'sw;
                    }
                    // check for invalid operators on floating point values
                    if integer == 0
                        && (tokens[ti].subtype == P_BIN_NOT
                            || tokens[ti].subtype == P_MOD
                            || tokens[ti].subtype == P_RSHIFT
                            || tokens[ti].subtype == P_LSHIFT
                            || tokens[ti].subtype == P_BIN_AND
                            || tokens[ti].subtype == P_BIN_OR
                            || tokens[ti].subtype == P_BIN_XOR)
                    {
                        SourceError(
                            bot,
                            source,
                            &format!(
                                "illigal operator {} on floating point operands\n",
                                tokens[ti].string
                            ),
                        );
                        error = 1;
                        break 'sw;
                    }
                    'subsw: {
                        let st = tokens[ti].subtype;
                        if st == P_LOGIC_NOT || st == P_BIN_NOT {
                            if lastwasvalue != 0 {
                                SourceError(bot, source, "! or ~ after value in #if/#elif");
                                error = 1;
                                break 'subsw;
                            }
                        } else if st == P_INC || st == P_DEC {
                            SourceError(bot, source, "++ or -- used in #if/#elif");
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
                                SourceError(
                                    bot,
                                    source,
                                    &format!(
                                        "operator {} after operator in #if/#elif",
                                        tokens[ti].string
                                    ),
                                );
                                error = 1;
                                break 'subsw;
                            }
                        } else {
                            SourceError(
                                bot,
                                source,
                                &format!("invalid operator {} in #if/#elif", tokens[ti].string),
                            );
                            error = 1;
                            break 'subsw;
                        }
                    }
                    if error == 0 && negativevalue == 0 {
                        // AllocOperator(o)
                        if numoperators >= MAX_OPERATORS as c_int {
                            SourceError(bot, source, "out of operator space\n");
                            error = 1;
                            break 'sw;
                        }
                        let o: *mut operator_t = &mut operator_heap[numoperators as usize];
                        numoperators += 1;
                        (*o).mOperator = tokens[ti].subtype;
                        (*o).priority = PC_OperatorPriority(tokens[ti].subtype);
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
                    SourceError(
                        bot,
                        source,
                        &format!("unknown {} in #if/#elif", tokens[ti].string),
                    );
                    error = 1;
                }
            }
            if error != 0 {
                break 'tokens;
            }
            ti += 1;
        }
        if error == 0 {
            if lastwasvalue == 0 {
                SourceError(bot, source, "trailing operator in #if/#elif");
                error = 1;
            } else if parentheses != 0 {
                SourceError(bot, source, "too many ( in #if/#elif");
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
                    SourceError(bot, source, "mising values in #if/#elif");
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
                        SourceError(bot, source, "divide by zero in #if/#elif\n");
                        error = 1;
                    } else {
                        (*v1).intvalue /= (*v2).intvalue;
                        (*v1).floatvalue /= (*v2).floatvalue;
                    }
                }
                P_MOD => {
                    if (*v2).intvalue == 0 {
                        SourceError(bot, source, "divide by zero in #if/#elif\n");
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
                        SourceError(bot, source, ": without ? in #if/#elif");
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
                        SourceError(bot, source, "? after ? in #if/#elif");
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
            if let Some(v) = intvalue.as_deref_mut() {
                *v = (*firstvalue).intvalue;
            }
            if let Some(v) = floatvalue.as_deref_mut() {
                *v = (*firstvalue).floatvalue;
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
        if let Some(v) = intvalue.as_deref_mut() {
            *v = 0;
        }
        if let Some(v) = floatvalue.as_deref_mut() {
            *v = 0.0;
        }
        qfalse
    }
}

/// Raven `FreeSource` — free a source and every script/token/define/indent it
/// owns.
///
/// Redesigned: consuming the owned `Source` drops its script stack, token queue,
/// arena, and indent stack in one move (Raven walked and freed each list). The
/// copies of global defines the source holds are dropped with it, leaving the
/// shared global arena untouched.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3113-3177`
pub fn FreeSource(source: Source) {
    drop(source);
}

/// Raven `PC_ReadSourceToken` — read the next token from the source, popping
/// finished scripts.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:293-329`
fn PC_ReadSourceToken(bot: &mut BotLib, source: &mut Source, token: &mut Token) -> c_int {
    // if there's no token already available
    while source.tokens.is_empty() {
        // if there's a token to read from the script
        {
            let script = source.scriptstack.last_mut().unwrap();
            if PS_ReadToken(bot, script, token) != 0 {
                return qtrue;
            }
        }
        // if at the end of the script
        {
            let at_end = EndOfScript(source.scriptstack.last().unwrap()) != 0;
            if at_end {
                // remove all indents of the script
                let top = source.scriptstack.len() - 1;
                while !source.indentstack.is_empty()
                    && source.indentstack.last().unwrap().script == top
                {
                    SourceWarning(bot, source, "missing #endif");
                    let mut r#type: c_int = 0;
                    let mut skip: c_int = 0;
                    PC_PopIndent(source, &mut r#type, &mut skip);
                }
            }
        }
        // if this was the initial script
        if source.scriptstack.len() == 1 {
            return qfalse;
        }
        // remove the script and return to the last one
        source.scriptstack.pop();
    }
    // copy the already available token
    *token = source.tokens.pop_front().unwrap();
    qtrue
}

/// Raven `PC_UnreadSourceToken` — push a copy of a token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:336-344`
fn PC_UnreadSourceToken(source: &mut Source, token: &Token) -> c_int {
    source.tokens.push_front(token.clone());
    qtrue
}

/// Raven `PC_ReadDefineParms` — read the actual parameters of a macro
/// invocation into per-parameter token lists.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:351-443`
fn PC_ReadDefineParms(
    bot: &mut BotLib,
    source: &mut Source,
    define: &Define,
    maxparms: c_int,
) -> Option<Vec<Vec<Token>>> {
    let mut token = Token::default();

    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, &format!("define {} missing parms", define.name));
        return None;
    }
    //
    if define.numparms > maxparms {
        SourceError(
            bot,
            source,
            &format!("define with more than {} parameters", maxparms),
        );
        return None;
    }
    //
    let mut parms: Vec<Vec<Token>> = vec![Vec::new(); define.numparms as usize];
    // if no leading "("
    if token.string != "(" {
        PC_UnreadSourceToken(source, &token);
        SourceError(bot, source, &format!("define {} missing parms", define.name));
        return None;
    }
    // read the define parameters
    let mut done = 0;
    let mut numparms = 0;
    let mut indent = 0;
    while done == 0 {
        if numparms >= maxparms {
            SourceError(
                bot,
                source,
                &format!("define {} with too many parms", define.name),
            );
            return None;
        }
        if numparms >= define.numparms {
            SourceWarning(
                bot,
                source,
                &format!("define {} has too many parms", define.name),
            );
            return None;
        }
        let mut lastcomma = 1;
        while done == 0 {
            //
            if PC_ReadSourceToken(bot, source, &mut token) == 0 {
                SourceError(bot, source, &format!("define {} incomplete", define.name));
                return None;
            }
            //
            if token.string == "," && indent <= 0 {
                if lastcomma != 0 {
                    SourceWarning(bot, source, "too many comma's");
                }
                lastcomma = 1;
                break;
            }
            lastcomma = 0;
            //
            if token.string == "(" {
                indent += 1;
                continue;
            } else if token.string == ")" {
                indent -= 1;
                if indent <= 0 {
                    if parms[(define.numparms - 1) as usize].is_empty() {
                        SourceWarning(bot, source, "too few define parms");
                    }
                    done = 1;
                    break;
                }
            }
            //
            if numparms < define.numparms {
                parms[numparms as usize].push(token.clone());
            }
        }
        numparms += 1;
    }
    Some(parms)
}

/// Raven `PC_Directive_include` — handle `#include "file"` / `#include <file>`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:970-1053`
fn PC_Directive_include(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();
    let mut path = String::new();

    if source.skip > 0 {
        return qtrue;
    }
    //
    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, "#include without file name");
        return qfalse;
    }
    if token.linescrossed > 0 {
        SourceError(bot, source, "#include without file name");
        return qfalse;
    }
    let mut script: Option<Script> = None;
    if token.type_ == TT_STRING {
        StripDoubleQuotes(&mut token.string);
        PC_ConvertPath(&mut token.string);
        script = LoadScriptFile(bot, &token.string);
        if script.is_none() {
            path = format!("{}{}", source.includepath, token.string);
            script = LoadScriptFile(bot, &path);
        }
    } else if token.type_ == TT_PUNCTUATION && token.string.as_bytes().first() == Some(&b'<') {
        path = source.includepath.clone();
        while PC_ReadSourceToken(bot, source, &mut token) != 0 {
            if token.linescrossed > 0 {
                PC_UnreadSourceToken(source, &token);
                break;
            }
            if token.type_ == TT_PUNCTUATION && token.string.as_bytes().first() == Some(&b'>') {
                break;
            }
            path.push_str(&token.string);
        }
        if token.string.as_bytes().first() != Some(&b'>') {
            SourceWarning(bot, source, "#include missing trailing >");
        }
        if path.is_empty() {
            SourceError(bot, source, "#include without file name between < >");
            return qfalse;
        }
        PC_ConvertPath(&mut path);
        script = LoadScriptFile(bot, &path);
    } else {
        SourceError(bot, source, "#include without file name");
        return qfalse;
    }
    // #ifdef QUAKE (not defined) omitted.
    let script = match script {
        Some(s) => s,
        None => {
            // #ifdef SCREWUP (not defined) -> SourceError branch
            SourceError(bot, source, &format!("file {} not found", path));
            return qfalse;
        }
    };
    PC_PushScript(bot, source, script);
    qtrue
}

/// Raven `PC_ReadLine` — read a token on the current logical line, honoring
/// line continuations.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1062-1079`
fn PC_ReadLine(bot: &mut BotLib, source: &mut Source, token: &mut Token) -> c_int {
    let mut crossline: c_int = 0;
    loop {
        if PC_ReadSourceToken(bot, source, token) == 0 {
            return qfalse;
        }

        if token.linescrossed > crossline {
            PC_UnreadSourceToken(source, token);
            return qfalse;
        }
        crossline = 1;
        if token.string != "\\" {
            break;
        }
    }
    qtrue
}

/// Raven `PC_Directive_undef` — handle `#undef name`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1108-1173`
fn PC_Directive_undef(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();

    if source.skip > 0 {
        return qtrue;
    }
    //
    if PC_ReadLine(bot, source, &mut token) == 0 {
        SourceError(bot, source, "undef without name");
        return qfalse;
    }
    if token.type_ != TT_NAME {
        PC_UnreadSourceToken(source, &token);
        SourceError(bot, source, &format!("expected name, found {}", token.string));
        return qfalse;
    }
    // #if DEFINEHASHING (live)
    let hash = PC_NameHash(&token.string) as usize;
    let mut found: Option<(usize, usize)> = None;
    for (pos, &idx) in source.definehash[hash].iter().enumerate() {
        if source.defines[idx].name == token.string {
            found = Some((pos, idx));
            break;
        }
    }
    if let Some((pos, idx)) = found {
        if source.defines[idx].flags & DEFINE_FIXED != 0 {
            SourceWarning(bot, source, &format!("can't undef {}", token.string));
        } else {
            // unlink the define from its hash chain (arena slot left dead)
            source.definehash[hash].remove(pos);
        }
    }
    qtrue
}

/// Raven `PC_Directive_define` — handle `#define name[(...)] tokens`.
///
/// The define is built entirely in locals and inserted into the source's
/// arena/hash at the end (`PC_AddDefineToHash`). Raven links it into the hash
/// *before* reading the body, but body reading never performs a hashed lookup
/// (`PC_ReadLine` → `PC_ReadSourceToken`, no macro expansion) and the
/// recursion/duplicate-parm checks use the local name/parm list, so end-insert
/// is behaviorally identical and sidesteps aliasing the arena mid-build.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1180-1316`
fn PC_Directive_define(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();

    if source.skip > 0 {
        return qtrue;
    }
    //
    if PC_ReadLine(bot, source, &mut token) == 0 {
        SourceError(bot, source, "#define without name");
        return qfalse;
    }
    if token.type_ != TT_NAME {
        PC_UnreadSourceToken(source, &token);
        SourceError(
            bot,
            source,
            &format!("expected name after #define, found {}", token.string),
        );
        return qfalse;
    }
    // check if the define already exists (#if DEFINEHASHING, live)
    if let Some(idx) = PC_FindHashedDefine(source, &token.string) {
        if source.defines[idx].flags & DEFINE_FIXED != 0 {
            SourceError(bot, source, &format!("can't redefine {}", token.string));
            return qfalse;
        }
        SourceWarning(bot, source, &format!("redefinition of {}", token.string));
        // unread the define name before executing the #undef directive
        PC_UnreadSourceToken(source, &token);
        if PC_Directive_undef(bot, source) == 0 {
            return qfalse;
        }
        // Raven re-finds the define here; the result is immediately overwritten
        // by the fresh allocation below, so the re-find is dropped.
    }
    // allocate define (built locally, inserted at the end)
    let name = token.string.clone();
    let mut define = Define::default();
    define.name = name.clone();
    // if nothing is defined, just return
    if PC_ReadLine(bot, source, &mut token) == 0 {
        PC_AddDefineToHash(bot, source, define);
        return qtrue;
    }
    // if it is a define with parameters
    if PC_WhiteSpaceBeforeToken(&token) == 0 && token.string == "(" {
        // read the define parameters
        if PC_CheckTokenString(bot, source, ")") == 0 {
            loop {
                if PC_ReadLine(bot, source, &mut token) == 0 {
                    SourceError(bot, source, "expected define parameter");
                    return qfalse;
                }
                // if it isn't a name
                if token.type_ != TT_NAME {
                    SourceError(bot, source, "invalid define parameter");
                    return qfalse;
                }
                //
                if PC_FindDefineParm(&define, &token.string) >= 0 {
                    SourceError(bot, source, "two the same define parameters");
                    return qfalse;
                }
                // add the define parm
                let mut t = token.clone();
                PC_ClearTokenWhiteSpace(&mut t);
                define.parms.push(t);
                define.numparms += 1;
                // read next token
                if PC_ReadLine(bot, source, &mut token) == 0 {
                    SourceError(bot, source, "define parameters not terminated");
                    return qfalse;
                }
                //
                if token.string == ")" {
                    break;
                }
                // then it must be a comma
                if token.string != "," {
                    SourceError(bot, source, "define not terminated");
                    return qfalse;
                }
            }
        }
        if PC_ReadLine(bot, source, &mut token) == 0 {
            PC_AddDefineToHash(bot, source, define);
            return qtrue;
        }
    }
    // read the defined stuff
    loop {
        let mut t = token.clone();
        if t.type_ == TT_NAME && t.string == name {
            SourceError(bot, source, "recursive define (removed recursion)");
            if PC_ReadLine(bot, source, &mut token) == 0 {
                break;
            }
            continue;
        }
        PC_ClearTokenWhiteSpace(&mut t);
        define.tokens.push(t);
        if PC_ReadLine(bot, source, &mut token) == 0 {
            break;
        }
    }
    //
    if let (Some(first), Some(last)) = (define.tokens.first(), define.tokens.last()) {
        // check for merge operators at the beginning or end
        if first.string == "##" || last.string == "##" {
            SourceError(bot, source, "define with misplaced ##");
            return qfalse;
        }
    }
    PC_AddDefineToHash(bot, source, define);
    qtrue
}

/// Raven `PC_DefineFromString` — build a define from a `name value` string.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1323-1374`
fn PC_DefineFromString(bot: &mut BotLib, string: &str) -> Option<Define> {
    let script = LoadScriptMemory(string.as_bytes(), string.len() as c_int, "*extern");
    // create a new source
    let mut src = Source::default();
    src.filename = "*extern".to_string();
    src.scriptstack.push(script);
    // #if DEFINEHASHING (live)
    src.definehash = vec![Vec::new(); DEFINEHASHSIZE];
    // create a define from the source
    let res = PC_Directive_define(bot, &mut src);
    // any tokens left in src.tokens are dropped with `src`.
    // #ifdef DEFINEHASHING (live) — retrieve the first define created
    let def = src
        .definehash
        .iter()
        .flatten()
        .next()
        .map(|&idx| src.defines[idx].clone());
    // if the define was created succesfully
    if res > 0 {
        def
    } else {
        None
    }
}

/// Raven `PC_AddDefine` — add a `name value` define to a source (or globally).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1381-1399`
pub fn PC_AddDefine(bot: &mut BotLib, source: &mut Source, string: &str) -> c_int {
    if bot.addGlobalDefine == qtrue {
        return PC_AddGlobalDefine(core::ptr::null_mut());
    }

    let define = match PC_DefineFromString(bot, string) {
        Some(d) => d,
        None => return qfalse,
    };
    // #if DEFINEHASHING (live)
    PC_AddDefineToHash(bot, source, define);
    qtrue
}

/// Raven `PC_Directive_if_def` — shared body of `#ifdef`/`#ifndef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1558-1583`
fn PC_Directive_if_def(bot: &mut BotLib, source: &mut Source, r#type: c_int) -> c_int {
    let mut token = Token::default();

    if PC_ReadLine(bot, source, &mut token) == 0 {
        SourceError(bot, source, "#ifdef without name");
        return qfalse;
    }
    if token.type_ != TT_NAME {
        PC_UnreadSourceToken(source, &token);
        SourceError(
            bot,
            source,
            &format!("expected name after #ifdef, found {}", token.string),
        );
        return qfalse;
    }
    // #if DEFINEHASHING (live)
    let d = PC_FindHashedDefine(source, &token.string);
    let skip = ((r#type == INDENT_IFDEF) == d.is_none()) as c_int;
    PC_PushIndent(source, r#type, skip);
    qtrue
}

/// Raven `PC_Directive_ifdef` — handle `#ifdef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1590-1593`
fn PC_Directive_ifdef(bot: &mut BotLib, source: &mut Source) -> c_int {
    PC_Directive_if_def(bot, source, INDENT_IFDEF)
}

/// Raven `PC_Directive_ifndef` — handle `#ifndef`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1600-1603`
fn PC_Directive_ifndef(bot: &mut BotLib, source: &mut Source) -> c_int {
    PC_Directive_if_def(bot, source, INDENT_IFNDEF)
}

/// Raven `PC_Directive_else` — handle `#else`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1610-1627`
fn PC_Directive_else(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(source, &mut r#type, &mut skip);
    if r#type == 0 {
        SourceError(bot, source, "misplaced #else");
        return qfalse;
    }
    if r#type == INDENT_ELSE {
        SourceError(bot, source, "#else after #else");
        return qfalse;
    }
    PC_PushIndent(source, INDENT_ELSE, (skip == 0) as c_int);
    qtrue
}

/// Raven `PC_Directive_endif` — handle `#endif`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:1634-1645`
fn PC_Directive_endif(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(source, &mut r#type, &mut skip);
    if r#type == 0 {
        SourceError(bot, source, "misplaced #endif");
        return qfalse;
    }
    qtrue
}

/// Raven `PC_ExpandDefine` — expand a macro invocation into a token list.
///
/// Returns the expanded token list on success (empty is possible), `None` on
/// error. The `define` is passed by (cloned) reference so the source can be
/// mutated during parameter reading without aliasing the arena.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:782-913`
fn PC_ExpandDefine(
    bot: &mut BotLib,
    source: &mut Source,
    deftoken: &Token,
    define: &Define,
) -> Option<Vec<Token>> {
    // if it is a builtin define
    if define.builtin != 0 {
        return Some(PC_ExpandBuiltinDefine(bot, source, deftoken, define));
    }
    // if the define has parameters
    let parms: Vec<Vec<Token>> = if define.numparms != 0 {
        PC_ReadDefineParms(bot, source, define, MAX_DEFINEPARMS as c_int)?
    } else {
        Vec::new()
    };
    // create a list with tokens of the expanded define
    let mut list: Vec<Token> = Vec::new();
    let dtokens = &define.tokens;
    let mut dti = 0;
    while dti < dtokens.len() {
        let dt = &dtokens[dti];
        let mut parmnum: c_int = -1;
        // if the token is a name, it could be a define parameter
        if dt.type_ == TT_NAME {
            parmnum = PC_FindDefineParm(define, &dt.string);
        }
        // if it is a define parameter
        if parmnum >= 0 {
            for pt in &parms[parmnum as usize] {
                list.push(pt.clone());
            }
        } else {
            // if stringizing operator
            if dt.string == "#" {
                // the stringizing operator must be followed by a define parameter
                let np = if dti + 1 < dtokens.len() {
                    PC_FindDefineParm(define, &dtokens[dti + 1].string)
                } else {
                    -1
                };
                //
                if np >= 0 {
                    // step over the stringizing operator
                    dti += 1;
                    // stringize the define parameter tokens
                    list.push(PC_StringizeTokens(&parms[np as usize]));
                } else {
                    SourceWarning(
                        bot,
                        source,
                        "stringizing operator without define parameter",
                    );
                    dti += 1;
                    continue;
                }
            } else {
                list.push(dt.clone());
            }
        }
        dti += 1;
    }
    // check for the merging operator
    let mut i = 0;
    while i < list.len() {
        if i + 1 < list.len() && list[i + 1].string.starts_with("##") {
            // if the merging operator
            if i + 2 < list.len() {
                let t2 = list[i + 2].clone();
                if PC_MergeTokens(&mut list[i], &t2) == 0 {
                    SourceError(
                        bot,
                        source,
                        &format!("can't merge {} with {}", list[i].string, t2.string),
                    );
                    return None;
                }
                // remove the "##" and the merged-in token
                list.remove(i + 2);
                list.remove(i + 1);
                continue;
            }
        }
        i += 1;
    }
    Some(list)
}

/// Raven `PC_ExpandDefineIntoSource` — expand a macro and push its tokens back
/// onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:920-933`
fn PC_ExpandDefineIntoSource(
    bot: &mut BotLib,
    source: &mut Source,
    deftoken: &Token,
    define: &Define,
) -> c_int {
    match PC_ExpandDefine(bot, source, deftoken, define) {
        None => qfalse,
        Some(list) if !list.is_empty() => {
            // prepend the expanded list so it reads back in order
            for tok in list.into_iter().rev() {
                source.tokens.push_front(tok);
            }
            qtrue
        }
        Some(_) => qfalse,
    }
}

/// Raven `PC_Directive_line` — `#line` is not supported.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2429-2433`
fn PC_Directive_line(bot: &mut BotLib, source: &mut Source) -> c_int {
    SourceError(bot, source, "#line directive not supported");
    qfalse
}

/// Raven `PC_Directive_error` — `#error directive: <text>`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2440-2448`
fn PC_Directive_error(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();
    PC_ReadSourceToken(bot, source, &mut token);
    SourceError(bot, source, &format!("#error directive: {}", token.string));
    qfalse
}

/// Raven `PC_Directive_pragma` — `#pragma` is not supported (skip the line).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2455-2462`
fn PC_Directive_pragma(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();
    SourceWarning(bot, source, "#pragma directive not supported");
    while PC_ReadLine(bot, source, &mut token) != 0 {}
    qtrue
}

/// Raven `UnreadSignToken` — push a synthesized `-` token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2469-2481`
fn UnreadSignToken(source: &mut Source) {
    let mut token = Token::default();
    let (line, sp) = {
        let s = source.scriptstack.last().unwrap();
        (s.line, s.script_p)
    };
    token.line = line;
    // whitespace_p == endwhitespace_p (zero-length span → no leading whitespace)
    token.whitespace_span = Some((sp, sp));
    token.linescrossed = 0;
    token.string = "-".to_string();
    token.type_ = TT_PUNCTUATION;
    token.subtype = P_SUB;
    PC_UnreadSourceToken(source, &token);
}

/// Raven `PC_Directive_eval` — `#eval expr` pushes the integer result back.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2488-2505`
fn PC_Directive_eval(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: c_long = 0;

    if PC_Evaluate(bot, source, Some(&mut value), None, qtrue) == 0 {
        return qfalse;
    }
    //
    let mut token = Token::default();
    let (line, sp) = {
        let s = source.scriptstack.last().unwrap();
        (s.line, s.script_p)
    };
    token.line = line;
    token.whitespace_span = Some((sp, sp));
    token.linescrossed = 0;
    token.string = format!("{}", (value as c_int).abs());
    token.type_ = TT_NUMBER;
    token.subtype = TT_INTEGER | TT_LONG | TT_DECIMAL;
    PC_UnreadSourceToken(source, &token);
    if value < 0 {
        UnreadSignToken(source);
    }
    qtrue
}

/// Raven `PC_Directive_evalfloat` — `#evalfloat expr` pushes the float result.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2512-2528`
fn PC_Directive_evalfloat(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: f64 = 0.0;

    if PC_Evaluate(bot, source, None, Some(&mut value), qfalse) == 0 {
        return qfalse;
    }
    let mut token = Token::default();
    let (line, sp) = {
        let s = source.scriptstack.last().unwrap();
        (s.line, s.script_p)
    };
    token.line = line;
    token.whitespace_span = Some((sp, sp));
    token.linescrossed = 0;
    token.string = format!("{:.2}", value.abs());
    token.type_ = TT_NUMBER;
    token.subtype = TT_FLOAT | TT_LONG | TT_DECIMAL;
    PC_UnreadSourceToken(source, &token);
    if value < 0.0 {
        UnreadSignToken(source);
    }
    qtrue
}

/// Raven `directives[]` — file-scope `#`-directive dispatch table. The trailing
/// `{NULL, NULL}` sentinel dissolves (slice iteration ends at the bound).
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2535-2551`.
const directives: &[Directive] = &[
    Directive { name: "if", func: PC_Directive_if },
    Directive { name: "ifdef", func: PC_Directive_ifdef },
    Directive { name: "ifndef", func: PC_Directive_ifndef },
    Directive { name: "elif", func: PC_Directive_elif },
    Directive { name: "else", func: PC_Directive_else },
    Directive { name: "endif", func: PC_Directive_endif },
    Directive { name: "include", func: PC_Directive_include },
    Directive { name: "define", func: PC_Directive_define },
    Directive { name: "undef", func: PC_Directive_undef },
    Directive { name: "line", func: PC_Directive_line },
    Directive { name: "error", func: PC_Directive_error },
    Directive { name: "pragma", func: PC_Directive_pragma },
    Directive { name: "eval", func: PC_Directive_eval },
    Directive { name: "evalfloat", func: PC_Directive_evalfloat },
];

/// Raven `dollardirectives[]` — file-scope `$`-directive dispatch table.
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2648-2653`.
const dollardirectives: &[Directive] = &[
    Directive { name: "evalint", func: PC_DollarDirective_evalint },
    Directive { name: "evalfloat", func: PC_DollarDirective_evalfloat },
];

/// Raven `PC_ReadDirective` — dispatch a `#`-directive to its handler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2554-2586`
fn PC_ReadDirective(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();

    // read the directive name
    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, "found # without name");
        return qfalse;
    }
    // directive name must be on the same line
    if token.linescrossed > 0 {
        PC_UnreadSourceToken(source, &token);
        SourceError(bot, source, "found # at end of line");
        return qfalse;
    }
    // if it is a name
    if token.type_ == TT_NAME {
        // find the precompiler directive
        for d in directives {
            if d.name == token.string {
                return (d.func)(bot, source);
            }
        }
    }
    SourceError(
        bot,
        source,
        &format!("unknown precompiler directive {}", token.string),
    );
    qfalse
}

/// Raven `PC_ReadDollarDirective` — dispatch a `$`-directive to its handler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2655-2688`
fn PC_ReadDollarDirective(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut token = Token::default();

    // read the directive name
    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, "found $ without name");
        return qfalse;
    }
    // directive name must be on the same line
    if token.linescrossed > 0 {
        PC_UnreadSourceToken(source, &token);
        SourceError(bot, source, "found $ at end of line");
        return qfalse;
    }
    // if it is a name
    if token.type_ == TT_NAME {
        // find the precompiler directive
        for d in dollardirectives {
            if d.name == token.string {
                return (d.func)(bot, source);
            }
        }
    }
    PC_UnreadSourceToken(source, &token);
    SourceError(
        bot,
        source,
        &format!("unknown precompiler directive {}", token.string),
    );
    qfalse
}

/// Raven `PC_DollarDirective_evalint` — `$evalint(expr)`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2593-2614`
fn PC_DollarDirective_evalint(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: c_long = 0;

    if PC_DollarEvaluate(bot, source, Some(&mut value), None, qtrue) == 0 {
        return qfalse;
    }
    //
    let mut token = Token::default();
    let (line, sp) = {
        let s = source.scriptstack.last().unwrap();
        (s.line, s.script_p)
    };
    token.line = line;
    token.whitespace_span = Some((sp, sp));
    token.linescrossed = 0;
    token.string = format!("{}", (value as c_int).abs());
    token.type_ = TT_NUMBER;
    token.subtype = TT_INTEGER | TT_LONG | TT_DECIMAL;
    // #ifdef NUMBERVALUE (not defined) omitted.
    PC_UnreadSourceToken(source, &token);
    if value < 0 {
        UnreadSignToken(source);
    }
    qtrue
}

/// Raven `PC_DollarDirective_evalfloat` — `$evalfloat(expr)`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2621-2641`
fn PC_DollarDirective_evalfloat(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: f64 = 0.0;

    if PC_DollarEvaluate(bot, source, None, Some(&mut value), qfalse) == 0 {
        return qfalse;
    }
    let mut token = Token::default();
    let (line, sp) = {
        let s = source.scriptstack.last().unwrap();
        (s.line, s.script_p)
    };
    token.line = line;
    token.whitespace_span = Some((sp, sp));
    token.linescrossed = 0;
    token.string = format!("{:.2}", value.abs());
    token.type_ = TT_NUMBER;
    token.subtype = TT_FLOAT | TT_LONG | TT_DECIMAL;
    // #ifdef NUMBERVALUE (not defined) omitted.
    PC_UnreadSourceToken(source, &token);
    if value < 0.0 {
        UnreadSignToken(source);
    }
    qtrue
}

/// Raven `PC_Evaluate` — evaluate a `#if`/`#elif` expression line.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2184-2276`
fn PC_Evaluate(
    bot: &mut BotLib,
    source: &mut Source,
    mut intvalue: Option<&mut c_long>,
    mut floatvalue: Option<&mut f64>,
    integer: c_int,
) -> c_int {
    let mut token = Token::default();
    let mut defined: c_int = qfalse;

    if let Some(v) = intvalue.as_deref_mut() {
        *v = 0;
    }
    if let Some(v) = floatvalue.as_deref_mut() {
        *v = 0.0;
    }
    //
    if PC_ReadLine(bot, source, &mut token) == 0 {
        SourceError(bot, source, "no value after #if/#elif");
        return qfalse;
    }
    let mut firsttoken: Vec<Token> = Vec::new();
    loop {
        // if the token is a name
        if token.type_ == TT_NAME {
            if defined != 0 {
                defined = qfalse;
                firsttoken.push(token.clone());
            } else if token.string == "defined" {
                defined = qtrue;
                firsttoken.push(token.clone());
            } else {
                // then it must be a define (#if DEFINEHASHING, live)
                let idx = match PC_FindHashedDefine(source, &token.string) {
                    Some(i) => i,
                    None => {
                        SourceError(
                            bot,
                            source,
                            &format!("can't evaluate {}, not defined", token.string),
                        );
                        return qfalse;
                    }
                };
                let define = source.defines[idx].clone();
                if PC_ExpandDefineIntoSource(bot, source, &token, &define) == 0 {
                    return qfalse;
                }
            }
        }
        // if the token is a number or a punctuation
        else if token.type_ == TT_NUMBER || token.type_ == TT_PUNCTUATION {
            firsttoken.push(token.clone());
        } else {
            SourceError(bot, source, &format!("can't evaluate {}", token.string));
            return qfalse;
        }
        if PC_ReadLine(bot, source, &mut token) == 0 {
            break;
        }
    }
    //
    if PC_EvaluateTokens(
        bot,
        source,
        &firsttoken,
        intvalue.as_deref_mut(),
        floatvalue.as_deref_mut(),
        integer,
    ) == 0
    {
        return qfalse;
    }
    //
    qtrue
}

/// Raven `PC_DollarEvaluate` — evaluate a `$evalint`/`$evalfloat(expr)` body.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2283-2384`
fn PC_DollarEvaluate(
    bot: &mut BotLib,
    source: &mut Source,
    mut intvalue: Option<&mut c_long>,
    mut floatvalue: Option<&mut f64>,
    integer: c_int,
) -> c_int {
    let mut defined: c_int = qfalse;
    let mut token = Token::default();

    if let Some(v) = intvalue.as_deref_mut() {
        *v = 0;
    }
    if let Some(v) = floatvalue.as_deref_mut() {
        *v = 0.0;
    }
    //
    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, "no leading ( after $evalint/$evalfloat");
        return qfalse;
    }
    if PC_ReadSourceToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, "nothing to evaluate");
        return qfalse;
    }
    let mut indent = 1;
    let mut firsttoken: Vec<Token> = Vec::new();
    loop {
        // if the token is a name
        if token.type_ == TT_NAME {
            if defined != 0 {
                defined = qfalse;
                firsttoken.push(token.clone());
            } else if token.string == "defined" {
                defined = qtrue;
                firsttoken.push(token.clone());
            } else {
                // then it must be a define (#if DEFINEHASHING, live)
                let idx = match PC_FindHashedDefine(source, &token.string) {
                    Some(i) => i,
                    None => {
                        SourceError(
                            bot,
                            source,
                            &format!("can't evaluate {}, not defined", token.string),
                        );
                        return qfalse;
                    }
                };
                let define = source.defines[idx].clone();
                if PC_ExpandDefineIntoSource(bot, source, &token, &define) == 0 {
                    return qfalse;
                }
            }
        }
        // if the token is a number or a punctuation
        else if token.type_ == TT_NUMBER || token.type_ == TT_PUNCTUATION {
            if token.string.as_bytes().first() == Some(&b'(') {
                indent += 1;
            } else if token.string.as_bytes().first() == Some(&b')') {
                indent -= 1;
            }
            if indent <= 0 {
                break;
            }
            firsttoken.push(token.clone());
        } else {
            SourceError(bot, source, &format!("can't evaluate {}", token.string));
            return qfalse;
        }
        if PC_ReadSourceToken(bot, source, &mut token) == 0 {
            break;
        }
    }
    //
    if PC_EvaluateTokens(
        bot,
        source,
        &firsttoken,
        intvalue.as_deref_mut(),
        floatvalue.as_deref_mut(),
        integer,
    ) == 0
    {
        return qfalse;
    }
    //
    qtrue
}

/// Raven `PC_ReadToken` — read a fully-resolved token (directives expanded,
/// defines applied, adjacent strings concatenated).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2749-2837`
pub fn PC_ReadToken(bot: &mut BotLib, source: &mut Source, token: &mut Token) -> c_int {
    loop {
        if PC_ReadSourceToken(bot, source, token) == 0 {
            return qfalse;
        }
        // check for precompiler directives
        if token.type_ == TT_PUNCTUATION && token.string.as_bytes().first() == Some(&b'@') {
            // It is a StringEd key: read the next token and prefix it with '@'.
            // (Raven's overlapping memcpy shift yields the same bytes: "@" + next.)
            PC_ReadSourceToken(bot, source, token);
            token.string = format!("@{}", token.string);
            return qtrue;
        }

        if token.type_ == TT_PUNCTUATION && token.string.as_bytes().first() == Some(&b'#') {
            // #ifdef QUAKEC (not defined) -> block always runs
            // read the precompiler directive
            if PC_ReadDirective(bot, source) == 0 {
                return qfalse;
            }
            continue;
        }
        if token.type_ == TT_PUNCTUATION && token.string.as_bytes().first() == Some(&b'$') {
            // #ifdef QUAKEC (not defined) -> block always runs
            // read the precompiler directive
            if PC_ReadDollarDirective(bot, source) == 0 {
                return qfalse;
            }
            continue;
        }
        // recursively concatenate strings that are behind each other still resolving defines
        if token.type_ == TT_STRING {
            let mut newtoken = Token::default();
            if PC_ReadToken(bot, source, &mut newtoken) != 0 {
                if newtoken.type_ == TT_STRING {
                    // remove trailing double quote
                    token.string.pop();
                    if token.string.len() + newtoken.string.len().saturating_sub(1) + 1 >= MAX_TOKEN
                    {
                        SourceError(
                            bot,
                            source,
                            &format!("string longer than MAX_TOKEN {}\n", MAX_TOKEN),
                        );
                        return qfalse;
                    }
                    // concat without leading double quote
                    token.string.push_str(&newtoken.string[1..]);
                } else {
                    PC_UnreadToken(source, &newtoken);
                }
            }
        }
        // if skipping source because of conditional compilation
        if source.skip != 0 {
            continue;
        }
        // if the token is a name
        if token.type_ == TT_NAME {
            // check if the name is a define macro (#if DEFINEHASHING, live)
            if let Some(idx) = PC_FindHashedDefine(source, &token.string) {
                // expand the defined macro
                let define = source.defines[idx].clone();
                if PC_ExpandDefineIntoSource(bot, source, token, &define) == 0 {
                    return qfalse;
                }
                continue;
            }
        }
        // copy token for unreading
        source.token = token.clone();
        // found a token
        return qtrue;
    }
}

/// Raven `PC_Directive_elif` — handle `#elif`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2391-2406`
fn PC_Directive_elif(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: c_long = 0;
    let mut r#type: c_int = 0;
    let mut skip: c_int = 0;

    PC_PopIndent(source, &mut r#type, &mut skip);
    if r#type == 0 || r#type == INDENT_ELSE {
        SourceError(bot, source, "misplaced #elif");
        return qfalse;
    }
    if PC_Evaluate(bot, source, Some(&mut value), None, qtrue) == 0 {
        return qfalse;
    }
    skip = (value == 0) as c_int;
    PC_PushIndent(source, INDENT_ELIF, skip);
    qtrue
}

/// Raven `PC_Directive_if` — handle `#if`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2413-2422`
fn PC_Directive_if(bot: &mut BotLib, source: &mut Source) -> c_int {
    let mut value: c_long = 0;

    if PC_Evaluate(bot, source, Some(&mut value), None, qtrue) == 0 {
        return qfalse;
    }
    let skip = (value == 0) as c_int;
    PC_PushIndent(source, INDENT_IF, skip);
    qtrue
}

/// Raven `PC_ExpectTokenString` — read the next token and require it to equal
/// `string`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2844-2860`
pub fn PC_ExpectTokenString(bot: &mut BotLib, source: &mut Source, string: &str) -> c_int {
    let mut token = Token::default();

    if PC_ReadToken(bot, source, &mut token) == 0 {
        SourceError(bot, source, &format!("couldn't find expected {}", string));
        return qfalse;
    }

    if token.string != string {
        SourceError(
            bot,
            source,
            &format!("expected {}, found {}", string, token.string),
        );
        return qfalse;
    }
    qtrue
}

/// Raven `PC_ExpectTokenType` — read the next token and require a matching
/// type/subtype.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2867-2913`
pub fn PC_ExpectTokenType(
    bot: &mut BotLib,
    source: &mut Source,
    r#type: c_int,
    subtype: c_int,
    token: &mut Token,
) -> c_int {
    let mut str = String::new();

    if PC_ReadToken(bot, source, token) == 0 {
        SourceError(bot, source, "couldn't read expected token");
        return qfalse;
    }

    if token.type_ != r#type {
        if r#type == TT_STRING {
            str = "string".to_string();
        }
        if r#type == TT_LITERAL {
            str = "literal".to_string();
        }
        if r#type == TT_NUMBER {
            str = "number".to_string();
        }
        if r#type == TT_NAME {
            str = "name".to_string();
        }
        if r#type == TT_PUNCTUATION {
            str = "punctuation".to_string();
        }
        SourceError(
            bot,
            source,
            &format!("expected a {}, found {}", str, token.string),
        );
        return qfalse;
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
            SourceError(
                bot,
                source,
                &format!("expected {}, found {}", str, token.string),
            );
            return qfalse;
        }
    } else if token.type_ == TT_PUNCTUATION {
        if token.subtype != subtype {
            SourceError(bot, source, &format!("found {}", token.string));
            return qfalse;
        }
    }
    qtrue
}

/// Raven `PC_ExpectAnyToken` — read the next token, erroring only at EOF.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2920-2931`
pub fn PC_ExpectAnyToken(bot: &mut BotLib, source: &mut Source, token: &mut Token) -> c_int {
    if PC_ReadToken(bot, source, token) == 0 {
        SourceError(bot, source, "couldn't read expected token");
        qfalse
    } else {
        qtrue
    }
}

/// Raven `PC_CheckTokenString` — read the next token; if it equals `string`
/// consume it, else unread and fail.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2938-2948`
pub fn PC_CheckTokenString(bot: &mut BotLib, source: &mut Source, string: &str) -> c_int {
    let mut tok = Token::default();

    if PC_ReadToken(bot, source, &mut tok) == 0 {
        return qfalse;
    }
    // if the token is available
    if tok.string == string {
        return qtrue;
    }
    //
    PC_UnreadSourceToken(source, &tok);
    qfalse
}

/// Raven `PC_CheckTokenType` — read the next token; if type/subtype match copy
/// it out, else unread and fail.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2955-2970`
pub fn PC_CheckTokenType(
    bot: &mut BotLib,
    source: &mut Source,
    r#type: c_int,
    subtype: c_int,
    token: &mut Token,
) -> c_int {
    let mut tok = Token::default();

    if PC_ReadToken(bot, source, &mut tok) == 0 {
        return qfalse;
    }
    // if the type matches
    if tok.type_ == r#type && (tok.subtype & subtype) == subtype {
        *token = tok;
        return qtrue;
    }
    //
    PC_UnreadSourceToken(source, &tok);
    qfalse
}

/// Raven `PC_SkipUntilString` — read tokens until `string` is found.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2977-2986`
pub fn PC_SkipUntilString(bot: &mut BotLib, source: &mut Source, string: &str) -> c_int {
    let mut token = Token::default();

    while PC_ReadToken(bot, source, &mut token) != 0 {
        if token.string == string {
            return qtrue;
        }
    }
    qfalse
}

/// Raven `PC_UnreadLastToken` — push the source's last-read token back.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:2993-2996`
pub fn PC_UnreadLastToken(source: &mut Source) {
    let t = source.token.clone();
    PC_UnreadSourceToken(source, &t);
}

/// Raven `PC_UnreadToken` — push a token back onto the source.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3003-3006`
pub fn PC_UnreadToken(source: &mut Source, token: &Token) {
    PC_UnreadSourceToken(source, token);
}

/// Raven `PC_SetIncludePath` — set a source's include path, ensuring a trailing
/// separator.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3013-3022`
pub fn PC_SetIncludePath(source: &mut Source, path: &str) {
    source.includepath = path.to_string();
    // add trailing path seperator (Raven reads includepath[n-1]; on an empty
    // path that underruns, §F19 — here `ends_with` is false so a separator is
    // appended, giving "/")
    if !source.includepath.ends_with('\\') && !source.includepath.ends_with('/') {
        source.includepath.push_str(PATHSEPERATOR_STR);
    }
}

/// Raven `LoadSourceFile` — open a preprocessor source over a script file.
///
/// Returns the owned `Source` (Raven returned a malloc'd `source_t *`; `None`
/// mirrors the null return). The caller owns it directly, or the seam stows it
/// in `bot.sourceFiles[handle]`.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3039-3073`
pub fn LoadSourceFile(bot: &mut BotLib, filename: &str) -> Option<Source> {
    // #if DEFINEHASHING (live) — the global arena is always present as a `Vec`.
    let script = LoadScriptFile(bot, filename)?;

    let mut source = Source::default();
    source.filename = filename.to_string();
    source.scriptstack.push(script);
    // #if DEFINEHASHING (live) — size the bucket table
    source.definehash = vec![Vec::new(); DEFINEHASHSIZE];
    PC_AddGlobalDefinesToSource(bot, &mut source);
    Some(source)
}

/// Raven `LoadSourceMemory` — open a preprocessor source over an in-memory
/// script.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3080-3106`
pub fn LoadSourceMemory(bot: &mut BotLib, ptr: &[u8], length: c_int, name: &str) -> Source {
    let script = LoadScriptMemory(ptr, length, name);

    let mut source = Source::default();
    source.filename = name.to_string();
    source.scriptstack.push(script);
    // #if DEFINEHASHING (live) — size the bucket table
    source.definehash = vec![Vec::new(); DEFINEHASHSIZE];
    PC_AddGlobalDefinesToSource(bot, &mut source);
    source
}

/// Raven `PC_FreeSourceHandle` — free the source at `handle` in the handle
/// table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3214-3224`
pub fn PC_FreeSourceHandle(bot: &mut BotLib, handle: c_int) -> c_int {
    if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
        return qfalse;
    }
    if bot.sourceFiles[handle as usize].is_none() {
        return qfalse;
    }

    let source = bot.sourceFiles[handle as usize].take().unwrap();
    FreeSource(source);
    qtrue
}

/// Raven `PC_LoadSourceHandle` — load a source file and store it in the handle
/// table.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3189-3207`
pub fn PC_LoadSourceHandle(bot: &mut BotLib, filename: *const c_char) -> c_int {
    let mut i: c_int = 1;
    while i < MAX_SOURCEFILES as c_int {
        if bot.sourceFiles[i as usize].is_none() {
            break;
        }
        i += 1;
    }
    if i >= MAX_SOURCEFILES as c_int {
        return 0;
    }
    PS_SetBaseFolder(bot, "");
    let filename = unsafe { CStr::from_ptr(filename) }.to_string_lossy().into_owned();
    let source = match LoadSourceFile(bot, &filename) {
        Some(s) => s,
        None => return 0,
    };
    bot.sourceFiles[i as usize] = Some(source);
    i
}

/// Raven `PC_SetBaseFolder` — forward to the script tokenizer's base folder.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3300-3303`
pub fn PC_SetBaseFolder(bot: &mut BotLib, path: &str) {
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
    if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
        return qfalse;
    }
    let source = match &bot.sourceFiles[handle as usize] {
        Some(s) => s,
        None => return qfalse,
    };

    // strcpy(filename, source->filename)
    unsafe {
        let bytes = source.filename.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            *filename.add(i) = b as c_char;
        }
        *filename.add(bytes.len()) = 0;
        *line = source.scriptstack.last().map(|s| s.line).unwrap_or(0);
    }
    qtrue
}

/// Raven `PC_CheckOpenSourceHandles` — warn about any source left open in the
/// precompiler.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3310-3323`
pub fn PC_CheckOpenSourceHandles(bot: &mut BotLib) {
    let print = bot.botimport.Print.unwrap();
    for i in 1..MAX_SOURCEFILES {
        if let Some(source) = &bot.sourceFiles[i] {
            // #ifdef BOTLIB (defined)
            let name = source
                .scriptstack
                .last()
                .map(|s| s.filename.as_str())
                .unwrap_or("");
            let msg =
                CString::new(format!("file {} still open in precompiler\n", name)).unwrap_or_default();
            unsafe {
                print(PRT_ERROR, c"%s".as_ptr() as *mut c_char, msg.as_ptr());
            }
        }
    }
}

/// Raven `PC_LoadGlobalDefines` — load a file purely to register its defines
/// globally.
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3226-3245`
pub fn PC_LoadGlobalDefines(bot: &mut BotLib, filename: *const c_char) -> c_int {
    let handle = PC_LoadSourceHandle(bot, filename);
    if handle < 1 {
        return qfalse;
    }

    bot.addGlobalDefine = qtrue;

    // Read all the tokens which will add the defines globally. The source is
    // taken out of the slab so `bot.globaldefines` can be appended to without
    // aliasing it.
    let mut source = bot.sourceFiles[handle as usize].take().unwrap();
    let mut token = Token::default();
    while PC_ReadToken(bot, &mut source, &mut token) != 0 {}
    bot.sourceFiles[handle as usize] = Some(source);

    bot.addGlobalDefine = qfalse;

    PC_FreeSourceHandle(bot, handle);

    qtrue
}

/// Raven `PC_ReadTokenHandle` — read a token from an open source handle into a
/// public `pc_token_t`.
///
/// The shared ABI `pc_token_t` keeps its `[c_char; MAX_TOKENLENGTH]` buffer. One
/// bounded copy of the decoded `Token.string` writes it, preserving Raven's
/// `TT_STRING`/`@`-guarded `StripDoubleQuotes` (applied on the Rust side before
/// the copy, equivalent bytes) and truncating at `MAX_TOKENLENGTH - 1` + NUL
/// (Raven's unbounded `strcpy` would overrun by one byte for a max-length token,
/// §F19).
///
/// Source: `oracle/codemp/botlib/l_precomp.cpp:3253-3273`
pub fn PC_ReadTokenHandle(bot: &mut BotLib, handle: c_int, pc_token: *mut pc_token_t) -> c_int {
    if handle < 1 || handle >= MAX_SOURCEFILES as c_int {
        return 0;
    }
    if bot.sourceFiles[handle as usize].is_none() {
        return 0;
    }

    let mut source = bot.sourceFiles[handle as usize].take().unwrap();
    let mut token = Token::default();
    let ret = PC_ReadToken(bot, &mut source, &mut token);
    bot.sourceFiles[handle as usize] = Some(source);

    // strip on the Rust side before the bounded copy (equivalent bytes)
    if token.type_ == TT_STRING && token.string.as_bytes().first() != Some(&b'@') {
        StripDoubleQuotes(&mut token.string);
    }
    unsafe {
        let bytes = token.string.as_bytes();
        let n = bytes.len().min(MAX_TOKENLENGTH - 1);
        for i in 0..n {
            (*pc_token).string[i] = bytes[i] as c_char;
        }
        (*pc_token).string[n] = 0;
        (*pc_token).type_ = token.type_;
        (*pc_token).subtype = token.subtype;
        (*pc_token).intvalue = token.intvalue as c_int;
        (*pc_token).floatvalue = token.floatvalue as f32;
    }

    ret
}
