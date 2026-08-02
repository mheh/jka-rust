//! `cl_cgame.cpp` — the cgame VM host: system-call trap dispatch, snapshot and
//! usercmd handoff, and cgame lifecycle (init, shutdown, render, time).
//!
//! Source: `oracle/codemp/client/cl_cgame.cpp`

#![allow(non_snake_case, non_camel_case_types, unused_variables, unused_mut)]

use core::ffi::{c_char, c_int};

use mp_bg::public::configstring::{CS_G2BONES, CS_PLAYERS, CS_SERVERINFO, CS_SYSTEMINFO};
use mp_bg::public::entity_flags::EF_PERMANENT;
use mp_engine_core::lifecycle::sys_milliseconds;
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::cgame::stereo_frame_t::stereoFrame_t;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::shared_ragdoll_params::sharedRagDollParams_t;
use mp_qshared::common::mp::qcommon::shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::add_electricity_arg::addElectricityArgStruct_t;
use mp_qshared::shared::addbezier_arg::addbezierArgStruct_t;
use mp_qshared::shared::addpoly_arg::addpolyArgStruct_t;
use mp_qshared::shared::addsprite_arg::addspriteArgStruct_t;
use mp_qshared::shared::collision::CollisionRecord_t;
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::cvar::vmCvar_t;
use mp_qshared::shared::effect_trail_arg::effectTrailArgStruct_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::file_mode::fsMode_t;
use mp_qshared::shared::game_state::{gameState_t, MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
use mp_qshared::shared::limits::{BIG_INFO_STRING, MAX_GENTITIES};
use mp_qshared::shared::q_string::Com_sprintf;
use mp_qshared::shared::shared_ik_move_params::sharedIKMoveParams_t;
use native_string::atoi::atoi;

use crate::client::cl_main_consts::MAX_STRINGED_SV_STRING;
use crate::client::client_consts::{CMD_BACKUP, CMD_MASK, MAX_PARSE_ENTITIES, RESET_TIME};
use mp_qshared::shared::keycatch::KEYCATCH_CGAME;
use native_math::qmath::vec3_origin;

// PORT-NOTE(cross-shard): `Con_Close`, `Con_ClearNotify`, `CL_ReadDemoMessage`,
// `CL_FirstSnapshot`, and `CL_SystemInfoChanged` are in-engine callees this
// porter's packets name by packet file, not by Rust path (they land in a
// different oracle-file module: cl_console.cpp / cl_parse.cpp). Called by
// their Raven names below; wire the imports at integration.

use native_math::eorientations::Eorientations;
use native_math::orientation::orientation_t;
use native_math::vector::{vec3_t, vec_t};
use native_types::qboolean;

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::imports::MpCgameImport;
use mp_abi::cgame::public::snapshot_t::{snapshot_t, MAX_ENTITIES_IN_SNAPSHOT};
use mp_abi::ui::exports::MpUiExport;

use mp_abi::cgame::syscalls::CG_CM_MARKFRAGMENTS::markFragment_t;
use mp_engine_qcommon::cm_load::{
    CM_InlineModel, CM_LoadMap, CM_LoadSubBSP, CM_NumInlineModels, CM_TempBoxModel,
};
use mp_engine_qcommon::cm_test::{CM_PointContents, CM_TransformedPointContents};
use mp_engine_qcommon::cm_trace::{CM_BoxTrace, CM_TransformedBoxTrace};
use mp_engine_qcommon::cmd_common::{
    Cbuf_AddText, Cmd_Argc, Cmd_ArgsBuffer, Cmd_ArgsFrom, Cmd_Argv, Cmd_TokenizeString,
};
use mp_engine_qcommon::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{
    Com_DPrintf, Com_Memcpy, Com_Memset, Com_RealTime, Q_acos, Q_asin,
};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Register, Cvar_Set, Cvar_Update, Cvar_VariableStringBuffer, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_Write};
use mp_engine_qcommon::files_pc::{FS_FOpenFileByMode, FS_GetFileList, FS_Read2};
use mp_engine_qcommon::qcommon::net_limits::{MAX_RELIABLE_COMMANDS, PACKET_BACKUP, PACKET_MASK};
use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t;
use mp_engine_qcommon::qcommon::vm_interpret_t::vmInterpret_t;
use mp_engine_qcommon::stringed::api::SE_GetString;
use mp_engine_qcommon::timing::timing_c::timing_c;
use mp_engine_qcommon::vm_fns::{
    VM_ArgPtr, VM_Call, VM_Create, VM_Debug, VM_Free, VM_Shifted_Alloc, VM_Shifted_Free,
};
use mp_engine_qcommon::z_memman_pc::{Com_TouchMemory, Hunk_MemoryRemaining};
use native_types::mdxaBone_t;
use native_types::qhandle_t;

use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::gore::crag_doll_params::CRagDollParams;
use mp_engine_ghoul2::gore::sskin_gore_data::SSkinGoreData;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;

use mp_engine_rmg::rm_manager::RmManager;
use mp_renderer::tr_model::render_models::RenderModels;

use native_string::info::Info_ValueForKey;
use native_string::q_string::Q_strcat;
use native_string::q_strncpyz::Q_strncpyz;

use crate::client::cl_snapshot_t::clSnapshot_t;
use crate::client_host::Client;

// PORT-NOTE(rosetta-gap): `byte` did not carry a rosetta path this porter could
// resolve without an upward `mp_game` dependency from an engine-tier crate
// (layering, docs/workspace-architecture.md). It is unused in the ported
// bodies below (the one `byte *` case is memcpy-shaped), so it is omitted.

// PORT-NOTE(rosetta-gap): `ERagEffector`/`ERagPhase` are declared in
// `sp_qshared`, an SP-tier crate this MP-tier module does not depend on.
// The CRagDollParams fields that use them are set by numeric cast, matching
// Raven's own `(CRagDollParams::ERagPhase)` cast.

/// Raven `CL_GetGameState`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:78-80`
pub fn CL_GetGameState(cl: &mut Client, gs: *mut gameState_t) {
    unsafe {
        *gs = cl.cl.gameState;
    }
}

/// Raven `CL_GetGlconfig`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:87-89`
pub fn CL_GetGlconfig(cl: &mut Client, glconfig: *mut glconfig_t) {
    unsafe {
        *glconfig = cl.cls.glconfig;
    }
}

/// Raven `CL_GetUserCmd`.
/// The usercmd wrapping buffer holds only `CMD_BACKUP` entries, so a request
/// older than that window returns false instead of stale data.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:97-114`
pub fn CL_GetUserCmd(cl: &mut Client, cmdNumber: c_int, ucmd: *mut usercmd_t) -> qboolean {
    // cmds[cmdNumber] is the last properly generated command.
    if cmdNumber > cl.cl.cmdNumber {
        com_error(
            errorParm_t::ERR_DROP,
            format!("CL_GetUserCmd: {} >= {}", cmdNumber, cl.cl.cmdNumber),
        );
    }

    // The usercmd has been overwritten in the wrapping buffer because it is too far out of date.
    //TODO: Port CMD_BACKUP
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h
    if cmdNumber <= cl.cl.cmdNumber - CMD_BACKUP {
        return qboolean::qfalse;
    }

    unsafe {
        //TODO: Port CMD_MASK
        *ucmd = cl.cl.cmds[(cmdNumber & CMD_MASK) as usize];
    }

    qboolean::qtrue
}

/// Raven `CL_GetCurrentCmdNumber`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:116-118`
pub fn CL_GetCurrentCmdNumber(cl: &mut Client) -> c_int {
    cl.cl.cmdNumber
}

/// Raven `CL_GetParseEntityState`.
/// The parse-entities ring holds only `MAX_PARSE_ENTITIES` entries, so a
/// request that fell out of the ring returns false instead of stale data.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:126-140`
pub fn CL_GetParseEntityState(
    cl: &mut Client,
    parseEntityNumber: c_int,
    state: *mut entityState_t,
) -> qboolean {
    // Can't return anything that hasn't been parsed yet.
    if parseEntityNumber >= cl.cl.parseEntitiesNum {
        com_error(
            errorParm_t::ERR_DROP,
            format!(
                "CL_GetParseEntityState: {} >= {}",
                parseEntityNumber, cl.cl.parseEntitiesNum
            ),
        );
    }

    // Can't return anything that has been overwritten in the circular buffer.
    //TODO: Port MAX_PARSE_ENTITIES
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h
    if parseEntityNumber <= cl.cl.parseEntitiesNum - MAX_PARSE_ENTITIES {
        return qboolean::qfalse;
    }

    unsafe {
        *state = cl.cl.parseEntities[(parseEntityNumber & (MAX_PARSE_ENTITIES - 1)) as usize];
    }
    qboolean::qtrue
}

/// Raven `CL_GetCurrentSnapshotNumber`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:147-150`
pub fn CL_GetCurrentSnapshotNumber(
    cl: &mut Client,
    snapshotNumber: *mut c_int,
    serverTime: *mut c_int,
) {
    unsafe {
        *snapshotNumber = cl.cl.snap.messageNum;
        *serverTime = cl.cl.snap.serverTime;
    }
}

/// Raven `CL_GetSnapshot`.
/// Refuses a request that fell out of the packet ring or whose entities fell
/// out of the parse-entities ring, rather than returning stale data.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:157-208`
pub fn CL_GetSnapshot(
    cl: &mut Client,
    snapshotNumber: c_int,
    snapshot: *mut snapshot_t,
) -> qboolean {
    if snapshotNumber > cl.cl.snap.messageNum {
        com_error(
            errorParm_t::ERR_DROP,
            "CL_GetSnapshot: snapshotNumber > cl.snapshot.messageNum".to_string(),
        );
    }

    // If the frame has fallen out of the circular buffer, we can't return it.
    if cl.cl.snap.messageNum - snapshotNumber >= PACKET_BACKUP {
        return qboolean::qfalse;
    }

    // If the frame is not valid, we can't return it.
    let clSnap: &clSnapshot_t = &cl.cl.snapshots[(snapshotNumber & PACKET_MASK) as usize];
    if clSnap.valid == qboolean::qfalse {
        return qboolean::qfalse;
    }

    // If the entities in the frame have fallen out of their circular buffer, we can't return it.
    //TODO: Port MAX_PARSE_ENTITIES
    if cl.cl.parseEntitiesNum - clSnap.parseEntitiesNum >= MAX_PARSE_ENTITIES {
        return qboolean::qfalse;
    }

    unsafe {
        // Write the snapshot.
        (*snapshot).snapFlags = clSnap.snapFlags;
        (*snapshot).serverCommandSequence = clSnap.serverCommandNum;
        (*snapshot).ping = clSnap.ping;
        (*snapshot).serverTime = clSnap.serverTime;
        Com_Memcpy(
            (*snapshot).areamask.as_mut_ptr() as *mut (),
            clSnap.areamask.as_ptr() as *const (),
            core::mem::size_of_val(&(*snapshot).areamask),
        );
        (*snapshot).ps = clSnap.ps;
        (*snapshot).vps = clSnap.vps; // get the vehicle ps
        let mut count = clSnap.numEntities;
        if count > MAX_ENTITIES_IN_SNAPSHOT as c_int {
            Com_DPrintf(
                common,
                &format!(
                    "CL_GetSnapshot: truncated {} entities to {}\n",
                    count, MAX_ENTITIES_IN_SNAPSHOT
                ),
            );
            count = MAX_ENTITIES_IN_SNAPSHOT as c_int;
        }
        (*snapshot).numEntities = count;

        for i in 0..count {
            //TODO: Port MAX_PARSE_ENTITIES
            let entNum = (clSnap.parseEntitiesNum + i) & (MAX_PARSE_ENTITIES - 1);
            // Copy everything but the ghoul2 pointer.
            (*snapshot).entities[i as usize] = cl.cl.parseEntities[entNum as usize];
        }
    }

    // FIXME: configstring changes and server commands!!!

    qboolean::qtrue
}

/// Raven `CL_GetDefaultState`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:210-225`
pub fn CL_GetDefaultState(cl: &mut Client, index: c_int, state: *mut entityState_t) -> qboolean {
    //TODO: Port MAX_GENTITIES
    if index < 0 || index >= MAX_GENTITIES {
        return qboolean::qfalse;
    }

    //TODO: Port EF_PERMANENT
    if cl.cl.entityBaselines[index as usize].eFlags & EF_PERMANENT == 0 {
        return qboolean::qfalse;
    }

    unsafe {
        *state = cl.cl.entityBaselines[index as usize];
    }

    qboolean::qtrue
}

/// Raven `CL_SetUserCmdValue`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:235-243`
pub fn CL_SetUserCmdValue(
    cl: &mut Client,
    userCmdValue: c_int,
    sensitivityScale: f32,
    mPitchOverride: f32,
    mYawOverride: f32,
    mSensitivityOverride: f32,
    fpSel: c_int,
    invenSel: c_int,
) {
    cl.cl.cgameUserCmdValue = userCmdValue;
    cl.cl.cgameSensitivity = sensitivityScale;
    //TODO: Port cl_mPitchOverride
    // Source: oracle/codemp/client/cl_cgame.cpp:232
    cl.cl_mPitchOverride = mPitchOverride;
    //TODO: Port cl_mYawOverride
    // Source: oracle/codemp/client/cl_cgame.cpp:233
    cl.cl_mYawOverride = mYawOverride;
    //TODO: Port cl_mSensitivityOverride
    // Source: oracle/codemp/client/cl_cgame.cpp:234
    cl.cl_mSensitivityOverride = mSensitivityOverride;
    cl.cl.cgameForceSelection = fpSel;
    cl.cl.cgameInvenSelection = invenSel;
}

/// Raven `CL_SetClientForceAngle`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:250-254`
pub fn CL_SetClientForceAngle(cl: &mut Client, time: c_int, angle: vec3_t) {
    cl.cl.cgameViewAngleForceTime = time;
    cl.cl.cgameViewAngleForce = angle;
}

/// Raven `CL_AddCgameCommand`.
/// `Cmd_AddCommand`'s LAW signature needs a `&mut EngineHostView` this
/// packet's resolved signature carries no receiver for.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:261-263`
pub fn CL_AddCgameCommand(cmdName: *const c_char) {
    let cmd_name = unsafe {
        core::ffi::CStr::from_ptr(cmdName)
            .to_string_lossy()
            .into_owned()
    };
    //TODO: Port Cmd_AddCommand receiver
    // Source: crates/mp/engine/qcommon/src/cmd_pc.rs (needs `&mut EngineHostView`)
    Cmd_AddCommand(host, &cmd_name, None);
}

/// Raven `CL_CgameError`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:270-272`
pub fn CL_CgameError(string: *const c_char) {
    let s = unsafe {
        core::ffi::CStr::from_ptr(string)
            .to_string_lossy()
            .into_owned()
    };
    com_error(errorParm_t::ERR_DROP, s);
}

/// Raven `CL_DoAutoLODScale`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:280-291`
pub fn CL_DoAutoLODScale(cl: &mut Client) {
    let mut finalLODScaleFactor: f32 = 0.0;

    //TODO: Port gCLTotalClientNum
    // Source: oracle/codemp/client/cl_cgame.cpp:275
    if cl.gCLTotalClientNum >= 8 {
        finalLODScaleFactor = cl.gCLTotalClientNum as f32 / (-8.0f32 as f64) as f32;
    }

    Cvar_Set(
        cl,
        "r_autolodscalevalue",
        &format!("{}", finalLODScaleFactor),
    );
}

/// Raven `CL_CheckSVStringEdRef`.
/// StringEd references are marked with a leading `@@@` run in the source
/// string; everything after it up to a space/colon/period/newline is the
/// StringEd key to substitute in place.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:387-454`
pub fn CL_CheckSVStringEdRef(buf: *mut c_char, str: *const c_char) {
    unsafe {
        if str.is_null() || *str == 0 {
            if !str.is_null() {
                libc::strcpy(buf, str);
            }
            return;
        }

        libc::strcpy(buf, str);

        let strLen = libc::strlen(str) as isize;

        //TODO: Port MAX_STRINGED_SV_STRING
        if strLen >= MAX_STRINGED_SV_STRING as isize {
            return;
        }

        let mut i: isize = 0;
        let mut b: isize = 0;

        while i < strLen && *str.offset(i) != 0 {
            let mut gotStrip = false;

            if *str.offset(i) == b'@' as c_char && (i + 1) < strLen {
                if *str.offset(i + 1) == b'@' as c_char && (i + 2) < strLen {
                    if *str.offset(i + 2) == b'@' as c_char && (i + 3) < strLen {
                        // @@@ should mean to insert a StringEd reference here, so insert it into buf at the current place.
                        let mut stringRef = [0 as c_char; MAX_STRINGED_SV_STRING];
                        let mut r: usize = 0;

                        while i < strLen && *str.offset(i) == b'@' as c_char {
                            i += 1;
                        }

                        while i < strLen
                            && *str.offset(i) != 0
                            && *str.offset(i) != b' ' as c_char
                            && *str.offset(i) != b':' as c_char
                            && *str.offset(i) != b'.' as c_char
                            && *str.offset(i) != b'\n' as c_char
                        {
                            stringRef[r] = *str.offset(i);
                            r += 1;
                            i += 1;
                        }
                        stringRef[r] = 0;

                        *buf.offset(b) = 0;
                        let string_ref = core::ffi::CStr::from_ptr(stringRef.as_ptr())
                            .to_string_lossy()
                            .into_owned();
                        //TODO: Port MP_SVGAME
                        let replacement = SE_GetString(host, "MP_SVGAME");
                        let buf_slice = core::slice::from_raw_parts_mut(
                            buf as *mut c_char,
                            MAX_STRINGED_SV_STRING,
                        );
                        Q_strcat(buf_slice, MAX_STRINGED_SV_STRING, &replacement);
                        b = libc::strlen(buf) as isize;
                    }
                }
            }

            if !gotStrip {
                *buf.offset(b) = *str.offset(i);
                b += 1;
            }
            i += 1;
        }

        *buf.offset(b) = 0;
    }
}

/// Raven `CL_CM_LoadMap`.
/// `CM_LoadMap`'s LAW signature needs a `&mut EngineHostView` this packet's
/// resolved signature carries no receiver for.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:583-587`
pub fn CL_CM_LoadMap(mapname: *const c_char) {
    let mut checksum: c_int = 0;
    let name = unsafe {
        core::ffi::CStr::from_ptr(mapname)
            .to_string_lossy()
            .into_owned()
    };
    //TODO: Port CM_LoadMap receiver
    // Source: crates/mp/engine/qcommon/src/cm_load.rs (needs `&mut EngineHostView`)
    CM_LoadMap(host, &name, qboolean::qtrue, &mut checksum);
}

/// Raven `CL_ShutdownCGame`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:595-607`
pub fn CL_ShutdownCGame(common: &mut Common, cl: &mut Client) {
    //TODO: Port KEYCATCH_CGAME
    cl.cls.keyCatchers &= !KEYCATCH_CGAME;
    cl.cls.cgameStarted = qboolean::qfalse;
    if cl.cgvm.is_null() {
        return;
    }
    VM_Call(common, cl.cgvm, MpCgameExport::CG_SHUTDOWN as c_int, &[]);
    VM_Free(common, cl.cgvm);
    cl.cgvm = core::ptr::null_mut();
}

/// Raven `FloatAsInt`.
/// Reinterprets the float's bit pattern as an int, the way the VM syscall
/// bridge passes floats through an int argument slot.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:609-615`
fn FloatAsInt(f: f32) -> c_int {
    f.to_bits() as c_int
}

/// Raven `CL_GameCommand`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1815-1821`
pub fn CL_GameCommand(cl: &mut Client) -> qboolean {
    if cl.cgvm.is_null() {
        return qboolean::qfalse;
    }

    unsafe {
        core::mem::transmute(VM_Call(
            common,
            cl.cgvm,
            MpCgameExport::CG_CONSOLE_COMMAND as c_int,
            &[],
        ) as c_int)
    }
}

/// Raven `CL_CGameRendering`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1830-1845`
pub fn CL_CGameRendering(common: &mut Common, cl: &mut Client, stereo: stereoFrame_t) {
    // rww - RAGDOLL_BEGIN
    //TODO: Port com_sv_running
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h:693
    if unsafe { (*common.com_sv_running).integer } == 0 {
        // Set the server time to match the client time, if we don't have a server going.
        G2API_SetTime(cl.cl.serverTime, 0);
    }
    G2API_SetTime(cl.cl.serverTime, 1);
    // rww - RAGDOLL_END

    VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_DRAW_ACTIVE_FRAME as c_int,
        &[
            cl.cl.serverTime as isize,
            stereo as isize,
            cl.clc.demoplaying as isize,
        ],
    );
    VM_Debug(common, 0);
}

/// Raven `CL_AdjustTimeDelta`.
/// Snaps the delta back hard on a big jump, halves it on a moderate one, and
/// nudges it by 1-2 msec on a small one so latency drifts smoothly.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1870-1927`
pub fn CL_AdjustTimeDelta(common: &mut Common, cl: &mut Client) {
    cl.cl.newSnapshots = qboolean::qfalse;

    // The delta never drifts when replaying a demo.
    if cl.clc.demoplaying != qboolean::qfalse {
        return;
    }

    // If the current time is WAY off, just correct to the current value.
    //TODO: Port com_sv_running
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h:693
    //TODO: Port RESET_TIME
    let resetTime = if unsafe { (*common.com_sv_running).integer } != 0 {
        100
    } else {
        RESET_TIME
    };
    let _ = resetTime; // Raven computes this but never reads it back (dead in the oracle too).

    let newDelta = cl.cl.snap.serverTime - cl.cls.realtime;
    let deltaDelta = (newDelta - cl.cl.serverTimeDelta).abs();

    //TODO: Port cl_showTimeDelta
    // Source: oracle/codemp/client/../RMG/../client/client.h:403
    if deltaDelta > RESET_TIME {
        cl.cl.serverTimeDelta = newDelta;
        cl.cl.oldServerTime = cl.cl.snap.serverTime; // FIXME: is this a problem for cgame?
        cl.cl.serverTime = cl.cl.snap.serverTime;
        if unsafe { (*cl.cl_showTimeDelta).integer } != 0 {
            com_printf(common, "<RESET> ");
        }
    } else if deltaDelta > 100 {
        // Fast adjust, cut the difference in half.
        if unsafe { (*cl.cl_showTimeDelta).integer } != 0 {
            com_printf(common, "<FAST> ");
        }
        cl.cl.serverTimeDelta = (cl.cl.serverTimeDelta + newDelta) >> 1;
    } else {
        // Slow drift adjust, only move 1 or 2 msec.
        // If any of the frames between this and the previous snapshot had to be extrapolated,
        // nudge our sense of time back a little. The granularity of +1 / -2 is too high for
        // timescale modified frametimes.
        //TODO: Port com_timescale
        // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h:692
        if unsafe { (*common.com_timescale).value } == 0.0
            || unsafe { (*common.com_timescale).value } == 1.0
        {
            if cl.cl.extrapolatedSnapshot != qboolean::qfalse {
                cl.cl.extrapolatedSnapshot = qboolean::qfalse;
                cl.cl.serverTimeDelta -= 2;
            } else {
                // Otherwise, move our sense of time forward to minimize total latency.
                cl.cl.serverTimeDelta += 1;
            }
        }
    }

    if unsafe { (*cl.cl_showTimeDelta).integer } != 0 {
        com_printf(common, &format!("{} ", cl.cl.serverTimeDelta));
    }
}

/// Raven `CL_ConfigstringModified`.
/// Rebuilds the whole gamestate string table around the one changed index,
/// because Raven repacks `stringData` densely rather than patching in place.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:298-382`
pub fn CL_ConfigstringModified(common: &mut Common, cl: &mut Client) {
    let index = atoi(Cmd_Argv(common, 1));
    //TODO: Port MAX_CONFIGSTRINGS
    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        com_error(
            errorParm_t::ERR_DROP,
            "configstring > MAX_CONFIGSTRINGS".to_string(),
        );
    }
    // Get everything after "cs <num>".
    let s = Cmd_ArgsFrom(common, 2);

    let old = unsafe {
        core::ffi::CStr::from_ptr(
            cl.cl
                .gameState
                .stringData
                .as_ptr()
                .offset(cl.cl.gameState.stringOffsets[index as usize] as isize),
        )
        .to_string_lossy()
        .into_owned()
    };
    if old == s {
        return; // unchanged
    }

    // Build the new gameState_t.
    let oldGs = cl.cl.gameState;

    cl.cl.gameState = gameState_t {
        stringOffsets: [0; MAX_CONFIGSTRINGS],
        stringData: [0; MAX_GAMESTATE_CHARS],
        dataCount: 0,
    };

    // Leave the first 0 for uninitialized strings.
    cl.cl.gameState.dataCount = 1;

    for i in 0..MAX_CONFIGSTRINGS as c_int {
        let dup = if i == index {
            s.clone()
        } else {
            unsafe {
                core::ffi::CStr::from_ptr(
                    oldGs
                        .stringData
                        .as_ptr()
                        .offset(oldGs.stringOffsets[i as usize] as isize),
                )
                .to_string_lossy()
                .into_owned()
            }
        };
        if dup.is_empty() {
            continue; // leave with the default empty string
        }

        let len = dup.len() as c_int;

        if len + 1 + cl.cl.gameState.dataCount > MAX_GAMESTATE_CHARS as c_int {
            com_error(
                errorParm_t::ERR_DROP,
                "MAX_GAMESTATE_CHARS exceeded".to_string(),
            );
        }

        // Append it to the gameState string buffer.
        cl.cl.gameState.stringOffsets[i as usize] = cl.cl.gameState.dataCount;
        unsafe {
            Com_Memcpy(
                cl.cl
                    .gameState
                    .stringData
                    .as_mut_ptr()
                    .offset(cl.cl.gameState.dataCount as isize) as *mut (),
                dup.as_ptr() as *const (),
                len as usize + 1,
            );
        }
        cl.cl.gameState.dataCount += len + 1;
    }

    //TODO: Port cl_autolodscale
    // Source: oracle/codemp/client/cl_cgame.cpp:277
    if !cl.cl_autolodscale.is_null() && unsafe { (*cl.cl_autolodscale).integer } != 0 {
        //TODO: Port CS_PLAYERS
        //TODO: Port CS_G2BONES
        if index >= CS_PLAYERS && index < CS_G2BONES {
            // This means that a client was updated in some way. Go through and count the clients.
            let mut clientCount = 0;
            let mut i = CS_PLAYERS;

            while i < CS_G2BONES {
                let s = unsafe {
                    core::ffi::CStr::from_ptr(
                        cl.cl
                            .gameState
                            .stringData
                            .as_ptr()
                            .offset(cl.cl.gameState.stringOffsets[i as usize] as isize),
                    )
                    .to_string_lossy()
                    .into_owned()
                };

                if !s.is_empty() {
                    clientCount += 1;
                }

                i += 1;
            }

            //TODO: Port gCLTotalClientNum
            // Source: oracle/codemp/client/cl_cgame.cpp:275
            cl.gCLTotalClientNum = clientCount;

            CL_DoAutoLODScale(cl);
        }
    }

    //TODO: Port CS_SYSTEMINFO
    if index == CS_SYSTEMINFO {
        // Parse serverId and other cvars.
        CL_SystemInfoChanged(cl);
    }
}

/// Raven `CL_GetServerCommand`.
/// `bcs0`/`bcs1`/`bcs2` reassemble a big configstring split across several
/// reliable commands into `bigConfigString` before re-tokenizing it as `cs`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:462-573`
pub fn CL_GetServerCommand(
    common: &mut Common,
    cl: &mut Client,
    serverCommandNumber: c_int,
) -> qboolean {
    // The fork-3 rotating-scratch static becomes an owned local; Raven's
    // cross-call persistence (bcs0 -> bcs1 -> bcs2) only spans one dispatch
    // chain, so a local re-created per call is behavior-preserving here.
    //TODO: Port bigConfigString
    // Source: oracle/codemp/client/cl_cgame.cpp:465
    let mut bigConfigString: [c_char; BIG_INFO_STRING] = [0; BIG_INFO_STRING];

    // If we have irretrievably lost a reliable command, drop the connection.
    if serverCommandNumber <= cl.clc.serverCommandSequence - MAX_RELIABLE_COMMANDS as c_int {
        // When a demo record was started after the client got a whole bunch of reliable
        // commands then the client never got those first reliable commands.
        if cl.clc.demoplaying != qboolean::qfalse {
            return qboolean::qfalse;
        }
        let mut i = 0;
        while i < MAX_RELIABLE_COMMANDS {
            // Spew out the reliable command buffer.
            if cl.clc.reliableCommands[i][0] != 0 {
                let cmd = unsafe {
                    core::ffi::CStr::from_ptr(cl.clc.reliableCommands[i].as_ptr())
                        .to_string_lossy()
                        .into_owned()
                };
                com_printf(common, &format!("{}: {}\n", i, cmd));
            }
            i += 1;
        }
        com_error(
            errorParm_t::ERR_DROP,
            "CL_GetServerCommand: a reliable command was cycled out".to_string(),
        );
    }

    if serverCommandNumber > cl.clc.serverCommandSequence {
        com_error(
            errorParm_t::ERR_DROP,
            "CL_GetServerCommand: requested a command not received".to_string(),
        );
    }

    let mut s = unsafe {
        core::ffi::CStr::from_ptr(
            cl.clc.serverCommands
                [(serverCommandNumber & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize]
                .as_ptr(),
        )
        .to_string_lossy()
        .into_owned()
    };
    cl.clc.lastExecutedServerCommand = serverCommandNumber;

    Com_DPrintf(
        common,
        &format!("serverCommand: {} : {}\n", serverCommandNumber, s),
    );

    loop {
        // rescan:
        Cmd_TokenizeString(common, &s);
        let cmd = Cmd_Argv(common, 0).to_string();

        if cmd == "disconnect" {
            let mut strEd: [c_char; MAX_STRINGED_SV_STRING] = [0; MAX_STRINGED_SV_STRING];
            let arg1 = Cmd_Argv(common, 1);
            let arg1_c = std::ffi::CString::new(arg1).unwrap_or_default();
            CL_CheckSVStringEdRef(strEd.as_mut_ptr(), arg1_c.as_ptr());
            let str_ed = unsafe {
                core::ffi::CStr::from_ptr(strEd.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };
            //TODO: Port MP_SVGAME_SERVER_DISCONNECTED
            com_error(
                errorParm_t::ERR_SERVERDISCONNECT,
                format!(
                    "{}: {}\n",
                    SE_GetString(host, "MP_SVGAME_SERVER_DISCONNECTED"),
                    str_ed
                ),
            );
        }

        if cmd == "bcs0" {
            let msg = format!("cs {} \"{}", Cmd_Argv(common, 1), Cmd_Argv(common, 2));
            let msg_c = std::ffi::CString::new(msg).unwrap_or_default();
            unsafe {
                libc::strcpy(bigConfigString.as_mut_ptr(), msg_c.as_ptr());
            }
            return qboolean::qfalse;
        }

        if cmd == "bcs1" {
            let arg = Cmd_Argv(common, 2).to_string();
            let cur_len = unsafe { libc::strlen(bigConfigString.as_ptr()) };
            if cur_len + arg.len() >= BIG_INFO_STRING {
                com_error(
                    errorParm_t::ERR_DROP,
                    "bcs exceeded BIG_INFO_STRING".to_string(),
                );
            }
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();
            unsafe {
                libc::strcat(bigConfigString.as_mut_ptr(), arg_c.as_ptr());
            }
            return qboolean::qfalse;
        }

        if cmd == "bcs2" {
            let arg = Cmd_Argv(common, 2).to_string();
            let cur_len = unsafe { libc::strlen(bigConfigString.as_ptr()) };
            if cur_len + arg.len() + 1 >= BIG_INFO_STRING {
                com_error(
                    errorParm_t::ERR_DROP,
                    "bcs exceeded BIG_INFO_STRING".to_string(),
                );
            }
            let arg_c = std::ffi::CString::new(arg).unwrap_or_default();
            unsafe {
                libc::strcat(bigConfigString.as_mut_ptr(), arg_c.as_ptr());
                libc::strcat(
                    bigConfigString.as_mut_ptr(),
                    b"\"\0".as_ptr() as *const c_char,
                );
            }
            s = unsafe {
                core::ffi::CStr::from_ptr(bigConfigString.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            };
            continue; // goto rescan
        }

        if cmd == "cs" {
            CL_ConfigstringModified(common, cl);
            // Reparse the string, because CL_ConfigstringModified may have done another Cmd_TokenizeString().
            Cmd_TokenizeString(common, &s);
            return qboolean::qtrue;
        }

        if cmd == "map_restart" {
            // Clear notify lines and outgoing commands before passing the restart to the cgame.
            Con_ClearNotify(cl);
            unsafe {
                Com_Memset(
                    cl.cl.cmds.as_mut_ptr() as *mut (),
                    0,
                    core::mem::size_of_val(&cl.cl.cmds),
                );
            }
            return qboolean::qtrue;
        }

        // The clientLevelShot command is used during development to generate 128*128
        // screenshots from the intermission point of levels for the menu system to use.
        // We pass it along to the cgame to make apropriate adjustments, but we also clear
        // the console and notify lines here.
        if cmd == "clientLevelShot" {
            // Don't do it if we aren't running the server locally, otherwise malicious
            // remote servers could overwrite the existing thumbnails.
            if unsafe { (*common.com_sv_running).integer } == 0 {
                return qboolean::qfalse;
            }
            // Close the console.
            Con_Close(common, cl);
            // Take a special screenshot next frame.
            Cbuf_AddText(common, "wait ; wait ; wait ; wait ; screenshot levelshot\n");
            return qboolean::qtrue;
        }

        // We may want to put a "connect to other server" command here.

        // Cgame can now act on the command.
        return qboolean::qtrue;
    }
}

/// Raven `CL_InitCGame`.
/// Loads the cgame module against the interpreter the connected server used
/// (or `vm_cgame` off a pure server), then drives it through init to primed.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1743-1805`
pub fn CL_InitCGame(common: &mut Common, cl: &mut Client) {
    // Put away the console.
    Con_Close(common, cl);

    // Find the current mapname.
    //TODO: Port CS_SERVERINFO
    let info = unsafe {
        core::ffi::CStr::from_ptr(
            cl.cl
                .gameState
                .stringData
                .as_ptr()
                .offset(cl.cl.gameState.stringOffsets[CS_SERVERINFO as usize] as isize),
        )
        .to_string_lossy()
        .into_owned()
    };
    let mapname = Info_ValueForKey(&info, "mapname");
    let bsp_path = format!("maps/{}.bsp", mapname);
    let bsp_path_c = std::ffi::CString::new(bsp_path).unwrap_or_default();
    unsafe {
        Com_sprintf(
            cl.cl.mapname.as_mut_ptr(),
            cl.cl.mapname.len() as c_int,
            &bsp_path_c.to_string_lossy(),
        );
    }

    // Load the dll or bytecode.
    let interpret: vmInterpret_t;
    //TODO: Port cl_connectedToPureServer
    // Source: oracle/codemp/client/../RMG/../client/client.h:507
    if cl.cl_connectedToPureServer != 0 {
        // Load the module type based on what the server is doing -rww.
        //TODO: Port cl_connectedCGAME
        // Source: oracle/codemp/client/../RMG/../client/client.h:509
        interpret = unsafe { core::mem::transmute(cl.cl_connectedCGAME) };
    } else {
        interpret =
            unsafe { core::mem::transmute(Cvar_VariableValue(common, "vm_cgame") as c_int) };
    }
    cl.cgvm = VM_Create(
        host,
        "cgame",
        Some(CL_CgameSystemCalls_trampoline),
        interpret,
    );
    if cl.cgvm.is_null() {
        com_error(
            errorParm_t::ERR_DROP,
            "VM_Create on cgame failed".to_string(),
        );
    }
    cl.cls.state = connstate_t::CA_LOADING;

    // Init for this gamestate.
    // Use the lastExecutedServerCommand instead of the serverCommandSequence, otherwise
    // server commands sent just before a gamestate are dropped.
    VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_INIT as c_int,
        &[
            cl.clc.serverMessageSequence as isize,
            cl.clc.lastExecutedServerCommand as isize,
            cl.clc.clientNum as isize,
        ],
    );

    // We will send a usercmd this frame, which will cause the server to send us the first
    // snapshot.
    cl.cls.state = connstate_t::CA_PRIMED;

    // Have the renderer touch all its images, so they are present on the card even if the
    // driver does deferred loading.
    //TODO: Port re
    // Source: oracle/codemp/client/../RMG/../client/client.h:388
    unsafe {
        ((*cl.re).EndRegistration)();
    }

    // Make sure everything is paged in.
    Com_TouchMemory(common);

    // Clear anything that got printed.
    Con_ClearNotify(cl);
}

/// Raven `CL_SetCGameTime`.
/// Derives `cl.serverTime` from `serverTimeDelta`, clamped so it never flows
/// backwards, then drains queued demo messages until caught up.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1980-2105`
pub fn CL_SetCGameTime(common: &mut Common, cl: &mut Client) {
    // Getting a valid frame message ends the connection process.
    if cl.cls.state != connstate_t::CA_ACTIVE {
        if cl.cls.state != connstate_t::CA_PRIMED {
            return;
        }
        if cl.clc.demoplaying != qboolean::qfalse {
            // We shouldn't get the first snapshot on the same frame as the gamestate,
            // because it causes a bad time skip.
            if cl.clc.firstDemoFrameSkipped == qboolean::qfalse {
                cl.clc.firstDemoFrameSkipped = qboolean::qtrue;
                return;
            }
            CL_ReadDemoMessage(common, cl);
        }
        if cl.cl.newSnapshots != qboolean::qfalse {
            cl.cl.newSnapshots = qboolean::qfalse;
            CL_FirstSnapshot(cl);
        }
        if cl.cls.state != connstate_t::CA_ACTIVE {
            return;
        }
    }

    // If we have gotten to this point, cl.snap is guaranteed to be valid.
    if cl.cl.snap.valid == qboolean::qfalse {
        com_error(
            errorParm_t::ERR_DROP,
            "CL_SetCGameTime: !cl.snap.valid".to_string(),
        );
    }

    // Allow pause in single player.
    //TODO: Port sv_paused
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h:712
    //TODO: Port cl_paused
    // Source: oracle/codemp/client/../qcommon/../qcommon/qcommon.h:711
    if unsafe { (*cl.sv_paused).integer } != 0
        && unsafe { (*cl.cl_paused).integer } != 0
        && unsafe { (*common.com_sv_running).integer } != 0
    {
        // Paused.
        return;
    }

    if cl.cl.snap.serverTime < cl.cl.oldFrameServerTime {
        com_error(
            errorParm_t::ERR_DROP,
            "cl.snap.serverTime < cl.oldFrameServerTime".to_string(),
        );
    }
    cl.cl.oldFrameServerTime = cl.cl.snap.serverTime;

    // Get our current view of time.
    //TODO: Port cl_freezeDemo
    // Source: oracle/codemp/client/../RMG/../client/client.h:404
    if cl.clc.demoplaying != qboolean::qfalse && unsafe { (*cl.cl_freezeDemo).integer } != 0 {
        // cl_freezeDemo is used to lock a demo in place for single frame advances.
    } else {
        // cl_timeNudge is a user adjustable cvar that allows more or less latency to be
        // added in the interest of better smoothness or better responsiveness.
        //TODO: Port cl_timeNudge
        // Source: oracle/codemp/client/../RMG/../client/client.h:402
        let mut tn = unsafe { (*cl.cl_timeNudge).integer };
        if tn < -30 {
            tn = -30;
        } else if tn > 30 {
            tn = 30;
        }

        cl.cl.serverTime = cl.cls.realtime + cl.cl.serverTimeDelta - tn;

        // Guarantee that time will never flow backwards, even if serverTimeDelta made an
        // adjustment or cl_timeNudge was changed.
        if cl.cl.serverTime < cl.cl.oldServerTime {
            cl.cl.serverTime = cl.cl.oldServerTime;
        }
        cl.cl.oldServerTime = cl.cl.serverTime;

        // Note if we are almost past the latest frame (without timeNudge), so we will try
        // and adjust back a bit when the next snapshot arrives.
        if cl.cls.realtime + cl.cl.serverTimeDelta >= cl.cl.snap.serverTime - 5 {
            cl.cl.extrapolatedSnapshot = qboolean::qtrue;
        }
    }

    // If we have gotten new snapshots, drift serverTimeDelta. Don't do this every frame, or
    // a period of packet loss would make a huge adjustment.
    if cl.cl.newSnapshots != qboolean::qfalse {
        CL_AdjustTimeDelta(common, cl);
    }

    if cl.clc.demoplaying == qboolean::qfalse {
        return;
    }

    // If we are playing a demo back, we can just keep reading messages from the demo file
    // until the cgame definately has valid snapshots to interpolate between.

    // A timedemo will always use a deterministic set of time samples no matter what speed
    // machine it is run on, while a normal demo may have different time samples each time
    // it is played back.
    //TODO: Port cl_timedemo
    // Source: oracle/codemp/client/../RMG/../client/client.h:424
    if unsafe { (*cl.cl_timedemo).integer } != 0 {
        if cl.clc.timeDemoStart == 0 {
            cl.clc.timeDemoStart = sys_milliseconds(host, false);
        }
        cl.clc.timeDemoFrames += 1;
        cl.cl.serverTime = cl.clc.timeDemoBaseTime + cl.clc.timeDemoFrames * 50;
    }

    while cl.cl.serverTime >= cl.cl.snap.serverTime {
        // Feed another message, which should change the contents of cl.snap.
        CL_ReadDemoMessage(common, cl);
        if cl.cls.state != connstate_t::CA_ACTIVE {
            return; // end of demo
        }
    }
}

/// The `extern "C"` trampoline `VM_Create` calls into. The VM only knows a
/// bare `fn(*mut c_int) -> c_int`; the real state receivers get threaded
/// through the one retained `Engine` instance at the call site (integration
/// wires this the same way the other module hosts do).
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:644` (call-site shape only)
extern "C" fn CL_CgameSystemCalls_trampoline(args: *mut c_int) -> c_int {
    //TODO: Port CL_CgameSystemCalls_trampoline
    // Source: oracle/codemp/client/cl_cgame.cpp:644-1733 (needs the retained Engine instance)
    todo!("Port CL_CgameSystemCalls_trampoline — oracle/codemp/client/cl_cgame.cpp:644")
}

/// Raven `CL_CgameSystemCalls`.
/// The cgame VM's syscall trap dispatcher: one `args[0]` op code per Raven
/// `CG_*`/`TRAP_*` constant, routed to the matching engine call.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:644-1733`
pub fn CL_CgameSystemCalls(
    common: &mut Common,
    cm: &mut CollisionWorld,
    cl: &mut Client,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    args: *mut c_int,
) -> c_int {
    let arg = |i: isize| -> c_int { unsafe { *args.offset(i) } };
    let vma = |i: isize| -> *mut () { VM_ArgPtr(common, arg(i)) };
    let vmf = |i: isize| -> f32 { unsafe { *(vma(i) as *const f32) } };
    let op = arg(0);

    // rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap to
    // match, and all of these traps must be shared and have cases in sv_game, cl_cgame, and
    // cl_ui. They must also all be in the same order, and start at 100.
    if op == sharedTraps_t::TRAP_MEMSET as c_int {
        unsafe { Com_Memset(vma(1), arg(2), arg(3) as usize) };
        0
    } else if op == sharedTraps_t::TRAP_MEMCPY as c_int {
        unsafe { Com_Memcpy(vma(1), vma(2) as *const (), arg(3) as usize) };
        0
    } else if op == sharedTraps_t::TRAP_STRNCPY as c_int {
        unsafe {
            libc::strncpy(
                vma(1) as *mut c_char,
                vma(2) as *const c_char,
                arg(3) as usize,
            ) as isize as c_int
        }
    } else if op == sharedTraps_t::TRAP_SIN as c_int {
        FloatAsInt(vmf(1).sin())
    } else if op == sharedTraps_t::TRAP_COS as c_int {
        FloatAsInt(vmf(1).cos())
    } else if op == sharedTraps_t::TRAP_ATAN2 as c_int {
        FloatAsInt(vmf(1).atan2(vmf(2)))
    } else if op == sharedTraps_t::TRAP_SQRT as c_int {
        FloatAsInt(vmf(1).sqrt())
    } else if op == sharedTraps_t::TRAP_MATRIXMULTIPLY as c_int {
        unsafe {
            MatrixMultiply(
                &*(vma(1) as *const [[f32; 3]; 3]),
                &*(vma(2) as *const [[f32; 3]; 3]),
                &mut *(vma(3) as *mut [[f32; 3]; 3]),
            );
        }
        0
    } else if op == sharedTraps_t::TRAP_ANGLEVECTORS as c_int {
        unsafe {
            let angles = *(vma(1) as *const vec3_t);
            AngleVectors(
                angles,
                (vma(2) as *mut vec3_t).as_mut(),
                (vma(3) as *mut vec3_t).as_mut(),
                (vma(4) as *mut vec3_t).as_mut(),
            );
        }
        0
    } else if op == sharedTraps_t::TRAP_PERPENDICULARVECTOR as c_int {
        //TODO: Port PerpendicularVector
        unsafe { PerpendicularVector(vma(1) as *mut f32, vma(2) as *const f32) };
        0
    } else if op == sharedTraps_t::TRAP_FLOOR as c_int {
        FloatAsInt(vmf(1).floor())
    } else if op == sharedTraps_t::TRAP_CEIL as c_int {
        FloatAsInt(vmf(1).ceil())
    } else if op == sharedTraps_t::TRAP_TESTPRINTINT as c_int {
        0
    } else if op == sharedTraps_t::TRAP_TESTPRINTFLOAT as c_int {
        0
    } else if op == sharedTraps_t::TRAP_ACOS as c_int {
        FloatAsInt(Q_acos(vmf(1)))
    } else if op == sharedTraps_t::TRAP_ASIN as c_int {
        FloatAsInt(Q_asin(vmf(1)))
    } else if op == MpCgameImport::CG_PRINT as c_int {
        let s = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        com_printf(common, &s);
        0
    } else if op == MpCgameImport::CG_ERROR as c_int {
        let s = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        com_error(errorParm_t::ERR_DROP, s);
    } else if op == MpCgameImport::CG_MILLISECONDS as c_int {
        sys_milliseconds(host, false)
    } else if op == MpCgameImport::CG_PRECISIONTIMER_START as c_int {
        // rww - precision timer funcs... -ALWAYS- call end after start with supplied ptr, or
        // you'll get a nasty memory leak. Not that you should be using these outside of
        // debug anyway.. because you shouldn't be. So don't.
        unsafe {
            let suppliedPtr = vma(1) as *mut *mut timing_c;
            let newTimer = Box::into_raw(Box::new(timing_c::default()));
            *suppliedPtr = newTimer;
            (*newTimer).Start();
        }
        0
    } else if op == MpCgameImport::CG_PRECISIONTIMER_END as c_int {
        unsafe {
            let timer = arg(1) as *mut timing_c;
            let r = (*timer).End();
            drop(Box::from_raw(timer));
            r
        }
    } else if op == MpCgameImport::CG_CVAR_REGISTER as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let value = unsafe {
            core::ffi::CStr::from_ptr(vma(3) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Cvar_Register(host, vma(1) as *mut vmCvar_t, &name, &value, arg(4));
        0
    } else if op == MpCgameImport::CG_CVAR_UPDATE as c_int {
        Cvar_Update(common, vma(1) as *mut vmCvar_t);
        0
    } else if op == MpCgameImport::CG_CVAR_SET as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let value = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Cvar_Set(host, &name, &value);
        0
    } else if op == MpCgameImport::CG_CVAR_VARIABLESTRINGBUFFER as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Cvar_VariableStringBuffer(common, &name, vma(2) as *mut c_char, arg(3));
        0
    } else if op == MpCgameImport::CG_CVAR_GETHIDDENVALUE as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        CL_GetValueForHidden(cl, &name)
    } else if op == MpCgameImport::CG_ARGC as c_int {
        Cmd_Argc(common)
    } else if op == MpCgameImport::CG_ARGV as c_int {
        //TODO: Port Cmd_ArgvBuffer
        unsafe { Cmd_ArgvBuffer(common, arg(1), vma(2) as *mut c_char, arg(3)) };
        0
    } else if op == MpCgameImport::CG_ARGS as c_int {
        let s = Cmd_ArgsBuffer(common, arg(2) as usize);
        let s_c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { libc::strcpy(vma(1) as *mut c_char, s_c.as_ptr()) };
        0
    } else if op == MpCgameImport::CG_FS_FOPENFILE as c_int {
        let path = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        FS_FOpenFileByMode(host, &path, vma(2) as *mut fileHandle_t, unsafe {
            core::mem::transmute(arg(3))
        })
    } else if op == MpCgameImport::CG_FS_READ as c_int {
        FS_Read2(common, vma(1), arg(2), unsafe {
            core::mem::transmute(arg(3))
        });
        0
    } else if op == MpCgameImport::CG_FS_WRITE as c_int {
        FS_Write(common, vma(1) as *const (), arg(2), unsafe {
            core::mem::transmute(arg(3))
        });
        0
    } else if op == MpCgameImport::CG_FS_FCLOSEFILE as c_int {
        FS_FCloseFile(common, unsafe { core::mem::transmute(arg(1)) });
        0
    } else if op == MpCgameImport::CG_FS_GETFILELIST as c_int {
        let path = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let ext = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        FS_GetFileList(host, &path, &ext, vma(3) as *mut c_char, arg(4))
    } else if op == MpCgameImport::CG_SENDCONSOLECOMMAND as c_int {
        let s = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Cbuf_AddText(common, &s);
        0
    } else if op == MpCgameImport::CG_ADDCOMMAND as c_int {
        CL_AddCgameCommand(vma(1) as *const c_char);
        0
    } else if op == MpCgameImport::CG_REMOVECOMMAND as c_int {
        let s = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Cmd_RemoveCommand(common, &s);
        0
    } else if op == MpCgameImport::CG_SENDCLIENTCOMMAND as c_int {
        //TODO: Port CL_AddReliableCommand
        CL_AddReliableCommand(cl, vma(1) as *const c_char);
        0
    } else if op == MpCgameImport::CG_UPDATESCREEN as c_int {
        // This is used during lengthy level loading, so pump message loop.
        // FIXME: if a server restarts here, BAD THINGS HAPPEN!
        // We can't call Com_EventLoop here, a restart will crash and this _does_ happen if
        // there is a map change while we are downloading at pk3. -ZOID
        SCR_UpdateScreen(common, cl);
        0
    } else if op == MpCgameImport::CG_CM_LOADMAP as c_int {
        if arg(2) != 0 {
            let name = unsafe {
                core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                    .to_string_lossy()
                    .into_owned()
            };
            CM_LoadSubBSP(host, &format!("maps/{}.bsp", &name[1..]), qboolean::qfalse);
        } else {
            CL_CM_LoadMap(vma(1) as *const c_char);
        }
        0
    } else if op == MpCgameImport::CG_CM_NUMINLINEMODELS as c_int {
        CM_NumInlineModels(cm)
    } else if op == MpCgameImport::CG_CM_INLINEMODEL as c_int {
        CM_InlineModel(cm, arg(1))
    } else if op == MpCgameImport::CG_CM_TEMPBOXMODEL as c_int {
        CM_TempBoxModel(
            cm,
            unsafe { *(vma(1) as *const vec3_t) },
            unsafe { *(vma(2) as *const vec3_t) },
            qboolean::qfalse as c_int,
        )
    } else if op == MpCgameImport::CG_CM_TEMPCAPSULEMODEL as c_int {
        CM_TempBoxModel(
            cm,
            unsafe { *(vma(1) as *const vec3_t) },
            unsafe { *(vma(2) as *const vec3_t) },
            qboolean::qtrue as c_int,
        )
    } else if op == MpCgameImport::CG_CM_POINTCONTENTS as c_int {
        CM_PointContents(cm, unsafe { *(vma(1) as *const vec3_t) }, arg(2))
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS as c_int {
        CM_TransformedPointContents(
            cm,
            unsafe { *(vma(1) as *const vec3_t) },
            arg(2),
            unsafe { *(vma(3) as *const vec3_t) },
            unsafe { *(vma(4) as *const vec3_t) },
        )
    } else if op == MpCgameImport::CG_CM_BOXTRACE as c_int {
        CM_BoxTrace(
            host,
            vma(1) as *mut trace_t,
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            unsafe { *(vma(4) as *const vec3_t) },
            unsafe { *(vma(5) as *const vec3_t) },
            arg(6),
            arg(7),
            qboolean::qfalse as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_CAPSULETRACE as c_int {
        CM_BoxTrace(
            host,
            vma(1) as *mut trace_t,
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            unsafe { *(vma(4) as *const vec3_t) },
            unsafe { *(vma(5) as *const vec3_t) },
            arg(6),
            arg(7),
            qboolean::qtrue as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDBOXTRACE as c_int {
        CM_TransformedBoxTrace(
            host,
            vma(1) as *mut trace_t,
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            unsafe { *(vma(4) as *const vec3_t) },
            unsafe { *(vma(5) as *const vec3_t) },
            arg(6),
            arg(7),
            unsafe { *(vma(8) as *const vec3_t) },
            unsafe { *(vma(9) as *const vec3_t) },
            qboolean::qfalse as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDCAPSULETRACE as c_int {
        CM_TransformedBoxTrace(
            host,
            vma(1) as *mut trace_t,
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            unsafe { *(vma(4) as *const vec3_t) },
            unsafe { *(vma(5) as *const vec3_t) },
            arg(6),
            arg(7),
            unsafe { *(vma(8) as *const vec3_t) },
            unsafe { *(vma(9) as *const vec3_t) },
            qboolean::qtrue as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_MARKFRAGMENTS as c_int {
        unsafe {
            ((*cl.re).MarkFragments)(
                arg(1),
                vma(2) as *const vec3_t,
                vma(3) as *const f32,
                arg(4),
                vma(5) as *mut f32,
                arg(6),
                vma(7) as *mut markFragment_t,
            )
        }
    } else if op == MpCgameImport::CG_S_GETVOICEVOLUME as c_int {
        //TODO: Port s_entityWavVol
        // Source: oracle/codemp/client/cl_cgame.cpp:629
        cl.s_entityWavVol[arg(1) as usize]
    } else if op == MpCgameImport::CG_S_MUTESOUND as c_int {
        S_MuteSound(cl, arg(1), arg(2));
        0
    } else if op == MpCgameImport::CG_S_STARTSOUND as c_int {
        S_StartSound(cl, vma(1) as *mut f32, arg(2), arg(3), arg(4));
        0
    } else if op == MpCgameImport::CG_S_STARTLOCALSOUND as c_int {
        S_StartLocalSound(cl, arg(1), arg(2));
        0
    } else if op == MpCgameImport::CG_S_CLEARLOOPINGSOUNDS as c_int {
        S_ClearLoopingSounds(cl);
        0
    } else if op == MpCgameImport::CG_S_ADDLOOPINGSOUND as c_int {
        S_AddLoopingSound(
            cl,
            arg(1),
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_ADDREALLOOPINGSOUND as c_int {
        // S_AddRealLoopingSound(args[1], (const float *)VMA(2), (const float *)VMA(3), args[4]);
        S_AddLoopingSound(
            cl,
            arg(1),
            unsafe { *(vma(2) as *const vec3_t) },
            unsafe { *(vma(3) as *const vec3_t) },
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_STOPLOOPINGSOUND as c_int {
        S_StopLoopingSound(cl, arg(1));
        0
    } else if op == MpCgameImport::CG_S_UPDATEENTITYPOSITION as c_int {
        S_UpdateEntityPosition(cl, arg(1), unsafe { *(vma(2) as *const vec3_t) });
        0
    } else if op == MpCgameImport::CG_S_RESPATIALIZE as c_int {
        S_Respatialize(
            cl,
            arg(1),
            unsafe { *(vma(2) as *const vec3_t) },
            vma(3) as *mut vec3_t,
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_SHUTUP as c_int {
        //TODO: Port s_shutUp
        // Source: oracle/codemp/client/../RMG/../client/snd_public.h:59
        cl.s_shutUp = unsafe { core::mem::transmute(arg(1)) };
        0
    } else if op == MpCgameImport::CG_S_REGISTERSOUND as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        S_RegisterSound(cl, &name)
    } else if op == MpCgameImport::CG_S_STARTBACKGROUNDTRACK as c_int {
        let a = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let b = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        S_StartBackgroundTrack(
            cl,
            &a,
            &b,
            if arg(3) != 0 {
                qboolean::qtrue
            } else {
                qboolean::qfalse
            },
        );
        0
    } else if op == MpCgameImport::CG_S_UPDATEAMBIENTSET as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        S_UpdateAmbientSet(common, cl, &name, vma(2) as *mut f32);
        0
    } else if op == MpCgameImport::CG_AS_PARSESETS as c_int {
        AS_ParseSets(cl);
        0
    } else if op == MpCgameImport::CG_AS_ADDPRECACHEENTRY as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        AS_AddPrecacheEntry(cl, &name);
        0
    } else if op == MpCgameImport::CG_S_ADDLOCALSET as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        S_AddLocalSet(
            common,
            cl,
            &name,
            vma(2) as *mut f32,
            vma(3) as *mut f32,
            arg(4),
            arg(5),
        )
    } else if op == MpCgameImport::CG_AS_GETBMODELSOUND as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        AS_GetBModelSound(cl, &name, arg(2))
    } else if op == MpCgameImport::CG_R_LOADWORLDMAP as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).LoadWorld)(&name) };
        0
    } else if op == MpCgameImport::CG_R_REGISTERMODEL as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RegisterModel)(&name) }
    } else if op == MpCgameImport::CG_R_REGISTERSKIN as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RegisterSkin)(&name) }
    } else if op == MpCgameImport::CG_R_REGISTERSHADER as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RegisterShader)(&name) }
    } else if op == MpCgameImport::CG_R_REGISTERSHADERNOMIP as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RegisterShaderNoMip)(&name) }
    } else if op == MpCgameImport::CG_R_REGISTERFONT as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RegisterFont)(&name) }
    } else if op == MpCgameImport::CG_R_FONT_STRLENPIXELS as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).Font_StrLenPixels)(&name, arg(2), vmf(3)) }
    } else if op == MpCgameImport::CG_R_FONT_STRLENCHARS as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).Font_StrLenChars)(&name) }
    } else if op == MpCgameImport::CG_R_FONT_STRHEIGHTPIXELS as c_int {
        unsafe { ((*cl.re).Font_HeightPixels)(arg(1), vmf(2)) }
    } else if op == MpCgameImport::CG_R_FONT_DRAWSTRING as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(3) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe {
            ((*cl.re).Font_DrawString)(
                arg(1),
                arg(2),
                &name,
                vma(4) as *const f32,
                arg(5),
                arg(6),
                vmf(7),
            )
        };
        0
    } else if op == MpCgameImport::CG_LANGUAGE_ISASIAN as c_int {
        unsafe { ((*cl.re).Language_IsAsian)() }
    } else if op == MpCgameImport::CG_LANGUAGE_USESSPACES as c_int {
        unsafe { ((*cl.re).Language_UsesSpaces)() }
    } else if op == MpCgameImport::CG_ANYLANGUAGE_READCHARFROMSTRING as c_int {
        unsafe {
            ((*cl.re).AnyLanguage_ReadCharFromString)(
                vma(1) as *const c_char,
                vma(2) as *mut c_int,
                vma(3) as *mut qboolean,
            )
        }
    } else if op == MpCgameImport::CG_R_CLEARSCENE as c_int {
        unsafe { ((*cl.re).ClearScene)() };
        0
    } else if op == MpCgameImport::CG_R_CLEARDECALS as c_int {
        unsafe { ((*cl.re).ClearDecals)() };
        0
    } else if op == MpCgameImport::CG_R_ADDREFENTITYTOSCENE as c_int {
        unsafe { ((*cl.re).AddRefEntityToScene)(vma(1) as *const refEntity_t) };
        0
    } else if op == MpCgameImport::CG_R_ADDPOLYTOSCENE as c_int {
        unsafe { ((*cl.re).AddPolyToScene)(arg(1), arg(2), vma(3) as *const polyVert_t, 1) };
        0
    } else if op == MpCgameImport::CG_R_ADDPOLYSTOSCENE as c_int {
        unsafe { ((*cl.re).AddPolyToScene)(arg(1), arg(2), vma(3) as *const polyVert_t, arg(4)) };
        0
    } else if op == MpCgameImport::CG_R_ADDDECALTOSCENE as c_int {
        unsafe {
            ((*cl.re).AddDecalToScene)(
                arg(1),
                vma(2) as *const f32,
                vma(3) as *const f32,
                vmf(4),
                vmf(5),
                vmf(6),
                vmf(7),
                vmf(8),
                core::mem::transmute(arg(9)),
                vmf(10),
                core::mem::transmute(arg(11)),
            );
        }
        0
    } else if op == MpCgameImport::CG_R_LIGHTFORPOINT as c_int {
        unsafe {
            ((*cl.re).LightForPoint)(
                vma(1) as *mut f32,
                vma(2) as *mut f32,
                vma(3) as *mut f32,
                vma(4) as *mut f32,
            )
        }
    } else if op == MpCgameImport::CG_R_ADDLIGHTTOSCENE as c_int {
        unsafe { ((*cl.re).AddLightToScene)(vma(1) as *const f32, vmf(2), vmf(3), vmf(4), vmf(5)) };
        0
    } else if op == MpCgameImport::CG_R_ADDADDITIVELIGHTTOSCENE as c_int {
        unsafe {
            ((*cl.re).AddAdditiveLightToScene)(vma(1) as *const f32, vmf(2), vmf(3), vmf(4), vmf(5))
        };
        0
    } else if op == MpCgameImport::CG_R_RENDERSCENE as c_int {
        unsafe { ((*cl.re).RenderScene)(vma(1) as *const refdef_t) };
        0
    } else if op == MpCgameImport::CG_R_SETCOLOR as c_int {
        unsafe { ((*cl.re).SetColor)(vma(1) as *const f32) };
        0
    } else if op == MpCgameImport::CG_R_DRAWSTRETCHPIC as c_int {
        unsafe {
            ((*cl.re).DrawStretchPic)(
                vmf(1),
                vmf(2),
                vmf(3),
                vmf(4),
                vmf(5),
                vmf(6),
                vmf(7),
                vmf(8),
                arg(9),
            )
        };
        0
    } else if op == MpCgameImport::CG_R_MODELBOUNDS as c_int {
        unsafe { ((*cl.re).ModelBounds)(arg(1), vma(2) as *mut f32, vma(3) as *mut f32) };
        0
    } else if op == MpCgameImport::CG_R_LERPTAG as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(6) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe {
            ((*cl.re).LerpTag)(
                vma(1) as *mut orientation_t,
                arg(2),
                arg(3),
                arg(4),
                vmf(5),
                &name,
            )
        }
    } else if op == MpCgameImport::CG_R_DRAWROTATEPIC as c_int {
        unsafe {
            ((*cl.re).DrawRotatePic)(
                vmf(1),
                vmf(2),
                vmf(3),
                vmf(4),
                vmf(5),
                vmf(6),
                vmf(7),
                vmf(8),
                vmf(9),
                arg(10),
            )
        };
        0
    } else if op == MpCgameImport::CG_R_DRAWROTATEPIC2 as c_int {
        unsafe {
            ((*cl.re).DrawRotatePic2)(
                vmf(1),
                vmf(2),
                vmf(3),
                vmf(4),
                vmf(5),
                vmf(6),
                vmf(7),
                vmf(8),
                vmf(9),
                arg(10),
            )
        };
        0
    } else if op == MpCgameImport::CG_R_SETRANGEFOG as c_int {
        //TODO: Port tr
        // Source: oracle/codemp/client/../renderer/tr_local.h:1434
        rm.tr.rangedFog = vmf(1);
        0
    } else if op == MpCgameImport::CG_R_SETREFRACTIONPROP as c_int {
        //TODO: Port tr_distortionAlpha / tr_distortionStretch / tr_distortionPrePost / tr_distortionNegate
        // Source: oracle/codemp/client/cl_cgame.cpp:638-641
        cl.tr_distortionAlpha = vmf(1);
        cl.tr_distortionStretch = vmf(2);
        cl.tr_distortionPrePost = unsafe { core::mem::transmute(arg(3)) };
        cl.tr_distortionNegate = unsafe { core::mem::transmute(arg(4)) };
        0
    } else if op == MpCgameImport::CG_GETGLCONFIG as c_int {
        CL_GetGlconfig(cl, vma(1) as *mut glconfig_t);
        0
    } else if op == MpCgameImport::CG_GETGAMESTATE as c_int {
        CL_GetGameState(cl, vma(1) as *mut gameState_t);
        0
    } else if op == MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER as c_int {
        CL_GetCurrentSnapshotNumber(cl, vma(1) as *mut c_int, vma(2) as *mut c_int);
        0
    } else if op == MpCgameImport::CG_GETSNAPSHOT as c_int {
        CL_GetSnapshot(cl, arg(1), vma(2) as *mut snapshot_t) as c_int
    } else if op == MpCgameImport::CG_GETDEFAULTSTATE as c_int {
        CL_GetDefaultState(cl, arg(1), vma(2) as *mut entityState_t) as c_int
    } else if op == MpCgameImport::CG_GETSERVERCOMMAND as c_int {
        CL_GetServerCommand(common, cl, arg(1)) as c_int
    } else if op == MpCgameImport::CG_GETCURRENTCMDNUMBER as c_int {
        CL_GetCurrentCmdNumber(cl)
    } else if op == MpCgameImport::CG_GETUSERCMD as c_int {
        CL_GetUserCmd(cl, arg(1), vma(2) as *mut usercmd_t) as c_int
    } else if op == MpCgameImport::CG_SETUSERCMDVALUE as c_int {
        //TODO: Port cl_bUseFighterPitch
        // Source: oracle/codemp/client/cl_cgame.cpp:642
        cl.cl_bUseFighterPitch = unsafe { core::mem::transmute(arg(8)) };
        CL_SetUserCmdValue(cl, arg(1), vmf(2), vmf(3), vmf(4), vmf(5), arg(6), arg(7));
        0
    } else if op == MpCgameImport::CG_SETCLIENTFORCEANGLE as c_int {
        CL_SetClientForceAngle(cl, arg(1), unsafe { *(vma(2) as *const vec3_t) });
        0
    } else if op == MpCgameImport::CG_SETCLIENTTURNEXTENT as c_int {
        0
    } else if op == MpCgameImport::CG_OPENUIMENU as c_int {
        //TODO: Port uivm
        // Source: oracle/codemp/client/../RMG/../client/client.h:387
        VM_Call(
            common,
            cl.uivm,
            MpUiExport::UI_SET_ACTIVE_MENU as c_int,
            &[arg(1) as isize],
        );
        0
    } else if op == MpCgameImport::CG_MEMORY_REMAINING as c_int {
        Hunk_MemoryRemaining(common)
    } else if op == MpCgameImport::CG_KEY_ISDOWN as c_int {
        Key_IsDown(cl, arg(1))
    } else if op == MpCgameImport::CG_KEY_GETCATCHER as c_int {
        Key_GetCatcher(cl)
    } else if op == MpCgameImport::CG_KEY_SETCATCHER as c_int {
        Key_SetCatcher(cl, arg(1));
        0
    } else if op == MpCgameImport::CG_KEY_GETKEY as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        Key_GetKey(cl, &name)
    } else if op == MpCgameImport::CG_PC_ADD_GLOBAL_DEFINE as c_int {
        //TODO: Port botlib_export
        // Source: oracle/codemp/client/cl_cgame.cpp:61
        unsafe { ((*cl.botlib_export).PC_AddGlobalDefine)(vma(1) as *mut c_char) }
    } else if op == MpCgameImport::CG_PC_LOAD_SOURCE as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.botlib_export).PC_LoadSourceHandle)(&name) }
    } else if op == MpCgameImport::CG_PC_FREE_SOURCE as c_int {
        unsafe { ((*cl.botlib_export).PC_FreeSourceHandle)(arg(1)) }
    } else if op == MpCgameImport::CG_PC_READ_TOKEN as c_int {
        unsafe { ((*cl.botlib_export).PC_ReadTokenHandle)(arg(1), vma(2) as *mut pc_token_s) }
    } else if op == MpCgameImport::CG_PC_SOURCE_FILE_AND_LINE as c_int {
        unsafe {
            ((*cl.botlib_export).PC_SourceFileAndLine)(
                arg(1),
                vma(2) as *mut c_char,
                vma(3) as *mut c_int,
            )
        }
    } else if op == MpCgameImport::CG_PC_LOAD_GLOBAL_DEFINES as c_int {
        unsafe { ((*cl.botlib_export).PC_LoadGlobalDefines)(vma(1) as *mut c_char) }
    } else if op == MpCgameImport::CG_PC_REMOVE_ALL_GLOBAL_DEFINES as c_int {
        unsafe { ((*cl.botlib_export).PC_RemoveAllGlobalDefines)() };
        0
    } else if op == MpCgameImport::CG_S_STOPBACKGROUNDTRACK as c_int {
        S_StopBackgroundTrack(cl);
        0
    } else if op == MpCgameImport::CG_REAL_TIME as c_int {
        Com_RealTime(vma(1) as *mut qtime_t)
    } else if op == MpCgameImport::CG_SNAPVECTOR as c_int {
        Sys_SnapVector(vma(1) as *mut f32);
        0
    } else if op == MpCgameImport::CG_CIN_PLAYCINEMATIC as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        CIN_PlayCinematic(common, cl, &name, arg(2), arg(3), arg(4), arg(5), arg(6))
    } else if op == MpCgameImport::CG_CIN_STOPCINEMATIC as c_int {
        CIN_StopCinematic(cl, arg(1))
    } else if op == MpCgameImport::CG_CIN_RUNCINEMATIC as c_int {
        CIN_RunCinematic(common, cl, arg(1))
    } else if op == MpCgameImport::CG_CIN_DRAWCINEMATIC as c_int {
        CIN_DrawCinematic(cl, arg(1));
        0
    } else if op == MpCgameImport::CG_CIN_SETEXTENTS as c_int {
        CIN_SetExtents(cl, arg(1), arg(2), arg(3), arg(4), arg(5));
        0
    } else if op == MpCgameImport::CG_R_REMAP_SHADER as c_int {
        let a = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let b = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        let c = unsafe {
            core::ffi::CStr::from_ptr(vma(3) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { ((*cl.re).RemapShader)(&a, &b, &c) };
        0
    } else if op == MpCgameImport::CG_R_GET_LIGHT_STYLE as c_int {
        unsafe { ((*cl.re).GetLightStyle)(arg(1), vma(2) as *mut u8) };
        0
    } else if op == MpCgameImport::CG_R_SET_LIGHT_STYLE as c_int {
        unsafe { ((*cl.re).SetLightStyle)(arg(1), arg(2)) };
        0
    } else if op == MpCgameImport::CG_R_GET_BMODEL_VERTS as c_int {
        unsafe { ((*cl.re).GetBModelVerts)(arg(1), vma(2) as *mut [f32; 3], vma(3) as *mut f32) };
        0
    } else if op == MpCgameImport::CG_R_GETDISTANCECULL as c_int {
        unsafe {
            //TODO: Port tr
            *(vma(1) as *mut f32) = rm.tr.distanceCull;
        }
        0
    } else if op == MpCgameImport::CG_R_GETREALRES as c_int {
        unsafe {
            //TODO: Port glConfig
            // Source: oracle/codemp/client/../renderer/tr_local.h:1435
            *(vma(1) as *mut c_int) = rm.glConfig.vidWidth;
            *(vma(2) as *mut c_int) = rm.glConfig.vidHeight;
        }
        0
    } else if op == MpCgameImport::CG_R_AUTOMAPELEVADJ as c_int {
        R_AutomapElevationAdjustment(rm, vmf(1));
        0
    } else if op == MpCgameImport::CG_R_INITWIREFRAMEAUTO as c_int {
        R_InitializeWireframeAutomap(rm) as c_int
    } else if op == MpCgameImport::CG_GET_ENTITY_TOKEN as c_int {
        unsafe { ((*cl.re).GetEntityToken)(vma(1) as *mut c_char, arg(2)) }
    } else if op == MpCgameImport::CG_R_INPVS as c_int {
        unsafe {
            ((*cl.re).inPVS)(
                vma(1) as *const f32,
                vma(2) as *const f32,
                vma(3) as *mut u8,
            )
        }
    } else if op == MpCgameImport::CG_FX_ADDLINE as c_int {
        //TODO: Port FX_AddLine
        // Source: FX subsystem design pending (gh#26)
        unsafe {
            FX_AddLine(
                vma(1) as *mut f32,
                vma(2) as *mut f32,
                vmf(3),
                vmf(4),
                vmf(5),
                vmf(6),
                vmf(7),
                vmf(8),
                vma(9) as *mut f32,
                vma(10) as *mut f32,
                vmf(11),
                arg(12),
                arg(13),
                arg(14),
            );
        }
        0
    } else if op == MpCgameImport::CG_FX_REGISTER_EFFECT as c_int {
        //TODO: Port FX_RegisterEffect
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe { FX_RegisterEffect(&name) }
    } else if op == MpCgameImport::CG_FX_PLAY_EFFECT as c_int {
        //TODO: Port FX_PlayEffect
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        unsafe {
            FX_PlayEffect(
                &name,
                vma(2) as *mut f32,
                vma(3) as *mut f32,
                arg(4),
                arg(5),
            )
        };
        0
    } else if op == MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT as c_int {
        // Raven: assert(0);//gone! — the entity-effect entry point was removed upstream.
        unreachable!("CG_FX_PLAY_ENTITY_EFFECT — gone in the oracle (cl_cgame.cpp:1112-1115)")
    } else if op == MpCgameImport::CG_FX_PLAY_EFFECT_ID as c_int {
        //TODO: Port FX_PlayEffectID
        unsafe {
            FX_PlayEffectID(
                arg(1),
                vma(2) as *mut f32,
                vma(3) as *mut f32,
                arg(4),
                arg(5),
                qboolean::qfalse,
            )
        };
        0
    } else if op == MpCgameImport::CG_FX_PLAY_PORTAL_EFFECT_ID as c_int {
        unsafe {
            FX_PlayEffectID(
                arg(1),
                vma(2) as *mut f32,
                vma(3) as *mut f32,
                arg(4),
                arg(5),
                qboolean::qtrue,
            )
        };
        0
    } else if op == MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT_ID as c_int {
        //TODO: Port FX_PlayEntityEffectID
        unsafe {
            FX_PlayEntityEffectID(
                arg(1),
                vma(2) as *mut f32,
                vma(3) as *mut vec3_t,
                arg(4),
                arg(5),
                arg(6),
                arg(7),
            )
        };
        0
    } else if op == MpCgameImport::CG_FX_PLAY_BOLTED_EFFECT_ID as c_int {
        //TODO: Port CGhoul2Info_v index / G2API_AttachEnt / FX_PlayBoltedEffectID
        unsafe {
            let g2v = &mut *(arg(3) as *mut CGhoul2Info_v);
            let mut boltInfo: c_int = 0;
            if G2API_AttachEnt(
                &mut boltInfo,
                &mut g2v[arg(6) as usize],
                arg(4),
                arg(5),
                arg(6),
            ) != qboolean::qfalse
            {
                FX_PlayBoltedEffectID(
                    arg(1),
                    vma(2) as *mut f32,
                    boltInfo,
                    g2v.mItem,
                    arg(7),
                    core::mem::transmute(arg(8)),
                );
                1
            } else {
                0
            }
        }
    } else if op == MpCgameImport::CG_FX_ADD_SCHEDULED_EFFECTS as c_int {
        unsafe { FX_AddScheduledEffects(core::mem::transmute(arg(1))) };
        0
    } else if op == MpCgameImport::CG_FX_DRAW_2D_EFFECTS as c_int {
        unsafe { FX_Draw2DEffects(vmf(1), vmf(2)) };
        0
    } else if op == MpCgameImport::CG_FX_INIT_SYSTEM as c_int {
        unsafe { FX_InitSystem(vma(1) as *mut refdef_t) }
    } else if op == MpCgameImport::CG_FX_SET_REFDEF as c_int {
        unsafe { FX_SetRefDefFromCGame(vma(1) as *mut refdef_t) };
        0
    } else if op == MpCgameImport::CG_FX_FREE_SYSTEM as c_int {
        unsafe { FX_FreeSystem() }
    } else if op == MpCgameImport::CG_FX_ADJUST_TIME as c_int {
        unsafe { FX_AdjustTime(arg(1)) };
        0
    } else if op == MpCgameImport::CG_FX_RESET as c_int {
        unsafe { FX_Free(false) };
        0
    } else if op == MpCgameImport::CG_FX_ADDPOLY as c_int {
        unsafe {
            let p = vma(1) as *mut addpolyArgStruct_t;
            if !p.is_null() {
                FX_AddPoly(*p);
            }
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDBEZIER as c_int {
        unsafe {
            let b = vma(1) as *mut addbezierArgStruct_t;
            if !b.is_null() {
                FX_AddBezier(*b);
            }
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDPRIMITIVE as c_int {
        unsafe {
            let a = vma(1) as *mut effectTrailArgStruct_t;
            if !a.is_null() {
                FX_FeedTrail(a);
            }
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDSPRITE as c_int {
        unsafe {
            let s = vma(1) as *mut addspriteArgStruct_t;
            if !s.is_null() {
                let rgb: vec3_t = [1.0, 1.0, 1.0];
                // FX_AddSprite(NULL, s->origin, s->vel, s->accel, s->scale, s->dscale, s->sAlpha,
                //   s->eAlpha, s->rotation, s->bounce, s->life, s->shader, s->flags);
                FX_AddParticle(
                    (*s).origin,
                    (*s).vel,
                    (*s).accel,
                    (*s).scale,
                    (*s).dscale,
                    0.0,
                    (*s).sAlpha,
                    (*s).eAlpha,
                    0.0,
                    rgb,
                    rgb,
                    0,
                    (*s).rotation,
                    0,
                    vec3_origin,
                    vec3_origin,
                    (*s).bounce,
                    0,
                    0,
                    (*s).life,
                    (*s).shader,
                    (*s).flags,
                );
            }
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDELECTRICITY as c_int {
        unsafe {
            let p = vma(1) as *mut addElectricityArgStruct_t;
            if !p.is_null() {
                FX_AddElectricity(*p);
            }
        }
        0
    } else if op == MpCgameImport::CG_ROFF_CLEAN as c_int {
        //TODO: Port theROFFSystem
        // Source: oracle/codemp/client/../qcommon/ROFFSystem.h:183
        cl.theROFFSystem.Clean(qboolean::qtrue)
    } else if op == MpCgameImport::CG_ROFF_UPDATE_ENTITIES as c_int {
        cl.theROFFSystem.UpdateEntities(qboolean::qtrue);
        0
    } else if op == MpCgameImport::CG_ROFF_CACHE as c_int {
        cl.theROFFSystem
            .Cache(vma(1) as *mut c_char, qboolean::qtrue)
    } else if op == MpCgameImport::CG_ROFF_PLAY as c_int {
        cl.theROFFSystem.Play(
            arg(1),
            arg(2),
            unsafe { core::mem::transmute(arg(3)) },
            qboolean::qtrue,
        )
    } else if op == MpCgameImport::CG_ROFF_PURGE_ENT as c_int {
        cl.theROFFSystem.PurgeEnt(arg(1), qboolean::qtrue)
    } else if op == MpCgameImport::CG_TRUEMALLOC as c_int {
        VM_Shifted_Alloc(host, vma(1) as *mut *mut (), arg(2));
        0
    } else if op == MpCgameImport::CG_TRUEFREE as c_int {
        VM_Shifted_Free(common, vma(1) as *mut *mut ());
        0
    } else if op == MpCgameImport::CG_G2_LISTSURFACES as c_int {
        unsafe { G2API_ListSurfaces(arg(1) as *mut CGhoul2Info) };
        0
    } else if op == MpCgameImport::CG_G2_LISTBONES as c_int {
        unsafe { G2API_ListBones(arg(1) as *mut CGhoul2Info, arg(2)) };
        0
    } else if op == MpCgameImport::CG_G2_HAVEWEGHOULMODELS as c_int {
        unsafe { G2API_HaveWeGhoul2Models(&*(arg(1) as *const CGhoul2Info_v)) }
    } else if op == MpCgameImport::CG_G2_SETMODELS as c_int {
        unsafe {
            G2API_SetGhoul2ModelIndexes(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *mut qhandle_t,
                vma(3) as *mut qhandle_t,
            );
        }
        0
    } else if op == MpCgameImport::CG_G2_GETBOLT as c_int {
        unsafe {
            G2API_GetBoltMatrix(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                arg(3),
                vma(4) as *mut mdxaBone_t,
                vma(5) as *const f32,
                vma(6) as *const f32,
                arg(7),
                vma(8) as *mut qhandle_t,
                vma(9) as *mut f32,
            )
        }
    } else if op == MpCgameImport::CG_G2_GETBOLT_NOREC as c_int {
        //TODO: Port gG2_GBMNoReconstruct
        // Source: oracle/codemp/client/../ghoul2/G2_local.h:211
        g2.gG2_GBMNoReconstruct = qboolean::qtrue;
        unsafe {
            G2API_GetBoltMatrix(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                arg(3),
                vma(4) as *mut mdxaBone_t,
                vma(5) as *const f32,
                vma(6) as *const f32,
                arg(7),
                vma(8) as *mut qhandle_t,
                vma(9) as *mut f32,
            )
        }
    } else if op == MpCgameImport::CG_G2_GETBOLT_NOREC_NOROT as c_int {
        // gG2_GBMNoReconstruct = qtrue; // Yeah, this was probably BAD.
        //TODO: Port gG2_GBMUseSPMethod
        // Source: oracle/codemp/client/../ghoul2/G2_local.h:212
        g2.gG2_GBMUseSPMethod = qboolean::qtrue;
        unsafe {
            G2API_GetBoltMatrix(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                arg(3),
                vma(4) as *mut mdxaBone_t,
                vma(5) as *const f32,
                vma(6) as *const f32,
                arg(7),
                vma(8) as *mut qhandle_t,
                vma(9) as *mut f32,
            )
        }
    } else if op == MpCgameImport::CG_G2_INITGHOUL2MODEL as c_int {
        unsafe {
            G2API_InitGhoul2Model(
                vma(1) as *mut *mut CGhoul2Info_v,
                vma(2) as *const c_char,
                arg(3),
                arg(4) as qhandle_t,
                arg(5) as qhandle_t,
                arg(6),
                arg(7),
            )
        }
    } else if op == MpCgameImport::CG_G2_SETSKIN as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            let modelIndex = arg(2) as usize;
            G2API_SetSkin(&mut g2v[modelIndex], arg(3), arg(4))
        }
    } else if op == MpCgameImport::CG_G2_COLLISIONDETECT as c_int {
        unsafe {
            G2API_CollisionDetect(
                vma(1) as *mut CollisionRecord_t,
                &*(arg(2) as *const CGhoul2Info_v),
                vma(3) as *const f32,
                vma(4) as *const f32,
                arg(5),
                arg(6),
                vma(7) as *mut f32,
                vma(8) as *mut f32,
                vma(9) as *mut f32,
                //TODO: Port G2VertSpaceClient
                // Source: oracle/codemp/client/cl_cgame.cpp:45
                cl.G2VertSpaceClient,
                arg(10),
                arg(11),
                vmf(12),
            );
        }
        0
    } else if op == MpCgameImport::CG_G2_COLLISIONDETECTCACHE as c_int {
        unsafe {
            G2API_CollisionDetectCache(
                vma(1) as *mut CollisionRecord_t,
                &*(arg(2) as *const CGhoul2Info_v),
                vma(3) as *const f32,
                vma(4) as *const f32,
                arg(5),
                arg(6),
                vma(7) as *mut f32,
                vma(8) as *mut f32,
                vma(9) as *mut f32,
                cl.G2VertSpaceClient,
                arg(10),
                arg(11),
                vmf(12),
            );
        }
        0
    } else if op == MpCgameImport::CG_G2_ANGLEOVERRIDE as c_int {
        unsafe {
            G2API_SetBoneAngles(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *const c_char,
                vma(4) as *mut f32,
                arg(5),
                core::mem::transmute(arg(6)),
                core::mem::transmute(arg(7)),
                core::mem::transmute(arg(8)),
                vma(9) as *mut qhandle_t,
                arg(10),
                arg(11),
            )
        }
    } else if op == MpCgameImport::CG_G2_CLEANMODELS as c_int {
        unsafe { G2API_CleanGhoul2Models(vma(1) as *mut *mut CGhoul2Info_v) };
        0
    } else if op == MpCgameImport::CG_G2_PLAYANIM as c_int {
        unsafe {
            G2API_SetBoneAnim(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *const c_char,
                arg(4),
                arg(5),
                arg(6),
                vmf(7),
                arg(8),
                vmf(9),
                arg(10),
            )
        }
    } else if op == MpCgameImport::CG_G2_GETBONEANIM as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            let modelIndex = arg(10) as usize;
            G2API_GetBoneAnim(
                &mut g2v[modelIndex],
                vma(2) as *const c_char,
                arg(3),
                vma(4) as *mut f32,
                vma(5) as *mut c_int,
                vma(6) as *mut c_int,
                vma(7) as *mut c_int,
                vma(8) as *mut f32,
                vma(9) as *mut c_int,
            )
        }
    } else if op == MpCgameImport::CG_G2_GETBONEFRAME as c_int {
        // rwwFIXMEFIXME: Just make a G2API_GetBoneFrame func too. This is dirty.
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            let modelIndex = arg(6) as usize;
            let mut iDontCare1: c_int = 0;
            let mut iDontCare2: c_int = 0;
            let mut iDontCare3: c_int = 0;
            let mut fDontCare1: f32 = 0.0;

            G2API_GetBoneAnim(
                &mut g2v[modelIndex],
                vma(2) as *const c_char,
                arg(3),
                vma(4) as *mut f32,
                &mut iDontCare1,
                &mut iDontCare2,
                &mut iDontCare3,
                &mut fDontCare1,
                vma(5) as *mut c_int,
            )
        }
    } else if op == MpCgameImport::CG_G2_GETGLANAME as c_int {
        unsafe {
            let point = vma(3) as *mut c_char;
            let local = G2API_GetGLAName(&*(arg(1) as *const CGhoul2Info_v), arg(2));
            if !local.is_null() {
                libc::strcpy(point, local);
            }
        }
        0
    } else if op == MpCgameImport::CG_G2_COPYGHOUL2INSTANCE as c_int {
        unsafe {
            G2API_CopyGhoul2Instance(
                &*(arg(1) as *const CGhoul2Info_v),
                &*(arg(2) as *const CGhoul2Info_v),
                arg(3),
            )
        }
    } else if op == MpCgameImport::CG_G2_COPYSPECIFICGHOUL2MODEL as c_int {
        unsafe {
            G2API_CopySpecificG2Model(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                &*(arg(3) as *const CGhoul2Info_v),
                arg(4),
            );
        }
        0
    } else if op == MpCgameImport::CG_G2_DUPLICATEGHOUL2INSTANCE as c_int {
        unsafe {
            G2API_DuplicateGhoul2Instance(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *mut *mut CGhoul2Info_v,
            )
        };
        0
    } else if op == MpCgameImport::CG_G2_HASGHOUL2MODELONINDEX as c_int {
        unsafe { G2API_HasGhoul2ModelOnIndex(vma(1) as *mut *mut CGhoul2Info_v, arg(2)) }
    } else if op == MpCgameImport::CG_G2_REMOVEGHOUL2MODEL as c_int {
        unsafe { G2API_RemoveGhoul2Model(vma(1) as *mut *mut CGhoul2Info_v, arg(2)) }
    } else if op == MpCgameImport::CG_G2_SKINLESSMODEL as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            G2API_SkinlessModel(&mut g2v[arg(2) as usize])
        }
    } else if op == MpCgameImport::CG_G2_GETNUMGOREMARKS as c_int {
        // Raven gates this on `_G2_GORE`, undefined in this build; the oracle falls through
        // to the trailing `return 0`.
        0
    } else if op == MpCgameImport::CG_G2_ADDSKINGORE as c_int {
        // Raven gates this on `_G2_GORE`, undefined in this build.
        0
    } else if op == MpCgameImport::CG_G2_CLEARSKINGORE as c_int {
        // Raven gates this on `_G2_GORE`, undefined in this build.
        0
    } else if op == MpCgameImport::CG_G2_SIZE as c_int {
        unsafe { G2API_Ghoul2Size(&*(arg(1) as *const CGhoul2Info_v)) }
    } else if op == MpCgameImport::CG_G2_ADDBOLT as c_int {
        unsafe {
            G2API_AddBolt(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *const c_char,
            )
        }
    } else if op == MpCgameImport::CG_G2_ATTACHENT as c_int {
        // G2API_AttachEnt(int *boltInfo, CGhoul2Info *ghlInfoTo, int toBoltIndex, int entNum, int toModelNum)
        unsafe {
            let g2v = &mut *(arg(2) as *mut CGhoul2Info_v);
            G2API_AttachEnt(vma(1) as *mut c_int, &mut g2v[0], arg(3), arg(4), arg(5)) as c_int
        }
    } else if op == MpCgameImport::CG_G2_SETBOLTON as c_int {
        unsafe { G2API_SetBoltInfo(&*(arg(1) as *const CGhoul2Info_v), arg(2), arg(3)) };
        0
    } else if op == MpCgameImport::CG_G2_SETROOTSURFACE as c_int {
        unsafe {
            G2API_SetRootSurface(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *const c_char,
            )
        }
    } else if op == MpCgameImport::CG_G2_SETSURFACEONOFF as c_int {
        unsafe {
            G2API_SetSurfaceOnOff(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                arg(3),
            )
        }
    } else if op == MpCgameImport::CG_G2_SETNEWORIGIN as c_int {
        unsafe { G2API_SetNewOrigin(&*(arg(1) as *const CGhoul2Info_v), arg(2)) }
    } else if op == MpCgameImport::CG_G2_DOESBONEEXIST as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            G2API_DoesBoneExist(&mut g2v[arg(2) as usize], vma(3) as *const c_char)
        }
    } else if op == MpCgameImport::CG_G2_GETSURFACERENDERSTATUS as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            G2API_GetSurfaceRenderStatus(&mut g2v[arg(2) as usize], vma(3) as *const c_char)
        }
    } else if op == MpCgameImport::CG_G2_GETTIME as c_int {
        unsafe { G2API_GetTime(0) }
    } else if op == MpCgameImport::CG_G2_SETTIME as c_int {
        unsafe { G2API_SetTime(arg(1), arg(2)) };
        0
    } else if op == MpCgameImport::CG_G2_ABSURDSMOOTHING as c_int {
        unsafe {
            G2API_AbsurdSmoothing(
                &*(arg(1) as *const CGhoul2Info_v),
                core::mem::transmute(arg(2)),
            )
        };
        0
    } else if op == MpCgameImport::CG_G2_SETRAGDOLL as c_int {
        // Converts the info in the shared structure over to the class-based version.
        unsafe {
            let rdParamst = vma(2) as *mut sharedRagDollParams_t;
            if rdParamst.is_null() {
                G2API_ResetRagDoll(&*(arg(1) as *const CGhoul2Info_v));
                return 0;
            }

            let mut rdParams = CRagDollParams::default();
            rdParams.angles = (*rdParamst).angles;
            rdParams.position = (*rdParamst).position;
            rdParams.scale = (*rdParamst).scale;
            rdParams.pelvisAnglesOffset = (*rdParamst).pelvisAnglesOffset;
            rdParams.pelvisPositionOffset = (*rdParamst).pelvisPositionOffset;

            rdParams.fImpactStrength = (*rdParamst).fImpactStrength;
            rdParams.fShotStrength = (*rdParamst).fShotStrength;
            rdParams.me = (*rdParamst).me;

            rdParams.startFrame = (*rdParamst).startFrame;
            rdParams.endFrame = (*rdParamst).endFrame;

            rdParams.collisionType = (*rdParamst).collisionType;
            rdParams.CallRagDollBegin = (*rdParamst).CallRagDollBegin;

            // PORT-NOTE(rosetta-gap): `ERagPhase`/`ERagEffector` are SP-tier enums this
            // MP-tier module cannot import (layering); the numeric cast Raven itself
            // performs is preserved as a raw field assignment.
            rdParams.RagPhase = (*rdParamst).RagPhase;
            rdParams.effectorsToTurnOff = (*rdParamst).effectorsToTurnOff;

            G2API_SetRagDoll(&*(arg(1) as *const CGhoul2Info_v), &rdParams);
        }
        0
    } else if op == MpCgameImport::CG_G2_ANIMATEG2MODELS as c_int {
        unsafe {
            let rduParamst = vma(3) as *mut sharedRagDollUpdateParams_t;
            if rduParamst.is_null() {
                return 0;
            }

            let mut rduParams = CRagDollUpdateParams::default();
            rduParams.angles = (*rduParamst).angles;
            rduParams.position = (*rduParamst).position;
            rduParams.scale = (*rduParamst).scale;
            rduParams.velocity = (*rduParamst).velocity;

            rduParams.me = (*rduParamst).me;
            rduParams.settleFrame = (*rduParamst).settleFrame;

            G2API_AnimateG2Models(&*(arg(1) as *const CGhoul2Info_v), arg(2), &rduParams);
        }
        0
    } else if op == MpCgameImport::CG_G2_RAGPCJCONSTRAINT as c_int {
        unsafe {
            G2API_RagPCJConstraint(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                vma(3) as *mut f32,
                vma(4) as *mut f32,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGPCJGRADIENTSPEED as c_int {
        unsafe {
            G2API_RagPCJGradientSpeed(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                vmf(3),
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGEFFECTORGOAL as c_int {
        unsafe {
            G2API_RagEffectorGoal(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                vma(3) as *mut f32,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_GETRAGBONEPOS as c_int {
        unsafe {
            G2API_GetRagBonePos(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                vma(3) as *mut f32,
                vma(4) as *mut f32,
                vma(5) as *mut f32,
                vma(6) as *mut f32,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGEFFECTORKICK as c_int {
        unsafe {
            G2API_RagEffectorKick(
                &*(arg(1) as *const CGhoul2Info_v),
                vma(2) as *const c_char,
                vma(3) as *mut f32,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGFORCESOLVE as c_int {
        unsafe {
            G2API_RagForceSolve(
                &*(arg(1) as *const CGhoul2Info_v),
                core::mem::transmute(arg(2)),
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_SETBONEIKSTATE as c_int {
        unsafe {
            G2API_SetBoneIKState(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *const c_char,
                arg(4),
                vma(5) as *mut sharedSetBoneIKStateParams_t,
            )
        }
    } else if op == MpCgameImport::CG_G2_IKMOVE as c_int {
        unsafe {
            G2API_IKMove(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                vma(3) as *mut sharedIKMoveParams_t,
            )
        }
    } else if op == MpCgameImport::CG_G2_REMOVEBONE as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            G2API_RemoveBone(&mut g2v[arg(3) as usize], vma(2) as *const c_char)
        }
    } else if op == MpCgameImport::CG_G2_ATTACHINSTANCETOENTNUM as c_int {
        unsafe {
            G2API_AttachInstanceToEntNum(
                &*(arg(1) as *const CGhoul2Info_v),
                arg(2),
                core::mem::transmute(arg(3)),
            )
        };
        0
    } else if op == MpCgameImport::CG_G2_CLEARATTACHEDINSTANCE as c_int {
        unsafe { G2API_ClearAttachedInstance(arg(1)) };
        0
    } else if op == MpCgameImport::CG_G2_CLEANENTATTACHMENTS as c_int {
        unsafe { G2API_CleanEntAttachments() };
        0
    } else if op == MpCgameImport::CG_G2_OVERRIDESERVER as c_int {
        unsafe {
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            G2API_OverrideServerWithClientData(&mut g2v[0])
        }
    } else if op == MpCgameImport::CG_G2_GETSURFACENAME as c_int {
        // Since returning a pointer in such a way to a VM seems to cause reliability
        // problems, we shove data into the pointer the vm passes instead.
        unsafe {
            let point = vma(4) as *mut c_char;
            let modelindex = arg(3) as usize;
            let g2v = &mut *(arg(1) as *mut CGhoul2Info_v);
            let local = G2API_GetSurfaceName(&mut g2v[modelindex], arg(2));
            if !local.is_null() {
                libc::strcpy(point, local);
            }
        }
        0
    } else if op == MpCgameImport::CG_SP_GETSTRINGTEXTSTRING as c_int {
        unsafe {
            let key = core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned();
            let text = SE_GetString(host, &key);
            if !text.is_empty() {
                let dest = core::slice::from_raw_parts_mut(vma(2) as *mut c_char, arg(3) as usize);
                Q_strncpyz(dest, &text, arg(3) as usize);
                qboolean::qtrue as c_int
            } else {
                Com_sprintf(vma(2) as *mut c_char, arg(3), &format!("??{}", key));
                qboolean::qfalse as c_int
            }
        }
    } else if op == MpCgameImport::CG_SET_SHARED_BUFFER as c_int {
        cl.cl.mSharedMemory = vma(1) as *mut c_char;
        0
    } else if op == MpCgameImport::CG_CM_REGISTER_TERRAIN as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        //TODO: Port CM_RegisterTerrain
        unsafe { (*CM_RegisterTerrain(cm, &name, false)).GetTerrainId() }
    } else if op == MpCgameImport::CG_RMG_INIT as c_int {
        //TODO: Port TheRandomMissionManager / cmg / CRMManager
        // Source: oracle/codemp/client/../RMG/RM_Manager.h:60, .../qcommon/cm_local.h:220
        if unsafe { (*common.com_sv_running).integer } == 0 {
            // Don't do this if we are connected locally.
            if rmg.TheRandomMissionManager.is_null() {
                rmg.TheRandomMissionManager =
                    unsafe { Box::into_raw(Box::new(CRMManager::default())) };
            }
            unsafe {
                (*rmg.TheRandomMissionManager).SetLandScape(cm.cmg.landScape);
                if (*rmg.TheRandomMissionManager).LoadMission(qboolean::qfalse) != qboolean::qfalse
                {
                    if (*rmg.TheRandomMissionManager).SpawnMission(qboolean::qfalse)
                        == qboolean::qfalse
                    {
                        com_error(
                            errorParm_t::ERR_DROP,
                            "Error spawning mission for terrain".to_string(),
                        );
                    }
                }
                (*cm.cmg.landScape).UpdatePatches();
            }
        }
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(2) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        RM_CreateRandomModels(rmg, arg(1), &name);
        0
    } else if op == MpCgameImport::CG_RE_INIT_RENDERER_TERRAIN as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        RE_InitRendererTerrain(common, &name);
        0
    } else if op == MpCgameImport::CG_R_WEATHER_CONTENTS_OVERRIDE as c_int {
        // contentOverride = args[1]; (dead in the oracle)
        0
    } else if op == MpCgameImport::CG_R_WORLDEFFECTCOMMAND as c_int {
        let name = unsafe {
            core::ffi::CStr::from_ptr(vma(1) as *const c_char)
                .to_string_lossy()
                .into_owned()
        };
        R_WorldEffectCommand(rm, &name);
        0
    } else if op == MpCgameImport::CG_WE_ADDWEATHERZONE as c_int {
        R_AddWeatherZone(rm, unsafe { *(vma(1) as *const vec3_t) }, unsafe {
            *(vma(2) as *const vec3_t)
        });
        0
    } else {
        com_error(
            errorParm_t::ERR_DROP,
            format!("Bad cgame system trap: {}", op),
        );
    }
}
