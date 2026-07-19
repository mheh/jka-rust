#![allow(non_snake_case, non_camel_case_types, dead_code)]
//! `cmd_pc.cpp` — the registered-command list (`cmd_functions`) and dispatch
//! (`Cmd_ExecuteString`/`Cmd_List_f`/`Cmd_CommandCompletion`).
//!
//! Source: `oracle/codemp/qcommon/cmd_pc.cpp`

use core::ffi::{c_char, CStr};
use std::ffi::CString;

use native_string::filter::Com_Filter;

use crate::cmd::cmd_function_t::{cmd_function_t, CmdFunction};
use crate::common::engine_host_view::EngineHostView;
use crate::common::Common;

// `Server` is a type-erased receiver slot: the real type lives in
// mp_engine_server, which depends on this crate (importing it would cycle).
// Re-exported at this historical home; defined once in `common::opaque_slots`.
pub use crate::common::opaque_slots::Server;

use crate::common::com_printf;
use crate::cvar_fns::Cvar_Command;

/// `Cmd_AddCommand`.
///
/// Raven `strcmp`s the exact name, `S_Malloc`s a node, `CopyString`s the
/// name, and links it at the list head; the owned Vec front-inserts a
/// `String`-named entry.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:18-39`
pub fn Cmd_AddCommand(view: &mut EngineHostView, cmd_name: &str, function: Option<CmdFunction>) {
    // fail if the command already exists
    if view.common.cmd_functions.iter().any(|c| c.name == cmd_name) {
        // allow completion-only commands to be silently doubled
        if function.is_some() {
            com_printf(
                view.common,
                &format!("Cmd_AddCommand: {cmd_name} already defined\n"),
            );
        }
        return;
    }
    view.common.cmd_functions.insert(
        0,
        cmd_function_t {
            name: cmd_name.to_owned(),
            function,
        },
    );
}

/// `Cmd_RemoveCommand`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:46-67`
pub fn Cmd_RemoveCommand(common: &mut Common, cmd_name: &str) {
    // Raven unlinks and frees the node; absent names return silently.
    if let Some(i) = common.cmd_functions.iter().position(|c| c.name == cmd_name) {
        common.cmd_functions.remove(i);
    }
}

/// `Cmd_CommandCompletion`.
///
/// Raven walks the registered-command list, invoking `callback` with each
/// command's name. The callback is a C seam: each name crosses as a
/// NUL-terminated `CString` for the call's duration.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:75-81`
pub fn Cmd_CommandCompletion(common: &mut Common, callback: extern "C" fn(*const c_char)) {
    for cmd in &common.cmd_functions {
        let name = CString::new(cmd.name.as_str()).unwrap_or_default();
        callback(name.as_ptr());
    }
}

/// `Cmd_List_f`.
///
/// Source: `oracle/codemp/qcommon/cmd_pc.cpp:153-173`
pub fn Cmd_List_f(common: &mut Common) {
    let match_: Option<String> = if crate::cmd_common::Cmd_Argc(common) > 1 {
        let arg = crate::cmd_common::Cmd_Argv(common, 1);
        Some(
            unsafe { CStr::from_ptr(arg) }
                .to_string_lossy()
                .into_owned(),
        )
    } else {
        None
    };

    let names: Vec<String> = common
        .cmd_functions
        .iter()
        .filter(|cmd| {
            match_
                .as_deref()
                .map_or(true, |m| Com_Filter(m, &cmd.name, false))
        })
        .map(|cmd| cmd.name.clone())
        .collect();
    for name in &names {
        com_printf(common, &format!("{name}\n"));
    }
    com_printf(common, &format!("{} commands\n", names.len()));
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
    let arg0 = unsafe { CStr::from_ptr(crate::cmd_common::Cmd_Argv(view.common, 0)) }
        .to_bytes()
        .to_vec();
    if let Some(idx) = view
        .common
        .cmd_functions
        .iter()
        .position(|c| c.name.as_bytes().eq_ignore_ascii_case(&arg0))
    {
        // rearrange the links so that the command will be
        // near the head of the list next time it is used
        let cmd = view.common.cmd_functions.remove(idx);
        let function = cmd.function;
        view.common.cmd_functions.insert(0, cmd);

        // perform the action
        if let Some(function) = function {
            function(view);
            return;
        }
        // NULL function: let the cgame or game handle it (fall through, as
        // Raven's `break`)
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
