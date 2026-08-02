//! `cvar.cpp` — dynamic variable tracking.
//!
//! Destination `_fns` escape: the `cvar/` directory already holds the cvar
//! types/consts, so `cvar.cpp`'s functions land here (per packet DESTINATION).
//!
//! String-data migration (DEC-32): the registry owns `String` fields in the
//! `Common.cvar_indexes` slot arena (slot index = Raven's `cvarHandle_t`,
//! oracle-identical numbering) with `Common.cvar_vars` as the enumeration
//! order (index 0 = Raven's head-inserted list head). Dropped as pure
//! allocator/lookup internals with no observable behavior: the name hash
//! table (`generateHashValue`/`hashTable`, `cvar.cpp:41-55`) — `Cvar_FindVar`
//! scans the order list — and the `Cvar_FreeString`/`Cvar_Realloc`/
//! `Cvar_Defrag` string defrag pool (`cvar.cpp:26-32,965-1018`, whose only
//! oracle caller is the client, `cl_main.cpp:715`, outside the dedicated
//! island).
//!
//! Source: `oracle/codemp/qcommon/cvar.cpp`

use core::ffi::{c_char, c_float, c_int, c_uint};

use native_types::fileHandle_t;

use mp_qshared::shared::cvar::{
    cvarHandle_t, cvar_t, vmCvar_t, CvarHandle, CVAR_ARCHIVE, CVAR_CHEAT, CVAR_INIT, CVAR_INTERNAL,
    CVAR_LATCH, CVAR_NORESTART, CVAR_ROM, CVAR_SERVERINFO, CVAR_SYSTEMINFO, CVAR_USERINFO,
    CVAR_USER_CREATED, MAX_CVAR_VALUE_STRING,
};
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::MAX_STRING_TOKENS;
use native_string::{Info_SetValueForKey, Info_SetValueForKey_Big};

use native_string::atof::atof;
use native_string::atoi::atoi;
use native_string::filter::Com_Filter;
use native_string::q_strncpyz::Q_strncpyz;

use crate::cmd::Cmd_AddCommand;
use crate::cmd_common::{Cmd_Argc, Cmd_Argv};
use crate::common::engine_host_view::EngineHostView;
use crate::common::error::com_error;
use crate::common::{com_printf, info_set_report, Common};
use crate::common_fns::Com_DPrintf;
use crate::cvar::cvar_consts::MAX_CVARS;
use crate::files_common::FS_Printf;

/// Raven `Cvar_ValidateString`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:62-76`
pub fn Cvar_ValidateString(s: &str) -> bool {
    !s.contains(['\\', '"', ';'])
}

/// Raven `Cvar_FindVar`.
///
/// The name hash table is dropped (module note); the scan covers the same
/// linked set with Raven's `Q_stricmp` ASCII-case fold.
/// Source: `oracle/codemp/qcommon/cvar.cpp:83-96`
pub fn Cvar_FindVar(common: &Common, var_name: &str) -> Option<CvarHandle> {
    common.cvar_vars.iter().copied().find(|&h| {
        common
            .cvar(h)
            .name
            .as_bytes()
            .eq_ignore_ascii_case(var_name.as_bytes())
    })
}

/// Raven `Cvar_VariableValue`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:103-110`
pub fn Cvar_VariableValue(common: &Common, var_name: &str) -> f32 {
    match Cvar_FindVar(common, var_name) {
        Some(h) => common.cvar(h).value,
        None => 0.0,
    }
}

/// Raven `Cvar_VariableIntegerValue`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:118-125`
pub fn Cvar_VariableIntegerValue(common: &Common, var_name: &str) -> c_int {
    match Cvar_FindVar(common, var_name) {
        Some(h) => common.cvar(h).integer,
        None => 0,
    }
}

/// Raven `Cvar_VariableString`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:133-140`
pub fn Cvar_VariableString<'a>(common: &'a Common, var_name: &str) -> &'a str {
    match Cvar_FindVar(common, var_name) {
        Some(h) => &common.cvar(h).string,
        None => "",
    }
}

/// Raven `Cvar_VariableStringBuffer` — fills a caller-owned C buffer (the
/// module-memory seam shape; in-engine readers use [`Cvar_VariableString`]).
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:148-158`
pub fn Cvar_VariableStringBuffer(
    common: &Common,
    var_name: &str,
    buffer: *mut c_char,
    bufsize: c_int,
) {
    unsafe {
        match Cvar_FindVar(common, var_name) {
            None => *buffer = 0,
            Some(h) => Q_strncpyz(
                core::slice::from_raw_parts_mut(buffer, bufsize as usize),
                &common.cvar(h).string,
                bufsize as usize,
            ),
        }
    }
}

/// Raven `Cvar_CommandCompletion`.
///
/// Raven invokes a callback per name. The port returns the names in that same
/// order instead, so the caller keeps its own receivers (porting-rules §B4, §C7).
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:166-177`
pub fn Cvar_CommandCompletion(common: &Common) -> Vec<String> {
    let mut names = Vec::new();
    for &h in &common.cvar_vars {
        let cvar = common.cvar(h);
        // Dont show internal cvars
        if cvar.flags & CVAR_INTERNAL != 0 {
            continue;
        }
        names.push(cvar.name.clone());
    }
    names
}

/// Raven `Cvar_Get`.
///
/// Raven: if the variable already exists, the value will not be set unless
/// CVAR_ROM; the flags will be or'ed in if the variable exists.
/// Source: `oracle/codemp/qcommon/cvar.cpp:188-280`
pub fn Cvar_Get(
    view: &mut EngineHostView,
    var_name: &str,
    var_value: &str,
    flags: c_int,
) -> CvarHandle {
    let mut var_name = var_name;

    if !Cvar_ValidateString(var_name) {
        com_printf(
            view.common,
            &format!("invalid cvar name string: {var_name}\n"),
        );
        var_name = "BADNAME";
    }

    // `#if 0` backslash-value validation is compiled out in this build.

    if let Some(h) = Cvar_FindVar(view.common, var_name) {
        // if the C code is now specifying a variable that the user already
        // set a value for, take the new value as the reset value
        if (view.common.cvar(h).flags & CVAR_USER_CREATED) != 0
            && (flags & CVAR_USER_CREATED) == 0
            && !var_value.is_empty()
        {
            let var = view.common.cvar_mut(h);
            var.flags &= !CVAR_USER_CREATED;
            var.resetString = var_value.to_string();

            // ZOID -- needs to be set so that cvars the game sets as
            // SERVERINFO get sent to clients
            view.common.cvar_modifiedFlags |= flags;
        }

        view.common.cvar_mut(h).flags |= flags;
        // only allow one non-empty reset string without a warning
        if view.common.cvar(h).resetString.is_empty() {
            // we don't have a reset string yet
            view.common.cvar_mut(h).resetString = var_value.to_string();
        } else if !var_value.is_empty() && view.common.cvar(h).resetString != var_value {
            let msg = format!(
                "Warning: cvar \"{}\" given initial values: \"{}\" and \"{}\"\n",
                var_name,
                view.common.cvar(h).resetString,
                var_value
            );
            Com_DPrintf(view.common, &msg);
        }
        // if we have a latched string, take that value now (Raven nulls
        // `latchedString` before the set so Cvar_Set2 won't free it — the
        // `take()` is that null-then-free dance).
        if let Some(s) = view.common.cvar_mut(h).latchedString.take() {
            Cvar_Set2(view, var_name, Some(&s), true);
        }

        // `#if 0` CVAR_ROM-override block is compiled out in this build.
        return h;
    }

    //
    // allocate a new cvar
    //
    if view.common.cvar_indexes.len() >= MAX_CVARS {
        com_error(errorParm_t::ERR_FATAL, "MAX_CVARS".to_string());
    }
    let h = CvarHandle::from_slot(view.common.cvar_indexes.len());
    view.common.cvar_indexes.push(cvar_t {
        name: var_name.to_string(),
        string: var_value.to_string(),
        resetString: var_value.to_string(),
        latchedString: None,
        flags,
        modified: true,
        modificationCount: 1,
        value: atof(var_value) as c_float,
        integer: atoi(var_value),
    });
    // link the variable in (Raven's head insert)
    view.common.cvar_vars.insert(0, h);
    h
}

/// Raven `Cvar_Set2`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:287-395`
pub fn Cvar_Set2(
    view: &mut EngineHostView,
    var_name: &str,
    value: Option<&str>,
    force: bool,
) -> Option<CvarHandle> {
    let mut var_name = var_name;

    if !Cvar_ValidateString(var_name) {
        com_printf(
            view.common,
            &format!("invalid cvar name string: {var_name}\n"),
        );
        var_name = "BADNAME";
    }

    // `#if 0` value validation is compiled out in this build.

    let Some(h) = Cvar_FindVar(view.common, var_name) else {
        let value = value?;
        // create it
        return Some(if !force {
            Cvar_Get(view, var_name, value, CVAR_USER_CREATED)
        } else {
            Cvar_Get(view, var_name, value, 0)
        });
    };

    // Dont display the update when its internal
    if (view.common.cvar(h).flags & CVAR_INTERNAL) == 0 {
        // Raven's `%s` of the null reset-path value renders "(null)" on the
        // oracle's libc.
        let msg = format!("Cvar_Set2: {} {}\n", var_name, value.unwrap_or("(null)"));
        Com_DPrintf(view.common, &msg);
    }

    let reset_value;
    let value = match value {
        Some(v) => v,
        None => {
            reset_value = view.common.cvar(h).resetString.clone();
            &reset_value
        }
    };

    if view.common.cvar(h).string == value {
        return Some(h);
    }
    // note what types of cvars have been modified (userinfo, archive,
    // serverinfo, systeminfo)
    let flags = view.common.cvar(h).flags;
    view.common.cvar_modifiedFlags |= flags;

    if !force {
        if (flags & CVAR_ROM) != 0 {
            com_printf(view.common, &format!("{var_name} is read only.\n"));
            return Some(h);
        }

        if (flags & CVAR_INIT) != 0 {
            com_printf(view.common, &format!("{var_name} is write protected.\n"));
            return Some(h);
        }

        if (flags & CVAR_LATCH) != 0 {
            if let Some(latched) = &view.common.cvar(h).latchedString {
                if value == latched.as_str() {
                    return Some(h);
                }
                // (the old latched string frees via the overwrite below)
            } else if view.common.cvar(h).string == value {
                return Some(h);
            }

            com_printf(
                view.common,
                &format!("{var_name} will be changed upon restarting.\n"),
            );
            let var = view.common.cvar_mut(h);
            var.latchedString = Some(value.to_string());
            var.modified = true;
            var.modificationCount += 1;
            return Some(h);
        }

        if (flags & CVAR_CHEAT) != 0 && view.common.cvar(view.common.cvar_cheats).integer == 0 {
            com_printf(view.common, &format!("{var_name} is cheat protected.\n"));
            return Some(h);
        }
    } else if view.common.cvar(h).latchedString.is_some() {
        view.common.cvar_mut(h).latchedString = None;
    }

    if view.common.cvar(h).string == value {
        return Some(h); // not changed
    }

    let var = view.common.cvar_mut(h);
    var.modified = true;
    var.modificationCount += 1;

    var.string = value.to_string(); // (frees the old value string)
    var.value = atof(value) as c_float;
    var.integer = atoi(value);

    Some(h)
}

/// Raven `Cvar_Set`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:402-404`
pub fn Cvar_Set(view: &mut EngineHostView, var_name: &str, value: &str) {
    Cvar_Set2(view, var_name, Some(value), true);
}

/// Raven `Cvar_SetLatched`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:411-413`
pub fn Cvar_SetLatched(view: &mut EngineHostView, var_name: &str, value: &str) {
    Cvar_Set2(view, var_name, Some(value), false);
}

/// Raven `Cvar_SetValue`.
///
/// The Raven `Com_sprintf` renders are ported via the `format!` pre-render
/// idiom (`Com_sprintf` has no landed qcommon home).
/// Source: `oracle/codemp/qcommon/cvar.cpp:420-429`
pub fn Cvar_SetValue(view: &mut EngineHostView, var_name: &str, value: f32) {
    let val = if value == (value as c_int) as f32 {
        format!("{}", value as c_int)
    } else {
        format!("{value:.6}")
    };
    Cvar_Set(view, var_name, &val);
}

/// Raven `Cvar_Reset`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:437-439`
pub fn Cvar_Reset(view: &mut EngineHostView, var_name: &str) {
    Cvar_Set2(view, var_name, None, false);
}

/// Raven `Cvar_SetCheatState`.
///
/// Raven: any testing variables will be reset to the safe values.
/// Source: `oracle/codemp/qcommon/cvar.cpp:449-467`
pub fn Cvar_SetCheatState(view: &mut EngineHostView) {
    // set all default vars to the safe value
    let mut i = 0;
    while i < view.common.cvar_vars.len() {
        let h = view.common.cvar_vars[i];
        if (view.common.cvar(h).flags & CVAR_CHEAT) != 0 {
            // the CVAR_LATCHED|CVAR_CHEAT vars might escape the reset here
            // because of a different var->latchedString
            if view.common.cvar(h).latchedString.is_some() {
                view.common.cvar_mut(h).latchedString = None;
            }
            if view.common.cvar(h).resetString != view.common.cvar(h).string {
                let name = view.common.cvar(h).name.clone();
                let reset = view.common.cvar(h).resetString.clone();
                Cvar_Set(view, &name, &reset);
            }
        }
        i += 1;
    }
}

/// Raven `Cvar_Command` — handles variable inspection and changing from the
/// console.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:476-515`
pub fn Cvar_Command(view: &mut EngineHostView) -> bool {
    // check variables
    let arg0 = Cmd_Argv(view.common, 0).to_owned();
    let Some(h) = Cvar_FindVar(view.common, &arg0) else {
        return false;
    };

    // perform a variable print or set
    if Cmd_Argc(view.common) == 1 {
        // `S_COLOR_WHITE` = "^7" (oracle/codemp/game/q_shared.h:1167).
        let var = view.common.cvar(h);
        let msg = format!(
            "\"{}\" is:\"{}^7\" default:\"{}^7\"\n",
            var.name, var.string, var.resetString
        );
        let latched = var.latchedString.clone();
        com_printf(view.common, &msg);
        if let Some(latched) = latched {
            com_printf(view.common, &format!("latched: \"{latched}\"\n"));
        }
        return true;
    }

    // JFM toggle test
    let value = Cmd_Argv(view.common, 1).to_owned();
    let name = view.common.cvar(h).name.clone();
    if value.as_bytes().first() == Some(&b'!') {
        // toggle
        let nv = if view.common.cvar(h).value == 0.0 {
            1
        } else {
            0
        };
        Cvar_Set2(view, &name, Some(&format!("{nv}")), false); // toggle the value
    } else {
        Cvar_Set2(view, &name, Some(&value), false); // set the value if forcing isn't required
    }

    true
}

/// Raven `Cvar_Toggle_f` — toggles a cvar for easy single key binding.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:525-537`
pub fn Cvar_Toggle_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "usage: toggle <variable>\n");
        return;
    }

    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    let mut v = Cvar_VariableValue(view.common, &arg1) as c_int;
    v = if v == 0 { 1 } else { 0 };

    Cvar_Set2(view, &arg1, Some(&format!("{v}")), false);
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

    let mut combined = String::new();
    let mut l: c_int = 0;
    for i in 2..c {
        let arg = Cmd_Argv(view.common, i).to_owned();
        // Raven's length bookkeeping is `strlen(arg+1)` — one short per arg
        // (and past-NUL UB on an empty arg; saturating 0 is the defined pick).
        let len = arg.len().saturating_sub(1) as c_int;
        if l + len >= MAX_STRING_TOKENS as c_int - 2 {
            break;
        }
        combined.push_str(&arg);
        if i != c - 1 {
            combined.push(' ');
        }
        l += len;
    }
    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    Cvar_Set2(view, &arg1, Some(&combined), false);
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
    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    let Some(h) = Cvar_FindVar(view.common, &arg1) else {
        return;
    };
    view.common.cvar_mut(h).flags |= CVAR_USERINFO;
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
    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    let Some(h) = Cvar_FindVar(view.common, &arg1) else {
        return;
    };
    view.common.cvar_mut(h).flags |= CVAR_SERVERINFO;
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
    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    let Some(h) = Cvar_FindVar(view.common, &arg1) else {
        return;
    };
    view.common.cvar_mut(h).flags |= CVAR_ARCHIVE;
}

/// Raven `Cvar_Reset_f`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:644-650`
pub fn Cvar_Reset_f(view: &mut EngineHostView) {
    if Cmd_Argc(view.common) != 2 {
        com_printf(view.common, "usage: reset <variable>\n");
        return;
    }
    let arg1 = Cmd_Argv(view.common, 1).to_owned();
    Cvar_Reset(view, &arg1);
}

/// Raven `Cvar_WriteVariables` — appends lines containing "set variable value"
/// for all variables with the archive flag set to qtrue.
///
/// The Raven `Com_sprintf`/`FS_Printf` pair is ported via the `format!`
/// pre-render idiom; the `USE_CD_KEY` `cl_cdkey` skip is compiled out.
/// Source: `oracle/codemp/qcommon/cvar.cpp:660-680`
pub fn Cvar_WriteVariables(common: &mut Common, f: fileHandle_t) {
    let mut i = 0;
    while i < common.cvar_vars.len() {
        let h = common.cvar_vars[i];
        let line = {
            let var = common.cvar(h);
            if (var.flags & CVAR_ARCHIVE) != 0 {
                // write the latched value, even if it hasn't taken effect yet
                Some(if let Some(latched) = &var.latchedString {
                    format!("seta {} \"{}\"\n", var.name, latched)
                } else {
                    format!("seta {} \"{}\"\n", var.name, var.string)
                })
            } else {
                None
            }
        };
        if let Some(buffer) = line {
            FS_Printf(common, f, &buffer);
        }
        i += 1;
    }
}

/// Raven `Cvar_List_f`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:687-750`
pub fn Cvar_List_f(common: &mut Common) {
    let r#match: Option<String> = if Cmd_Argc(common) > 1 {
        Some(Cmd_Argv(common, 1).to_owned())
    } else {
        None
    };

    let mut i: c_int = 0;
    let mut idx = 0;
    while idx < common.cvar_vars.len() {
        let h = common.cvar_vars[idx];
        idx += 1;
        // `i` counts every cvar (Raven increments in the loop step, so
        // `continue`d internal/unmatched cvars are still counted).
        i += 1;

        let line = {
            let cur = common.cvar(h);
            // Dont show internal cvars
            if (cur.flags & CVAR_INTERNAL) != 0 {
                continue;
            }

            if let Some(m) = &r#match {
                if !Com_Filter(m, &cur.name, false) {
                    continue;
                }
            }

            let mut line = String::new();
            line.push(if (cur.flags & CVAR_SERVERINFO) != 0 {
                'S'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_USERINFO) != 0 {
                'U'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_ROM) != 0 {
                'R'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_INIT) != 0 {
                'I'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_ARCHIVE) != 0 {
                'A'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_LATCH) != 0 {
                'L'
            } else {
                ' '
            });
            line.push(if (cur.flags & CVAR_CHEAT) != 0 {
                'C'
            } else {
                ' '
            });
            line.push_str(&format!(" {} \"{}\"\n", cur.name, cur.string));
            line
        };
        com_printf(common, &line);
    }

    com_printf(common, &format!("\n{i} total cvars\n"));
}

/// Raven `Cvar_Restart_f` — resets all cvars to their hardcoded values.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:759-802`
pub fn Cvar_Restart_f(view: &mut EngineHostView) {
    let mut i = 0;
    while i < view.common.cvar_vars.len() {
        let h = view.common.cvar_vars[i];
        let flags = view.common.cvar(h).flags;

        // don't mess with rom values, or some inter-module communication
        // will get broken (com_cl_running, etc)
        if (flags & (CVAR_ROM | CVAR_INIT | CVAR_NORESTART)) != 0 {
            i += 1;
            continue;
        }

        // throw out any variables the user created
        if (flags & CVAR_USER_CREATED) != 0 {
            view.common.cvar_vars.remove(i);
            // §19: Raven frees all four strings but its `memset(var, 0,
            // sizeof(var))` clears only the pointer-width `name`, leaving
            // dangling pointers in the slot (UB to read); the owned slot
            // clears all four. The slot itself stays (indexes never reuse).
            let var = view.common.cvar_mut(h);
            var.name = String::new();
            var.string = String::new();
            var.resetString = String::new();
            var.latchedString = None;
            continue;
        }

        let name = view.common.cvar(h).name.clone();
        let reset = view.common.cvar(h).resetString.clone();
        Cvar_Set(view, &name, &reset);

        i += 1;
    }
}

/// Raven `Cvar_InfoString`.
///
/// Raven's `static char info[MAX_INFO_STRING]` return buffer becomes a
/// returned owned `String` (string-data migration).
/// Source: `oracle/codemp/qcommon/cvar.cpp:811-845`
pub fn Cvar_InfoString(common: &Common, bit: c_int) -> String {
    let mut info = String::new();

    for &h in &common.cvar_vars {
        let var = common.cvar(h);
        if (var.flags & CVAR_INTERNAL) == 0 && (var.flags & bit) != 0 {
            info_set_report(
                Info_SetValueForKey(&mut info, &var.name, &var.string),
                "Info string length exceeded\n",
            );
        }
    }
    // The `kungFuSafety` g_debugMelee block is commented out in the oracle.
    info
}

/// Raven `Cvar_InfoString_Big` — handles large info strings (`CS_SYSTEMINFO`).
///
/// Raven's `static char info[BIG_INFO_STRING]` return buffer becomes a
/// returned owned `String` (string-data migration).
/// Source: `oracle/codemp/qcommon/cvar.cpp:854-869`
pub fn Cvar_InfoString_Big(common: &Common, bit: c_int) -> String {
    let mut info = String::new();

    for &h in &common.cvar_vars {
        let var = common.cvar(h);
        if (var.flags & CVAR_INTERNAL) == 0 && (var.flags & bit) != 0 {
            info_set_report(
                Info_SetValueForKey_Big(&mut info, &var.name, &var.string),
                "BIG Info string length exceeded\n",
            );
        }
    }
    info
}

/// Raven `Cvar_InfoStringBuffer`.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:878-880`
pub fn Cvar_InfoStringBuffer(common: &Common, bit: c_int, buff: *mut c_char, buffsize: c_int) {
    let info = Cvar_InfoString(common, bit);
    unsafe {
        Q_strncpyz(
            core::slice::from_raw_parts_mut(buff, buffsize as usize),
            &info,
            buffsize as usize,
        )
    };
}

/// Raven `Cvar_Register` — basically a slightly modified `Cvar_Get` for the
/// interpreted modules.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:889-899`
pub fn Cvar_Register(
    view: &mut EngineHostView,
    vmCvar: *mut vmCvar_t,
    varName: &str,
    defaultValue: &str,
    flags: c_int,
) {
    let h = Cvar_Get(view, varName, defaultValue, flags);
    if vmCvar.is_null() {
        return;
    }
    unsafe {
        (*vmCvar).handle = h.slot() as cvarHandle_t;
        (*vmCvar).modificationCount = -1;
    }
    Cvar_Update(view.common, vmCvar);
}

/// Raven `Cvar_Update` — updates an interpreted module's version of a cvar.
///
/// Source: `oracle/codemp/qcommon/cvar.cpp:909-941`
pub fn Cvar_Update(common: &Common, vmCvar: *mut vmCvar_t) {
    assert!(!vmCvar.is_null());
    unsafe {
        if (*vmCvar).handle as c_uint >= common.cvar_indexes.len() as c_uint {
            com_error(
                errorParm_t::ERR_DROP,
                "Cvar_Update: handle out of range".to_string(),
            );
        }

        let cv = &common.cvar_indexes[(*vmCvar).handle as usize];

        if cv.modificationCount == (*vmCvar).modificationCount {
            return;
        }
        if cv.name.is_empty() {
            // variable might have been cleared by a cvar_restart
            // (§19: Raven checks `!cv->string`, dead under its pointer-width
            // memset; the cleared-slot skip is the defined equivalent.)
            return;
        }
        (*vmCvar).modificationCount = cv.modificationCount;
        if cv.string.len() + 1 > MAX_CVAR_VALUE_STRING {
            com_error(
                errorParm_t::ERR_DROP,
                format!(
                    "Cvar_Update: src {} length {} exceeds MAX_CVAR_VALUE_STRING",
                    cv.string,
                    cv.string.len()
                ),
            );
        }
        Q_strncpyz(&mut (*vmCvar).string, &cv.string, MAX_CVAR_VALUE_STRING);

        (*vmCvar).value = cv.value;
        (*vmCvar).integer = cv.integer;
    }
}

/// Raven `Cvar_Init` — reads in all archived cvars.
///
/// Divergence: the registered handlers keep their pinned-receiver signatures
/// (the codebase pattern for state-threaded console commands); they are stored
/// as raw pointers pending the dispatch-table reconciliation (ruling 5).
/// Source: `oracle/codemp/qcommon/cvar.cpp:951-962`
pub fn Cvar_Init(view: &mut EngineHostView) {
    let cheats = Cvar_Get(view, "sv_cheats", "0", CVAR_ROM | CVAR_SYSTEMINFO);
    view.common.cvar_cheats = Some(cheats);

    Cmd_AddCommand(view, "toggle", Some(|view| Cvar_Toggle_f(view)));
    Cmd_AddCommand(view, "set", Some(|view| Cvar_Set_f(view)));
    Cmd_AddCommand(view, "sets", Some(|view| Cvar_SetS_f(view)));
    Cmd_AddCommand(view, "setu", Some(|view| Cvar_SetU_f(view)));
    Cmd_AddCommand(view, "seta", Some(|view| Cvar_SetA_f(view)));
    Cmd_AddCommand(view, "reset", Some(|view| Cvar_Reset_f(view)));
    Cmd_AddCommand(view, "cvarlist", Some(|view| Cvar_List_f(view.common)));
    Cmd_AddCommand(view, "cvar_restart", Some(|view| Cvar_Restart_f(view)));
}
