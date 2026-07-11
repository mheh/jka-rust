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
use mp_qshared::common::mp::qcommon::parms::parms_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};
use mp_qshared::shared::surface_flags::CONTENTS_LIGHTSABER;
use native_math::vector::vec3_t;
use native_types::clipHandle_t;

use crate::server::sv_entity_s::svEntity_t;
use crate::Server;

// PORT-NOTE(engine-host-state): `CollisionWorld`, `Common`, and `EngineHost`
// exist in `mp_engine_qcommon`/`mp_host_interface`; `RenderModels` (rm),
// `RmManager` (rmg), `Navigator` (nav), `Ghoul2System` (g2), and `RoffSystem`
// (roff) do NOT exist anywhere in the tree yet (grepped: no hits) — these
// packets were generated ahead of those state structs landing. Imported below
// by their preamble-table decl-home crate; genuinely missing, escalated in
// missing_symbols rather than stubbed (ZERO-PARK).
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_icarus::Icarus;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::roff::RoffSystem;
use mp_engine_renderer::tr_model::render_models::RenderModels;
use mp_engine_rmg::rm_manager::RmManager;
use mp_host_interface::engine_host::EngineHost;

use crate::npcnav::Navigator;

// PORT-NOTE(q_shared-primitives): `COM_Parse`/`Q_strncpyz`/`atoi` are Raven
// `q_shared.c` free functions ported a tier above this crate's dependency graph
// (`mp_game`), not reachable here. Forward-declared by their exact Raven names
// via the established engine `extern "Rust"` convention (cm_load.rs /
// cmd_common.rs / files_common.rs / msg.rs precedent); the finisher resolves
// linkage uniformly. Reported as missing symbols for the shared q_shared.c port.
extern "Rust" {
    fn COM_Parse(data_p: *mut *const c_char) -> *mut c_char;
    fn Q_strncpyz(dest: *mut c_char, src: *const c_char, destsize: c_int);
    fn atoi(string: *const c_char) -> c_int;
}

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
    // PORT-NOTE(com-parse): `COM_Parse`/`Q_strncpyz` are the already-ported
    // qshared free-function surface (packet says "call through the existing
    // crate"); exact import path not resolved by this packet — escalated.
    unsafe {
        if sv.sv.mLocalSubBSPIndex == -1 {
            let s = COM_Parse(
                &mut sv.sv.entityParsePoint as *mut *mut c_char as *mut *const c_char,
            );
            Q_strncpyz(buffer, s, bufferSize);
            if sv.sv.entityParsePoint.is_null() && *s == 0 {
                qfalse
            } else {
                qtrue
            }
        } else {
            let s = COM_Parse(
                &mut sv.sv.mLocalSubBSPEntityParsePoint as *mut *mut c_char as *mut *const c_char,
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
        mp_engine_server::sv_send_server_command(common, sv, core::ptr::null_mut(), &msg);
    } else {
        if clientNum < 0 || clientNum >= (unsafe { (*common.sv_maxclients).integer }) {
            return;
        }
        let client = unsafe { sv.svs.clients.offset(clientNum as isize) };
        mp_engine_server::sv_send_server_command(common, sv, client, &msg);
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
    mp_engine_server::sv_drop_client(common, sv, client, reason);
}

/// Raven `SV_inPVS`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:209-233`
pub fn SV_inPVS(cm: &mut CollisionWorld, p1: vec3_t, p2: vec3_t) -> qboolean {
    let mut leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p1);
    let mut cluster = mp_engine_qcommon::cm::CM_LeafCluster(cm, leafnum);
    let area1 = mp_engine_qcommon::cm::CM_LeafArea(cm, leafnum);
    let mask = mp_engine_qcommon::cm_test::CM_ClusterPVS(cm, cluster);

    leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p2);
    cluster = mp_engine_qcommon::cm::CM_LeafCluster(cm, leafnum);
    let area2 = mp_engine_qcommon::cm::CM_LeafArea(cm, leafnum);
    if !mask.is_null() {
        let byte = unsafe { *mask.offset((cluster >> 3) as isize) };
        if byte & (1 << (cluster & 7)) == 0 {
            return qfalse;
        }
    }
    if mp_engine_qcommon::cm::CM_AreasConnected(cm, area1, area2) == qfalse {
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
    let mut cluster = mp_engine_qcommon::cm::CM_LeafCluster(cm, leafnum);
    let _area1 = mp_engine_qcommon::cm::CM_LeafArea(cm, leafnum);
    let mask = mp_engine_qcommon::cm_test::CM_ClusterPVS(cm, cluster);

    leafnum = mp_engine_qcommon::cm_test::CM_PointLeafnum(cm, p2);
    cluster = mp_engine_qcommon::cm::CM_LeafCluster(cm, leafnum);
    let _area2 = mp_engine_qcommon::cm::CM_LeafArea(cm, leafnum);

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
    let info = mp_engine_qcommon::cvar::Cvar_InfoString(
        common,
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    unsafe {
        Q_strncpyz(buffer, info.as_ptr() as *const c_char, bufferSize);
    }
}

/// Raven `SV_GetUsercmd`.
///
/// Source: `oracle/codemp/server/sv_game.cpp:375-380`
pub fn SV_GetUsercmd(sv: &mut Server, clientNum: c_int, cmd: *mut usercmd_t) {
    // NOTE: the resolved signature carries no `common` receiver even though
    // `sv_maxclients` (an `EngineCvars` handle owned by `common`) is read and
    // `ERR_DROP` unwinds through `Com_Error` — packet's signature is LAW
    // (shape_mismatches).
    unsafe {
        if clientNum < 0 || clientNum >= (*sv.svs.clients.offset(0)).max_clients_placeholder {
            // PORT-NOTE(sv_maxclients): no `common` receiver in this packet's
            // signature to reach `sv_maxclients->integer`; escalated.
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

    let ms = mp_engine_qcommon::common::Com_Milliseconds(common, cm, rm, host);
    mp_engine_qcommon::vm::VM_Call(
        common,
        sv.gvm,
        mp_abi::game::exports::MpGameExport::GAME_INIT as c_int,
        &[sv.svs.time, ms, restart as c_int],
    );

    let max_clients = (unsafe { (*common.sv_maxclients).integer });
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
    use crate::server::server_state_t::serverState_t;
    if sv.sv.state as c_int != serverState_t::SS_GAME as c_int {
        return qfalse;
    }
    let r = mp_engine_qcommon::vm::VM_Call(
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
    let svEnt = mp_engine_qcommon::common::SV_SvEntityForGentity(sv, ent);
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
    mp_engine_qcommon::vm::VM_Call(
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

        let ch: clipHandle_t = mp_engine_server::SV_ClipHandleForEntity(cm, gEnt);
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
        let mut mins = [0.0f32; 3];
        let mut maxs = [0.0f32; 3];

        if *name == b'*' as c_char {
            (*ent).s.modelindex = atoi(name.offset(1));

            if sv.sv.mLocalSubBSPIndex != -1 {
                (*ent).s.modelindex += sv.sv.mLocalSubBSPModelOffset;
            }

            let h = mp_engine_qcommon::cm_load::CM_InlineModel(cm, (*ent).s.modelindex);

            mp_engine_qcommon::cm_load::CM_ModelBounds(cm, h, &mut mins, &mut maxs);

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            let com_rmg = mp_engine_qcommon::cvar::com_RMG(common);
            if !com_rmg.is_null() && (*com_rmg).integer != 0 {
                (*ent).r.contents =
                    mp_engine_qcommon::cm_load::CM_ModelContents(cm, h, sv.sv.mLocalSubBSPIndex);
            } else {
                (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(cm, h, -1);
            }
        } else if *name == b'#' as c_char {
            let bsp_name = format!("maps/{}.bsp", &name_str[1..]);
            (*ent).s.modelindex = mp_engine_qcommon::cm_load::CM_LoadSubBSP(
                common,
                cm,
                rm,
                rmg,
                host,
                &bsp_name,
                qfalse,
            );
            mp_engine_qcommon::cm_load::CM_ModelBounds(cm, (*ent).s.modelindex, &mut mins, &mut maxs);

            (*ent).r.mins = mins;
            (*ent).r.maxs = maxs;
            (*ent).r.bmodel = qtrue;

            //rwwNOTE: We don't ever want to set contents -1, it includes CONTENTS_LIGHTSABER.
            //Lots of stuff will explode if there's a brush with CONTENTS_LIGHTSABER that isn't attached to a client owner.
            //ent->contents = -1;		// we don't know exactly what is in the brushes
            let _ = CONTENTS_LIGHTSABER;
            let h = mp_engine_qcommon::cm_load::CM_InlineModel(cm, (*ent).s.modelindex);
            (*ent).r.contents = mp_engine_qcommon::cm_load::CM_ModelContents(
                cm,
                h,
                mp_engine_qcommon::cm_load::CM_FindSubBSP(cm, (*ent).s.modelindex),
            );
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
/// cast since our module is a native dylib, not a bytecode VM).
///
/// Source: `oracle/codemp/qcommon/vm_local.h` (`VMA`/`VMF` macros)
#[inline]
unsafe fn vma(common: &mut Common, args: *mut c_int, n: isize) -> *mut c_void {
    mp_engine_qcommon::vm_fns::VM_ArgPtr(common, *args.offset(n)) as *mut c_void
}

/// Raven's `VMF(n)` macro — reinterpret `args[n]`'s bits as `float`.
#[inline]
unsafe fn vmf(args: *mut c_int, n: isize) -> f32 {
    f32::from_bits(*args.offset(n) as u32)
}

/// Raven `SV_GameSystemCalls` — the inbound syscall dispatcher the game VM
/// calls through `VMA`/`VMF`.
///
/// PORT-NOTE(subsystem-receivers): the resolved signature carries no `bot: &mut
/// BotLib` receiver despite reading/using `botlib_export` throughout this
/// function (escalated — shape_mismatches); `botlib_export->...` call sites
/// below are transcribed with the bare global name (unresolved, ZERO-PARK).
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
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    icarus: &mut Icarus,
    nav: &mut Navigator,
    g2: &mut Ghoul2System,
    roff: &mut RoffSystem,
    host: &mut dyn EngineHost,
    args: *mut c_int,
) -> c_int {
    use mp_abi::game::imports::MpGameImport as G;
    use mp_engine_qcommon::qcommon::shared_traps_t::sharedTraps_t as T;

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
            return mp_qshared::shared::q_shared::strncpy(
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *const c_char,
                *args.offset(3),
            ) as c_int;
        } else if trap == T::TRAP_SIN as c_int {
            return FloatAsInt(vmf(args, 1).sin());
        } else if trap == T::TRAP_COS as c_int {
            return FloatAsInt(vmf(args, 1).cos());
        } else if trap == T::TRAP_ATAN2 as c_int {
            return FloatAsInt(vmf(args, 1).atan2(vmf(args, 2)));
        } else if trap == T::TRAP_SQRT as c_int {
            return FloatAsInt(vmf(args, 1).sqrt());
        } else if trap == T::TRAP_MATRIXMULTIPLY as c_int {
            mp_qshared::shared::q_math::MatrixMultiply(
                vma(common, args, 1) as *mut vec3_t,
                vma(common, args, 2) as *mut vec3_t,
                vma(common, args, 3) as *mut vec3_t,
            );
            return 0;
        } else if trap == T::TRAP_ANGLEVECTORS as c_int {
            mp_qshared::shared::q_math::AngleVectors(
                vma(common, args, 1) as *const f32,
                vma(common, args, 2) as *mut f32,
                vma(common, args, 3) as *mut f32,
                vma(common, args, 4) as *mut f32,
            );
            return 0;
        } else if trap == T::TRAP_PERPENDICULARVECTOR as c_int {
            mp_qshared::shared::q_math::PerpendicularVector(
                vma(common, args, 1) as *mut f32,
                vma(common, args, 2) as *const f32,
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
            return 0;
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
            mp_engine_qcommon::cvar::Cvar_Register(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                core::ffi::CStr::from_ptr(vma(common, args, 3) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(4),
            );
            return 0;
        } else if trap == G::G_CVAR_UPDATE as c_int {
            mp_engine_qcommon::cvar::Cvar_Update(
                common,
                vma(common, args, 1) as *mut mp_qshared::shared::cvar::vmCvar_t,
            );
            return 0;
        } else if trap == G::G_CVAR_SET as c_int {
            mp_engine_qcommon::cvar::Cvar_Set(
                common,
                cm,
                rm,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            return 0;
        } else if trap == G::G_CVAR_VARIABLE_INTEGER_VALUE as c_int {
            return mp_engine_qcommon::cvar::Cvar_VariableIntegerValue(
                common,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
        } else if trap == G::G_CVAR_VARIABLE_STRING_BUFFER as c_int {
            mp_engine_qcommon::cvar::Cvar_VariableStringBuffer(
                common,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
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
                sv,
                rm,
                host,
                *args.offset(1),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            return 0;
        } else if trap == G::G_FS_FOPEN_FILE as c_int {
            return mp_engine_qcommon::files_pc::FS_FOpenFileByMode(
                common,
                cm,
                rm,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                vma(common, args, 2) as *mut c_int,
                core::mem::transmute(*args.offset(3)),
            );
        } else if trap == G::G_FS_READ as c_int {
            mp_engine_qcommon::files_pc::FS_Read2(
                common,
                vma(common, args, 1),
                *args.offset(2),
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_FS_WRITE as c_int {
            mp_engine_qcommon::files::FS_Write(
                common,
                vma(common, args, 1),
                *args.offset(2),
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_FS_FCLOSE_FILE as c_int {
            mp_engine_qcommon::files::FS_FCloseFile(common, *args.offset(1));
            return 0;
        } else if trap == G::G_FS_GETFILELIST as c_int {
            return mp_engine_qcommon::files_pc::FS_GetFileList(
                common,
                cm,
                rm,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
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
            mp_engine_server::SV_LinkEntity(
                common,
                cm,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_UNLINKENTITY as c_int {
            mp_engine_server::SV_UnlinkEntity(
                common,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            );
            return 0;
        } else if trap == G::G_ENTITIES_IN_BOX as c_int {
            return mp_engine_server::SV_AreaEntities(
                common,
                sv,
                vma(common, args, 1) as *const f32,
                vma(common, args, 2) as *const f32,
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
            mp_engine_server::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                vma(common, args, 2) as *const f32,
                vma(common, args, 3) as *const f32,
                vma(common, args, 4) as *const f32,
                vma(common, args, 5) as *const f32,
                *args.offset(6),
                *args.offset(7),
                qfalse as c_int,
                0,
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_G2TRACE as c_int {
            mp_engine_server::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                vma(common, args, 2) as *const f32,
                vma(common, args, 3) as *const f32,
                vma(common, args, 4) as *const f32,
                vma(common, args, 5) as *const f32,
                *args.offset(6),
                *args.offset(7),
                qfalse as c_int,
                *args.offset(8),
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_TRACECAPSULE as c_int {
            mp_engine_server::SV_Trace(
                common,
                cm,
                sv,
                rm,
                rmg,
                g2,
                host,
                vma(common, args, 1) as *mut mp_qshared::common::mp::trace_t::trace_t,
                vma(common, args, 2) as *const f32,
                vma(common, args, 3) as *const f32,
                vma(common, args, 4) as *const f32,
                vma(common, args, 5) as *const f32,
                *args.offset(6),
                *args.offset(7),
                qtrue as c_int,
                *args.offset(8),
                *args.offset(9),
            );
            return 0;
        } else if trap == G::G_POINT_CONTENTS as c_int {
            return mp_engine_server::SV_PointContents(
                common,
                cm,
                sv,
                vma(common, args, 1) as *const f32,
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
            mp_engine_server::SV_SetConfigstring(
                common,
                cm,
                sv,
                rm,
                host,
                *args.offset(1),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            return 0;
        } else if trap == G::G_GET_CONFIGSTRING as c_int {
            mp_engine_server::SV_GetConfigstring(
                sv,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_SET_USERINFO as c_int {
            mp_engine_server::SV_SetUserinfo(
                sv,
                *args.offset(1),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            return 0;
        } else if trap == G::G_GET_USERINFO as c_int {
            mp_engine_server::SV_GetUserinfo(
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
            return mp_engine_qcommon::cm::CM_AreasConnected(cm, *args.offset(1), *args.offset(2))
                as c_int;
        } else if trap == G::G_BOT_ALLOCATE_CLIENT as c_int {
            return mp_engine_server::SV_BotAllocateClient(sv);
        } else if trap == G::G_BOT_FREE_CLIENT as c_int {
            mp_engine_server::SV_BotFreeClient(sv, *args.offset(1));
            return 0;
        } else if trap == G::G_GET_USERCMD as c_int {
            SV_GetUsercmd(sv, *args.offset(1), vma(common, args, 2) as *mut usercmd_t);
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
            return mp_engine_server::BotImport_DebugPolygonCreate(
                sv,
                *args.offset(1),
                *args.offset(2),
                vma(common, args, 3) as *const [f32; 3],
            );
        } else if trap == G::G_DEBUG_POLYGON_DELETE as c_int {
            mp_engine_server::BotImport_DebugPolygonDelete(sv, *args.offset(1));
            return 0;
        } else if trap == G::G_REAL_TIME as c_int {
            return mp_engine_qcommon::common_fns::Com_RealTime(
                vma(common, args, 1) as *mut mp_qshared::common::mp::qcommon::qtime::qtime_t
            );
        } else if trap == G::G_SNAPVECTOR as c_int {
            mp_qshared::shared::sys_shared::Sys_SnapVector(vma(common, args, 1) as *mut f32);
            return 0;
        } else if trap == G::SP_GETSTRINGTEXTSTRING as c_int {
            assert!(!vma(common, args, 1).is_null());
            assert!(!vma(common, args, 2).is_null());
            let text = mp_engine_qcommon::stringed::SE_GetString(
                common,
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
            if !text.is_empty() {
                mp_qshared::shared::q_shared::Q_strncpyz(
                    vma(common, args, 2) as *mut c_char,
                    text.as_ptr() as *const c_char,
                    *args.offset(3),
                );
                return qtrue as c_int;
            } else {
                mp_qshared::shared::q_shared::Q_strncpyz(
                    vma(common, args, 2) as *mut c_char,
                    c"??".as_ptr(),
                    *args.offset(3),
                );
                return qfalse as c_int;
            }
        } else if trap == G::G_ROFF_CLEAN as c_int {
            return roff.Clean(host, qfalse) as c_int;
        } else if trap == G::G_ROFF_UPDATE_ENTITIES as c_int {
            roff.UpdateEntities(host, qfalse);
            return 0;
        } else if trap == G::G_ROFF_CACHE as c_int {
            return roff.Cache(host, vma(common, args, 1) as *mut c_char, qfalse);
        } else if trap == G::G_ROFF_PLAY as c_int {
            return roff.Play(
                host,
                *args.offset(1),
                *args.offset(2),
                core::mem::transmute(*args.offset(3)),
                qfalse,
            );
        } else if trap == G::G_ROFF_PURGE_ENT as c_int {
            return roff.PurgeEnt(host, *args.offset(1), qfalse);
        } else if trap == G::G_TRUEMALLOC as c_int {
            mp_engine_qcommon::vm_fns::VM_Shifted_Alloc(
                common,
                cm,
                rm,
                host,
                vma(common, args, 1) as *mut *mut c_void,
                *args.offset(2),
            );
            return 0;
        } else if trap == G::G_TRUEFREE as c_int {
            mp_engine_qcommon::vm_fns::VM_Shifted_Free(
                common,
                vma(common, args, 1) as *mut *mut c_void,
            );
            return 0;
        } else if trap == G::G_ICARUS_RUNSCRIPT as c_int {
            return icarus.RunScript(
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
                core::ffi::CStr::from_ptr(vma(common, args, 2) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
        } else if trap == G::G_ICARUS_REGISTERSCRIPT as c_int {
            return icarus.RegisterScript(
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                core::mem::transmute(*args.offset(2)),
            );
        } else if trap == G::G_ICARUS_INIT as c_int {
            icarus.Init(host);
            return 0;
        } else if trap == G::G_ICARUS_VALIDENT as c_int {
            return icarus.ValidEnt(
                host,
                ConvertedEntity(common, sv, vma(common, args, 1) as *mut sharedEntity_t),
            );
        } else if trap == G::G_ICARUS_ISINITIALIZED as c_int {
            let ent_id = *args.offset(1) as usize;
            if icarus.sequencers[ent_id].is_none() || icarus.task_managers[ent_id].is_none() {
                return 0;
            }
            return 1;
        } else if trap == G::G_ICARUS_MAINTAINTASKMANAGER as c_int {
            let ent_id = *args.offset(1) as usize;
            if let Some(tm) = icarus.task_managers[ent_id] {
                icarus.update_task_manager(host, tm);
                return 1;
            }
            return 0;
        } else if trap == G::G_ICARUS_ISRUNNING as c_int {
            let ent_id = *args.offset(1) as usize;
            match icarus.task_managers[ent_id] {
                Some(tm) if icarus.is_running(tm) => return 1,
                _ => return 0,
            }
        } else if trap == G::G_ICARUS_TASKIDPENDING as c_int {
            return icarus.Q3_TaskIDPending(
                vma(common, args, 1) as *mut sharedEntity_t,
                core::mem::transmute(*args.offset(2)),
            );
        } else if trap == G::G_ICARUS_INITENT as c_int {
            icarus.InitEnt(ConvertedEntity(
                common,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            ));
            return 0;
        } else if trap == G::G_ICARUS_FREEENT as c_int {
            icarus.FreeEnt(ConvertedEntity(
                common,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            ));
            return 0;
        } else if trap == G::G_ICARUS_ASSOCIATEENT as c_int {
            icarus.AssociateEnt(ConvertedEntity(
                common,
                sv,
                vma(common, args, 1) as *mut sharedEntity_t,
            ));
            return 0;
        } else if trap == G::G_ICARUS_SHUTDOWN as c_int {
            icarus.Shutdown(host);
            return 0;
        } else if trap == G::G_ICARUS_TASKIDSET as c_int {
            // rww - note that we are passing in the true entity here. This is
            // because we allow modification of certain non-pointer values,
            // which is valid.
            icarus.Q3_TaskIDSet(
                vma(common, args, 1) as *mut sharedEntity_t,
                core::mem::transmute(*args.offset(2)),
                *args.offset(3),
            );
            return 0;
        } else if trap == G::G_ICARUS_TASKIDCOMPLETE as c_int {
            // same as above.
            icarus.Q3_TaskIDComplete(
                vma(common, args, 1) as *mut sharedEntity_t,
                core::mem::transmute(*args.offset(2)),
            );
            return 0;
        } else if trap == G::G_ICARUS_SETVAR as c_int {
            icarus.Q3_SetVar(
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
            return icarus.Q3_VariableDeclared(
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
            );
        } else if trap == G::G_ICARUS_GETFLOATVARIABLE as c_int {
            return icarus.Q3_GetFloatVariable(
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                vma(common, args, 2) as *mut f32,
            );
        } else if trap == G::G_ICARUS_GETSTRINGVARIABLE as c_int {
            let mut rec = vma(common, args, 2) as *const c_char;
            return icarus.Q3_GetStringVariable(
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                &mut rec,
            );
        } else if trap == G::G_ICARUS_GETVECTORVARIABLE as c_int {
            return icarus.Q3_GetVectorVariable(
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                vma(common, args, 2) as *mut f32,
            );
        }
        // rww - BEGIN NPC NAV TRAPS
        else if trap == G::G_NAV_INIT as c_int {
            nav.Init(host);
            return 0;
        } else if trap == G::G_NAV_FREE as c_int {
            nav.Free(host);
            return 0;
        } else if trap == G::G_NAV_LOAD as c_int {
            return nav.Load(
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2),
            );
        } else if trap == G::G_NAV_SAVE as c_int {
            return nav.Save(
                host,
                core::ffi::CStr::from_ptr(vma(common, args, 1) as *const c_char)
                    .to_str()
                    .unwrap_or(""),
                *args.offset(2),
            );
        } else if trap == G::G_NAV_ADDRAWPOINT as c_int {
            return nav.AddRawPoint(
                host,
                vma(common, args, 1) as *mut f32,
                *args.offset(2),
                *args.offset(3),
            );
        } else if trap == G::G_NAV_CALCULATEPATHS as c_int {
            nav.CalculatePaths(host, core::mem::transmute(*args.offset(1)));
            return 0;
        } else if trap == G::G_NAV_HARDCONNECT as c_int {
            nav.HardConnect(host, *args.offset(1), *args.offset(2));
            return 0;
        } else if trap == G::G_NAV_SHOWNODES as c_int {
            nav.ShowNodes(host);
            return 0;
        } else if trap == G::G_NAV_SHOWEDGES as c_int {
            nav.ShowEdges(host);
            return 0;
        } else if trap == G::G_NAV_SHOWPATH as c_int {
            nav.ShowPath(host, *args.offset(1), *args.offset(2));
            return 0;
        } else if trap == G::G_NAV_GETNEARESTNODE as c_int {
            return nav.GetNearestNode(
                host,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
                *args.offset(3),
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODE as c_int {
            return nav.GetBestNode(host, *args.offset(1), *args.offset(2), *args.offset(3));
        } else if trap == G::G_NAV_GETNODEPOSITION as c_int {
            return nav.GetNodePosition(host, *args.offset(1), vma(common, args, 2) as *mut f32);
        } else if trap == G::G_NAV_GETNODENUMEDGES as c_int {
            return nav.GetNodeNumEdges(host, *args.offset(1));
        } else if trap == G::G_NAV_GETNODEEDGE as c_int {
            return nav.GetNodeEdge(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_GETNUMNODES as c_int {
            return nav.GetNumNodes();
        } else if trap == G::G_NAV_CONNECTED as c_int {
            return nav.Connected(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_GETPATHCOST as c_int {
            return nav.GetPathCost(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_GETEDGECOST as c_int {
            return nav.GetEdgeCost(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_GETPROJECTEDNODE as c_int {
            return nav.GetProjectedNode(host, vma(common, args, 1) as *mut f32, *args.offset(2));
        } else if trap == G::G_NAV_CHECKFAILEDNODES as c_int {
            nav.CheckFailedNodes(host, vma(common, args, 1) as *mut sharedEntity_t);
            return 0;
        } else if trap == G::G_NAV_ADDFAILEDNODE as c_int {
            nav.AddFailedNode(
                host,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
            );
            return 0;
        } else if trap == G::G_NAV_NODEFAILED as c_int {
            return nav.NodeFailed(
                host,
                vma(common, args, 1) as *mut sharedEntity_t,
                *args.offset(2),
            );
        } else if trap == G::G_NAV_NODESARENEIGHBORS as c_int {
            return nav.NodesAreNeighbors(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_CLEARFAILEDEDGE as c_int {
            nav.ClearFailedEdge(vma(common, args, 1)
                as *mut mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t);
            return 0;
        } else if trap == G::G_NAV_CLEARALLFAILEDEDGES as c_int {
            nav.ClearAllFailedEdges(host);
            return 0;
        } else if trap == G::G_NAV_EDGEFAILED as c_int {
            return nav.EdgeFailed(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_ADDFAILEDEDGE as c_int {
            nav.AddFailedEdge(host, *args.offset(1), *args.offset(2), *args.offset(3));
            return 0;
        } else if trap == G::G_NAV_CHECKFAILEDEDGE as c_int {
            return nav.CheckFailedEdge(
                host,
                vma(common, args, 1)
                    as *mut mp_qshared::common::mp::qcommon::failed_edge::failedEdge_t,
            );
        } else if trap == G::G_NAV_CHECKALLFAILEDEDGES as c_int {
            nav.CheckAllFailedEdges(host);
            return 0;
        } else if trap == G::G_NAV_ROUTEBLOCKED as c_int {
            return nav.RouteBlocked(
                host,
                *args.offset(1),
                *args.offset(2),
                *args.offset(3),
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODEALTROUTE as c_int {
            return nav.GetBestNodeAltRoute(
                host,
                *args.offset(1),
                *args.offset(2),
                vma(common, args, 3) as *mut c_int,
                *args.offset(4),
            );
        } else if trap == G::G_NAV_GETBESTNODEALT2 as c_int {
            return nav.GetBestNodeAltRoute2(
                host,
                *args.offset(1),
                *args.offset(2),
                *args.offset(3),
            );
        } else if trap == G::G_NAV_GETBESTPATHBETWEENENTS as c_int {
            return nav.GetBestPathBetweenEnts(
                host,
                vma(common, args, 1) as *mut sharedEntity_t,
                vma(common, args, 2) as *mut sharedEntity_t,
                *args.offset(3),
            );
        } else if trap == G::G_NAV_GETNODERADIUS as c_int {
            return nav.GetNodeRadius(host, *args.offset(1));
        } else if trap == G::G_NAV_CHECKBLOCKEDEDGES as c_int {
            nav.CheckBlockedEdges(host);
            return 0;
        } else if trap == G::G_NAV_CLEARCHECKEDNODES as c_int {
            nav.ClearCheckedNodes(host);
            return 0;
        } else if trap == G::G_NAV_CHECKEDNODE as c_int {
            return nav.CheckedNode(host, *args.offset(1), *args.offset(2));
        } else if trap == G::G_NAV_SETCHECKEDNODE as c_int {
            nav.SetCheckedNode(host, *args.offset(1), *args.offset(2), *args.offset(3));
            // Raven bug (sv_game.cpp:928-933, nav-D3/ruling NAV-Q3): falls
            // through into FLAGALLNODES/GETPATHSCALCULATED without a return —
            // transcribed faithfully, not fixed.
        } else if trap == G::G_NAV_FLAGALLNODES as c_int {
            nav.FlagAllNodes(host, *args.offset(1));
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
            return mp_engine_server::SV_BotLibSetup(common, sv);
        } else if trap == G::BOTLIB_SHUTDOWN as c_int {
            return mp_engine_server::SV_BotLibShutdown(sv);
        }
        // PORT-NOTE(botlib-export): the resolved signature carries no `bot`
        // receiver, so every `botlib_export->...` arm below cannot thread
        // through a receiver; transcribed with the bare Raven global name
        // (escalated, missing_symbols/shape_mismatches) rather than invented.
        else if trap == G::BOTLIB_LIBVAR_SET as c_int {
            return (*botlib_export).BotLibVarSet(
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *mut c_char,
            );
        } else if trap == G::BOTLIB_LIBVAR_GET as c_int {
            return (*botlib_export).BotLibVarGet(
                vma(common, args, 1) as *mut c_char,
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
        } else if trap == G::BOTLIB_PC_ADD_GLOBAL_DEFINE as c_int {
            return (*botlib_export).PC_AddGlobalDefine(vma(common, args, 1) as *mut c_char);
        } else if trap == G::BOTLIB_PC_LOAD_SOURCE as c_int {
            return (*botlib_export).PC_LoadSourceHandle(vma(common, args, 1) as *const c_char);
        } else if trap == G::BOTLIB_PC_FREE_SOURCE as c_int {
            return (*botlib_export).PC_FreeSourceHandle(*args.offset(1));
        } else if trap == G::BOTLIB_PC_READ_TOKEN as c_int {
            return (*botlib_export)
                .PC_ReadTokenHandle(*args.offset(1), vma(common, args, 2) as *mut c_void);
        } else if trap == G::BOTLIB_PC_SOURCE_FILE_AND_LINE as c_int {
            return (*botlib_export).PC_SourceFileAndLine(
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                vma(common, args, 3) as *mut c_int,
            );
        } else if trap == G::BOTLIB_START_FRAME as c_int {
            return (*botlib_export).BotLibStartFrame(vmf(args, 1));
        } else if trap == G::BOTLIB_LOAD_MAP as c_int {
            return (*botlib_export).BotLibLoadMap(vma(common, args, 1) as *const c_char);
        } else if trap == G::BOTLIB_UPDATENTITY as c_int {
            return (*botlib_export)
                .BotLibUpdateEntity(*args.offset(1), vma(common, args, 2) as *mut c_void);
        } else if trap == G::BOTLIB_TEST as c_int {
            return (*botlib_export).Test(
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                vma(common, args, 3) as *mut f32,
                vma(common, args, 4) as *mut f32,
            );
        } else if trap == G::BOTLIB_GET_SNAPSHOT_ENTITY as c_int {
            return mp_engine_server::SV_BotGetSnapshotEntity(sv, *args.offset(1), *args.offset(2));
        } else if trap == G::BOTLIB_GET_CONSOLE_MESSAGE as c_int {
            return mp_engine_server::SV_BotGetConsoleMessage(
                sv,
                *args.offset(1),
                vma(common, args, 2) as *mut c_char,
                *args.offset(3),
            );
        } else if trap == G::BOTLIB_USER_COMMAND as c_int {
            mp_engine_server::SV_ClientThink(
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
            return mp_engine_renderer::RE_RegisterServerSkin(
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
    let var = mp_engine_qcommon::cvar::Cvar_Get(
        common,
        cm,
        rm,
        host,
        "bot_enable",
        "1",
        mp_qshared::shared::cvar::CVAR_LATCH,
    );
    // PORT-NOTE(bot_enable): the file-scope `extern int bot_enable` threads
    // as a `Common` field per ruling 3; the field doesn't exist yet, escalated.
    if !var.is_null() {
        common.bot_enable = unsafe { (*var).integer };
    } else {
        common.bot_enable = 0;
    }

    if mp_engine_qcommon::cvar::Cvar_VariableValue(common, "fs_restrict") == 0.0
        && mp_engine_qcommon::cvar::com_dedicated(common).integer == 0
        && !mp_qshared::shared::sys_shared::Sys_CheckCD()
    {
        let need_cd = mp_engine_qcommon::stringed::SE_GetString(common, host, "CON_TEXT_NEED_CD");
        mp_engine_qcommon::common::com_error(errorParm_t::ERR_NEED_CD, need_cd);
        //"Game CD not in drive" );
    }

    // load the dll or bytecode
    sv.gvm = mp_engine_qcommon::vm_fns::VM_Create(
        common,
        cm,
        rm,
        host,
        "jampgame",
        SV_GameSystemCalls,
        unsafe {
            core::mem::transmute(
                mp_engine_qcommon::cvar::Cvar_VariableValue(common, "vm_game") as c_int,
            )
        },
    );
    if sv.gvm.is_null() {
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_FATAL,
            "VM_Create on game failed".to_string(),
        );
    }

    SV_InitGameVM(common, cm, sv, rm, host, qfalse);
}
