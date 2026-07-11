#![allow(non_snake_case, non_camel_case_types, dead_code)]
//! `cmd_pc.cpp` — the registered-command list (`cmd_functions`) and dispatch
//! (`Cmd_ExecuteString`/`Cmd_List_f`/`Cmd_CommandCompletion`).
//!
//! Source: `oracle/codemp/qcommon/cmd_pc.cpp`

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;

use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::Com_Filter;

// PORT-NOTE(rm-types): `RenderModels`/`Server` are state-receiver types pinned
// by the engine-fork-discovery preamble's receiver order; neither has landed
// in this crate yet (`Server` lives in `mp_engine_server`, which already
// depends on this crate — importing it here would cycle). Referenced by their
// exact resolved-signature names per the no-stub rule; reported as missing
// symbols (`cmd_common.rs` precedent).
pub(crate) use crate::cm_load::RenderModels;
pub(crate) struct Server;

// PORT-NOTE(q_math-reach): `Q_stricmp` (q_shared primitive) is ported only in
// `mp_game`, a tier above this crate's dependency graph (cm_shader.rs
// precedent) — not reachable here. `Com_Printf` is not yet landed in this
// crate under any importable path. Both narrowed to this file's call-site
// shapes; escalated as missing symbols.
extern "Rust" {
    fn Q_stricmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn Com_Printf(common: &mut Common, msg: *const c_char);
    // PORT-NOTE(Cvar_Command): missing symbol — `qcommon::cvar` doesn't expose
    // this yet under this name/shape; resolution packet
    // `qcommon__1774_Cvar_Command.md`. Referenced verbatim; escalated as a
    // missing symbol.
    fn Cvar_Command(
        common: &mut Common,
        cm: &mut CollisionWorld,
        rm: &mut RenderModels,
        host: &mut dyn EngineHost,
    ) -> c_int;
    // PORT-NOTE(CL_GameCommand/UI_GameCommand/CL_ForwardCommandToServer): live
    // in `mp_engine_client`'s `null/` stubs (`null_client.rs`), which this
    // crate cannot depend on without cycling. Referenced verbatim per the
    // no-stub rule; escalated as missing symbols.
    fn CL_GameCommand() -> c_int;
    fn UI_GameCommand() -> c_int;
    fn CL_ForwardCommandToServer(text: *const c_char);
    // PORT-NOTE(SV_GameCommand): lives in `mp_engine_server::sv_game`, which
    // this crate cannot depend on without cycling. Referenced verbatim per
    // the no-stub rule; escalated as missing symbol.
    fn SV_GameCommand(common: &mut Common, sv: &mut Server) -> c_int;
}

// PORT-NOTE(cmd_function_t): `cmd_function_t` (linked-list node: `next`,
// `name`, `function`) has no rosetta row — `Common`'s command-registry field
// (`cmd_functions`) isn't landed yet either. Referenced verbatim per the
// no-stub rule; both escalated as missing symbols.
pub(crate) struct cmd_function_t {
    next: *mut cmd_function_t,
    name: *const c_char,
    function: Option<extern "C" fn()>,
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
            Com_Printf(common, format!("{}\n", name).as_ptr() as *const c_char);
            i += 1;
            cmd = (*cmd).next;
        }
    }
    unsafe {
        Com_Printf(
            common,
            format!("{} commands\n", i).as_ptr() as *const c_char,
        );
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
                    function();
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
    if !common.com_cl_running.is_null()
        && unsafe { (*common.com_cl_running).integer != 0 && CL_GameCommand() != 0 }
    {
        return;
    }

    // check server game commands
    if !common.com_sv_running.is_null()
        && unsafe { (*common.com_sv_running).integer != 0 && SV_GameCommand(common, sv) != 0 }
    {
        return;
    }

    // check ui commands
    if !common.com_cl_running.is_null()
        && unsafe { (*common.com_cl_running).integer != 0 && UI_GameCommand() != 0 }
    {
        return;
    }

    // send it as a server command if we are connected
    // this will usually result in a chat message
    //CL_ForwardCommandToServer ( text );
    unsafe {
        CL_ForwardCommandToServer(text);
    }
}
