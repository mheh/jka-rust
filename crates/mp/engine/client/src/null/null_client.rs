//! Raven's `null` client stubs — the DEDICATED/no-renderer build's client
//! entry points, every body an intentional no-op (no client to drive).
//!
//! Source: `oracle/codemp/null/null_client.cpp`

use std::os::raw::{c_char, c_int, c_uint};

use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::{fileHandle_t, qboolean, qfalse};

/// Raven `CL_Shutdown`.
///
/// Source: `oracle/codemp/null/null_client.cpp:9-10`
pub fn CL_Shutdown() {}

/// Raven `CL_MouseEvent`.
///
/// Source: `oracle/codemp/null/null_client.cpp:16-17`
pub fn CL_MouseEvent(dx: c_int, dy: c_int, time: c_int) {
    let _ = (dx, dy, time);
}

/// Raven `Key_WriteBindings`.
///
/// Source: `oracle/codemp/null/null_client.cpp:19-20`
pub fn Key_WriteBindings(f: fileHandle_t) {
    let _ = f;
}

/// Raven `CL_Frame`.
///
/// Source: `oracle/codemp/null/null_client.cpp:22-23`
pub fn CL_Frame(msec: c_int) {
    let _ = msec;
}

/// Raven `CL_PacketEvent`.
///
/// Source: `oracle/codemp/null/null_client.cpp:25-26`
pub fn CL_PacketEvent(from: netadr_t, msg: *mut msg_t) {
    let _ = (from, msg);
}

/// Raven `CL_CharEvent`.
///
/// Source: `oracle/codemp/null/null_client.cpp:28-29`
pub fn CL_CharEvent(key: c_int) {
    let _ = key;
}

/// Raven `CL_Disconnect`.
///
/// Source: `oracle/codemp/null/null_client.cpp:31-32`
pub fn CL_Disconnect(showMainMenu: qboolean) {
    let _ = showMainMenu;
}

/// Raven `CL_MapLoading`.
///
/// Source: `oracle/codemp/null/null_client.cpp:34-35`
pub fn CL_MapLoading() {}

/// Raven `CL_GameCommand`.
///
/// Raven: `return qfalse;`
/// Source: `oracle/codemp/null/null_client.cpp:37-39`
pub fn CL_GameCommand() -> qboolean {
    qfalse
}

/// Raven `CL_KeyEvent`.
///
/// Source: `oracle/codemp/null/null_client.cpp:41-42`
pub fn CL_KeyEvent(key: c_int, down: qboolean, time: c_uint) {
    let _ = (key, down, time);
}

/// Raven `UI_GameCommand`.
///
/// Raven: `return qfalse;`
/// Source: `oracle/codemp/null/null_client.cpp:44-46`
pub fn UI_GameCommand() -> qboolean {
    qfalse
}

/// Raven `CL_ForwardCommandToServer`.
///
/// Source: `oracle/codemp/null/null_client.cpp:48-49`
pub fn CL_ForwardCommandToServer(string: *const c_char) {
    let _ = string;
}

/// Raven `CL_ConsolePrint`.
///
/// Source: `oracle/codemp/null/null_client.cpp:51-52`
pub fn CL_ConsolePrint(txt: *const c_char, silent: qboolean) {
    let _ = (txt, silent);
}

/// Raven `CL_JoystickEvent`.
///
/// Source: `oracle/codemp/null/null_client.cpp:54-55`
pub fn CL_JoystickEvent(axis: c_int, value: c_int, time: c_int) {
    let _ = (axis, value, time);
}

/// Raven `CL_InitKeyCommands`.
///
/// Source: `oracle/codemp/null/null_client.cpp:57-58`
pub fn CL_InitKeyCommands() {}

/// Raven `CL_CDDialog`.
///
/// Source: `oracle/codemp/null/null_client.cpp:60-61`
pub fn CL_CDDialog(msg: *const c_char) {
    let _ = msg;
}

/// Raven `CL_FlushMemory`.
///
/// Source: `oracle/codemp/null/null_client.cpp:63-64`
pub fn CL_FlushMemory() {}

/// Raven `CL_StartHunkUsers`.
///
/// Source: `oracle/codemp/null/null_client.cpp:66-67`
pub fn CL_StartHunkUsers() {}

/// Raven `CL_Init`.
///
/// Raven: registers the `cl_shownet` cvar.
/// Source: `oracle/codemp/null/null_client.cpp:12-14`
pub fn CL_Init() {
    // This no-arg signature can't reach `Cvar_Get`; the real `cl_shownet`
    // registration is wired through `mp_engine_qcommon::common::engine_hooks::CL_Init_null`.
}
