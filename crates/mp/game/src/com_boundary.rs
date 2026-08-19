//! Print-sink boundary for the two ctx-free fn-pointer callers `Com_Printf` and `Com_Error` (`g_main.rs`).
//! It mirrors Raven's own file-static syscall pointer at the same boundary.
//!
//! Source: `oracle/codemp/game/g_syscalls.c` (file-static `trap_*` pointers filled by `dllEntry`).
//! The ctx-free callers here stand in for `oracle/codemp/game/g_main.c:1208-1228`.
//!
//! This is the SEAM-D1 narrow extension (DEC-12).
//! The shell registers these two fn pointers at `dllEntry`.
//! That lets `Com_Printf` and `Com_Error` reach `G_PRINT` and `G_ERROR` instead of `eprint!` and `panic!`.
//! The route only prints.
//! Widening it to a general ambient engine handle needs a new ruling.

use std::ffi::c_char;
use std::sync::OnceLock;

use mp_engine_select::Engine;

use crate::cstr_util::cstr_to_str;
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

/// This builds the shell's `Com_Printf` sink fn body.
/// It routes `msg` through `trap_Printf` (`G_PRINT`), matching `G_Printf` in `g_main.rs`.
///
/// Source: `oracle/codemp/game/g_main.c:1219-1228`
pub fn route_print(engine: &Engine, msg: *const c_char) {
    unsafe {
        let text = cstr_to_str(msg);
        trap::Printf(engine, &text);
    }
}

/// This builds the shell's `Com_Error` sink fn body.
/// It routes `msg` through `trap_Error` (`G_ERROR`), matching `G_Error` in `g_main.rs`.
///
/// Source: `oracle/codemp/game/g_main.c:1208-1217`
pub fn route_error(engine: &Engine, msg: *const c_char) {
    unsafe {
        let text = cstr_to_str(msg);
        trap::Error(engine, &text);
    }
}
