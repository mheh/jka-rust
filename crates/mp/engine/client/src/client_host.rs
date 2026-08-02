//! `Client` (the `Engine.cl` island host) + `SoundSystem` (`Engine.snd`) +
//! the two armed client-slot syscall targets (`cgame_system_calls_shim`,
//! `ui_system_calls_shim`), which read the boot-built `ClientDispatchCtx` note.

use core::ffi::{c_char, c_int, c_long, c_uint, c_ushort};
use std::ffi::CString;

use mp_abi::cgame::shared_buffer::autoMapInput_t;
use mp_engine_botlib::BotLib;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::common::com_error;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::opaque_slots;
use mp_engine_qcommon::vm::vm_s::vm_t;
use mp_engine_server::Server;
use mp_qshared::shared::cvar::CvarHandle;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::limits::{
    MAX_GENTITIES, MAX_PINGREQUESTS, MAX_SERVERSTATUSREQUESTS, MAX_TOKEN_CHARS,
};
use mp_qshared::shared::{qboolean, qfalse};
use native_math::vector::vec3_t;
use native_platform::zeroed_box;
use native_types::{field_t, MAX_QPATH};

use crate::cin::cin_cache::cin_cache;
use crate::cin::cin_consts::MAX_VIDEO_HANDLES;
use crate::cin::cinematics_t::cinematics_t;
use crate::cl_cgame::CL_CgameSystemCalls;
use crate::cl_ui::CL_UISystemCalls;
use crate::client::client_active_t::clientActive_t;
use crate::client::client_connection_t::clientConnection_t;
use crate::client::client_static_t::clientStatic_t;
use crate::client::console_t::console_t;
use crate::client::graphsamp_t::graphsamp_t;
use crate::client::kbutton_t::kbutton_t;
use crate::client::ping_t::ping_t;
use crate::client::server_status_t::serverStatus_t;
use crate::client_dispatch_ctx::ClientDispatchCtx;
use crate::keys::key_globals_s::{keyGlobals_t, MAX_KEYS};
use crate::keys::keyname_t::{keyname_t, KEYNAMES};

/// Raven `MAX_SCR_LINES` — lines the center-print string may wrap to.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:508`
pub const MAX_SCR_LINES: usize = 10;

/// Raven `SCR_DebugGraph` ring length — `values[1024]`, masked with `1023`.
///
/// Source: `oracle/codemp/client/cl_scrn.cpp:319`
pub const MAX_GRAPH_SAMPLES: usize = 1024;

/// The all-zero `kbutton_t`, `field_t`, and `autoMapInput_t` images. Raven
/// declares each one as a zero-filled file static, and none of the three types
/// carries a `Default`, so `Client::default` writes these instead.
const KBUTTON_ZERO: kbutton_t = kbutton_t {
    down: [0; 2],
    downtime: 0,
    msec: 0,
    active: qfalse,
    wasPressed: qfalse,
};
const FIELD_ZERO: field_t = field_t {
    cursor: 0,
    scroll: 0,
    widthInChars: 0,
    buffer: [0; 256],
};
const AUTOMAP_INPUT_ZERO: autoMapInput_t = autoMapInput_t {
    up: 0.0,
    down: 0.0,
    yaw: 0.0,
    pitch: 0.0,
    goToDefaults: qfalse,
};

/// The client-island state owned by `Engine.cl: Option<Client>`, and `None` on dedicated.
/// The five boxed aggregates are Raven's zero-filled client globals, and the flat
/// fields after them are the `codemp/client/*.cpp` file-scope globals and statics
/// (state-ownership § Client).
/// Each aggregate is a `Box` because the 2.6 MB mass must never transit the stack (STATE-D9 `zeroed_box`).
///
/// Source: `oracle/codemp/client/cl_main.cpp:105-107`
pub struct Client {
    /// Raven `cl` - the active game state that the engine parses from the server and wipes per gamestate.
    /// `cl.mSharedMemory` stays the raw module window, the same as `sv.mSharedMemory` on the server.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:105`
    pub cl: Box<clientActive_t>,
    /// Raven `clc` - the connection state that the engine wipes on every connect and every disconnect.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:106`
    pub clc: Box<clientConnection_t>,
    /// Raven `cls` - the client state that survives level loads, so the engine never wipes it.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:107`
    pub cls: Box<clientStatic_t>,
    /// Raven `kg` - the key bindings, the key-down table, and the console edit field with its history.
    ///
    /// Source: `oracle/codemp/client/cl_keys.cpp:17`
    pub kg: Box<keyGlobals_t>,
    /// Raven `con` - the console scrollback buffer and its display state.
    ///
    /// Source: `oracle/codemp/client/cl_console.cpp:13`
    pub con: Box<console_t>,

    // ---- `cl_main.cpp` file-scope globals ----
    /// Raven `cgvm` / `uivm` — the cgame and ui virtual machines, null until
    /// `CL_InitCGame` / `CL_InitUI` create them.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:108`; `oracle/codemp/client/cl_ui.cpp:28`
    pub cgvm: *mut vm_t,
    pub uivm: *mut vm_t,
    /// Raven `cl_pinglist` — the outstanding server-ping slots.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:113`
    pub cl_pinglist: Box<[ping_t; MAX_PINGREQUESTS]>,
    /// Raven `cl_serverStatusList` / `serverStatusCount` — the server-status
    /// request cache and its rolling allocation counter.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:125-126`
    pub cl_serverStatusList: Box<[serverStatus_t; MAX_SERVERSTATUSREQUESTS]>,
    pub serverStatusCount: c_int,
    /// Raven `CL_Record_f::demoName` (`static char[MAX_QPATH]`) — the record
    /// command's name buffer, a function-scope static that outlives the call.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:294`
    pub demoName: [c_char; MAX_QPATH],
    /// Raven `CL_Frame::frameCount` / `avgFrametime` — the `cl_framerate` report
    /// accumulators, function-scope statics that persist across frames.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:2265-2266`
    pub frameCount: c_uint,
    pub avgFrametime: f32,
    /// The four strings `cls.glconfig`'s `const char *` fields point at. Raven
    /// points them at the renderer's own static buffers, which the port's
    /// renderer owns as `String`s, so the client keeps the NUL-terminated copies.
    ///
    /// Source: `oracle/codemp/cgame/tr_types.h:299-303`
    pub glconfigStrings: [CString; 4],
    /// Raven `CL_Shutdown::recursive` — the re-entry guard, a function-scope
    /// static that persists across the call.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:2720`
    pub recursive: qboolean,
    /// Raven `cl_main.cpp` `cvar_t*` globals — cached registration handles
    /// (`None` = Raven's not-yet-registered null).
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:41-94`
    pub cl_nodelta: Option<CvarHandle>,
    pub cl_debugMove: Option<CvarHandle>,
    pub cl_noprint: Option<CvarHandle>,
    pub cl_motd: Option<CvarHandle>,
    pub rcon_client_password: Option<CvarHandle>,
    pub rconAddress: Option<CvarHandle>,
    pub cl_timeout: Option<CvarHandle>,
    pub cl_maxpackets: Option<CvarHandle>,
    pub cl_packetdup: Option<CvarHandle>,
    pub cl_timeNudge: Option<CvarHandle>,
    pub cl_showTimeDelta: Option<CvarHandle>,
    pub cl_freezeDemo: Option<CvarHandle>,
    pub cl_shownet: Option<CvarHandle>,
    pub cl_showSend: Option<CvarHandle>,
    pub cl_timedemo: Option<CvarHandle>,
    pub cl_avidemo: Option<CvarHandle>,
    pub cl_forceavidemo: Option<CvarHandle>,
    pub cl_freelook: Option<CvarHandle>,
    pub cl_sensitivity: Option<CvarHandle>,
    pub cl_mouseAccel: Option<CvarHandle>,
    pub cl_showMouseRate: Option<CvarHandle>,
    pub m_pitchVeh: Option<CvarHandle>,
    pub m_pitch: Option<CvarHandle>,
    pub m_yaw: Option<CvarHandle>,
    pub m_forward: Option<CvarHandle>,
    pub m_side: Option<CvarHandle>,
    pub m_filter: Option<CvarHandle>,
    pub cl_activeAction: Option<CvarHandle>,
    pub cl_motdString: Option<CvarHandle>,
    pub cl_allowDownload: Option<CvarHandle>,
    pub cl_allowAltEnter: Option<CvarHandle>,
    pub cl_conXOffset: Option<CvarHandle>,
    pub cl_inGameVideo: Option<CvarHandle>,
    pub cl_serverStatusResendTime: Option<CvarHandle>,
    pub cl_framerate: Option<CvarHandle>,
    pub cl_autolodscale: Option<CvarHandle>,

    // ---- `cl_input.cpp` file-scope globals ----
    /// Raven `frame_msec` / `old_com_frameTime` — the key-hold timing window
    /// that `CL_KeyState` divides by.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:11-12`
    pub frame_msec: c_uint,
    pub old_com_frameTime: c_int,
    /// Raven `cl_mPitchOverride` / `cl_mYawOverride` / `cl_mSensitivityOverride`
    /// — the vehicle-mode mouse overrides that cgame writes through
    /// `CG_SET_ORIENTATION`. Zero means "no override".
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:14-16`
    pub cl_mPitchOverride: f32,
    pub cl_mYawOverride: f32,
    pub cl_mSensitivityOverride: f32,
    /// Raven `cl_bUseFighterPitch` / `cl_crazyShipControls` — the vehicle
    /// control-scheme flags cgame sets alongside the mouse overrides.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:17-18`
    pub cl_bUseFighterPitch: qboolean,
    pub cl_crazyShipControls: qboolean,
    /// Raven `in_*` key-hold trackers, and `in_buttons[16]` for the numbered
    /// `+button<n>` commands.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:46-51`
    pub in_left: kbutton_t,
    pub in_right: kbutton_t,
    pub in_forward: kbutton_t,
    pub in_back: kbutton_t,
    pub in_lookup: kbutton_t,
    pub in_lookdown: kbutton_t,
    pub in_moveleft: kbutton_t,
    pub in_moveright: kbutton_t,
    pub in_strafe: kbutton_t,
    pub in_speed: kbutton_t,
    pub in_up: kbutton_t,
    pub in_down: kbutton_t,
    pub in_buttons: [kbutton_t; 16],
    /// Raven `in_mlooking` — true while `+mlook` holds the mouse in look mode.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:54`
    pub in_mlooking: qboolean,
    /// Raven `g_clAutoMapMode` / `g_clAutoMapInput` — the automap capture flag
    /// and the movement deltas `CL_AutoMapKey` accumulates for cgame.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:388,559`
    pub g_clAutoMapMode: bool,
    pub g_clAutoMapInput: autoMapInput_t,
    /// Raven `cl_input.cpp` `cvar_t*` globals — the turn-rate set.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:875-880`
    pub cl_yawspeed: Option<CvarHandle>,
    pub cl_pitchspeed: Option<CvarHandle>,
    pub cl_run: Option<CvarHandle>,
    pub cl_anglespeedkey: Option<CvarHandle>,
    /// Raven `in_joystick` — the platform input layer's cvar handle, read by
    /// `CL_JoystickMove`.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:1034`
    pub in_joystick: Option<CvarHandle>,
    /// Raven `cl_sendAngles` / `cl_lastViewAngles` — the angles the last
    /// usercmd carried and the angles before them, for the vehicle deltas.
    ///
    /// Source: `oracle/codemp/client/cl_input.cpp:1347-1348`
    pub cl_sendAngles: vec3_t,
    pub cl_lastViewAngles: vec3_t,

    // ---- `cl_keys.cpp` file-scope globals ----
    /// Raven `chatField` / `chat_team` / `chat_playerNum` — the message-mode
    /// edit line and its target.
    ///
    /// Source: `oracle/codemp/client/cl_keys.cpp:12-15`
    pub chatField: field_t,
    pub chat_team: qboolean,
    pub chat_playerNum: c_int,
    /// Raven `keynames[MAX_KEYS]` — the key name/keynum table that
    /// `Key_StringToKeynum` and `Key_KeynumToString` walk.
    /// The rows come from the `KEYNAMES` const and nothing writes them back.
    ///
    /// Source: `oracle/codemp/client/keys.h:46`; `oracle/codemp/client/cl_keys.cpp:22-353`
    pub keynames: Box<[keyname_t; MAX_KEYS]>,
    /// Raven `completionString` / `shortestMatch` / `matchCount` — the
    /// command-completion pass state that `FindMatches` accumulates.
    ///
    /// Source: `oracle/codemp/client/cl_keys.cpp:658-660`
    pub completionString: *const c_char,
    pub shortestMatch: [c_char; MAX_TOKEN_CHARS],
    pub matchCount: c_int,
    /// Raven `Key_WriteBindings::tinyString` — the `bind` line scratch buffer.
    ///
    /// Source: `oracle/codemp/client/cl_keys.cpp:1087`
    pub tinyString: [c_char; 16],

    // ---- `cl_console.cpp` file-scope globals ----
    /// Raven `g_console_field_width` — the console edit line width in characters.
    ///
    /// Source: `oracle/codemp/client/cl_console.cpp:11`
    pub g_console_field_width: c_int,
    /// Raven `con_conspeed` / `con_notifytime` cvar handles.
    ///
    /// Source: `oracle/codemp/client/cl_console.cpp:15-16`
    pub con_conspeed: Option<CvarHandle>,
    pub con_notifytime: Option<CvarHandle>,
    /// Raven `Con_DrawSolidConsole::iFontIndexForAsian` — the lazily registered
    /// `ocr_a` font handle, a function-scope static reused across frames.
    ///
    /// Source: `oracle/codemp/client/cl_console.cpp:666`
    pub iFontIndexForAsian: c_int,

    // ---- `cl_scrn.cpp` file-scope globals ----
    /// Raven `scr_initialized` — true once `SCR_Init` has registered its cvars.
    ///
    /// Source: `oracle/codemp/client/cl_scrn.cpp:9`
    pub scr_initialized: qboolean,
    /// Raven `cl_scrn.cpp` `cvar_t*` globals — the debug-graph set.
    ///
    /// Source: `oracle/codemp/client/cl_scrn.cpp:11-15`
    pub cl_timegraph: Option<CvarHandle>,
    pub cl_debuggraph: Option<CvarHandle>,
    pub cl_graphheight: Option<CvarHandle>,
    pub cl_graphscale: Option<CvarHandle>,
    pub cl_graphshift: Option<CvarHandle>,
    /// Raven `current` / `values[1024]` — the `SCR_DebugGraph` ring and its
    /// write cursor.
    ///
    /// Source: `oracle/codemp/client/cl_scrn.cpp:318-319`
    pub current: c_int,
    pub values: Box<[graphsamp_t; MAX_GRAPH_SAMPLES]>,
    /// Raven `scr_centertime_off` / `scr_centerstring` / `scr_center_lines` /
    /// `scr_center_widths` — the center-print message and its wrap layout.
    ///
    /// Source: `oracle/codemp/client/cl_scrn.cpp:510-515`
    pub scr_centertime_off: f32,
    pub scr_centerstring: [c_char; 1024],
    pub scr_center_lines: c_int,
    pub scr_center_widths: [c_int; MAX_SCR_LINES],
    /// Raven `scr_centertime` cvar handle.
    ///
    /// Source: `oracle/codemp/client/cl_scrn.cpp:517`
    pub scr_centertime: Option<CvarHandle>,

    // ---- `cl_parse.cpp` file-scope globals ----
    /// Raven `CL_SystemInfoChanged::hiddenCvarVal` — the scratch buffer that
    /// holds a hidden cvar's value while the systeminfo string is rewritten.
    ///
    /// Source: `oracle/codemp/client/cl_parse.cpp:20`
    pub hiddenCvarVal: [c_char; 128],
    /// Raven `cl_connectedToPureServer` / `cl_connectedGAME` /
    /// `cl_connectedCGAME` / `cl_connectedUI` — the pure-server checksum gates.
    ///
    /// Source: `oracle/codemp/client/cl_parse.cpp:378-381`
    pub cl_connectedToPureServer: c_int,
    pub cl_connectedGAME: c_int,
    pub cl_connectedCGAME: c_int,
    pub cl_connectedUI: c_int,

    // ---- `cl_cin.cpp` file-scope globals ----
    /// Raven `ROQ_*_tab[256]` — the YUV-to-RGB lookup tables `RllSetupTable`
    /// fills once.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:56-60`
    pub ROQ_YY_tab: Box<[c_long; 256]>,
    pub ROQ_UB_tab: Box<[c_long; 256]>,
    pub ROQ_UG_tab: Box<[c_long; 256]>,
    pub ROQ_VG_tab: Box<[c_long; 256]>,
    pub ROQ_VR_tab: Box<[c_long; 256]>,
    /// Raven `vq2` / `vq4` / `vq8` — the 2x2, 4x4, and 8x8 vector-quantizer
    /// codebooks the RoQ blitters read.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:61-63`
    pub vq2: Box<[c_ushort; 256 * 16 * 4]>,
    pub vq4: Box<[c_ushort; 256 * 64 * 4]>,
    pub vq8: Box<[c_ushort; 256 * 256 * 4]>,
    /// Raven `cin` / `cinTable` — the shared decode surface and the per-handle
    /// playback slots.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:117-118`
    pub cin: Box<cinematics_t>,
    pub cinTable: Box<[cin_cache; MAX_VIDEO_HANDLES]>,
    /// Raven `gCLTotalClientNum` (`cl_cgame.cpp`) — the client count the last
    /// gamestate carried, which scales the automatic LOD distance.
    ///
    /// Source: `oracle/codemp/client/cl_cgame.cpp:275`
    pub gCLTotalClientNum: c_int,
    /// Raven `newsize` (`cl_net_chan.cpp`) — the running total of decoded
    /// message bytes, the counterpart of qcommon's `oldsize`.
    ///
    /// Source: `oracle/codemp/client/cl_net_chan.cpp:150`
    pub newsize: c_int,
    /// Raven `currentHandle` / `CL_handle` — the slot the decoder is inside and
    /// the slot the client plays full screen. Both start at `-1`, so
    /// `Client::default` writes them after the zero fill.
    ///
    /// Source: `oracle/codemp/client/cl_cin.cpp:119-120`
    pub currentHandle: c_int,
    pub CL_handle: c_int,

    // ---- `snd_dma.cpp` file-scope globals ----
    // The sound stack itself is a pending lane (gh#24/gh#25, DEC-57). These are
    // the globals the already-ported client files read and write, so they take
    // their carrier home now and the lane fills in the behavior around them.
    /// Raven `s_shutUp` — silences the per-frame sound spam when set.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:16`
    pub s_shutUp: qboolean,
    /// Raven `s_soundMuted` — true between `S_DisableSounds` and the next restart.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:130`
    pub s_soundMuted: qboolean,
    /// Raven `s_volume` cvar handle.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:151`
    pub s_volume: Option<CvarHandle>,
    /// Raven `s_entityWavVol[MAX_GENTITIES]` — the per-entity lipsync volume
    /// the cgame reads through `CG_S_GETVOICEVOLUME`.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:193`
    pub s_entityWavVol: Box<[c_int; MAX_GENTITIES]>,
    /// Raven `s_rawend` / `s_soundtime` — the raw-sample write cursor and the
    /// mixer's current sample time, which the RoQ audio path advances.
    ///
    /// Source: `oracle/codemp/client/snd_dma.cpp:504,1731`
    pub s_rawend: c_int,
    pub s_soundtime: c_int,

    /// Raven `g_nOverrideChecked` (`cl_input.cpp`) — false again after a
    /// `vid_restart`, so the net overrides are re-read for the new mod.
    ///
    /// Source: `oracle/codemp/client/cl_main.cpp:1310-1316`
    pub g_nOverrideChecked: bool,

    /// The client referee state (`cl_referee.rs`), which is new engine tooling
    /// and not a Raven field. `Default` is `Off`, so a retail boot carries an
    /// inactive referee the same way `Server.referee` does.
    pub referee: crate::cl_referee::ClientReferee,
}

impl Default for Client {
    /// Returns the all-zero client island, the direct dual of Raven's zero-filled client globals.
    /// Every boxed field is `ZeroValid`, so each box comes back zeroed and never builds on the stack.
    /// The two cinematic handles then take Raven's `-1` static initializer.
    fn default() -> Self {
        Self {
            cl: zeroed_box(),
            clc: zeroed_box(),
            cls: zeroed_box(),
            kg: zeroed_box(),
            con: zeroed_box(),

            cgvm: core::ptr::null_mut(),
            uivm: core::ptr::null_mut(),
            cl_pinglist: zeroed_box(),
            cl_serverStatusList: zeroed_box(),
            serverStatusCount: 0,
            demoName: [0; MAX_QPATH],
            frameCount: 0,
            avgFrametime: 0.0,
            glconfigStrings: Default::default(),
            recursive: qfalse,
            cl_nodelta: None,
            cl_debugMove: None,
            cl_noprint: None,
            cl_motd: None,
            rcon_client_password: None,
            rconAddress: None,
            cl_timeout: None,
            cl_maxpackets: None,
            cl_packetdup: None,
            cl_timeNudge: None,
            cl_showTimeDelta: None,
            cl_freezeDemo: None,
            cl_shownet: None,
            cl_showSend: None,
            cl_timedemo: None,
            cl_avidemo: None,
            cl_forceavidemo: None,
            cl_freelook: None,
            cl_sensitivity: None,
            cl_mouseAccel: None,
            cl_showMouseRate: None,
            m_pitchVeh: None,
            m_pitch: None,
            m_yaw: None,
            m_forward: None,
            m_side: None,
            m_filter: None,
            cl_activeAction: None,
            cl_motdString: None,
            cl_allowDownload: None,
            cl_allowAltEnter: None,
            cl_conXOffset: None,
            cl_inGameVideo: None,
            cl_serverStatusResendTime: None,
            cl_framerate: None,
            cl_autolodscale: None,

            frame_msec: 0,
            old_com_frameTime: 0,
            cl_mPitchOverride: 0.0,
            cl_mYawOverride: 0.0,
            cl_mSensitivityOverride: 0.0,
            cl_bUseFighterPitch: qfalse,
            cl_crazyShipControls: qfalse,
            in_left: KBUTTON_ZERO,
            in_right: KBUTTON_ZERO,
            in_forward: KBUTTON_ZERO,
            in_back: KBUTTON_ZERO,
            in_lookup: KBUTTON_ZERO,
            in_lookdown: KBUTTON_ZERO,
            in_moveleft: KBUTTON_ZERO,
            in_moveright: KBUTTON_ZERO,
            in_strafe: KBUTTON_ZERO,
            in_speed: KBUTTON_ZERO,
            in_up: KBUTTON_ZERO,
            in_down: KBUTTON_ZERO,
            in_buttons: [KBUTTON_ZERO; 16],
            in_mlooking: qfalse,
            g_clAutoMapMode: false,
            g_clAutoMapInput: AUTOMAP_INPUT_ZERO,
            cl_yawspeed: None,
            cl_pitchspeed: None,
            cl_run: None,
            cl_anglespeedkey: None,
            in_joystick: None,
            cl_sendAngles: [0.0; 3],
            cl_lastViewAngles: [0.0; 3],

            chatField: FIELD_ZERO,
            chat_team: qfalse,
            chat_playerNum: 0,
            keynames: Box::new(KEYNAMES),
            completionString: core::ptr::null(),
            shortestMatch: [0; MAX_TOKEN_CHARS],
            matchCount: 0,
            tinyString: [0; 16],

            g_console_field_width: 78,
            con_conspeed: None,
            con_notifytime: None,
            iFontIndexForAsian: 0,

            scr_initialized: qfalse,
            cl_timegraph: None,
            cl_debuggraph: None,
            cl_graphheight: None,
            cl_graphscale: None,
            cl_graphshift: None,
            current: 0,
            values: zeroed_box(),
            scr_centertime_off: 0.0,
            scr_centerstring: [0; 1024],
            scr_center_lines: 0,
            scr_center_widths: [0; MAX_SCR_LINES],
            scr_centertime: None,

            hiddenCvarVal: [0; 128],
            cl_connectedToPureServer: 0,
            cl_connectedGAME: 0,
            cl_connectedCGAME: 0,
            cl_connectedUI: 0,

            ROQ_YY_tab: zeroed_box(),
            ROQ_UB_tab: zeroed_box(),
            ROQ_UG_tab: zeroed_box(),
            ROQ_VG_tab: zeroed_box(),
            ROQ_VR_tab: zeroed_box(),
            vq2: zeroed_box(),
            vq4: zeroed_box(),
            vq8: zeroed_box(),
            cin: zeroed_box(),
            cinTable: zeroed_box(),
            gCLTotalClientNum: 0,
            newsize: 0,
            currentHandle: -1,
            CL_handle: -1,

            s_shutUp: qfalse,
            s_soundMuted: qfalse,
            s_volume: None,
            s_entityWavVol: zeroed_box(),
            s_rawend: 0,
            s_soundtime: 0,

            g_nOverrideChecked: false,

            referee: Default::default(),
        }
    }
}

/// Cast the view's type-erased `cl` slot back to the live `Client` — the twin
/// of `mp_engine_server`'s `sv_from_view`. The raw pointer is copied out first
/// (`as_raw`), so the returned borrow is NOT tied to the view, and the per-slot
/// rule governs its use: nothing called while this borrow is live may cast the
/// SAME slot again.
///
/// Every client-tier hook body, every `Cmd_AddCommand` adapter, and every
/// client function that holds a view reaches `Client` through here.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.cl`; the engine is single-threaded and no
/// other cast of this slot is live for the returned borrow's duration. The slot
/// is NULL on dedicated (`Engine.cl` is `None`), where the null-build client
/// hooks run instead and never call this.
pub unsafe fn cl_from_view<'a>(view: &mut EngineHostView) -> &'a mut Client {
    &mut *(view.cl.as_raw() as *mut Client)
}

/// Cast the view's type-erased `g2` slot back to the live `Ghoul2System` — the
/// client's own boundary cast, since the cgame and ui dispatchers take `g2` as
/// a declared receiver (DEC-55.2's G2 trap block).
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.g2`; single-threaded, and no other cast
/// of this slot is live for the returned borrow's duration.
pub unsafe fn g2_from_view<'a>(view: &mut EngineHostView) -> &'a mut Ghoul2System {
    &mut *(view.g2.as_raw() as *mut Ghoul2System)
}

/// Cast the view's type-erased `sv` slot back to the live `Server`. Raven keeps
/// one process-wide `botlib_export`, and the port gives it one home on `Server`
/// (DEC-32), so the client's seven `PC_*` trap arms read it through here.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.sv`; single-threaded, and no other cast
/// of this slot is live for the returned borrow's duration.
pub unsafe fn sv_from_view<'a>(view: &mut EngineHostView) -> &'a mut Server {
    &mut *(view.sv.as_raw() as *mut Server)
}

/// Cast the view's type-erased `bot` slot back to the live `BotLib`, the
/// receiver every `botlib_export_t` entry except `PC_AddGlobalDefine` takes.
///
/// SAFETY (caller): the slot was built by `mp_engine_core`'s view constructor
/// from the live, unique `&mut Engine.bot`; single-threaded, and no other cast
/// of this slot is live for the returned borrow's duration.
pub unsafe fn bot_from_view<'a>(view: &mut EngineHostView) -> &'a mut BotLib {
    &mut *(view.bot.as_raw() as *mut BotLib)
}

/// Rebuild the dispatcher receivers from the boot-built note, the shared body
/// of the two shims below.
///
/// # Safety
/// `ctx` must be the boot-built [`ClientDispatchCtx`] (leaked, process
/// lifetime; every pointer is a field of the one boxed `Engine`, so the
/// addresses are stable). The engine caller that entered the module sits
/// suspended in `VM_Call` for this whole dispatch (single-threaded synchronous
/// traps), so its borrows of these same objects are dormant — the DEC-23
/// slot-cast discipline at the module seam.
unsafe fn client_dispatch_note<'a>(
    ctx: *mut core::ffi::c_void,
    who: &str,
) -> (&'a ClientDispatchCtx, &'a mut Client) {
    if ctx.is_null() {
        // A syscall before the boot arming would read a fabricated world — a
        // silent fake (porting-rules #14); die loudly instead.
        com_error(
            errorParm_t::ERR_FATAL,
            format!("{who} syscall with no dispatch context armed"),
        );
    }
    let c = &*(ctx as *const ClientDispatchCtx);
    let Some(cl) = (*c.cl).as_mut() else {
        // The module loaded before the client island was seated, so there is no
        // `cl`/`clc`/`cls` to answer with. Same loud disposition as above.
        com_error(
            errorParm_t::ERR_FATAL,
            format!("{who} syscall with no client island seated"),
        );
    };
    (c, cl)
}

/// Copy the trampoline's 16 words into the frame both client dispatchers
/// declare (`args: *mut isize`), at the width the shim delivered them.
///
/// Raven's `int args[16]` is the ILP32 shape, and the server dispatcher already
/// reads the widened `isize` word (`sv_game.rs`'s `vma`, `VM_ArgPtrWord`). The
/// client pair follows it: this engine hosts a 64-bit cgame and a 64-bit ui
/// module, so a pointer argument does not fit in a `c_int`, and macOS arm64
/// maps nothing below 4 GB for it to fit into. Value arguments are still read
/// as `c_int` inside the dispatchers, which is Raven's own width for them.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:366` (`int args[16]`).
///
/// # Safety
/// `args` must point at a shim's 16-word frame.
unsafe fn client_dispatch_frame(args: *const isize) -> [isize; 16] {
    let mut frame = [0isize; 16];
    for (i, w) in frame.iter_mut().enumerate() {
        *w = *args.add(i);
    }
    frame
}

/// The narrowed `int args[16]` frame the ui dispatcher still declares.
///
/// `CL_UISystemCalls` reads every word as a `c_int` value, so it keeps the
/// ILP32 shape until it takes the same widening the cgame dispatcher just did.
/// A ui trap that carries a host pointer truncates here, which is the open half
/// of the width finding recorded on ticket gh#30.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:366` (`int args[16]`).
///
/// # Safety
/// `args` must point at a shim's 16-word frame.
unsafe fn client_dispatch_frame_narrow(args: *const isize) -> [c_int; 16] {
    let mut frame = [0 as c_int; 16];
    for (i, w) in frame.iter_mut().enumerate() {
        *w = *args.add(i) as c_int;
    }
    frame
}

/// Rebuild the live world view from the dispatch note, the shape both client
/// dispatchers declare. This is the shim-side twin of
/// `mp_engine_core::engine_host_view`, which cannot run here because the note
/// holds raw pointers rather than the one `&mut Engine`.
///
/// # Safety
/// `c` must be the boot-built note, whose pointers are fields of the one boxed
/// `Engine` and therefore stable for the process. The engine caller sits
/// suspended in `VM_Call` for the whole dispatch, so its borrows are dormant.
unsafe fn client_dispatch_view<'a>(c: &ClientDispatchCtx) -> EngineHostView<'a> {
    let cl_raw = match (*c.cl).as_mut() {
        Some(cl) => cl as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    let re_raw = match (*c.re).as_mut() {
        Some(re) => re as *mut _ as *mut (),
        None => core::ptr::null_mut(),
    };
    EngineHostView {
        sv: opaque_slots::Server::from_raw(c.sv),
        cl: opaque_slots::Client::from_raw(cl_raw),
        bot: opaque_slots::BotLib::from_raw(c.bot),
        rm: opaque_slots::RenderModels::from_raw(c.rm as *mut ()),
        re: opaque_slots::Renderer::from_raw(re_raw),
        rmg: opaque_slots::RmManager::from_raw(c.rmg as *mut ()),
        g2: opaque_slots::Ghoul2System::from_raw(c.g2 as *mut ()),
        common: &mut *c.common,
        cm: &mut *c.cm,
    }
}

/// The injected `SlotSyscall` target for the cgame slot (LOAD-D8 injection):
/// the module's variadic syscall lands here (through
/// `cgame_syscall_trampoline`'s 16-word frame) with the slot's armed `ctx` —
/// the [`ClientDispatchCtx`] note `mp_engine_core::install_engine_hooks` built
/// at boot. Rebuild the dispatcher's receivers from the note and enter
/// `cl_cgame.rs::CL_CgameSystemCalls`, our routing dual of Raven's
/// `currentVM->systemCall( args )`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:377`;
/// `oracle/codemp/client/cl_cgame.cpp:644` (the dispatcher itself).
pub extern "C-unwind" fn cgame_system_calls_shim(
    ctx: *mut core::ffi::c_void,
    args: *const isize,
) -> isize {
    // SAFETY: the note's contract, restated on `client_dispatch_note`. `args`
    // is the trampoline's 16-word frame.
    unsafe {
        let (c, cl) = client_dispatch_note(ctx, "cgame");
        let mut view = client_dispatch_view(c);
        let mut frame = client_dispatch_frame(args);
        CL_CgameSystemCalls(
            &mut view,
            cl,
            &mut *c.rm,
            &mut *c.rmg,
            &mut *c.g2,
            &mut *c.roff,
            frame.as_mut_ptr(),
        ) as isize
    }
}

/// The ui slot's twin of [`cgame_system_calls_shim`], entering
/// `cl_ui.rs::CL_UISystemCalls`.
///
/// Source: `oracle/codemp/qcommon/vm.cpp:377`;
/// `oracle/codemp/client/cl_ui.cpp:813` (the dispatcher itself).
pub extern "C-unwind" fn ui_system_calls_shim(
    ctx: *mut core::ffi::c_void,
    args: *const isize,
) -> isize {
    // SAFETY: the note's contract, restated on `client_dispatch_note`. `args`
    // is the trampoline's 16-word frame.
    unsafe {
        let (c, cl) = client_dispatch_note(ctx, "ui");
        let mut view = client_dispatch_view(c);
        let mut frame = client_dispatch_frame_narrow(args);
        CL_UISystemCalls(&mut view, cl, &mut *c.g2, frame.as_mut_ptr()) as isize
    }
}

/// Widen the legacy `VM_DllSyscall` int arg block to the trampoline's `isize`
/// words and dispatch through an armed client slot — the shared body of the
/// two `vm->systemCall` adapters (`cl_cgame.rs`/`cl_ui.rs`), the twin of
/// `sv_game_system_call`.
///
/// # Safety
/// `args` must be the legacy convention's contiguous 16-int arg block
/// (`args[i] = va_arg(...)`, `vm.cpp:366`).
pub unsafe fn client_legacy_syscall(
    args: *mut c_int,
    dispatch: extern "C-unwind" fn(*const isize) -> isize,
) -> c_int {
    let mut frame = [0isize; 16];
    for (i, w) in frame.iter_mut().enumerate() {
        *w = *args.add(i) as isize;
    }
    dispatch(frame.as_ptr()) as c_int
}

/// The `Engine.snd` faithful mixer (DEC-03; EAX/force-feedback dropped). `None`
/// on dedicated (`S_Init` gated `!com_dedicated`). Placeheld so `Engine` names it.
///
/// Source: `oracle/codemp/client/snd_dma.cpp:127-268`
pub struct SoundSystem {
    //TODO: Port SoundSystem fields (channels, dma, listener, knownSfx)
    // Source: oracle/codemp/client/snd_dma.cpp:127-268
    _private: (),
}
