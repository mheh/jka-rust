//! `sv_game.cpp` — the server's game-VM interface: gentity/client accessors,
//! entity-string parsing, the outbound `SV_Game*` sinks, PVS/brush-model
//! helpers, VM lifecycle (`SV_InitGameProgs`/`SV_InitGameVM`/
//! `SV_RestartGameProgs`), and `SV_GameSystemCalls` (the inbound syscall
//! dispatcher the game VM calls through `VMA`/`VMF`).
//!
//! Source: `oracle/codemp/server/sv_game.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_ulong, c_void};
use core::sync::atomic::{AtomicU64, Ordering};

use mp_qshared::common::mp::botlib::bot_entitystate_s::bot_entitystate_t;
use mp_qshared::common::mp::gentity::{NUM_BSETS, NUM_TIDS};
use mp_qshared::common::mp::qcommon::failedEdge_t;
use mp_qshared::common::mp::qcommon::parms::parms_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::task_id_t::taskID_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::probe;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::pc_token_t;
use mp_qshared::shared::q_math::{
    vec3_origin, AngleVectors, MatrixMultiply, PerpendicularVector, Sys_SnapVector,
};
use mp_qshared::shared::surface_flags::CONTENTS_LIGHTSABER;
use mp_qshared::shared::wpobject::wpobject_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use native_math::vector::vec3_t;
use native_platform::Sys_CheckCD;
use native_types::clipHandle_t;

use crate::server::client_s::client_t;
use crate::server::server_state_t::serverState_t;
use crate::server::sv_entity_s::svEntity_t;
use crate::hook_install::sv_from_view;
use crate::server_host::sv_game_system_call;
use crate::sv_referee::ref_tap_syscall;
use crate::sv_renderer::RE_RegisterServerSkin;
use crate::Server;
use crate::{
    SV_BotAllocateClient, SV_BotCalculatePaths, SV_BotFreeClient, SV_BotGetConsoleMessage,
    SV_BotGetSnapshotEntity, SV_BotLibSetup, SV_BotLibShutdown, SV_BotWaypointReception,
    SV_DropClient, SV_SendServerCommand, SV_SetUserinfo,
};
use mp_abi::game::imports::MpGameImport as G;
use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t as T;
use mp_engine_qcommon::vm::VM_Call;

use mp_engine_botlib::BotLib;
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
use mp_engine_icarus::Icarus;
use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::CM_AreasConnected;
use mp_engine_qcommon::cmd_common::Cmd_Argv;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::roff::RoffSystem;
use mp_engine_qcommon::stringed::SE_GetString;
use native_string::q_strncpyz::{Q_strncpyz, Q_strncpyzBytes};

use crate::npcnav::Navigator;

// Ghoul2 (§F) API surface for the `G_G2_*` arms — the idiomatic reimplementation
// in `mp_engine_ghoul2` (ruling 40 dropped the Raven `G2API_` C prefix; fns carry
// the threaded `g2: &mut Ghoul2System` + `host: &mut impl EngineHost` receivers).
use mp_engine_ghoul2::api_bolts::{
    g2api_add_bolt, g2api_attach_instance_to_ent_num, g2api_clean_ent_attachments,
    g2api_clear_attached_instance, g2api_get_bolt_matrix, g2api_set_bolt_info,
    g2api_set_new_origin,
};
use mp_engine_ghoul2::api_bones::{
    g2api_does_bone_exist, g2api_get_bone_anim, g2api_list_bones, g2api_remove_bone,
    g2api_set_bone_angles, g2api_set_bone_anim,
};
use mp_engine_ghoul2::api_collision::{
    g2api_collision_detect, g2api_collision_detect_cache, g2api_override_server_with_client_data,
};
use mp_engine_ghoul2::api_models::{
    g2api_clean_ghoul2_models, g2api_copy_ghoul2_instance, g2api_copy_specific_g2_model,
    g2api_duplicate_ghoul2_instance, g2api_ghoul2_size, g2api_has_ghoul2_model_on_index,
    g2api_have_we_ghoul2_models, g2api_init_ghoul2_model, g2api_remove_ghoul2_model,
    g2api_remove_ghoul2_models, g2api_set_ghoul2_model_indexes, g2api_set_skin,
};
use mp_engine_ghoul2::api_ragdoll::{
    g2api_absurd_smoothing, g2api_animate_g2_models_rag, g2api_get_rag_bone_pos, g2api_ik_move,
    g2api_rag_effector_goal, g2api_rag_effector_kick, g2api_rag_force_solve,
    g2api_rag_pcj_constraint, g2api_rag_pcj_gradient_speed, g2api_reset_ragdoll,
    g2api_set_bone_ik_state, g2api_set_ragdoll,
};
use mp_engine_ghoul2::api_saveload::g2api_get_gla_name;
use mp_engine_ghoul2::api_surfaces::{
    g2api_get_surface_name, g2api_get_surface_render_status, g2api_list_surfaces,
    g2api_set_root_surface, g2api_set_surface_on_off,
};
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_ghoul2::gore::crag_doll_params::CRagDollParams;
use mp_engine_ghoul2::ragdoll_update_params::{RagDollUpdateKind, RagDollUpdateParams};
use mp_engine_ghoul2::shared::cghoul2_info::CGhoul2Info;
use mp_engine_ghoul2::shared::cghoul2_info_v::CGhoul2Info_v;
use mp_engine_qcommon::cm_terrain::register_terrain;
use mp_engine_qcommon::terrain_handle::TerrainHandle;
use mp_engine_rmg::rm_manager::RmManager;

use mp_qshared::common::mp::botlib::aas_altroutegoal_s::aas_altroutegoal_t;
use mp_qshared::common::mp::botlib::aas_clientmove_s::aas_clientmove_s;
use mp_qshared::common::mp::botlib::aas_entityinfo_s::aas_entityinfo_t;
use mp_qshared::common::mp::botlib::aas_predictroute_s::aas_predictroute_s;
use mp_qshared::common::mp::botlib::bot_consolemessage_s::bot_consolemessage_t;
use mp_qshared::common::mp::botlib::bot_initmove_s::bot_initmove_t;
use mp_qshared::common::mp::botlib::bot_input_s::bot_input_t;
use mp_qshared::common::mp::botlib::bot_match_s::bot_match_t;
use mp_qshared::common::mp::botlib::bot_moveresult_s::bot_moveresult_t;
use mp_qshared::common::mp::botlib::weaponinfo_s::weaponinfo_t;
use mp_qshared::common::mp::qcommon::aas_areainfo::aas_areainfo_t;
use mp_qshared::common::mp::qcommon::bot_goal::bot_goal_t;
use mp_qshared::common::mp::qcommon::collision_record::MAX_G2_COLLISIONS;
use mp_qshared::common::mp::qcommon::shared_ragdoll_params::sharedRagDollParams_t;
use mp_qshared::common::mp::qcommon::shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
use mp_qshared::shared::sharedIKMoveParams_t;
use mp_qshared::shared::{
    mdxaBone_t, sharedERagEffector, sharedERagPhase, CollisionRecord_t, Eorientations,
};

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
use mp_qshared::shared::q_string::COM_ParseExt;

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
            Q_strncpyzBytes(
                core::slice::from_raw_parts_mut(buffer, bufferSize as usize),
                core::ffi::CStr::from_ptr(s).to_bytes(),
                bufferSize as usize,
            );
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
            Q_strncpyzBytes(
                core::slice::from_raw_parts_mut(buffer, bufferSize as usize),
                core::ffi::CStr::from_ptr(s).to_bytes(),
                bufferSize as usize,
            );
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
/// Raven's `(int)` casts on the pointer fields are 32-bit-era; on LP64 they
/// truncate module string pointers (SP-map ICARUS ents), so the words stay
/// `isize` into `VM_ArgPtr`.
///
/// The file-static `gLocalModifier` (ruling 3: genuine cross-frame state)
/// threads as a `Server` field (`sv.g_local_modifier`).
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
            mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).parms as isize) as *mut parms_t;
        for i in 0..NUM_BSETS as usize {
            sv.g_local_modifier.behaviorSet[i] =
                mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).behaviorSet[i] as isize)
                    as *mut c_char;
        }
        sv.g_local_modifier.script_targetname =
            mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).script_targetname as isize)
                as *mut c_char;
        sv.g_local_modifier.delayScriptTime = (*ent).delayScriptTime;
        sv.g_local_modifier.fullName =
            mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).fullName as isize)
                as *mut c_char;
        sv.g_local_modifier.targetname =
            mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).targetname as isize)
                as *mut c_char;
        sv.g_local_modifier.classname =
            mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, (*ent).classname as isize)
                as *mut c_char;

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
        if clientNum < 0 || clientNum >= common.cvar(common.sv_maxclients).integer {
            return;
        }
        let client = &mut sv.svs.clients[clientNum as usize] as *mut client_t;
        SV_SendServerCommand(common, sv, client, &msg);
    }
}

/// Raven `SV_GameDropClient`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:110-115`
pub fn SV_GameDropClient(common: &mut Common, sv: &mut Server, clientNum: c_int, reason: &str) {
    if clientNum < 0 || clientNum >= common.cvar(common.sv_maxclients).integer {
        return;
    }
    let client = &mut sv.svs.clients[clientNum as usize] as *mut client_t;
    SV_DropClient(common, sv, client, reason);
}

/// Raven `SV_inPVS`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:209-233`
pub fn SV_inPVS(common: &Common, cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t) -> qboolean {
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
    if CM_AreasConnected(common, cm, area1, area2) == qfalse {
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
    unsafe {
        Q_strncpyz(
            core::slice::from_raw_parts_mut(buffer, bufferSize as usize),
            &info,
            bufferSize as usize,
        );
    }
}

/// Raven `SV_GetUsercmd`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:375-380`
pub fn SV_GetUsercmd(common: &mut Common, sv: &mut Server, clientNum: c_int, cmd: *mut usercmd_t) {
    // `sv_maxclients` is a `Common`-owned cvar handle; `common` is threaded in
    // to reach it (the `ERR_DROP`/`Com_Error` unwind on a bad clientNum is not
    // yet reachable from this crate — its call is unported).
    if clientNum < 0 || clientNum >= common.cvar(common.sv_maxclients).integer {
        // Com_Error(ERR_DROP, ...) — unported in this crate; see SV_GameError.
    }
    unsafe {
        *cmd = sv.svs.clients[clientNum as usize].lastUsercmd;
    }
}

/// Raven `SV_InitGameVM`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1682-1697`
pub fn SV_InitGameVM(view: &mut EngineHostView, sv: &mut Server, restart: qboolean) {
    sv.sv.entityParsePoint = mp_engine_qcommon::cm_load::CM_EntityString(view.cm);

    let ms = Com_Milliseconds(view);
    VM_Call(
        view.common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_INIT as c_int,
        &[sv.svs.time as isize, ms as isize, restart as isize],
    );

    let max_clients = view.common.cvar(view.common.sv_maxclients).integer;
    for i in 0..max_clients {
        sv.svs.clients[i as usize].gentity = core::ptr::null_mut();
    }
}

/// Raven `SV_GameCommand`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1766-1772`
pub fn SV_GameCommand(view: &mut EngineHostView) -> qboolean {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of this
    // slot for the borrow's duration; `VM_Call` reads `sv.gvm` and never
    // re-casts `view.sv` (rule 7).
    let sv = unsafe { sv_from_view(view) };
    if sv.sv.state as c_int != serverState_t::SS_GAME as c_int {
        return qfalse;
    }
    let r = VM_Call(
        view.common,
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
pub fn SV_RestartGameProgs(view: &mut EngineHostView, sv: &mut Server) {
    if sv.gvm.is_null() {
        return;
    }
    VM_Call(
        view.common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_SHUTDOWN as c_int,
        &[qtrue as isize],
    );

    // do a restart instead of a free
    sv.gvm = mp_engine_qcommon::vm_fns::VM_Restart(view, sv.gvm);
    if sv.gvm.is_null() {
        // bk001212 - as done below
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_FATAL,
            "VM_Restart on game failed".to_string(),
        );
    }

    SV_InitGameVM(view, sv, qtrue);
}

/// Raven `SV_EntityContact`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:291-305`
pub fn SV_EntityContact(
    view: &mut EngineHostView,
    mins: vec3_t,
    maxs: vec3_t,
    gEnt: *const sharedEntity_t,
    capsule: c_int,
) -> qboolean {
    unsafe {
        let origin = (*gEnt).r.currentOrigin;
        let angles = (*gEnt).r.currentAngles;

        let ch: clipHandle_t = crate::sv_world::SV_ClipHandleForEntity(view.cm, gEnt);
        let mut trace = core::mem::zeroed();
        mp_engine_qcommon::cm_trace::CM_TransformedBoxTrace(
            view,
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
    view: &mut EngineHostView,
    sv: &mut Server,
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
        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];

        if *name == b'*' as c_char {
            (*ent).s.modelindex = atoi(name.offset(1));

            if sv.sv.mLocalSubBSPIndex != -1 {
                (*ent).s.modelindex += sv.sv.mLocalSubBSPModelOffset;
            }

            let h = mp_engine_qcommon::cm_load::CM_InlineModel(view.cm, (*ent).s.modelindex);

            mp_engine_qcommon::cm_load::CM_ModelBounds(view.cm, h, &mut mins, &mut maxs);

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            let com_rmg = view.common.com_RMG;
            if com_rmg.is_some() && view.common.cvar(com_rmg).integer != 0 {
                (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(
                    view.cm,
                    h,
                    sv.sv.mLocalSubBSPIndex,
                );
            } else {
                (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(view.cm, h, -1);
            }
        } else if *name == b'#' as c_char {
            let bsp_name = format!("maps/{}.bsp", &name_str[1..]);
            (*ent).s.modelindex =
                mp_engine_qcommon::cm_load::CM_LoadSubBSP(view, &bsp_name, qfalse);
            mp_engine_qcommon::cm_load::CM_ModelBounds(
                view.cm,
                (*ent).s.modelindex,
                &mut mins,
                &mut maxs,
            );

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            //rwwNOTE: We don't ever want to set contents -1, it includes CONTENTS_LIGHTSABER.
            //Lots of stuff will explode if there's a brush with CONTENTS_LIGHTSABER that isn't attached to a client owner.
            //ent->contents = -1;		// we don't know exactly what is in the brushes
            let _ = CONTENTS_LIGHTSABER;
            let h = mp_engine_qcommon::cm_load::CM_InlineModel(view.cm, (*ent).s.modelindex);
            let sub_bsp = mp_engine_qcommon::cm_load::CM_FindSubBSP(view.cm, (*ent).s.modelindex);
            (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(view.cm, h, sub_bsp);
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
unsafe fn vma(common: &Common, args: *mut isize, n: isize) -> *mut c_void {
    mp_engine_qcommon::vm_fns::VM_ArgPtrWord(common, *args.offset(n)) as *mut c_void
}

/// Raven's `VMF(n)` macro — reinterpret `args[n]`'s bits as `float`.
#[inline]
unsafe fn vmf(args: *mut isize, n: isize) -> f32 {
    f32::from_bits(*args.offset(n) as u32)
}

/// Nullable `vec3_t` arg word for the trace arms. Raven's `SV_Trace` accepts
/// NULL `mins`/`maxs` and substitutes `vec3_origin` (`sv_world.cpp:816-822`);
/// our `SV_Trace` takes them by value, so the substitution lives at this seam,
/// where the game module's `trap_Trace(…, NULL, NULL, …)` (bot AI) still
/// delivers a null word.
#[inline]
unsafe fn vma_vec3_or_origin(common: &Common, args: *mut isize, n: isize) -> vec3_t {
    let p = vma(common, args, n) as *const vec3_t;
    if p.is_null() {
        vec3_origin
    } else {
        *p
    }
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
/// Raven's `int SV_GameSystemCalls(int *args)` is the all-int 32-bit original;
/// `args: *mut isize` / `-> isize` is the settled 64-bit-widened dual (AbiWord =
/// isize), matching the `vmMain` pair so pointer-valued arg words survive on
/// LP64 (docs/architecture/state-ownership.md, vmMain pair).
/// Source: `oracle/codemp/server/sv_game.cpp:458-1657`
#[allow(clippy::too_many_arguments, unused_variables)]
pub fn SV_GameSystemCalls(
    view: &mut EngineHostView,
    icarus: &mut Icarus,
    nav: &mut Navigator,
    roff: &mut RoffSystem,
    args: *mut isize,
) -> isize {
    // SAFETY: `args` is the trampoline's raw syscall word array (seam
    // pointer, porting-rules §D11); every arm reads only the words its trap
    // number defines, exactly as Raven's `int *args` does.
    unsafe {
        let trap = *args.offset(0);

        // Engine referee (record/replay): fold the ordered import number into
        // the frame's syscall digest (no-op when un-armed).
        // SAFETY: view-constructor slot, single-threaded (per-arm pattern).
        ref_tap_syscall(sv_from_view(view), trap);

        // rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without
        // adding a trap to match, and all of these traps must be shared and
        // have cases in sv_game, cl_cgame, and cl_ui. They must also all be
        // in the same order, and start at 100.
        if trap == T::TRAP_MEMSET as isize {
            mp_engine_qcommon::common_fns::Com_Memset(
                vma(view.common, args, 1) as *mut (),
                *args.offset(2) as c_int,
                *args.offset(3) as usize,
            );
            return 0;
        } else if trap == T::TRAP_MEMCPY as isize {
            mp_engine_qcommon::common_fns::Com_Memcpy(
                vma(view.common, args, 1) as *mut (),
                vma(view.common, args, 2) as *const (),
                *args.offset(3) as usize,
            );
            return 0;
        } else if trap == T::TRAP_STRNCPY as isize {
            return strncpy(
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *const c_char,
                *args.offset(3) as usize,
            ) as isize;
        } else if trap == T::TRAP_SIN as isize {
            return FloatAsInt(vmf(args, 1).sin()) as isize;
        } else if trap == T::TRAP_COS as isize {
            return FloatAsInt(vmf(args, 1).cos()) as isize;
        } else if trap == T::TRAP_ATAN2 as isize {
            return FloatAsInt(vmf(args, 1).atan2(vmf(args, 2))) as isize;
        } else if trap == T::TRAP_SQRT as isize {
            return FloatAsInt(vmf(args, 1).sqrt()) as isize;
        } else if trap == T::TRAP_MATRIXMULTIPLY as isize {
            MatrixMultiply(
                &*(vma(view.common, args, 1) as *const [[f32; 3]; 3]),
                &*(vma(view.common, args, 2) as *const [[f32; 3]; 3]),
                &mut *(vma(view.common, args, 3) as *mut [[f32; 3]; 3]),
            );
            return 0;
        } else if trap == T::TRAP_ANGLEVECTORS as isize {
            AngleVectors(
                *(vma(view.common, args, 1) as *const vec3_t),
                (vma(view.common, args, 2) as *mut vec3_t).as_mut(),
                (vma(view.common, args, 3) as *mut vec3_t).as_mut(),
                (vma(view.common, args, 4) as *mut vec3_t).as_mut(),
            );
            return 0;
        } else if trap == T::TRAP_PERPENDICULARVECTOR as isize {
            PerpendicularVector(
                &mut *(vma(view.common, args, 1) as *mut vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
            );
            return 0;
        } else if trap == T::TRAP_FLOOR as isize {
            return FloatAsInt(vmf(args, 1).floor()) as isize;
        } else if trap == T::TRAP_CEIL as isize {
            return FloatAsInt(vmf(args, 1).ceil()) as isize;
        } else if trap == T::TRAP_TESTPRINTINT as isize || trap == T::TRAP_TESTPRINTFLOAT as isize {
            return 0;
        } else if trap == T::TRAP_ACOS as isize {
            return FloatAsInt(mp_engine_qcommon::common_fns::Q_acos(vmf(args, 1))) as isize;
        } else if trap == T::TRAP_ASIN as isize {
            return FloatAsInt(mp_engine_qcommon::common_fns::Q_asin(vmf(args, 1))) as isize;
        } else if trap == G::G_PRINT as isize {
            let s = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_string_lossy();
            mp_engine_qcommon::common::common::com_printf(view.common, &s);
            return 0;
        } else if trap == G::G_ERROR as isize {
            let s = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_string_lossy()
                .into_owned();
            mp_engine_qcommon::common::com_error(errorParm_t::ERR_DROP, s);
        } else if trap == G::G_MILLISECONDS as isize {
            return mp_engine_qcommon::timing::sys_milliseconds(view.common) as isize;
        } else if trap == G::G_PRECISIONTIMER_START as isize {
            // rww - precision timer funcs. -ALWAYS- call end after start with
            // supplied ptr, or you'll get a nasty memory leak. Not that you
            // should be using these outside of debug anyway.
            let supplied_ptr = vma(view.common, args, 1) as *mut *mut c_void;
            let new_timer = Box::new(mp_engine_qcommon::timing::timing_c::timing_c::default());
            *supplied_ptr = Box::into_raw(new_timer) as *mut c_void;
            (**(supplied_ptr as *mut *mut mp_engine_qcommon::timing::timing_c::timing_c)).Start();
            return 0;
        } else if trap == G::G_PRECISIONTIMER_END as isize {
            let timer = *args.offset(1) as *mut mp_engine_qcommon::timing::timing_c::timing_c;
            let r = (*timer).End();
            drop(Box::from_raw(timer));
            return r as isize;
        } else if trap == G::G_CVAR_REGISTER as isize {
            Cvar_Register(
                view,
                vma(view.common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
                &core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                    .to_string_lossy(),
                &core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                    .to_string_lossy(),
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::G_CVAR_UPDATE as isize {
            Cvar_Update(
                view.common,
                vma(view.common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
            );
            return 0;
        } else if trap == G::G_CVAR_SET as isize {
            Cvar_Set(
                view,
                &core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_string_lossy(),
                &core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                    .to_string_lossy(),
            );
            return 0;
        } else if trap == G::G_CVAR_VARIABLE_INTEGER_VALUE as isize {
            return Cvar_VariableIntegerValue(
                view.common,
                &core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_string_lossy(),
            ) as isize;
        } else if trap == G::G_CVAR_VARIABLE_STRING_BUFFER as isize {
            Cvar_VariableStringBuffer(
                view.common,
                &core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_string_lossy(),
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_ARGC as isize {
            return mp_engine_qcommon::cmd_common::Cmd_Argc(view.common) as isize;
        } else if trap == G::G_ARGV as isize {
            // Raven `Cmd_ArgvBuffer` inlined at its one caller: the module
            // out-buffer fill (caller-owned memory, caller-supplied length).
            let len = *args.offset(3) as usize;
            Q_strncpyz(
                core::slice::from_raw_parts_mut(vma(view.common, args, 2) as *mut c_char, len),
                Cmd_Argv(view.common, *args.offset(1) as c_int),
                len,
            );
            return 0;
        } else if trap == G::G_SEND_CONSOLE_COMMAND as isize {
            // module-memory seam: one conversion at the trap arm.
            let text = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_string_lossy()
                .into_owned();
            mp_engine_qcommon::cmd_common::Cbuf_ExecuteText(view, *args.offset(1) as c_int, &text);
            return 0;
        } else if trap == G::G_FS_FOPEN_FILE as isize {
            let qpath = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_string_lossy()
                .into_owned();
            return mp_engine_qcommon::files_pc::FS_FOpenFileByMode(
                view,
                &qpath,
                vma(view.common, args, 2) as *mut c_int,
                core::mem::transmute(*args.offset(3) as c_int),
            ) as isize;
        } else if trap == G::G_FS_READ as isize {
            mp_engine_qcommon::files_pc::FS_Read2(
                view.common,
                vma(view.common, args, 1) as *mut (),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_FS_WRITE as isize {
            FS_Write(
                view.common,
                vma(view.common, args, 1) as *const (),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_FS_FCLOSE_FILE as isize {
            FS_FCloseFile(view.common, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_FS_GETFILELIST as isize {
            // module-memory seam: path/extension convert at the trap arm; the
            // out-buffer stays caller-owned module memory.
            let path = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_string_lossy()
                .into_owned();
            let extension = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_string_lossy()
                .into_owned();
            return mp_engine_qcommon::files_pc::FS_GetFileList(
                view,
                &path,
                &extension,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_LOCATE_GAME_DATA as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_LocateGameData(
                sv,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                vma(view.common, args, 4) as *mut playerState_t,
                *args.offset(5) as c_int,
            );
            return 0;
        } else if trap == G::G_DROP_CLIENT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            // module-memory seam: one conversion at the trap arm.
            let reason = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_string_lossy()
                .into_owned();
            SV_GameDropClient(view.common, sv, *args.offset(1) as c_int, &reason);
            return 0;
        } else if trap == G::G_SEND_SERVER_COMMAND as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_GameSendServerCommand(
                view.common,
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_LINKENTITY as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::sv_world::SV_LinkEntity(
                view.common,
                view.cm,
                sv,
                vma(view.common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_UNLINKENTITY as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::sv_world::SV_UnlinkEntity(
                view.common,
                sv,
                vma(view.common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_ENTITIES_IN_BOX as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return crate::sv_world::SV_AreaEntities(
                view.common,
                sv,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
                vma(view.common, args, 3) as *mut c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_ENTITY_CONTACT as isize {
            return SV_EntityContact(
                view,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
                vma(view.common, args, 3) as *const sharedEntity_t,
                qfalse as c_int,
            ) as isize;
        } else if trap == G::G_ENTITY_CONTACTCAPSULE as isize {
            return SV_EntityContact(
                view,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
                vma(view.common, args, 3) as *const sharedEntity_t,
                qtrue as c_int,
            ) as isize;
        } else if trap == G::G_TRACE as isize {
            crate::sv_world::SV_Trace(
                view,
                vma(view.common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(view.common, args, 2) as *const vec3_t),
                vma_vec3_or_origin(view.common, args, 3),
                vma_vec3_or_origin(view.common, args, 4),
                *(vma(view.common, args, 5) as *const vec3_t),
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
                qfalse as c_int,
                0,
                *args.offset(9) as c_int,
            );
            return 0;
        } else if trap == G::G_G2TRACE as isize {
            crate::sv_world::SV_Trace(
                view,
                vma(view.common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(view.common, args, 2) as *const vec3_t),
                vma_vec3_or_origin(view.common, args, 3),
                vma_vec3_or_origin(view.common, args, 4),
                *(vma(view.common, args, 5) as *const vec3_t),
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
                qfalse as c_int,
                *args.offset(8) as c_int,
                *args.offset(9) as c_int,
            );
            return 0;
        } else if trap == G::G_TRACECAPSULE as isize {
            crate::sv_world::SV_Trace(
                view,
                vma(view.common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                *(vma(view.common, args, 2) as *const vec3_t),
                vma_vec3_or_origin(view.common, args, 3),
                vma_vec3_or_origin(view.common, args, 4),
                *(vma(view.common, args, 5) as *const vec3_t),
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
                qtrue as c_int,
                *args.offset(8) as c_int,
                *args.offset(9) as c_int,
            );
            return 0;
        } else if trap == G::G_POINT_CONTENTS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return crate::sv_world::SV_PointContents(
                view.common,
                view.cm,
                sv,
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_SET_SERVER_CULL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            sv.g_svCullDist = vmf(args, 1);
            return 0;
        } else if trap == G::G_SET_BRUSH_MODEL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_SetBrushModel(
                view,
                sv,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                vma(view.common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_IN_PVS as isize {
            let r = SV_inPVS(
                view.common,
                view.cm,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
            );
            // Referee probe: aggregate InPVS hit ratio (bot vision dies if ~0).
            {
                static PVS_CALLS: AtomicU64 = AtomicU64::new(0);
                static PVS_TRUE: AtomicU64 = AtomicU64::new(0);
                let n = PVS_CALLS.fetch_add(1, Ordering::Relaxed) + 1;
                if r != 0 {
                    PVS_TRUE.fetch_add(1, Ordering::Relaxed);
                }
                if n % 4096 == 0 {
                    probe!(
                        "PVS_RATIO",
                        "calls={} true={}",
                        n,
                        PVS_TRUE.load(Ordering::Relaxed)
                    );
                }
            }
            return r as isize;
        } else if trap == G::G_IN_PVS_IGNORE_PORTALS as isize {
            return SV_inPVSIgnorePortals(
                view.cm,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
            ) as isize;
        } else if trap == G::G_SET_CONFIGSTRING as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::sv_init::SV_SetConfigstring(
                view,
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_GET_CONFIGSTRING as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::sv_init::SV_GetConfigstring(
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_SET_USERINFO as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_SetUserinfo(
                view.common,
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const c_char,
            );
            return 0;
        } else if trap == G::G_GET_USERINFO as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::sv_init::SV_GetUserinfo(
                view.common,
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_GET_SERVERINFO as isize {
            SV_GetServerinfo(
                view.common,
                vma(view.common, args, 1) as *mut c_char,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::G_ADJUST_AREA_PORTAL_STATE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_AdjustAreaPortalState(
                view.cm,
                sv,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                core::mem::transmute(*args.offset(2) as c_int),
            );
            return 0;
        } else if trap == G::G_AREAS_CONNECTED as isize {
            return CM_AreasConnected(
                view.common,
                view.cm,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_BOT_ALLOCATE_CLIENT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return SV_BotAllocateClient(view.common, sv) as isize;
        } else if trap == G::G_BOT_FREE_CLIENT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_BotFreeClient(view.common, sv, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_GET_USERCMD as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_GetUsercmd(
                view.common,
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut usercmd_t,
            );
            return 0;
        } else if trap == G::G_SIEGEPERSSET as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            sv.sv_siegePersData = *(vma(view.common, args, 1)
                as *const mp_qshared::common::mp::qcommon::siege_pers::siegePers_t);
            return 0;
        } else if trap == G::G_SIEGEPERSGET as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            *(vma(view.common, args, 1)
                as *mut mp_qshared::common::mp::qcommon::siege_pers::siegePers_t) =
                sv.sv_siegePersData;
            return 0;
        }
        // rwwRMG - G_GET_ENTITY_TOKEN and G_BOT_GET_MEMORY/G_BOT_FREE_MEMORY
        // stay commented out in Raven (sv_game.cpp:648-672) — not transcribed.
        else if trap == G::G_DEBUG_POLYGON_CREATE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return crate::BotImport_DebugPolygonCreate(
                sv,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *const [f32; 3],
            ) as isize;
        } else if trap == G::G_DEBUG_POLYGON_DELETE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            crate::BotImport_DebugPolygonDelete(sv, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_REAL_TIME as isize {
            return mp_engine_qcommon::common_fns::Com_RealTime(
                vma(view.common, args, 1) as *mut mp_qshared::common::mp::qcommon::qtime::qtime_t
            ) as isize;
        } else if trap == G::G_SNAPVECTOR as isize {
            Sys_SnapVector(vma(view.common, args, 1) as *mut f32);
            return 0;
        } else if trap == G::SP_GETSTRINGTEXTSTRING as isize {
            assert!(!vma(view.common, args, 1).is_null());
            assert!(!vma(view.common, args, 2).is_null());
            let text = SE_GetString(
                view,
                core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            let out_len = *args.offset(3) as usize;
            let out =
                core::slice::from_raw_parts_mut(vma(view.common, args, 2) as *mut c_char, out_len);
            if !text.is_empty() {
                Q_strncpyz(out, &text, out_len);
                return qtrue as isize;
            } else {
                Q_strncpyz(out, "??", out_len);
                return qfalse as isize;
            }
        } else if trap == G::G_ROFF_CLEAN as isize {
            return roff.clean(false) as isize;
        } else if trap == G::G_ROFF_UPDATE_ENTITIES as isize {
            roff.update_entities(false, view);
            return 0;
        } else if trap == G::G_ROFF_CACHE as isize {
            return roff.cache(
                core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                false,
                view,
            ) as isize;
        } else if trap == G::G_ROFF_PLAY as isize {
            return roff.play(
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) != 0,
                false,
                view,
            ) as isize;
        } else if trap == G::G_ROFF_PURGE_ENT as isize {
            return roff.purge_ent(*args.offset(1) as c_int, false, view) as isize;
        } else if trap == G::G_TRUEMALLOC as isize {
            mp_engine_qcommon::vm_fns::VM_Shifted_Alloc(
                view,
                vma(view.common, args, 1) as *mut *mut (),
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::G_TRUEFREE as isize {
            mp_engine_qcommon::vm_fns::VM_Shifted_Free(
                view.common,
                vma(view.common, args, 1) as *mut *mut (),
            );
            return 0;
        } else if trap == G::G_ICARUS_RUNSCRIPT as isize {
            // SAFETY: view-constructor slot, single-threaded; the `sv` cast is
            // dropped before `icarus_run_script` (which may reach sv-touching
            // host methods) so no other cast of this slot is live (rule 7).
            let ent = {
                let sv = sv_from_view(view);
                ConvertedEntity(
                    view.common,
                    sv,
                    vma(view.common, args, 1) as *mut sharedEntity_t,
                )
            };
            let script = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            return icarus_run_script(icarus, view, ent, script) as isize;
        } else if trap == G::G_ICARUS_REGISTERSCRIPT as isize {
            let script = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            return icarus_register_script(icarus, view, script, *args.offset(2) != 0) as isize;
        } else if trap == G::G_ICARUS_INIT as isize {
            icarus_init(icarus, view);
            return 0;
        } else if trap == G::G_ICARUS_VALIDENT as isize {
            // SAFETY: view-constructor slot, single-threaded; `sv` cast dropped
            // before the icarus call (rule 7).
            let ent = {
                let sv = sv_from_view(view);
                ConvertedEntity(
                    view.common,
                    sv,
                    vma(view.common, args, 1) as *mut sharedEntity_t,
                )
            };
            return icarus_valid_ent(icarus, view, ent) as isize;
        } else if trap == G::G_ICARUS_ISINITIALIZED as isize {
            return icarus_is_initialized(icarus, view, *args.offset(1) as c_int) as isize;
        } else if trap == G::G_ICARUS_MAINTAINTASKMANAGER as isize {
            return icarus_maintain_task_manager(icarus, view, *args.offset(1) as c_int) as isize;
        } else if trap == G::G_ICARUS_ISRUNNING as isize {
            return icarus_is_running(icarus, view, *args.offset(1) as c_int) as isize;
        } else if trap == G::G_ICARUS_TASKIDPENDING as isize {
            return match task_id_from_word(*args.offset(2) as c_int) {
                Some(task_type) => {
                    let ent = vma(view.common, args, 1) as *mut sharedEntity_t;
                    q3_task_id_pending(icarus, view, ent, task_type) as c_int
                }
                None => 0,
            } as isize;
        } else if trap == G::G_ICARUS_INITENT as isize {
            // SAFETY: view-constructor slot, single-threaded; `sv` cast dropped
            // before the icarus call (rule 7).
            let ent = {
                let sv = sv_from_view(view);
                ConvertedEntity(
                    view.common,
                    sv,
                    vma(view.common, args, 1) as *mut sharedEntity_t,
                )
            };
            icarus_init_ent(icarus, view, ent);
            return 0;
        } else if trap == G::G_ICARUS_FREEENT as isize {
            // SAFETY: view-constructor slot, single-threaded; `sv` cast dropped
            // before the icarus call (rule 7).
            let ent = {
                let sv = sv_from_view(view);
                ConvertedEntity(
                    view.common,
                    sv,
                    vma(view.common, args, 1) as *mut sharedEntity_t,
                )
            };
            icarus_free_ent(icarus, view, ent);
            return 0;
        } else if trap == G::G_ICARUS_ASSOCIATEENT as isize {
            // SAFETY: view-constructor slot, single-threaded; `sv` cast dropped
            // before the icarus call (rule 7).
            let ent = {
                let sv = sv_from_view(view);
                ConvertedEntity(
                    view.common,
                    sv,
                    vma(view.common, args, 1) as *mut sharedEntity_t,
                )
            };
            icarus_associate_ent(icarus, view, ent);
            return 0;
        } else if trap == G::G_ICARUS_SHUTDOWN as isize {
            icarus_shutdown(icarus, view);
            return 0;
        } else if trap == G::G_ICARUS_TASKIDSET as isize {
            // rww - note that we are passing in the true entity here. This is
            // because we allow modification of certain non-pointer values,
            // which is valid.
            if let Some(task_type) = task_id_from_word(*args.offset(2) as c_int) {
                let ent = vma(view.common, args, 1) as *mut sharedEntity_t;
                q3_task_id_set(icarus, view, ent, task_type, *args.offset(3) as c_int);
            }
            return 0;
        } else if trap == G::G_ICARUS_TASKIDCOMPLETE as isize {
            // same as above.
            if let Some(task_type) = task_id_from_word(*args.offset(2) as c_int) {
                let ent = vma(view.common, args, 1) as *mut sharedEntity_t;
                q3_task_id_complete(icarus, view, ent, task_type);
            }
            return 0;
        } else if trap == G::G_ICARUS_SETVAR as isize {
            let var_name = core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                .to_str()
                .unwrap_or("");
            let var_value = core::ffi::CStr::from_ptr(vma(view.common, args, 4) as *const c_char)
                .to_str()
                .unwrap_or("");
            q3_set_var(
                icarus,
                view,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                var_name,
                var_value,
            );
            return 0;
        } else if trap == G::G_ICARUS_VARIABLEDECLARED as isize {
            let name = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            return q3_variable_declared(icarus, view, name) as isize;
        } else if trap == G::G_ICARUS_GETFLOATVARIABLE as isize {
            let name = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            match q3_get_float_variable(icarus, view, name) {
                Some(value) => {
                    *(vma(view.common, args, 2) as *mut f32) = value;
                    return 1;
                }
                None => return 0,
            }
        } else if trap == G::G_ICARUS_GETSTRINGVARIABLE as isize {
            // Raven writes the found `c_str()` pointer into a discarded local
            // `rec` (`sv_game.cpp:826-829`) — the string never reaches the game
            // module, so only the found/not-found int is observable.
            let name = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            return q3_get_string_variable(icarus, view, name).is_some() as isize;
        } else if trap == G::G_ICARUS_GETVECTORVARIABLE as isize {
            let name = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            match q3_get_vector_variable(icarus, view, name) {
                Some(value) => {
                    let out = vma(view.common, args, 2) as *mut f32;
                    *out = value[0];
                    *out.add(1) = value[1];
                    *out.add(2) = value[2];
                    return 1;
                }
                None => return 0,
            }
        }
        // rww - BEGIN NPC NAV TRAPS
        else if trap == G::G_NAV_INIT as isize {
            nav.init();
            return 0;
        } else if trap == G::G_NAV_FREE as isize {
            nav.free();
            return 0;
        } else if trap == G::G_NAV_LOAD as isize {
            return nav.load(
                view,
                core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_SAVE as isize {
            return nav.save(
                view,
                core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_ADDRAWPOINT as isize {
            return nav.add_raw_point(
                view,
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_CALCULATEPATHS as isize {
            nav.calculate_paths(view, core::mem::transmute(*args.offset(1) as c_int));
            return 0;
        } else if trap == G::G_NAV_HARDCONNECT as isize {
            nav.hard_connect(view, *args.offset(1) as c_int, *args.offset(2) as c_int);
            return 0;
        } else if trap == G::G_NAV_SHOWNODES as isize {
            nav.show_nodes(view);
            return 0;
        } else if trap == G::G_NAV_SHOWEDGES as isize {
            nav.show_edges(view);
            return 0;
        } else if trap == G::G_NAV_SHOWPATH as isize {
            nav.show_path(view, *args.offset(1) as c_int, *args.offset(2) as c_int);
            return 0;
        } else if trap == G::G_NAV_GETNEARESTNODE as isize {
            return nav.get_nearest_node(
                view,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETBESTNODE as isize {
            return nav.get_best_node(
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETNODEPOSITION as isize {
            return nav.get_node_position(
                *args.offset(1) as c_int,
                &mut *(vma(view.common, args, 2) as *mut vec3_t),
            ) as isize;
        } else if trap == G::G_NAV_GETNODENUMEDGES as isize {
            return nav.get_node_num_edges(*args.offset(1) as c_int) as isize;
        } else if trap == G::G_NAV_GETNODEEDGE as isize {
            return nav.get_node_edge(*args.offset(1) as c_int, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_NAV_GETNUMNODES as isize {
            return nav.get_num_nodes() as isize;
        } else if trap == G::G_NAV_CONNECTED as isize {
            return nav.connected(*args.offset(1) as c_int, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_NAV_GETPATHCOST as isize {
            return nav.get_path_cost(*args.offset(1) as c_int, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_NAV_GETEDGECOST as isize {
            return nav.get_edge_cost(view, *args.offset(1) as c_int, *args.offset(2) as c_int)
                as isize;
        } else if trap == G::G_NAV_GETPROJECTEDNODE as isize {
            return nav.get_projected_node(
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_CHECKFAILEDNODES as isize {
            nav.check_failed_nodes(view, vma(view.common, args, 1) as *mut sharedEntity_t);
            return 0;
        } else if trap == G::G_NAV_ADDFAILEDNODE as isize {
            nav.add_failed_node(
                view,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::G_NAV_NODEFAILED as isize {
            return nav.node_failed(
                vma(view.common, args, 1) as *mut sharedEntity_t,
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_NODESARENEIGHBORS as isize {
            return nav.nodes_are_neighbors(*args.offset(1) as c_int, *args.offset(2) as c_int)
                as isize;
        } else if trap == G::G_NAV_CLEARFAILEDEDGE as isize {
            nav.clear_failed_edge(&mut *(vma(view.common, args, 1) as *mut failedEdge_t));
            return 0;
        } else if trap == G::G_NAV_CLEARALLFAILEDEDGES as isize {
            nav.clear_all_failed_edges();
            return 0;
        } else if trap == G::G_NAV_EDGEFAILED as isize {
            return nav.edge_failed(*args.offset(1) as c_int, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_NAV_ADDFAILEDEDGE as isize {
            nav.add_failed_edge(
                view,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_NAV_CHECKFAILEDEDGE as isize {
            return nav
                .check_failed_edge(view, &mut *(vma(view.common, args, 1) as *mut failedEdge_t))
                as isize;
        } else if trap == G::G_NAV_CHECKALLFAILEDEDGES as isize {
            nav.check_all_failed_edges(view);
            return 0;
        } else if trap == G::G_NAV_ROUTEBLOCKED as isize {
            return nav.route_blocked(
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETBESTNODEALTROUTE as isize {
            return nav.get_best_node_alt_route(
                view,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                &mut *(vma(view.common, args, 3) as *mut c_int),
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETBESTNODEALT2 as isize {
            return nav.get_best_node_alt_route2(
                view,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETBESTPATHBETWEENENTS as isize {
            return nav.get_best_path_between_ents(
                view,
                vma(view.common, args, 1) as *mut sharedEntity_t,
                vma(view.common, args, 2) as *mut sharedEntity_t,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::G_NAV_GETNODERADIUS as isize {
            return nav.get_node_radius(*args.offset(1) as c_int) as isize;
        } else if trap == G::G_NAV_CHECKBLOCKEDEDGES as isize {
            nav.check_blocked_edges(view);
            return 0;
        } else if trap == G::G_NAV_CLEARCHECKEDNODES as isize {
            nav.clear_checked_nodes();
            return 0;
        } else if trap == G::G_NAV_CHECKEDNODE as isize {
            return nav.checked_node(*args.offset(1) as c_int, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_NAV_SETCHECKEDNODE as isize {
            nav.set_checked_node(
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as u8,
            );
            // Raven bug (sv_game.cpp:928-933, nav-D3/ruling NAV-Q3): falls
            // through into FLAGALLNODES/GETPATHSCALCULATED without a return —
            // transcribed faithfully, not fixed.
        } else if trap == G::G_NAV_FLAGALLNODES as isize {
            nav.flag_all_nodes(*args.offset(1) as c_int);
        } else if trap == G::G_NAV_GETPATHSCALCULATED as isize {
            return nav.pathsCalculated as isize;
        } else if trap == G::G_NAV_SETPATHSCALCULATED as isize {
            nav.pathsCalculated = core::mem::transmute(*args.offset(1) as c_int);
            return 0;
        }
        // rww - END NPC NAV TRAPS
        else if trap == G::G_SET_SHARED_BUFFER as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            sv.sv.mSharedMemory = vma(view.common, args, 1) as *mut c_char;
            return 0;
        } else if trap == G::BOTLIB_SETUP as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return SV_BotLibSetup(view.common, sv, bot) as isize;
        } else if trap == G::BOTLIB_SHUTDOWN as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return SV_BotLibShutdown(sv, bot) as isize;
        }
        // Raven's `botlib_export` global is homed on `Server` (`sv.botlib_export`,
        // set by `SV_BotInitBotLib`); its ported fn-ptr fields carry the `bot:
        // &mut BotLib` receiver, threaded through each arm.
        else if trap == G::BOTLIB_LIBVAR_SET as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).BotLibVarSet.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut c_char,
            ) as isize;
        } else if trap == G::BOTLIB_LIBVAR_GET as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).BotLibVarGet.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_PC_ADD_GLOBAL_DEFINE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return ((*sv.botlib_export).PC_AddGlobalDefine.unwrap())(
                vma(view.common, args, 1) as *mut c_char
            ) as isize;
        } else if trap == G::BOTLIB_PC_LOAD_SOURCE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).PC_LoadSourceHandle.unwrap())(
                bot,
                vma(view.common, args, 1) as *const c_char,
            ) as isize;
        } else if trap == G::BOTLIB_PC_FREE_SOURCE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).PC_FreeSourceHandle.unwrap())(bot, *args.offset(1) as c_int)
                as isize;
        } else if trap == G::BOTLIB_PC_READ_TOKEN as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).PC_ReadTokenHandle.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut pc_token_t,
            ) as isize;
        } else if trap == G::BOTLIB_PC_SOURCE_FILE_AND_LINE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).PC_SourceFileAndLine.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut c_int,
            ) as isize;
        } else if trap == G::BOTLIB_START_FRAME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).BotLibStartFrame.unwrap())(bot, vmf(args, 1)) as isize;
        } else if trap == G::BOTLIB_LOAD_MAP as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).BotLibLoadMap.unwrap())(
                bot,
                vma(view.common, args, 1) as *const c_char,
            ) as isize;
        } else if trap == G::BOTLIB_UPDATENTITY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).BotLibUpdateEntity.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_entitystate_t,
            ) as isize;
        } else if trap == G::BOTLIB_TEST as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            // Ported `Test` takes `vec3_t` by value; read Raven's `(float*)VMA(n)` through.
            return ((*sv.botlib_export).Test.unwrap())(
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *(vma(view.common, args, 3) as *const vec3_t),
                *(vma(view.common, args, 4) as *const vec3_t),
            ) as isize;
        } else if trap == G::BOTLIB_GET_SNAPSHOT_ENTITY as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return SV_BotGetSnapshotEntity(sv, *args.offset(1) as c_int, *args.offset(2) as c_int)
                as isize;
        } else if trap == G::BOTLIB_GET_CONSOLE_MESSAGE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return SV_BotGetConsoleMessage(
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_USER_COMMAND as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            // The clientNum trap word is ABI-typed `int`; a C module's variadic
            // slot leaves the high 32 bits garbage, so read only the low 32.
            // Hoist the client pointer before the call — `sv` is also passed by
            // &mut, so the Vec index must resolve to a raw pointer first.
            let cl = &mut sv.svs.clients[*args.offset(1) as c_int as usize] as *mut client_t;
            crate::sv_client::SV_ClientThink(
                view.common,
                sv,
                cl,
                vma(view.common, args, 2) as *mut usercmd_t,
            );
            return 0;
        }
        // Raven's `botlib_export->aas.*` table (`sv_game.cpp:965-1063`). The
        // ported fn-ptr fields carry the `bot: &mut BotLib` receiver; Raven's
        // `(float *)VMA(n)` vec3 args are read through by value where the ported
        // signature takes `vec3_t`.
        else if trap == G::BOTLIB_AAS_BBOX_AREAS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_BBoxAreas.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
                vma(view.common, args, 3) as *mut c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_AREA_INFO as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_AreaInfo.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut aas_areainfo_t,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_ALTERNATIVE_ROUTE_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_AlternativeRouteGoals.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
                *(vma(view.common, args, 3) as *const vec3_t),
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                vma(view.common, args, 6) as *mut aas_altroutegoal_t,
                *args.offset(7) as c_int,
                *args.offset(8) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_ENTITY_INFO as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).aas.AAS_EntityInfo.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut aas_entityinfo_t,
            );
            return 0;
        } else if trap == G::BOTLIB_AAS_INITIALIZED as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_Initialized.unwrap())(bot) as isize;
        } else if trap == G::BOTLIB_AAS_PRESENCE_TYPE_BOUNDING_BOX as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).aas.AAS_PresenceTypeBoundingBox.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut vec3_t,
                vma(view.common, args, 3) as *mut vec3_t,
            );
            return 0;
        } else if trap == G::BOTLIB_AAS_TIME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return FloatAsInt(((*sv.botlib_export).aas.AAS_Time.unwrap())(bot)) as isize;
        } else if trap == G::BOTLIB_AAS_POINT_AREA_NUM as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_PointAreaNum.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
            ) as isize;
        } else if trap == G::BOTLIB_AAS_POINT_REACHABILITY_AREA_INDEX as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export)
                .aas
                .AAS_PointReachabilityAreaIndex
                .unwrap())(bot, *(vma(view.common, args, 1) as *const vec3_t))
                as isize;
        } else if trap == G::BOTLIB_AAS_TRACE_AREAS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_TraceAreas.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                *(vma(view.common, args, 2) as *const vec3_t),
                vma(view.common, args, 3) as *mut c_int,
                vma(view.common, args, 4) as *mut vec3_t,
                *args.offset(5) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_POINT_CONTENTS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_PointContents.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
            ) as isize;
        } else if trap == G::BOTLIB_AAS_NEXT_BSP_ENTITY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_NextBSPEntity.unwrap())(
                bot,
                *args.offset(1) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_VALUE_FOR_BSP_EPAIR_KEY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_ValueForBSPEpairKey.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_VECTOR_FOR_BSP_EPAIR_KEY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_VectorForBSPEpairKey.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut vec3_t,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_FLOAT_FOR_BSP_EPAIR_KEY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_FloatForBSPEpairKey.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut f32,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_INT_FOR_BSP_EPAIR_KEY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_IntForBSPEpairKey.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_AREA_REACHABILITY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_AreaReachability.unwrap())(
                bot,
                *args.offset(1) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_AREA_TRAVEL_TIME_TO_GOAL_AREA as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export)
                .aas
                .AAS_AreaTravelTimeToGoalArea
                .unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *const vec3_t,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_ENABLE_ROUTING_AREA as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_EnableRoutingArea.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_PREDICT_ROUTE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_PredictRoute.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut aas_predictroute_s,
                *args.offset(2) as c_int,
                *(vma(view.common, args, 3) as *const vec3_t),
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
                *args.offset(8) as c_int,
                *args.offset(9) as c_int,
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AAS_SWIMMING as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_Swimming.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
            ) as isize;
        } else if trap == G::BOTLIB_AAS_PREDICT_CLIENT_MOVEMENT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).aas.AAS_PredictClientMovement.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut aas_clientmove_s,
                *args.offset(2) as c_int,
                *(vma(view.common, args, 3) as *const vec3_t),
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *(vma(view.common, args, 6) as *const vec3_t),
                *(vma(view.common, args, 7) as *const vec3_t),
                *args.offset(8) as c_int,
                *args.offset(9) as c_int,
                vmf(args, 10),
                *args.offset(11) as c_int,
                *args.offset(12) as c_int,
                *args.offset(13) as c_int,
            ) as isize;
        }
        // Raven's `botlib_export->ea.*` elementary-action table
        // (`sv_game.cpp:1065-1152`).
        else if trap == G::BOTLIB_EA_SAY as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Say.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            );
            return 0;
        } else if trap == G::BOTLIB_EA_SAY_TEAM as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_SayTeam.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            );
            return 0;
        } else if trap == G::BOTLIB_EA_COMMAND as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Command.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            );
            return 0;
        } else if trap == G::BOTLIB_EA_ACTION as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Action.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            );
            // Raven `break;` (not `return 0;`) — falls out of the switch to the
            // trailing `return -1`, reproduced by this arm's absent return.
        } else if trap == G::BOTLIB_EA_GESTURE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Gesture.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_TALK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Talk.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_ATTACK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Attack.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_ALT_ATTACK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Alt_Attack.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_FORCEPOWER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_ForcePower.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_USE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Use.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_RESPAWN as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Respawn.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_CROUCH as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Crouch.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_UP as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveUp.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_DOWN as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveDown.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_FORWARD as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveForward.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_BACK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveBack.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_LEFT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveLeft.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE_RIGHT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_MoveRight.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_SELECT_WEAPON as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_SelectWeapon.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_EA_JUMP as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Jump.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_DELAYED_JUMP as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_DelayedJump.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_EA_MOVE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_Move.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *(vma(view.common, args, 2) as *const vec3_t),
                vmf(args, 3),
            );
            return 0;
        } else if trap == G::BOTLIB_EA_VIEW as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_View.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *(vma(view.common, args, 2) as *const vec3_t),
            );
            return 0;
        } else if trap == G::BOTLIB_EA_END_REGULAR as isize {
            // Raven `EA_EndRegular` carries no `bot` receiver (`ea_export_t`).
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            ((*sv.botlib_export).ea.EA_EndRegular.unwrap())(*args.offset(1) as c_int, vmf(args, 2));
            return 0;
        } else if trap == G::BOTLIB_EA_GET_INPUT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_GetInput.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vmf(args, 2),
                vma(view.common, args, 3) as *mut bot_input_t,
            );
            return 0;
        } else if trap == G::BOTLIB_EA_RESET_INPUT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ea.EA_ResetInput.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        }
        // Raven's `botlib_export->ai.*` table (`sv_game.cpp:1154-1310`). Some
        // ported fns additionally carry a leading `common: &mut Common`; those
        // resolve their `VMA` args into locals first so `view.common` is free to
        // reborrow mutably for the receiver.
        else if trap == G::BOTLIB_AI_LOAD_CHARACTER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotLoadCharacter.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                vmf(args, 2),
            ) as isize;
        } else if trap == G::BOTLIB_AI_FREE_CHARACTER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeCharacter.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_CHARACTERISTIC_FLOAT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return FloatAsInt(((*sv.botlib_export).ai.Characteristic_Float.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            )) as isize;
        } else if trap == G::BOTLIB_AI_CHARACTERISTIC_BFLOAT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return FloatAsInt(((*sv.botlib_export).ai.Characteristic_BFloat.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vmf(args, 3),
                vmf(args, 4),
            )) as isize;
        } else if trap == G::BOTLIB_AI_CHARACTERISTIC_INTEGER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.Characteristic_Integer.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_CHARACTERISTIC_BINTEGER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.Characteristic_BInteger.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_CHARACTERISTIC_STRING as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.Characteristic_String.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_ALLOC_CHAT_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotAllocChatState.unwrap())(bot) as isize;
        } else if trap == G::BOTLIB_AI_FREE_CHAT_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeChatState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_QUEUE_CONSOLE_MESSAGE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotQueueConsoleMessage.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut c_char,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_REMOVE_CONSOLE_MESSAGE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotRemoveConsoleMessage.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_NEXT_CONSOLE_MESSAGE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotNextConsoleMessage.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_consolemessage_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_NUM_CONSOLE_MESSAGE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotNumConsoleMessages.unwrap())(
                bot,
                *args.offset(1) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_INITIAL_CHAT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let a2 = vma(view.common, args, 2) as *mut c_char;
            let a4 = vma(view.common, args, 4) as *mut c_char;
            let a5 = vma(view.common, args, 5) as *mut c_char;
            let a6 = vma(view.common, args, 6) as *mut c_char;
            let a7 = vma(view.common, args, 7) as *mut c_char;
            let a8 = vma(view.common, args, 8) as *mut c_char;
            let a9 = vma(view.common, args, 9) as *mut c_char;
            let a10 = vma(view.common, args, 10) as *mut c_char;
            let a11 = vma(view.common, args, 11) as *mut c_char;
            ((*sv.botlib_export).ai.BotInitialChat.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                a2,
                *args.offset(3) as c_int,
                a4,
                a5,
                a6,
                a7,
                a8,
                a9,
                a10,
                a11,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_NUM_INITIAL_CHATS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotNumInitialChats.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            ) as isize;
        } else if trap == G::BOTLIB_AI_REPLY_CHAT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let a2 = vma(view.common, args, 2) as *mut c_char;
            let a5 = vma(view.common, args, 5) as *mut c_char;
            let a6 = vma(view.common, args, 6) as *mut c_char;
            let a7 = vma(view.common, args, 7) as *mut c_char;
            let a8 = vma(view.common, args, 8) as *mut c_char;
            let a9 = vma(view.common, args, 9) as *mut c_char;
            let a10 = vma(view.common, args, 10) as *mut c_char;
            let a11 = vma(view.common, args, 11) as *mut c_char;
            let a12 = vma(view.common, args, 12) as *mut c_char;
            return ((*sv.botlib_export).ai.BotReplyChat.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                a2,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                a5,
                a6,
                a7,
                a8,
                a9,
                a10,
                a11,
                a12,
            ) as isize;
        } else if trap == G::BOTLIB_AI_CHAT_LENGTH as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotChatLength.unwrap())(bot, *args.offset(1) as c_int)
                as isize;
        } else if trap == G::BOTLIB_AI_ENTER_CHAT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotEnterChat.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_GET_CHAT_MESSAGE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotGetChatMessage.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_STRING_CONTAINS as isize {
            // Raven `StringContains` carries no `bot` receiver.
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return ((*sv.botlib_export).ai.StringContains.unwrap())(
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_FIND_MATCH as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotFindMatch.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut bot_match_t,
                *args.offset(3) as c_ulong,
            ) as isize;
        } else if trap == G::BOTLIB_AI_MATCH_VARIABLE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotMatchVariable.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut bot_match_t,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut c_char,
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_UNIFY_WHITE_SPACES as isize {
            // Raven `UnifyWhiteSpaces` carries no `bot` receiver.
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            ((*sv.botlib_export).ai.UnifyWhiteSpaces.unwrap())(
                vma(view.common, args, 1) as *mut c_char
            );
            return 0;
        } else if trap == G::BOTLIB_AI_REPLACE_SYNONYMS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotReplaceSynonyms.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                *args.offset(2) as c_ulong,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_LOAD_CHAT_FILE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let a2 = vma(view.common, args, 2) as *mut c_char;
            let a3 = vma(view.common, args, 3) as *mut c_char;
            return ((*sv.botlib_export).ai.BotLoadChatFile.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                a2,
                a3,
            ) as isize;
        } else if trap == G::BOTLIB_AI_SET_CHAT_GENDER as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotSetChatGender.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_SET_CHAT_NAME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotSetChatName.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_RESET_GOAL_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetGoalState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_RESET_AVOID_GOALS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetAvoidGoals.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_REMOVE_FROM_AVOID_GOALS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotRemoveFromAvoidGoals.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_PUSH_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotPushGoal.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_goal_t,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_POP_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotPopGoal.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_EMPTY_GOAL_STACK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotEmptyGoalStack.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_DUMP_AVOID_GOALS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotDumpAvoidGoals.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_DUMP_GOAL_STACK as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotDumpGoalStack.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_GOAL_NAME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotGoalName.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_GET_TOP_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotGetTopGoal.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_GET_SECOND_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotGetSecondGoal.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_CHOOSE_LTG_ITEM as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let origin = *(vma(view.common, args, 2) as *const vec3_t);
            let inventory = vma(view.common, args, 3) as *mut c_int;
            return ((*sv.botlib_export).ai.BotChooseLTGItem.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                origin,
                inventory,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_CHOOSE_NBG_ITEM as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let origin = *(vma(view.common, args, 2) as *const vec3_t);
            let inventory = vma(view.common, args, 3) as *mut c_int;
            let ltg = vma(view.common, args, 5) as *mut bot_goal_t;
            return ((*sv.botlib_export).ai.BotChooseNBGItem.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                origin,
                inventory,
                *args.offset(4) as c_int,
                ltg,
                vmf(args, 6),
            ) as isize;
        } else if trap == G::BOTLIB_AI_TOUCHING_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotTouchingGoal.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                vma(view.common, args, 2) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_ITEM_GOAL_IN_VIS_BUT_NOT_VISIBLE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export)
                .ai
                .BotItemGoalInVisButNotVisible
                .unwrap())(
                bot,
                *args.offset(1) as c_int,
                *(vma(view.common, args, 2) as *const vec3_t),
                *(vma(view.common, args, 3) as *const vec3_t),
                vma(view.common, args, 4) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_GET_LEVEL_ITEM_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotGetLevelItemGoal.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
                vma(view.common, args, 3) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_GET_NEXT_CAMP_SPOT_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotGetNextCampSpotGoal.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_GET_MAP_LOCATION_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotGetMapLocationGoal.unwrap())(
                bot,
                vma(view.common, args, 1) as *mut c_char,
                vma(view.common, args, 2) as *mut bot_goal_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_AVOID_GOAL_TIME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return FloatAsInt(((*sv.botlib_export).ai.BotAvoidGoalTime.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
            )) as isize;
        } else if trap == G::BOTLIB_AI_SET_AVOID_GOAL_TIME as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotSetAvoidGoalTime.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vmf(args, 3),
            );
            return 0;
        } else if trap == G::BOTLIB_AI_INIT_LEVEL_ITEMS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotInitLevelItems.unwrap())(bot);
            return 0;
        } else if trap == G::BOTLIB_AI_UPDATE_ENTITY_ITEMS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotUpdateEntityItems.unwrap())(bot);
            return 0;
        } else if trap == G::BOTLIB_AI_LOAD_ITEM_WEIGHTS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotLoadItemWeights.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            ) as isize;
        } else if trap == G::BOTLIB_AI_FREE_ITEM_WEIGHTS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeItemWeights.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_INTERBREED_GOAL_FUZZY_LOGIC as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotInterbreedGoalFuzzyLogic.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_SAVE_GOAL_FUZZY_LOGIC as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotSaveGoalFuzzyLogic.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_MUTATE_GOAL_FUZZY_LOGIC as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotMutateGoalFuzzyLogic.unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                vmf(args, 2),
            );
            return 0;
        } else if trap == G::BOTLIB_AI_ALLOC_GOAL_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotAllocGoalState.unwrap())(
                bot,
                *args.offset(1) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_FREE_GOAL_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeGoalState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_RESET_MOVE_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetMoveState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_ADD_AVOID_SPOT as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotAddAvoidSpot.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *(vma(view.common, args, 2) as *const vec3_t),
                vmf(args, 3),
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_MOVE_TO_GOAL as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let result = vma(view.common, args, 1) as *mut bot_moveresult_t;
            let goal = vma(view.common, args, 3) as *mut bot_goal_t;
            ((*sv.botlib_export).ai.BotMoveToGoal.unwrap())(
                view.common,
                bot,
                result,
                *args.offset(2) as c_int,
                goal,
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_MOVE_IN_DIRECTION as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotMoveInDirection.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *(vma(view.common, args, 2) as *const vec3_t),
                vmf(args, 3),
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_RESET_AVOID_REACH as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetAvoidReach.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_RESET_LAST_AVOID_REACH as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetLastAvoidReach.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_REACHABILITY_AREA as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotReachabilityArea.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_MOVEMENT_VIEW_TARGET as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotMovementViewTarget.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_goal_t,
                *args.offset(3) as c_int,
                vmf(args, 4),
                // oracle sv_game.cpp:1282 passes `(float *)VMA(5)` — out-param.
                vma(view.common, args, 5) as *mut vec3_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_PREDICT_VISIBLE_POSITION as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotPredictVisiblePosition.unwrap())(
                bot,
                *(vma(view.common, args, 1) as *const vec3_t),
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut bot_goal_t,
                *args.offset(4) as c_int,
                // oracle sv_game.cpp:1284 passes `(float *)VMA(5)` — out-param.
                vma(view.common, args, 5) as *mut vec3_t,
            ) as isize;
        } else if trap == G::BOTLIB_AI_ALLOC_MOVE_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotAllocMoveState.unwrap())(bot) as isize;
        } else if trap == G::BOTLIB_AI_FREE_MOVE_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeMoveState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_INIT_MOVE_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotInitMoveState.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut bot_initmove_t,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_CHOOSE_BEST_FIGHT_WEAPON as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotChooseBestFightWeapon.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_int,
            ) as isize;
        } else if trap == G::BOTLIB_AI_GET_WEAPON_INFO as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotGetWeaponInfo.unwrap())(
                bot,
                *args.offset(1) as c_int,
                *args.offset(2) as c_int,
                vma(view.common, args, 3) as *mut weaponinfo_t,
            );
            return 0;
        } else if trap == G::BOTLIB_AI_LOAD_WEAPON_WEIGHTS as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotLoadWeaponWeights.unwrap())(
                bot,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut c_char,
            ) as isize;
        } else if trap == G::BOTLIB_AI_ALLOC_WEAPON_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            return ((*sv.botlib_export).ai.BotAllocWeaponState.unwrap())(bot) as isize;
        } else if trap == G::BOTLIB_AI_FREE_WEAPON_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotFreeWeaponState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_RESET_WEAPON_STATE as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            ((*sv.botlib_export).ai.BotResetWeaponState.unwrap())(bot, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::BOTLIB_AI_GENETIC_PARENTS_AND_CHILD_SELECTION as isize {
            // SAFETY: view-constructor slots, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            let bot = &mut *(view.bot.as_raw() as *mut BotLib);
            let ranks = vma(view.common, args, 2) as *mut f32;
            let parent1 = vma(view.common, args, 3) as *mut c_int;
            let parent2 = vma(view.common, args, 4) as *mut c_int;
            let child = vma(view.common, args, 5) as *mut c_int;
            return ((*sv.botlib_export)
                .ai
                .GeneticParentsAndChildSelection
                .unwrap())(
                view.common,
                bot,
                *args.offset(1) as c_int,
                ranks,
                parent1,
                parent2,
                child,
            ) as isize;
        } else if trap == G::G_R_REGISTERSKIN as isize {
            return RE_RegisterServerSkin(
                view,
                core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            ) as isize;
        }
        // Raven's `G_G2_*` ghoul2 (§F) table (`sv_game.cpp:1316-1618`). Each arm
        // casts the `view.g2` slot to the live `Ghoul2System` (per-slot rule);
        // host-taking g2 fns receive `view` (whose host methods touch only
        // common/rm, never g2). The VM `CGhoul2Info_v` handle is a raw
        // `*mut`/`**` into game/engine memory (ruling 40 dropped the `G2API_`
        // C prefix).
        else if trap == G::G_G2_LISTBONES as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            // Raven casts the VM handle straight to `CGhoul2Info*`
            // (`sv_game.cpp:1318`) — a debug-only reinterpret kept faithfully
            // (§19; dead in normal play).
            let ghl = &mut *(vma(view.common, args, 1) as *mut CGhoul2Info);
            g2api_list_bones(g2, view, ghl, *args.offset(2) as c_int);
            return 0;
        } else if trap == G::G_G2_LISTSURFACES as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghl = &mut *(*args.offset(1) as *mut CGhoul2Info);
            g2api_list_surfaces(g2, view, ghl);
            return 0;
        } else if trap == G::G_G2_HAVEWEGHOULMODELS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &*(*args.offset(1) as *const CGhoul2Info_v);
            return g2api_have_we_ghoul2_models(g2, ghoul2) as isize;
        } else if trap == G::G_G2_SETMODELS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            // Raven's `(qhandle_t*)VMA(2)`/`VMA(3)` arrays are length-less at the
            // seam and the ported fn ignores both (api_models.rs) — empty slices.
            g2api_set_ghoul2_model_indexes(g2, ghoul2, &[], &[]);
            return 0;
        } else if trap == G::G_G2_GETBOLT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bolt_matrix = &mut *(vma(view.common, args, 4) as *mut mdxaBone_t);
            let angles = *(vma(view.common, args, 5) as *const vec3_t);
            let position = *(vma(view.common, args, 6) as *const vec3_t);
            let scale = *(vma(view.common, args, 9) as *const vec3_t);
            return g2api_get_bolt_matrix(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                angles,
                position,
                *args.offset(7) as c_int,
                &[],
                scale,
                bolt_matrix,
            ) as isize;
        } else if trap == G::G_G2_GETBOLT_NOREC as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            g2.gbm_no_reconstruct = true;
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bolt_matrix = &mut *(vma(view.common, args, 4) as *mut mdxaBone_t);
            let angles = *(vma(view.common, args, 5) as *const vec3_t);
            let position = *(vma(view.common, args, 6) as *const vec3_t);
            let scale = *(vma(view.common, args, 9) as *const vec3_t);
            return g2api_get_bolt_matrix(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                angles,
                position,
                *args.offset(7) as c_int,
                &[],
                scale,
                bolt_matrix,
            ) as isize;
        } else if trap == G::G_G2_GETBOLT_NOREC_NOROT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            g2.gbm_no_reconstruct = true;
            g2.gbm_use_sp_method = true;
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bolt_matrix = &mut *(vma(view.common, args, 4) as *mut mdxaBone_t);
            let angles = *(vma(view.common, args, 5) as *const vec3_t);
            let position = *(vma(view.common, args, 6) as *const vec3_t);
            let scale = *(vma(view.common, args, 9) as *const vec3_t);
            return g2api_get_bolt_matrix(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
                angles,
                position,
                *args.offset(7) as c_int,
                &[],
                scale,
                bolt_matrix,
            ) as isize;
        } else if trap == G::G_G2_INITGHOUL2MODEL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            // Raven `if (!(*ghoul2Ptr)) *ghoul2Ptr = new CGhoul2Info_v;` — the
            // ported `g2api_init_ghoul2_model` takes the deref'd handle, so the
            // handle object's `new`/`delete` is the seam's job: the engine owns
            // the `Box`, the game holds the raw pointer, freed at G_G2_CLEANMODELS.
            if (*pp).is_null() {
                *pp = Box::into_raw(Box::new(CGhoul2Info_v { mItem: 0 }));
            }
            let ghoul2 = &mut **pp;
            let file_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            return g2api_init_ghoul2_model(
                g2,
                view,
                ghoul2,
                file_name,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
            ) as isize;
        } else if trap == G::G_G2_SETSKIN as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            // The indexed `CGhoul2Info` lives inside `g2.info_array`; the raw
            // reborrow detaches it so `g2` also threads through (the callee
            // touches only its own fields — the §F per-slot escape, cf.
            // `api_ragdoll::split_info`). Applies to every indexed-info arm below.
            let info = ghoul2.get_mut(g2, *args.offset(2) as c_int) as *mut CGhoul2Info;
            return g2api_set_skin(
                g2,
                view,
                &mut *info,
                *args.offset(3) as c_int,
                *args.offset(4) as c_int,
            ) as isize;
        } else if trap == G::G_G2_SIZE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &*(*args.offset(1) as *const CGhoul2Info_v);
            return g2api_ghoul2_size(g2, ghoul2) as isize;
        } else if trap == G::G_G2_ADDBOLT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                .to_str()
                .unwrap_or("");
            return g2api_add_bolt(g2, view, ghoul2, *args.offset(2) as c_int, bone_name) as isize;
        } else if trap == G::G_G2_SETBOLTINFO as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            g2api_set_bolt_info(
                g2,
                ghoul2,
                *args.offset(2) as c_int,
                *args.offset(3) as c_int,
            );
            return 0;
        } else if trap == G::G_G2_ANGLEOVERRIDE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                .to_str()
                .unwrap_or("");
            let angles = *(vma(view.common, args, 4) as *const vec3_t);
            // Referee probe: per-frame G2 bone-angle override args (floats the digest never hashed).
            probe!(
                "BONE_ANG",
                "b={} a={:08x},{:08x},{:08x} fl={:x} o={},{},{} bt={} ct={}",
                bone_name,
                angles[0].to_bits(),
                angles[1].to_bits(),
                angles[2].to_bits(),
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                *args.offset(7) as c_int,
                *args.offset(8) as c_int,
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
            );
            // Raven casts the arg words straight to `Eorientations`
            // (`sv_game.cpp:1371`); transmute reproduces that (§19 for an
            // out-of-range word — Raven's own UB).
            return g2api_set_bone_angles(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                bone_name,
                angles,
                *args.offset(5) as c_int,
                core::mem::transmute::<c_int, Eorientations>(*args.offset(6) as c_int),
                core::mem::transmute::<c_int, Eorientations>(*args.offset(7) as c_int),
                core::mem::transmute::<c_int, Eorientations>(*args.offset(8) as c_int),
                &[],
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
            ) as isize;
        } else if trap == G::G_G2_PLAYANIM as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                .to_str()
                .unwrap_or("");
            // Referee probe: G2 bone-anim override args the module passed (digest never hashed them).
            probe!(
                "BONE_ANIM",
                "m={} b={} sf={} ef={} fl={:x} sp={:08x} ct={} setf={:08x} bt={}",
                *args.offset(2) as c_int,
                bone_name,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                vmf(args, 7).to_bits(),
                *args.offset(8) as c_int,
                vmf(args, 9).to_bits(),
                *args.offset(10) as c_int,
            );
            return g2api_set_bone_anim(
                g2,
                ghoul2,
                *args.offset(2) as c_int,
                bone_name,
                *args.offset(4) as c_int,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                vmf(args, 7),
                *args.offset(8) as c_int,
                vmf(args, 9),
                *args.offset(10) as c_int,
            ) as isize;
        } else if trap == G::G_G2_GETBONEANIM as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            let info = ghoul2.get_mut(g2, *args.offset(10) as c_int) as *mut CGhoul2Info;
            match g2api_get_bone_anim(
                g2,
                view,
                &mut *info,
                bone_name,
                *args.offset(3) as c_int,
                &[],
            ) {
                Some((current_frame, start_frame, end_frame, flags, anim_speed)) => {
                    *(vma(view.common, args, 4) as *mut f32) = current_frame;
                    *(vma(view.common, args, 5) as *mut c_int) = start_frame;
                    *(vma(view.common, args, 6) as *mut c_int) = end_frame;
                    *(vma(view.common, args, 7) as *mut c_int) = flags;
                    *(vma(view.common, args, 8) as *mut f32) = anim_speed;
                    return qtrue as isize;
                }
                None => return qfalse as isize,
            }
        } else if trap == G::G_G2_GETGLANAME as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let point = vma(view.common, args, 3) as *mut c_char;
            let ghoul2 = &*(*args.offset(1) as *const CGhoul2Info_v);
            // Raven shoves the name into the VM-supplied buffer instead of
            // returning a pointer; the ported fn returns `Option<String>`.
            if let Some(name) = g2api_get_gla_name(g2, view, ghoul2, *args.offset(2) as c_int) {
                let bytes = name.as_bytes();
                core::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, point, bytes.len());
                *point.add(bytes.len()) = 0;
            }
            return 0;
        } else if trap == G::G_G2_COPYGHOUL2INSTANCE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let g2_from = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let g2_to = &mut *(*args.offset(2) as *mut CGhoul2Info_v);
            return g2api_copy_ghoul2_instance(g2, g2_from, g2_to, *args.offset(3) as c_int)
                as isize;
        } else if trap == G::G_G2_COPYSPECIFICGHOUL2MODEL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let g2_from = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let g2_to = &mut *(*args.offset(3) as *mut CGhoul2Info_v);
            g2api_copy_specific_g2_model(
                g2,
                g2_from,
                *args.offset(2) as c_int,
                g2_to,
                *args.offset(4) as c_int,
            );
            return 0;
        } else if trap == G::G_G2_DUPLICATEGHOUL2INSTANCE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let g2_from = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let pp = vma(view.common, args, 2) as *mut *mut CGhoul2Info_v;
            // Raven `*g2To = new CGhoul2Info_v` (assert `!*g2To`) — the seam owns
            // the box; the ported fn takes the deref'd handle.
            if (*pp).is_null() {
                *pp = Box::into_raw(Box::new(CGhoul2Info_v { mItem: 0 }));
            }
            let g2_to = &mut **pp;
            g2api_duplicate_ghoul2_instance(g2, g2_from, g2_to);
            return 0;
        } else if trap == G::G_G2_HASGHOUL2MODELONINDEX as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            let ghoul2 = &**pp;
            return g2api_has_ghoul2_model_on_index(g2, ghoul2, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_G2_REMOVEGHOUL2MODEL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            let ghoul2 = &mut **pp;
            return g2api_remove_ghoul2_model(g2, ghoul2, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_G2_REMOVEGHOUL2MODELS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            let ghoul2 = &mut **pp;
            return g2api_remove_ghoul2_models(g2, ghoul2) as isize;
        } else if trap == G::G_G2_CLEANMODELS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let pp = vma(view.common, args, 1) as *mut *mut CGhoul2Info_v;
            if !(*pp).is_null() {
                g2api_clean_ghoul2_models(g2, &mut **pp);
                // Raven `delete *ghoul2Ptr; *ghoul2Ptr = NULL;` — free the
                // seam-owned box (allocated at G_G2_INITGHOUL2MODEL); the ported
                // fn only ran the vector `Free`.
                drop(Box::from_raw(*pp));
                *pp = core::ptr::null_mut();
            }
            return 0;
        } else if trap == G::G_G2_COLLISIONDETECT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(2) as *mut CGhoul2Info_v);
            let coll_rec_map = vma(view.common, args, 1) as *mut CollisionRecord_t;
            let angles = *(vma(view.common, args, 3) as *const vec3_t);
            let position = *(vma(view.common, args, 4) as *const vec3_t);
            let ray_start = *(vma(view.common, args, 7) as *const vec3_t);
            let ray_end = *(vma(view.common, args, 8) as *const vec3_t);
            let scale = *(vma(view.common, args, 9) as *const vec3_t);
            let records = g2api_collision_detect(
                g2,
                view,
                ghoul2,
                angles,
                position,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                ray_start,
                ray_end,
                scale,
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
                vmf(args, 12),
            );
            // Raven fills `collRecMap` in place and terminates the caller's scan
            // at the first `mEntityNum == -1`; the ported fn returns the
            // populated, distance-ordered records, copied back with the sentinel.
            let n = records.len().min(MAX_G2_COLLISIONS);
            for i in 0..n {
                *coll_rec_map.add(i) = records[i];
            }
            if n < MAX_G2_COLLISIONS {
                (*coll_rec_map.add(n)).mEntityNum = -1;
            }
            return 0;
        } else if trap == G::G_G2_COLLISIONDETECTCACHE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(2) as *mut CGhoul2Info_v);
            let coll_rec_map = vma(view.common, args, 1) as *mut CollisionRecord_t;
            let angles = *(vma(view.common, args, 3) as *const vec3_t);
            let position = *(vma(view.common, args, 4) as *const vec3_t);
            let ray_start = *(vma(view.common, args, 7) as *const vec3_t);
            let ray_end = *(vma(view.common, args, 8) as *const vec3_t);
            let scale = *(vma(view.common, args, 9) as *const vec3_t);
            let records = g2api_collision_detect_cache(
                g2,
                view,
                ghoul2,
                angles,
                position,
                *args.offset(5) as c_int,
                *args.offset(6) as c_int,
                ray_start,
                ray_end,
                scale,
                *args.offset(10) as c_int,
                *args.offset(11) as c_int,
                vmf(args, 12),
            );
            let n = records.len().min(MAX_G2_COLLISIONS);
            for i in 0..n {
                *coll_rec_map.add(i) = records[i];
            }
            if n < MAX_G2_COLLISIONS {
                (*coll_rec_map.add(n)).mEntityNum = -1;
            }
            return 0;
        } else if trap == G::G_G2_SETROOTSURFACE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let surface_name =
                core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                    .to_str()
                    .unwrap_or("");
            return g2api_set_root_surface(g2, view, ghoul2, *args.offset(2) as c_int, surface_name)
                as isize;
        } else if trap == G::G_G2_SETSURFACEONOFF as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let surface_name =
                core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or("");
            return g2api_set_surface_on_off(
                g2,
                view,
                ghoul2,
                surface_name,
                *args.offset(3) as c_int,
            ) as isize;
        } else if trap == G::G_G2_SETNEWORIGIN as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            return g2api_set_new_origin(g2, view, ghoul2, *args.offset(2) as c_int) as isize;
        } else if trap == G::G_G2_DOESBONEEXIST as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let info = ghoul2.get_mut(g2, *args.offset(2) as c_int) as *mut CGhoul2Info;
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                .to_str()
                .unwrap_or("");
            return g2api_does_bone_exist(g2, view, &mut *info, bone_name) as isize;
        } else if trap == G::G_G2_GETSURFACERENDERSTATUS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let info = ghoul2.get_mut(g2, *args.offset(2) as c_int) as *mut CGhoul2Info;
            let surface_name =
                core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                    .to_str()
                    .unwrap_or("");
            return g2api_get_surface_render_status(g2, view, &mut *info, surface_name) as isize;
        } else if trap == G::G_G2_ABSURDSMOOTHING as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            g2api_absurd_smoothing(g2, ghoul2, *args.offset(2) != 0);
            return 0;
        } else if trap == G::G_G2_SETRAGDOLL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let rd_paramst = vma(view.common, args, 2) as *mut sharedRagDollParams_t;
            if rd_paramst.is_null() {
                g2api_reset_ragdoll(g2, ghoul2);
                return 0;
            }
            // Raven copies the C-ified shared struct into the class-based
            // `CRagDollParams` (`sv_game.cpp:1503-1533`), casting the int
            // `RagPhase`/`effectorsToTurnOff` straight to the enum types (§19:
            // `effectorsToTurnOff` is a bit-or of flags, not a discriminant).
            let st = &*rd_paramst;
            let mut rd_params = CRagDollParams {
                angles: st.angles,
                position: st.position,
                scale: st.scale,
                pelvisAnglesOffset: st.pelvis_angles_offset,
                pelvisPositionOffset: st.pelvis_position_offset,
                fImpactStrength: st.f_impact_strength,
                fShotStrength: st.f_shot_strength,
                me: st.me,
                startFrame: st.start_frame,
                endFrame: st.end_frame,
                collisionType: st.collision_type,
                CallRagDollBegin: st.call_rag_doll_begin,
                RagPhase: core::mem::transmute::<c_int, sharedERagPhase>(st.rag_phase),
                effectorsToTurnOff: core::mem::transmute::<c_int, sharedERagEffector>(
                    st.effectors_to_turn_off,
                ),
            };
            g2api_set_ragdoll(g2, view, ghoul2, &mut rd_params);
            return 0;
        } else if trap == G::G_G2_ANIMATEG2MODELS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let rdu_paramst = vma(view.common, args, 3) as *mut sharedRagDollUpdateParams_t;
            if rdu_paramst.is_null() {
                return 0;
            }
            let st = &*rdu_paramst;
            let mut rdu_params = RagDollUpdateParams {
                angles: st.angles,
                position: st.position,
                scale: st.scale,
                velocity: st.velocity,
                me: st.me,
                settle_frame: st.settle_frame,
                kind: RagDollUpdateKind::Server,
            };
            g2api_animate_g2_models_rag(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                &mut rdu_params,
            );
            return 0;
        } else if trap == G::G_G2_RAGPCJCONSTRAINT as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            let min = *(vma(view.common, args, 3) as *const vec3_t);
            let max = *(vma(view.common, args, 4) as *const vec3_t);
            return g2api_rag_pcj_constraint(g2, ghoul2, bone_name, min, max) as isize;
        } else if trap == G::G_G2_RAGPCJGRADIENTSPEED as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            return g2api_rag_pcj_gradient_speed(g2, ghoul2, bone_name, vmf(args, 3)) as isize;
        } else if trap == G::G_G2_RAGEFFECTORGOAL as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            // Raven's `(float*)VMA(3)` goal is nullable; the ported fn takes an
            // `Option<vec3_t>`.
            let pos = if *args.offset(3) == 0 {
                None
            } else {
                Some(*(vma(view.common, args, 3) as *const vec3_t))
            };
            return g2api_rag_effector_goal(g2, ghoul2, bone_name, pos) as isize;
        } else if trap == G::G_G2_GETRAGBONEPOS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            let ent_angles = *(vma(view.common, args, 4) as *const vec3_t);
            let ent_pos = *(vma(view.common, args, 5) as *const vec3_t);
            let ent_scale = *(vma(view.common, args, 6) as *const vec3_t);
            let out = vma(view.common, args, 3) as *mut vec3_t;
            match g2api_get_rag_bone_pos(g2, ghoul2, bone_name, ent_angles, ent_pos, ent_scale) {
                Some(pos) => {
                    *out = pos;
                    return qtrue as isize;
                }
                None => return qfalse as isize,
            }
        } else if trap == G::G_G2_RAGEFFECTORKICK as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            let velocity = *(vma(view.common, args, 3) as *const vec3_t);
            return g2api_rag_effector_kick(g2, ghoul2, bone_name, velocity) as isize;
        } else if trap == G::G_G2_RAGFORCESOLVE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            return g2api_rag_force_solve(g2, ghoul2, *args.offset(2) != 0) as isize;
        } else if trap == G::G_G2_SETBONEIKSTATE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let bone_name = if *args.offset(3) == 0 {
                None
            } else {
                Some(
                    core::ffi::CStr::from_ptr(vma(view.common, args, 3) as *const c_char)
                        .to_str()
                        .unwrap_or(""),
                )
            };
            let params = if *args.offset(5) == 0 {
                None
            } else {
                Some(&mut *(vma(view.common, args, 5) as *mut sharedSetBoneIKStateParams_t))
            };
            return g2api_set_bone_ik_state(
                g2,
                view,
                ghoul2,
                *args.offset(2) as c_int,
                bone_name,
                *args.offset(4) as c_int,
                params,
            ) as isize;
        } else if trap == G::G_G2_IKMOVE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let params = &mut *(vma(view.common, args, 3) as *mut sharedIKMoveParams_t);
            return g2api_ik_move(g2, view, ghoul2, *args.offset(2) as c_int, params) as isize;
        } else if trap == G::G_G2_REMOVEBONE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let info = ghoul2.get_mut(g2, *args.offset(3) as c_int) as *mut CGhoul2Info;
            let bone_name = core::ffi::CStr::from_ptr(vma(view.common, args, 2) as *const c_char)
                .to_str()
                .unwrap_or("");
            return g2api_remove_bone(g2, view, &mut *info, bone_name) as isize;
        } else if trap == G::G_G2_ATTACHINSTANCETOENTNUM as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            g2api_attach_instance_to_ent_num(
                g2,
                ghoul2,
                *args.offset(2) as c_int,
                *args.offset(3) != 0,
            );
            return 0;
        } else if trap == G::G_G2_CLEARATTACHEDINSTANCE as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            g2api_clear_attached_instance(g2, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_G2_CLEANENTATTACHMENTS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            g2api_clean_ent_attachments(g2);
            return 0;
        } else if trap == G::G_G2_OVERRIDESERVER as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let info = ghoul2.get_mut(g2, 0) as *mut CGhoul2Info;
            return g2api_override_server_with_client_data(g2, &mut *info) as isize;
        } else if trap == G::G_G2_GETSURFACENAME as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live g2 cast.
            let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
            let ghoul2 = &mut *(*args.offset(1) as *mut CGhoul2Info_v);
            let point = vma(view.common, args, 4) as *mut c_char;
            let info = ghoul2.get_mut(g2, *args.offset(3) as c_int) as *mut CGhoul2Info;
            // Raven `if (local) strcpy(point, local)` — the ported fn returns an
            // owned `String` (empty when Raven's `local` is null), copied through.
            let name = g2api_get_surface_name(g2, view, &mut *info, *args.offset(2) as c_int);
            if !name.is_empty() {
                let bytes = name.as_bytes();
                core::ptr::copy_nonoverlapping(bytes.as_ptr() as *const c_char, point, bytes.len());
                *point.add(bytes.len()) = 0;
            }
            return 0;
        } else if trap == G::G_SET_ACTIVE_SUBBSP as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_SetActiveSubBSP(view.cm, sv, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_RMG_INIT as isize {
            // Raven: `TheRandomMissionManager` (the one `CRMManager`, lazily
            // `new`'d) is the owned `Engine.rmg` here (`view.rmg` opaque slot,
            // always present) — the lazy `if (!TheRandomMissionManager) new
            // CRMManager` collapses to that owned instance. `cmg.landScape` is
            // `view.cm.land_scape`; its terrain id is always `0` (`GetTerrainId`,
            // ruling 28), so the handle passed to `set_landscape` is
            // `TerrainHandle(0)`.
            //
            // The whole body is gated on `com_RMG` (`sv_game.cpp:1624-1638`).
            // Under DEDICATED `load_mission` always early-outs `false` (RMG-D1 /
            // ruling 25: `mTerrain` is always NULL, `GetRandomTerrain() == 0`),
            // so `SpawnMission(qtrue)` is unreachable and §20-dropped — the whole
            // generation subtree is dead code on the dedicated server.
            let com_rmg = view.common.com_RMG;
            if com_rmg.is_some() && view.common.cvar(com_rmg).integer != 0 {
                // SAFETY: view-constructor slots, single-threaded, no other live
                // cast. `load_mission`'s host calls (print/error) never touch
                // `view.cm` or `view.rmg`, so the DEC-23 per-slot raw reborrows of
                // `view`/`view.cm` (register_terrain-arm precedent) do not alias in
                // practice.
                let rmg = &mut *(view.rmg.as_raw() as *mut RmManager);
                rmg.set_landscape(TerrainHandle(0));
                let cm = &mut *(view.cm as *mut CollisionWorld);
                if rmg.load_mission(cm, &mut *view, qtrue != 0) {
                    // `SpawnMission(qtrue)` — dead under DEDICATED (RMG-D1 /
                    // ruling 25); `load_mission` never returns `true`, so this arm
                    // is unreachable and the mission-generation body is §20-dropped.
                    // Source: oracle/codemp/server/sv_game.cpp:1632-1634
                }
            }
            return 0;
        } else if trap == G::G_CM_REGISTER_TERRAIN as isize {
            let config_str = core::ffi::CStr::from_ptr(vma(view.common, args, 1) as *const c_char)
                .to_str()
                .unwrap_or("");
            // SAFETY: register_terrain's host calls (fs/print/error) never touch
            // view.cm; the raw reborrow follows the DEC-23 per-slot rule.
            let cm = &mut *(view.cm as *mut CollisionWorld);
            return register_terrain(cm, &mut *view, config_str, qtrue != 0).0 as isize;
        } else if trap == G::G_BOT_UPDATEWAYPOINTS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            SV_BotWaypointReception(
                sv,
                *args.offset(1) as c_int,
                vma(view.common, args, 2) as *mut *mut wpobject_t,
            );
            return 0;
        } else if trap == G::G_BOT_CALCULATEPATHS as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast;
            // `SV_BotCalculatePaths` reaches `sv.bot.gWP*` and `SV_Trace(view, …)`
            // exactly as `SV_SetBrushModel(view, sv, …)` does (SEAM-D11).
            let sv = sv_from_view(view);
            SV_BotCalculatePaths(view, sv, *args.offset(1) as c_int);
            return 0;
        } else if trap == G::G_GET_ENTITY_TOKEN as isize {
            // SAFETY: view-constructor slot, single-threaded, no other live cast.
            let sv = sv_from_view(view);
            return SV_GetEntityToken(
                sv,
                vma(view.common, args, 1) as *mut c_char,
                *args.offset(2) as c_int,
            ) as isize;
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
pub fn SV_InitGameProgs(view: &mut EngineHostView, sv: &mut Server) {
    let var = Cvar_Get(
        view,
        "bot_enable",
        "1",
        mp_qshared::shared::cvar::CVAR_LATCH,
    );
    view.common.bot_enable = view.common.cvar(var).integer;

    if Cvar_VariableValue(view.common, "fs_restrict") == 0.0
        && view.common.cvar(view.common.com_dedicated).integer == 0
        && Sys_CheckCD() == qfalse
    {
        let need_cd = SE_GetString(view, "CON_TEXT_NEED_CD");
        mp_engine_qcommon::common::com_error(errorParm_t::ERR_NEED_CD, need_cd);
        //"Game CD not in drive" );
    }

    // load the dll or bytecode
    let vm_game = Cvar_VariableValue(view.common, "vm_game");
    // SEAM-D11: the game slot is armed once at boot by
    // `mp_engine_core::install_engine_hooks` (with the `GameDispatchCtx` note);
    // re-arming here at map load with a raw `sv` pointer would clobber that note
    // with a wrong ctx, so it is not re-armed. `VM_Create`'s `systemCalls`
    // parameter (Raven `SV_GameSystemCalls`, `vm.cpp:471-472`) takes the C-ABI
    // `sv_game_system_call` adapter, which routes the legacy `VM_DllSyscall`
    // path to the same slot.
    sv.gvm =
        mp_engine_qcommon::vm_fns::VM_Create(view, "jampgame", Some(sv_game_system_call), unsafe {
            core::mem::transmute(vm_game as c_int)
        });
    if sv.gvm.is_null() {
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_FATAL,
            "VM_Create on game failed".to_string(),
        );
    }

    SV_InitGameVM(view, sv, qfalse);
}

/// Raven `SV_ShutdownGameProgs` — called every time a map changes. The hook
/// target (view signature); callers already holding the real `&mut Server`
/// (SV_SpawnServer / the SV_Shutdown body) use [`SV_ShutdownGameProgs_body`]
/// directly so no second cast of the `sv` slot is ever created.
///
/// Source: `oracle/codemp/server/sv_game.cpp:1665-1673`
pub fn SV_ShutdownGameProgs(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of this
    // slot for the borrow's duration (rule 7).
    let sv = unsafe { sv_from_view(view) };
    SV_ShutdownGameProgs_body(view.common, sv);
}

/// [`SV_ShutdownGameProgs`]'s body over the already-cast receivers.
pub fn SV_ShutdownGameProgs_body(common: &mut Common, sv: &mut Server) {
    if sv.gvm.is_null() {
        return;
    }
    VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_SHUTDOWN as c_int,
        &[qfalse as isize],
    );
    mp_engine_qcommon::vm_fns::VM_Free(common, sv.gvm);
    sv.gvm = core::ptr::null_mut();
}
