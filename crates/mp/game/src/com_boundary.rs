//! Print-sink seam for the two ctx-free fn-ptr boundaries `Com_Printf`/
//! `Com_Error` (`g_main.rs`), mirroring Raven's own file-static syscall
//! pointer at this same boundary.
//!
//! Source: `oracle/oracle/codemp/game/g_syscalls.c` (file-static
//! `trap_*` pointers filled by `dllEntry`); the ctx-free callers this
//! stands in for are `oracle/oracle/codemp/game/g_main.c:1208-1228`.
//!
//! Narrow SEAM-D1 extension (approved 2026-07-06): the shell registers these
//! two fn pointers at `dllEntry` so `Com_Printf`/`Com_Error` can reach
//! `G_PRINT`/`G_ERROR` instead of `eprint!`/`panic!`. Deliberately print-only
//! — widening this to a general ambient engine handle needs a new ruling.

use std::ffi::c_char;
use std::sync::OnceLock;

use mp_abi::game::syscalls::G_ERROR::GErrorArgs;
use mp_abi::game::syscalls::G_PRINT::GPrintArgs;
use mp_engine_select::Engine;

use crate::cstr_util::{cstr, cstr_to_str};
use crate::trap;

static COM_PRINT_SINK: OnceLock<fn(*const c_char)> = OnceLock::new();
static COM_ERROR_SINK: OnceLock<fn(*const c_char)> = OnceLock::new();

/// Registers the shell's `Com_Printf` route (called once from `dllEntry`).
pub fn set_com_print_sink(sink: fn(*const c_char)) {
    COM_PRINT_SINK.set(sink).ok();
}

/// Registers the shell's `Com_Error` route (called once from `dllEntry`).
pub fn set_com_error_sink(sink: fn(*const c_char)) {
    COM_ERROR_SINK.set(sink).ok();
}

/// The registered `Com_Printf` sink, if `dllEntry` has run.
pub(crate) fn com_print_sink() -> Option<fn(*const c_char)> {
    COM_PRINT_SINK.get().copied()
}

/// The registered `Com_Error` sink, if `dllEntry` has run.
pub(crate) fn com_error_sink() -> Option<fn(*const c_char)> {
    COM_ERROR_SINK.get().copied()
}

/// Builds the shell's `Com_Printf` sink fn body: routes `msg` through
/// `trap_Printf` (`G_PRINT`) exactly as `G_Printf` does (`g_main.rs`).
///
/// Source: `oracle/oracle/codemp/game/g_main.c:1219-1228`
pub fn route_print(engine: &Engine, msg: *const c_char) {
    unsafe {
        let text = cstr_to_str(msg);
        trap::Printf(engine, GPrintArgs::new(cstr(&text)));
    }
}

/// Builds the shell's `Com_Error` sink fn body: routes `msg` through
/// `trap_Error` (`G_ERROR`) exactly as `G_Error` does (`g_main.rs`).
///
/// Source: `oracle/oracle/codemp/game/g_main.c:1208-1217`
pub fn route_error(engine: &Engine, msg: *const c_char) {
    unsafe {
        let text = cstr_to_str(msg);
        trap::Error(engine, GErrorArgs::new(cstr(&text)));
    }
}
