//! `cvar.cpp` — dynamic variable tracking.
//!
//! Destination `_fns` escape: the `cvar/` directory already holds the cvar
//! types/consts, so `cvar.cpp`'s functions land here (per packet DESTINATION).
//!
//! Source: `oracle/codemp/qcommon/cvar.cpp`

use core::ffi::{c_char, c_float, c_int, c_long, c_uint, CStr};

use native_types::fileHandle_t;

use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::cvar::{
    cvarHandle_t, cvar_t, vmCvar_t, CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INIT, CVAR_INTERNAL, CVAR_LATCH,
    CVAR_NORESTART, CVAR_ROM, CVAR_SERVERINFO, CVAR_SYSTEMINFO, CVAR_USERINFO, CVAR_USER_CREATED,
    MAX_CVAR_VALUE_STRING,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_STRING_TOKENS;
use mp_qshared::shared::q_string::{
    Info_SetValueForKey, Info_SetValueForKey_Big, Q_stricmp, Q_strncpyz,
};
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::cmd::Cmd_AddCommand;
use crate::cmd_common::{Cmd_Argc, Cmd_Argv};
use crate::common::engine_host_view::EngineHostView;
use crate::common::error::com_error;
use crate::common::{com_printf, Common};
use crate::common_fns::Com_Filter;
use crate::cvar::cvar_consts::{FILE_HASH_SIZE, MAX_CVARS};
use crate::files_common::FS_Printf;
use crate::z_memman_pc::{CopyString, Z_Free, Z_Malloc};

/// `strchr(s, c) != NULL` — scans the NUL-terminated string for `c`.
///
/// Local libc mirror; house rule: libc symbols use the Rust equivalent, no
/// resolved signature needed.
unsafe fn strchr_present(s: *const c_char, c: c_char) -> bool {
    let mut p = s;
    loop {
        if *p == c {
            return true;
        }
        if *p == 0 {
            return false;
        }
        p = p.offset(1);
    }
}

/// `Cvar_FreeString`.
///
/// Raven: if the string came from the memory pool, don't really free it — the
/// entire memory pool will be wiped during the next level load.
/// Source: `oracle/codemp/qcommon/cvar.cpp:26-32`
pub fn Cvar_FreeString(common: &mut Common, string: *mut c_char) {
    if common.cvar_lastMemPool.is_null()
        || (string as usize) < (common.cvar_lastMemPool as usize)
        || (string as usize)
            >= (common.cvar_lastMemPool as usize + common.cvar_memPoolSize as usize)
    {
        Z_Free(common, string as *mut ());
    }
}

/// Raven `generateHashValue` — cvar-name hash (file-local static).
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:41-55`
pub fn generateHashValue(fname: *const c_char) -> c_long {
    let mut hash: c_long = 0;
    let mut i: c_int = 0;
    unsafe {
        while *fname.offset(i as isize) != 0 {
            // tolower((unsigned char)fname[i]) stored into `char letter`.
            let letter = (*fname.offset(i as isize) as u8).to_ascii_lowercase() as c_char;
            hash += (letter as c_long) * ((i + 119) as c_long);
            i += 1;
        }
    }
    hash &= FILE_HASH_SIZE as c_long - 1;
    hash
}

/// Raven `Cvar_ValidateString`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:62-76`
pub fn Cvar_ValidateString(s: *const c_char) -> qboolean {
    unsafe {
        if s.is_null() {
            return qfalse;
        }
        if strchr_present(s, b'\\' as c_char) {
            return qfalse;
        }
        if strchr_present(s, b'"' as c_char) {
            return qfalse;
        }
        if strchr_present(s, b';' as c_char) {
            return qfalse;
        }
        qtrue
    }
}

/// Raven `Cvar_FindVar`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:83-96`
pub fn Cvar_FindVar(common: &mut Common, var_name: *const c_char) -> *mut cvar_t {
    let hash = generateHashValue(var_name);
    let mut var: *mut cvar_t = common.cvar_hashTable[hash as usize];
    unsafe {
        while !var.is_null() {
            if Q_stricmp(var_name, (*var).name) == 0 {
                return var;
            }
            var = (*var).hashNext;
        }
    }
    core::ptr::null_mut()
}

/// Raven `Cvar_VariableValue`.
///
/// Divergence: the sole in-engine caller (`vm_fns`) threads the full pinned
/// receiver set, so this keeps all four though only `common` is used.
/// Source: `oracle/codemp/qcommon/cvar.cpp:103-110`
pub fn Cvar_VariableValue(view: &mut EngineHostView, var_name: *const c_char) -> f32 {
    let var = Cvar_FindVar(view.common, var_name);
    if var.is_null() {
        return 0.0;
    }
    unsafe { (*var).value }
}

/// Raven `Cvar_VariableIntegerValue`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:118-125`
pub fn Cvar_VariableIntegerValue(common: &mut Common, var_name: *const c_char) -> c_int {
    let var = Cvar_FindVar(common, var_name);
    if var.is_null() {
        return 0;
    }
    unsafe { (*var).integer }
}

/// Raven `Cvar_VariableString`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:133-140`
pub fn Cvar_VariableString(common: &mut Common, var_name: *const c_char) -> *mut c_char {
    let var = Cvar_FindVar(common, var_name);
    if var.is_null() {
        return c"".as_ptr() as *mut c_char;
    }
    unsafe { (*var).string }
}

/// Raven `Cvar_VariableStringBuffer`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:148-158`
pub fn Cvar_VariableStringBuffer(
    common: &mut Common,
    var_name: *const c_char,
    buffer: *mut c_char,
    bufsize: c_int,
) {
    let var = Cvar_FindVar(common, var_name);
    unsafe {
        if var.is_null() {
            *buffer = 0;
        } else {
            Q_strncpyz(buffer, (*var).string, bufsize);
        }
    }
}

/// Raven `Cvar_CommandCompletion`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:166-177`
pub fn Cvar_CommandCompletion(common: &mut Common, callback: fn(*const c_char)) {
    let mut cvar: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !cvar.is_null() {
            // Dont show internal cvars
            if (*cvar).flags & CVAR_INTERNAL != 0 {
                cvar = (*cvar).next;
                continue;
            }
            callback((*cvar).name);
            cvar = (*cvar).next;
        }
    }
}

/// Raven `Cvar_Get`.
///
/// Raven: if the variable already exists, the value will not be set unless
/// CVAR_ROM; the flags will be or'ed in if the variable exists.
/// Source: `oracle/codemp/qcommon/cvar.cpp:188-280`
pub fn Cvar_Get(
    view: &mut EngineHostView,
    var_name: *const c_char,
    var_value: *const c_char,
    flags: c_int,
) -> *mut cvar_t {
    let mut var_name = var_name;

    unsafe {
        if var_name.is_null() || var_value.is_null() {
            com_error(
                errorParm_t::ERR_FATAL,
                "Cvar_Get: NULL parameter".to_string(),
            );
        }

        if Cvar_ValidateString(var_name) == qfalse {
            com_printf(
                view.common,
                &format!(
                    "invalid cvar name string: {}\n",
                    CStr::from_ptr(var_name).to_string_lossy()
                ),
            );
            var_name = c"BADNAME".as_ptr();
        }

        // `#if 0` backslash-value validation is compiled out in this build.

        let var = Cvar_FindVar(view.common, var_name);
        if !var.is_null() {
            // if the C code is now specifying a variable that the user already
            // set a value for, take the new value as the reset value
            if ((*var).flags & CVAR_USER_CREATED) != 0
                && (flags & CVAR_USER_CREATED) == 0
                && *var_value != 0
            {
                (*var).flags &= !CVAR_USER_CREATED;
                Cvar_FreeString(view.common, (*var).resetString);
                (*var).resetString = CopyString(view, var_value);

                // ZOID -- needs to be set so that cvars the game sets as
                // SERVERINFO get sent to clients
                view.common.cvar_modifiedFlags |= flags;
            }

            (*var).flags |= flags;
            // only allow one non-empty reset string without a warning
            if *(*var).resetString == 0 {
                // we don't have a reset string yet
                Cvar_FreeString(view.common, (*var).resetString);
                (*var).resetString = CopyString(view, var_value);
            } else if *var_value != 0 && libc::strcmp((*var).resetString, var_value) != 0 {
                crate::common_fns::Com_DPrintf(
                    view.common,
                    &format!(
                        "Warning: cvar \"{}\" given initial values: \"{}\" and \"{}\"\n",
                        CStr::from_ptr(var_name).to_string_lossy(),
                        CStr::from_ptr((*var).resetString).to_string_lossy(),
                        CStr::from_ptr(var_value).to_string_lossy()
                    ),
                );
            }
            // if we have a latched string, take that value now
            if !(*var).latchedString.is_null() {
                let s = (*var).latchedString;
                (*var).latchedString = core::ptr::null_mut(); // otherwise cvar_set2 would free it
                Cvar_Set2(view, var_name, s, qtrue);
                Cvar_FreeString(view.common, s);
            }

            // `#if 0` CVAR_ROM-override block is compiled out in this build.
            return var;
        }

        //
        // allocate a new cvar
        //
        if view.common.cvar_numIndexes >= MAX_CVARS as c_int {
            com_error(errorParm_t::ERR_FATAL, "MAX_CVARS".to_string());
        }
        let var =
            &mut view.common.cvar_indexes[view.common.cvar_numIndexes as usize] as *mut cvar_t;
        view.common.cvar_numIndexes += 1;
        (*var).name = CopyString(view, var_name);
        (*var).string = CopyString(view, var_value);
        (*var).modified = qtrue;
        (*var).modificationCount = 1;
        (*var).value = libc::atof((*var).string) as c_float;
        (*var).integer = libc::atoi((*var).string);
        (*var).resetString = CopyString(view, var_value);

        // link the variable in
        (*var).next = view.common.cvar_vars;
        view.common.cvar_vars = var;

        (*var).flags = flags;

        let hash = generateHashValue(var_name);
        (*var).hashNext = view.common.cvar_hashTable[hash as usize];
        view.common.cvar_hashTable[hash as usize] = var;

        var
    }
}

/// Raven `Cvar_Set2`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:287-395`
pub fn Cvar_Set2(
    view: &mut EngineHostView,
    var_name: *const c_char,
    value: *const c_char,
    force: qboolean,
) -> *mut cvar_t {
    let mut var_name = var_name;
    let mut value = value;

    unsafe {
        if Cvar_ValidateString(var_name) == qfalse {
            com_printf(
                view.common,
                &format!(
                    "invalid cvar name string: {}\n",
                    CStr::from_ptr(var_name).to_string_lossy()
                ),
            );
            var_name = c"BADNAME".as_ptr();
        }

        // `#if 0` value validation is compiled out in this build.

        let var = Cvar_FindVar(view.common, var_name);
        if var.is_null() {
            if value.is_null() {
                return core::ptr::null_mut();
            }
            // create it
            if force == qfalse {
                return Cvar_Get(view, var_name, value, CVAR_USER_CREATED);
            } else {
                return Cvar_Get(view, var_name, value, 0);
            }
        }

        // Dont display the update when its internal
        if ((*var).flags & CVAR_INTERNAL) == 0 {
            crate::common_fns::Com_DPrintf(
                view.common,
                &format!(
                    "Cvar_Set2: {} {}\n",
                    CStr::from_ptr(var_name).to_string_lossy(),
                    CStr::from_ptr(value).to_string_lossy()
                ),
            );
        }

        if value.is_null() {
            value = (*var).resetString;
        }

        if libc::strcmp(value, (*var).string) == 0 {
            return var;
        }
        // note what types of cvars have been modified (userinfo, archive,
        // serverinfo, systeminfo)
        view.common.cvar_modifiedFlags |= (*var).flags;

        if force == qfalse {
            if ((*var).flags & CVAR_ROM) != 0 {
                com_printf(
                    view.common,
                    &format!(
                        "{} is read only.\n",
                        CStr::from_ptr(var_name).to_string_lossy()
                    ),
                );
                return var;
            }

            if ((*var).flags & CVAR_INIT) != 0 {
                com_printf(
                    view.common,
                    &format!(
                        "{} is write protected.\n",
                        CStr::from_ptr(var_name).to_string_lossy()
                    ),
                );
                return var;
            }

            if ((*var).flags & CVAR_LATCH) != 0 {
                if !(*var).latchedString.is_null() {
                    if libc::strcmp(value, (*var).latchedString) == 0 {
                        return var;
                    }
                    Cvar_FreeString(view.common, (*var).latchedString);
                } else if libc::strcmp(value, (*var).string) == 0 {
                    return var;
                }

                com_printf(
                    view.common,
                    &format!(
                        "{} will be changed upon restarting.\n",
                        CStr::from_ptr(var_name).to_string_lossy()
                    ),
                );
                (*var).latchedString = CopyString(view, value);
                (*var).modified = qtrue;
                (*var).modificationCount += 1;
                return var;
            }

            if ((*var).flags & CVAR_CHEAT) != 0 && (*view.common.cvar_cheats).integer == 0 {
                com_printf(
                    view.common,
                    &format!(
                        "{} is cheat protected.\n",
                        CStr::from_ptr(var_name).to_string_lossy()
                    ),
                );
                return var;
            }
        } else if !(*var).latchedString.is_null() {
            Cvar_FreeString(view.common, (*var).latchedString);
            (*var).latchedString = core::ptr::null_mut();
        }

        if libc::strcmp(value, (*var).string) == 0 {
            return var; // not changed
        }

        (*var).modified = qtrue;
        (*var).modificationCount += 1;

        Cvar_FreeString(view.common, (*var).string); // free the old value string

        (*var).string = CopyString(view, value);
        (*var).value = libc::atof((*var).string) as c_float;
        (*var).integer = libc::atoi((*var).string);

        var
    }
}

/// Raven `Cvar_Set`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:402-404`
pub fn Cvar_Set(view: &mut EngineHostView, var_name: *const c_char, value: *const c_char) {
    Cvar_Set2(view, var_name, value, qtrue);
}

/// Raven `Cvar_SetLatched`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:411-413`
pub fn Cvar_SetLatched(view: &mut EngineHostView, var_name: *const c_char, value: *const c_char) {
    Cvar_Set2(view, var_name, value, qfalse);
}

/// Raven `Cvar_SetValue`.
///
/// The Raven `Com_sprintf` renders are ported via the CString/`format!`
/// pre-render idiom (`Com_sprintf` has no landed qcommon home).
/// Source: `oracle/codemp/qcommon/cvar.cpp:420-429`
pub fn Cvar_SetValue(view: &mut EngineHostView, var_name: *const c_char, value: f32) {
    let val = if value == (value as c_int) as f32 {
        std::ffi::CString::new(format!("{}", value as c_int)).unwrap()
    } else {
        std::ffi::CString::new(format!("{value:.6}")).unwrap()
    };
    Cvar_Set(view, var_name, val.as_ptr());
}

/// Raven `Cvar_Reset`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:437-439`
pub fn Cvar_Reset(view: &mut EngineHostView, var_name: *const c_char) {
    Cvar_Set2(view, var_name, core::ptr::null(), qfalse);
}

/// Raven `Cvar_SetCheatState`.
///
/// Raven: any testing variables will be reset to the safe values.
/// Source: `oracle/codemp/qcommon/cvar.cpp:449-467`
pub fn Cvar_SetCheatState(view: &mut EngineHostView) {
    // set all default vars to the safe value
    let mut var: *mut cvar_t = view.common.cvar_vars;
    unsafe {
        while !var.is_null() {
            if ((*var).flags & CVAR_CHEAT) != 0 {
                // the CVAR_LATCHED|CVAR_CHEAT vars might escape the reset here
                // because of a different var->latchedString
                if !(*var).latchedString.is_null() {
                    Cvar_FreeString(view.common, (*var).latchedString);
                    (*var).latchedString = core::ptr::null_mut();
                }
                if libc::strcmp((*var).resetString, (*var).string) != 0 {
                    let name = (*var).name;
                    let reset = (*var).resetString;
                    Cvar_Set(view, name, reset);
                }
            }
            var = (*var).next;
        }
    }
}

/// Raven `Cvar_Command` — handles variable inspection and changing from the
/// console.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:476-515`
pub fn Cvar_Command(view: &mut EngineHostView) -> qboolean {
    unsafe {
        // check variables
        let arg0 = Cmd_Argv(view.common, 0);
        let v = Cvar_FindVar(view.common, arg0);
        if v.is_null() {
            return qfalse;
        }

        // perform a variable print or set
        if Cmd_Argc(view.common) == 1 {
            // `S_COLOR_WHITE` = "^7" (oracle/codemp/game/q_shared.h:1167).
            com_printf(
                view.common,
                &format!(
                    "\"{}\" is:\"{}^7\" default:\"{}^7\"\n",
                    CStr::from_ptr((*v).name).to_string_lossy(),
                    CStr::from_ptr((*v).string).to_string_lossy(),
                    CStr::from_ptr((*v).resetString).to_string_lossy()
                ),
            );
            if !(*v).latchedString.is_null() {
                com_printf(
                    view.common,
                    &format!(
                        "latched: \"{}\"\n",
                        CStr::from_ptr((*v).latchedString).to_string_lossy()
                    ),
                );
            }
            return qtrue;
        }

        // JFM toggle test
        let value = Cmd_Argv(view.common, 1);
        let name = (*v).name;
        if *value == b'!' as c_char {
            // toggle
            let nv = if (*v).value == 0.0 { 1 } else { 0 };
            let buff = std::ffi::CString::new(format!("{nv}")).unwrap();
            Cvar_Set2(view, name, buff.as_ptr(), qfalse); // toggle the value
        } else {
            Cvar_Set2(view, name, value, qfalse); // set the value if forcing isn't required
        }

        qtrue
    }
}

/// Raven `Cvar_Toggle_f` — toggles a cvar for easy single key binding.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:525-537`
pub fn Cvar_Toggle_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "usage: toggle <variable>\n");
        return;
    }

    let arg1 = Cmd_Argv(view.common, 1);
    let mut v = Cvar_VariableValue(view, arg1) as c_int;
    v = if v == 0 { 1 } else { 0 };

    let arg1 = Cmd_Argv(view.common, 1);
    let val = std::ffi::CString::new(format!("{v}")).unwrap();
    Cvar_Set2(view, arg1, val.as_ptr(), qfalse);
}

/// Raven `Cvar_Set_f` — allows setting and defining of arbitrary cvars from
/// console, even if they weren't declared in C code.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:547-571`
pub fn Cvar_Set_f(view: &mut EngineHostView) {
    let c = Cmd_Argc(view.common);
    if c < 3 {
        com_printf(view.common, "usage: set <variable> <value>\n");
        return;
    }

    let mut combined = [0 as c_char; MAX_STRING_TOKENS];
    combined[0] = 0;
    let mut l: c_int = 0;
    unsafe {
        for i in 2..c {
            let arg = Cmd_Argv(view.common, i);
            let len = libc::strlen(arg.add(1)) as c_int;
            if l + len >= MAX_STRING_TOKENS as c_int - 2 {
                break;
            }
            libc::strcat(combined.as_mut_ptr(), arg);
            if i != c - 1 {
                libc::strcat(combined.as_mut_ptr(), c" ".as_ptr());
            }
            l += len;
        }
        let arg1 = Cmd_Argv(view.common, 1);
        Cvar_Set2(view, arg1, combined.as_ptr(), qfalse);
    }
}

/// Raven `Cvar_SetU_f` — as `Cvar_Set`, but also flags it as userinfo.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:580-593`
pub fn Cvar_SetU_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 3 {
        com_printf(view.common, "usage: setu <variable> <value>\n");
        return;
    }
    Cvar_Set_f(view);
    let arg1 = Cmd_Argv(view.common, 1);
    let v = Cvar_FindVar(view.common, arg1);
    if v.is_null() {
        return;
    }
    unsafe {
        (*v).flags |= CVAR_USERINFO;
    }
}

/// Raven `Cvar_SetS_f` — as `Cvar_Set`, but also flags it as serverinfo.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:602-615`
pub fn Cvar_SetS_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 3 {
        com_printf(view.common, "usage: sets <variable> <value>\n");
        return;
    }
    Cvar_Set_f(view);
    let arg1 = Cmd_Argv(view.common, 1);
    let v = Cvar_FindVar(view.common, arg1);
    if v.is_null() {
        return;
    }
    unsafe {
        (*v).flags |= CVAR_SERVERINFO;
    }
}

/// Raven `Cvar_SetA_f` — as `Cvar_Set`, but also flags it as archived.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:624-637`
pub fn Cvar_SetA_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 3 {
        com_printf(view.common, "usage: seta <variable> <value>\n");
        return;
    }
    Cvar_Set_f(view);
    let arg1 = Cmd_Argv(view.common, 1);
    let v = Cvar_FindVar(view.common, arg1);
    if v.is_null() {
        return;
    }
    unsafe {
        (*v).flags |= CVAR_ARCHIVE;
    }
}

/// Raven `Cvar_Reset_f`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:644-650`
pub fn Cvar_Reset_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "usage: reset <variable>\n");
        return;
    }
    let arg1 = Cmd_Argv(view.common, 1);
    Cvar_Reset(view, arg1);
}

/// Raven `Cvar_WriteVariables` — appends lines containing "set variable value"
/// for all variables with the archive flag set to qtrue.
///
/// The Raven `Com_sprintf`/`FS_Printf` pair is ported via the `format!`
/// pre-render idiom; the `USE_CD_KEY` `cl_cdkey` skip is compiled out.
/// Source: `oracle/codemp/qcommon/cvar.cpp:660-680`
pub fn Cvar_WriteVariables(common: &mut Common, f: fileHandle_t) {
    let mut var: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !var.is_null() {
            if ((*var).flags & CVAR_ARCHIVE) != 0 {
                // write the latched value, even if it hasn't taken effect yet
                let buffer = if !(*var).latchedString.is_null() {
                    format!(
                        "seta {} \"{}\"\n",
                        CStr::from_ptr((*var).name).to_string_lossy(),
                        CStr::from_ptr((*var).latchedString).to_string_lossy()
                    )
                } else {
                    format!(
                        "seta {} \"{}\"\n",
                        CStr::from_ptr((*var).name).to_string_lossy(),
                        CStr::from_ptr((*var).string).to_string_lossy()
                    )
                };
                FS_Printf(common, f, &buffer);
            }
            var = (*var).next;
        }
    }
}

/// Raven `Cvar_List_f`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:687-750`
pub fn Cvar_List_f(common: &mut Common) {
    let r#match: *mut c_char = if Cmd_Argc(common) > 1 {
        Cmd_Argv(common, 1)
    } else {
        core::ptr::null_mut()
    };

    let mut i: c_int = 0;
    let mut var: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !var.is_null() {
            // `i` counts every cvar (Raven increments in the loop step, so
            // `continue`d internal/unmatched cvars are still counted).
            let cur = var;
            var = (*cur).next;
            i += 1;

            // Dont show internal cvars
            if ((*cur).flags & CVAR_INTERNAL) != 0 {
                continue;
            }

            if !r#match.is_null() && !Com_Filter(r#match, (*cur).name, false) {
                continue;
            }

            let mut line = String::new();
            line.push(if ((*cur).flags & CVAR_SERVERINFO) != 0 {
                'S'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_USERINFO) != 0 {
                'U'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_ROM) != 0 {
                'R'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_INIT) != 0 {
                'I'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_ARCHIVE) != 0 {
                'A'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_LATCH) != 0 {
                'L'
            } else {
                ' '
            });
            line.push(if ((*cur).flags & CVAR_CHEAT) != 0 {
                'C'
            } else {
                ' '
            });
            line.push_str(&format!(
                " {} \"{}\"\n",
                CStr::from_ptr((*cur).name).to_string_lossy(),
                CStr::from_ptr((*cur).string).to_string_lossy()
            ));
            com_printf(common, &line);
        }
    }

    com_printf(common, &format!("\n{i} total cvars\n"));
    let numIndexes = common.cvar_numIndexes;
    com_printf(common, &format!("{numIndexes} cvar indexes\n"));
}

/// Raven `Cvar_Restart_f` — resets all cvars to their hardcoded values.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:759-802`
pub fn Cvar_Restart_f(view: &mut EngineHostView) {
    let mut prev: *mut *mut cvar_t = &mut view.common.cvar_vars as *mut _;
    loop {
        unsafe {
            let var = *prev;
            if var.is_null() {
                break;
            }

            // don't mess with rom values, or some inter-module communication
            // will get broken (com_cl_running, etc)
            if ((*var).flags & (CVAR_ROM | CVAR_INIT | CVAR_NORESTART)) != 0 {
                prev = &mut (*var).next as *mut _;
                continue;
            }

            // throw out any variables the user created
            if ((*var).flags & CVAR_USER_CREATED) != 0 {
                *prev = (*var).next;
                if !(*var).name.is_null() {
                    Cvar_FreeString(view.common, (*var).name);
                }
                if !(*var).string.is_null() {
                    Cvar_FreeString(view.common, (*var).string);
                }
                if !(*var).latchedString.is_null() {
                    Cvar_FreeString(view.common, (*var).latchedString);
                }
                if !(*var).resetString.is_null() {
                    Cvar_FreeString(view.common, (*var).resetString);
                }
                // clear the var completely, since we can't remove the index
                // from the list. Raven's `Com_Memset(var, 0, sizeof(var))`
                // zeroes only a pointer's worth of bytes (`sizeof(var)`, not
                // `sizeof(*var)`); ported faithfully.
                libc::memset(
                    var as *mut libc::c_void,
                    0,
                    core::mem::size_of::<*mut cvar_t>(),
                );
                continue;
            }

            let name = (*var).name;
            let reset = (*var).resetString;
            Cvar_Set(view, name, reset);

            prev = &mut (*var).next as *mut _;
        }
    }
}

/// Raven `Cvar_InfoString`.
///
/// The `static char info[MAX_INFO_STRING]` return buffer is the owning
/// `Common.cvar_info_string` field; the returned pointer aliases it exactly as
/// Raven's static.
/// Source: `oracle/codemp/qcommon/cvar.cpp:811-845`
pub fn Cvar_InfoString(common: &mut Common, bit: c_int) -> *mut c_char {
    common.cvar_info_string[0] = 0;
    let info = common.cvar_info_string.as_mut_ptr();

    let mut var: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !var.is_null() {
            if ((*var).flags & CVAR_INTERNAL) == 0 && ((*var).flags & bit) != 0 {
                Info_SetValueForKey(info, (*var).name, (*var).string);
            }
            var = (*var).next;
        }
    }
    // The `kungFuSafety` g_debugMelee block is commented out in the oracle.
    info
}

/// Raven `Cvar_InfoString_Big` — handles large info strings (`CS_SYSTEMINFO`).
///
/// The `static char info[BIG_INFO_STRING]` return buffer is the owning
/// `Common.cvar_info_string_big` field.
/// Source: `oracle/codemp/qcommon/cvar.cpp:854-869`
pub fn Cvar_InfoString_Big(common: &mut Common, bit: c_int) -> *mut c_char {
    common.cvar_info_string_big[0] = 0;
    let info = common.cvar_info_string_big.as_mut_ptr();

    let mut var: *mut cvar_t = common.cvar_vars;
    unsafe {
        while !var.is_null() {
            if ((*var).flags & CVAR_INTERNAL) == 0 && ((*var).flags & bit) != 0 {
                Info_SetValueForKey_Big(info, (*var).name, (*var).string);
            }
            var = (*var).next;
        }
    }
    info
}

/// Raven `Cvar_InfoStringBuffer`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:878-880`
pub fn Cvar_InfoStringBuffer(common: &mut Common, bit: c_int, buff: *mut c_char, buffsize: c_int) {
    let info = Cvar_InfoString(common, bit);
    Q_strncpyz(buff, info, buffsize);
}

/// Raven `Cvar_Register` — basically a slightly modified `Cvar_Get` for the
/// interpreted modules.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:889-899`
pub fn Cvar_Register(
    view: &mut EngineHostView,
    vmCvar: *mut vmCvar_t,
    varName: *const c_char,
    defaultValue: *const c_char,
    flags: c_int,
) {
    let cv = Cvar_Get(view, varName, defaultValue, flags);
    if vmCvar.is_null() {
        return;
    }
    unsafe {
        (*vmCvar).handle =
            (cv as *const cvar_t).offset_from(view.common.cvar_indexes.as_ptr()) as cvarHandle_t;
        (*vmCvar).modificationCount = -1;
    }
    Cvar_Update(view.common, vmCvar);
}

/// Raven `Cvar_Update` — updates an interpreted module's version of a cvar.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:909-941`
pub fn Cvar_Update(common: &mut Common, vmCvar: *mut vmCvar_t) {
    assert!(!vmCvar.is_null());
    unsafe {
        if (*vmCvar).handle as c_uint >= common.cvar_numIndexes as c_uint {
            com_error(
                errorParm_t::ERR_DROP,
                "Cvar_Update: handle out of range".to_string(),
            );
        }

        let cv = common
            .cvar_indexes
            .as_mut_ptr()
            .add((*vmCvar).handle as usize);

        if (*cv).modificationCount == (*vmCvar).modificationCount {
            return;
        }
        if (*cv).string.is_null() {
            return; // variable might have been cleared by a cvar_restart
        }
        (*vmCvar).modificationCount = (*cv).modificationCount;
        if libc::strlen((*cv).string) + 1 > MAX_CVAR_VALUE_STRING {
            com_error(
                errorParm_t::ERR_DROP,
                format!(
                    "Cvar_Update: src {} length {} exceeds MAX_CVAR_VALUE_STRING",
                    CStr::from_ptr((*cv).string).to_string_lossy(),
                    libc::strlen((*cv).string)
                ),
            );
        }
        Q_strncpyz(
            (*vmCvar).string.as_mut_ptr(),
            (*cv).string,
            MAX_CVAR_VALUE_STRING as c_int,
        );

        (*vmCvar).value = (*cv).value;
        (*vmCvar).integer = (*cv).integer;
    }
}

/// Raven `Cvar_Init` — reads in all archived cvars.
///
/// Divergence: the registered handlers keep their pinned-receiver signatures
/// (the codebase pattern for state-threaded console commands); they are stored
/// as raw pointers pending the dispatch-table reconciliation (ruling 5).
/// Source: `oracle/codemp/qcommon/cvar.cpp:951-962`
pub fn Cvar_Init(view: &mut EngineHostView) {
    let cheats = Cvar_Get(
        view,
        c"sv_cheats".as_ptr(),
        c"0".as_ptr(),
        CVAR_ROM | CVAR_SYSTEMINFO,
    );
    view.common.cvar_cheats = cheats;

    Cmd_AddCommand(view, c"toggle".as_ptr(), Some(|view| Cvar_Toggle_f(view)));
    Cmd_AddCommand(view, c"set".as_ptr(), Some(|view| Cvar_Set_f(view)));
    Cmd_AddCommand(view, c"sets".as_ptr(), Some(|view| Cvar_SetS_f(view)));
    Cmd_AddCommand(view, c"setu".as_ptr(), Some(|view| Cvar_SetU_f(view)));
    Cmd_AddCommand(view, c"seta".as_ptr(), Some(|view| Cvar_SetA_f(view)));
    Cmd_AddCommand(view, c"reset".as_ptr(), Some(|view| Cvar_Reset_f(view)));
    Cmd_AddCommand(
        view,
        c"cvarlist".as_ptr(),
        Some(|view| Cvar_List_f(view.common)),
    );
    Cmd_AddCommand(
        view,
        c"cvar_restart".as_ptr(),
        Some(|view| Cvar_Restart_f(view)),
    );
}

/// Raven `Cvar_Realloc` — copies one cvar string into the defrag pool.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:965-975`
pub fn Cvar_Realloc(
    common: &mut Common,
    string: *mut *mut c_char,
    memPool: *mut c_char,
    memPoolUsed: &mut c_int,
) {
    unsafe {
        if !string.is_null() && !(*string).is_null() {
            let temp = memPool.add(*memPoolUsed as usize);
            libc::strcpy(temp, *string);
            *memPoolUsed += libc::strlen(*string) as c_int + 1;
            Cvar_FreeString(common, *string);
            *string = temp;
        }
    }
}

/// Raven `Cvar_Defrag` — turns many small allocation blocks into one big one.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:979-1018`
pub fn Cvar_Defrag(view: &mut EngineHostView) {
    let mut totalMem: c_int = 0;
    unsafe {
        let mut var: *mut cvar_t = view.common.cvar_vars;
        while !var.is_null() {
            if !(*var).name.is_null() {
                totalMem += libc::strlen((*var).name) as c_int + 1;
            }
            if !(*var).string.is_null() {
                totalMem += libc::strlen((*var).string) as c_int + 1;
            }
            if !(*var).resetString.is_null() {
                totalMem += libc::strlen((*var).resetString) as c_int + 1;
            }
            if !(*var).latchedString.is_null() {
                totalMem += libc::strlen((*var).latchedString) as c_int + 1;
            }
            var = (*var).next;
        }

        let mem = Z_Malloc(view, totalMem, memtag_t::TAG_SMALL, qfalse, 4) as *mut c_char;
        let nextMemPoolSize = totalMem;
        totalMem = 0;

        let mut var: *mut cvar_t = view.common.cvar_vars;
        while !var.is_null() {
            Cvar_Realloc(view.common, &mut (*var).name, mem, &mut totalMem);
            Cvar_Realloc(view.common, &mut (*var).string, mem, &mut totalMem);
            Cvar_Realloc(view.common, &mut (*var).resetString, mem, &mut totalMem);
            Cvar_Realloc(view.common, &mut (*var).latchedString, mem, &mut totalMem);
            var = (*var).next;
        }

        if !view.common.cvar_lastMemPool.is_null() {
            Z_Free(view.common, view.common.cvar_lastMemPool as *mut ());
        }
        view.common.cvar_lastMemPool = mem;
        view.common.cvar_memPoolSize = nextMemPoolSize;
    }
}
