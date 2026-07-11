//! `EngineHooks` — the qcommon->server/client/sound/renderer upcall table.
//!
//! Raven's one-binary C build resolves qcommon's calls up into `SV_*`/`CL_*`/
//! `SND_*`/`RE_*`/`R_*` symbols at link time. Here `mp_engine_qcommon` sits
//! BELOW `mp_engine_server`/`mp_engine_client` in the crate graph, so it cannot
//! import those symbols (that would cycle). Per the user ruling (2026-07-12) the
//! seam is a hook table carried on `Common`: one `Option<fn(...)>` per upward
//! symbol, installed by the app/core layer at boot.
//!
//! Which entrypoints get a null default vs. stay a mandatory hook follows
//! Raven's own dedicated-server link set (WinDed.vcproj): the dedicated binary
//! links `null_client.cpp`/`null_snddma.cpp` (so the `CL_*`/`SND_*` upcalls have
//! real no-op bodies) but the REAL `sv_*.cpp` and `tr_model.cpp` (so `SV_*` and
//! the model-cache `RE_*`/`R_*` upcalls are live code). The null bodies below
//! are faithful ports of `null_client.cpp`/`null_snddma.cpp`; the `SV_*`/`RE_*`/
//! `R_HunkClearCrap` fields have NO default and are installed later by their
//! owning subsystem.
//!
//! `Option<fn(..)>` uses the null-pointer niche, so `None == 0` — every field is
//! zero-init-valid and covered by `Engine::new`'s `alloc_zeroed` mass; the
//! explicit `null_dedicated()` write then swaps in the client/sound no-ops.

use core::ffi::{c_char, c_int};

use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::cvar::CVAR_TEMP;
use native_types::{fileHandle_t, qboolean, qfalse};

use crate::cm_load::{RenderModels, RmManager};
use crate::cmd_pc::Server;
use crate::collision_world::CollisionWorld;
use crate::common::Common;
use crate::common_fns::{BotLib, Client};
use crate::z_memman_pc::Ghoul2System;

/// The qcommon upcall table (see module doc). Field names are the exact Raven
/// symbol names for greppability; each signature is fixed by its qcommon call
/// site(s). A field of `Common`.
#[allow(non_snake_case)]
pub struct EngineHooks {
    // ---- client tier (null-build defaults; `null_client.cpp`) ----
    /// Source: `oracle/codemp/null/null_client.cpp:9-10`
    pub CL_Shutdown: Option<fn()>,
    /// Source: `oracle/codemp/null/null_client.cpp:12-14`
    pub CL_Init: Option<
        fn(&mut Common, &mut CollisionWorld, &mut Client, &mut RenderModels, &mut dyn EngineHost),
    >,
    /// Source: `oracle/codemp/null/null_client.cpp:66-67`
    pub CL_StartHunkUsers: Option<fn()>,
    /// Source: `oracle/codemp/null/null_client.cpp:25-26`
    pub CL_PacketEvent: Option<fn(netadr_t, *mut msg_t)>,
    /// Source: `oracle/codemp/null/null_client.cpp:22-23`
    pub CL_Frame: Option<fn(c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:57-58`
    pub CL_InitKeyCommands: Option<fn()>,
    /// Source: `oracle/codemp/null/null_client.cpp:54-55`
    pub CL_JoystickEvent: Option<fn(c_int, c_int, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:16-17`
    pub CL_MouseEvent: Option<fn(c_int, c_int, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:28-29`
    pub CL_CharEvent: Option<fn(c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:41-42`
    pub CL_KeyEvent: Option<fn(c_int, bool, c_int)>,
    /// Source: `oracle/codemp/null/null_client.cpp:48-49`
    pub CL_ForwardCommandToServer: Option<fn(*const c_char)>,
    /// Source: `oracle/codemp/null/null_client.cpp:37-39`
    pub CL_GameCommand: Option<fn() -> qboolean>,
    /// Source: `oracle/codemp/null/null_client.cpp:44-46`
    pub UI_GameCommand: Option<fn() -> qboolean>,
    /// Source: `oracle/codemp/null/null_client.cpp:19-20`
    pub Key_WriteBindings: Option<fn(fileHandle_t)>,

    // ---- sound tier (null-build defaults; `null_snddma.cpp`) ----
    /// Source: `oracle/codemp/null/null_snddma.cpp:46-49`
    pub SND_FreeOldestSound: Option<fn(&mut dyn EngineHost) -> c_int>,
    /// Source: `oracle/codemp/null/null_snddma.cpp:41-44`
    pub SND_RegisterAudio_LevelLoadEnd: Option<fn(&mut dyn EngineHost, qboolean) -> qboolean>,

    // ---- server tier (mandatory hooks, installed by mp_engine_server) ----
    /// Source: `oracle/codemp/server/sv_init.cpp:929`
    pub SV_Shutdown: Option<
        fn(
            &mut Common,
            &mut CollisionWorld,
            &mut Server,
            &mut RenderModels,
            &mut RmManager,
            &mut dyn EngineHost,
            &str,
        ),
    >,
    /// Source: `oracle/codemp/server/sv_init.cpp:803`
    pub SV_Init: Option<
        fn(
            &mut Common,
            &mut CollisionWorld,
            &mut Server,
            &mut BotLib,
            &mut RenderModels,
            &mut dyn EngineHost,
        ),
    >,
    /// Source: `oracle/codemp/server/sv_main.cpp:826`
    pub SV_Frame: Option<
        fn(
            &mut Common,
            &mut CollisionWorld,
            &mut Server,
            &mut RenderModels,
            &mut RmManager,
            &mut Ghoul2System,
            &mut dyn EngineHost,
            c_int,
        ),
    >,
    /// Source: `oracle/codemp/server/sv_main.cpp:594`
    pub SV_PacketEvent: Option<
        fn(
            &mut Common,
            &mut CollisionWorld,
            &mut Server,
            &mut RenderModels,
            &mut RmManager,
            &mut dyn EngineHost,
            netadr_t,
            *mut msg_t,
        ),
    >,
    /// Source: `oracle/codemp/server/sv_game.cpp:1766`
    pub SV_GameCommand: Option<fn(&mut Common, &mut Server) -> qboolean>,
    /// Source: `oracle/codemp/server/sv_game.cpp:1666`
    pub SV_ShutdownGameProgs: Option<fn(&mut Common, &mut Server)>,

    // ---- renderer-model tier (mandatory hooks; real `tr_model.cpp`) ----
    /// Source: `oracle/codemp/renderer/tr_model.cpp:337`
    pub RE_RegisterModels_LevelLoadEnd: Option<fn(&mut RenderModels, &mut dyn EngineHost, qboolean) -> qboolean>,
    /// Source: `oracle/codemp/renderer/tr_model.cpp:1683`
    pub R_HunkClearCrap: Option<fn(&mut RenderModels, &mut dyn EngineHost)>,
}

impl EngineHooks {
    /// Boot state for a build that links `null_client.cpp`/`null_snddma.cpp`
    /// (Raven's dedicated set): the client/sound tier gets the null no-op
    /// bodies below; the mandatory `SV_*`/`RE_*`/`R_HunkClearCrap` fields stay
    /// `None` until their owning subsystem installs them.
    pub fn null_dedicated() -> EngineHooks {
        EngineHooks {
            CL_Shutdown: Some(CL_Shutdown_null),
            CL_Init: Some(CL_Init_null),
            CL_StartHunkUsers: Some(CL_StartHunkUsers_null),
            CL_PacketEvent: Some(CL_PacketEvent_null),
            CL_Frame: Some(CL_Frame_null),
            CL_InitKeyCommands: Some(CL_InitKeyCommands_null),
            CL_JoystickEvent: Some(CL_JoystickEvent_null),
            CL_MouseEvent: Some(CL_MouseEvent_null),
            CL_CharEvent: Some(CL_CharEvent_null),
            CL_KeyEvent: Some(CL_KeyEvent_null),
            CL_ForwardCommandToServer: Some(CL_ForwardCommandToServer_null),
            CL_GameCommand: Some(CL_GameCommand_null),
            UI_GameCommand: Some(UI_GameCommand_null),
            Key_WriteBindings: Some(Key_WriteBindings_null),
            SND_FreeOldestSound: Some(SND_FreeOldestSound_null),
            SND_RegisterAudio_LevelLoadEnd: Some(SND_RegisterAudio_LevelLoadEnd_null),
            SV_Shutdown: None,
            SV_Init: None,
            SV_Frame: None,
            SV_PacketEvent: None,
            SV_GameCommand: None,
            SV_ShutdownGameProgs: None,
            RE_RegisterModels_LevelLoadEnd: None,
            R_HunkClearCrap: None,
        }
    }
}

// ---- null_client.cpp bodies (faithful no-op ports) ----

/// Raven null `CL_Shutdown`. Source: `oracle/codemp/null/null_client.cpp:9-10`
#[allow(non_snake_case)]
fn CL_Shutdown_null() {}

/// Raven null `CL_Init` — registers the `cl_shownet` cvar.
/// Raven stores the result in a file-scope `cl_shownet` cvar_t* no dedicated
/// build reads; only the registration side effect is kept.
/// Source: `oracle/codemp/null/null_client.cpp:12-14`
#[allow(non_snake_case)]
fn CL_Init_null(
    common: &mut Common,
    cm: &mut CollisionWorld,
    cl: &mut Client,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let _ = cl;
    crate::cvar_fns::Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"cl_shownet".as_ptr(),
        c"0".as_ptr(),
        CVAR_TEMP,
    );
}

/// Raven null `CL_StartHunkUsers`. Source: `oracle/codemp/null/null_client.cpp:66-67`
#[allow(non_snake_case)]
fn CL_StartHunkUsers_null() {}

/// Raven null `CL_PacketEvent`. Source: `oracle/codemp/null/null_client.cpp:25-26`
#[allow(non_snake_case)]
fn CL_PacketEvent_null(_from: netadr_t, _msg: *mut msg_t) {}

/// Raven null `CL_Frame`. Source: `oracle/codemp/null/null_client.cpp:22-23`
#[allow(non_snake_case)]
fn CL_Frame_null(_msec: c_int) {}

/// Raven null `CL_InitKeyCommands`. Source: `oracle/codemp/null/null_client.cpp:57-58`
#[allow(non_snake_case)]
fn CL_InitKeyCommands_null() {}

/// Raven null `CL_JoystickEvent`. Source: `oracle/codemp/null/null_client.cpp:54-55`
#[allow(non_snake_case)]
fn CL_JoystickEvent_null(_axis: c_int, _value: c_int, _time: c_int) {}

/// Raven null `CL_MouseEvent`. Source: `oracle/codemp/null/null_client.cpp:16-17`
#[allow(non_snake_case)]
fn CL_MouseEvent_null(_dx: c_int, _dy: c_int, _time: c_int) {}

/// Raven null `CL_CharEvent`. Source: `oracle/codemp/null/null_client.cpp:28-29`
#[allow(non_snake_case)]
fn CL_CharEvent_null(_key: c_int) {}

/// Raven null `CL_KeyEvent`. Source: `oracle/codemp/null/null_client.cpp:41-42`
#[allow(non_snake_case)]
fn CL_KeyEvent_null(_key: c_int, _down: bool, _time: c_int) {}

/// Raven null `CL_ForwardCommandToServer`. Source: `oracle/codemp/null/null_client.cpp:48-49`
#[allow(non_snake_case)]
fn CL_ForwardCommandToServer_null(_string: *const c_char) {}

/// Raven null `CL_GameCommand` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_client.cpp:37-39`
#[allow(non_snake_case)]
fn CL_GameCommand_null() -> qboolean {
    qfalse
}

/// Raven null `UI_GameCommand` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_client.cpp:44-46`
#[allow(non_snake_case)]
fn UI_GameCommand_null() -> qboolean {
    qfalse
}

/// Raven null `Key_WriteBindings`. Source: `oracle/codemp/null/null_client.cpp:19-20`
#[allow(non_snake_case)]
fn Key_WriteBindings_null(_f: fileHandle_t) {}

// ---- null_snddma.cpp bodies (faithful no-op ports) ----

/// Raven null `SND_FreeOldestSound` — returns `0`.
/// Source: `oracle/codemp/null/null_snddma.cpp:46-49`
#[allow(non_snake_case)]
fn SND_FreeOldestSound_null(_host: &mut dyn EngineHost) -> c_int {
    0
}

/// Raven null `SND_RegisterAudio_LevelLoadEnd` — returns `qfalse`.
/// Source: `oracle/codemp/null/null_snddma.cpp:41-44`
#[allow(non_snake_case)]
fn SND_RegisterAudio_LevelLoadEnd_null(_host: &mut dyn EngineHost, _something: qboolean) -> qboolean {
    qfalse
}
