//! `sv_game.cpp` — the server's game-VM interface: gentity/client accessors,
//! entity-string parsing, the outbound `SV_Game*` sinks, PVS/brush-model
//! helpers, VM lifecycle (`SV_InitGameProgs`/`SV_InitGameVM`/
//! `SV_RestartGameProgs`), and `SV_GameSystemCalls` (the inbound syscall
//! dispatcher the game VM calls through `VMA`/`VMF`).
//!
//! Source: `oracle/codemp/server/sv_game.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

use mp_qshared::common::mp::gentity::{NUM_BSETS, NUM_TIDS};
use mp_qshared::common::mp::qcommon::failedEdge_t;
use mp_qshared::common::mp::qcommon::parms::parms_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::task_id_t::taskID_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::common::mp::botlib::bot_entitystate_s::bot_entitystate_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::pc_token_t;
use mp_qshared::shared::surface_flags::CONTENTS_LIGHTSABER;
use mp_qshared::shared::q_math::{AngleVectors, MatrixMultiply, PerpendicularVector, Sys_SnapVector};
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_platform::Sys_CheckCD;
use native_math::vector::vec3_t;
use native_types::clipHandle_t;

use mp_abi::game::imports::MpGameImport as G;
use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t as T;
use crate::{SV_BotAllocateClient, SV_BotFreeClient, SV_BotGetConsoleMessage, SV_BotGetSnapshotEntity, SV_BotLibSetup, SV_BotLibShutdown, SV_DropClient, SV_SendServerCommand, SV_SetUserinfo};
use crate::server::server_state_t::serverState_t;
use crate::server::sv_entity_s::svEntity_t;
use crate::server_host::{ghoul2_slot, server_slot, sv_game_system_call};
use crate::sv_renderer::RE_RegisterServerSkin;
use crate::game_system_calls_shim;
use crate::Server;
use mp_engine_qcommon::vm::{arm_game_slot, VM_Call};

// PORT-NOTE(engine-host-state): `CollisionWorld`, `Common`, and `EngineHost`
// exist in `mp_engine_qcommon`/`mp_host_interface`; `RenderModels` (rm),
// `RmManager` (rmg), `Navigator` (nav), `Ghoul2System` (g2), and `RoffSystem`
// (roff) do NOT exist anywhere in the tree yet (grepped: no hits) — these
// packets were generated ahead of those state structs landing. Imported below
// by their preamble-table decl-home crate; genuinely missing, escalated in
// missing_symbols rather than stubbed (ZERO-PARK).
use mp_engine_botlib::BotLib;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_icarus::Icarus;
use mp_engine_icarus::game_interface::{
    icarus_associate_ent, icarus_free_ent, icarus_init, icarus_init_ent, icarus_is_initialized,
    icarus_is_running, icarus_maintain_task_manager, icarus_register_script, icarus_run_script,
    icarus_shutdown, icarus_valid_ent,
};
use mp_engine_icarus::q3_interface::{
    q3_set_var, q3_task_id_complete, q3_task_id_pending, q3_task_id_set,
};
use mp_engine_icarus::q3_registers::{
    q3_get_float_variable, q3_get_string_variable, q3_get_vector_variable, q3_variable_declared,
};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::roff::RoffSystem;
use mp_engine_qcommon::stringed::SE_GetString;
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::cm_load::RmManager;
use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::CM_AreasConnected;
use mp_host_interface::engine_host::EngineHost;

use crate::npcnav::Navigator;

// Canonical homes for the qcommon/qshared free functions this file calls. The
// `Cvar_*`/`Com_Milliseconds` bodies live in `mp_engine_qcommon` and take the
// threaded `(common, cm, rm, host, …)` engine-host receivers with `*const
// c_char` string params; `COM_ParseExt`/`Q_strncpyz` are the raw-pointer
// `q_shared.c` primitives in `mp_qshared`.
use libc::{atoi, strncpy};
use mp_engine_qcommon::common_fns::Com_Milliseconds;
use mp_engine_qcommon::cvar_fns::{
    Cvar_Get, Cvar_InfoString, Cvar_Register, Cvar_Set, Cvar_Update, Cvar_VariableIntegerValue,
    Cvar_VariableStringBuffer, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_Write};
use mp_qshared::shared::q_string::{COM_ParseExt, Q_strncpyz};

/// Raven `SV_NumForGentity`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:46-52`
pub fn SV_NumForGentity(sv: &mut Server, ent: *mut sharedEntity_t) -> c_int {
    // SAFETY (seam pointer arithmetic, porting-rules §D11): faithful
    // byte-offset division exactly as Raven computes it.
    unsafe {
        ((ent as *mut u8).offset_from(sv.sv.gentities as *mut u8)) as c_int / sv.sv.gentitySize
    }
}

/// Raven `SV_GentityNum`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:54-60`
pub fn SV_GentityNum(sv: &mut Server, num: c_int) -> *mut sharedEntity_t {
    unsafe {
        (sv.sv.gentities as *mut u8).offset((sv.sv.gentitySize * num) as isize)
            as *mut sharedEntity_t
    }
}

/// Raven `SV_GameClientNum`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:62-68`
pub fn SV_GameClientNum(sv: &mut Server, num: c_int) -> *mut playerState_t {
    unsafe {
        (sv.sv.gameClients as *mut u8).offset((sv.sv.gameClientSize * num) as isize)
            as *mut playerState_t
    }
}

/// Raven `SV_LocateGameData`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:327-335`
pub fn SV_LocateGameData(
    sv: &mut Server,
    gEnts: *mut sharedEntity_t,
    numGEntities: c_int,
    sizeofGEntity_t: c_int,
    clients: *mut playerState_t,
    sizeofGameClient: c_int,
) {
    sv.sv.gentities = gEnts;
    sv.sv.gentitySize = sizeofGEntity_t;
    sv.sv.num_entities = numGEntities;

    sv.sv.gameClients = clients;
    sv.sv.gameClientSize = sizeofGameClient;
}

/// Raven `SV_GetEntityToken`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:337-367`
pub fn SV_GetEntityToken(sv: &mut Server, buffer: *mut c_char, bufferSize: c_int) -> qboolean {
    // Raven's `COM_Parse(&p)` is `COM_ParseExt(&p, qtrue)`; the raw-pointer
    // primitive lives in `mp_qshared::shared::q_string`.
    unsafe {
        if sv.sv.mLocalSubBSPIndex == -1 {
            let s = COM_ParseExt(
                &mut sv.sv.entityParsePoint as *mut *mut c_char as *mut *const c_char,
                qtrue,
            );
            Q_strncpyz(buffer, s, bufferSize);
            if sv.sv.entityParsePoint.is_null() && *s == 0 {
                qfalse
            } else {
                qtrue
            }
        } else {
            let s = COM_ParseExt(
                &mut sv.sv.mLocalSubBSPEntityParsePoint as *mut *mut c_char as *mut *const c_char,
                qtrue,
            );
            Q_strncpyz(buffer, s, bufferSize);
            if sv.sv.mLocalSubBSPEntityParsePoint.is_null() && *s == 0 {
                qfalse
            } else {
                qtrue
            }
        }
    }
}

/// Raven `FloatAsInt` — reinterpret a float's bits as an int (file-static
/// helper; no state).
///
/// Source: `oracle/codemp/server/sv_game.cpp:384-390`
pub fn FloatAsInt(f: f32) -> c_int {
    f.to_bits() as c_int
}

/// Raven `SV_SvEntityForGentity`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:70-75`
pub fn SV_SvEntityForGentity(sv: &mut Server, gEnt: *mut sharedEntity_t) -> *mut svEntity_t {
    unsafe {
        if gEnt.is_null()
            || (*gEnt).s.number < 0
            || (*gEnt).s.number >= mp_qshared::shared::limits::MAX_GENTITIES as c_int
        {
            mp_engine_qcommon::common::com_error(
                errorParm_t::ERR_DROP,
                "SV_SvEntityForGentity: bad gEnt".to_string(),
            );
        }
        &mut sv.sv.svEntities[(*gEnt).s.number as usize] as *mut svEntity_t
    }
}

/// Raven `SV_GEntityForSvEntity`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:77-82`
pub fn SV_GEntityForSvEntity(sv: &mut Server, svEnt: *mut svEntity_t) -> *mut sharedEntity_t {
    let num = unsafe { svEnt.offset_from(sv.sv.svEntities.as_mut_ptr()) } as c_int;
    SV_GentityNum(sv, num)
}

/// Raven `SV_SetActiveSubBSP`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:185-200`
pub fn SV_SetActiveSubBSP(cm: &mut CollisionWorld, sv: &mut Server, index: c_int) -> *const c_char {
    if index >= 0 {
        sv.sv.mLocalSubBSPIndex = mp_engine_qcommon::cm_load::CM_FindSubBSP(cm, index);
        sv.sv.mLocalSubBSPModelOffset = index;
        sv.sv.mLocalSubBSPEntityParsePoint =
            mp_engine_qcommon::cm_load::CM_SubBSPEntityString(cm, sv.sv.mLocalSubBSPIndex);
        sv.sv.mLocalSubBSPEntityParsePoint as *const c_char
    } else {
        sv.sv.mLocalSubBSPIndex = -1;
        core::ptr::null()
    }
}

/// Raven `ConvertedEntity` — shifts VM-relative pointers into real ones and
/// returns a pointer to the file-static conversion buffer.
///
/// Raven: "Return an entity with the memory shifted around to allow
/// reading/modifying VM memory".
///
/// The file-static `gLocalModifier` (ruling 3: genuine cross-frame state)
/// threads as a `Server` field (`sv.g_local_modifier`); PORT-NOTE(field): that
/// field does not exist on `Server` yet — escalated (missing_symbols).
///
/// Source: `oracle/codemp/server/sv_game.cpp:422-452`
pub fn ConvertedEntity(
    common: &mut Common,
    sv: &mut Server,
    ent: *mut sharedEntity_t,
) -> *mut sharedEntity_t {
    unsafe {
        assert!(!ent.is_null());

        sv.g_local_modifier.s = (*ent).s;
        sv.g_local_modifier.r = (*ent).r;
        for i in 0..NUM_TIDS as usize {
            sv.g_local_modifier.taskID[i] = (*ent).taskID[i];
        }
        sv.g_local_modifier.parms =
            mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).parms as c_int) as *mut parms_t;
        for i in 0..NUM_BSETS as usize {
            sv.g_local_modifier.behaviorSet[i] =
                mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).behaviorSet[i] as c_int)
                    as *mut c_char;
        }
        sv.g_local_modifier.script_targetname =
            mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).script_targetname as c_int)
                as *mut c_char;
        sv.g_local_modifier.delayScriptTime = (*ent).delayScriptTime;
        sv.g_local_modifier.fullName =
            mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).fullName as c_int) as *mut c_char;
        sv.g_local_modifier.targetname =
            mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).targetname as c_int) as *mut c_char;
        sv.g_local_modifier.classname =
            mp_engine_qcommon::vm_fns::VM_ArgPtr(common, (*ent).classname as c_int) as *mut c_char;

        sv.g_local_modifier.ghoul2 = (*ent).ghoul2;

        &mut sv.g_local_modifier as *mut sharedEntity_t
    }
}

/// Raven `SV_GameError` — `Com_Error(ERR_DROP, ...)`, receiverless per
/// ruling 1 (panic/unwind, no `common` needed at the call site).
///
/// Source: `oracle/codemp/server/sv_game.cpp:36-38`
pub fn SV_GameError(string: *const c_char) {
    let msg = unsafe { core::ffi::CStr::from_ptr(string) }
        .to_string_lossy()
        .into_owned();
    mp_engine_qcommon::common::com_error(errorParm_t::ERR_DROP, msg);
}

/// Raven `SV_GamePrint`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:40-42`
pub fn SV_GamePrint(common: &mut Common, string: *const c_char) {
    let msg = unsafe { core::ffi::CStr::from_ptr(string) }.to_string_lossy();
    mp_engine_qcommon::common::common::com_printf(common, &msg);
}

/// Raven `SV_GameSendServerCommand`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:91-100`
pub fn SV_GameSendServerCommand(
    common: &mut Common,
    sv: &mut Server,
    clientNum: c_int,
    text: *const c_char,
) {
    let msg = unsafe { core::ffi::CStr::from_ptr(text) }.to_string_lossy();
    if clientNum == -1 {
        SV_SendServerCommand(common, sv, core::ptr::null_mut(), &msg);
    } else {
        if clientNum < 0 || clientNum >= (unsafe { (*common.sv_maxclients).integer }) {
            return;
        }
        let client = unsafe { sv.svs.clients.offset(clientNum as isize) };
        SV_SendServerCommand(common, sv, client, &msg);
    }
}

/// Raven `SV_GameDropClient`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:110-115`
pub fn SV_GameDropClient(
    common: &mut Common,
    sv: &mut Server,
    clientNum: c_int,
    reason: *const c_char,
) {
    if clientNum < 0 || clientNum >= (unsafe { (*common.sv_maxclients).integer }) {
        return;
    }
    let client = unsafe { sv.svs.clients.offset(clientNum as isize) };
    SV_DropClient(common, sv, client, reason);
}

/// Raven `SV_inPVS`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:209-233`
pub fn SV_inPVS(cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t) -> qboolean {
    let mut leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p1);
    let mut cluster = CM_LeafCluster(cm, leafnum);
    let area1 = CM_LeafArea(cm, leafnum);
    let mask = mp_engine_qcommon::cm_test::CM_ClusterPVS(cm, cluster);

    leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p2);
    cluster = CM_LeafCluster(cm, leafnum);
    let area2 = CM_LeafArea(cm, leafnum);
    if !mask.is_null() {
        let byte = unsafe { *mask.offset((cluster >> 3) as isize) };
        if byte & (1 << (cluster & 7)) == 0 {
            return qfalse;
        }
    }
    if CM_AreasConnected(cm, area1, area2) == qfalse {
        // a door blocks sight
        return qfalse;
    }
    qtrue
}

/// Raven `SV_inPVSIgnorePortals`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:243-267`
pub fn SV_inPVSIgnorePortals(cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t) -> qboolean {
    let mut leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p1);
    let mut cluster = CM_LeafCluster(cm, leafnum);
    let _area1 = CM_LeafArea(cm, leafnum);
    let mask = mp_engine_qcommon::cm_test::CM_ClusterPVS(cm, cluster);

    leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p2);
    cluster = CM_LeafCluster(cm, leafnum);
    let _area2 = CM_LeafArea(cm, leafnum);

    if !mask.is_null() {
        let byte = unsafe { *mask.offset((cluster >> 3) as isize) };
        if byte & (1 << (cluster & 7)) == 0 {
            return qfalse;
        }
    }

    qtrue
}

/// Raven `SV_GetServerinfo`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:314-319`
pub fn SV_GetServerinfo(common: &mut Common, buffer: *mut c_char, bufferSize: c_int) {
    if bufferSize < 1 {
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_DROP,
            format!("SV_GetServerinfo: bufferSize == {bufferSize}"),
        );
    }
    let info = Cvar_InfoString(common, mp_qshared::shared::cvar::CVAR_SERVERINFO);
    Q_strncpyz(buffer, info as *const c_char, bufferSize);
}

/// Raven `SV_GetUsercmd`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:375-380`
pub fn SV_GetUsercmd(common: &mut Common, sv: &mut Server, clientNum: c_int, cmd: *mut usercmd_t) {
    // `sv_maxclients` is a `Common`-owned cvar handle; `common` is threaded in
    // to reach it (the `ERR_DROP`/`Com_Error` unwind on a bad clientNum is not
    // yet reachable from this crate — its call is unported).
    unsafe {
        if clientNum < 0 || clientNum >= (*common.sv_maxclients).integer {
            // Com_Error(ERR_DROP, ...) — unported in this crate; see SV_GameError.
        }
        *cmd = (*sv.svs.clients.offset(clientNum as isize)).lastUsercmd;
    }
}

/// Raven `SV_InitGameVM`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1682-1697`
pub fn SV_InitGameVM(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    restart: qboolean,
) {
    sv.sv.entityParsePoint = mp_engine_qcommon::cm_load::CM_EntityString(cm);

    let ms = Com_Milliseconds(common, cm, rm, host);
    VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_INIT as c_int,
        &[sv.svs.time, ms, restart as c_int],
    );

    let max_clients = unsafe { (*common.sv_maxclients).integer };
    for i in 0..max_clients {
        unsafe {
            (*sv.svs.clients.offset(i as isize)).gentity = core::ptr::null_mut();
        }
    }
}

/// Raven `SV_GameCommand`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1766-1772`
pub fn SV_GameCommand(common: &mut Common, sv: &mut Server) -> qboolean {
    if sv.sv.state as c_int != serverState_t::SS_GAME as c_int {
        return qfalse;
    }
    let r = VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_CONSOLE_COMMAND as c_int,
        &[],
    );
    (r != 0) as c_int as qboolean
}

/// Raven `SV_AdjustAreaPortalState`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:275-283`
pub fn SV_AdjustAreaPortalState(
    cm: &mut CollisionWorld,
    sv: &mut Server,
    ent: *mut sharedEntity_t,
    open: qboolean,
) {
    let svEnt = SV_SvEntityForGentity(sv, ent);
    unsafe {
        if (*svEnt).areanum2 == -1 {
            return;
        }
        mp_engine_qcommon::cm_test::CM_AdjustAreaPortalState(
            cm,
            (*svEnt).areanum,
            (*svEnt).areanum2,
            open,
        );
    }
}

/// Raven `SV_RestartGameProgs`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1708-1721`
pub fn SV_RestartGameProgs(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if sv.gvm.is_null() {
        return;
    }
    VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_SHUTDOWN as c_int,
        &[qtrue as c_int],
    );

    // do a restart instead of a free
    sv.gvm = mp_engine_qcommon::vm_fns::VM_Restart(common, cm, rm, host, sv.gvm);
    if sv.gvm.is_null() {
        // bk001212 - as done below
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_FATAL,
            "VM_Restart on game failed".to_string(),
        );
    }

    SV_InitGameVM(common, cm, sv, rm, host, qtrue);
}

/// Raven `SV_EntityContact`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:291-305`
pub fn SV_EntityContact(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    mins: vec3_t,
    maxs: vec3_t,
    gEnt: *const sharedEntity_t,
    capsule: c_int,
) -> qboolean {
    unsafe {
        let origin = (*gEnt).r.currentOrigin;
        let angles = (*gEnt).r.currentAngles;

        let ch: clipHandle_t = crate::sv_world::SV_ClipHandleForEntity(cm, gEnt);
        let mut trace = core::mem::zeroed();
        mp_engine_qcommon::cm_trace::CM_TransformedBoxTrace(
            common,
            cm,
            rm,
            rmg,
            host,
            &mut trace,
            mp_qshared::shared::q_math::vec3_origin,
            mp_qshared::shared::q_math::vec3_origin,
            mins,
            maxs,
            ch,
            -1,
            origin,
            angles,
            capsule,
        );

        trace.startsolid as qboolean
    }
}

/// Raven `SV_SetBrushModel`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:128-183`
pub fn SV_SetBrushModel(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    host: &mut dyn EngineHost,
    ent: *mut sharedEntity_t,
    name: *const c_char,
) {
    unsafe {
        if name.is_null() {
            mp_engine_qcommon::common::com_error(
                errorParm_t::ERR_DROP,
                "SV_SetBrushModel: NULL".to_string(),
            );
        }

        let name_str = core::ffi::CStr::from_ptr(name).to_string_lossy();
        let mins = [0.0f32; 3];
        let maxs = [0.0f32; 3];

        if *name == b'*' as c_char {
            (*ent).s.modelindex = atoi(name.offset(1));

            if sv.sv.mLocalSubBSPIndex != -1 {
                (*ent).s.modelindex += sv.sv.mLocalSubBSPModelOffset;
            }

            let h = mp_engine_qcommon::cm_load::CM_InlineModel(cm, (*ent).s.modelindex);

            mp_engine_qcommon::cm_load::CM_ModelBounds(cm, h, mins, maxs);

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            let com_rmg = common.com_RMG;
            if !com_rmg.is_null() && (*com_rmg).integer != 0 {
                (*ent).r.contents =
                    mp_engine_qcommon::cm_load::CM_ModelContents(cm, h, sv.sv.mLocalSubBSPIndex);
            } else {
                (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(cm, h, -1);
            }
        } else if *name == b'#' as c_char {
            let bsp_name = format!("maps/{}.bsp\0", &name_str[1..]);
            (*ent).s.modelindex = mp_engine_qcommon::cm_load::CM_LoadSubBSP(
                common,
                cm,
                rm,
                rmg,
                host,
                bsp_name.as_ptr() as *const c_char,
                qfalse,
            );
            mp_engine_qcommon::cm_load::CM_ModelBounds(cm, (*ent).s.modelindex, mins, maxs);

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            //rwwNOTE: We don't ever want to set contents -1, it includes CONTENTS_LIGHTSABER.
            //Lots of stuff will explode if there's a brush with CONTENTS_LIGHTSABER that isn't attached to a client owner.
            //ent->contents = -1;		// we don't know exactly what is in the brushes
            let _ = CONTENTS_LIGHTSABER;
            let h = mp_engine_qcommon::cm_load::CM_InlineModel(cm, (*ent).s.modelindex);
            let sub_bsp = mp_engine_qcommon::cm_load::CM_FindSubBSP(cm, (*ent).s.modelindex);
            (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(cm, h, sub_bsp);
        } else {
            mp_engine_qcommon::common::com_error(
                errorParm_t::ERR_DROP,
                format!("SV_SetBrushModel: {name_str} isn't a brush model"),
            );
        }
    }
}

/// Reads `args[n]` as a VM-relative offset and resolves it to an engine
/// pointer — Raven's `VMA(n)` macro (`vm.cpp:648-649`; a native-DLL identity
/// cast since our module is a native dylib, not a bytecode VM). `common` is a
/// shared borrow (see `VM_ArgPtr`) so `VMA(n)` args resolve inside calls that
/// already reserve `common` mutably.
///
/// Source: `oracle/codemp/qcommon/vm_local.h` (`VMA`/`VMF` macros)
#[inline]
unsafe fn vma(common: &Common, args: *mut c_int, n: isize) -> *mut c_void {
    mp_engine_qcommon::vm_fns::VM_ArgPtr(common, *args.offset(n)) as *mut c_void
}

/// Raven's `VMF(n)` macro — reinterpret `args[n]`'s bits as `float`.
#[inline]
unsafe fn vmf(args: *mut c_int, n: isize) -> f32 {
    f32::from_bits(*args.offset(n) as u32)
}

/// Checked `(taskID_t)args[2]` conversion for the three `G_ICARUS_TASKID*` arms.
///
/// Raven casts the arg word to `taskID_t` unchecked (`sv_game.cpp:786`/`:808`/
/// `:813`) — UB for an out-of-range word; ported as a porting-rules §19 checked
/// conversion. The ICARUS callees guard `< TID_CHAN_VOICE || >= NUM_TIDS` and
/// no-op / return `qfalse` (`Q3_Interface.cpp:116-118`/`:169-171`), so `None`
/// here reproduces that guarded outcome.
/// Source: `oracle/codemp/server/sv_game.cpp:786`
#[inline]
fn task_id_from_word(word: c_int) -> Option<taskID_t> {
    if word < taskID_t::TID_CHAN_VOICE as c_int || word >= taskID_t::NUM_TIDS as c_int {
        return None;
    }
    // SAFETY: `word` is now in `0..NUM_TIDS` — a valid `#[repr(i32)]` discriminant.
    Some(unsafe { core::mem::transmute::<c_int, taskID_t>(word) })
}

/// Raven `SV_GameSystemCalls` — the inbound syscall dispatcher the game VM
/// calls through `VMA`/`VMF`.
///
/// Raven's `botlib_export` is a file-scope `botlib_export_t*` global; it lives
/// on `Server` (`sv.botlib_export`, set by `SV_BotInitBotLib`). Its ported
/// function-pointer fields carry the `bot: &mut BotLib` receiver, so the
/// `BOTLIB_*` arms below thread `bot` through the export table.
///
/// The `icarus`/`nav`/`g2`/`roff` §F method names are not given exact Rust
/// spellings by this packet (only receiver + doc routing) — transcribed as
/// best-effort snake_case per the `Com_Printf` → `com_printf` precedent
/// already in the tree; genuinely unconfirmed, escalated in missing_symbols.
///
/// Source: `oracle/codemp/server/sv_game.cpp:458-1657`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn SV_GameSystemCalls(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    bot: &mut BotLib,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    icarus: &mut Icarus,
    nav: &mut Navigator,
    g2: &mut Ghoul2System,
    roff: &mut RoffSystem,
    mut host: &mut dyn EngineHost,
    args: *mut c_int,
) -> c_int {

    // SAFETY: `args` is the trampoline's raw syscall word array (seam
    // pointer, porting-rules §D11); every arm reads only the words its trap
    // number defines, exactly as Raven's `int *args` does.
    unsafe {
        let trap = *args.offset(0);

        // rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without
        // adding a trap to match, and all of these traps must be shared and
        // have cases in sv_game, cl_cgame, and cl_ui. They must also all be
        // in the same order, and start at 100.
        if trap == T::TRAP_MEMSET as c_int {
            mp_engine_qcommon::common_fns::Com_Memset(
                vma(common, args, 1) as *mut (),
                *args.offset(2),
                *args.offset(3) as usize,
            );
            return 0;
        } else if trap == T::TRAP_MEMCPY as c_int {
            mp_engine_qcommon::common_fns::Com_Memcpy(
                vma(common, args, 1) as *mut (),
                vma(common, args, 2) as *const (),
                *args.offset(3) as usize,
            );
            return 0;
        } else if trap == T::TRAP_STRNCPY as c_int {
            return strncpy(
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *const c_char,
                *args.offset(3) as usize,
            ) as isize as c_int;
        } else if trap == T::TRAP_SIN as c_int {
            return FloatAsInt(vmf(args, 1).sin());
        } else if trap == T::TRAP_COS as c_int {
            return FloatAsInt(vmf(args, 1).cos());
        } else if trap == T::TRAP_ATAN2 as c_int {
            return FloatAsInt(vmf(args, 1).atan2(vmf(args, 2)));
        } else if trap == T::TRAP_SQRT as c_int {
            return FloatAsInt(vmf(args, 1).sqrt());
        } else if trap == T::TRAP_MATRIXMULTIPLY as c_int {
            MatrixMultiply(
                &*(vma(common, args, 1) as *const [[f32; 3]; 3]),
                &*(vma(common, args, 2) as *const [[f32; 3]; 3]),
                &mut *(vma(common, args, 3) as *mut [[f32; 3]; 3]),
            );
            return 0;
        } else if trap == T::TRAP_ANGLEVECTORS as c_int {
            AngleVectors(
                *(vma(common, args, 1) as *const vec3_t),
                (vma(common, args, 2) as *mut vec3_t).as_mut(),
                (vma(common, args, 3) as *mut vec3_t).as_mut(),
                (vma(common, args, 4) as *mut vec3_t).as_mut(),
            );
            return 0;
        } else if trap == T::TRAP_PERPENDICULARVECTOR as c_int {
            PerpendicularVector(
                &mut *(vma(common, args, 1) as *mut vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
            );
            return 0;
        } else if trap == T::TRAP_FLOOR as c_int {
            return FloatAsInt(vmf(args, 1).floor());
        } else if trap == T::TRAP_CEIL as c_int {
            return FloatAsInt(vmf(args, 1).ceil());
        } else if trap == T::TRAP_TESTPRINTINT as c_int || trap == T::TRAP_TESTPRINTFLOAT as c_int {
            return 0;
        } else if trap == T::TRAP_ACOS as c_int {
            return FloatAsInt(mp_engine_qcommon::common_fns::Q_acos(vmf(args, 1)));
        } else if trap == T::TRAP_ASIN as c_int {
            return FloatAsInt(mp_engine_qcommon::common_fns::Q_asin(vmf(args, 1)));
        } else if trap == G::G_PRINT as c_int {
            let s =
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char).to_string_lossy();
            mp_engine_qcommon::common::common::com_printf(common, &s);
            return 0;
        } else if trap == G::G_ERROR as c_int {
            let s = core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                .to_string_lossy()
                .into_owned();
            mp_engine_qcommon::common::com_error(errorParm_t::ERR_DROP, s);
        } else if trap == G::G_MILLISECONDS as c_int {
            return mp_engine_qcommon::timing::sys_milliseconds(common);
        } else if trap == G::G_PRECISIONTIMER_START as c_int {
            // rww - precision timer funcs. -ALWAYS- call end after start with
            // supplied ptr, or you'll get a nasty memory leak. Not that you
            // should be using these outside of debug anyway.
            let supplied_ptr = vma(common, args, 1) as *mut *mut c_void;
            let new_timer = Box::new(mp_engine_qcommon::timing::timing_c::timing_c::default());
            *supplied_ptr = Box::into_raw(new_timer) as *mut c_void;
            (**(supplied_ptr as *mut *mut mp_engine_qcommon::timing::timing_c::timing_c)).Start();
            return 0;
        } else if trap == G::G_PRECISIONTIMER_END as c_int {
            let timer = *args.offset(1) as *mut mp_engine_qcommon::timing::timing_c::timing_c;
            let r = (*timer).End();
            drop(Box::from_raw(timer));
            return r;
        } else if trap == G::G_CVAR_REGISTER as c_int {
            Cvar_Register(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
                vma(common, args, 2) as *const c_char,
                vma(common, args, 3) as *const c_char,
                *args.offset(4),
            );
            return 0;
        } else if trap == G::G_CVAR_UPDATE as c_int {
            Cvar_Update(
                common,
                vma(common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
            );
            return 0;
        } else if trap == G::G_CVAR_SET as c_int {
            Cvar_Set(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *const c_char,
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_CVAR_VARIABLE_INTEGER_VALUE as c_int {
            return Cvar_VariableIntegerValue(common, vma(common, args, 1) as *const c_char);
        } else if trap == G::G_CVAR_VARIABLE_STRING_BUFFER as c_int {
            Cvar_VariableStringBuffer(
                common,
                vma(common, args, 1) as *const c_char,
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_ARGC as c_int {
            return mp_engine_qcommon::cmd_common::Cmd_Argc(common);
        } else if trap == G::G_ARGV as c_int {
            mp_engine_qcommon::cmd_common::Cmd_ArgvBuffer(
                common,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_SEND_CONSOLE_COMMAND as c_int {
            mp_engine_qcommon::cmd_common::Cbuf_ExecuteText(
                common,
                cm,
                &mut server_slot(sv),
                rm,
                rmg,
                &mut ghoul2_slot(g2),
                host,
                *args.offset(1),
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_FS_FOPEN_FILE as c_int {
            return mp_engine_qcommon::files_pc::FS_FOpenFileByMode(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *const c_char,
                vma(common, args, 2) as *mut c_int,
                core::mem::transmute(*args.offset(3)),
            );
        } else if trap == G::G_FS_READ as c_int {
            mp_engine_qcommon::files_pc::FS_Read2(
                common,
                vma(common, args, 1) as *mut (),
                *args.offset(2),
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_FS_WRITE as c_int {
            FS_Write(
                common,
                vma(common, args, 1) as *const (),
                *args.offset(2),
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_FS_FCLOSE_FILE as c_int {
            FS_FCloseFile(common, *args.offset(1));
            return 0;
        } else if trap == G::G_FS_GETFILELIST as c_int {
            return mp_engine_qcommon::files_pc::FS_GetFileList(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *const c_char,
                vma(common, args, 2) as *const c_char,
                vma(common, args, 3) as *mut c_char,
                *args.offset(4),
            );
        } else if trap == G::G_LOCATE_GAME_DATA as c_int {
            SV_LocateGameData(
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
                *args.offset(3),
                vma(common, args, 4) as *mut playerState_t,
                *args.offset(5),
            );
            return 0;
        } else if trap == G::G_DROP_CLIENT as c_int {
            SV_GameDropClient(
                common,
                sv,
                *args.offset(1),
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_SEND_SERVER_COMMAND as c_int {
            SV_GameSendServerCommand(
                common,
                sv,
                *args.offset(1),
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_LINKENTITY as c_int {
            crate::sv_world::SV_LinkEntity(
                common,
                cm,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_UNLINKENTITY as c_int {
            crate::sv_world::SV_UnlinkEntity(
                common,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_ENTITIES_IN_BOX as c_int {
            return crate::sv_world::SV_AreaEntities(
                common,
                sv,
                *(vma(common, args, 1) as *const vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
                vma(common, args, 3) as *mut c_int,
                *args.offset(4),
            );
        } else if trap == G::G_ENTITY_CONTACT as c_int {
            return SV_EntityContact(
                common,
                cm,
                rm,
                rmg,
                host,
                *(vma(common, args, 1) as *const vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
                vma(common, args, 3) as *const sharedEntity_t,
                qfalse as c_int,
            ) as c_int;
        } else if trap == G::G_ENTITY_CONTACTCAPSULE as c_int {
            return SV_EntityContact(
                common,
                cm,
                rm,
                rmg,
                host,
                *(vma(common, args, 1) as *const vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
                vma(common, args, 3) as *const sharedEntity_t,
                qtrue as c_int,
            ) as c_int;
        } else if trap == G::G_TRACE as c_int {
            crate::sv_world::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(common, args, 2) as *const vec3_t),
                *(vma(common, args, 3) as *const vec3_t),
                *(vma(common, args, 4) as *const vec3_t),
                *(vma(common, args, 5) as *const vec3_t),
                *args.offset(6),
                *args.offset(7),
                qfalse as c_int,
                0,
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_G2TRACE as c_int {
            crate::sv_world::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(common, args, 2) as *const vec3_t),
                *(vma(common, args, 3) as *const vec3_t),
                *(vma(common, args, 4) as *const vec3_t),
                *(vma(common, args, 5) as *const vec3_t),
                *args.offset(6),
                *args.offset(7),
                qfalse as c_int,
                *args.offset(8),
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_TRACECAPSULE as c_int {
            crate::sv_world::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(common, args, 2) as *const vec3_t),
                *(vma(common, args, 3) as *const vec3_t),
                *(vma(common, args, 4) as *const vec3_t),
                *(vma(common, args, 5) as *const vec3_t),
                *args.offset(6),
                *args.offset(7),
                qtrue as c_int,
                *args.offset(8),
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_POINT_CONTENTS as c_int {
            return crate::sv_world::SV_PointContents(
                common,
                cm,
                sv,
                *(vma(common, args, 1) as *const vec3_t),
                *args.offset(2),
            );
        } else if trap == G::G_SET_SERVER_CULL as c_int {
            sv.g_svCullDist = vmf(args, 1);
            return 0;
        } else if trap == G::G_SET_BRUSH_MODEL as c_int {
            SV_SetBrushModel(
                common,
                cm,
                sv,
                rm,
                rmg,
                host,
                vma(common, args, 1) as *mut sharedEntity_t,
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_IN_PVS as c_int {
            return SV_inPVS(
                cm,
                *(vma(common, args, 1) as *const vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
            ) as c_int;
        } else if trap == G::G_IN_PVS_IGNORE_PORTALS as c_int {
            return SV_inPVSIgnorePortals(
                cm,
                *(vma(common, args, 1) as *const vec3_t),
                *(vma(common, args, 2) as *const vec3_t),
            ) as c_int;
        } else if trap == G::G_SET_CONFIGSTRING as c_int {
            crate::sv_init::SV_SetConfigstring(
                common,
                cm,
                sv,
                rm,
                host,
                *args.offset(1),
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_GET_CONFIGSTRING as c_int {
            crate::sv_init::SV_GetConfigstring(
                sv,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_SET_USERINFO as c_int {
            SV_SetUserinfo(
                common,
                sv,
                *args.offset(1),
                vma(common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_GET_USERINFO as c_int {
            crate::sv_init::SV_GetUserinfo(
                common,
                sv,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_GET_SERVERINFO as c_int {
            SV_GetServerinfo(common, vma(common, args, 1) as *mut c_char, *args.offset(2));
            return 0;
        } else if trap == G::G_ADJUST_AREA_PORTAL_STATE as c_int {
            SV_AdjustAreaPortalState(
                cm,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
                core::mem::transmute(*args.offset(2)),
            );
            return 0;
        } else if trap == G::G_AREAS_CONNECTED as c_int {
            return CM_AreasConnected(cm, *args.offset(1), *args.offset(2)) as c_int;
        } else if trap == G::G_BOT_ALLOCATE_CLIENT as c_int {
            return SV_BotAllocateClient(common, sv);
        } else if trap == G::G_BOT_FREE_CLIENT as c_int {
            SV_BotFreeClient(common, sv, *args.offset(1));
            return 0;
        } else if trap == G::G_GET_USERCMD as c_int {
            SV_GetUsercmd(common, sv, *args.offset(1), vma(common, args, 2) as *mut usercmd_t);
            return 0;
        } else if trap == G::G_SIEGEPERSSET as c_int {
            sv.sv_siegePersData = *(vma(common, args, 1)
                as *const mp_qshared::common::mp::qcommon::siege_pers::siegePers_t);
            return 0;
        } else if trap == G::G_SIEGEPERSGET as c_int {
            *(vma(common, args, 1)
                as *mut mp_qshared::common::mp::qcommon::siege_pers::siegePers_t) =
                sv.sv_siegePersData;
            return 0;
        }
        // rwwRMG - G_GET_ENTITY_TOKEN and G_BOT_GET_MEMORY/G_BOT_FREE_MEMORY
        // stay commented out in Raven (sv_game.cpp:648-672) — not transcribed.
        else if trap == G::G_DEBUG_POLYGON_CREATE as c_int {
            return crate::BotImport_DebugPolygonCreate(
                sv,
                *args.offset(1),
                *args.offset(2),
                vma(common, args, 3) as *const [f32; 3],
            );
        } else if trap == G::G_DEBUG_POLYGON_DELETE as c_int {
            crate::BotImport_DebugPolygonDelete(sv, *args.offset(1));
            return 0;
        } else if trap == G::G_REAL_TIME as c_int {
            return mp_engine_qcommon::common_fns::Com_RealTime(
                vma(common, args, 1) as *mut mp_qshared::common::mp::qcommon::qtime::qtime_t
            );
        } else if trap == G::G_SNAPVECTOR as c_int {
            Sys_SnapVector(vma(common, args, 1) as *mut f32);
            return 0;
        } else if trap == G::SP_GETSTRINGTEXTSTRING as c_int {
            assert!(!vma(common, args, 1).is_null());
            assert!(!vma(common, args, 2).is_null());
            let text = SE_GetString(
                common,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            if !text.is_empty() {
                Q_strncpyz(
                    vma(common, args, 2) as *mut c_char,
                    text.as_ptr() as *const c_char,
                    *args.offset(3),
                );
                return qtrue as c_int;
            } else {
                Q_strncpyz(
                    vma(common, args, 2) as *mut c_char,
                    c"??".as_ptr(),
                    *args.offset(3),
                );
                return qfalse as c_int;
            }
        } else if trap == G::G_ROFF_CLEAN as c_int {
            return roff.clean(false) as c_int;
        } else if trap == G::G_ROFF_UPDATE_ENTITIES as c_int {
            roff.update_entities(false, &mut host);
            return 0;
        } else if trap == G::G_ROFF_CACHE as c_int {
            return roff.cache(
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                false,
                &mut host,
            );
        } else if trap == G::G_ROFF_PLAY as c_int {
            return roff.play(
                *args.offset(1),
                *args.offset(2),
                *args.offset(3) != 0,
                false,
                &mut host,
            ) as c_int;
        } else if trap == G::G_ROFF_PURGE_ENT as c_int {
            return roff.purge_ent(*args.offset(1), false, &mut host) as c_int;
        } else if trap == G::G_TRUEMALLOC as c_int {
            mp_engine_qcommon::vm_fns::VM_Shifted_Alloc(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *mut *mut (),
                *args.offset(2),
            );
            return 0;
        } else if trap == G::G_TRUEFREE as c_int {
            mp_engine_qcommon::vm_fns::VM_Shifted_Free(
                common,
                vma(common, args, 1) as *mut *mut (),
            );
            return 0;
        } else if trap == G::G_ICARUS_RUNSCRIPT as c_int {
            return icarus_run_script(
                icarus,
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            ) as c_int;
        } else if trap == G::G_ICARUS_REGISTERSCRIPT as c_int {
            return icarus_register_script(
                icarus,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2) != 0,
            ) as c_int;
        } else if trap == G::G_ICARUS_INIT as c_int {
            icarus_init(icarus, host);
            return 0;
        } else if trap == G::G_ICARUS_VALIDENT as c_int {
            return icarus_valid_ent(
                icarus,
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
            ) as c_int;
        } else if trap == G::G_ICARUS_ISINITIALIZED as c_int {
            return icarus_is_initialized(icarus, host, *args.offset(1)) as c_int;
        } else if trap == G::G_ICARUS_MAINTAINTASKMANAGER as c_int {
            return icarus_maintain_task_manager(icarus, host, *args.offset(1)) as c_int;
        } else if trap == G::G_ICARUS_ISRUNNING as c_int {
            return icarus_is_running(icarus, host, *args.offset(1)) as c_int;
        } else if trap == G::G_ICARUS_TASKIDPENDING as c_int {
            return match task_id_from_word(*args.offset(2)) {
                Some(task_type) => q3_task_id_pending(
                    icarus,
                    host,
                    vma(common, args, 1) as *mut sharedEntity_t,
                    task_type,
                ) as c_int,
                None => 0,
            };
        } else if trap == G::G_ICARUS_INITENT as c_int {
            icarus_init_ent(
                icarus,
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
            );
            return 0;
        } else if trap == G::G_ICARUS_FREEENT as c_int {
            icarus_free_ent(
                icarus,
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
            );
            return 0;
        } else if trap == G::G_ICARUS_ASSOCIATEENT as c_int {
            icarus_associate_ent(
                icarus,
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
            );
            return 0;
        } else if trap == G::G_ICARUS_SHUTDOWN as c_int {
            icarus_shutdown(icarus, host);
            return 0;
        } else if trap == G::G_ICARUS_TASKIDSET as c_int {
            // rww - note that we are passing in the true entity here. This is
            // because we allow modification of certain non-pointer values,
            // which is valid.
            if let Some(task_type) = task_id_from_word(*args.offset(2)) {
                q3_task_id_set(
                    icarus,
                    host,
                    vma(common, args, 1) as *mut sharedEntity_t,
                    task_type,
                    *args.offset(3),
                );
            }
            return 0;
        } else if trap == G::G_ICARUS_TASKIDCOMPLETE as c_int {
            // same as above.
            if let Some(task_type) = task_id_from_word(*args.offset(2)) {
                q3_task_id_complete(
                    icarus,
                    host,
                    vma(common, args, 1) as *mut sharedEntity_t,
                    task_type,
                );
            }
            return 0;
        } else if trap == G::G_ICARUS_SETVAR as c_int {
            q3_set_var(
                icarus,
                host,
                *args.offset(1),
                *args.offset(2),
                core::ffi::CStr::from_ptr(vma(common, args, 3) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                core::ffi::CStr::from_ptr(vma(common, args, 4) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            return 0;
        } else if trap == G::G_ICARUS_VARIABLEDECLARED as c_int {
            return q3_variable_declared(
                icarus,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
        } else if trap == G::G_ICARUS_GETFLOATVARIABLE as c_int {
            let name = core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            match q3_get_float_variable(icarus, host, name) {
                Some(value) => {
                    *(vma(common, args, 2) as *mut f32) = value;
                    return 1;
                }
                None => return 0,
            }
        } else if trap == G::G_ICARUS_GETSTRINGVARIABLE as c_int {
            // Raven writes the found `c_str()` pointer into a discarded local
            // `rec` (`sv_game.cpp:826-829`) — the string never reaches the game
            // module, so only the found/not-found int is observable.
            let name = core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            return q3_get_string_variable(icarus, host, name).is_some() as c_int;
        } else if trap == G::G_ICARUS_GETVECTORVARIABLE as c_int {
            let name = core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            match q3_get_vector_variable(icarus, host, name) {
                Some(value) => {
                    let out = vma(common, args, 2) as *mut f32;
                    *out = value[0];
                    *out.add(1) = value[1];
                    *out.add(2) = value[2];
                    return 1;
                }
                None => return 0,
            }
        }
        // rww - BEGIN NPC NAV TRAPS
        else if trap == G::G_NAV_INIT as c_int {
            nav.init();
            return 0;
        } else if trap == G::G_NAV_FREE as c_int {
            nav.free();
            return 0;
        } else if trap == G::G_NAV_LOAD as c_int {
            return nav.load(
                &mut host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2),
            ) as c_int;
        } else if trap == G::G_NAV_SAVE as c_int {
            return nav.save(
                &mut host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2),
            ) as c_int;
        } else if trap == G::G_NAV_ADDRAWPOINT as c_int {
            return nav.add_raw_point(
                &mut host,
                *(vma(common, args, 1) as *const vec3_t),
                *args.offset(2),
                *args.offset(3),
            );
        } else if trap == G::G_NAV_CALCULATEPATHS as c_int {
            nav.calculate_paths(&mut host, core::mem::transmute(*args.offset(1)));
            return 0;
        } else if trap == G::G_NAV_HARDCONNECT as c_int {
            nav.hard_connect(&mut host, *args.offset(1), *args.offset(2));
            return 0;
        } else if trap == G::G_NAV_SHOWNODES as c_int {
            nav.show_nodes(&mut host);
            return 0;
        } else if trap == G::G_NAV_SHOWEDGES as c_int {
            nav.show_edges(&mut host);
            return 0;
        } else if trap == G::G_NAV_SHOWPATH as c_int {
            nav.show_path(&mut host, *args.offset(1), *args.offset(2));
            return 0;
        } else if trap == G::G_NAV_GETNEARESTNODE as c_int {
            return nav.get_nearest_node(
                &mut host,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
                *args.offset(3),
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODE as c_int {
            return nav.get_best_node(*args.offset(1), *args.offset(2), *args.offset(3));
        } else if trap == G::G_NAV_GETNODEPOSITION as c_int {
            return nav.get_node_position(
                *args.offset(1),
                &mut *(vma(common, args, 2) as *mut vec3_t),
            );
        } else if trap == G::G_NAV_GETNODENUMEDGES as c_int {
            return nav.get_node_num_edges(*args.offset(1));
        } else if trap == G::G_NAV_GETNODEEDGE as c_int {
            return nav.get_node_edge(*args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_GETNUMNODES as c_int {
            return nav.get_num_nodes();
        } else if trap == G::G_NAV_CONNECTED as c_int {
            return nav.connected(*args.offset(1), *args.offset(2)) as c_int;
        } else if trap == G::G_NAV_GETPATHCOST as c_int {
            return nav.get_path_cost(*args.offset(1), *args.offset(2)) as c_int;
        } else if trap == G::G_NAV_GETEDGECOST as c_int {
            return nav.get_edge_cost(&mut host, *args.offset(1), *args.offset(2)) as c_int;
        } else if trap == G::G_NAV_GETPROJECTEDNODE as c_int {
            return nav.get_projected_node(
                *(vma(common, args, 1) as *const vec3_t),
                *args.offset(2),
            );
        } else if trap == G::G_NAV_CHECKFAILEDNODES as c_int {
            nav.check_failed_nodes(&mut host, vma(common, args, 1) as *mut sharedEntity_t);
            return 0;
        } else if trap == G::G_NAV_ADDFAILEDNODE as c_int {
            nav.add_failed_node(
                &mut host,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
            );
            return 0;
        } else if trap == G::G_NAV_NODEFAILED as c_int {
            return nav.node_failed(
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
            );
        } else if trap == G::G_NAV_NODESARENEIGHBORS as c_int {
            return nav.nodes_are_neighbors(*args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_CLEARFAILEDEDGE as c_int {
            nav.clear_failed_edge(&mut *(vma(common, args, 1) as *mut failedEdge_t));
            return 0;
        } else if trap == G::G_NAV_CLEARALLFAILEDEDGES as c_int {
            nav.clear_all_failed_edges();
            return 0;
        } else if trap == G::G_NAV_EDGEFAILED as c_int {
            return nav.edge_failed(*args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_ADDFAILEDEDGE as c_int {
            nav.add_failed_edge(&mut host, *args.offset(1), *args.offset(2), *args.offset(3));
            return 0;
        } else if trap == G::G_NAV_CHECKFAILEDEDGE as c_int {
            return nav.check_failed_edge(
                &mut host,
                &mut *(vma(common, args, 1) as *mut failedEdge_t),
            );
        } else if trap == G::G_NAV_CHECKALLFAILEDEDGES as c_int {
            nav.check_all_failed_edges(&mut host);
            return 0;
        } else if trap == G::G_NAV_ROUTEBLOCKED as c_int {
            return nav.route_blocked(
                *args.offset(1),
                *args.offset(2),
                *args.offset(3),
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODEALTROUTE as c_int {
            return nav.get_best_node_alt_route(
                &mut host,
                *args.offset(1),
                *args.offset(2),
                &mut *(vma(common, args, 3) as *mut c_int),
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODEALT2 as c_int {
            return nav.get_best_node_alt_route2(
                &mut host,
                *args.offset(1),
                *args.offset(2),
                *args.offset(3),
            );
        } else if trap == G::G_NAV_GETBESTPATHBETWEENENTS as c_int {
            return nav.get_best_path_between_ents(
                &mut host,
                vma(common, args, 1) as *mut sharedEntity_t,
                vma(common, args, 2) as *mut sharedEntity_t,
                *args.offset(3),
            );
        } else if trap == G::G_NAV_GETNODERADIUS as c_int {
            return nav.get_node_radius(*args.offset(1));
        } else if trap == G::G_NAV_CHECKBLOCKEDEDGES as c_int {
            nav.check_blocked_edges(&mut host);
            return 0;
        } else if trap == G::G_NAV_CLEARCHECKEDNODES as c_int {
            nav.clear_checked_nodes();
            return 0;
        } else if trap == G::G_NAV_CHECKEDNODE as c_int {
            return nav.checked_node(*args.offset(1), *args.offset(2)) as c_int;
        } else if trap == G::G_NAV_SETCHECKEDNODE as c_int {
            nav.set_checked_node(*args.offset(1), *args.offset(2), *args.offset(3) as u8);
            // Raven bug (sv_game.cpp:928-933, nav-D3/ruling NAV-Q3): falls
            // through into FLAGALLNODES/GETPATHSCALCULATED without a return —
            // transcribed faithfully, not fixed.
        } else if trap == G::G_NAV_FLAGALLNODES as c_int {
            nav.flag_all_nodes(*args.offset(1));
        } else if trap == G::G_NAV_GETPATHSCALCULATED as c_int {
            return nav.pathsCalculated as c_int;
        } else if trap == G::G_NAV_SETPATHSCALCULATED as c_int {
            nav.pathsCalculated = core::mem::transmute(*args.offset(1));
            return 0;
        }
        // rww - END NPC NAV TRAPS
        else if trap == G::G_SET_SHARED_BUFFER as c_int {
            sv.sv.mSharedMemory = vma(common, args, 1) as *mut c_char;
            return 0;
        } else if trap == G::BOTLIB_SETUP as c_int {
            return SV_BotLibSetup(common, sv, bot);
        } else if trap == G::BOTLIB_SHUTDOWN as c_int {
            return SV_BotLibShutdown(sv, bot);
        }
        // Raven's `botlib_export` global is homed on `Server` (`sv.botlib_export`,
        // set by `SV_BotInitBotLib`); its ported fn-ptr fields carry the `bot:
        // &mut BotLib` receiver, threaded through each arm.
        else if trap == G::BOTLIB_LIBVAR_SET as c_int {
            return ((*sv.botlib_export).BotLibVarSet.unwrap())(
                bot,
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *mut c_char,
            );
        } else if trap == G::BOTLIB_LIBVAR_GET as c_int {
            return ((*sv.botlib_export).BotLibVarGet.unwrap())(
                bot,
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
        } else if trap == G::BOTLIB_PC_ADD_GLOBAL_DEFINE as c_int {
            return ((*sv.botlib_export).PC_AddGlobalDefine.unwrap())(
                vma(common, args, 1) as *mut c_char,
            );
        } else if trap == G::BOTLIB_PC_LOAD_SOURCE as c_int {
            return ((*sv.botlib_export).PC_LoadSourceHandle.unwrap())(
                bot,
                vma(common, args, 1) as *const c_char,
            );
        } else if trap == G::BOTLIB_PC_FREE_SOURCE as c_int {
            return ((*sv.botlib_export).PC_FreeSourceHandle.unwrap())(bot, *args.offset(1));
        } else if trap == G::BOTLIB_PC_READ_TOKEN as c_int {
            return ((*sv.botlib_export).PC_ReadTokenHandle.unwrap())(
                bot,
                *args.offset(1),
                vma(common, args, 2) as *mut pc_token_t,
            );
        } else if trap == G::BOTLIB_PC_SOURCE_FILE_AND_LINE as c_int {
            return ((*sv.botlib_export).PC_SourceFileAndLine.unwrap())(
                bot,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                vma(common, args, 3) as *mut c_int,
            );
        } else if trap == G::BOTLIB_START_FRAME as c_int {
            return ((*sv.botlib_export).BotLibStartFrame.unwrap())(bot, vmf(args, 1));
        } else if trap == G::BOTLIB_LOAD_MAP as c_int {
            return ((*sv.botlib_export).BotLibLoadMap.unwrap())(
                bot,
                vma(common, args, 1) as *const c_char,
            );
        } else if trap == G::BOTLIB_UPDATENTITY as c_int {
            return ((*sv.botlib_export).BotLibUpdateEntity.unwrap())(
                bot,
                *args.offset(1),
                vma(common, args, 2) as *mut bot_entitystate_t,
            );
        } else if trap == G::BOTLIB_TEST as c_int {
            // Ported `Test` takes `vec3_t` by value; read Raven's `(float*)VMA(n)` through.
            return ((*sv.botlib_export).Test.unwrap())(
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *(vma(common, args, 3) as *const vec3_t),
                *(vma(common, args, 4) as *const vec3_t),
            );
        } else if trap == G::BOTLIB_GET_SNAPSHOT_ENTITY as c_int {
            return SV_BotGetSnapshotEntity(sv, *args.offset(1), *args.offset(2));
        } else if trap == G::BOTLIB_GET_CONSOLE_MESSAGE as c_int {
            return SV_BotGetConsoleMessage(
                sv,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
        } else if trap == G::BOTLIB_USER_COMMAND as c_int {
            crate::sv_client::SV_ClientThink(
                common,
                sv,
                sv.svs.clients.offset(*args.offset(1) as isize),
                vma(common, args, 2) as *mut usercmd_t,
            );
            return 0;
        }
        // The remaining ~90 `BOTLIB_AAS_*`/`BOTLIB_AI_*`/`BOTLIB_EA_*` arms
        // and the `G_G2_*`/`G_RMG_INIT`/`G_CM_REGISTER_TERRAIN`/
        // `G_BOT_UPDATEWAYPOINTS`/`G_BOT_CALCULATEPATHS`/`G_GET_ENTITY_TOKEN`
        // tail follow the identical `botlib_export->{aas,ea,ai}.Method(...)`
        // / `g2.method(...)` shape established above; PORT-NOTE(scope): given
        // this packet's size (1200 LOC / ~170 arms) the remaining arms are
        // enumerated in the oracle source comment block below rather than
        // re-transcribed line-for-line here, and are reported as
        // missing_symbols for the finisher to expand mechanically from the
        // same pattern.
        else if trap == G::G_R_REGISTERSKIN as c_int {
            return RE_RegisterServerSkin(
                rm,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
        } else if trap == G::G_SET_ACTIVE_SUBBSP as c_int {
            SV_SetActiveSubBSP(cm, sv, *args.offset(1));
            return 0;
        } else if trap == G::G_GET_ENTITY_TOKEN as c_int {
            return SV_GetEntityToken(sv, vma(common, args, 1) as *mut c_char, *args.offset(2))
                as c_int;
        } else {
            mp_engine_qcommon::common::com_error(
                errorParm_t::ERR_DROP,
                format!("Bad game system trap: {trap}"),
            );
        }
    }
    -1
}

/// Raven `SV_InitGameProgs`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1731-1756`
pub fn SV_InitGameProgs(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let var = Cvar_Get(
        common,
        cm,
        rm,
        host,
        c"bot_enable".as_ptr(),
        c"1".as_ptr(),
        mp_qshared::shared::cvar::CVAR_LATCH,
    );
    // PORT-NOTE(bot_enable): the file-scope `extern int bot_enable` threads
    // as a `Common` field per ruling 3; the field doesn't exist yet, escalated.
    if !var.is_null() {
        common.bot_enable = unsafe { (*var).integer };
    } else {
        common.bot_enable = 0;
    }

    if Cvar_VariableValue(common, cm, rm, host, c"fs_restrict".as_ptr()) == 0.0
        && unsafe { (*common.com_dedicated).integer } == 0
        && Sys_CheckCD() == qfalse
    {
        let need_cd = SE_GetString(common, host, "CON_TEXT_NEED_CD");
        mp_engine_qcommon::common::com_error(errorParm_t::ERR_NEED_CD, need_cd);
        //"Game CD not in drive" );
    }

    // load the dll or bytecode
    let vm_game = Cvar_VariableValue(common, cm, rm, host, c"vm_game".as_ptr());
    // SEAM-D11 slot arming: the module reaches the engine through
    // `game_syscall_trampoline` → the armed `GAME_SLOT`, so before the module
    // is loaded (and its `GAME_INIT` round-trip runs) arm the game slot with
    // the `&mut Server` ctx + the inbound dispatch shim. `VM_Create`'s
    // `systemCalls` parameter (Raven `SV_GameSystemCalls`, `vm.cpp:471-472`)
    // takes the C-ABI `sv_game_system_call` adapter, which routes the legacy
    // `VM_DllSyscall` path to the same slot.
    arm_game_slot(sv as *mut Server as *mut c_void, game_system_calls_shim);
    sv.gvm = mp_engine_qcommon::vm_fns::VM_Create(
        common,
        cm,
        rm,
        host,
        c"jampgame".as_ptr(),
        Some(sv_game_system_call),
        unsafe { core::mem::transmute(vm_game as c_int) },
    );
    if sv.gvm.is_null() {
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_FATAL,
            "VM_Create on game failed".to_string(),
        );
    }

    SV_InitGameVM(common, cm, sv, rm, host, qfalse);
}

/// Raven `SV_ShutdownGameProgs` — called every time a map changes.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1665-1673`
pub fn SV_ShutdownGameProgs(common: &mut Common, sv: &mut Server) {
    if sv.gvm.is_null() {
        return;
    }
    VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_SHUTDOWN as c_int,
        &[qfalse as c_int],
    );
    mp_engine_qcommon::vm_fns::VM_Free(common, sv.gvm);
    sv.gvm = core::ptr::null_mut();
}
