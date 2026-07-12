#![allow(non_camel_case_types)]

use core::ffi::c_char;

use crate::common::engine_host_view::EngineHostView;

/// Console-command handler slot. Receiver-threaded in place of Raven's
/// global-reaching `void (*xcommand_t)(void)` (user ruling 2026-07-11); the
/// receiver list collapsed to the single `EngineHostView` world bundle in the
/// host-seam restructure (user ruling 2026-07-11 pt. 2, amending the pinned
/// receiver order): the dispatch site (`Cmd_ExecuteString`) passes the view in
/// scope there, and a handler that needs its island's real state casts the
/// view's type-erased slot at its boundary.
pub type CmdFunction = fn(&mut EngineHostView);

/// Raven `cmd_function_t` — a node in the registered-command linked list
/// (`Cmd_AddCommand`/`Cmd_ExecuteString`). `Common::cmd_functions` walks it as a
/// raw `*mut`, so it keeps Raven's C layout (`#[repr(C)]`, field order); a NULL
/// `function` marks a completion-only command handled by the cgame/game.
///
/// Type definition source: `oracle/codemp/qcommon/cmd_pc.cpp:3-8`
#[repr(C)]
pub struct cmd_function_t {
    pub next: *mut cmd_function_t,
    pub name: *mut c_char,
    pub function: Option<CmdFunction>,
}
