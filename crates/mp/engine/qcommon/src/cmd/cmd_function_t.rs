#![allow(non_camel_case_types)]

use core::ffi::c_char;

use mp_host_interface::engine_host::EngineHost;

use crate::cm_load::RmManager;
use crate::cmd_pc::{RenderModels, Server};
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::z_memman_pc::Ghoul2System;

/// Console-command handler slot. Receiver-threaded in place of Raven's
/// global-reaching `void (*xcommand_t)(void)` (user ruling 2026-07-11): the
/// dispatch site (`Cmd_ExecuteString`) threads the receivers in scope there
/// (`common`/`cm`/`sv`/`rm`/`rmg`/`g2`/`host`, pinned to match the
/// `EngineHooks::SV_Frame` order), so every registered command reaches real
/// engine state instead of a no-op shim.
pub type CmdFunction = fn(
    &mut Common,
    &mut CollisionWorld,
    &mut Server,
    &mut RenderModels,
    &mut RmManager,
    &mut Ghoul2System,
    &mut dyn EngineHost,
);

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
