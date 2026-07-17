#![allow(non_snake_case, non_camel_case_types, dead_code)]
//! `cmd_pc.cpp` — the registered-command list (`cmd_functions`) and dispatch
//! (`Cmd_ExecuteString`/`Cmd_List_f`/`Cmd_CommandCompletion`).
//!
//! Source: `oracle/codemp/qcommon/cmd_pc.cpp`

use core::ffi::{c_char, c_int, CStr};

use crate::cmd::cmd_function_t::{cmd_function_t, CmdFunction};
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;
use crate::common_fns::Com_Filter;
use crate::z_memman_pc::{CopyString, S_Malloc, Z_Free};

// `Server` is a type-erased receiver slot: the real type lives in
// mp_engine_server, which depends on this crate (importing it would cycle).
// Re-exported at this historical home; defined once in `common::opaque_slots`.
pub use crate::common::opaque_slots::Server;

use crate::common::com_printf;
use crate::cvar_fns::Cvar_Command;
use mp_qshared::shared::q_string::Q_stricmp;

/// `Cmd_AddCommand`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:18-39`
pub fn Cmd_AddCommand(
    view: &mut EngineHostView,
    cmd_name: *const c_char,
    function: Option<CmdFunction>,
) {
    unsafe {
        // fail if the command already exists
        let mut cmd: *mut cmd_function_t = view.common.cmd_functions;
        while !cmd.is_null() {
            if libc::strcmp(cmd_name, (*cmd).name) == 0 {
                // allow completion-only commands to be silently doubled
                if function.is_some() {
                    let name = CStr::from_ptr(cmd_name).to_string_lossy();
                    com_printf(
                        view.common,
                        &format!("Cmd_AddCommand: {name} already defined\n"),
                    );
                }
                return;
            }
            cmd = (*cmd).next;
        }

        // use a small malloc to avoid zone fragmentation
        let cmd =
            S_Malloc(view, core::mem::size_of::<cmd_function_t>() as c_int) as *mut cmd_function_t;
        (*cmd).name = CopyString(view, cmd_name);
        (*cmd).function = function;
        (*cmd).next = view.common.cmd_functions;
        view.common.cmd_functions = cmd;
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
    let mut cmd: *mut cmd_function_t = common.cmd_functions;
    while !cmd.is_null() {
        unsafe {
            if !match_.is_null() && Com_Filter(match_, (*cmd).name as *mut c_char, 0) == 0 {
                cmd = (*cmd).next;
                continue;
            }

            // No safe C-variadic fn defs — pre-format the name here, matching
            // the `cmd_common.rs` `Cmd_Echo_f` precedent.
            let name = core::ffi::CStr::from_ptr((*cmd).name).to_string_lossy();
            com_printf(common, &format!("{}\n", name));
            i += 1;
            cmd = (*cmd).next;
        }
    }
    com_printf(common, &format!("{} commands\n", i));
}

/// `Cmd_ExecuteString`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:91-145`
pub fn Cmd_ExecuteString(view: &mut EngineHostView, text: *const c_char) {
    // execute the command line
    crate::cmd_common::Cmd_TokenizeString(view.common, text);
    if crate::cmd_common::Cmd_Argc(view.common) == 0 {
        return; // no tokens
    }

    // check registered command functions
    // The `prev`/`cmd` double-pointer walk is transcribed with raw pointers
    // to match Raven's link-rearrangement exactly.
    unsafe {
        let mut prev: *mut *mut cmd_function_t = &mut view.common.cmd_functions as *mut _;
        while !(*prev).is_null() {
            let cmd = *prev;
            if Q_stricmp(crate::cmd_common::Cmd_Argv(view.common, 0), (*cmd).name) == 0 {
                // rearrange the links so that the command will be
                // near the head of the list next time it is used
                *prev = (*cmd).next;
                (*cmd).next = view.common.cmd_functions;
                view.common.cmd_functions = cmd;

                // perform the action
                if let Some(function) = (*cmd).function {
                    function(view);
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
    if Cvar_Command(view) != 0 {
        return;
    }

    // check client game commands
    let cl_game_command = view
        .common
        .hooks
        .CL_GameCommand
        .expect("CL_GameCommand hook");
    if !view.common.com_cl_running.is_null()
        && unsafe { (*view.common.com_cl_running).integer != 0 && cl_game_command(view) != 0 }
    {
        return;
    }

    // check server game commands
    let sv_game_command = view
        .common
        .hooks
        .SV_GameCommand
        .expect("SV_GameCommand hook — installed by mp_engine_server at boot");
    if !view.common.com_sv_running.is_null()
        && unsafe { (*view.common.com_sv_running).integer != 0 && sv_game_command(view) != 0 }
    {
        return;
    }

    // check ui commands
    let ui_game_command = view
        .common
        .hooks
        .UI_GameCommand
        .expect("UI_GameCommand hook");
    if !view.common.com_cl_running.is_null()
        && unsafe { (*view.common.com_cl_running).integer != 0 && ui_game_command(view) != 0 }
    {
        return;
    }

    // send it as a server command if we are connected
    // this will usually result in a chat message
    //CL_ForwardCommandToServer ( text );
    let cl_forward = view
        .common
        .hooks
        .CL_ForwardCommandToServer
        .expect("CL_ForwardCommandToServer hook");
    cl_forward(view, text);
}
