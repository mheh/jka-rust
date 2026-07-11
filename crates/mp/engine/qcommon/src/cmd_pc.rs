#![allow(non_snake_case, non_camel_case_types, dead_code)]
//! `cmd_pc.cpp` — the registered-command list (`cmd_functions`) and dispatch
//! (`Cmd_ExecuteString`/`Cmd_List_f`/`Cmd_CommandCompletion`).
//!
//! Source: `oracle/codemp/qcommon/cmd_pc.cpp`

use core::ffi::{c_char, c_int, CStr};

use mp_host_interface::engine_host::EngineHost;

use crate::cmd::cmd_function_t::{cmd_function_t, CmdFunction};
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::Com_Filter;
use crate::z_memman_pc::{CopyString, S_Malloc, Z_Free};

// `Server` is a pinned-receiver placeholder: the real type lives in
// mp_engine_server, which depends on this crate (importing it would cycle).
pub(crate) use crate::cm_load::{RenderModels, RmManager};
pub(crate) struct Server;

use crate::common::com_printf;
use crate::cvar_fns::Cvar_Command;
use mp_qshared::shared::q_string::Q_stricmp;

/// `Cmd_AddCommand`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:18-39`
pub fn Cmd_AddCommand(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    cmd_name: *const c_char,
    function: Option<CmdFunction>,
) {
    unsafe {
        // fail if the command already exists
        let mut cmd: *mut cmd_function_t = common.cmd_functions;
        while !cmd.is_null() {
            if libc::strcmp(cmd_name, (*cmd).name) == 0 {
                // allow completion-only commands to be silently doubled
                if function.is_some() {
                    let name = CStr::from_ptr(cmd_name).to_string_lossy();
                    com_printf(common, &format!("Cmd_AddCommand: {name} already defined\n"));
                }
                return;
            }
            cmd = (*cmd).next;
        }

        // use a small malloc to avoid zone fragmentation
        let cmd = S_Malloc(
            common,
            cm,
            rm,
            host,
            core::mem::size_of::<cmd_function_t>() as c_int,
        ) as *mut cmd_function_t;
        (*cmd).name = CopyString(common, cm, rm, host, cmd_name);
        (*cmd).function = function;
        (*cmd).next = common.cmd_functions;
        common.cmd_functions = cmd;
    }
}

/// `Cmd_RemoveCommand`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:46-67`
pub fn Cmd_RemoveCommand(common: &mut Common, cmd_name: *const c_char) {
    // Raw double-pointer walk (as `Cmd_ExecuteString`): `&mut …` is cast to a
    // raw `*mut *mut` immediately so no borrow of `common` outlives the
    // `Z_Free(common, …)` calls in the removal branch.
    let mut back: *mut *mut cmd_function_t = &mut common.cmd_functions as *mut _;
    loop {
        unsafe {
            let cmd = *back;
            if cmd.is_null() {
                // command wasn't active
                return;
            }
            if libc::strcmp(cmd_name, (*cmd).name) == 0 {
                *back = (*cmd).next;
                if !(*cmd).name.is_null() {
                    Z_Free(common, (*cmd).name as *mut ());
                }
                Z_Free(common, cmd as *mut ());
                return;
            }
            back = &mut (*cmd).next as *mut _;
        }
    }
}

/// `Cmd_CommandCompletion`.
///
/// Raven walks the registered-command list, invoking `callback` with each
/// command's name.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:75-81`
pub fn Cmd_CommandCompletion(common: &mut Common, callback: extern "C" fn(*const c_char)) {
    // PORT-NOTE(cmd_functions): `common.cmd_functions` — the `cmd_function_t*`
    // list head — is not yet a `Common` field (missing symbol, see above).
    let mut cmd: *mut cmd_function_t = common.cmd_functions;
    while !cmd.is_null() {
        unsafe {
            callback((*cmd).name);
            cmd = (*cmd).next;
        }
    }
}

/// `Cmd_List_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:153-173`
pub fn Cmd_List_f(common: &mut Common) {
    let match_: *mut c_char = if crate::cmd_common::Cmd_Argc(common) > 1 {
        crate::cmd_common::Cmd_Argv(common, 1)
    } else {
        core::ptr::null_mut()
    };

    let mut i: c_int = 0;
    // PORT-NOTE(cmd_functions): see `Cmd_CommandCompletion` above.
    let mut cmd: *mut cmd_function_t = common.cmd_functions;
    while !cmd.is_null() {
        unsafe {
            if !match_.is_null() && Com_Filter(match_, (*cmd).name as *mut c_char, 0) == 0 {
                cmd = (*cmd).next;
                continue;
            }

            // PORT-NOTE(Com_Printf): the qcommon-side `Com_Printf` (routes
            // through the engine print sink / console) has no landed symbol
            // in this crate yet (escalated as missing, resolution packet
            // `qcommon__1592_CM_DeleteCachedMap.md`); narrowed to a single
            // `*const c_char` (no safe C-variadic fn defs) — pre-format the
            // name here, matching the `cmd_common.rs` `Cmd_Echo_f` precedent.
            let name = core::ffi::CStr::from_ptr((*cmd).name).to_string_lossy();
            com_printf(common, &format!("{}\n", name));
            i += 1;
            cmd = (*cmd).next;
        }
    }
    unsafe {
        com_printf(common, &format!("{} commands\n", i));
    }
}

/// `Cmd_ExecuteString`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:91-145`
pub fn Cmd_ExecuteString(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    text: *const c_char,
) {
    // execute the command line
    crate::cmd_common::Cmd_TokenizeString(common, text);
    if crate::cmd_common::Cmd_Argc(common) == 0 {
        return; // no tokens
    }

    // check registered command functions
    // PORT-NOTE(cmd_functions): see `Cmd_CommandCompletion` above; the
    // `prev`/`cmd` double-pointer walk is transcribed with raw pointers to
    // match Raven's link-rearrangement exactly.
    unsafe {
        let mut prev: *mut *mut cmd_function_t = &mut common.cmd_functions as *mut _;
        while !(*prev).is_null() {
            let cmd = *prev;
            if Q_stricmp(crate::cmd_common::Cmd_Argv(common, 0), (*cmd).name) == 0 {
                // rearrange the links so that the command will be
                // near the head of the list next time it is used
                *prev = (*cmd).next;
                (*cmd).next = common.cmd_functions;
                common.cmd_functions = cmd;

                // perform the action
                if let Some(function) = (*cmd).function {
                    function(common, cm, sv, rm, rmg, host);
                } else {
                    // let the cgame or game handle it
                    break;
                }
                return;
            }
            prev = &mut (*cmd).next as *mut _;
        }
    }

    // check cvars
    if unsafe { Cvar_Command(common, cm, rm, host) } != 0 {
        return;
    }

    // check client game commands
    // PORT-NOTE(com_cl_running/com_sv_running): `Common`'s `cvar_t*` handle
    // fields aren't landed yet (the cvar sub-struct TODO in `common/common.rs`);
    // referenced verbatim as missing symbols.
    let cl_game_command = common.hooks.CL_GameCommand.expect("CL_GameCommand hook");
    if !common.com_cl_running.is_null()
        && unsafe { (*common.com_cl_running).integer != 0 && cl_game_command() != 0 }
    {
        return;
    }

    // check server game commands
    let sv_game_command = common
        .hooks
        .SV_GameCommand
        .expect("SV_GameCommand hook — installed by mp_engine_server at boot");
    if !common.com_sv_running.is_null()
        && unsafe { (*common.com_sv_running).integer != 0 && sv_game_command(common, sv) != 0 }
    {
        return;
    }

    // check ui commands
    let ui_game_command = common.hooks.UI_GameCommand.expect("UI_GameCommand hook");
    if !common.com_cl_running.is_null()
        && unsafe { (*common.com_cl_running).integer != 0 && ui_game_command() != 0 }
    {
        return;
    }

    // send it as a server command if we are connected
    // this will usually result in a chat message
    //CL_ForwardCommandToServer ( text );
    (common
        .hooks
        .CL_ForwardCommandToServer
        .expect("CL_ForwardCommandToServer hook"))(text);
}
