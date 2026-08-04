//! `cl_cgame.cpp` — the cgame VM host: system-call trap dispatch, snapshot and
//! usercmd handoff, and cgame lifecycle (init, shutdown, render, time).
//!
//! Source: `oracle/codemp/client/cl_cgame.cpp`

#![allow(non_snake_case, non_camel_case_types, unused_variables, unused_mut)]

use core::ffi::{c_char, c_int, c_void};
use std::sync::Arc;

use mp_abi::cgame::exports::MpCgameExport;
use mp_abi::cgame::imports::MpCgameImport;
use mp_abi::cgame::public::snapshot_t::{snapshot_t, MAX_ENTITIES_IN_SNAPSHOT};
use mp_abi::ui::exports::MpUiExport;
use mp_bg::public::configstring::{CS_G2BONES, CS_PLAYERS, CS_SERVERINFO, CS_SYSTEMINFO};
use mp_bg::public::entity_flags::EF_PERMANENT;
use mp_engine_ghoul2::api_bolts::{
    g2api_add_bolt, g2api_attach_ent, g2api_attach_instance_to_ent_num,
    g2api_clean_ent_attachments, g2api_clear_attached_instance, g2api_get_bolt_matrix,
    g2api_set_bolt_info, g2api_set_new_origin,
};
use mp_engine_ghoul2::api_bones::{
    g2api_does_bone_exist, g2api_get_bone_anim, g2api_list_bones, g2api_remove_bone,
    g2api_set_bone_angles, g2api_set_bone_anim,
};
use mp_engine_ghoul2::api_collision::{
    g2api_collision_detect, g2api_collision_detect_cache, g2api_get_time,
    g2api_override_server_with_client_data, g2api_set_time,
};
use mp_engine_ghoul2::api_models::{
    g2api_clean_ghoul2_models, g2api_copy_ghoul2_instance, g2api_copy_specific_g2_model,
    g2api_duplicate_ghoul2_instance, g2api_ghoul2_size, g2api_has_ghoul2_model_on_index,
    g2api_have_we_ghoul2_models, g2api_init_ghoul2_model, g2api_remove_ghoul2_model,
    g2api_set_ghoul2_model_indexes, g2api_set_skin, g2api_skinless_model,
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
use mp_engine_qcommon::cm_load::{
    CM_InlineModel, CM_LoadMap, CM_LoadSubBSP, CM_NumInlineModels, CM_TempBoxModel,
};
use mp_engine_qcommon::cm_terrain::register_terrain;
use mp_engine_qcommon::cm_test::{CM_PointContents, CM_TransformedPointContents};
use mp_engine_qcommon::cm_trace::{CM_BoxTrace, CM_TransformedBoxTrace};
use mp_engine_qcommon::cmd_common::{
    Cbuf_AddText, Cmd_Argc, Cmd_ArgsBuffer, Cmd_ArgsFrom, Cmd_Argv, Cmd_ArgvBuffer,
    Cmd_TokenizeString,
};
use mp_engine_qcommon::cmd_pc::{Cmd_AddCommand, Cmd_RemoveCommand};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common::error::com_error;
use mp_engine_qcommon::common_fns::{
    Com_DPrintf, Com_Memcpy, Com_Memset, Com_RealTime, Q_acos, Q_asin,
};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Register, Cvar_Set, Cvar_Update, Cvar_VariableStringBuffer, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Write};
use mp_engine_qcommon::files_pc::{FS_FOpenFileByMode, FS_GetFileList, FS_Read2};
use mp_engine_qcommon::qcommon::net_limits::{MAX_RELIABLE_COMMANDS, PACKET_BACKUP, PACKET_MASK};
use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t;
use mp_engine_qcommon::qcommon::vm_interpret_t::vmInterpret_t;
use mp_engine_qcommon::roff::RoffSystem;
use mp_engine_qcommon::stringed::api::{SE_GetString, SE_GetString2};
use mp_engine_qcommon::terrain_handle::TerrainHandle;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::timing::timing_c::timing_c;
use mp_engine_qcommon::vm::cgame_syscall_trampoline_words;
use mp_engine_qcommon::vm_fns::{
    VM_ArgPtrWord, VM_Call, VM_Create, VM_Debug, VM_Free, VM_Shifted_Alloc, VM_Shifted_Free,
};
use mp_engine_qcommon::z_memman_pc::{Com_TouchMemory, Hunk_MemoryRemaining};
use mp_engine_rmg::rm_manager::RmManager;
use mp_qshared::common::mp::cgame::glconfig_t::glconfig_t;
use mp_qshared::common::mp::cgame::poly_vert_t::polyVert_t;
use mp_qshared::common::mp::cgame::ref_entity_t::refEntity_t;
use mp_qshared::common::mp::cgame::refdef_t::refdef_t;
use mp_qshared::common::mp::cgame::stereo_frame_t::stereoFrame_t;
use mp_qshared::common::mp::qcommon::collision_record::{CollisionRecord_t, MAX_G2_COLLISIONS};
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::qtime::qtime_t;
use mp_qshared::common::mp::qcommon::shared_ragdoll_params::sharedRagDollParams_t;
use mp_qshared::common::mp::qcommon::shared_ragdoll_update_params::sharedRagDollUpdateParams_t;
use mp_qshared::common::mp::qcommon::shared_set_bone_ik_state_params::sharedSetBoneIKStateParams_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::common::mp::trace_t::trace_t;
use mp_qshared::shared::add_electricity_arg::addElectricityArgStruct_t;
use mp_qshared::shared::addbezier_arg::addbezierArgStruct_t;
use mp_qshared::shared::addpoly_arg::addpolyArgStruct_t;
use mp_qshared::shared::addsprite_arg::addspriteArgStruct_t;
use mp_qshared::shared::connstate::connstate_t;
use mp_qshared::shared::cvar::vmCvar_t;
use mp_qshared::shared::effect_trail_arg::effectTrailArgStruct_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::file_mode::fsMode_t;
use mp_qshared::shared::game_state::{gameState_t, MAX_CONFIGSTRINGS, MAX_GAMESTATE_CHARS};
use mp_qshared::shared::keycatch::KEYCATCH_CGAME;
use mp_qshared::shared::limits::{BIG_INFO_STRING, MAX_GENTITIES, SNAPFLAG_NOT_ACTIVE};
use mp_qshared::shared::q_math::Sys_SnapVector;
use mp_qshared::shared::q_string::Com_sprintf;
use mp_qshared::shared::shared_ik_move_params::sharedIKMoveParams_t;
use mp_qshared::shared::{pc_token_t, sharedERagEffector, sharedERagPhase};
use mp_renderer::hook_install::{re_from_view, rm_from_view};
use mp_renderer::render_state::frame_event::FrameEvent;
use mp_renderer::render_state::bmodel_table::BModelTable;
use mp_renderer::render_state::render_cvar_snapshot::RenderCvarSnapshot;
use mp_renderer::render_state::world_generation::WorldGeneration;
use mp_renderer::tr_bsp::{RE_LoadWorldMap, R_GetEntityToken};
use mp_renderer::tr_cmds::{RE_RotatePic, RE_RotatePic2, RE_SetColor, RE_StretchPic};
use mp_renderer::tr_font::{
    AnyLanguage_ReadCharFromString, GetLanguageEnum, Language_IsAsian, Language_UsesSpaces,
    RE_Font_DrawString, RE_Font_HeightPixels, RE_Font_StrLenChars, RE_Font_StrLenPixels,
    RE_RegisterFont,
};
use mp_renderer::tr_image::{RE_RegisterImages_LevelLoadEnd, RE_RegisterSkin};
use mp_renderer::tr_init::{RE_EndRegistration, RE_GetLightStyle, RE_SetLightStyle};
use mp_renderer::tr_light::R_LightForPoint;
use mp_renderer::tr_model::frontend::{r_lerp_tag, r_model_bounds, RE_RegisterModel};
use mp_renderer::tr_model::render_models::RenderModels;
use mp_renderer::tr_scene::{
    RE_AddAdditiveLightToScene, RE_AddLightToScene, RE_AddPolyToScene, RE_AddRefEntityToScene,
    RE_ClearDecals, RE_ClearScene, RE_RenderScene,
};
use mp_renderer::tr_shader::{RE_RegisterShader, RE_RegisterShaderNoMip, R_RemapShader};
use mp_renderer::tr_terrain::RE_InitRendererTerrain;
use mp_renderer::tr_world::{
    RE_GetBModelVerts, R_AutomapElevationAdjustment, R_InitializeWireframeAutomap, R_inPVS,
};
use native_math::eorientations::Eorientations;
use native_math::orientation::orientation_t;
use native_math::qmath::{vec3_origin, AngleVectors, MatrixMultiply, PerpendicularVectorMP};
use native_math::vector::vec3_t;
use native_string::atoi::atoi;
use native_string::info::Info_ValueForKey;
use native_string::q_string::Q_strcat;
use native_string::q_strncpyz::Q_strncpyz;
use native_types::{fileHandle_t, mdxaBone_t, qboolean, qfalse, qhandle_t, qtrue};

use crate::cl_cin::{
    CIN_DrawCinematic, CIN_PlayCinematic, CIN_RunCinematic, CIN_SetExtents, CIN_StopCinematic,
};
use crate::cl_console::{Con_ClearNotify, Con_Close};
use crate::cl_keys::{Key_GetKey, Key_IsDown};
use crate::cl_main::{CL_AddReliableCommand, CL_ReadDemoMessage};
use crate::cl_parse::{CL_GetValueForHidden, CL_SystemInfoChanged};
use crate::cl_referee::ref_headless;
use crate::cl_scrn::SCR_UpdateScreen;
use crate::cl_ui::{Key_GetCatcher, Key_SetCatcher};
use crate::client::cl_main_consts::MAX_STRINGED_SV_STRING;
use crate::client::cl_snapshot_t::clSnapshot_t;
use crate::client::client_consts::{CMD_BACKUP, CMD_MASK, MAX_PARSE_ENTITIES, RESET_TIME};
use crate::client_host::fx_from_view;
use crate::client_host::{
    bot_from_view, client_legacy_syscall, g2_from_view, sv_from_view, Client,
};
use crate::fx::ctrail::FX_FeedTrail;
use crate::fx::emat_impact_effect::EMatImpactEffect;
use crate::fx::fx_export::{
    FX_AddScheduledEffects, FX_AdjustTime, FX_Draw2DEffects, FX_FreeSystem, FX_InitSystem,
    FX_PlayBoltedEffectID, FX_PlayEffect, FX_PlayEffectID, FX_PlayEntityEffectID,
    FX_RegisterEffect, FX_SetRefDefFromCGame,
};
use crate::client_host::{cl_from_view, snd_from_view};
use crate::fx::fx_host::FxHost;
use crate::fx::fx_util::{
    FX_AddBezier, FX_AddElectricity, FX_AddLine, FX_AddParticle, FX_AddPoly, FX_Free,
};
use crate::snd_dma::{
    S_AddLoopingSound, S_ClearLoopingSounds, S_MuteSound, S_RegisterSound, S_Respatialize,
    S_StartLocalSound, S_StartSound, S_StopBackgroundTrack, S_StopLoopingSound,
    S_UpdateEntityPosition,
};
use crate::snd_ambient::{
    AS_AddPrecacheEntry, AS_GetBModelSound, AS_ParseSets, S_AddLocalSet, S_UpdateAmbientSet,
};
use crate::snd_dma::{S_RestartMusic, S_StartBackgroundTrack};

/// The `VMA(x)` macro: the module-space pointer the syscall word at `x` names.
///
/// The dispatcher hands this a raw `Common` copy, not a borrow, so a trap arm
/// can hold `&mut EngineHostView` and read an argument in the same expression.
/// `VM_ArgPtrWord` only reads the loaded module's data base, and it takes the
/// full-width word, because a 64-bit module hands us a 64-bit pointer.
///
/// Source: `oracle/codemp/qcommon/vm_local.h` (`VMA`)
fn vma(common: *const Common, args: *mut isize, i: isize) -> *mut () {
    // SAFETY: `common` is the view's own `Common`, alive for the whole
    // dispatch; `args` is the trampoline's 16-word frame (porting-rules §D11).
    unsafe { VM_ArgPtrWord(&*common, *args.offset(i)) }
}

/// The `VMF(x)` macro: the float bits the syscall word at `x` carries.
/// Raven indexes the arg block as floats (`((float *)args)[x]`), so the word
/// holds the bits themselves - it is never a pointer to translate.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:625` (`VMF`)
fn vmf(_common: *const Common, args: *mut isize, i: isize) -> f32 {
    // SAFETY: `args` is the trampoline's 16-word frame (porting-rules §D11).
    f32::from_bits(unsafe { *args.offset(i) } as u32)
}

/// Read a module-space origin, where a NULL pointer is Raven's "no origin".
/// `S_StartSound` sources the sound from the entity in that case.
fn vec3_from_module(p: *const vec3_t) -> Option<vec3_t> {
    if p.is_null() {
        return None;
    }
    // SAFETY: the module passed a live `vec3_t` across the seam.
    Some(unsafe { *p })
}

/// Read a module-space trace bound, where a NULL pointer is the origin.
///
/// Raven's trace entry points take `mins`/`maxs` as pointers and let a module
/// pass NULL for a point trace; `CM_Trace` substitutes `vec3_origin` for each
/// NULL bound. This port takes the bounds by value, so the substitution happens
/// where the seam reads the module pointer.
///
/// Source: `oracle/codemp/qcommon/cm_trace.cpp:1602-1608`
fn vec3_or_origin(p: *const vec3_t) -> vec3_t {
    if p.is_null() {
        return vec3_origin;
    }
    // SAFETY: the module passed a live `vec3_t` across the seam.
    unsafe { *p }
}

/// Read a module-space C string as an owned `String`, the shape every trap arm
/// that takes a `const char *` needs.
fn cstr_to_string(p: *const c_char) -> String {
    // SAFETY: the module passed a NUL-terminated string across the seam.
    unsafe { core::ffi::CStr::from_ptr(p).to_string_lossy().into_owned() }
}

/// Borrow a module-space C string as its Latin-1 bytes, the shape the `RE_Font_*`
/// signatures take. The bytes stay in module memory for the whole dispatch.
fn cstr_bytes<'a>(p: *const c_char) -> &'a [u8] {
    // SAFETY: the module passed a NUL-terminated string across the seam.
    unsafe { core::ffi::CStr::from_ptr(p).to_bytes() }
}

/// Reborrow one `CGhoul2Info` out of the handle's arena without keeping the
/// `Ghoul2System` borrow, so the same call can still pass `g2` as its own
/// receiver. This is the arena twin of the view's slot-cast discipline.
fn g2_info<'a>(g2: &mut Ghoul2System, ghoul2: &CGhoul2Info_v, index: c_int) -> &'a mut CGhoul2Info {
    let p = ghoul2.get_mut(g2, index) as *mut CGhoul2Info;
    // SAFETY: the arena slot outlives the dispatch, and no other borrow of the
    // same slot is live (single-threaded synchronous traps).
    unsafe { &mut *p }
}

/// The shared body of the three `CG_G2_GETBOLT*` arms, which differ only in the
/// `Ghoul2System` flag each sets before the call.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1265-1300`
fn get_bolt_matrix_arm(
    view: &mut EngineHostView,
    g2: &mut Ghoul2System,
    vc: *const Common,
    args: *mut isize,
    arg: &dyn Fn(isize) -> c_int,
) -> bool {
    // SAFETY: the handle, the model list, and the matrix out-param are all
    // module-space (porting-rules §D11).
    unsafe {
        let mut ghoul2 = CGhoul2Info_v::from_token(*args.offset(1) as *mut c_void);
        let bolt_matrix = &mut *(vma(vc, args, 4) as *mut mdxaBone_t);
        g2api_get_bolt_matrix(
            g2,
            view,
            &mut ghoul2,
            arg(2),
            arg(3),
            *(vma(vc, args, 5) as *const vec3_t),
            *(vma(vc, args, 6) as *const vec3_t),
            arg(7),
            core::slice::from_raw_parts(vma(vc, args, 8) as *const qhandle_t, 0),
            *(vma(vc, args, 9) as *const vec3_t),
            bolt_matrix,
        )
    }
}

/// Copy the collision hits back into the module's `CollisionRecord_t` array.
/// Raven's `G2API_CollisionDetect` writes that array in place, and the Rust
/// twin returns the hits, so the seam copy happens here.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1339-1353`
fn write_collision_records(out: *mut CollisionRecord_t, hits: &[CollisionRecord_t]) {
    if out.is_null() {
        return;
    }
    let count = hits.len().min(MAX_G2_COLLISIONS);
    // SAFETY: `VMA(1)` is the module's `CollisionRecord_t[MAX_G2_COLLISIONS]`
    // out-buffer (porting-rules §D11).
    unsafe { core::ptr::copy_nonoverlapping(hits.as_ptr(), out, count) };
}

/// Read a `float[4]` colour argument, which Raven passes as NULL to mean "no
/// colour". `RE_SetColor` and `RE_Font_DrawString` both take that as `None`.
fn rgba_arg(p: *const f32) -> Option<[f32; 4]> {
    if p.is_null() {
        return None;
    }
    // SAFETY: a non-NULL colour argument is the module's `float[4]` (§D11).
    unsafe { Some([*p, *p.add(1), *p.add(2), *p.add(3)]) }
}

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
    // `glconfig_t` is an ABI-frozen `#[repr(C)]` block with no `Copy`, so the
    // seam copy is the raw structure copy Raven's `*config = cls.glconfig` is.
    // SAFETY: `glconfig` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        core::ptr::copy_nonoverlapping(&cl.cls.glconfig as *const glconfig_t, glconfig, 1);
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
    if cmdNumber <= cl.cl.cmdNumber - CMD_BACKUP {
        return qfalse;
    }

    // SAFETY: `ucmd` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        *ucmd = cl.cl.cmds[(cmdNumber & CMD_MASK) as usize];
    }

    qtrue
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
    if parseEntityNumber <= cl.cl.parseEntitiesNum - MAX_PARSE_ENTITIES as c_int {
        return qfalse;
    }

    // SAFETY: `state` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        *state = cl.cl.parseEntities[(parseEntityNumber as usize) & (MAX_PARSE_ENTITIES - 1)];
    }
    qtrue
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
    common: &mut Common,
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
    if cl.cl.snap.messageNum - snapshotNumber >= PACKET_BACKUP as c_int {
        return qfalse;
    }

    // If the frame is not valid, we can't return it.
    let clSnap: &clSnapshot_t = &cl.cl.snapshots[(snapshotNumber as usize) & PACKET_MASK];
    if clSnap.valid == qfalse {
        return qfalse;
    }

    // If the entities in the frame have fallen out of their circular buffer, we can't return it.
    if cl.cl.parseEntitiesNum - clSnap.parseEntitiesNum >= MAX_PARSE_ENTITIES as c_int {
        return qfalse;
    }

    // SAFETY: `snapshot` is the VM's seam out-param pointer (porting-rules §D11).
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
                &mut *common,
                &format!(
                    "CL_GetSnapshot: truncated {} entities to {}\n",
                    count, MAX_ENTITIES_IN_SNAPSHOT
                ),
            );
            count = MAX_ENTITIES_IN_SNAPSHOT as c_int;
        }
        (*snapshot).numEntities = count;

        for i in 0..count {
            let entNum = ((clSnap.parseEntitiesNum + i) as usize) & (MAX_PARSE_ENTITIES - 1);
            // Copy everything but the ghoul2 pointer.
            (*snapshot).entities[i as usize] = cl.cl.parseEntities[entNum];
        }
    }

    // FIXME: configstring changes and server commands!!!

    qtrue
}

/// Raven `CL_GetDefaultState`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:210-225`
pub fn CL_GetDefaultState(cl: &mut Client, index: c_int, state: *mut entityState_t) -> qboolean {
    if index < 0 || index >= MAX_GENTITIES as c_int {
        return qfalse;
    }

    if cl.cl.entityBaselines[index as usize].eFlags & EF_PERMANENT == 0 {
        return qfalse;
    }

    // SAFETY: `state` is the VM's seam out-param pointer (porting-rules §D11).
    unsafe {
        *state = cl.cl.entityBaselines[index as usize];
    }

    qtrue
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
    cl.cl_mPitchOverride = mPitchOverride;
    cl.cl_mYawOverride = mYawOverride;
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
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:261-263`
pub fn CL_AddCgameCommand(view: &mut EngineHostView, cmdName: *const c_char) {
    let cmd_name = cstr_to_string(cmdName);
    Cmd_AddCommand(view, &cmd_name, None);
}

/// Raven `CL_CgameError`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:270-272`
pub fn CL_CgameError(string: *const c_char) {
    let s = cstr_to_string(string);
    com_error(errorParm_t::ERR_DROP, s);
}

/// Raven `CL_DoAutoLODScale`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:280-291`
pub fn CL_DoAutoLODScale(view: &mut EngineHostView, cl: &mut Client) {
    let mut finalLODScaleFactor: f32 = 0.0;

    if cl.gCLTotalClientNum >= 8 {
        finalLODScaleFactor = cl.gCLTotalClientNum as f32 / (-8.0f32 as f64) as f32;
    }

    Cvar_Set(
        view,
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
pub fn CL_CheckSVStringEdRef(view: &mut EngineHostView, buf: *mut c_char, str: *const c_char) {
    // SAFETY: `buf`/`str` are the caller's raw seam buffers (porting-rules §D11).
    unsafe {
        if str.is_null() || *str == 0 {
            if !str.is_null() {
                libc::strcpy(buf, str);
            }
            return;
        }

        libc::strcpy(buf, str);

        let strLen = libc::strlen(str) as isize;

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
                        let mut stringRef = [0 as c_char; MAX_STRINGED_SV_STRING as usize];
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
                        let string_ref = cstr_to_string(stringRef.as_ptr());
                        let replacement = SE_GetString2(view, "MP_SVGAME", &string_ref);
                        let buf_slice = core::slice::from_raw_parts_mut(
                            buf as *mut c_char,
                            MAX_STRINGED_SV_STRING as usize,
                        );
                        Q_strcat(buf_slice, MAX_STRINGED_SV_STRING as usize, &replacement);
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
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:583-587`
pub fn CL_CM_LoadMap(view: &mut EngineHostView, mapname: *const c_char) {
    let mut checksum: c_int = 0;
    let name = cstr_to_string(mapname);
    CM_LoadMap(view, &name, qtrue, &mut checksum);
}

/// Raven `CL_ShutdownCGame`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:595-607`
pub fn CL_ShutdownCGame(common: &mut Common, cl: &mut Client) {
    cl.cls.keyCatchers &= !KEYCATCH_CGAME;
    cl.cls.cgameStarted = qfalse;
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
pub fn CL_GameCommand(common: &mut Common, cl: &mut Client) -> qboolean {
    if cl.cgvm.is_null() {
        return qfalse;
    }

    VM_Call(
        common,
        cl.cgvm,
        MpCgameExport::CG_CONSOLE_COMMAND as c_int,
        &[],
    ) as qboolean
}

/// Raven `CL_CGameRendering`.
/// The view receiver is the `Ghoul2System` reach: `G2API_SetTime`'s Rust twin
/// takes the threaded `g2` state (ruling 40), which only the view carries.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1830-1845`
pub fn CL_CGameRendering(view: &mut EngineHostView, cl: &mut Client, stereo: stereoFrame_t) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let g2 = unsafe { g2_from_view(view) };

    // rww - RAGDOLL_BEGIN
    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        // Set the server time to match the client time, if we don't have a server going.
        g2api_set_time(g2, cl.cl.serverTime, 0);
    }
    g2api_set_time(g2, cl.cl.serverTime, 1);
    // rww - RAGDOLL_END

    VM_Call(
        view.common,
        cl.cgvm,
        MpCgameExport::CG_DRAW_ACTIVE_FRAME as c_int,
        &[
            cl.cl.serverTime as isize,
            stereo as isize,
            cl.clc.demoplaying as isize,
        ],
    );
    VM_Debug(view.common, 0);
}

/// Raven `CL_AdjustTimeDelta`.
/// Snaps the delta back hard on a big jump, halves it on a moderate one, and
/// nudges it by 1-2 msec on a small one so latency drifts smoothly.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1870-1927`
pub fn CL_AdjustTimeDelta(common: &mut Common, cl: &mut Client) {
    cl.cl.newSnapshots = qfalse;

    // The delta never drifts when replaying a demo.
    if cl.clc.demoplaying != qfalse {
        return;
    }

    // If the current time is WAY off, just correct to the current value.
    let resetTime = if common.cvar(common.com_sv_running).integer != 0 {
        100
    } else {
        RESET_TIME
    };
    let _ = resetTime; // Raven computes this but never reads it back (dead in the oracle too).

    let newDelta = cl.cl.snap.serverTime - cl.cls.realtime;
    let deltaDelta = (newDelta - cl.cl.serverTimeDelta).abs();

    if deltaDelta > RESET_TIME {
        cl.cl.serverTimeDelta = newDelta;
        cl.cl.oldServerTime = cl.cl.snap.serverTime; // FIXME: is this a problem for cgame?
        cl.cl.serverTime = cl.cl.snap.serverTime;
        if common.cvar(cl.cl_showTimeDelta).integer != 0 {
            com_printf(common, "<RESET> ");
        }
    } else if deltaDelta > 100 {
        // Fast adjust, cut the difference in half.
        if common.cvar(cl.cl_showTimeDelta).integer != 0 {
            com_printf(common, "<FAST> ");
        }
        cl.cl.serverTimeDelta = (cl.cl.serverTimeDelta + newDelta) >> 1;
    } else {
        // Slow drift adjust, only move 1 or 2 msec.
        // If any of the frames between this and the previous snapshot had to be extrapolated,
        // nudge our sense of time back a little. The granularity of +1 / -2 is too high for
        // timescale modified frametimes.
        if common.cvar(common.com_timescale).value == 0.0
            || common.cvar(common.com_timescale).value == 1.0
        {
            if cl.cl.extrapolatedSnapshot != qfalse {
                cl.cl.extrapolatedSnapshot = qfalse;
                cl.cl.serverTimeDelta -= 2;
            } else {
                // Otherwise, move our sense of time forward to minimize total latency.
                cl.cl.serverTimeDelta += 1;
            }
        }
    }

    if common.cvar(cl.cl_showTimeDelta).integer != 0 {
        com_printf(common, &format!("{} ", cl.cl.serverTimeDelta));
    }
}

/// Raven `CL_ConfigstringModified`.
/// Rebuilds the whole gamestate string table around the one changed index,
/// because Raven repacks `stringData` densely rather than patching in place.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:298-382`
pub fn CL_ConfigstringModified(view: &mut EngineHostView, cl: &mut Client) {
    let index = atoi(Cmd_Argv(view.common, 1));
    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        com_error(
            errorParm_t::ERR_DROP,
            "configstring > MAX_CONFIGSTRINGS".to_string(),
        );
    }
    // Get everything after "cs <num>".
    let s = Cmd_ArgsFrom(view.common, 2);

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
        // Raven copies `len + 1` bytes out of a NUL-terminated C buffer. A Rust
        // `String` carries no terminator, so the bytes and the NUL are written
        // separately. This is the `CL_ParseGamestate` fix at its twin site.
        let at = cl.cl.gameState.dataCount as usize;
        let dst = &mut cl.cl.gameState.stringData[at..at + len as usize + 1];
        for (slot, b) in dst.iter_mut().zip(dup.as_bytes()) {
            *slot = *b as c_char;
        }
        dst[len as usize] = 0;
        cl.cl.gameState.dataCount += len + 1;
    }

    if cl.cl_autolodscale.is_some() && view.common.cvar(cl.cl_autolodscale).integer != 0 {
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

            cl.gCLTotalClientNum = clientCount;

            CL_DoAutoLODScale(view, cl);
        }
    }

    if index == CS_SYSTEMINFO {
        // Parse serverId and other cvars.
        CL_SystemInfoChanged(view, cl);
    }
}

/// Raven `CL_GetServerCommand`.
/// `bcs0`/`bcs1`/`bcs2` reassemble a big configstring split across several
/// reliable commands into `bigConfigString` before re-tokenizing it as `cs`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:462-573`
pub fn CL_GetServerCommand(
    view: &mut EngineHostView,
    cl: &mut Client,
    serverCommandNumber: c_int,
) -> qboolean {
    // The fork-3 rotating-scratch static becomes an owned local; Raven's
    // cross-call persistence (bcs0 -> bcs1 -> bcs2) only spans one dispatch
    // chain, so a local re-created per call is behavior-preserving here.
    let mut bigConfigString: [c_char; BIG_INFO_STRING] = [0; BIG_INFO_STRING];

    // If we have irretrievably lost a reliable command, drop the connection.
    if serverCommandNumber <= cl.clc.serverCommandSequence - MAX_RELIABLE_COMMANDS as c_int {
        // When a demo record was started after the client got a whole bunch of reliable
        // commands then the client never got those first reliable commands.
        if cl.clc.demoplaying != qfalse {
            return qfalse;
        }
        let mut i = 0;
        while i < MAX_RELIABLE_COMMANDS {
            // Spew out the reliable command buffer.
            if cl.clc.reliableCommands[i][0] != 0 {
                let cmd = cstr_to_string(cl.clc.reliableCommands[i].as_ptr());
                com_printf(view.common, &format!("{}: {}\n", i, cmd));
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

    let mut s = cstr_to_string(
        cl.clc.serverCommands
            [(serverCommandNumber & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize]
            .as_ptr(),
    );
    cl.clc.lastExecutedServerCommand = serverCommandNumber;

    Com_DPrintf(
        view.common,
        &format!("serverCommand: {} : {}\n", serverCommandNumber, s),
    );

    loop {
        // rescan:
        Cmd_TokenizeString(view.common, &s);
        let cmd = Cmd_Argv(view.common, 0).to_string();

        if cmd == "disconnect" {
            let mut strEd: [c_char; MAX_STRINGED_SV_STRING as usize] =
                [0; MAX_STRINGED_SV_STRING as usize];
            // The argument is copied out first, because `CL_CheckSVStringEdRef`
            // needs the view mutably and `Cmd_Argv` borrows it.
            let arg1 = Cmd_Argv(view.common, 1).to_string();
            let arg1_c = std::ffi::CString::new(arg1).unwrap_or_default();
            CL_CheckSVStringEdRef(view, strEd.as_mut_ptr(), arg1_c.as_ptr());
            let str_ed = cstr_to_string(strEd.as_ptr());
            com_error(
                errorParm_t::ERR_SERVERDISCONNECT,
                format!(
                    "{}: {}\n",
                    SE_GetString(view, "MP_SVGAME_SERVER_DISCONNECTED"),
                    str_ed
                ),
            );
        }

        if cmd == "bcs0" {
            let msg = format!(
                "cs {} \"{}",
                Cmd_Argv(view.common, 1),
                Cmd_Argv(view.common, 2)
            );
            let msg_c = std::ffi::CString::new(msg).unwrap_or_default();
            unsafe {
                libc::strcpy(bigConfigString.as_mut_ptr(), msg_c.as_ptr());
            }
            return qfalse;
        }

        if cmd == "bcs1" {
            let arg = Cmd_Argv(view.common, 2).to_string();
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
            return qfalse;
        }

        if cmd == "bcs2" {
            let arg = Cmd_Argv(view.common, 2).to_string();
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
            s = cstr_to_string(bigConfigString.as_ptr());
            continue; // goto rescan
        }

        if cmd == "cs" {
            CL_ConfigstringModified(view, cl);
            // Reparse the string, because CL_ConfigstringModified may have done another Cmd_TokenizeString().
            Cmd_TokenizeString(view.common, &s);
            return qtrue;
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
            return qtrue;
        }

        // The clientLevelShot command is used during development to generate 128*128
        // screenshots from the intermission point of levels for the menu system to use.
        // We pass it along to the cgame to make apropriate adjustments, but we also clear
        // the console and notify lines here.
        if cmd == "clientLevelShot" {
            // Don't do it if we aren't running the server locally, otherwise malicious
            // remote servers could overwrite the existing thumbnails.
            if view.common.cvar(view.common.com_sv_running).integer == 0 {
                return qfalse;
            }
            // Close the console.
            Con_Close(view.common, cl);
            // Take a special screenshot next frame.
            Cbuf_AddText(
                view.common,
                "wait ; wait ; wait ; wait ; screenshot levelshot\n",
            );
            return qtrue;
        }

        // We may want to put a "connect to other server" command here.

        // Cgame can now act on the command.
        return qtrue;
    }
}

/// Raven `CL_InitCGame`.
/// Loads the cgame module against the interpreter the connected server used
/// (or `vm_cgame` off a pure server), then drives it through init to primed.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1743-1805`
pub fn CL_InitCGame(view: &mut EngineHostView, cl: &mut Client) {
    // Put away the console.
    Con_Close(view.common, cl);

    // Find the current mapname.
    // SAFETY: the offset table indexes the fixed `stringData` block.
    let info = unsafe {
        cstr_to_string(
            cl.cl
                .gameState
                .stringData
                .as_ptr()
                .offset(cl.cl.gameState.stringOffsets[CS_SERVERINFO as usize] as isize),
        )
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
    if cl.cl_connectedToPureServer != 0 {
        // Load the module type based on what the server is doing -rww.
        interpret = unsafe { core::mem::transmute::<c_int, vmInterpret_t>(cl.cl_connectedCGAME) };
    } else {
        interpret = unsafe {
            core::mem::transmute::<c_int, vmInterpret_t>(
                Cvar_VariableValue(view.common, "vm_cgame") as c_int,
            )
        };
    }
    // Raven's win32 client opens the module dll through the pak search before
    // the OS load, which marks that pak's FS_CGAME_REF for the pure reply
    // (`SV_VerifyPaks_f` expects it first, `sv_client.cpp:1301-1341`). The
    // disk-side dylib load skips the pak search, so the same open lands here.
    {
        let mut h: fileHandle_t = 0;
        FS_FOpenFileRead(view, "cgamex86.dll", &mut h, false);
        if h != 0 {
            FS_FCloseFile(view.common, h);
        }
    }
    cl.cgvm = VM_Create(
        view,
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
        view.common,
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
    //
    // Demo referee seam (`cl_referee.rs`): the platform shell that builds and
    // seats `Engine.re` is not ported (gh#22, DEC-56), so the renderer slot is
    // NULL under the headless rig. This gate goes away when the shell lands.
    if !ref_headless(cl) {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_EndRegistration(view.common, &re.cvars, &re.sim.published, &mut re.frame);
    }

    // Make sure everything is paged in.
    Com_TouchMemory(view.common);

    // Clear anything that got printed.
    Con_ClearNotify(cl);
}

/// Raven `CL_FirstSnapshot` — the connection process ends here, on the first
/// snapshot that carries entities.
///
/// Raven's `RE_RegisterMedia_LevelLoadEnd` is inlined as its three live calls.
/// `SND_RegisterAudio_LevelLoadEnd` waits on the sound lane (gh#24), and
/// `Sys_BeginProfiling` is empty outside the Mac build.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1936-1972`;
/// `oracle/codemp/renderer/tr_model.cpp:577-587`
pub fn CL_FirstSnapshot(view: &mut EngineHostView, cl: &mut Client) {
    // ignore snapshots that don't have entities
    if cl.cl.snap.snapFlags & SNAPFLAG_NOT_ACTIVE != 0 {
        return;
    }

    // SAFETY (both casts): view-constructor slots, single-threaded, no other
    // cast of the same slot is live across the calls.
    let re = unsafe { re_from_view(view) };
    let rm = unsafe { rm_from_view(view) };
    rm.models_level_load_end(view, false);
    RE_RegisterImages_LevelLoadEnd(Arc::make_mut(&mut re.sim.published), &mut re.img_state, view, rm);
    // SAFETY: view-constructor slot, single-threaded, no other live cast.
    let snd = unsafe { snd_from_view(view) };
    S_RestartMusic(view, snd);

    cl.cls.state = connstate_t::CA_ACTIVE;

    // set the timedelta so we are exactly on this first frame
    cl.cl.serverTimeDelta = cl.cl.snap.serverTime - cl.cls.realtime;
    cl.cl.oldServerTime = cl.cl.snap.serverTime;

    cl.clc.timeDemoBaseTime = cl.cl.snap.serverTime;

    // if this is the first frame of active play,
    // execute the contents of activeAction now
    // this is to allow scripting a timedemo to start right
    // after loading
    if !view.common.cvar(cl.cl_activeAction).string.is_empty() {
        let active_action = view.common.cvar(cl.cl_activeAction).string.clone();
        Cbuf_AddText(view.common, &active_action);
        Cvar_Set(view, "activeAction", "");
    }
}

/// Raven `CL_SetCGameTime`.
/// Derives `cl.serverTime` from `serverTimeDelta`, clamped so it never flows
/// backwards, then drains queued demo messages until caught up.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:1980-2105`
pub fn CL_SetCGameTime(view: &mut EngineHostView, cl: &mut Client) {
    // Getting a valid frame message ends the connection process.
    if cl.cls.state != connstate_t::CA_ACTIVE {
        if cl.cls.state != connstate_t::CA_PRIMED {
            return;
        }
        if cl.clc.demoplaying != qfalse {
            // We shouldn't get the first snapshot on the same frame as the gamestate,
            // because it causes a bad time skip.
            if cl.clc.firstDemoFrameSkipped == qfalse {
                cl.clc.firstDemoFrameSkipped = qtrue;
                return;
            }
            CL_ReadDemoMessage(view, cl);
        }
        if cl.cl.newSnapshots != qfalse {
            cl.cl.newSnapshots = qfalse;
            CL_FirstSnapshot(view, cl);
        }
        if cl.cls.state != connstate_t::CA_ACTIVE {
            return;
        }
    }

    // If we have gotten to this point, cl.snap is guaranteed to be valid.
    if cl.cl.snap.valid == qfalse {
        com_error(
            errorParm_t::ERR_DROP,
            "CL_SetCGameTime: !cl.snap.valid".to_string(),
        );
    }

    // Allow pause in single player.
    // `sv_paused`/`cl_paused` are `Common` file-scope cvar handles, not `Client`
    // fields (CLIENT CARRIER RULE).
    if view.common.cvar(view.common.sv_paused).integer != 0
        && view.common.cvar(view.common.cl_paused).integer != 0
        && view.common.cvar(view.common.com_sv_running).integer != 0
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
    if cl.clc.demoplaying != qfalse && view.common.cvar(cl.cl_freezeDemo).integer != 0 {
        // cl_freezeDemo is used to lock a demo in place for single frame advances.
    } else {
        // cl_timeNudge is a user adjustable cvar that allows more or less latency to be
        // added in the interest of better smoothness or better responsiveness.
        let mut tn = view.common.cvar(cl.cl_timeNudge).integer;
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
            cl.cl.extrapolatedSnapshot = qtrue;
        }
    }

    // If we have gotten new snapshots, drift serverTimeDelta. Don't do this every frame, or
    // a period of packet loss would make a huge adjustment.
    if cl.cl.newSnapshots != qfalse {
        CL_AdjustTimeDelta(view.common, cl);
    }

    if cl.clc.demoplaying == qfalse {
        return;
    }

    // If we are playing a demo back, we can just keep reading messages from the demo file
    // until the cgame definately has valid snapshots to interpolate between.

    // A timedemo will always use a deterministic set of time samples no matter what speed
    // machine it is run on, while a normal demo may have different time samples each time
    // it is played back.
    if view.common.cvar(cl.cl_timedemo).integer != 0 {
        if cl.clc.timeDemoStart == 0 {
            cl.clc.timeDemoStart = sys_milliseconds(view.common);
        }
        cl.clc.timeDemoFrames += 1;
        cl.cl.serverTime = cl.clc.timeDemoBaseTime + cl.clc.timeDemoFrames * 50;
    }

    while cl.cl.serverTime >= cl.cl.snap.serverTime {
        // Feed another message, which should change the contents of cl.snap.
        CL_ReadDemoMessage(view, cl);
        if cl.cls.state != connstate_t::CA_ACTIVE {
            return; // end of demo
        }
    }
}

/// The `int (*)(int*)` C-ABI adapter handed to `VM_Create` as `systemCalls`
/// (`vm.cpp:471-472`, stored `vm->systemCall`). On the SEAM-D11 native path the
/// module reaches the engine through `cgame_syscall_trampoline` → the armed
/// cgame slot, so `vm->systemCall` (the legacy `VM_DllSyscall` target,
/// `vm.cpp:363-380`) is vestigial; this adapter widens the legacy contiguous
/// int arg block to the trampoline's `isize` words and forwards to the same
/// armed slot for parity if ever invoked. The real receivers come from the
/// boot-armed `ClientDispatchCtx` note, which `cgame_system_calls_shim` reads
/// (DEC-55.1) — the twin of the server's `sv_game_system_call`.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:644`
extern "C" fn CL_CgameSystemCalls_trampoline(args: *mut c_int) -> c_int {
    // SAFETY: the legacy `VM_DllSyscall` convention passes a contiguous 16-int
    // arg block (`args[i] = va_arg(...)`, vm.cpp:366).
    unsafe { client_legacy_syscall(args, cgame_syscall_trampoline_words) }
}

/// Raven `CL_CgameSystemCalls`.
/// The cgame VM's syscall trap dispatcher: one `args[0]` op code per Raven
/// `CG_*`/`TRAP_*` constant, routed to the matching engine call.
///
/// Source: `oracle/codemp/client/cl_cgame.cpp:644-1733`
#[allow(clippy::too_many_arguments)]
pub fn CL_CgameSystemCalls(
    view: &mut EngineHostView,
    cl: &mut Client,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    roff: &mut RoffSystem,
    args: *mut isize,
) -> c_int {
    // The raw `Common` copy the `VMA`/`VMF` helpers read. It is never used as a
    // receiver, so no arm holds a `Common` borrow beside the view.
    let vc: *const Common = view.common;
    let arg = |i: isize| -> c_int {
        // SAFETY: `args` is the trampoline's 16-word frame (porting-rules §D11).
        // A value argument is `int`-wide on Raven's side, so the word narrows
        // here the way the dispatch case would read it.
        unsafe { *args.offset(i) as c_int }
    };
    // The full-width word, for the arms that carry a host handle rather than a
    // value. The server dispatcher reads its handles the same way.
    let argw = |i: isize| -> isize {
        // SAFETY: same frame as `arg` above.
        unsafe { *args.offset(i) }
    };
    let op = arg(0);

    // rww - alright, DO NOT EVER add a GAME/CGAME/UI generic call without adding a trap to
    // match, and all of these traps must be shared and have cases in sv_game, cl_cgame, and
    // cl_ui. They must also all be in the same order, and start at 100.
    if op == sharedTraps_t::TRAP_MEMSET as c_int {
        unsafe { Com_Memset(vma(vc, args, 1), arg(2), arg(3) as usize) };
        0
    } else if op == sharedTraps_t::TRAP_MEMCPY as c_int {
        unsafe {
            Com_Memcpy(
                vma(vc, args, 1),
                vma(vc, args, 2) as *const (),
                arg(3) as usize,
            )
        };
        0
    } else if op == sharedTraps_t::TRAP_STRNCPY as c_int {
        unsafe {
            libc::strncpy(
                vma(vc, args, 1) as *mut c_char,
                vma(vc, args, 2) as *const c_char,
                arg(3) as usize,
            ) as isize as c_int
        }
    } else if op == sharedTraps_t::TRAP_SIN as c_int {
        FloatAsInt(vmf(vc, args, 1).sin())
    } else if op == sharedTraps_t::TRAP_COS as c_int {
        FloatAsInt(vmf(vc, args, 1).cos())
    } else if op == sharedTraps_t::TRAP_ATAN2 as c_int {
        FloatAsInt(vmf(vc, args, 1).atan2(vmf(vc, args, 2)))
    } else if op == sharedTraps_t::TRAP_SQRT as c_int {
        FloatAsInt(vmf(vc, args, 1).sqrt())
    } else if op == sharedTraps_t::TRAP_MATRIXMULTIPLY as c_int {
        unsafe {
            MatrixMultiply(
                &*(vma(vc, args, 1) as *const [[f32; 3]; 3]),
                &*(vma(vc, args, 2) as *const [[f32; 3]; 3]),
                &mut *(vma(vc, args, 3) as *mut [[f32; 3]; 3]),
            );
        }
        0
    } else if op == sharedTraps_t::TRAP_ANGLEVECTORS as c_int {
        unsafe {
            let angles = *(vma(vc, args, 1) as *const vec3_t);
            AngleVectors(
                angles,
                (vma(vc, args, 2) as *mut vec3_t).as_mut(),
                (vma(vc, args, 3) as *mut vec3_t).as_mut(),
                (vma(vc, args, 4) as *mut vec3_t).as_mut(),
            );
        }
        0
    } else if op == sharedTraps_t::TRAP_PERPENDICULARVECTOR as c_int {
        unsafe {
            PerpendicularVectorMP(
                &mut *(vma(vc, args, 1) as *mut vec3_t),
                *(vma(vc, args, 2) as *const vec3_t),
            )
        };
        0
    } else if op == sharedTraps_t::TRAP_FLOOR as c_int {
        FloatAsInt(vmf(vc, args, 1).floor())
    } else if op == sharedTraps_t::TRAP_CEIL as c_int {
        FloatAsInt(vmf(vc, args, 1).ceil())
    } else if op == sharedTraps_t::TRAP_TESTPRINTINT as c_int {
        0
    } else if op == sharedTraps_t::TRAP_TESTPRINTFLOAT as c_int {
        0
    } else if op == sharedTraps_t::TRAP_ACOS as c_int {
        FloatAsInt(Q_acos(vmf(vc, args, 1)))
    } else if op == sharedTraps_t::TRAP_ASIN as c_int {
        FloatAsInt(Q_asin(vmf(vc, args, 1)))
    } else if op == MpCgameImport::CG_PRINT as c_int {
        let s = cstr_to_string(vma(vc, args, 1) as *const c_char);
        com_printf(view.common, &s);
        0
    } else if op == MpCgameImport::CG_ERROR as c_int {
        let s = cstr_to_string(vma(vc, args, 1) as *const c_char);
        com_error(errorParm_t::ERR_DROP, s);
    } else if op == MpCgameImport::CG_MILLISECONDS as c_int {
        sys_milliseconds(view.common)
    } else if op == MpCgameImport::CG_PRECISIONTIMER_START as c_int {
        // rww - precision timer funcs... -ALWAYS- call end after start with supplied ptr, or
        // you'll get a nasty memory leak. Not that you should be using these outside of
        // debug anyway.. because you shouldn't be. So don't.
        unsafe {
            let suppliedPtr = vma(vc, args, 1) as *mut *mut timing_c;
            let newTimer = Box::into_raw(Box::new(timing_c::default()));
            *suppliedPtr = newTimer;
            (*newTimer).Start();
        }
        0
    } else if op == MpCgameImport::CG_PRECISIONTIMER_END as c_int {
        unsafe {
            let timer = argw(1) as *mut timing_c;
            let r = (*timer).End();
            drop(Box::from_raw(timer));
            r
        }
    } else if op == MpCgameImport::CG_CVAR_REGISTER as c_int {
        let name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        let value = cstr_to_string(vma(vc, args, 3) as *const c_char);
        let cvar = vma(vc, args, 1) as *mut vmCvar_t;
        Cvar_Register(view, cvar, &name, &value, arg(4));
        0
    } else if op == MpCgameImport::CG_CVAR_UPDATE as c_int {
        Cvar_Update(view.common, vma(vc, args, 1) as *mut vmCvar_t);
        0
    } else if op == MpCgameImport::CG_CVAR_SET as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let value = cstr_to_string(vma(vc, args, 2) as *const c_char);
        Cvar_Set(view, &name, &value);
        0
    } else if op == MpCgameImport::CG_CVAR_VARIABLESTRINGBUFFER as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        Cvar_VariableStringBuffer(view.common, &name, vma(vc, args, 2) as *mut c_char, arg(3));
        0
    } else if op == MpCgameImport::CG_CVAR_GETHIDDENVALUE as c_int {
        CL_GetValueForHidden(cl, vma(vc, args, 1) as *const c_char)
    } else if op == MpCgameImport::CG_ARGC as c_int {
        Cmd_Argc(view.common)
    } else if op == MpCgameImport::CG_ARGV as c_int {
        // SAFETY: `VMA(2)` is the module's seam out-buffer (porting-rules §D11).
        unsafe { Cmd_ArgvBuffer(view.common, arg(1), vma(vc, args, 2) as *mut c_char, arg(3)) };
        0
    } else if op == MpCgameImport::CG_ARGS as c_int {
        let s = Cmd_ArgsBuffer(view.common, arg(2) as usize);
        let s_c = std::ffi::CString::new(s).unwrap_or_default();
        unsafe { libc::strcpy(vma(vc, args, 1) as *mut c_char, s_c.as_ptr()) };
        0
    } else if op == MpCgameImport::CG_FS_FOPENFILE as c_int {
        let path = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let handle = vma(vc, args, 2) as *mut fileHandle_t;
        let mode = unsafe { core::mem::transmute::<c_int, fsMode_t>(arg(3)) };
        FS_FOpenFileByMode(view, &path, handle, mode)
    } else if op == MpCgameImport::CG_FS_READ as c_int {
        FS_Read2(
            view.common,
            vma(vc, args, 1),
            arg(2),
            arg(3) as fileHandle_t,
        );
        0
    } else if op == MpCgameImport::CG_FS_WRITE as c_int {
        FS_Write(
            view.common,
            vma(vc, args, 1) as *const (),
            arg(2),
            arg(3) as fileHandle_t,
        );
        0
    } else if op == MpCgameImport::CG_FS_FCLOSEFILE as c_int {
        FS_FCloseFile(view.common, arg(1) as fileHandle_t);
        0
    } else if op == MpCgameImport::CG_FS_GETFILELIST as c_int {
        let path = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let ext = cstr_to_string(vma(vc, args, 2) as *const c_char);
        let listbuf = vma(vc, args, 3) as *mut c_char;
        FS_GetFileList(view, &path, &ext, listbuf, arg(4))
    } else if op == MpCgameImport::CG_SENDCONSOLECOMMAND as c_int {
        let s = cstr_to_string(vma(vc, args, 1) as *const c_char);
        Cbuf_AddText(view.common, &s);
        0
    } else if op == MpCgameImport::CG_ADDCOMMAND as c_int {
        CL_AddCgameCommand(view, vma(vc, args, 1) as *const c_char);
        0
    } else if op == MpCgameImport::CG_REMOVECOMMAND as c_int {
        let s = cstr_to_string(vma(vc, args, 1) as *const c_char);
        Cmd_RemoveCommand(view.common, &s);
        0
    } else if op == MpCgameImport::CG_SENDCLIENTCOMMAND as c_int {
        let s = cstr_to_string(vma(vc, args, 1) as *const c_char);
        CL_AddReliableCommand(cl, &s);
        0
    } else if op == MpCgameImport::CG_UPDATESCREEN as c_int {
        // This is used during lengthy level loading, so pump message loop.
        // FIXME: if a server restarts here, BAD THINGS HAPPEN!
        // We can't call Com_EventLoop here, a restart will crash and this _does_ happen if
        // there is a map change while we are downloading at pk3. -ZOID
        SCR_UpdateScreen(view, cl);
        0
    } else if op == MpCgameImport::CG_CM_LOADMAP as c_int {
        if arg(2) != 0 {
            let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
            CM_LoadSubBSP(view, &format!("maps/{}.bsp", &name[1..]), qfalse);
        } else {
            CL_CM_LoadMap(view, vma(vc, args, 1) as *const c_char);
        }
        0
    } else if op == MpCgameImport::CG_CM_NUMINLINEMODELS as c_int {
        CM_NumInlineModels(view.cm)
    } else if op == MpCgameImport::CG_CM_INLINEMODEL as c_int {
        CM_InlineModel(view.cm, arg(1))
    } else if op == MpCgameImport::CG_CM_TEMPBOXMODEL as c_int {
        CM_TempBoxModel(
            view.cm,
            unsafe { *(vma(vc, args, 1) as *const vec3_t) },
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            qfalse,
        )
    } else if op == MpCgameImport::CG_CM_TEMPCAPSULEMODEL as c_int {
        CM_TempBoxModel(
            view.cm,
            unsafe { *(vma(vc, args, 1) as *const vec3_t) },
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            qtrue,
        )
    } else if op == MpCgameImport::CG_CM_POINTCONTENTS as c_int {
        CM_PointContents(
            view.cm,
            unsafe { *(vma(vc, args, 1) as *const vec3_t) },
            arg(2),
        )
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDPOINTCONTENTS as c_int {
        CM_TransformedPointContents(
            view.cm,
            unsafe { *(vma(vc, args, 1) as *const vec3_t) },
            arg(2),
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            unsafe { *(vma(vc, args, 4) as *const vec3_t) },
        )
    } else if op == MpCgameImport::CG_CM_BOXTRACE as c_int {
        CM_BoxTrace(
            view,
            vma(vc, args, 1) as *mut trace_t,
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            vec3_or_origin(vma(vc, args, 4) as *const vec3_t),
            vec3_or_origin(vma(vc, args, 5) as *const vec3_t),
            arg(6),
            arg(7),
            qfalse as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_CAPSULETRACE as c_int {
        CM_BoxTrace(
            view,
            vma(vc, args, 1) as *mut trace_t,
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            vec3_or_origin(vma(vc, args, 4) as *const vec3_t),
            vec3_or_origin(vma(vc, args, 5) as *const vec3_t),
            arg(6),
            arg(7),
            qtrue as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDBOXTRACE as c_int {
        CM_TransformedBoxTrace(
            view,
            vma(vc, args, 1) as *mut trace_t,
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            vec3_or_origin(vma(vc, args, 4) as *const vec3_t),
            vec3_or_origin(vma(vc, args, 5) as *const vec3_t),
            arg(6),
            arg(7),
            unsafe { *(vma(vc, args, 8) as *const vec3_t) },
            unsafe { *(vma(vc, args, 9) as *const vec3_t) },
            qfalse as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_TRANSFORMEDCAPSULETRACE as c_int {
        CM_TransformedBoxTrace(
            view,
            vma(vc, args, 1) as *mut trace_t,
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            vec3_or_origin(vma(vc, args, 4) as *const vec3_t),
            vec3_or_origin(vma(vc, args, 5) as *const vec3_t),
            arg(6),
            arg(7),
            unsafe { *(vma(vc, args, 8) as *const vec3_t) },
            unsafe { *(vma(vc, args, 9) as *const vec3_t) },
            qtrue as c_int,
        );
        0
    } else if op == MpCgameImport::CG_CM_MARKFRAGMENTS as c_int {
        //TODO: Port R_MarkFragments world root
        // Source: oracle/codemp/client/cl_cgame.cpp:719 (`re.MarkFragments`)
        // `R_MarkFragments` takes a `world_root: &mut MarkNode`, and `MarkNode`
        // is still the scoped-local stand-in `tr_marks.rs` declares. No carrier
        // owns a root, so the arm reports zero fragments until the renderer
        // census merges that node arena into `RenderAssets::world` (gh#31).
        0
    } else if op == MpCgameImport::CG_S_GETVOICEVOLUME as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        snd.s_entityWavVol[arg(1) as usize]
    } else if op == MpCgameImport::CG_S_MUTESOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_MuteSound(view.common, snd, arg(1), arg(2));
        0
    } else if op == MpCgameImport::CG_S_STARTSOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        let origin = vec3_from_module(vma(vc, args, 1) as *const vec3_t);
        S_StartSound(view, snd, origin, arg(2), arg(3), arg(4));
        0
    } else if op == MpCgameImport::CG_S_STARTLOCALSOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StartLocalSound(view, snd, arg(1), arg(2));
        0
    } else if op == MpCgameImport::CG_S_CLEARLOOPINGSOUNDS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_ClearLoopingSounds(snd);
        0
    } else if op == MpCgameImport::CG_S_ADDLOOPINGSOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_AddLoopingSound(
            view,
            snd,
            arg(1),
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_ADDREALLOOPINGSOUND as c_int {
        // S_AddRealLoopingSound(args[1], (const float *)VMA(2), (const float *)VMA(3), args[4]);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_AddLoopingSound(
            view,
            snd,
            arg(1),
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_STOPLOOPINGSOUND as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopLoopingSound(snd, arg(1));
        0
    } else if op == MpCgameImport::CG_S_UPDATEENTITYPOSITION as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_UpdateEntityPosition(snd, arg(1), unsafe {
            *(vma(vc, args, 2) as *const vec3_t)
        });
        0
    } else if op == MpCgameImport::CG_S_RESPATIALIZE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_Respatialize(
            view.common,
            snd,
            arg(1),
            unsafe { *(vma(vc, args, 2) as *const vec3_t) },
            unsafe { *(vma(vc, args, 3) as *const [vec3_t; 3]) },
            arg(4),
        );
        0
    } else if op == MpCgameImport::CG_S_SHUTUP as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        snd.s_shutUp = arg(1) != 0;
        0
    } else if op == MpCgameImport::CG_S_REGISTERSOUND as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_RegisterSound(view, snd, &name)
    } else if op == MpCgameImport::CG_S_STARTBACKGROUNDTRACK as c_int {
        let a = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let b = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StartBackgroundTrack(view, snd, &a, &b, arg(3) != 0);
        0
    } else if op == MpCgameImport::CG_S_UPDATEAMBIENTSET as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let origin = unsafe { *(vma(vc, args, 2) as *const vec3_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let realtime = unsafe { cl_from_view(view) }.cls.realtime;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_UpdateAmbientSet(view, snd, &name, origin, realtime);
        0
    } else if op == MpCgameImport::CG_AS_PARSESETS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        AS_ParseSets(view, snd);
        0
    } else if op == MpCgameImport::CG_AS_ADDPRECACHEENTRY as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        AS_AddPrecacheEntry(&mut snd.ambient, &name);
        0
    } else if op == MpCgameImport::CG_S_ADDLOCALSET as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let listener_origin = unsafe { *(vma(vc, args, 2) as *const vec3_t) };
        let origin = unsafe { *(vma(vc, args, 3) as *const vec3_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let realtime = unsafe { cl_from_view(view) }.cls.realtime;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_AddLocalSet(
            view,
            snd,
            &name,
            listener_origin,
            origin,
            arg(4),
            arg(5),
            realtime,
        )
    } else if op == MpCgameImport::CG_AS_GETBMODELSOUND as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        AS_GetBModelSound(&snd.ambient, &name, arg(2))
    } else if op == MpCgameImport::CG_R_LOADWORLDMAP as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_LoadWorldMap(
            &mut re.qs,
            &mut re.world_load,
            &mut re.scene,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.world_effects,
            &name,
        );
        // W2-F7: hand the render thread the new world and the brush-submodel
        // rows this load registered. `RE_EndFrame` moves them onto the next
        // package, so the render thread uploads the geometry once.
        re.pending_world = Some(WorldGeneration {
            world: re.sim.published.world.clone(),
            bmodels: BModelTable::build(rm),
        });
        0
    } else if op == MpCgameImport::CG_R_REGISTERMODEL as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RegisterModel(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.world_effects,
            &name,
        )
    } else if op == MpCgameImport::CG_R_REGISTERSKIN as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RegisterSkin(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &name,
        )
    } else if op == MpCgameImport::CG_R_REGISTERSHADER as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RegisterShader(
            &name,
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
        )
    } else if op == MpCgameImport::CG_R_REGISTERSHADERNOMIP as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RegisterShaderNoMip(
            &name,
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
        )
    } else if op == MpCgameImport::CG_R_REGISTERFONT as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_RegisterFont(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.font,
            language,
            mod_count,
            &name,
        )
    } else if op == MpCgameImport::CG_R_FONT_STRLENPIXELS as c_int {
        let text = cstr_bytes(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_Font_StrLenPixels(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.font,
            language,
            mod_count,
            text,
            arg(2),
            vmf(vc, args, 3),
        )
    } else if op == MpCgameImport::CG_R_FONT_STRLENCHARS as c_int {
        let text = cstr_bytes(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        RE_Font_StrLenChars(&re.font, language, text)
    } else if op == MpCgameImport::CG_R_FONT_STRHEIGHTPIXELS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        RE_Font_HeightPixels(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.font,
            language,
            mod_count,
            arg(1),
            vmf(vc, args, 2),
        )
    } else if op == MpCgameImport::CG_R_FONT_DRAWSTRING as c_int {
        let text = cstr_bytes(vma(vc, args, 3) as *const c_char);
        let rgba = rgba_arg(vma(vc, args, 4) as *const f32);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let mod_count = re.font.iSE_Language_ModificationCount.unwrap_or(-1234);
        let millis = sys_milliseconds(view.common);
        RE_Font_DrawString(
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
            &mut re.font,
            language,
            mod_count,
            &mut re.frame_data,
            arg(1),
            arg(2),
            text,
            rgba,
            arg(5),
            arg(6),
            vmf(vc, args, 7),
            millis,
        );
        0
    } else if op == MpCgameImport::CG_LANGUAGE_ISASIAN as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        Language_IsAsian(language) as c_int
    } else if op == MpCgameImport::CG_LANGUAGE_USESSPACES as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        Language_UsesSpaces(language) as c_int
    } else if op == MpCgameImport::CG_ANYLANGUAGE_READCHARFROMSTRING as c_int {
        let text = cstr_bytes(vma(vc, args, 1) as *const c_char);
        let advance_out = vma(vc, args, 2) as *mut c_int;
        let punctuation_out = vma(vc, args, 3) as *mut qboolean;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let language = GetLanguageEnum(view.common, &mut re.font);
        let (uiLetter, advance, trailing) =
            AnyLanguage_ReadCharFromString(&re.font, language, text, !punctuation_out.is_null());
        // SAFETY: both are the module's seam out-params (porting-rules §D11).
        unsafe {
            if !advance_out.is_null() {
                *advance_out = advance;
            }
            if !punctuation_out.is_null() {
                *punctuation_out = trailing.unwrap_or(false) as qboolean;
            }
        }
        uiLetter as c_int
    } else if op == MpCgameImport::CG_R_CLEARSCENE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_ClearScene(&mut re.frame_data, &mut re.scene);
        0
    } else if op == MpCgameImport::CG_R_CLEARDECALS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_ClearDecals(&mut re.scene);
        0
    } else if op == MpCgameImport::CG_R_ADDREFENTITYTOSCENE as c_int {
        let ent = vma(vc, args, 1) as *const refEntity_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `refEntity_t` (porting-rules §D11).
        RE_AddRefEntityToScene(&mut re.frame_data, &re.sim.published, &mut re.scene, unsafe {
            &*ent
        });
        0
    } else if op == MpCgameImport::CG_R_ADDPOLYTOSCENE as c_int {
        let num_verts = arg(2) as usize;
        // SAFETY: `VMA(3)` is the module's `polyVert_t` run (porting-rules §D11).
        let verts = unsafe {
            core::slice::from_raw_parts(vma(vc, args, 3) as *const polyVert_t, num_verts)
        };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddPolyToScene(
            &mut re.frame_data,
            &re.sim.published,
            view.common,
            arg(1),
            verts,
            num_verts,
            1,
        );
        0
    } else if op == MpCgameImport::CG_R_ADDPOLYSTOSCENE as c_int {
        let num_verts = arg(2) as usize;
        let num_polys = arg(4) as usize;
        // SAFETY: `VMA(3)` is the module's `polyVert_t` run (porting-rules §D11).
        let verts = unsafe {
            core::slice::from_raw_parts(
                vma(vc, args, 3) as *const polyVert_t,
                num_verts * num_polys,
            )
        };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddPolyToScene(
            &mut re.frame_data,
            &re.sim.published,
            view.common,
            arg(1),
            verts,
            num_verts,
            num_polys,
        );
        0
    } else if op == MpCgameImport::CG_R_ADDDECALTOSCENE as c_int {
        //TODO: Port RE_AddDecalToScene world root
        // Source: oracle/codemp/client/cl_cgame.cpp:1027 (`re.AddDecalToScene`)
        // Same open owner as `CG_CM_MARKFRAGMENTS` above: `RE_AddDecalToScene`
        // takes a `world_root: &mut MarkNode`, and no carrier owns a root until
        // the renderer census merges that node arena (gh#31). The arm adds no
        // decal until then.
        0
    } else if op == MpCgameImport::CG_R_LIGHTFORPOINT as c_int {
        let point = unsafe { *(vma(vc, args, 1) as *const vec3_t) };
        let ambient_out = vma(vc, args, 2) as *mut vec3_t;
        let directed_out = vma(vc, args, 3) as *mut vec3_t;
        let light_dir_out = vma(vc, args, 4) as *mut vec3_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // W2-F1 put the cvar reads on the frame snapshot, so this synchronous
        // trap resolves one before the call.
        let cvar_snapshot = RenderCvarSnapshot::from_cvars(&re.cvars, view.common);
        match R_LightForPoint(
            cvar_snapshot,
            &re.sim.published,
            &re.world_load,
            &re.frame,
            point,
        ) {
            Some((ambient, directed, light_dir)) => {
                // SAFETY: the three are the module's seam out-params (§D11).
                unsafe {
                    *ambient_out = ambient;
                    *directed_out = directed;
                    *light_dir_out = light_dir;
                }
                qtrue
            }
            None => qfalse,
        }
    } else if op == MpCgameImport::CG_R_ADDLIGHTTOSCENE as c_int {
        let org = unsafe { *(vma(vc, args, 1) as *const vec3_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddLightToScene(
            &mut re.frame_data,
            &re.sim.published,
            org,
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
        );
        0
    } else if op == MpCgameImport::CG_R_ADDADDITIVELIGHTTOSCENE as c_int {
        let org = unsafe { *(vma(vc, args, 1) as *const vec3_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_AddAdditiveLightToScene(
            &mut re.frame_data,
            &re.sim.published,
            org,
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
        );
        0
    } else if op == MpCgameImport::CG_R_RENDERSCENE as c_int {
        let fd = vma(vc, args, 1) as *const refdef_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `refdef_t` (porting-rules §D11).
        RE_RenderScene(
            unsafe { &*fd },
            &mut re.frame_data,
            &re.sim.published,
            &re.cvars,
            &mut re.scene,
            view.common,
            &re.sim.light_styles,
        );
        0
    } else if op == MpCgameImport::CG_R_SETCOLOR as c_int {
        let rgba = rgba_arg(vma(vc, args, 1) as *const f32);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_SetColor(&mut re.frame_data, rgba);
        0
    } else if op == MpCgameImport::CG_R_DRAWSTRETCHPIC as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_StretchPic(
            &mut re.frame_data,
            &re.sim.published,
            view.common,
            vmf(vc, args, 1),
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
            vmf(vc, args, 6),
            vmf(vc, args, 7),
            vmf(vc, args, 8),
            arg(9),
        );
        0
    } else if op == MpCgameImport::CG_R_MODELBOUNDS as c_int {
        let mins_out = vma(vc, args, 2) as *mut vec3_t;
        let maxs_out = vma(vc, args, 3) as *mut vec3_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: the two are the module's seam out-params (porting-rules §D11).
        unsafe {
            let (mins, maxs) = r_model_bounds(rm, &re.sim.published, arg(1));
            *mins_out = mins;
            *maxs_out = maxs;
        }
        0
    } else if op == MpCgameImport::CG_R_LERPTAG as c_int {
        let name = cstr_to_string(vma(vc, args, 6) as *const c_char);
        let tag = vma(vc, args, 1) as *mut orientation_t;
        // SAFETY: `VMA(1)` is the module's seam out-param (porting-rules §D11).
        unsafe {
            r_lerp_tag(
                rm,
                &mut *tag,
                arg(2),
                arg(3),
                arg(4),
                vmf(vc, args, 5),
                &name,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_R_DRAWROTATEPIC as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RotatePic(
            &mut re.frame_data,
            &re.sim.published,
            view.common,
            vmf(vc, args, 1),
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
            vmf(vc, args, 6),
            vmf(vc, args, 7),
            vmf(vc, args, 8),
            vmf(vc, args, 9),
            arg(10),
        );
        0
    } else if op == MpCgameImport::CG_R_DRAWROTATEPIC2 as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_RotatePic2(
            &mut re.frame_data,
            &re.sim.published,
            view.common,
            vmf(vc, args, 1),
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
            vmf(vc, args, 6),
            vmf(vc, args, 7),
            vmf(vc, args, 8),
            vmf(vc, args, 9),
            arg(10),
        );
        0
    } else if op == MpCgameImport::CG_R_SETRANGEFOG as c_int {
        // Raven writes `tr.rangedFog` directly. The Rust renderer takes that
        // table-bypass write as a frame event (`FrameEvent::SetRangeFog`).
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        re.frame_data
            .events
            .push(FrameEvent::SetRangeFog(vmf(vc, args, 1)));
        0
    } else if op == MpCgameImport::CG_R_SETREFRACTIONPROP as c_int {
        // Raven writes the four `tr_distortion*` globals directly; the Rust
        // renderer takes them as one frame event.
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        re.frame_data.events.push(FrameEvent::SetRefractionProp {
            alpha: vmf(vc, args, 1),
            stretch: vmf(vc, args, 2),
            pre_post: arg(3) != 0,
            negate: arg(4) != 0,
        });
        0
    } else if op == MpCgameImport::CG_GETGLCONFIG as c_int {
        CL_GetGlconfig(cl, vma(vc, args, 1) as *mut glconfig_t);
        0
    } else if op == MpCgameImport::CG_GETGAMESTATE as c_int {
        CL_GetGameState(cl, vma(vc, args, 1) as *mut gameState_t);
        0
    } else if op == MpCgameImport::CG_GETCURRENTSNAPSHOTNUMBER as c_int {
        CL_GetCurrentSnapshotNumber(
            cl,
            vma(vc, args, 1) as *mut c_int,
            vma(vc, args, 2) as *mut c_int,
        );
        0
    } else if op == MpCgameImport::CG_GETSNAPSHOT as c_int {
        CL_GetSnapshot(view.common, cl, arg(1), vma(vc, args, 2) as *mut snapshot_t) as c_int
    } else if op == MpCgameImport::CG_GETDEFAULTSTATE as c_int {
        CL_GetDefaultState(cl, arg(1), vma(vc, args, 2) as *mut entityState_t) as c_int
    } else if op == MpCgameImport::CG_GETSERVERCOMMAND as c_int {
        CL_GetServerCommand(view, cl, arg(1)) as c_int
    } else if op == MpCgameImport::CG_GETCURRENTCMDNUMBER as c_int {
        CL_GetCurrentCmdNumber(cl)
    } else if op == MpCgameImport::CG_GETUSERCMD as c_int {
        CL_GetUserCmd(cl, arg(1), vma(vc, args, 2) as *mut usercmd_t) as c_int
    } else if op == MpCgameImport::CG_SETUSERCMDVALUE as c_int {
        cl.cl_bUseFighterPitch = arg(8) as qboolean;
        CL_SetUserCmdValue(
            cl,
            arg(1),
            vmf(vc, args, 2),
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
            arg(6),
            arg(7),
        );
        0
    } else if op == MpCgameImport::CG_SETCLIENTFORCEANGLE as c_int {
        CL_SetClientForceAngle(cl, arg(1), unsafe { *(vma(vc, args, 2) as *const vec3_t) });
        0
    } else if op == MpCgameImport::CG_SETCLIENTTURNEXTENT as c_int {
        0
    } else if op == MpCgameImport::CG_OPENUIMENU as c_int {
        VM_Call(
            view.common,
            cl.uivm,
            MpUiExport::UI_SET_ACTIVE_MENU as c_int,
            &[arg(1) as isize],
        );
        0
    } else if op == MpCgameImport::CG_MEMORY_REMAINING as c_int {
        Hunk_MemoryRemaining(view.common)
    } else if op == MpCgameImport::CG_KEY_ISDOWN as c_int {
        Key_IsDown(cl, arg(1))
    } else if op == MpCgameImport::CG_KEY_GETCATCHER as c_int {
        Key_GetCatcher(cl)
    } else if op == MpCgameImport::CG_KEY_SETCATCHER as c_int {
        Key_SetCatcher(cl, arg(1));
        0
    } else if op == MpCgameImport::CG_KEY_GETKEY as c_int {
        Key_GetKey(cl, vma(vc, args, 1) as *const c_char)
    } else if op == MpCgameImport::CG_PC_ADD_GLOBAL_DEFINE as c_int {
        // Raven keeps one process-wide `botlib_export`, and the port gives it
        // one home on `Server` (DEC-32), so the client reads it through the
        // view's `sv` slot.
        // Source: oracle/codemp/client/cl_cgame.cpp:61 (`botlib_export`)
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let sv = unsafe { sv_from_view(view) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_AddGlobalDefine.unwrap())(vma(vc, args, 1) as *mut c_char)
        }
    } else if op == MpCgameImport::CG_PC_LOAD_SOURCE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_LoadSourceHandle.unwrap())(
                bot,
                vma(vc, args, 1) as *const c_char,
            )
        }
    } else if op == MpCgameImport::CG_PC_FREE_SOURCE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: `sv.botlib_export` is the table `SV_BotInitBotLib` installed.
        unsafe { ((*sv.botlib_export).PC_FreeSourceHandle.unwrap())(bot, arg(1)) }
    } else if op == MpCgameImport::CG_PC_READ_TOKEN as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointer is the module's out-param (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_ReadTokenHandle.unwrap())(
                bot,
                arg(1),
                vma(vc, args, 2) as *mut pc_token_t,
            )
        }
    } else if op == MpCgameImport::CG_PC_SOURCE_FILE_AND_LINE as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointers are the module's out-params (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_SourceFileAndLine.unwrap())(
                bot,
                arg(1),
                vma(vc, args, 2) as *mut c_char,
                vma(vc, args, 3) as *mut c_int,
            )
        }
    } else if op == MpCgameImport::CG_PC_LOAD_GLOBAL_DEFINES as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: the seam pointer is the module's string (porting-rules §D11).
        unsafe {
            ((*sv.botlib_export).PC_LoadGlobalDefines.unwrap())(
                bot,
                vma(vc, args, 1) as *const c_char,
            )
        }
    } else if op == MpCgameImport::CG_PC_REMOVE_ALL_GLOBAL_DEFINES as c_int {
        // SAFETY: view-constructor slots, single-threaded, no other live cast.
        let (sv, bot) = unsafe { (sv_from_view(view), bot_from_view(view)) };
        // SAFETY: `sv.botlib_export` is the table `SV_BotInitBotLib` installed.
        unsafe { ((*sv.botlib_export).PC_RemoveAllGlobalDefines.unwrap())(bot) };
        0
    } else if op == MpCgameImport::CG_S_STOPBACKGROUNDTRACK as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let snd = unsafe { snd_from_view(view) };
        S_StopBackgroundTrack(view.common, snd);
        0
    } else if op == MpCgameImport::CG_REAL_TIME as c_int {
        Com_RealTime(vma(vc, args, 1) as *mut qtime_t)
    } else if op == MpCgameImport::CG_SNAPVECTOR as c_int {
        Sys_SnapVector(vma(vc, args, 1) as *mut f32);
        0
    } else if op == MpCgameImport::CG_CIN_PLAYCINEMATIC as c_int {
        let name = vma(vc, args, 1) as *const c_char;
        CIN_PlayCinematic(view, cl, name, arg(2), arg(3), arg(4), arg(5), arg(6))
    } else if op == MpCgameImport::CG_CIN_STOPCINEMATIC as c_int {
        CIN_StopCinematic(view, cl, arg(1)) as c_int
    } else if op == MpCgameImport::CG_CIN_RUNCINEMATIC as c_int {
        CIN_RunCinematic(view, cl, arg(1)) as c_int
    } else if op == MpCgameImport::CG_CIN_DRAWCINEMATIC as c_int {
        CIN_DrawCinematic(view, cl, arg(1));
        0
    } else if op == MpCgameImport::CG_CIN_SETEXTENTS as c_int {
        CIN_SetExtents(cl, arg(1), arg(2), arg(3), arg(4), arg(5));
        0
    } else if op == MpCgameImport::CG_R_REMAP_SHADER as c_int {
        let a = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let b = cstr_to_string(vma(vc, args, 2) as *const c_char);
        let c = cstr_to_string(vma(vc, args, 3) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        R_RemapShader(
            &a,
            &b,
            Some(&c),
            &mut re.qs,
            &mut re.world_load,
            Arc::make_mut(&mut re.sim.published),
            view,
            &re.cvars,
            rm,
            &mut re.img_state,
            &mut re.sky_view,
        );
        0
    } else if op == MpCgameImport::CG_R_GET_LIGHT_STYLE as c_int {
        let color_out = vma(vc, args, 2) as *mut u8;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let color = RE_GetLightStyle(&re.sim, arg(1) as usize);
        // SAFETY: `VMA(2)` is the module's seam out-param (porting-rules §D11).
        unsafe { core::ptr::copy_nonoverlapping(color.as_ptr(), color_out, 4) };
        0
    } else if op == MpCgameImport::CG_R_SET_LIGHT_STYLE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        RE_SetLightStyle(&mut re.sim, arg(1) as usize, arg(2).to_le_bytes());
        0
    } else if op == MpCgameImport::CG_R_GET_BMODEL_VERTS as c_int {
        let verts_out = vma(vc, args, 2) as *mut vec3_t;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let verts = RE_GetBModelVerts(arg(1), rm, &re.sim.published, &re.frame);
        // SAFETY: `VMA(2)` is the module's four-vertex out-buffer (§D11).
        unsafe { core::ptr::copy_nonoverlapping(verts.as_ptr(), verts_out, 4) };
        0
    } else if op == MpCgameImport::CG_R_GETDISTANCECULL as c_int {
        let out = vma(vc, args, 1) as *mut f32;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: `VMA(1)` is the module's seam out-param (porting-rules §D11).
        unsafe { *out = re.sim.published.distance_cull };
        0
    } else if op == MpCgameImport::CG_R_GETREALRES as c_int {
        let w_out = vma(vc, args, 1) as *mut c_int;
        let h_out = vma(vc, args, 2) as *mut c_int;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // SAFETY: both are the module's seam out-params (porting-rules §D11).
        unsafe {
            *w_out = re.sim.published.glconfig.vid_width;
            *h_out = re.sim.published.glconfig.vid_height;
        }
        0
    } else if op == MpCgameImport::CG_R_AUTOMAPELEVADJ as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        R_AutomapElevationAdjustment(&mut re.frame_data, vmf(vc, args, 1));
        0
    } else if op == MpCgameImport::CG_R_INITWIREFRAMEAUTO as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        let disable = view.common.cvar(re.cvars.r_autoMapDisable).integer;
        R_InitializeWireframeAutomap(&mut re.automap, &re.sim.published, disable) as c_int
    } else if op == MpCgameImport::CG_GET_ENTITY_TOKEN as c_int {
        let buffer = vma(vc, args, 1) as *mut c_char;
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        // `R_GetEntityToken` advances the world's own parse cursor, so this
        // takes the world mutably. W2-F7 put it behind its own `Arc`, and
        // `make_mut` copies only while the render thread still holds the
        // generation this trap is walking.
        let Some(world) = Arc::make_mut(&mut re.sim.published)
            .world
            .as_mut()
            .map(Arc::make_mut)
        else {
            return qfalse;
        };
        let (found, token) = R_GetEntityToken(world, arg(2));
        // The force-reset call is `(NULL, -1)`, and Raven returns from the
        // size check before any buffer write (`tr_bsp.cpp:1981-1985`).
        if !buffer.is_null() {
            let token_c = std::ffi::CString::new(token).unwrap_or_default();
            // SAFETY: `VMA(1)` is the module's seam out-buffer (porting-rules §D11).
            unsafe { libc::strcpy(buffer, token_c.as_ptr()) };
        }
        found as c_int
    } else if op == MpCgameImport::CG_R_INPVS as c_int {
        let p1 = unsafe { *(vma(vc, args, 1) as *const vec3_t) };
        let p2 = unsafe { *(vma(vc, args, 2) as *const vec3_t) };
        R_inPVS(view.cm, p1, p2) as c_int
    } else if op == MpCgameImport::CG_FX_ADDLINE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)`, `VMA(2)`, `VMA(9)`, and `VMA(10)` are the module's
        // three-float vectors (porting-rules §D11).
        let (start, end, s_rgb, e_rgb) = unsafe {
            (
                *(vma(vc, args, 1) as *const vec3_t),
                *(vma(vc, args, 2) as *const vec3_t),
                *(vma(vc, args, 9) as *const vec3_t),
                *(vma(vc, args, 10) as *const vec3_t),
            )
        };
        let (size1, size2, size_parm, alpha1, alpha2, alpha_parm, rgb_parm) = (
            vmf(vc, args, 3),
            vmf(vc, args, 4),
            vmf(vc, args, 5),
            vmf(vc, args, 6),
            vmf(vc, args, 7),
            vmf(vc, args, 8),
            vmf(vc, args, 11),
        );
        let mut host = FxHost::Engine { view, cl };
        FX_AddLine(
            fx,
            &mut host,
            start,
            end,
            size1,
            size2,
            size_parm,
            alpha1,
            alpha2,
            alpha_parm,
            s_rgb,
            e_rgb,
            rgb_parm,
            arg(12),
            arg(13),
            arg(14),
            EMatImpactEffect::MATIMPACTFX_NONE,
            -1,
            0,
            -1,
            -1,
            -1,
        );
        0
    } else if op == MpCgameImport::CG_FX_REGISTER_EFFECT as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let mut host = FxHost::Engine { view, cl };
        FX_RegisterEffect(fx, &mut host, &name)
    } else if op == MpCgameImport::CG_FX_PLAY_EFFECT as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(2)` and `VMA(3)` are the module's three-float vectors.
        let (org, fwd) = unsafe {
            (
                *(vma(vc, args, 2) as *const vec3_t),
                *(vma(vc, args, 3) as *const vec3_t),
            )
        };
        let (vol, rad) = (arg(4), arg(5));
        let mut host = FxHost::Engine { view, cl };
        FX_PlayEffect(fx, &mut host, &name, org, fwd, vol, rad);
        0
    } else if op == MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT as c_int {
        // Raven: assert(0);//gone! — the entity-effect entry point was removed upstream.
        unreachable!("CG_FX_PLAY_ENTITY_EFFECT — gone in the oracle (cl_cgame.cpp:1112-1115)")
    } else if op == MpCgameImport::CG_FX_PLAY_EFFECT_ID as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(2)` and `VMA(3)` are the module's three-float vectors.
        let (org, fwd) = unsafe {
            (
                *(vma(vc, args, 2) as *const vec3_t),
                *(vma(vc, args, 3) as *const vec3_t),
            )
        };
        let (id, vol, rad) = (arg(1), arg(4), arg(5));
        let mut host = FxHost::Engine { view, cl };
        FX_PlayEffectID(fx, &mut host, id, org, fwd, vol, rad, false);
        0
    } else if op == MpCgameImport::CG_FX_PLAY_PORTAL_EFFECT_ID as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(2)` and `VMA(3)` are the module's three-float vectors.
        let (org, fwd) = unsafe {
            (
                *(vma(vc, args, 2) as *const vec3_t),
                *(vma(vc, args, 3) as *const vec3_t),
            )
        };
        let (id, vol, rad) = (arg(1), arg(4), arg(5));
        let mut host = FxHost::Engine { view, cl };
        FX_PlayEffectID(fx, &mut host, id, org, fwd, vol, rad, true);
        0
    } else if op == MpCgameImport::CG_FX_PLAY_ENTITY_EFFECT_ID as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(2)` is the module's origin and `VMA(3)` its three-vector axis.
        let (org, axis) = unsafe {
            (
                *(vma(vc, args, 2) as *const vec3_t),
                *(vma(vc, args, 3) as *const [vec3_t; 3]),
            )
        };
        let (id, bolt_info, ent_num, vol, rad) = (arg(1), arg(4), arg(5), arg(6), arg(7));
        let mut host = FxHost::Engine { view, cl };
        FX_PlayEntityEffectID(fx, &mut host, id, org, axis, bolt_info, ent_num, vol, rad);
        0
    } else if op == MpCgameImport::CG_FX_PLAY_BOLTED_EFFECT_ID as c_int {
        // SAFETY: `args[3]` is the module's ghoul2 token (§D11).
        unsafe {
            let g2v = CGhoul2Info_v::from_token(argw(3) as *mut c_void);
            let ghl_info = g2_info(g2, &g2v, arg(6));
            match g2api_attach_ent(g2, view, ghl_info, arg(4), arg(5), arg(6)) {
                Some(boltInfo) => {
                    let org = *(vma(vc, args, 2) as *const vec3_t);
                    let item = g2v.mItem;
                    let (id, loop_time, relative) = (arg(1), arg(7), arg(8) != 0);
                    // SAFETY: view-constructor slot, no other live cast.
                    let fx = fx_from_view(view);
                    let mut host = FxHost::Engine { view, cl };
                    FX_PlayBoltedEffectID(
                        fx, &mut host, id, org, boltInfo, item, loop_time, relative,
                    );
                    1
                }
                None => 0,
            }
        }
    } else if op == MpCgameImport::CG_FX_ADD_SCHEDULED_EFFECTS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let portal = arg(1) != 0;
        let mut host = FxHost::Engine { view, cl };
        FX_AddScheduledEffects(fx, &mut host, portal);
        0
    } else if op == MpCgameImport::CG_FX_DRAW_2D_EFFECTS as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let (x_scale, y_scale) = (vmf(vc, args, 1), vmf(vc, args, 2));
        let mut host = FxHost::Engine { view, cl };
        FX_Draw2DEffects(fx, &mut host, x_scale, y_scale);
        0
    } else if op == MpCgameImport::CG_FX_INIT_SYSTEM as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let refdef = vma(vc, args, 1) as *mut refdef_t;
        let mut host = FxHost::Engine { view, cl };
        FX_InitSystem(fx, &mut host, refdef)
    } else if op == MpCgameImport::CG_FX_SET_REFDEF as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        FX_SetRefDefFromCGame(fx, vma(vc, args, 1) as *mut refdef_t);
        0
    } else if op == MpCgameImport::CG_FX_FREE_SYSTEM as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let mut host = FxHost::Engine { view, cl };
        FX_FreeSystem(fx, &mut host)
    } else if op == MpCgameImport::CG_FX_ADJUST_TIME as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        FX_AdjustTime(fx, arg(1));
        0
    } else if op == MpCgameImport::CG_FX_RESET as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        let mut host = FxHost::Engine { view, cl };
        FX_Free(fx, &mut host, false);
        0
    } else if op == MpCgameImport::CG_FX_ADDPOLY as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `addpolyArgStruct_t` (§D11).
        let p = unsafe { (vma(vc, args, 1) as *mut addpolyArgStruct_t).as_ref() };
        if let Some(p) = p {
            let a = *p;
            let mut host = FxHost::Engine { view, cl };
            FX_AddPoly(
                fx,
                &mut host,
                &a.p,
                &a.ev,
                a.numVerts,
                a.vel,
                a.accel,
                a.alpha1,
                a.alpha2,
                a.alphaParm,
                a.rgb1,
                a.rgb2,
                a.rgbParm,
                a.rotationDelta,
                a.bounce,
                a.motionDelay,
                a.killTime,
                a.shader,
                a.flags,
            );
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDBEZIER as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `addbezierArgStruct_t` (§D11).
        let b = unsafe { (vma(vc, args, 1) as *mut addbezierArgStruct_t).as_ref() };
        if let Some(b) = b {
            let a = *b;
            let mut host = FxHost::Engine { view, cl };
            FX_AddBezier(
                fx,
                &mut host,
                a.start,
                a.end,
                a.control1,
                a.control1Vel,
                a.control2,
                a.control2Vel,
                a.size1,
                a.size2,
                a.sizeParm,
                a.alpha1,
                a.alpha2,
                a.alphaParm,
                a.sRGB,
                a.eRGB,
                a.rgbParm,
                a.killTime,
                a.shader,
                a.flags,
            );
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDPRIMITIVE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `effectTrailArgStruct_t` (§D11).
        let a = unsafe { (vma(vc, args, 1) as *mut effectTrailArgStruct_t).as_ref() };
        if let Some(a) = a {
            let trail = *a;
            let mut host = FxHost::Engine { view, cl };
            FX_FeedTrail(fx, &mut host, &trail);
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDSPRITE as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `addspriteArgStruct_t` (§D11).
        let s = unsafe { (vma(vc, args, 1) as *mut addspriteArgStruct_t).as_ref() };
        if let Some(s) = s {
            let a = *s;
            let rgb: vec3_t = [1.0, 1.0, 1.0];
            let mut host = FxHost::Engine { view, cl };
            // Raven's commented-out `FX_AddSprite` call sits above this one, and
            // the shipped arm builds a particle instead.
            FX_AddParticle(
                fx,
                &mut host,
                a.origin,
                a.vel,
                a.accel,
                a.scale,
                a.dscale,
                0.0,
                a.sAlpha,
                a.eAlpha,
                0.0,
                rgb,
                rgb,
                0.0,
                a.rotation,
                0.0,
                vec3_origin,
                vec3_origin,
                a.bounce,
                0,
                0,
                a.life,
                a.shader,
                a.flags,
                EMatImpactEffect::MATIMPACTFX_NONE,
                -1,
                0,
                -1,
                -1,
                -1,
            );
        }
        0
    } else if op == MpCgameImport::CG_FX_ADDELECTRICITY as c_int {
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let fx = unsafe { fx_from_view(view) };
        // SAFETY: `VMA(1)` is the module's `addElectricityArgStruct_t` (§D11).
        let p = unsafe { (vma(vc, args, 1) as *mut addElectricityArgStruct_t).as_ref() };
        if let Some(p) = p {
            let a = *p;
            let mut host = FxHost::Engine { view, cl };
            FX_AddElectricity(
                fx,
                &mut host,
                a.start,
                a.end,
                a.size1,
                a.size2,
                a.sizeParm,
                a.alpha1,
                a.alpha2,
                a.alphaParm,
                a.sRGB,
                a.eRGB,
                a.rgbParm,
                a.chaos,
                a.killTime,
                a.shader,
                a.flags,
                EMatImpactEffect::MATIMPACTFX_NONE,
                -1,
                0,
                -1,
                -1,
                -1,
            );
        }
        0
    } else if op == MpCgameImport::CG_ROFF_CLEAN as c_int {
        roff.clean(true) as c_int
    } else if op == MpCgameImport::CG_ROFF_UPDATE_ENTITIES as c_int {
        roff.update_entities(true, view);
        0
    } else if op == MpCgameImport::CG_ROFF_CACHE as c_int {
        let file = cstr_to_string(vma(vc, args, 1) as *const c_char);
        roff.cache(&file, true, view)
    } else if op == MpCgameImport::CG_ROFF_PLAY as c_int {
        roff.play(arg(1), arg(2), arg(3) != 0, true, view) as c_int
    } else if op == MpCgameImport::CG_ROFF_PURGE_ENT as c_int {
        roff.purge_ent(arg(1), true, view) as c_int
    } else if op == MpCgameImport::CG_TRUEMALLOC as c_int {
        VM_Shifted_Alloc(view, vma(vc, args, 1) as *mut *mut (), arg(2));
        0
    } else if op == MpCgameImport::CG_TRUEFREE as c_int {
        VM_Shifted_Free(view.common, vma(vc, args, 1) as *mut *mut ());
        0
    } else if op == MpCgameImport::CG_G2_LISTSURFACES as c_int {
        // SAFETY: `args[1]` is the module's `CGhoul2Info` handle (§D11).
        let ghl_info = unsafe { &mut *(argw(1) as *mut CGhoul2Info) };
        g2api_list_surfaces(g2, view, ghl_info);
        0
    } else if op == MpCgameImport::CG_G2_LISTBONES as c_int {
        // SAFETY: `args[1]` is the module's `CGhoul2Info` handle (§D11).
        let ghl_info = unsafe { &mut *(argw(1) as *mut CGhoul2Info) };
        g2api_list_bones(g2, view, ghl_info, arg(2));
        0
    } else if op == MpCgameImport::CG_G2_HAVEWEGHOULMODELS as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is qfalse.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1923`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            0
        } else {
            let ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_have_we_ghoul2_models(g2, &ghoul2) as c_int
        }
    } else if op == MpCgameImport::CG_G2_SETMODELS as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the call does nothing.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1942`
        let token = argw(1) as *mut c_void;
        if !token.is_null() {
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            // SAFETY: the two lists are module-space (§D11).
            unsafe {
                g2api_set_ghoul2_model_indexes(
                    g2,
                    &mut ghoul2,
                    core::slice::from_raw_parts(vma(vc, args, 2) as *const qhandle_t, 0),
                    core::slice::from_raw_parts(vma(vc, args, 3) as *const qhandle_t, 0),
                );
            }
        }
        0
    } else if op == MpCgameImport::CG_G2_GETBOLT as c_int {
        get_bolt_matrix_arm(view, g2, vc, args, &arg) as c_int
    } else if op == MpCgameImport::CG_G2_GETBOLT_NOREC as c_int {
        g2.gbm_no_reconstruct = true;
        get_bolt_matrix_arm(view, g2, vc, args, &arg) as c_int
    } else if op == MpCgameImport::CG_G2_GETBOLT_NOREC_NOROT as c_int {
        // gG2_GBMNoReconstruct = qtrue; // Yeah, this was probably BAD.
        g2.gbm_use_sp_method = true;
        get_bolt_matrix_arm(view, g2, vc, args, &arg) as c_int
    } else if op == MpCgameImport::CG_G2_INITGHOUL2MODEL as c_int {
        let file_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: `VMA(1)` is the module's ghoul2 token slot (§D11).
        // Raven `if (!(*ghoul2Ptr)) *ghoul2Ptr = new CGhoul2Info_v;` builds the handle object the module then holds.
        // The token scheme (DEC-65 ruling 3) needs no allocation.
        // The slot reads into a stack cell, and the write-back below carries the new handle out.
        let pp = vma(vc, args, 1) as *mut *mut c_void;
        // SAFETY: the slot lives in module memory for the whole dispatch (§D11).
        let mut ghoul2 = unsafe { CGhoul2Info_v::from_token(*pp) };
        let answer = g2api_init_ghoul2_model(
            g2,
            view,
            &mut ghoul2,
            &file_name,
            arg(3),
            arg(4) as qhandle_t,
            arg(5) as qhandle_t,
            arg(6),
            arg(7),
        );
        unsafe { *pp = ghoul2.to_token() };
        answer
    } else if op == MpCgameImport::CG_G2_SETSKIN as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(2));
        g2api_set_skin(g2, view, ghl_info, arg(3), arg(4)) as c_int
    } else if op == MpCgameImport::CG_G2_COLLISIONDETECT as c_int {
        let out = vma(vc, args, 1) as *mut CollisionRecord_t;
        // `args[2]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(2) as *mut c_void);
        let hits = g2api_collision_detect(
            g2,
            view,
            &mut ghoul2,
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            unsafe { *(vma(vc, args, 4) as *const vec3_t) },
            arg(5),
            arg(6),
            unsafe { *(vma(vc, args, 7) as *const vec3_t) },
            unsafe { *(vma(vc, args, 8) as *const vec3_t) },
            unsafe { *(vma(vc, args, 9) as *const vec3_t) },
            arg(10),
            arg(11),
            vmf(vc, args, 12),
        );
        write_collision_records(out, &hits);
        0
    } else if op == MpCgameImport::CG_G2_COLLISIONDETECTCACHE as c_int {
        let out = vma(vc, args, 1) as *mut CollisionRecord_t;
        // `args[2]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(2) as *mut c_void);
        let hits = g2api_collision_detect_cache(
            g2,
            view,
            &mut ghoul2,
            unsafe { *(vma(vc, args, 3) as *const vec3_t) },
            unsafe { *(vma(vc, args, 4) as *const vec3_t) },
            arg(5),
            arg(6),
            unsafe { *(vma(vc, args, 7) as *const vec3_t) },
            unsafe { *(vma(vc, args, 8) as *const vec3_t) },
            unsafe { *(vma(vc, args, 9) as *const vec3_t) },
            arg(10),
            arg(11),
            vmf(vc, args, 12),
        );
        write_collision_records(out, &hits);
        0
    } else if op == MpCgameImport::CG_G2_ANGLEOVERRIDE as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is qfalse.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1302`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            0
        } else {
            let bone_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            // SAFETY: the angles and the model list are module-space.
            unsafe {
                g2api_set_bone_angles(
                    g2,
                    view,
                    &mut ghoul2,
                    arg(2),
                    &bone_name,
                    *(vma(vc, args, 4) as *const vec3_t),
                    arg(5),
                    core::mem::transmute::<c_int, Eorientations>(arg(6)),
                    core::mem::transmute::<c_int, Eorientations>(arg(7)),
                    core::mem::transmute::<c_int, Eorientations>(arg(8)),
                    core::slice::from_raw_parts(vma(vc, args, 9) as *const qhandle_t, 0),
                    arg(10),
                    arg(11),
                ) as c_int
            }
        }
    } else if op == MpCgameImport::CG_G2_CLEANMODELS as c_int {
        // SAFETY: `VMA(1)` is the module's ghoul2 token slot (§D11).
        // Raven guards the null pointee and then deletes and nulls the handle (`G2_API.cpp:496-564`).
        // `g2api_clean_ghoul2_models` zeroes `mItem`, and a zeroed cell encodes back to the null token.
        // The write-back is therefore Raven's `*ghoul2Ptr = NULL`.
        unsafe {
            let pp = vma(vc, args, 1) as *mut *mut c_void;
            if !(*pp).is_null() {
                let mut ghoul2 = CGhoul2Info_v::from_token(*pp);
                g2api_clean_ghoul2_models(g2, &mut ghoul2);
                *pp = ghoul2.to_token();
            }
        }
        0
    } else if op == MpCgameImport::CG_G2_PLAYANIM as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is qfalse.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1103`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            0
        } else {
            let bone_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_set_bone_anim(
                g2,
                &mut ghoul2,
                arg(2),
                &bone_name,
                arg(4),
                arg(5),
                arg(6),
                vmf(vc, args, 7),
                arg(8),
                vmf(vc, args, 9),
                arg(10),
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_GETBONEANIM as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: the handle, the model list, and the five out-params are
        // module-space (porting-rules §D11).
        unsafe {
            let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let ghl_info = g2_info(g2, &g2v, arg(10));
            match g2api_get_bone_anim(
                g2,
                view,
                ghl_info,
                &bone_name,
                arg(3),
                core::slice::from_raw_parts(vma(vc, args, 9) as *const qhandle_t, 0),
            ) {
                Some((current_frame, start_frame, end_frame, flags, anim_speed)) => {
                    *(vma(vc, args, 4) as *mut f32) = current_frame;
                    *(vma(vc, args, 5) as *mut c_int) = start_frame;
                    *(vma(vc, args, 6) as *mut c_int) = end_frame;
                    *(vma(vc, args, 7) as *mut c_int) = flags;
                    *(vma(vc, args, 8) as *mut f32) = anim_speed;
                    qtrue
                }
                None => qfalse,
            }
        }
    } else if op == MpCgameImport::CG_G2_GETBONEFRAME as c_int {
        // rwwFIXMEFIXME: Just make a G2API_GetBoneFrame func too. This is dirty.
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: the handle, the model list, and the out-param are module-space.
        unsafe {
            let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let ghl_info = g2_info(g2, &g2v, arg(6));
            match g2api_get_bone_anim(
                g2,
                view,
                ghl_info,
                &bone_name,
                arg(3),
                core::slice::from_raw_parts(vma(vc, args, 5) as *const qhandle_t, 0),
            ) {
                // Raven discards startFrame/endFrame/flags/animSpeed here.
                Some((current_frame, ..)) => {
                    *(vma(vc, args, 4) as *mut f32) = current_frame;
                    qtrue
                }
                None => qfalse,
            }
        }
    } else if op == MpCgameImport::CG_G2_GETGLANAME as c_int {
        // Raven reads the handle before its `if ((int)&ghoul2)` check, so a null handle is UB.
        // §19: we take the NULL-name path, which leaves the out-buffer alone.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:2414-2416`
        let token = argw(1) as *mut c_void;
        if !token.is_null() {
            let point = vma(vc, args, 3) as *mut c_char;
            let ghoul2 = CGhoul2Info_v::from_token(token);
            if let Some(local) = g2api_get_gla_name(g2, view, &ghoul2, arg(2)) {
                let local_c = std::ffi::CString::new(local).unwrap_or_default();
                // SAFETY: `VMA(3)` is the module's seam out-buffer (§D11).
                unsafe { libc::strcpy(point, local_c.as_ptr()) };
            }
        }
        0
    } else if op == MpCgameImport::CG_G2_COPYGHOUL2INSTANCE as c_int {
        // Both sides come across by value, so this arm has no slot to write back through.
        // In-place `deep_copy` (DEC-65 ruling 3) keeps a live destination's handle, so the discarded cell is correct.
        // §19: a null destination is Raven's own crash on a null reference (`G2_API.cpp:2239-2259`).
        // A call would allocate into the discarded cell and strand the arena slot, so this answers -1 without calling.
        // The arm is dead in both trees.
        // No Rust module calls it, and no oracle game, cgame, or ui source calls `trap_G2API_CopyGhoul2Instance`.
        let to_token = argw(2) as *mut c_void;
        if to_token.is_null() {
            -1
        } else {
            let mut g2_from = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let mut g2_to = CGhoul2Info_v::from_token(to_token);
            g2api_copy_ghoul2_instance(g2, &mut g2_from, &mut g2_to, arg(3))
        }
    } else if op == MpCgameImport::CG_G2_COPYSPECIFICGHOUL2MODEL as c_int {
        // A null token makes Raven's `if (((int)&ghoul2From) && ((int)&ghoul2To))` false,
        // and the call does nothing.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:2291`
        // §19: after a remove empties a slot the slot now reads null, so this arm no-ops.
        // Raven no-ops there too, because Raven nulls the slot at remove (`G2_API.cpp:868-869`).
        // The pointer code that preceded this reallocated through a kept empty cell (`api_models.rs:695-697`), which was the outlier.
        let from_token = argw(1) as *mut c_void;
        let to_token = argw(3) as *mut c_void;
        if !from_token.is_null() && !to_token.is_null() {
            let mut ghoul2_from = CGhoul2Info_v::from_token(from_token);
            let mut ghoul2_to = CGhoul2Info_v::from_token(to_token);
            g2api_copy_specific_g2_model(g2, &mut ghoul2_from, arg(2), &mut ghoul2_to, arg(4));
        }
        0
    } else if op == MpCgameImport::CG_G2_DUPLICATEGHOUL2INSTANCE as c_int {
        // SAFETY: `VMA(2)` is the module's ghoul2 token slot (§D11).
        unsafe {
            let mut g2_from = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let pp = vma(vc, args, 2) as *mut *mut c_void;
            // Raven returns on a live destination (assert dropped, NDEBUG) and
            // allocates on a null one (`G2_API.cpp:2330-2340`).
            // The destination starts empty, so the copy allocates and the write-back carries the new handle to the module.
            if (*pp).is_null() {
                let mut g2_to = CGhoul2Info_v { mItem: 0 };
                g2api_duplicate_ghoul2_instance(g2, &mut g2_from, &mut g2_to);
                *pp = g2_to.to_token();
            }
        }
        0
    } else if op == MpCgameImport::CG_G2_HASGHOUL2MODELONINDEX as c_int {
        // SAFETY: `VMA(1)` is the module's ghoul2 token slot (§D11).
        // §19: Raven derefs the null pointee; the gone-instance sanity answer
        // is qfalse, so the null pointee takes it too.
        // The arm reads the slot but never writes it, because the call is a pure read.
        let pp = vma(vc, args, 1) as *mut *mut c_void;
        if unsafe { (*pp).is_null() } {
            0
        } else {
            let ghoul2 = unsafe { CGhoul2Info_v::from_token(*pp) };
            g2api_has_ghoul2_model_on_index(g2, &ghoul2, arg(2)) as c_int
        }
    } else if op == MpCgameImport::CG_G2_REMOVEGHOUL2MODEL as c_int {
        // SAFETY: `VMA(1)` is the module's ghoul2 token slot (§D11).
        // §19: same null-pointee answer as the HASGHOUL2MODELONINDEX arm.
        // The callee frees the handle when the vector empties (`api_models.rs:497`).
        // The write-back therefore restores Raven's `*ghlRemove = NULL` (`G2_API.cpp:868-869`).
        let pp = vma(vc, args, 1) as *mut *mut c_void;
        if unsafe { (*pp).is_null() } {
            0
        } else {
            let mut ghoul2 = unsafe { CGhoul2Info_v::from_token(*pp) };
            let answer = g2api_remove_ghoul2_model(g2, &mut ghoul2, arg(2)) as c_int;
            unsafe { *pp = ghoul2.to_token() };
            answer
        }
    } else if op == MpCgameImport::CG_G2_SKINLESSMODEL as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(2));
        g2api_skinless_model(g2, view, ghl_info) as c_int
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
        // `args[1]` is the module's ghoul2 token (§D11).
        let ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_ghoul2_size(g2, &ghoul2)
    } else if op == MpCgameImport::CG_G2_ADDBOLT as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is -1.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1637`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            -1
        } else {
            let bone_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_add_bolt(g2, view, &mut ghoul2, arg(2), &bone_name)
        }
    } else if op == MpCgameImport::CG_G2_ATTACHENT as c_int {
        // G2API_AttachEnt(int *boltInfo, CGhoul2Info *ghlInfoTo, int toBoltIndex, int entNum, int toModelNum)
        let bolt_info_out = vma(vc, args, 1) as *mut c_int;
        // `args[2]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(2) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, 0);
        match g2api_attach_ent(g2, view, ghl_info, arg(3), arg(4), arg(5)) {
            Some(bolt_info) => {
                // SAFETY: `VMA(1)` is the module's seam out-param (§D11).
                unsafe { *bolt_info_out = bolt_info };
                qtrue
            }
            None => qfalse,
        }
    } else if op == MpCgameImport::CG_G2_SETBOLTON as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the call does nothing.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:1686`
        let token = argw(1) as *mut c_void;
        if !token.is_null() {
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_set_bolt_info(g2, &mut ghoul2, arg(2), arg(3));
        }
        0
    } else if op == MpCgameImport::CG_G2_SETROOTSURFACE as c_int {
        let surface_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
        // `args[1]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_set_root_surface(g2, view, &mut ghoul2, arg(2), &surface_name) as c_int
    } else if op == MpCgameImport::CG_G2_SETSURFACEONOFF as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is qfalse.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:710`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            0
        } else {
            let surface_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_set_surface_on_off(g2, view, &mut ghoul2, &surface_name, arg(3)) as c_int
        }
    } else if op == MpCgameImport::CG_G2_SETNEWORIGIN as c_int {
        // A null token makes Raven's `if ((int)&ghoul2)` false, and the answer is qfalse.
        // Source: `oracle/codemp/ghoul2/G2_API.cpp:2432`
        let token = argw(1) as *mut c_void;
        if token.is_null() {
            0
        } else {
            let mut ghoul2 = CGhoul2Info_v::from_token(token);
            g2api_set_new_origin(g2, view, &mut ghoul2, arg(2)) as c_int
        }
    } else if op == MpCgameImport::CG_G2_DOESBONEEXIST as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(2));
        g2api_does_bone_exist(g2, view, ghl_info, &bone_name) as c_int
    } else if op == MpCgameImport::CG_G2_GETSURFACERENDERSTATUS as c_int {
        let surface_name = cstr_to_string(vma(vc, args, 3) as *const c_char);
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(2));
        g2api_get_surface_render_status(g2, view, ghl_info, &surface_name)
    } else if op == MpCgameImport::CG_G2_GETTIME as c_int {
        g2api_get_time(g2, 0)
    } else if op == MpCgameImport::CG_G2_SETTIME as c_int {
        g2api_set_time(g2, arg(1), arg(2));
        0
    } else if op == MpCgameImport::CG_G2_ABSURDSMOOTHING as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_absurd_smoothing(g2, &mut ghoul2, arg(2) != 0);
        0
    } else if op == MpCgameImport::CG_G2_SETRAGDOLL as c_int {
        // Converts the info in the shared structure over to the class-based version.
        // SAFETY: the handle and `VMA(2)` are module-space (porting-rules §D11).
        unsafe {
            let rdParamst = vma(vc, args, 2) as *mut sharedRagDollParams_t;
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            if rdParamst.is_null() {
                g2api_reset_ragdoll(g2, &mut ghoul2);
                return 0;
            }

            let mut rdParams = CRagDollParams {
                angles: (*rdParamst).angles,
                position: (*rdParamst).position,
                scale: (*rdParamst).scale,
                pelvisAnglesOffset: (*rdParamst).pelvis_angles_offset,
                pelvisPositionOffset: (*rdParamst).pelvis_position_offset,

                fImpactStrength: (*rdParamst).f_impact_strength,
                fShotStrength: (*rdParamst).f_shot_strength,
                me: (*rdParamst).me,

                startFrame: (*rdParamst).start_frame,
                endFrame: (*rdParamst).end_frame,

                collisionType: (*rdParamst).collision_type,
                CallRagDollBegin: (*rdParamst).call_rag_doll_begin,

                // Raven casts the two ints to its nested enums at this site.
                RagPhase: core::mem::transmute::<c_int, sharedERagPhase>((*rdParamst).rag_phase),
                effectorsToTurnOff: core::mem::transmute::<c_int, sharedERagEffector>(
                    (*rdParamst).effectors_to_turn_off,
                ),
            };

            g2api_set_ragdoll(g2, view, &mut ghoul2, &mut rdParams);
        }
        0
    } else if op == MpCgameImport::CG_G2_ANIMATEG2MODELS as c_int {
        // SAFETY: the handle and `VMA(3)` are module-space (porting-rules §D11).
        unsafe {
            let rduParamst = vma(vc, args, 3) as *mut sharedRagDollUpdateParams_t;
            if rduParamst.is_null() {
                return 0;
            }

            let mut rduParams = RagDollUpdateParams {
                angles: (*rduParamst).angles,
                position: (*rduParamst).position,
                scale: (*rduParamst).scale,
                velocity: (*rduParamst).velocity,

                me: (*rduParamst).me,
                settle_frame: (*rduParamst).settle_frame,

                kind: RagDollUpdateKind::Server,
            };

            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            g2api_animate_g2_models_rag(g2, view, &mut ghoul2, arg(2), &mut rduParams);
        }
        0
    } else if op == MpCgameImport::CG_G2_RAGPCJCONSTRAINT as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: the handle and the two vectors are module-space (§D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            g2api_rag_pcj_constraint(
                g2,
                &mut ghoul2,
                &bone_name,
                *(vma(vc, args, 3) as *const vec3_t),
                *(vma(vc, args, 4) as *const vec3_t),
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGPCJGRADIENTSPEED as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // `args[1]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_rag_pcj_gradient_speed(g2, &mut ghoul2, &bone_name, vmf(vc, args, 3)) as c_int
    } else if op == MpCgameImport::CG_G2_RAGEFFECTORGOAL as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        let pos_ptr = vma(vc, args, 3) as *const vec3_t;
        // SAFETY: the handle and a non-NULL `VMA(3)` are module-space (§D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let pos = if pos_ptr.is_null() {
                None
            } else {
                Some(*pos_ptr)
            };
            g2api_rag_effector_goal(g2, &mut ghoul2, &bone_name, pos) as c_int
        }
    } else if op == MpCgameImport::CG_G2_GETRAGBONEPOS as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        let pos_out = vma(vc, args, 3) as *mut vec3_t;
        // SAFETY: the handle, the three inputs, and the out-param are all
        // module-space (porting-rules §D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            match g2api_get_rag_bone_pos(
                g2,
                &mut ghoul2,
                &bone_name,
                *(vma(vc, args, 4) as *const vec3_t),
                *(vma(vc, args, 5) as *const vec3_t),
                *(vma(vc, args, 6) as *const vec3_t),
            ) {
                Some(pos) => {
                    *pos_out = pos;
                    qtrue
                }
                None => qfalse,
            }
        }
    } else if op == MpCgameImport::CG_G2_RAGEFFECTORKICK as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // SAFETY: the handle and the velocity are module-space (§D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            g2api_rag_effector_kick(
                g2,
                &mut ghoul2,
                &bone_name,
                *(vma(vc, args, 3) as *const vec3_t),
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_RAGFORCESOLVE as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_rag_force_solve(g2, &mut ghoul2, arg(2) != 0) as c_int
    } else if op == MpCgameImport::CG_G2_SETBONEIKSTATE as c_int {
        let bone_name_ptr = vma(vc, args, 3) as *const c_char;
        let bone_name = if bone_name_ptr.is_null() {
            None
        } else {
            Some(cstr_to_string(bone_name_ptr))
        };
        let params_ptr = vma(vc, args, 5) as *mut sharedSetBoneIKStateParams_t;
        // SAFETY: the handle and a non-NULL `VMA(5)` are module-space (§D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let params = if params_ptr.is_null() {
                None
            } else {
                Some(&mut *params_ptr)
            };
            g2api_set_bone_ik_state(
                g2,
                view,
                &mut ghoul2,
                arg(2),
                bone_name.as_deref(),
                arg(4),
                params,
            ) as c_int
        }
    } else if op == MpCgameImport::CG_G2_IKMOVE as c_int {
        // SAFETY: the handle and `VMA(3)` are module-space (porting-rules §D11).
        unsafe {
            let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
            let params = &mut *(vma(vc, args, 3) as *mut sharedIKMoveParams_t);
            g2api_ik_move(g2, view, &mut ghoul2, arg(2), params) as c_int
        }
    } else if op == MpCgameImport::CG_G2_REMOVEBONE as c_int {
        let bone_name = cstr_to_string(vma(vc, args, 2) as *const c_char);
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(3));
        g2api_remove_bone(g2, view, ghl_info, &bone_name) as c_int
    } else if op == MpCgameImport::CG_G2_ATTACHINSTANCETOENTNUM as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let mut ghoul2 = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        g2api_attach_instance_to_ent_num(g2, &mut ghoul2, arg(2), arg(3) != 0);
        0
    } else if op == MpCgameImport::CG_G2_CLEARATTACHEDINSTANCE as c_int {
        g2api_clear_attached_instance(g2, arg(1));
        0
    } else if op == MpCgameImport::CG_G2_CLEANENTATTACHMENTS as c_int {
        g2api_clean_ent_attachments(g2);
        0
    } else if op == MpCgameImport::CG_G2_OVERRIDESERVER as c_int {
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, 0);
        g2api_override_server_with_client_data(g2, ghl_info) as c_int
    } else if op == MpCgameImport::CG_G2_GETSURFACENAME as c_int {
        // Since returning a pointer in such a way to a VM seems to cause reliability
        // problems, we shove data into the pointer the vm passes instead.
        let point = vma(vc, args, 4) as *mut c_char;
        // `args[1]` is the module's ghoul2 token (§D11).
        let g2v = CGhoul2Info_v::from_token(argw(1) as *mut c_void);
        let ghl_info = g2_info(g2, &g2v, arg(3));
        let local = g2api_get_surface_name(g2, view, ghl_info, arg(2));
        if !local.is_empty() {
            let local_c = std::ffi::CString::new(local).unwrap_or_default();
            // SAFETY: `VMA(4)` is the module's seam out-buffer (§D11).
            unsafe { libc::strcpy(point, local_c.as_ptr()) };
        }
        0
    } else if op == MpCgameImport::CG_SP_GETSTRINGTEXTSTRING as c_int {
        let key = cstr_to_string(vma(vc, args, 1) as *const c_char);
        let dest_ptr = vma(vc, args, 2) as *mut c_char;
        let text = SE_GetString(view, &key);
        // SAFETY: `VMA(2)` is the module's seam out-buffer (porting-rules §D11).
        unsafe {
            if !text.is_empty() {
                let dest = core::slice::from_raw_parts_mut(dest_ptr, arg(3) as usize);
                Q_strncpyz(dest, &text, arg(3) as usize);
                qtrue
            } else {
                Com_sprintf(dest_ptr, arg(3), &format!("??{}", key));
                qfalse
            }
        }
    } else if op == MpCgameImport::CG_SET_SHARED_BUFFER as c_int {
        cl.cl.mSharedMemory = vma(vc, args, 1) as *mut c_char;
        0
    } else if op == MpCgameImport::CG_CM_REGISTER_TERRAIN as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        // SAFETY: `view.cm` is a real field borrow; `register_terrain`'s host
        // calls never touch it, so the raw reborrow follows the DEC-23 rule.
        let cm = unsafe { &mut *(view.cm as *mut CollisionWorld) };
        register_terrain(cm, view, &name, false).raw()
    } else if op == MpCgameImport::CG_RMG_INIT as c_int {
        // Raven's `TheRandomMissionManager` global IS the ported `RmManager`
        // (ruling 28/RMG-D1), so the lazy `new CRMManager` allocation folds
        // away. `SpawnMission`'s call is the §20-dropped arm (ruling 38), and
        // `UpdatePatches` already runs inside `register_terrain`.
        if view.common.cvar(view.common.com_sv_running).integer == 0 {
            // Don't do this if we are connected locally.
            // Raven passes `cmg.landScape` itself; the Rust twin passes its
            // handle, which `register_terrain` always builds as `TerrainHandle(0)`.
            if view.cm.land_scape.is_some() {
                rmg.set_landscape(TerrainHandle(0));
            }
            // SAFETY: `view.cm` is a real field borrow; `load_mission`'s host
            // calls never touch it, so the raw reborrow follows the DEC-23 rule.
            let cm = unsafe { &mut *(view.cm as *mut CollisionWorld) };
            rmg.load_mission(cm, view, false);
        }
        //TODO: Port RM_CreateRandomModels
        // Source: oracle/codemp/RMG/RM_Terrain.cpp:482-495
        // The body needs `CRMLandScape`'s client half (`LoadMiscentDef`,
        // `LoadDensityMap`, `SpawnPatchModels`) plus `CM_TerrainPatchIterate`.
        // None of that is ported, and the terrain lane (gh#29) owns it, so the
        // arm spawns no random models yet.
        0
    } else if op == MpCgameImport::CG_RE_INIT_RENDERER_TERRAIN as c_int {
        let name = cstr_to_string(vma(vc, args, 1) as *const c_char);
        RE_InitRendererTerrain(view.common, &name);
        0
    } else if op == MpCgameImport::CG_R_WEATHER_CONTENTS_OVERRIDE as c_int {
        // contentOverride = args[1]; (dead in the oracle)
        0
    } else if op == MpCgameImport::CG_R_WORLDEFFECTCOMMAND as c_int {
        let command = cstr_bytes(vma(vc, args, 1) as *const c_char);
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        re.world_effects.R_WorldEffectCommand(
            &mut re.qs,
            view,
            &re.cvars,
            Arc::make_mut(&mut re.sim.published),
            rm,
            &mut re.img_state,
            Some(command),
        );
        0
    } else if op == MpCgameImport::CG_WE_ADDWEATHERZONE as c_int {
        let mins = unsafe { *(vma(vc, args, 1) as *const vec3_t) };
        let maxs = unsafe { *(vma(vc, args, 2) as *const vec3_t) };
        // SAFETY: view-constructor slot, single-threaded, no other live cast.
        let re = unsafe { re_from_view(view) };
        re.world_effects.R_AddWeatherZone(mins, maxs);
        0
    } else {
        com_error(
            errorParm_t::ERR_DROP,
            format!("Bad cgame system trap: {}", op),
        );
    }
}
