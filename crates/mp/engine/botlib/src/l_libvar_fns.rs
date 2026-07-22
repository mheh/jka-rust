#![allow(non_camel_case_types, non_snake_case)]

//! Function bodies for Raven's `l_libvar.cpp` (bot library variables —
//! a cvar-like list of library variables).
//!
//! Redesigned per porting-rules §F17: Raven's malloc'd `libvar_t` linked list
//! is an owned `Vec<LibVar>` arena on `BotLib` (`libvars`), reached by index
//! (`LibVarHandle`) instead of raw pointer. Internal signatures take `&str` and
//! return owned `String`/`f32`/`LibVarHandle`; the C-string `char *` seam is
//! confined to the `Export_BotLibVar{Get,Set}` adapters in `be_interface_fns`.
//!
//! Source: `oracle/codemp/botlib/l_libvar.cpp`.

use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::l_libvar::libvar_s::{LibVar, LibVarHandle};
use crate::BotLib;

/// `LibVarStringValue` — parses a numeric string into a float value.
///
/// Faithful port of Raven's hand-rolled parser: leading digits (and one `.`)
/// build the value, any other character aborts with `0`.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:28-59`
pub fn LibVarStringValue(string: &str) -> f32 {
    let bytes = string.as_bytes();
    let mut dotfound: i32 = 0;
    let mut value: f32 = 0.0;
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if !c.is_ascii_digit() {
            if dotfound != 0 || c != b'.' {
                return 0.0;
            } else {
                dotfound = 10;
                i += 1;
            }
        }
        // Raven reads `*string` unconditionally here; a trailing `.` would run
        // it past the terminator (UB). Guard it (porting-rules §F19) — no real
        // libvar default ("0.7", "800", …) trails with a dot.
        if i >= bytes.len() {
            break;
        }
        if dotfound != 0 {
            value += (bytes[i] - b'0') as f32 / dotfound as f32;
            dotfound *= 10;
        } else {
            value = value * 10.0 + (bytes[i] - b'0') as f32;
        }
        i += 1;
    }
    value
}

/// `LibVarGet` — looks up a library variable by name (case-insensitive walk,
/// Raven's `Q_stricmp`), returning its handle if present.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:113-125`
pub fn LibVarGet(bot: &BotLib, var_name: &str) -> Option<LibVarHandle> {
    bot.libvars
        .iter()
        .position(|v| v.name.eq_ignore_ascii_case(var_name))
        .map(LibVarHandle)
}

/// `LibVarAlloc` — appends a new library variable and returns its handle.
///
/// Raven prepended to the malloc'd list; the arena appends instead. Names are
/// unique (callers `LibVarGet` before allocating), so list order never affects
/// lookup results.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:66-78`
pub fn LibVarAlloc(bot: &mut BotLib, var_name: &str) -> LibVarHandle {
    bot.libvars.push(LibVar {
        name: var_name.to_owned(),
        string: String::new(),
        modified: qfalse,
        value: 0.0,
    });
    LibVarHandle(bot.libvars.len() - 1)
}

/// `LibVarDeAllocAll` — removes all library variables and resets the list.
///
/// Raven walked the list freeing each node's string and node (`LibVarDeAlloc`,
/// which has no other caller); dropping the owned `Vec` reclaims both.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:96-106`
pub fn LibVarDeAllocAll(bot: &mut BotLib) {
    bot.libvars.clear();
}

/// `LibVarGetString` — returns a variable's string value, or `""` if unset.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:132-145`
pub fn LibVarGetString(bot: &BotLib, var_name: &str) -> String {
    match LibVarGet(bot, var_name) {
        Some(h) => bot.libvar(h).string.clone(),
        None => String::new(),
    }
}

/// `LibVarGetValue` — returns a variable's float value, or `0` if unset.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:152-165`
pub fn LibVarGetValue(bot: &BotLib, var_name: &str) -> f32 {
    match LibVarGet(bot, var_name) {
        Some(h) => bot.libvar(h).value,
        None => 0.0,
    }
}

/// `LibVar` — gets an existing variable, or creates one with the given default
/// value, returning its handle.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:172-188`
pub fn LibVar(bot: &mut BotLib, var_name: &str, value: &str) -> LibVarHandle {
    if let Some(h) = LibVarGet(bot, var_name) {
        return h;
    }
    // create new variable
    let h = LibVarAlloc(bot, var_name);
    let parsed = LibVarStringValue(value);
    let v = &mut bot.libvars[h.0];
    v.string = value.to_owned();
    v.value = parsed;
    v.modified = qtrue;
    h
}

/// `LibVarString` — gets-or-creates a variable, returning its string value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:195-201`
pub fn LibVarString(bot: &mut BotLib, var_name: &str, value: &str) -> String {
    let h = LibVar(bot, var_name, value);
    bot.libvar(h).string.clone()
}

/// `LibVarValue` — gets-or-creates a variable, returning its float value.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:208-214`
pub fn LibVarValue(bot: &mut BotLib, var_name: &str, value: &str) -> f32 {
    let h = LibVar(bot, var_name, value);
    bot.libvar(h).value
}

/// `LibVarSet` — sets a variable's value, allocating it if it doesn't exist.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:221-241`
pub fn LibVarSet(bot: &mut BotLib, var_name: &str, value: &str) {
    let h = match LibVarGet(bot, var_name) {
        Some(h) => h,
        None => LibVarAlloc(bot, var_name),
    };
    let parsed = LibVarStringValue(value);
    let v = &mut bot.libvars[h.0];
    v.string = value.to_owned();
    v.value = parsed;
    v.modified = qtrue;
}

/// `LibVarChanged` — returns whether a variable has been modified since last
/// check.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:248-261`
pub fn LibVarChanged(bot: &BotLib, var_name: &str) -> qboolean {
    match LibVarGet(bot, var_name) {
        Some(h) => bot.libvar(h).modified,
        None => qfalse,
    }
}

/// `LibVarSetNotModified` — clears a variable's modified flag.
///
/// Source: `oracle/codemp/botlib/l_libvar.cpp:268-277`
pub fn LibVarSetNotModified(bot: &mut BotLib, var_name: &str) {
    if let Some(h) = LibVarGet(bot, var_name) {
        bot.libvars[h.0].modified = qfalse;
    }
}
