//! `cl_input.cpp` — client input sampling, key/button state, and command build.
//!
//! Source: `oracle/codemp/client/cl_input.cpp`

#![allow(non_snake_case, non_upper_case_globals)]

use core::ffi::{c_char, c_int, CStr};

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::shared_buffer::autoMapInput_t;
use mp_abi::ui::exports::MpUiExport;
use mp_abi::ui::public::ui_menu_command_t::UIMENU_VOICECHAT;
use mp_engine_qcommon::cmd::cmd_function_t::CmdFunction;
use mp_engine_qcommon::cmd_common::Cmd_Argv;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{Com_HashKey, Com_Memset};
use mp_engine_qcommon::cvar_fns::{Cvar_Get, Cvar_Set, Cvar_VariableIntegerValue};
use mp_engine_qcommon::msg::{
    MSG_Bitstream, MSG_Init, MSG_WriteByte, MSG_WriteDeltaUsercmdKey, MSG_WriteLong,
    MSG_WriteString,
};
use mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e;
use mp_engine_qcommon::qcommon::joystick_axis_t::joystickAxis_t;
use mp_engine_qcommon::qcommon::net_limits::{
    MAX_MSGLEN, MAX_PACKET_USERCMDS, MAX_RELIABLE_COMMANDS, PACKET_MASK,
};
use mp_engine_qcommon::qcommon::netchan_t::netchan_t;
use mp_engine_qcommon::sys_net::Sys_IsLANAddress;
use mp_engine_qcommon::vm_fns::VM_Call;
use mp_game::prelude::byte;
use mp_game::q_math::{PITCH, ROLL, YAW};
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t::NA_LOOPBACK;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::common::mp::qcommon::usercmd_button::{
    BUTTON_ANY, BUTTON_FORCEPOWER, BUTTON_TALK, BUTTON_USE, BUTTON_USE_HOLDABLE, BUTTON_WALKING,
};
use mp_qshared::shared::connstate::connstate_t::{CA_ACTIVE, CA_CINEMATIC, CA_CONNECTED, CA_PRIMED};
use mp_qshared::shared::error_parm::errorParm_t::ERR_DROP;
use mp_qshared::shared::force_powers::{
    FP_ABSORB, FP_DRAIN, FP_GRIP, FP_HEAL, FP_LIGHTNING, FP_PROTECT, FP_PULL, FP_PUSH, FP_RAGE,
    FP_SEE, FP_SPEED, FP_TEAM_FORCE, FP_TEAM_HEAL, FP_TELEPATHY,
};
use mp_qshared::shared::gen_cmds::genCmds_t::{
    GENCMD_BOW, GENCMD_ENGAGE_DUEL, GENCMD_FLOURISH, GENCMD_FORCE_ABSORB,
    GENCMD_FORCE_DISTRACT, GENCMD_FORCE_FORCEPOWEROTHER, GENCMD_FORCE_HEAL,
    GENCMD_FORCE_HEALOTHER, GENCMD_FORCE_PROTECT, GENCMD_FORCE_PULL, GENCMD_FORCE_RAGE,
    GENCMD_FORCE_SEEING, GENCMD_FORCE_SPEED, GENCMD_FORCE_THROW, GENCMD_GLOAT,
    GENCMD_MEDITATE, GENCMD_SABERATTACKCYCLE, GENCMD_SABERSWITCH, GENCMD_TAUNT,
    GENCMD_USE_AMMODISP, GENCMD_USE_BACTA, GENCMD_USE_BACTABIG, GENCMD_USE_CLOAK,
    GENCMD_USE_ELECTROBINOCULARS, GENCMD_USE_EWEB, GENCMD_USE_FIELD, GENCMD_USE_HEALTHDISP,
    GENCMD_USE_JETPACK, GENCMD_USE_SEEKER, GENCMD_USE_SENTRY, GENCMD_ZOOM,
};
use mp_qshared::shared::keycatch::{KEYCATCH_CGAME, KEYCATCH_UI};
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_math::qmath::{AngleNormalize180, AngleNormalize360, AngleSubtract, ClampChar, Q_rsqrt};
use native_math::vector::vec3_t;
use native_string::atoi::atoi;
use native_string::cstr::latin1_to_string;

use crate::cl_net_chan::{CL_Netchan_Transmit, CL_Netchan_TransmitNextFragment};
use crate::cl_scrn::SCR_DebugGraph;
use crate::client::kbutton_t::kbutton_t;
use crate::client_host::Client;

// `SHORT2ANGLE`/`ANGLE2SHORT`/`SQRTFAST` are Raven function-like macros with no rosetta row.
// `SHORT2ANGLE(x)` expands to `(x) * (360.0 / 65536)`; `ANGLE2SHORT(x)` to
// `(int)((x) * 65536 / 360) & 65535`; `SQRTFAST(x)` to `1.0f / Q_rsqrt(x)`.
const SHORT2ANGLE_SCALE: f32 = 360.0 / 65536.0;
const ANGLE2SHORT_SCALE: f32 = 65536.0 / 360.0;

fn short2angle(x: c_int) -> f32 {
    (x as f32) * SHORT2ANGLE_SCALE
}

fn angle2short(x: f32) -> c_int {
    ((x * ANGLE2SHORT_SCALE) as c_int) & 65535
}

/// Raven `CMD_MASK` — the `cl.cmds[CMD_BACKUP]` ring mask.
///
/// Source: `oracle/codemp/cgame/cg_public.h:6-7`
const CMD_MASK: c_int = 63;

/// Raven `OVERRIDE_MOUSE_SENSITIVITY` — the vehicle-mode turn rate that
/// replaces the sensitivity cvars. `VEH_CONTROL_SCHEME_4` is never defined in
/// the MP tree, so the `#else` value stands.
///
/// Source: `oracle/codemp/client/cl_input.cpp:20-24`
const OVERRIDE_MOUSE_SENSITIVITY: f32 = 10.0;

fn sqrtfast(x: f32) -> f32 {
    1.0 / Q_rsqrt(x)
}

/// Raven `IN_MLookDown`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:177-179`
pub fn IN_MLookDown(cl: &mut Client) {
    cl.in_mlooking = qtrue;
}

/// Raven `IN_MLookUp`.
///
/// The packet's printed signature omits `common`, but the `cl_freelook` read
/// needs it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:181-186`
pub fn IN_MLookUp(common: &mut Common, cl: &mut Client) {
    cl.in_mlooking = qfalse;
    if common.cvar(cl.cl_freelook).integer == 0 {
        IN_CenterView(cl);
    }
}

/// Raven `IN_GenCMD1`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:188-192`
pub fn IN_GenCMD1(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_SABERSWITCH as u8;
}

/// Raven `IN_GenCMD2`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:194-198`
pub fn IN_GenCMD2(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_ENGAGE_DUEL as u8;
}

/// Raven `IN_GenCMD3`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:200-204`
pub fn IN_GenCMD3(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_HEAL as u8;
}

/// Raven `IN_GenCMD4`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:206-210`
pub fn IN_GenCMD4(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_SPEED as u8;
}

/// Raven `IN_GenCMD5`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:212-216`
pub fn IN_GenCMD5(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_PULL as u8;
}

/// Raven `IN_GenCMD6`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:218-222`
pub fn IN_GenCMD6(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_DISTRACT as u8;
}

/// Raven `IN_GenCMD7`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:224-228`
pub fn IN_GenCMD7(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_RAGE as u8;
}

/// Raven `IN_GenCMD8`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:230-234`
pub fn IN_GenCMD8(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_PROTECT as u8;
}

/// Raven `IN_GenCMD9`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:236-240`
pub fn IN_GenCMD9(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_ABSORB as u8;
}

/// Raven `IN_GenCMD10`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:242-246`
pub fn IN_GenCMD10(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_HEALOTHER as u8;
}

/// Raven `IN_GenCMD11`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:248-252`
pub fn IN_GenCMD11(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_FORCEPOWEROTHER as u8;
}

/// Raven `IN_GenCMD12`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:254-258`
pub fn IN_GenCMD12(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_SEEING as u8;
}

/// Raven `IN_GenCMD13`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:260-264`
pub fn IN_GenCMD13(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_SEEKER as u8;
}

/// Raven `IN_GenCMD14`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:266-270`
pub fn IN_GenCMD14(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_FIELD as u8;
}

/// Raven `IN_GenCMD15`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:272-276`
pub fn IN_GenCMD15(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_BACTA as u8;
}

/// Raven `IN_GenCMD16`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:278-282`
pub fn IN_GenCMD16(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_ELECTROBINOCULARS as u8;
}

/// Raven `IN_GenCMD17`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:284-288`
pub fn IN_GenCMD17(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_ZOOM as u8;
}

/// Raven `IN_GenCMD18`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:290-294`
pub fn IN_GenCMD18(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_SENTRY as u8;
}

/// Raven `IN_GenCMD19`.
///
/// The packet's printed signature omits `common`, but `Com_Printf` and
/// `Cvar_VariableIntegerValue` both need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:296-312`
pub fn IN_GenCMD19(common: &mut Common, cl: &mut Client) {
    // The oracle's `_XBOX` arm never compiles on this target, so it is dropped (rule 20).
    if Cvar_VariableIntegerValue(common, "d_saberStanceDebug") != 0 {
        com_printf(common, "SABERSTANCEDEBUG: Gencmd on client set successfully.\n");
    }
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_SABERATTACKCYCLE as u8;
}

/// Raven `IN_GenCMD20`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:314-318`
pub fn IN_GenCMD20(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FORCE_THROW as u8;
}

/// Raven `IN_GenCMD21`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:320-324`
pub fn IN_GenCMD21(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_JETPACK as u8;
}

/// Raven `IN_GenCMD22`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:326-330`
pub fn IN_GenCMD22(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_BACTABIG as u8;
}

/// Raven `IN_GenCMD23`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:332-336`
pub fn IN_GenCMD23(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_HEALTHDISP as u8;
}

/// Raven `IN_GenCMD24`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:338-342`
pub fn IN_GenCMD24(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_AMMODISP as u8;
}

/// Raven `IN_GenCMD25`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:344-348`
pub fn IN_GenCMD25(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_EWEB as u8;
}

/// Raven `IN_GenCMD26`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:350-354`
pub fn IN_GenCMD26(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_USE_CLOAK as u8;
}

/// Raven `IN_GenCMD27`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:356-360`
pub fn IN_GenCMD27(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_TAUNT as u8;
}

/// Raven `IN_GenCMD28`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:362-366`
pub fn IN_GenCMD28(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_BOW as u8;
}

/// Raven `IN_GenCMD29`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:368-372`
pub fn IN_GenCMD29(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_MEDITATE as u8;
}

/// Raven `IN_GenCMD30`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:374-378`
pub fn IN_GenCMD30(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_FLOURISH as u8;
}

/// Raven `IN_GenCMD31`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:380-384`
pub fn IN_GenCMD31(cl: &mut Client) {
    cl.cl.gcmdSendValue = qtrue;
    cl.cl.gcmdValue = GENCMD_GLOAT as u8;
}

/// Raven `IN_AutoMapButton`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:389-392`
pub fn IN_AutoMapButton(cl: &mut Client) {
    cl.g_clAutoMapMode = !cl.g_clAutoMapMode;
}

/// Raven `IN_AutoMapToggle`.
///
/// The packet's printed signature carries no receivers, but `Cvar_Set` needs
/// `&mut EngineHostView` and `Cvar_VariableIntegerValue` needs `&Common`
/// (reached through `view.common`), so this adds `view` (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:396-422`
pub fn IN_AutoMapToggle(view: &mut EngineHostView) {
    if Cvar_VariableIntegerValue(view.common, "cg_drawRadar") != 0 {
        Cvar_Set(view, "cg_drawRadar", "0");
    } else {
        Cvar_Set(view, "cg_drawRadar", "1");
    }
    // The commented-out `r_autoMap` arm never compiles; Raven left it dead in source.
}

/// Raven `IN_VoiceChatButton`.
///
/// The packet's printed signature omits `common`, but `VM_Call` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:424-431`
pub fn IN_VoiceChatButton(common: &mut Common, cl: &mut Client) {
    if cl.uivm.is_null() {
        // The ui module is not loaded, so this command does nothing.
        return;
    }
    VM_Call(
        common,
        cl.uivm,
        MpUiExport::UI_SET_ACTIVE_MENU as c_int,
        &[UIMENU_VOICECHAT as isize],
    );
}

/// Raven `IN_KeyDown`.
///
/// The packet's printed signature omits `common`, but `Cmd_Argv` and
/// `Com_Printf` both need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:433-467`
pub fn IN_KeyDown(common: &mut Common, b: *mut kbutton_t) {
    let c = Cmd_Argv(common, 1);
    let k = if !c.is_empty() { atoi(c) } else { -1 };

    unsafe {
        if k == (*b).down[0] || k == (*b).down[1] {
            // repeating key
            return;
        }

        if (*b).down[0] == 0 {
            (*b).down[0] = k;
        } else if (*b).down[1] == 0 {
            (*b).down[1] = k;
        } else {
            com_printf(common, "Three keys down for a button!\n");
            return;
        }

        if (*b).active != 0 {
            // still down
            return;
        }

        // save timestamp for partial frame summing
        let c2 = Cmd_Argv(common, 2);
        (*b).downtime = atoi(c2) as u32;

        (*b).active = qtrue;
        (*b).wasPressed = qtrue;
    }
}

/// Raven `IN_KeyUp`.
///
/// The packet's printed signature omits `common`, but `Cmd_Argv` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:469-507`
pub fn IN_KeyUp(common: &mut Common, cl: &mut Client, b: *mut kbutton_t) {
    let c = Cmd_Argv(common, 1);
    unsafe {
        let k = if !c.is_empty() {
            atoi(c)
        } else {
            // typed manually at the console, assume for unsticking, so clear all
            (*b).down[0] = 0;
            (*b).down[1] = 0;
            (*b).active = qfalse;
            return;
        };

        if (*b).down[0] == k {
            (*b).down[0] = 0;
        } else if (*b).down[1] == k {
            (*b).down[1] = 0;
        } else {
            // key up without corresponding down (menu pass through)
            return;
        }
        if (*b).down[0] != 0 || (*b).down[1] != 0 {
            // some other key is still holding it down
            return;
        }

        (*b).active = qfalse;

        // save timestamp for partial frame summing
        let c2 = Cmd_Argv(common, 2);
        let uptime = atoi(c2);
        if uptime != 0 {
            // Raven computes `uptime - b->downtime` in unsigned, so the wrapping
            // subtraction keeps the C result for a stale downtime.
            (*b).msec = (*b).msec.wrapping_add((uptime as u32).wrapping_sub((*b).downtime));
        } else {
            (*b).msec += cl.frame_msec / 2;
        }

        (*b).active = qfalse;
    }
}

/// Raven `CL_KeyState`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:518-550`
pub fn CL_KeyState(common: &mut Common, cl: &mut Client, key: *mut kbutton_t) -> f32 {
    unsafe {
        let mut msec = (*key).msec;
        (*key).msec = 0;

        if (*key).active != 0 {
            // still down
            if (*key).downtime == 0 {
                msec = common.com_frameTime as u32;
            } else {
                // Raven computes `com_frameTime - key->downtime` in unsigned.
                msec = msec.wrapping_add(
                    (common.com_frameTime as u32).wrapping_sub((*key).downtime),
                );
            }
            (*key).downtime = common.com_frameTime as u32;
        }

        let mut val = msec as f32 / cl.frame_msec as f32;
        if val < 0.0 {
            val = 0.0;
        }
        if val > 1.0 {
            val = 1.0;
        }

        val
    }
}

/// Raven `CL_AutoMapKey`.
///
/// The packet's printed signature omits `common`, but `VM_Call` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:561-643`
pub fn CL_AutoMapKey(common: &mut Common, cl: &mut Client, autoMapKey: c_int, up: qboolean) {
    let data = cl.cl.mSharedMemory as *mut autoMapInput_t;

    match autoMapKey {
        x if x == AUTOMAP_KEY_FORWARD => {
            cl.g_clAutoMapInput.up = if up != 0 { 0.0 } else { 16.0 };
        }
        x if x == AUTOMAP_KEY_BACK => {
            cl.g_clAutoMapInput.down = if up != 0 { 0.0 } else { 16.0 };
        }
        x if x == AUTOMAP_KEY_YAWLEFT => {
            cl.g_clAutoMapInput.yaw = if up != 0 { 0.0 } else { -4.0 };
        }
        x if x == AUTOMAP_KEY_YAWRIGHT => {
            cl.g_clAutoMapInput.yaw = if up != 0 { 0.0 } else { 4.0 };
        }
        x if x == AUTOMAP_KEY_PITCHUP => {
            cl.g_clAutoMapInput.pitch = if up != 0 { 0.0 } else { -4.0 };
        }
        x if x == AUTOMAP_KEY_PITCHDOWN => {
            cl.g_clAutoMapInput.pitch = if up != 0 { 0.0 } else { 4.0 };
        }
        x if x == AUTOMAP_KEY_DEFAULTVIEW => {
            // Raven's `memset(&g_clAutoMapInput, 0, sizeof(autoMapInput_t))`.
            cl.g_clAutoMapInput = autoMapInput_t {
                up: 0.0,
                down: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                goToDefaults: qfalse,
            };
            cl.g_clAutoMapInput.goToDefaults = qtrue;
        }
        _ => {}
    }

    unsafe {
        core::ptr::copy_nonoverlapping(
            &cl.g_clAutoMapInput as *const autoMapInput_t,
            data,
            1,
        );
    }

    if !cl.cgvm.is_null() {
        VM_Call(common, cl.cgvm, MpCgameExport::CG_AUTOMAP_INPUT as c_int, &[0]);
    }

    cl.g_clAutoMapInput.goToDefaults = qfalse;
}

/// Raven `AUTOMAP_KEY_*` — the automap key codes that `CL_AutoMapKey` switches on.
/// The numbering starts at 1, so a zero-filled slot never selects a key.
///
/// Source: `oracle/codemp/client/cl_input.cpp:552-558`
const AUTOMAP_KEY_FORWARD: c_int = 1;
const AUTOMAP_KEY_BACK: c_int = 2;
const AUTOMAP_KEY_YAWLEFT: c_int = 3;
const AUTOMAP_KEY_YAWRIGHT: c_int = 4;
const AUTOMAP_KEY_PITCHUP: c_int = 5;
const AUTOMAP_KEY_PITCHDOWN: c_int = 6;
const AUTOMAP_KEY_DEFAULTVIEW: c_int = 7;

/// Raven `IN_CenterView`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:859-861`
pub fn IN_CenterView(cl: &mut Client) {
    cl.cl.viewangles[PITCH as usize] = -short2angle(cl.cl.snap.ps.delta_angles[PITCH as usize]);
}

/// Raven `CL_MouseEvent`.
///
/// The packet's printed signature omits `common`, but `VM_Call` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:992-1013`
pub fn CL_MouseEvent(common: &mut Common, cl: &mut Client, dx: c_int, dy: c_int, _time: c_int) {
    if cl.g_clAutoMapMode && !cl.cgvm.is_null() {
        let data = cl.cl.mSharedMemory as *mut autoMapInput_t;

        cl.g_clAutoMapInput.yaw = dx as f32;
        cl.g_clAutoMapInput.pitch = dy as f32;
        unsafe {
            core::ptr::copy_nonoverlapping(
                &cl.g_clAutoMapInput as *const autoMapInput_t,
                data,
                1,
            );
        }
        VM_Call(common, cl.cgvm, MpCgameExport::CG_AUTOMAP_INPUT as c_int, &[1]);

        cl.g_clAutoMapInput.yaw = 0.0;
        cl.g_clAutoMapInput.pitch = 0.0;
    } else if cl.cls.keyCatchers & KEYCATCH_UI != 0 {
        VM_Call(
            common,
            cl.uivm,
            MpUiExport::UI_MOUSE_EVENT as c_int,
            &[dx as isize, dy as isize],
        );
    } else if cl.cls.keyCatchers & KEYCATCH_CGAME != 0 {
        VM_Call(
            common,
            cl.cgvm,
            MpCgameExport::CG_MOUSE_EVENT as c_int,
            &[dx as isize, dy as isize],
        );
    } else {
        let idx = cl.cl.mouseIndex as usize;
        cl.cl.mouseDx[idx] += dx;
        cl.cl.mouseDy[idx] += dy;
    }
}

/// Raven `CL_JoystickEvent`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1022-1027`
pub fn CL_JoystickEvent(cl: &mut Client, axis: c_int, value: c_int, _time: c_int) {
    if axis < 0 || axis >= joystickAxis_t::MAX_JOYSTICK_AXIS as c_int {
        com_error(ERR_DROP, format!("CL_JoystickEvent: bad axis {}", axis));
    }
    cl.cl.joystickAxis[axis as usize] = value;
}

/// Raven `CL_JoystickMove`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1035-1106`
pub fn CL_JoystickMove(common: &mut Common, cl: &mut Client, cmd: *mut usercmd_t) {
    unsafe {
        if common.cvar(cl.in_joystick).integer == 0 {
            return;
        }

        let movespeed;
        let anglespeed;

        if cl.in_speed.active ^ common.cvar(cl.cl_run).integer != 0 {
            movespeed = 2;
        } else {
            movespeed = 1;
            (*cmd).buttons |= BUTTON_WALKING;
        }

        if cl.in_speed.active != 0 {
            anglespeed = 0.001 * cl.cls.frametime as f32 * common.cvar(cl.cl_anglespeedkey).value;
        } else {
            anglespeed = 0.001 * cl.cls.frametime as f32;
        }

        if cl.in_strafe.active == 0 {
            if cl.cl_mYawOverride != 0.0 {
                if cl.cl_mSensitivityOverride != 0.0 {
                    cl.cl.viewangles[YAW as usize] += cl.cl_mYawOverride
                        * cl.cl_mSensitivityOverride
                        * cl.cl.joystickAxis[joystickAxis_t::AXIS_SIDE as usize] as f32
                        / 2.0;
                } else {
                    cl.cl.viewangles[YAW as usize] += cl.cl_mYawOverride
                        * OVERRIDE_MOUSE_SENSITIVITY
                        * cl.cl.joystickAxis[joystickAxis_t::AXIS_SIDE as usize] as f32
                        / 2.0;
                }
            } else {
                cl.cl.viewangles[YAW as usize] += anglespeed
                    * (common.cvar(cl.cl_yawspeed).value / 100.0)
                    * cl.cl.joystickAxis[joystickAxis_t::AXIS_SIDE as usize] as f32;
            }
        } else {
            (*cmd).rightmove = ClampChar(
                (*cmd).rightmove as c_int + cl.cl.joystickAxis[joystickAxis_t::AXIS_SIDE as usize],
            );
        }

        if cl.in_mlooking != 0 || common.cvar(cl.cl_freelook).integer != 0 {
            if cl.cl_mPitchOverride != 0.0 {
                if cl.cl_mSensitivityOverride != 0.0 {
                    cl.cl.viewangles[PITCH as usize] += cl.cl_mPitchOverride
                        * cl.cl_mSensitivityOverride
                        * cl.cl.joystickAxis[joystickAxis_t::AXIS_FORWARD as usize] as f32
                        / 2.0;
                } else {
                    cl.cl.viewangles[PITCH as usize] += cl.cl_mPitchOverride
                        * OVERRIDE_MOUSE_SENSITIVITY
                        * cl.cl.joystickAxis[joystickAxis_t::AXIS_FORWARD as usize] as f32
                        / 2.0;
                }
            } else {
                cl.cl.viewangles[PITCH as usize] += anglespeed
                    * (common.cvar(cl.cl_pitchspeed).value / 100.0)
                    * cl.cl.joystickAxis[joystickAxis_t::AXIS_FORWARD as usize] as f32;
            }
        } else {
            (*cmd).forwardmove = ClampChar(
                (*cmd).forwardmove as c_int
                    + cl.cl.joystickAxis[joystickAxis_t::AXIS_FORWARD as usize],
            );
        }

        (*cmd).upmove = ClampChar(
            (*cmd).upmove as c_int + cl.cl.joystickAxis[joystickAxis_t::AXIS_UP as usize],
        );
    }
}

/// Raven `CL_MouseMove`.
///
/// The packet's printed signature omits `common`, but `Com_Printf` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1132-1289`
pub fn CL_MouseMove(common: &mut Common, cl: &mut Client, cmd: *mut usercmd_t) {
    let speed = cl.frame_msec as f32;
    unsafe {
        let pitch = if cl.cl_bUseFighterPitch != 0 {
            common.cvar(cl.m_pitchVeh).value
        } else {
            common.cvar(cl.m_pitch).value
        };

        let (mut mx, mut my): (f32, f32);
        // The oracle's `_XBOX` arm never compiles on this target, so it is dropped (rule 20).
        if common.cvar(cl.m_filter).integer != 0 {
            mx = (cl.cl.mouseDx[0] + cl.cl.mouseDx[1]) as f32 * 0.5;
            my = (cl.cl.mouseDy[0] + cl.cl.mouseDy[1]) as f32 * 0.5;
        } else {
            mx = cl.cl.mouseDx[cl.cl.mouseIndex as usize] as f32;
            my = cl.cl.mouseDy[cl.cl.mouseIndex as usize] as f32;
        }

        cl.cl.mouseIndex ^= 1;
        cl.cl.mouseDx[cl.cl.mouseIndex as usize] = 0;
        cl.cl.mouseDy[cl.cl.mouseIndex as usize] = 0;

        let rate = sqrtfast(mx * mx + my * my) / speed;
        let mut accelSensitivity;
        if cl.cl_mYawOverride != 0.0 || cl.cl_mPitchOverride != 0.0 {
            if cl.cl_mSensitivityOverride != 0.0 {
                accelSensitivity = cl.cl_mSensitivityOverride;
            } else {
                accelSensitivity = common.cvar(cl.cl_sensitivity).value
                    + rate * common.cvar(cl.cl_mouseAccel).value;
                accelSensitivity *= cl.cl.cgameSensitivity;
            }
        } else {
            accelSensitivity =
                common.cvar(cl.cl_sensitivity).value + rate * common.cvar(cl.cl_mouseAccel).value;
            accelSensitivity *= cl.cl.cgameSensitivity;
        }

        if rate != 0.0 && common.cvar(cl.cl_showMouseRate).integer != 0 {
            com_printf(common, &format!("{} : {}\n", rate, accelSensitivity));
        }

        mx *= accelSensitivity;
        my *= accelSensitivity;

        if mx == 0.0 && my == 0.0 {
            return;
        }

        // add mouse X/Y movement to cmd
        if cl.in_strafe.active != 0 {
            (*cmd).rightmove =
                ClampChar((*cmd).rightmove as c_int + (common.cvar(cl.m_side).value * mx) as c_int);
        } else if cl.cl_mYawOverride != 0.0 {
            cl.cl.viewangles[YAW as usize] -= cl.cl_mYawOverride * mx;
        } else {
            cl.cl.viewangles[YAW as usize] -= common.cvar(cl.m_yaw).value * mx;
        }

        if (cl.in_mlooking != 0 || common.cvar(cl.cl_freelook).integer != 0)
            && cl.in_strafe.active == 0
        {
            let cl_pitchSensitivity: f32 = 1.0;
            if cl.cl_mPitchOverride != 0.0 {
                if pitch > 0.0 {
                    cl.cl.viewangles[PITCH as usize] +=
                        cl.cl_mPitchOverride * my * cl_pitchSensitivity;
                } else {
                    cl.cl.viewangles[PITCH as usize] -=
                        cl.cl_mPitchOverride * my * cl_pitchSensitivity;
                }
            } else {
                cl.cl.viewangles[PITCH as usize] += pitch * my * cl_pitchSensitivity;
            }
        } else {
            (*cmd).forwardmove = ClampChar(
                (*cmd).forwardmove as c_int - (common.cvar(cl.m_forward).value * my) as c_int,
            );
        }
    }
}

/// Raven `CL_NoUseableForce`.
///
/// The packet's printed signature omits `common`, but `VM_Call` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1291-1299`
pub fn CL_NoUseableForce(common: &mut Common, cl: &mut Client) -> qboolean {
    if cl.cgvm.is_null() {
        // no cgame loaded
        return qfalse;
    }
    VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_GET_USEABLE_FORCE as c_int,
        &[],
    ) as qboolean
}

/// Raven `CL_FinishMove`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1349-1432`
pub fn CL_FinishMove(cl: &mut Client, cmd: *mut usercmd_t) {
    unsafe {
        // copy the state that the cgame is currently sending
        (*cmd).weapon = cl.cl.cgameUserCmdValue as u8;
        (*cmd).forcesel = cl.cl.cgameForceSelection as u8;
        (*cmd).invensel = cl.cl.cgameInvenSelection as u8;

        if cl.cl.gcmdSendValue != 0 {
            (*cmd).generic_cmd = cl.cl.gcmdValue;
            cl.cl.gcmdSentValue = qtrue;
        } else {
            (*cmd).generic_cmd = 0;
        }

        // send the current server time so the amount of movement
        // can be determined without allowing cheating
        (*cmd).serverTime = cl.cl.serverTime;

        if cl.cl.cgameViewAngleForceTime > cl.cl.serverTime {
            cl.cl.cgameViewAngleForce[YAW as usize] -=
                short2angle(cl.cl.snap.ps.delta_angles[YAW as usize]);

            cl.cl.viewangles[YAW as usize] = cl.cl.cgameViewAngleForce[YAW as usize];
            cl.cl.cgameViewAngleForceTime = 0;
        }

        if cl.cl_crazyShipControls != 0 {
            let yawDelta = AngleSubtract(
                cl.cl.viewangles[YAW as usize],
                cl.cl_lastViewAngles[YAW as usize],
            );
            cl.cl_sendAngles[ROLL as usize] -= yawDelta;

            let mut nRoll = cl.cl_sendAngles[ROLL as usize].abs();

            let pitchDelta = AngleSubtract(
                cl.cl.viewangles[PITCH as usize],
                cl.cl_lastViewAngles[PITCH as usize],
            );
            let mut pitchSubtract = pitchDelta * (nRoll / 90.0);
            cl.cl_sendAngles[PITCH as usize] += pitchDelta - pitchSubtract;

            //yaw-roll calc should be different
            if nRoll > 90.0 {
                nRoll -= 180.0;
            }
            if nRoll < 0.0 {
                nRoll = -nRoll;
            }
            pitchSubtract = pitchDelta * (nRoll / 90.0);
            if cl.cl_sendAngles[ROLL as usize] > 0.0 {
                cl.cl_sendAngles[YAW as usize] += pitchSubtract;
            } else {
                cl.cl_sendAngles[YAW as usize] -= pitchSubtract;
            }

            cl.cl_sendAngles[PITCH as usize] = AngleNormalize180(cl.cl_sendAngles[PITCH as usize]);
            cl.cl_sendAngles[YAW as usize] = AngleNormalize360(cl.cl_sendAngles[YAW as usize]);
            cl.cl_sendAngles[ROLL as usize] = AngleNormalize180(cl.cl_sendAngles[ROLL as usize]);

            for i in 0..3 {
                (*cmd).angles[i] = angle2short(cl.cl_sendAngles[i]);
            }
        } else {
            for i in 0..3 {
                (*cmd).angles[i] = angle2short(cl.cl.viewangles[i]);
            }
            //in case we switch to the cl_crazyShipControls
            cl.cl_sendAngles = cl.cl.viewangles;
        }
        //always needed in for the cl_crazyShipControls
        cl.cl_lastViewAngles = cl.cl.viewangles;
    }
}

/// Raven `CL_ReadyToSendPacket`.
///
/// The packet's printed signature carries only `cl`, but `Cvar_Set` needs
/// `&mut EngineHostView`, so this adds `view` (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1529-1585`
pub fn CL_ReadyToSendPacket(view: &mut EngineHostView, cl: &mut Client) -> qboolean {
    // don't send anything if playing back a demo
    if cl.clc.demoplaying != 0 || cl.cls.state == CA_CINEMATIC {
        return qfalse;
    }

    // If we are downloading, we send no less than 50ms between packets
    if cl.clc.downloadTempName[0] != 0 && cl.cls.realtime - cl.clc.lastPacketSentTime < 50 {
        return qfalse;
    }

    // if we don't have a valid gamestate yet, only send one packet a second
    if cl.cls.state != CA_ACTIVE
        && cl.cls.state != CA_PRIMED
        && cl.clc.downloadTempName[0] == 0
        && cl.cls.realtime - cl.clc.lastPacketSentTime < 1000
    {
        return qfalse;
    }

    // send every frame for loopbacks
    if cl.clc.netchan.remoteAddress.r#type == NA_LOOPBACK {
        return qtrue;
    }

    // send every frame for LAN
    if Sys_IsLANAddress(&cl.clc.netchan.remoteAddress) {
        return qtrue;
    }

    // check for exceeding cl_maxpackets
    if view.common.cvar(cl.cl_maxpackets).integer < 15 {
        Cvar_Set(view, "cl_maxpackets", "15");
    } else if view.common.cvar(cl.cl_maxpackets).integer > 100 {
        Cvar_Set(view, "cl_maxpackets", "100");
    }

    let oldPacketNum = ((cl.clc.netchan.outgoingSequence - 1) as usize) & PACKET_MASK;
    let delta = cl.cls.realtime - cl.cl.outPackets[oldPacketNum].p_realtime;
    if delta < 1000 / view.common.cvar(cl.cl_maxpackets).integer {
        // the accumulated commands will go out in the next packet
        return qfalse;
    }

    qtrue
}

/// Raven `CL_AdjustAngles`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:890-938`
pub fn CL_AdjustAngles(common: &mut Common, cl: &mut Client) {
    // The key-hold pointers come out first, so `CL_KeyState` and the `viewangles`
    // writes do not borrow `cl` at the same time. Raven calls right before left
    // and lookup before lookdown, and that order stands.
    let in_right = &mut cl.in_right as *mut kbutton_t;
    let in_left = &mut cl.in_left as *mut kbutton_t;
    let in_lookup = &mut cl.in_lookup as *mut kbutton_t;
    let in_lookdown = &mut cl.in_lookdown as *mut kbutton_t;

    let speed = if cl.in_speed.active != 0 {
        0.001 * cl.cls.frametime as f32 * common.cvar(cl.cl_anglespeedkey).value
    } else {
        0.001 * cl.cls.frametime as f32
    };

    if cl.in_strafe.active == 0 {
        let yawspeed = common.cvar(cl.cl_yawspeed).value;
        let ksRight = CL_KeyState(common, cl, in_right);
        let ksLeft = CL_KeyState(common, cl, in_left);
        if cl.cl_mYawOverride != 0.0 {
            let sensitivity = if cl.cl_mSensitivityOverride != 0.0 {
                cl.cl_mSensitivityOverride
            } else {
                OVERRIDE_MOUSE_SENSITIVITY
            };
            cl.cl.viewangles[YAW as usize] -=
                cl.cl_mYawOverride * sensitivity * speed * yawspeed * ksRight;
            cl.cl.viewangles[YAW as usize] +=
                cl.cl_mYawOverride * sensitivity * speed * yawspeed * ksLeft;
        } else {
            cl.cl.viewangles[YAW as usize] -= speed * yawspeed * ksRight;
            cl.cl.viewangles[YAW as usize] += speed * yawspeed * ksLeft;
        }
    }

    let pitchspeed = common.cvar(cl.cl_pitchspeed).value;
    let ksLookup = CL_KeyState(common, cl, in_lookup);
    let ksLookdown = CL_KeyState(common, cl, in_lookdown);
    if cl.cl_mPitchOverride != 0.0 {
        let sensitivity = if cl.cl_mSensitivityOverride != 0.0 {
            cl.cl_mSensitivityOverride
        } else {
            OVERRIDE_MOUSE_SENSITIVITY
        };
        cl.cl.viewangles[PITCH as usize] -=
            cl.cl_mPitchOverride * sensitivity * speed * pitchspeed * ksLookup;
        cl.cl.viewangles[PITCH as usize] +=
            cl.cl_mPitchOverride * sensitivity * speed * pitchspeed * ksLookdown;
    } else {
        cl.cl.viewangles[PITCH as usize] -= speed * pitchspeed * ksLookup;
        cl.cl.viewangles[PITCH as usize] += speed * pitchspeed * ksLookdown;
    }
}

/// Raven `CL_KeyMove`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:947-985`
pub fn CL_KeyMove(common: &mut Common, cl: &mut Client, cmd: *mut usercmd_t) {
    // The key-hold pointers come out first, so `CL_KeyState` and the `cl` reads
    // do not borrow `cl` at the same time.
    let in_right = &mut cl.in_right as *mut kbutton_t;
    let in_left = &mut cl.in_left as *mut kbutton_t;
    let in_moveright = &mut cl.in_moveright as *mut kbutton_t;
    let in_moveleft = &mut cl.in_moveleft as *mut kbutton_t;
    let in_up = &mut cl.in_up as *mut kbutton_t;
    let in_down = &mut cl.in_down as *mut kbutton_t;
    let in_forward = &mut cl.in_forward as *mut kbutton_t;
    let in_back = &mut cl.in_back as *mut kbutton_t;

    unsafe {
        let movespeed: c_int;
        if cl.in_speed.active ^ common.cvar(cl.cl_run).integer != 0 {
            movespeed = 127;
            (*cmd).buttons &= !BUTTON_WALKING;
        } else {
            (*cmd).buttons |= BUTTON_WALKING;
            movespeed = 46;
        }

        // Raven accumulates an `int` from a float product, so each step
        // truncates the running sum, the same as C's `int += float`.
        let mut forward: c_int = 0;
        let mut side: c_int = 0;
        let mut up: c_int = 0;
        if cl.in_strafe.active != 0 {
            side = (side as f32 + movespeed as f32 * CL_KeyState(common, cl, in_right)) as c_int;
            side = (side as f32 - movespeed as f32 * CL_KeyState(common, cl, in_left)) as c_int;
        }

        side = (side as f32 + movespeed as f32 * CL_KeyState(common, cl, in_moveright)) as c_int;
        side = (side as f32 - movespeed as f32 * CL_KeyState(common, cl, in_moveleft)) as c_int;

        up = (up as f32 + movespeed as f32 * CL_KeyState(common, cl, in_up)) as c_int;
        up = (up as f32 - movespeed as f32 * CL_KeyState(common, cl, in_down)) as c_int;

        forward =
            (forward as f32 + movespeed as f32 * CL_KeyState(common, cl, in_forward)) as c_int;
        forward = (forward as f32 - movespeed as f32 * CL_KeyState(common, cl, in_back)) as c_int;

        (*cmd).forwardmove = ClampChar(forward);
        (*cmd).rightmove = ClampChar(side);
        (*cmd).upmove = ClampChar(up);
    }
}

/// Raven `CL_CmdButtons`.
///
/// The packet's printed signature omits `common`, but `CL_NoUseableForce`
/// needs it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1306-1339`
pub fn CL_CmdButtons(common: &mut Common, cl: &mut Client, cmd: *mut usercmd_t) {
    unsafe {
        // figure button bits, sending a button bit even if the key was pressed and
        // released in less than a frame
        for i in 0..15usize {
            if cl.in_buttons[i].active != 0 || cl.in_buttons[i].wasPressed != 0 {
                (*cmd).buttons |= 1 << i;
            }
            cl.in_buttons[i].wasPressed = qfalse;
        }

        if (*cmd).buttons & BUTTON_FORCEPOWER != 0 {
            // check for transferring a use force to a use inventory
            if (*cmd).buttons & BUTTON_USE != 0 || CL_NoUseableForce(common, cl) != 0 {
                (*cmd).buttons &= !BUTTON_FORCEPOWER;
                (*cmd).buttons |= BUTTON_USE_HOLDABLE;
            }
        }

        if cl.cls.keyCatchers != 0 {
            (*cmd).buttons |= BUTTON_TALK;
        }

        // allow the game to know if any key at all is currently pressed, even if it
        // isn't bound to anything
        if cl.kg.anykeydown != 0 && cl.cls.keyCatchers == 0 {
            (*cmd).buttons |= BUTTON_ANY;
        }
    }
}

/// Raven `IN_UseGivenForce`.
///
/// The packet's printed signature omits `common`, but `Cmd_Argv` (and the
/// button handlers it calls) need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:108-175`
pub fn IN_UseGivenForce(common: &mut Common, cl: &mut Client) {
    let c = Cmd_Argv(common, 1);
    let forceNum = if !c.is_empty() { atoi(c) } else { return };
    let mut genCmdNum = 0;

    match forceNum {
        x if x == FP_DRAIN as i32 => {
            IN_Button11Down(common, cl);
            IN_Button11Up(common, cl);
        }
        x if x == FP_PUSH as i32 => genCmdNum = GENCMD_FORCE_THROW as i32,
        x if x == FP_SPEED as i32 => genCmdNum = GENCMD_FORCE_SPEED as i32,
        x if x == FP_PULL as i32 => genCmdNum = GENCMD_FORCE_PULL as i32,
        x if x == FP_TELEPATHY as i32 => genCmdNum = GENCMD_FORCE_DISTRACT as i32,
        x if x == FP_GRIP as i32 => {
            IN_Button6Down(common, cl);
            IN_Button6Up(common, cl);
        }
        x if x == FP_LIGHTNING as i32 => {
            IN_Button10Down(common, cl);
            IN_Button10Up(common, cl);
        }
        x if x == FP_RAGE as i32 => genCmdNum = GENCMD_FORCE_RAGE as i32,
        x if x == FP_PROTECT as i32 => genCmdNum = GENCMD_FORCE_PROTECT as i32,
        x if x == FP_ABSORB as i32 => genCmdNum = GENCMD_FORCE_ABSORB as i32,
        x if x == FP_SEE as i32 => genCmdNum = GENCMD_FORCE_SEEING as i32,
        x if x == FP_HEAL as i32 => genCmdNum = GENCMD_FORCE_HEAL as i32,
        x if x == FP_TEAM_HEAL as i32 => genCmdNum = GENCMD_FORCE_HEALOTHER as i32,
        x if x == FP_TEAM_FORCE as i32 => genCmdNum = GENCMD_FORCE_FORCEPOWEROTHER as i32,
        _ => debug_assert!(false, "IN_UseGivenForce: unhandled forceNum"),
    }

    if genCmdNum != 0 {
        cl.cl.gcmdSendValue = qtrue;
        cl.cl.gcmdValue = genCmdNum as u8;
    }
}

/// Raven `CL_CreateCmd`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1439-1482`
pub fn CL_CreateCmd(common: &mut Common, cl: &mut Client) -> usercmd_t {
    let mut cmd: usercmd_t = unsafe { core::mem::zeroed() };
    let oldAngles: vec3_t = cl.cl.viewangles;

    // keyboard angle adjustment
    CL_AdjustAngles(common, cl);

    // get basic movement from keyboard
    CL_CmdButtons(common, cl, &mut cmd as *mut usercmd_t);

    // get basic movement from keyboard
    CL_KeyMove(common, cl, &mut cmd as *mut usercmd_t);

    // get basic movement from mouse
    CL_MouseMove(common, cl, &mut cmd as *mut usercmd_t);

    // get basic movement from joystick
    CL_JoystickMove(common, cl, &mut cmd as *mut usercmd_t);

    // check to make sure the angles haven't wrapped
    if cl.cl.viewangles[PITCH as usize] - oldAngles[PITCH as usize] > 90.0 {
        cl.cl.viewangles[PITCH as usize] = oldAngles[PITCH as usize] + 90.0;
    } else if oldAngles[PITCH as usize] - cl.cl.viewangles[PITCH as usize] > 90.0 {
        cl.cl.viewangles[PITCH as usize] = oldAngles[PITCH as usize] - 90.0;
    }

    // store out the final values
    CL_FinishMove(cl, &mut cmd as *mut usercmd_t);

    // draw debug graphs of turning for mouse testing
    if common.cvar(cl.cl_debugMove).integer == 1 {
        let value = (cl.cl.viewangles[YAW as usize] - oldAngles[YAW as usize]).abs();
        SCR_DebugGraph(cl, value, 0);
    }
    if common.cvar(cl.cl_debugMove).integer == 2 {
        let value = (cl.cl.viewangles[PITCH as usize] - oldAngles[PITCH as usize]).abs();
        SCR_DebugGraph(cl, value, 0);
    }

    cmd
}

/// Raven `CL_WritePacket`.
///
/// The packet's printed signature omits its receiver, but `MSG_Init` and
/// `Cvar_Set` need `&mut EngineHostView`, so this takes `view` and reaches
/// `Common` through `view.common` (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1608-1729`
pub fn CL_WritePacket(view: &mut EngineHostView, cl: &mut Client) {
    // don't send anything if playing back a demo
    if cl.clc.demoplaying != 0 || cl.cls.state == CA_CINEMATIC {
        return;
    }

    let mut nullcmd: usercmd_t = unsafe { core::mem::zeroed() };
    Com_Memset(
        &mut nullcmd as *mut usercmd_t as *mut (),
        0,
        core::mem::size_of::<usercmd_t>(),
    );
    let mut oldcmd: *mut usercmd_t = &mut nullcmd as *mut usercmd_t;

    let mut data = [0u8; MAX_MSGLEN];
    let mut buf: msg_t = unsafe { core::mem::zeroed() };
    MSG_Init(
        view,
        &mut buf as *mut msg_t,
        data.as_mut_ptr() as *mut byte,
        MAX_MSGLEN as c_int,
    );

    MSG_Bitstream(&mut buf as *mut msg_t);
    // write the current serverId so the server can tell if this is from the current gameState
    MSG_WriteLong(view.common, &mut buf as *mut msg_t, cl.cl.serverId);

    // write the last message we received, which can be used for delta compression, and
    // is also used to tell if we dropped a gamestate
    MSG_WriteLong(
        view.common,
        &mut buf as *mut msg_t,
        cl.clc.serverMessageSequence,
    );

    // write the last reliable message we received
    MSG_WriteLong(
        view.common,
        &mut buf as *mut msg_t,
        cl.clc.serverCommandSequence,
    );

    // write any unacknowledged clientCommands
    for i in (cl.clc.reliableAcknowledge + 1)..=cl.clc.reliableSequence {
        MSG_WriteByte(
            view.common,
            &mut buf as *mut msg_t,
            clc_ops_e::clc_clientCommand as c_int,
        );
        MSG_WriteLong(view.common, &mut buf as *mut msg_t, i);
        let idx = (i as usize) & (MAX_RELIABLE_COMMANDS - 1);
        // `reliableCommands` holds raw wire bytes, so the Latin-1 decode hands
        // `MSG_WriteString` every byte back verbatim.
        let command = latin1_to_string(
            unsafe { CStr::from_ptr(cl.clc.reliableCommands[idx].as_ptr()) }.to_bytes(),
        );
        MSG_WriteString(view.common, &mut buf as *mut msg_t, &command);
    }

    // we want to send all the usercmds that were generated in the last few packets, so
    // even if a couple packets are dropped in a row, all the cmds will make it to the server
    if view.common.cvar(cl.cl_packetdup).integer < 0 {
        Cvar_Set(view, "cl_packetdup", "0");
    } else if view.common.cvar(cl.cl_packetdup).integer > 5 {
        Cvar_Set(view, "cl_packetdup", "5");
    }

    let packetdup = view.common.cvar(cl.cl_packetdup).integer;
    let oldPacketNum =
        ((cl.clc.netchan.outgoingSequence - 1 - packetdup) as usize) & PACKET_MASK;
    let mut count = cl.cl.cmdNumber - cl.cl.outPackets[oldPacketNum].p_cmdNumber;
    if count > MAX_PACKET_USERCMDS as c_int {
        count = MAX_PACKET_USERCMDS as c_int;
        com_printf(view.common, "MAX_PACKET_USERCMDS\n");
    }
    if count >= 1 {
        if view.common.cvar(cl.cl_showSend).integer != 0 {
            com_printf(view.common, &format!("({})", count));
        }

        // begin a client move command
        if view.common.cvar(cl.cl_nodelta).integer != 0
            || cl.cl.snap.valid == 0
            || cl.clc.demowaiting != 0
            || cl.clc.serverMessageSequence != cl.cl.snap.messageNum
        {
            MSG_WriteByte(
                view.common,
                &mut buf as *mut msg_t,
                clc_ops_e::clc_moveNoDelta as c_int,
            );
        } else {
            MSG_WriteByte(
                view.common,
                &mut buf as *mut msg_t,
                clc_ops_e::clc_move as c_int,
            );
        }

        // write the command count
        MSG_WriteByte(view.common, &mut buf as *mut msg_t, count);

        // use the checksum feed in the key
        let mut key = cl.clc.checksumFeed;
        // also use the message acknowledge
        key ^= cl.clc.serverMessageSequence;
        // also use the last acknowledged server command in the key
        let scIdx = (cl.clc.serverCommandSequence as usize) & (MAX_RELIABLE_COMMANDS - 1);
        key ^= Com_HashKey(cl.clc.serverCommands[scIdx].as_ptr() as *mut c_char, 32);

        // write all the commands, including the predicted command
        for i in 0..count {
            let j = ((cl.cl.cmdNumber - count + i + 1) & CMD_MASK) as usize;
            let cmd = &mut cl.cl.cmds[j] as *mut usercmd_t;
            MSG_WriteDeltaUsercmdKey(view.common, &mut buf as *mut msg_t, key, oldcmd, cmd);
            oldcmd = cmd;
        }

        if cl.cl.gcmdSentValue != 0 {
            // clear here, hoping it resolves gencmd values sometimes not going through
            cl.cl.gcmdSendValue = qfalse;
            cl.cl.gcmdSentValue = qfalse;
            cl.cl.gcmdValue = 0;
        }
    }

    // deliver the message
    let packetNum = (cl.clc.netchan.outgoingSequence as usize) & PACKET_MASK;
    cl.cl.outPackets[packetNum].p_realtime = cl.cls.realtime;
    unsafe {
        cl.cl.outPackets[packetNum].p_serverTime = (*oldcmd).serverTime;
    }
    cl.cl.outPackets[packetNum].p_cmdNumber = cl.cl.cmdNumber;
    cl.clc.lastPacketSentTime = cl.cls.realtime;

    if view.common.cvar(cl.cl_showSend).integer != 0 {
        com_printf(view.common, &format!("{} ", buf.cursize));
    }

    let chan = &mut cl.clc.netchan as *mut netchan_t;
    CL_Netchan_Transmit(view, cl, chan, &mut buf as *mut msg_t);

    // clients never really should have messages large enough to fragment, but in case
    // they do, fire them all off at once
    while cl.clc.netchan.unsentFragments != 0 {
        CL_Netchan_TransmitNextFragment(view, chan);
    }
}

/// Raven `CL_CreateNewCommands`.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1492-1516`
pub fn CL_CreateNewCommands(common: &mut Common, cl: &mut Client) {
    // no need to create usercmds until we have a gamestate
    if (cl.cls.state as c_int) < CA_PRIMED as c_int {
        return;
    }

    cl.frame_msec = (common.com_frameTime - cl.old_com_frameTime) as u32;

    // if running less than 5fps, truncate the extra time to prevent unexpected moves
    // after a hitch
    if cl.frame_msec > 200 {
        cl.frame_msec = 200;
    }
    cl.old_com_frameTime = common.com_frameTime;

    // generate a command for this frame
    cl.cl.cmdNumber += 1;
    let cmdNum = (cl.cl.cmdNumber & CMD_MASK) as usize;
    cl.cl.cmds[cmdNum] = CL_CreateCmd(common, cl);
}

/// Raven `CL_SendCmd`.
///
/// The packet's printed signature omits its receiver, but `CL_ReadyToSendPacket`
/// needs `&mut EngineHostView`, so this takes `view` (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:1738-1761`
pub fn CL_SendCmd(view: &mut EngineHostView, cl: &mut Client) {
    // don't send any message if not connected
    if (cl.cls.state as c_int) < CA_CONNECTED as c_int {
        return;
    }

    // don't send commands if paused
    if view.common.cvar(view.common.com_sv_running).integer != 0
        && view.common.cvar(view.common.sv_paused).integer != 0
        && view.common.cvar(view.common.cl_paused).integer != 0
    {
        return;
    }

    // we create commands even if a demo is playing
    CL_CreateNewCommands(view.common, cl);

    // don't send a packet if the last packet was sent too recently
    if CL_ReadyToSendPacket(view, cl) == 0 {
        if view.common.cvar(cl.cl_showSend).integer != 0 {
            com_printf(view.common, ". ");
        }
        return;
    }

    CL_WritePacket(view, cl);
}

/// Raven `IN_UpDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:646-656`
pub fn IN_UpDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_PITCHUP, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_up as *mut kbutton_t);
    }
}

/// Raven `IN_UpUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:657-667`
pub fn IN_UpUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_PITCHUP, qtrue);
    } else {
        let b = &mut cl.in_up as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_DownDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:668-678`
pub fn IN_DownDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_PITCHDOWN, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_down as *mut kbutton_t);
    }
}

/// Raven `IN_DownUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:679-689`
pub fn IN_DownUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_PITCHDOWN, qtrue);
    } else {
        let b = &mut cl.in_down as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_LeftDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:690`
pub fn IN_LeftDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_left as *mut kbutton_t);
}

/// Raven `IN_LeftUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:691`
pub fn IN_LeftUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_left as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_RightDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:692`
pub fn IN_RightDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_right as *mut kbutton_t);
}

/// Raven `IN_RightUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:693`
pub fn IN_RightUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_right as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_ForwardDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:694-704`
pub fn IN_ForwardDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_FORWARD, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_forward as *mut kbutton_t);
    }
}

/// Raven `IN_ForwardUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:705-715`
pub fn IN_ForwardUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_FORWARD, qtrue);
    } else {
        let b = &mut cl.in_forward as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_BackDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:716-726`
pub fn IN_BackDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_BACK, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_back as *mut kbutton_t);
    }
}

/// Raven `IN_BackUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:727-737`
pub fn IN_BackUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_BACK, qtrue);
    } else {
        let b = &mut cl.in_back as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_LookupDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:738`
pub fn IN_LookupDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_lookup as *mut kbutton_t);
}

/// Raven `IN_LookupUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:739`
pub fn IN_LookupUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_lookup as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_LookdownDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:740`
pub fn IN_LookdownDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_lookdown as *mut kbutton_t);
}

/// Raven `IN_LookdownUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:741`
pub fn IN_LookdownUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_lookdown as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_MoveleftDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:742-752`
pub fn IN_MoveleftDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_YAWLEFT, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_moveleft as *mut kbutton_t);
    }
}

/// Raven `IN_MoveleftUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:753-763`
pub fn IN_MoveleftUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_YAWLEFT, qtrue);
    } else {
        let b = &mut cl.in_moveleft as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_MoverightDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:764-774`
pub fn IN_MoverightDown(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_YAWRIGHT, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_moveright as *mut kbutton_t);
    }
}

/// Raven `IN_MoverightUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:775-785`
pub fn IN_MoverightUp(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_YAWRIGHT, qtrue);
    } else {
        let b = &mut cl.in_moveright as *mut kbutton_t;
        IN_KeyUp(common, cl, b);
    }
}

/// Raven `IN_SpeedDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:794`
pub fn IN_SpeedDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_speed as *mut kbutton_t);
}

/// Raven `IN_SpeedUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:795`
pub fn IN_SpeedUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_speed as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_StrafeDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:796`
pub fn IN_StrafeDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_strafe as *mut kbutton_t);
}

/// Raven `IN_StrafeUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:797`
pub fn IN_StrafeUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_strafe as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button0Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:799`
pub fn IN_Button0Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[0] as *mut kbutton_t);
}

/// Raven `IN_Button0Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:800-806`
pub fn IN_Button0Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[0] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
    // The oracle's `_XBOX` arm (`sLastFireTime = Sys_Milliseconds()`) never compiles on
    // this target, so it is dropped (rule 20).
}

/// Raven `IN_Button1Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:807`
pub fn IN_Button1Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[1] as *mut kbutton_t);
}

/// Raven `IN_Button1Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:808`
pub fn IN_Button1Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[1] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button2Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:809`
pub fn IN_Button2Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[2] as *mut kbutton_t);
}

/// Raven `IN_Button2Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:810`
pub fn IN_Button2Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[2] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button3Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:811`
pub fn IN_Button3Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[3] as *mut kbutton_t);
}

/// Raven `IN_Button3Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:812`
pub fn IN_Button3Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[3] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button4Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:813`
pub fn IN_Button4Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[4] as *mut kbutton_t);
}

/// Raven `IN_Button4Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:814`
pub fn IN_Button4Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[4] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button5Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown`/`CL_AutoMapKey`
/// need it, so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:815-825`
pub fn IN_Button5Down(common: &mut Common, cl: &mut Client) {
    if cl.g_clAutoMapMode {
        CL_AutoMapKey(common, cl, AUTOMAP_KEY_DEFAULTVIEW, qfalse);
    } else {
        IN_KeyDown(common, &mut cl.in_buttons[5] as *mut kbutton_t);
    }
}

/// Raven `IN_Button5Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:826`
pub fn IN_Button5Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[5] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button6Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:827`
pub fn IN_Button6Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[6] as *mut kbutton_t);
}

/// Raven `IN_Button6Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:828`
pub fn IN_Button6Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[6] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button7Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:829`
pub fn IN_Button7Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[7] as *mut kbutton_t);
}

/// Raven `IN_Button7Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:830-836`
pub fn IN_Button7Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[7] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
    // The oracle's `_XBOX` arm (`sLastFireTime = Sys_Milliseconds()`) never compiles on
    // this target, so it is dropped (rule 20).
}

/// Raven `IN_Button8Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:837`
pub fn IN_Button8Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[8] as *mut kbutton_t);
}

/// Raven `IN_Button8Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:838`
pub fn IN_Button8Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[8] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button9Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:839`
pub fn IN_Button9Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[9] as *mut kbutton_t);
}

/// Raven `IN_Button9Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:840`
pub fn IN_Button9Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[9] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button10Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:841`
pub fn IN_Button10Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[10] as *mut kbutton_t);
}

/// Raven `IN_Button10Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:842`
pub fn IN_Button10Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[10] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button11Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:843`
pub fn IN_Button11Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[11] as *mut kbutton_t);
}

/// Raven `IN_Button11Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:844`
pub fn IN_Button11Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[11] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button12Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:845`
pub fn IN_Button12Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[12] as *mut kbutton_t);
}

/// Raven `IN_Button12Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:846`
pub fn IN_Button12Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[12] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button13Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:847`
pub fn IN_Button13Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[13] as *mut kbutton_t);
}

/// Raven `IN_Button13Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:848`
pub fn IN_Button13Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[13] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button14Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:849`
pub fn IN_Button14Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[14] as *mut kbutton_t);
}

/// Raven `IN_Button14Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:850`
pub fn IN_Button14Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[14] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_Button15Down`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:851`
pub fn IN_Button15Down(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[15] as *mut kbutton_t);
}

/// Raven `IN_Button15Up`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:852`
pub fn IN_Button15Up(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[15] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `IN_ButtonDown`.
///
/// The packet's printed signature omits `common`, but `IN_KeyDown` needs it,
/// so this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:854-855`
pub fn IN_ButtonDown(common: &mut Common, cl: &mut Client) {
    IN_KeyDown(common, &mut cl.in_buttons[1] as *mut kbutton_t);
}

/// Raven `IN_ButtonUp`.
///
/// The packet's printed signature omits `common`, but `IN_KeyUp` needs it, so
/// this adds it (shape_mismatch).
///
/// Source: `oracle/codemp/client/cl_input.cpp:856-857`
pub fn IN_ButtonUp(common: &mut Common, cl: &mut Client) {
    let b = &mut cl.in_buttons[1] as *mut kbutton_t;
    IN_KeyUp(common, cl, b);
}

/// Raven `CL_InitInput`.
///
/// The packet's printed signature carries only `cl`, but `Cmd_AddCommand`/
/// `Cvar_Get` both need `&mut EngineHostView`, so this adds `view`
/// (shape_mismatch). The registered command handlers no longer match
/// `CmdFunction`'s no-receiver shape once threaded state lands on them
/// (shape_mismatch); the referee wires the adapter table at integration.
///
/// Source: `oracle/codemp/client/cl_input.cpp:1769-1897`
pub fn CL_InitInput(view: &mut EngineHostView, cl: &mut Client) {
    let _ = cl;
    //TODO: Port CL_InitInput Cmd_AddCommand table
    // Source: oracle/codemp/client/cl_input.cpp:1769-1893
    // Every `+`/`-` handler in this file takes `common, cl` or `view, cl`, and
    // `CmdFunction` is `fn(&mut EngineHostView)`. The `*_cmd` adapters that bridge
    // the two are not written yet, so the table stays unregistered.
    let _: Option<CmdFunction> = None;

    cl.cl_nodelta = Some(Cvar_Get(view, "cl_nodelta", "0", 0));
    cl.cl_debugMove = Some(Cvar_Get(view, "cl_debugMove", "0", 0));
}
