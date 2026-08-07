//! Boot installation of the client tier's `EngineHooks` entries.
//!
//! Raven picks the sound tier at link time: the dedicated build links
//! `null_snddma.cpp` and the client build links `snd_dma.cpp`. One binary ships
//! both here, so the installed hook reads the view's `snd` slot and takes the
//! null-build answer when `Engine.snd` is `None`.
//!
//! Source: `oracle/codemp/null/null_snddma.cpp:41-49`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uint};

use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::EngineHooks;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_types::fileHandle_t;

use crate::cl_cgame::CL_GameCommand;
use crate::cl_console::CL_ConsolePrint;
use crate::cl_input::{CL_JoystickEvent, CL_MouseEvent};
use crate::cl_keys::{CL_CharEvent, CL_InitKeyCommands, CL_KeyEvent, Key_WriteBindings};
use crate::cl_main::{
    CL_Disconnect, CL_FlushMemory, CL_ForwardCommandToServer, CL_Frame, CL_Init, CL_MapLoading,
    CL_PacketEvent, CL_Shutdown, CL_StartHunkUsers,
};
use crate::cl_ui::UI_GameCommand;
use crate::client_host::{cl_from_view, snd_from_view};
use crate::snd_dma::{SND_FreeOldestSound, SND_RegisterAudio_LevelLoadEnd};

/// Install the client tier's hook fields over the null-build defaults.
/// Runs once in `main()` beside the server and renderer installers.
pub fn install_engine_hooks(hooks: &mut EngineHooks) {
    hooks.SND_FreeOldestSound = Some(SND_FreeOldestSound_hook);
    hooks.SND_RegisterAudio_LevelLoadEnd = Some(SND_RegisterAudio_LevelLoadEnd_hook);
}

/// Install the real `CL_*`/`Key_*` bodies over the null-build defaults.
/// Raven picks this tier at link time: `jamp` links `cl_main.cpp`, the
/// dedicated build links `null_client.cpp`. The core installer calls this only
/// when `Engine.cl` is seated, so `jampded` keeps the null table untouched and
/// every adapter below may cast the `cl` slot without a null check.
pub fn install_client_engine_hooks(hooks: &mut EngineHooks) {
    hooks.CL_Shutdown = Some(CL_Shutdown_hook);
    hooks.CL_Disconnect = Some(CL_Disconnect_hook);
    hooks.CL_FlushMemory = Some(CL_FlushMemory_hook);
    hooks.CL_Init = Some(CL_Init_hook);
    hooks.CL_StartHunkUsers = Some(CL_StartHunkUsers_hook);
    hooks.CL_MapLoading = Some(CL_MapLoading_hook);
    hooks.CL_PacketEvent = Some(CL_PacketEvent_hook);
    hooks.CL_Frame = Some(CL_Frame_hook);
    hooks.CL_InitKeyCommands = Some(CL_InitKeyCommands_hook);
    hooks.CL_JoystickEvent = Some(CL_JoystickEvent_hook);
    hooks.CL_MouseEvent = Some(CL_MouseEvent_hook);
    hooks.CL_CharEvent = Some(CL_CharEvent_hook);
    hooks.CL_KeyEvent = Some(CL_KeyEvent_hook);
    hooks.CL_ForwardCommandToServer = Some(CL_ForwardCommandToServer_hook);
    hooks.CL_ConsolePrint = Some(CL_ConsolePrint_hook);
    hooks.CL_GameCommand = Some(CL_GameCommand_hook);
    hooks.UI_GameCommand = Some(UI_GameCommand_hook);
    hooks.Key_WriteBindings = Some(Key_WriteBindings_hook);
}

/// Raven `CL_Shutdown`. Source: `oracle/codemp/client/cl_main.cpp:2719`
fn CL_Shutdown_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_Shutdown(view, cl);
}

/// Raven `CL_Disconnect`. Source: `oracle/codemp/client/cl_main.cpp:837`
fn CL_Disconnect_hook(view: &mut EngineHostView, show_main_menu: qboolean) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_Disconnect(view, cl, show_main_menu);
}

/// Raven `CL_FlushMemory`. Source: `oracle/codemp/client/cl_main.cpp:734`
fn CL_FlushMemory_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_FlushMemory(view, cl);
}

/// Raven `CL_Init`. Source: `oracle/codemp/client/cl_main.cpp:2549`
fn CL_Init_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_Init(view, cl);
}

/// Raven `CL_StartHunkUsers`. Source: `oracle/codemp/client/cl_main.cpp:2445`
fn CL_StartHunkUsers_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_StartHunkUsers(view, cl);
}

/// Raven `CL_MapLoading`. Source: `oracle/codemp/client/cl_main.cpp:778`
fn CL_MapLoading_hook(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_MapLoading(view, cl);
}

/// Raven `CL_PacketEvent`. Source: `oracle/codemp/client/cl_main.cpp:2151`
fn CL_PacketEvent_hook(view: &mut EngineHostView, from: netadr_t, msg: *mut msg_t) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_PacketEvent(view, cl, from, msg);
}

/// Raven `CL_Frame`. Source: `oracle/codemp/client/cl_main.cpp:2268`
fn CL_Frame_hook(view: &mut EngineHostView, msec: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_Frame(view, cl, msec);
}

/// Raven `CL_InitKeyCommands`. Source: `oracle/codemp/client/cl_keys.cpp:1403`
fn CL_InitKeyCommands_hook(view: &mut EngineHostView) {
    CL_InitKeyCommands(view);
}

/// Raven `CL_JoystickEvent`. Source: `oracle/codemp/client/cl_input.cpp:1022`
fn CL_JoystickEvent_hook(view: &mut EngineHostView, axis: c_int, value: c_int, time: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_JoystickEvent(cl, axis, value, time);
}

/// Raven `CL_MouseEvent`. Source: `oracle/codemp/client/cl_input.cpp:992`
fn CL_MouseEvent_hook(view: &mut EngineHostView, dx: c_int, dy: c_int, time: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_MouseEvent(view.common, cl, dx, dy, time);
}

/// Raven `CL_CharEvent`. Source: `oracle/codemp/client/cl_keys.cpp:1658`
fn CL_CharEvent_hook(view: &mut EngineHostView, key: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_CharEvent(view.common, cl, key);
}

/// Raven `CL_KeyEvent`. Source: `oracle/codemp/client/cl_keys.cpp:1462`
fn CL_KeyEvent_hook(view: &mut EngineHostView, key: c_int, down: bool, time: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    let down = if down { qtrue } else { qfalse };
    CL_KeyEvent(view, cl, key, down, time as c_uint);
}

/// Raven `CL_ForwardCommandToServer`.
/// Source: `oracle/codemp/client/cl_main.cpp:913`
fn CL_ForwardCommandToServer_hook(view: &mut EngineHostView, string: &str) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_ForwardCommandToServer(view.common, cl, string);
}

/// Raven `CL_ConsolePrint`, which `com_printf` reaches through the queue on `Common`.
/// The port keeps Raven's `char*` parameter, so the text gets a NUL terminator in a temporary buffer here.
/// Source: `oracle/codemp/client/cl_console.cpp:356-433`
fn CL_ConsolePrint_hook(view: &mut EngineHostView, txt: &str, silent: qboolean) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    let mut buffer: Vec<u8> = txt.as_bytes().to_vec();
    buffer.push(0);
    CL_ConsolePrint(view.common, cl, buffer.as_ptr() as *const c_char, silent);
}

/// Raven `CL_GameCommand`. Source: `oracle/codemp/client/cl_cgame.cpp:1815`
fn CL_GameCommand_hook(view: &mut EngineHostView) -> qboolean {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    CL_GameCommand(view.common, cl)
}

/// Raven `UI_GameCommand`. Source: `oracle/codemp/client/cl_ui.cpp:1513`
fn UI_GameCommand_hook(view: &mut EngineHostView) -> qboolean {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    UI_GameCommand(view.common, cl)
}

/// Raven `Key_WriteBindings`. Source: `oracle/codemp/client/cl_keys.cpp:1367`
fn Key_WriteBindings_hook(view: &mut EngineHostView, f: fileHandle_t) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let cl = unsafe { cl_from_view(view) };
    Key_WriteBindings(view.common, cl, f);
}

/// Raven `SND_FreeOldestSound(void)`, which the zone allocator calls to recover
/// from a failed `Z_Malloc`.
/// Source: `oracle/codemp/client/snd_dma.cpp:5216-5219`
fn SND_FreeOldestSound_hook(view: &mut EngineHostView) -> c_int {
    if view.snd.as_raw().is_null() {
        return 0;
    }
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    SND_FreeOldestSound(view, snd, None)
}

/// Raven `SND_RegisterAudio_LevelLoadEnd`, which the renderer and the zone
/// allocator call to bring the audio pool back under its cap.
/// Source: `oracle/codemp/client/snd_dma.cpp:5228`
fn SND_RegisterAudio_LevelLoadEnd_hook(
    view: &mut EngineHostView,
    bDeleteEverythingNotUsedThisLevel: qboolean,
) -> qboolean {
    if view.snd.as_raw().is_null() {
        return qfalse;
    }
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    let dropped = SND_RegisterAudio_LevelLoadEnd(view, snd, bDeleteEverythingNotUsedThisLevel != 0);
    if dropped {
        qtrue
    } else {
        qfalse
    }
}
