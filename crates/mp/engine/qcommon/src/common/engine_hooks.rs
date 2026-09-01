//! `EngineHooks` — the qcommon->server/client/sound/renderer upcall table.
//!
//! Raven's one-binary C build resolves qcommon's calls up into `SV_*`/`CL_*`/
//! `SND_*`/`RE_*`/`R_*` symbols at link time. Here `mp_engine_qcommon` sits
//! BELOW `mp_engine_server`/`mp_engine_client` in the crate graph, so it cannot
//! import those symbols (that would cycle). Per the user ruling (2026-07-12) the
//! seam is a hook table carried on `Common`: one `Option<fn(...)>` per upward
//! symbol, installed by the app/core layer at boot.
//!
//! Host-seam restructure (user ruling 2026-07-11): every hook now takes
//! `&mut EngineHostView` — the single world bundle — in place of the former
//! pinned receiver list (`common`/`cm`/`sv`/`rm`/`rmg`/`g2`/`host`); the
//! adapter installed by the owning crate casts the view's type-erased slots
//! back to its real state. The table also carries the ACCESSOR hooks backing
//! the `EngineHost` methods qcommon cannot implement itself (`SV_Trace`,
//! `SV_GentityNum`, model memory, …) — see `engine_host_view.rs`.
//!
//! Which entrypoints get a null default vs. stay a mandatory hook follows
//! Raven's own dedicated-server link set (WinDed.vcproj): the dedicated binary
//! links `null_client.cpp`/`null_snddma.cpp` (so the `CL_*`/`SND_*` upcalls have
//! real no-op bodies) but the REAL `sv_*.cpp` and `tr_model.cpp` (so `SV_*` and
//! the model-cache `RE_*`/`R_*` upcalls are live code). The null bodies below
//! are faithful ports of `null_client.cpp`/`null_snddma.cpp`; the `SV_*`/`RE_*`/
//! `R_*` fields have NO default and are installed later by their owning
//! subsystem.
//!
//! `Option<fn(..)>` uses the null-pointer niche, so `None == 0` — every field is
//! zero-init-valid and covered by `Engine::new`'s `alloc_zeroed` mass; the
//! explicit `null_dedicated()` write then swaps in the client/sound no-ops.

use core::ffi::{c_char, c_int, c_void};

use mp_host_interface::VmSlot;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::cvar::CVAR_TEMP;
use mp_qshared::shared::vec3_t;
use native_types::{fileHandle_t, qboolean, qfalse, qhandle_t};

use crate::common::engine_host_view::EngineHostView;
use crate::cvar_fns::Cvar_Get;

/// The qcommon upcall table (see module doc). Field names are the exact Raven
/// symbol names for greppability (the accessor hooks that have no Raven
/// function — `SVS_Time`, `VM_CallSlot`, the `R_Model*` reads — are named for
/// the state/`EngineHost` method they back); each signature is fixed by its
/// qcommon call site(s). A field of `Common`.
#[allow(non_snake_case)]
pub struct EngineHooks {
    // ---- client tier (null-build defaults; `null_client.cpp`) ----
    /// Source: `oracle/codemp/null/null_client.cpp:9-10`
    pub CL_Shutdown: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:31-32`
    pub CL_Disconnect: Option<fn(&mut EngineHostView, qboolean)>,
    /// Source: `oracle/codemp/null/null_client.cpp:63-64`
    pub CL_FlushMemory: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:12-14`
    pub CL_Init: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:66-67`
    pub CL_StartHunkUsers: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:34-35`
    pub CL_MapLoading: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:25-26`
    pub CL_PacketEvent: Option<fn(&mut EngineHostView, netadr_t, *mut msg_t)>,
    /// Source: `oracle/codemp/null/null_client.cpp:22-23`
    pub CL_Frame: Option<fn(&mut EngineHostView, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:57-58`
    pub CL_InitKeyCommands: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/null/null_client.cpp:54-55`
    pub CL_JoystickEvent: Option<fn(&mut EngineHostView, c_int, c_int, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:16-17`
    pub CL_MouseEvent: Option<fn(&mut EngineHostView, c_int, c_int, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:28-29`
    pub CL_CharEvent: Option<fn(&mut EngineHostView, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:41-42`
    pub CL_KeyEvent: Option<fn(&mut EngineHostView, c_int, bool, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:48-49`
    pub CL_ForwardCommandToServer: Option<fn(&mut EngineHostView, &str)>,
    /// Source: `oracle/codemp/null/null_client.cpp:51-52`
    pub CL_ConsolePrint: Option<fn(&mut EngineHostView, &str, qboolean)>,
    /// Source: `oracle/codemp/null/null_client.cpp:37-39`
    pub CL_GameCommand: Option<fn(&mut EngineHostView) -> qboolean>,
    /// Source: `oracle/codemp/null/null_client.cpp:44-46`
    pub UI_GameCommand: Option<fn(&mut EngineHostView) -> qboolean>,
    /// Source: `oracle/codemp/null/null_client.cpp:19-20`
    pub Key_WriteBindings: Option<fn(&mut EngineHostView, fileHandle_t)>,

    // ---- client tier: the `#ifndef DEDICATED` calls `null_client.cpp` never defines ----
    /// Source: `oracle/codemp/client/cl_main.cpp:657`
    pub CL_ShutdownAll: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/client/cl_cgame.cpp:595`
    pub CL_ShutdownCGame: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/client/cl_ui.cpp:1444`
    pub CL_ShutdownUI: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/client/cl_cin.cpp:126`
    pub CIN_CloseAllVideos: Option<fn(&mut EngineHostView)>,

    // ---- sound tier (null-build defaults; `null_snddma.cpp`) ----
    /// Source: `oracle/codemp/null/null_snddma.cpp:46-49`
    pub SND_FreeOldestSound: Option<fn(&mut EngineHostView) -> c_int>,
    /// Source: `oracle/codemp/null/null_snddma.cpp:41-44`
    pub SND_RegisterAudio_LevelLoadEnd: Option<fn(&mut EngineHostView, qboolean) -> qboolean>,

    // ---- server tier (mandatory hooks, installed by mp_engine_server) ----
    /// Source: `oracle/codemp/server/sv_init.cpp:929`
    pub SV_Shutdown: Option<fn(&mut EngineHostView, &str)>,
    /// Source: `oracle/codemp/server/sv_init.cpp:803`
    pub SV_Init: Option<fn(&mut EngineHostView)>,
    /// Source: `oracle/codemp/server/sv_main.cpp:826`
    pub SV_Frame: Option<fn(&mut EngineHostView, c_int)>,
    /// Source: `oracle/codemp/server/sv_main.cpp:594`
    pub SV_PacketEvent: Option<fn(&mut EngineHostView, netadr_t, *mut msg_t)>,
    /// Source: `oracle/codemp/server/sv_game.cpp:1766`
    pub SV_GameCommand: Option<fn(&mut EngineHostView) -> qboolean>,
    /// Source: `oracle/codemp/server/sv_game.cpp:1666`
    pub SV_ShutdownGameProgs: Option<fn(&mut EngineHostView)>,

    // ---- server tier: EngineHost accessor hooks (engine_host_view.rs) ----
    /// Backs `EngineHost::trace` — Raven `SV_Trace`.
    /// Source: `oracle/codemp/server/sv_world.cpp:803`
    #[allow(clippy::type_complexity)]
    pub SV_Trace: Option<
        fn(
            &mut EngineHostView,
            &mut trace_t,
            vec3_t,
            vec3_t,
            vec3_t,
            vec3_t,
            c_int,
            c_int,
            c_int,
            c_int,
            c_int,
        ),
    >,
    /// Backs `EngineHost::gentity` — Raven `SV_GentityNum`.
    /// Source: `oracle/codemp/server/sv_game.cpp:54`
    pub SV_GentityNum: Option<fn(&mut EngineHostView, c_int) -> *mut sharedEntity_t>,
    /// Backs `EngineHost::shared_memory` — Raven `sv.mSharedMemory`.
    /// Source: `oracle/codemp/server/server.h:87`
    pub SV_SharedMemory: Option<fn(&mut EngineHostView) -> *mut c_char>,
    /// Backs `EngineHost::sv_time` — Raven `svs.time` (no Raven accessor fn).
    /// Source: `oracle/codemp/server/server.h:211`
    pub SVS_Time: Option<fn(&mut EngineHostView) -> c_int>,
    /// Backs `EngineHost::sv_shownet_entity_classname` (ruling 56c).
    /// Source: `oracle/codemp/qcommon/msg.cpp:1268-1270`
    pub SV_ShownetEntityClassname: Option<fn(&mut EngineHostView, c_int) -> Option<String>>,
    /// Backs `EngineHost::vm_call` — Raven `VM_Call( vm, … )` with the
    /// `VmSlot` -> `sv.gvm`/`cgvm` resolution on the server side (ruling 33b).
    /// Source: `oracle/codemp/qcommon/vm.cpp:787`
    pub VM_CallSlot: Option<fn(&mut EngineHostView, VmSlot, c_int, &[isize]) -> isize>,

    // ---- renderer-model tier (mandatory hooks; real `tr_model.cpp`) ----
    /// Source: `oracle/codemp/renderer/tr_model.cpp:337`
    pub RE_RegisterModels_LevelLoadEnd: Option<fn(&mut EngineHostView, qboolean) -> qboolean>,
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1683`
    pub R_HunkClearCrap: Option<fn(&mut EngineHostView)>,
    /// Backs `EngineHost::model_mdxm` — Raven `R_GetModelByHandle(h)->mdxm`.
    /// Returns `(block, parsed)`: the loader `.glm` block pointer and the DEC-35
    /// parse-once `MdxmParsed` sidecar pointer, both null when absent.
    /// Source: `oracle/codemp/renderer/tr_local.h:1128`
    pub R_ModelMdxm: Option<fn(&mut EngineHostView, qhandle_t) -> (*mut c_void, *const c_void)>,
    /// Backs `EngineHost::model_mdxa` — Raven `R_GetModelByHandle(h)->mdxa`.
    /// Returns `(block, parsed)`: the loader `.gla` block pointer and the DEC-35
    /// parse-once `MdxaParsed` sidecar pointer, both null when absent.
    /// Source: `oracle/codemp/renderer/tr_local.h:1129`
    pub R_ModelMdxa: Option<fn(&mut EngineHostView, qhandle_t) -> (*mut c_void, *const c_void)>,
    /// Backs `EngineHost::skin_surfaces` — Raven `R_GetSkinByHandle` flattened
    /// (server skins name-pool ruling, 2026-07-12).
    /// Source: `oracle/codemp/renderer/tr_image.cpp:3342-3347`
    pub R_SkinSurfaces: Option<fn(&mut EngineHostView, qhandle_t) -> Vec<(String, String)>>,
    /// Backs `EngineHost::model_register` — Raven `RE_RegisterServerModel`.
    /// Source: `oracle/codemp/renderer/tr_model.cpp:588`
    pub R_RegisterServerModel: Option<fn(&mut EngineHostView, &str) -> qhandle_t>,
    /// The client-path register twin, live once a client build exists.
    /// Source: `oracle/codemp/renderer/tr_model.cpp:497` (`RE_RegisterModel`)
    pub RE_RegisterModel: Option<fn(&mut EngineHostView, &str) -> qhandle_t>,
    /// Source: `oracle/codemp/renderer/tr_shader.cpp:116` (`ShaderHashTableExists`)
    pub ShaderHashTableExists: Option<fn(&mut EngineHostView) -> qboolean>,
}

impl EngineHooks {
    /// Boot state for a build that links `null_client.cpp`/`null_snddma.cpp`
    /// (Raven's dedicated set): the client/sound tier gets the null no-op
    /// bodies below; the mandatory `SV_*`/`RE_*`/`R_*` fields stay `None`
    /// until their owning subsystem installs them.
    pub fn null_dedicated() -> EngineHooks {
        EngineHooks {
            CL_Shutdown: Some(CL_Shutdown_null),
            CL_Disconnect: Some(CL_Disconnect_null),
            CL_FlushMemory: Some(CL_FlushMemory_null),
            CL_Init: Some(CL_Init_null),
            CL_StartHunkUsers: Some(CL_StartHunkUsers_null),
            CL_MapLoading: Some(CL_MapLoading_null),
            CL_PacketEvent: Some(CL_PacketEvent_null),
            CL_Frame: Some(CL_Frame_null),
            CL_InitKeyCommands: Some(CL_InitKeyCommands_null),
            CL_JoystickEvent: Some(CL_JoystickEvent_null),
            CL_MouseEvent: Some(CL_MouseEvent_null),
            CL_CharEvent: Some(CL_CharEvent_null),
            CL_KeyEvent: Some(CL_KeyEvent_null),
            CL_ForwardCommandToServer: Some(CL_ForwardCommandToServer_null),
            CL_ConsolePrint: Some(CL_ConsolePrint_null),
            CL_GameCommand: Some(CL_GameCommand_null),
            UI_GameCommand: Some(UI_GameCommand_null),
            Key_WriteBindings: Some(Key_WriteBindings_null),
            CL_ShutdownAll: Some(CL_ShutdownAll_null),
            CL_ShutdownCGame: Some(CL_ShutdownCGame_null),
            CL_ShutdownUI: Some(CL_ShutdownUI_null),
            CIN_CloseAllVideos: Some(CIN_CloseAllVideos_null),
            SND_FreeOldestSound: Some(SND_FreeOldestSound_null),
            SND_RegisterAudio_LevelLoadEnd: Some(SND_RegisterAudio_LevelLoadEnd_null),
            SV_Shutdown: None,
            SV_Init: None,
            SV_Frame: None,
            SV_PacketEvent: None,
            SV_GameCommand: None,
            SV_ShutdownGameProgs: None,
            SV_Trace: None,
            SV_GentityNum: None,
            SV_SharedMemory: None,
            SVS_Time: None,
            SV_ShownetEntityClassname: None,
            VM_CallSlot: None,
            RE_RegisterModels_LevelLoadEnd: None,
            R_HunkClearCrap: None,
            R_ModelMdxm: None,
            R_ModelMdxa: None,
            R_SkinSurfaces: None,
            R_RegisterServerModel: None,
            RE_RegisterModel: None,
            ShaderHashTableExists: None,
        }
    }
}

// ---- null_client.cpp bodies (faithful no-op ports) ----

/// Raven null `CL_Shutdown`. Source: `oracle/codemp/null/null_client.cpp:9-10`
#[allow(non_snake_case)]
fn CL_Shutdown_null(_view: &mut EngineHostView) {}

/// Raven null `CL_Init` — registers the `cl_shownet` cvar.
/// Raven stores the result in a file-scope `cl_shownet` cvar_t* no dedicated
/// build reads; only the registration side effect is kept.
/// Source: `oracle/codemp/null/null_client.cpp:12-14`
#[allow(non_snake_case)]
fn CL_Init_null(view: &mut EngineHostView) {
    Cvar_Get(view, "cl_shownet", "0", CVAR_TEMP);
}

/// Raven null `CL_Disconnect`. Source: `oracle/codemp/null/null_client.cpp:31-32`
#[allow(non_snake_case)]
fn CL_Disconnect_null(_view: &mut EngineHostView, _show_main_menu: qboolean) {}

/// Raven null `CL_FlushMemory`. Source: `oracle/codemp/null/null_client.cpp:63-64`
#[allow(non_snake_case)]
fn CL_FlushMemory_null(_view: &mut EngineHostView) {}

/// Raven null `CL_StartHunkUsers`. Source: `oracle/codemp/null/null_client.cpp:66-67`
#[allow(non_snake_case)]
fn CL_StartHunkUsers_null(_view: &mut EngineHostView) {}

/// Raven null `CL_MapLoading`. Source: `oracle/codemp/null/null_client.cpp:34-35`
#[allow(non_snake_case)]
fn CL_MapLoading_null(_view: &mut EngineHostView) {}

/// Raven null `CL_PacketEvent`. Source: `oracle/codemp/null/null_client.cpp:25-26`
#[allow(non_snake_case)]
fn CL_PacketEvent_null(_view: &mut EngineHostView, _from: netadr_t, _msg: *mut msg_t) {}

/// Raven null `CL_Frame`. Source: `oracle/codemp/null/null_client.cpp:22-23`
#[allow(non_snake_case)]
fn CL_Frame_null(_view: &mut EngineHostView, _msec: c_int) {}

/// Raven null `CL_InitKeyCommands`. Source: `oracle/codemp/null/null_client.cpp:57-58`
#[allow(non_snake_case)]
fn CL_InitKeyCommands_null(_view: &mut EngineHostView) {}

/// Raven null `CL_JoystickEvent`. Source: `oracle/codemp/null/null_client.cpp:54-55`
#[allow(non_snake_case)]
fn CL_JoystickEvent_null(_view: &mut EngineHostView, _axis: c_int, _value: c_int, _time: c_int) {}

/// Raven null `CL_MouseEvent`. Source: `oracle/codemp/null/null_client.cpp:16-17`
#[allow(non_snake_case)]
fn CL_MouseEvent_null(_view: &mut EngineHostView, _dx: c_int, _dy: c_int, _time: c_int) {}

/// Raven null `CL_CharEvent`. Source: `oracle/codemp/null/null_client.cpp:28-29`
#[allow(non_snake_case)]
fn CL_CharEvent_null(_view: &mut EngineHostView, _key: c_int) {}

/// Raven null `CL_KeyEvent`. Source: `oracle/codemp/null/null_client.cpp:41-42`
#[allow(non_snake_case)]
fn CL_KeyEvent_null(_view: &mut EngineHostView, _key: c_int, _down: bool, _time: c_int) {}

/// Raven null `CL_ForwardCommandToServer`. Source: `oracle/codemp/null/null_client.cpp:48-49`
#[allow(non_snake_case)]
fn CL_ForwardCommandToServer_null(_view: &mut EngineHostView, _string: &str) {}

/// Raven null `CL_ConsolePrint`. Source: `oracle/codemp/null/null_client.cpp:51-52`
#[allow(non_snake_case)]
fn CL_ConsolePrint_null(_view: &mut EngineHostView, _txt: &str, _silent: qboolean) {}

/// Raven null `CL_GameCommand` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_client.cpp:37-39`
#[allow(non_snake_case)]
fn CL_GameCommand_null(_view: &mut EngineHostView) -> qboolean {
    qfalse
}

/// Raven null `UI_GameCommand` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_client.cpp:44-46`
#[allow(non_snake_case)]
fn UI_GameCommand_null(_view: &mut EngineHostView) -> qboolean {
    qfalse
}

/// Raven null `Key_WriteBindings`. Source: `oracle/codemp/null/null_client.cpp:19-20`
#[allow(non_snake_case)]
fn Key_WriteBindings_null(_view: &mut EngineHostView, _f: fileHandle_t) {}

// ---- guard-excised client bodies ----
//
// `null_client.cpp` defines none of the four functions below.
// Raven's dedicated build removes the call site with an `#ifndef DEDICATED` guard instead, so each null body cites that guard.

/// The dedicated build never calls `CL_ShutdownAll`.
/// Source: `oracle/codemp/server/sv_init.cpp:513-516`
#[allow(non_snake_case)]
fn CL_ShutdownAll_null(_view: &mut EngineHostView) {}

/// The dedicated build never calls `CL_ShutdownCGame`.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:754-757`
#[allow(non_snake_case)]
fn CL_ShutdownCGame_null(_view: &mut EngineHostView) {}

/// The dedicated build never calls `CL_ShutdownUI`.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:754-757`
#[allow(non_snake_case)]
fn CL_ShutdownUI_null(_view: &mut EngineHostView) {}

/// The dedicated build never calls `CIN_CloseAllVideos`.
/// Source: `oracle/codemp/qcommon/z_memman_pc.cpp:760-762`
#[allow(non_snake_case)]
fn CIN_CloseAllVideos_null(_view: &mut EngineHostView) {}

// ---- null_snddma.cpp bodies (faithful no-op ports) ----

/// Raven null `SND_FreeOldestSound` — returns `0`.
/// Source: `oracle/codemp/null/null_snddma.cpp:46-49`
#[allow(non_snake_case)]
fn SND_FreeOldestSound_null(_view: &mut EngineHostView) -> c_int {
    0
}

/// Raven null `SND_RegisterAudio_LevelLoadEnd` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_snddma.cpp:41-44`
#[allow(non_snake_case)]
fn SND_RegisterAudio_LevelLoadEnd_null(
    _view: &mut EngineHostView,
    _something: qboolean,
) -> qboolean {
    qfalse
}
