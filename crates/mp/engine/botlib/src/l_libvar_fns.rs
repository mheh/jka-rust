#![allow(
    non_camel_case_types,
    non_snake_case,
    unused_variables,
    unused_assignments
)]

//! Function bodies for Raven's `l_libvar.cpp` (bot library variables —
//! a cvar-like linked list of `libvar_t` nodes).
//!
//! Ported per the engine C-track packets (`botlib__0581`..`botlib__1499`).
//! Source: `oracle/codemp/botlib/l_libvar.cpp`.
//!
// The `bot: &mut BotLib` receiver named in every signature below is the
// campaign's threaded-state aggregate (ruling 2); `BotLib` does not exist in
// this worktree slice yet (`_PREAMBLE.md`'s "botlib waves" note,
// `be_aas_main.rs`/`be_aas_debug_fns.rs` precedent). The file-scope global
// `libvarlist` is reached as a field on `bot` — resolved when the aggregate
// lands.

use core::ffi::c_char;

use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::l_libvar::libvar_s::libvar_t;
use crate::BotLib;

// PORT-NOTE(fwd-decl): these callees are already-ported per their packets but
// their owning modules are not registered/reachable from this crate slice
// yet; forward-declared exactly as their resolved signatures, matching the
// established `be_aas_main.rs` precedent for not-yet-wired in-crate callees.
extern "C" {
    fn Com_Memset(dest: *mut (), val: core::ffi::c_int, count: usize);
    fn GetMemory(bot: &mut BotLib, size: core::ffi::c_ulong) -> *mut ();
    fn FreeMemory(bot: &mut BotLib, ptr: *mut ());
    // PORT-NOTE(Q_stricmp): the packet's rosetta routes this through the
    // "already-ported qshared surface", but no `Q_stricmp` exists in the
    // engine-reachable `mp_qshared` tier yet (only `mp_game`'s copy, a
    // different crate this engine slice does not depend on) — forward-declared
    // like the established not-yet-wired extern precedent pending that move.
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> core::ffi::c_int;
}

/// `LibVarStringValue` — parses a numeric string into a float value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:28-59`
pub fn LibVarStringValue(string: *mut c_char) -> f32 {
    let mut dotfound: i32 = 0;
    let mut value: f32 = 0.0;
    let mut p = string;
    unsafe {
        while *p != 0 {
            let c = *p;
            if !(c as u8 as char).is_ascii_digit() {
                if dotfound != 0 || c != b'.' as c_char {
                    return 0.0;
                } else {
                    dotfound = 10;
                    p = p.add(1);
                }
            }
            if dotfound != 0 {
                value += (*p - b'0' as c_char) as f32 / dotfound as f32;
                dotfound *= 10;
            } else {
                value = value * 10.0 + (*p - b'0' as c_char) as f32;
            }
            p = p.add(1);
        }
    }
    value
}

/// `LibVarGet` — looks up a library variable by name.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:113-125`
pub fn LibVarGet(bot: &mut BotLib, var_name: *mut c_char) -> *mut libvar_t {
    let mut v = bot.libvarlist;
    unsafe {
        while !v.is_null() {
            if Q_stricmp((*v).name, var_name) == 0 {
                return v;
            }
            v = (*v).next;
        }
    }
    core::ptr::null_mut()
}

/// `LibVarAlloc` — allocates a new library variable and links it into
/// `libvarlist`.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:66-78`
pub fn LibVarAlloc(bot: &mut BotLib, var_name: *mut c_char) -> *mut libvar_t {
    unsafe {
        let name_len = crate::l_libvar_fns::strlen(var_name);
        let v = GetMemory(
            bot,
            (core::mem::size_of::<libvar_t>() + name_len + 1) as core::ffi::c_ulong,
        ) as *mut libvar_t;
        Com_Memset(v as *mut (), 0, core::mem::size_of::<libvar_t>());
        (*v).name = (v as *mut c_char).add(core::mem::size_of::<libvar_t>());
        crate::l_libvar_fns::strcpy((*v).name, var_name);
        // add the variable in the list
        (*v).next = bot.libvarlist;
        bot.libvarlist = v;
        v
    }
}

/// `LibVarDeAlloc` — frees a library variable's string and node.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:85-89`
pub fn LibVarDeAlloc(bot: &mut BotLib, v: *mut libvar_t) {
    unsafe {
        if !(*v).string.is_null() {
            FreeMemory(bot, (*v).string as *mut ());
        }
        FreeMemory(bot, v as *mut ());
    }
}

/// `LibVarGetString` — returns a variable's string value, or `""` if unset.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:132-145`
pub fn LibVarGetString(bot: &mut BotLib, var_name: *mut c_char) -> *mut c_char {
    let v = LibVarGet(bot, var_name);
    if !v.is_null() {
        unsafe { (*v).string }
    } else {
        c"".as_ptr() as *mut c_char
    }
}

/// `LibVarGetValue` — returns a variable's float value, or `0` if unset.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:152-165`
pub fn LibVarGetValue(bot: &mut BotLib, var_name: *mut c_char) -> f32 {
    let v = LibVarGet(bot, var_name);
    if !v.is_null() {
        unsafe { (*v).value }
    } else {
        0.0
    }
}

/// `LibVarChanged` — returns whether a variable has been modified since last
/// check.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:248-261`
pub fn LibVarChanged(bot: &mut BotLib, var_name: *mut c_char) -> qboolean {
    let v = LibVarGet(bot, var_name);
    if !v.is_null() {
        unsafe { (*v).modified }
    } else {
        qfalse
    }
}

/// `LibVarSetNotModified` — clears a variable's modified flag.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:268-277`
pub fn LibVarSetNotModified(bot: &mut BotLib, var_name: *mut c_char) {
    let v = LibVarGet(bot, var_name);
    if !v.is_null() {
        unsafe {
            (*v).modified = qfalse;
        }
    }
}

/// `LibVarDeAllocAll` — frees every library variable and resets the list.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:96-106`
pub fn LibVarDeAllocAll(bot: &mut BotLib) {
    let mut v = bot.libvarlist;
    while !v.is_null() {
        bot.libvarlist = unsafe { (*v).next };
        LibVarDeAlloc(bot, v);
        v = bot.libvarlist;
    }
    bot.libvarlist = core::ptr::null_mut();
}

/// `LibVar` — gets an existing variable, or creates one with the given
/// default value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:172-188`
pub fn LibVar(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) -> *mut libvar_t {
    let mut v = LibVarGet(bot, var_name);
    if !v.is_null() {
        return v;
    }
    // create new variable
    v = LibVarAlloc(bot, var_name);
    unsafe {
        // variable string
        let len = crate::l_libvar_fns::strlen(value);
        (*v).string = GetMemory(bot, (len + 1) as core::ffi::c_ulong) as *mut c_char;
        crate::l_libvar_fns::strcpy((*v).string, value);
        // the value
        (*v).value = LibVarStringValue((*v).string);
        // variable is modified
        (*v).modified = qtrue;
    }
    v
}

/// `LibVarSet` — sets a variable's value, allocating it if it doesn't exist.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:221-241`
pub fn LibVarSet(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) {
    let mut v = LibVarGet(bot, var_name);
    unsafe {
        if !v.is_null() {
            FreeMemory(bot, (*v).string as *mut ());
        } else {
            v = LibVarAlloc(bot, var_name);
        }
        // variable string
        let len = crate::l_libvar_fns::strlen(value);
        (*v).string = GetMemory(bot, (len + 1) as core::ffi::c_ulong) as *mut c_char;
        crate::l_libvar_fns::strcpy((*v).string, value);
        // the value
        (*v).value = LibVarStringValue((*v).string);
        // variable is modified
        (*v).modified = qtrue;
    }
}

/// `LibVarString` — gets-or-creates a variable, returning its string value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:195-201`
pub fn LibVarString(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) -> *mut c_char {
    let v = LibVar(bot, var_name, value);
    unsafe { (*v).string }
}

/// `LibVarValue` — gets-or-creates a variable, returning its float value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:208-214`
pub fn LibVarValue(bot: &mut BotLib, var_name: *mut c_char, value: *mut c_char) -> f32 {
    let v = LibVar(bot, var_name, value);
    unsafe { (*v).value }
}

// PORT-NOTE(strcpy/strlen): externals per the packets; `strlen`/`strcpy`
// are libc byte-string helpers over raw `c_char` pointers, not yet routed
// through a shared crate wrapper in this slice — forward-declared like the
// `be_aas_main.rs` precedent for not-yet-wired externs.
extern "C" {
    fn strlen(s: *const c_char) -> usize;
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}
