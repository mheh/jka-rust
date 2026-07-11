#![allow(non_camel_case_types)]

use core::ffi::c_char;

use crate::qcommon::xcommand_t::xcommand_t;

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
    pub function: Option<xcommand_t>,
}
