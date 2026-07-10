#![allow(non_snake_case, non_camel_case_types, unused_variables)]
//! `sv_init.cpp` — server (re)initialization: `sv`/`svs` bootstrap,
//! configstring get/set/add, client-count bounding/resizing, cgame-touch,
//! server startup/shutdown-adjacent glue, and `SV_SpawnServer`/`SV_Init`.
//!
//! Source: `oracle/codemp/server/sv_init.cpp`

use core::ffi::{c_char, c_int};

use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::force_reload::ForceReload_e;
use mp_qshared::shared::game_state::MAX_CONFIGSTRINGS;
use mp_qshared::shared::limits::{MAX_CLIENTS, MAX_STRING_CHARS};
use mp_qshared::shared::qboolean;

// PORT-NOTE(engine-host-state): `CollisionWorld`/`Common`/`EngineHost` exist;
// `RenderModels`/`RmManager`/`Ghoul2System`/`BotLib` do not exist anywhere in
// the tree yet (grepped, no hits) — this packet shard was generated ahead of
// those state structs landing (same situation sv_game.rs/sv_ccmds.rs already
// note). Imported below by their preamble-table decl-home crate where one
// exists; genuinely missing types are escalated in missing_symbols rather
// than stubbed (ZERO-PARK), following the sibling files' precedent exactly.
use mp_engine_ghoul2::Ghoul2System;
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::Common;
use mp_engine_renderer::RenderModels;
use mp_engine_rmg::RmManager;
use mp_host_interface::engine_host::EngineHost;

use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::sv_game::SV_GentityNum;
use crate::Server;

// PORT-NOTE(bot-lib-type): the `bot: &mut BotLib` receiver (ruling 2/preamble
// pinned order) has no owning struct anywhere in the tree yet. Declared
// locally (common_fns.rs precedent) so `SV_Init`'s resolved signature can
// transcribe verbatim; escalated in missing_symbols instead of stubbed.
#[allow(dead_code)]
struct BotLib;

// PORT-NOTE(cvar-globals): `sv_maxclients`/`sv_gametype`/`sv_pure`/`sv_*`/
// `com_dedicated`/`gvm`/`cvar_modifiedFlags` are file-scope `cvar_t*`/scalar
// globals (server.h:232-262, qcommon.h:481,690,719) with no `EngineCvars`/
// `Common` home yet (grepped: `Common` has no cvar sub-struct, `Server` has
// no `gvm`/`sv_maxclients` field). Every reference below is written as the
// exact bare Raven identifier as a field access off `common`/`sv`
// (`common.com_dedicated`, `sv.sv_maxclients`, `sv.gvm`,
// `common.cvar_modifiedFlags`) — matching sv_ccmds.rs/sv_game.rs's existing
// identical precedent in this same crate — rather than inventing an
// accessor shim. Escalated in missing_symbols for the finisher to wire once
// `EngineCvars` lands.

/// Raven `SV_InitSV`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:284-289`
pub fn SV_InitSV(sv: &mut Server) {
    // `memset(&sv, 0, sizeof(sv))` — faithful full-struct clear via raw
    // zero-write (the struct carries raw pointers with no `Default` derive).
    unsafe {
        core::ptr::write_bytes(&mut sv.sv as *mut _, 0u8, 1);
    }
    sv.sv.mLocalSubBSPIndex = -1;
}

/// Raven `SV_CreateBaseline`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:209-225`
pub fn SV_CreateBaseline(sv: &mut Server) {
    for entnum in 1..sv.sv.num_entities {
        let svent = SV_GentityNum(sv, entnum);
        unsafe {
            if (*svent).r.linked == 0 {
                continue;
            }
            (*svent).s.number = entnum;

            // take current state as baseline
            sv.sv.svEntities[entnum as usize].baseline = (*svent).s;
        }
    }
}

/// Raven `SV_SetConfigstring`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:25-91`
pub fn SV_SetConfigstring(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    index: c_int,
    mut val: *const c_char,
) {
    let maxChunkSize: c_int = MAX_STRING_CHARS as c_int - 24;

    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        host.error(
            errorParm_t::ERR_DROP,
            &format!("SV_SetConfigstring: bad index {}\n", index),
        );
    }

    if val.is_null() {
        val = c"".as_ptr();
    }

    unsafe {
        // don't bother broadcasting an update if no change
        if libc::strcmp(val, sv.sv.configstrings[index as usize]) == 0 {
            return;
        }

        // change the string in sv
        Z_Free(common, sv.sv.configstrings[index as usize] as *mut _);
        sv.sv.configstrings[index as usize] = CopyString(common, cm, rm, host, val);
    }

    // send it to all the clients if we aren't spawning a new server
    if sv.sv.state == serverState_t::SS_GAME || sv.sv.restarting != 0 {
        // send the data to all relevant clients
        unsafe {
            for i in 0..(*common.sv_maxclients).integer {
                let client = sv.svs.clients.offset(i as isize);
                if (*client).state < clientState_t::CS_PRIMED {
                    continue;
                }
                // do not always send server info to all clients
                if index == CS_SERVERINFO
                    && !(*client).gentity.is_null()
                    && ((*(*client).gentity).r.svFlags & SVF_NOSERVERINFO) != 0
                {
                    continue;
                }

                let len = libc::strlen(val);
                if len >= maxChunkSize as usize {
                    let mut sent: c_int = 0;
                    let mut remaining: c_int = len as c_int;
                    let mut cmd: &str;
                    let mut buf = [0 as c_char; MAX_STRING_CHARS];

                    while remaining > 0 {
                        if sent == 0 {
                            cmd = "bcs0";
                        } else if remaining < maxChunkSize {
                            cmd = "bcs2";
                        } else {
                            cmd = "bcs1";
                        }
                        Q_strncpyz(
                            buf.as_mut_ptr(),
                            val.offset(sent as isize),
                            maxChunkSize as usize,
                        );

                        SV_SendServerCommand(
                            common,
                            sv,
                            client,
                            &format!(
                                "{} {} \"{}\"\n",
                                cmd,
                                index,
                                core::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy()
                            ),
                        );

                        sent += maxChunkSize - 1;
                        remaining -= maxChunkSize - 1;
                    }
                } else {
                    // standard cs, just send it
                    SV_SendServerCommand(
                        common,
                        sv,
                        client,
                        &format!(
                            "cs {} \"{}\"\n",
                            index,
                            core::ffi::CStr::from_ptr(val).to_string_lossy()
                        ),
                    );
                }
            }
        }
    }
}

/// Raven `SV_GetConfigstring`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:101-114`
pub fn SV_GetConfigstring(sv: &mut Server, index: c_int, buffer: *mut c_char, bufferSize: c_int) {
    if bufferSize < 1 {
        // PORT-NOTE(host-unavailable): this fn's resolved signature carries
        // no `host`/`common` receiver (only `sv`), yet Raven calls
        // `Com_Error` here — the resolved signature is LAW, so the panic
        // path can't reach `host.error`. Transcribed as a direct `panic!`
        // carrying the same message (ruling 1: `Com_Error` is already a
        // longjmp/panic-shaped unwind).
        panic!("SV_GetConfigstring: bufferSize == {}", bufferSize);
    }
    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        panic!("SV_GetConfigstring: bad index {}\n", index);
    }
    unsafe {
        if sv.sv.configstrings[index as usize].is_null() {
            *buffer = 0;
            return;
        }
        Q_strncpyz(
            buffer,
            sv.sv.configstrings[index as usize],
            bufferSize as usize,
        );
    }
}

/// Raven `SV_GetUserinfo`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:189-197`
pub fn SV_GetUserinfo(sv: &mut Server, index: c_int, buffer: *mut c_char, bufferSize: c_int) {
    if bufferSize < 1 {
        // PORT-NOTE(host-unavailable): see SV_GetConfigstring above.
        panic!("SV_GetUserinfo: bufferSize == {}", bufferSize);
    }
    unsafe {
        if index < 0 || index >= (*sv.sv_maxclients).integer {
            panic!("SV_GetUserinfo: bad index {}\n", index);
        }
        let client = sv.svs.clients.offset(index as isize);
        Q_strncpyz(buffer, (*client).userinfo.as_ptr(), bufferSize as usize);
    }
}

/// Raven `SV_BoundMaxClients`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:234-245`
pub fn SV_BoundMaxClients(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    minimum: c_int,
) {
    // get the current maxclients value
    Cvar_Get(common, cm, rm, host, "sv_maxclients", "8", 0);

    unsafe {
        (*common.sv_maxclients).modified = qboolean::from(0);

        if (*common.sv_maxclients).integer < minimum {
            Cvar_Set(common, cm, rm, host, "sv_maxclients", &va(minimum));
        } else if (*common.sv_maxclients).integer > MAX_CLIENTS as c_int {
            Cvar_Set(
                common,
                cm,
                rm,
                host,
                "sv_maxclients",
                &va(MAX_CLIENTS as c_int),
            );
        }
    }
}

/// Raven `SV_TouchCGame`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:396-412`
pub fn SV_TouchCGame(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    let filename: String = if Cvar_VariableValue(common, cm, rm, host, "vm_cgame") != 0.0 {
        Com_sprintf_vm_qvm("cgame")
    } else {
        "cgamex86.dll".to_string()
    };

    let f = FS_FOpenFileRead(common, cm, rm, host, &filename, qboolean::from(0));
    if let Some(f) = f {
        FS_FCloseFile(common, f);
    }
}

// PORT-NOTE(com_sprintf-shape): `Com_sprintf(filename, sizeof(filename), "vm/%s.qvm", "cgame")`
// is transcribed as a plain `format!` helper — `Com_sprintf` itself is not
// found in the tree at either qshared or engine tier (missing_symbols); the
// helper keeps the exact literal shape Raven builds.
fn Com_sprintf_vm_qvm(sub: &str) -> String {
    format!("vm/{}.qvm", sub)
}

/// Raven `SV_AddConfigstring`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:123-160`
pub fn SV_AddConfigstring(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    name: *const c_char,
    start: c_int,
    max: c_int,
) -> c_int {
    unsafe {
        if name.is_null() || *name == 0 {
            return 0;
        }

        let mut name = name;
        if *name == b'/' as c_char || *name == b'\\' as c_char {
            // #if _DEBUG: Com_DPrintf( "WARNING: Leading slash on '%s'\n", name);
            name = name.offset(1);

            if *name == 0 {
                return 0;
            }
        }

        for i in 1..max {
            if sv.sv.configstrings[(start + i) as usize].is_null()
                || *sv.sv.configstrings[(start + i) as usize] == 0
            {
                // Didn't find it
                SV_SetConfigstring(common, cm, sv, rm, host, start + i, name);
                break;
            } else if Q_stricmp(sv.sv.configstrings[(start + i) as usize], name) == 0 {
                return i;
            }
        }
    }

    0
}

/// Raven `SV_Startup`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:258-278`
pub fn SV_Startup(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    unsafe {
        if sv.svs.initialized != 0 {
            host.error(errorParm_t::ERR_FATAL, "SV_Startup: svs.initialized");
        }
        SV_BoundMaxClients(common, cm, sv, rm, host, 1);

        sv.svs.clients = Z_Malloc(
            common,
            cm,
            rm,
            host,
            core::mem::size_of::<crate::server::client_s::client_t>()
                * (*common.sv_maxclients).integer as usize,
            memtag_t::TAG_CLIENTS,
            qboolean::from(1),
        ) as *mut _;
        if (*common.com_dedicated).integer != 0 {
            sv.svs.numSnapshotEntities = (*common.sv_maxclients).integer * PACKET_BACKUP * 64;
            Cvar_Set(common, cm, rm, host, "r_ghoul2animsmooth", "0");
            Cvar_Set(common, cm, rm, host, "r_ghoul2unsqashaftersmooth", "0");
        } else {
            // we don't need nearly as many when playing locally
            sv.svs.numSnapshotEntities = (*common.sv_maxclients).integer * 4 * 64;
        }
        sv.svs.initialized = qboolean::from(1);
    }

    Cvar_Set(common, cm, rm, host, "sv_running", "1");
}

/// Raven `SV_ChangeMaxClients`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:299-358`
pub fn SV_ChangeMaxClients(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // get the highest client number in use
    let mut count: c_int = 0;
    unsafe {
        for i in 0..(*common.sv_maxclients).integer {
            if (*sv.svs.clients.offset(i as isize)).state >= clientState_t::CS_CONNECTED
                && i > count
            {
                count = i;
            }
        }
    }
    count += 1;

    let oldMaxClients = unsafe { (*common.sv_maxclients).integer };
    // never go below the highest client number in use
    SV_BoundMaxClients(common, cm, sv, rm, host, count);
    // if still the same
    if unsafe { (*common.sv_maxclients).integer } == oldMaxClients {
        return;
    }

    let oldClients = Hunk_AllocateTempMemory(
        common,
        cm,
        rm,
        host,
        (count as usize) * core::mem::size_of::<crate::server::client_s::client_t>(),
    ) as *mut crate::server::client_s::client_t;
    unsafe {
        // copy the clients to hunk memory
        for i in 0..count {
            if (*sv.svs.clients.offset(i as isize)).state >= clientState_t::CS_CONNECTED {
                *oldClients.offset(i as isize) = *sv.svs.clients.offset(i as isize);
            } else {
                Com_Memset(
                    oldClients.offset(i as isize) as *mut (),
                    0,
                    core::mem::size_of::<crate::server::client_s::client_t>(),
                );
            }
        }

        // free old clients arrays
        Z_Free(common, sv.svs.clients as *mut _);

        // allocate new clients
        sv.svs.clients = Z_Malloc(
            common,
            cm,
            rm,
            host,
            ((*common.sv_maxclients).integer as usize)
                * core::mem::size_of::<crate::server::client_s::client_t>(),
            memtag_t::TAG_CLIENTS,
            qboolean::from(1),
        ) as *mut _;
        Com_Memset(
            sv.svs.clients as *mut (),
            0,
            ((*common.sv_maxclients).integer as usize)
                * core::mem::size_of::<crate::server::client_s::client_t>(),
        );

        // copy the clients over
        for i in 0..count {
            if (*oldClients.offset(i as isize)).state >= clientState_t::CS_CONNECTED {
                *sv.svs.clients.offset(i as isize) = *oldClients.offset(i as isize);
            }
        }
    }

    // free the old clients on the hunk
    Hunk_FreeTempMemory(common, oldClients as *mut _);

    // allocate new snapshot entities
    unsafe {
        if (*common.com_dedicated).integer != 0 {
            sv.svs.numSnapshotEntities = (*common.sv_maxclients).integer * PACKET_BACKUP * 64;
        } else {
            // we don't need nearly as many when playing locally
            sv.svs.numSnapshotEntities = (*common.sv_maxclients).integer * 4 * 64;
        }
    }
}

/// Raven `SV_SendMapChange`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:414-431`
pub fn SV_SendMapChange(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    if !sv.svs.clients.is_null() {
        unsafe {
            for i in 0..(*common.sv_maxclients).integer {
                let client = sv.svs.clients.offset(i as isize);
                if (*client).state >= clientState_t::CS_CONNECTED
                    && (*client).netchan.remoteAddress.r#type != netadrtype_t::NA_BOT
                {
                    SV_SendClientMapChange(common, cm, sv, rm, host, client);
                }
            }
        }
    }
}

/// Raven `SV_SpawnServer`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:472-791`
pub fn SV_SpawnServer(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    rmg: &mut RmManager,
    g2: &mut Ghoul2System,
    host: &mut dyn EngineHost,
    server: *mut c_char,
    killBots: qboolean,
    eForceReload: ForceReload_e,
) {
    let mut checksum: c_int = 0;
    let isBot: qboolean;
    let mut systemInfo = [0 as c_char; 16384];
    let mut p: *const c_char;

    SV_SendMapChange(common, cm, sv, rm, host);

    RE_RegisterMedia_LevelLoadBegin(rm, host, server, eForceReload);

    // shut down the existing game if it is running
    SV_ShutdownGameProgs(common, sv);

    Com_Printf(common, "------ Server Initialization ------\n");
    unsafe {
        Com_Printf(
            common,
            &format!(
                "Server: {}\n",
                core::ffi::CStr::from_ptr(server).to_string_lossy()
            ),
        );
    }

    /*
    Ghoul2 Insert Start
    */
    // de allocate the snapshot entities
    if !sv.svs.snapshotEntities.is_null() {
        // `delete[] svs.snapshotEntities` — owned Vec/Box drop is the
        // idiomatic-eventual shape; faithful transcription keeps the free +
        // null-out here (§D9 manual-alloc precedent).
        Z_Free(common, sv.svs.snapshotEntities as *mut _);
        sv.svs.snapshotEntities = core::ptr::null_mut();
    }
    /*
    Ghoul2 Insert End
    */

    SV_SendMapChange(common, cm, sv, rm, host);

    // if not running a dedicated server CL_MapLoading will connect the client to the server
    // also print some status stuff
    CL_MapLoading();

    CM_ClearMap(cm, rmg);

    // clear the whole hunk because we're (re)loading the server
    Hunk_Clear(common, sv, rm, g2, host);

    R_InitSkins(rm, host);
    R_InitShaders(rm, host, qboolean::from(1));

    // init client structures and svs.numSnapshotEntities
    if Cvar_VariableValue(common, cm, rm, host, "sv_running") == 0.0 {
        SV_Startup(common, cm, sv, rm, host);
    } else {
        // check for maxclients change
        unsafe {
            if (*common.sv_maxclients).modified != 0 {
                SV_ChangeMaxClients(common, cm, sv, rm, host);
            }
        }
    }

    SV_SendMapChange(common, cm, sv, rm, host);

    /*
    Ghoul2 Insert Start
    */
    // clear out those shaders, images and Models as long as this
    // isnt a dedicated server.
    unsafe {
        if (*common.com_dedicated).integer != 0 {
            R_SVModelInit(rm, host);
        }
    }

    SV_SendMapChange(common, cm, sv, rm, host);

    // clear pak references
    FS_ClearPakReferences(common, 0);

    /*
    Ghoul2 Insert Start
    */
    // allocate the snapshot entities on the hunk
    sv.svs.nextSnapshotEntities = 0;

    // allocate the snapshot entities
    // PORT-NOTE(entity-state-array): `new entityState_s[svs.numSnapshotEntities]`
    // — `New_EntityStateArray` is not a real ported symbol; escalated
    // (missing_symbols) rather than inventing an ad hoc allocator here.
    sv.svs.snapshotEntities = New_EntityStateArray(sv.svs.numSnapshotEntities as usize);
    // we CAN afford to do this here, since we know the STL vectors in Ghoul2 are empty
    unsafe {
        core::ptr::write_bytes(
            sv.svs.snapshotEntities,
            0u8,
            sv.svs.numSnapshotEntities as usize,
        );
    }
    /*
    Ghoul2 Insert End
    */

    // toggle the server bit so clients can detect that a
    // server has changed
    sv.svs.snapFlagServerBit ^= SNAPFLAG_SERVERCOUNT;

    // set nextmap to the same map, but it may be overriden
    // by the game startup or another console command
    Cvar_Set(common, cm, rm, host, "nextmap", "map_restart 0");

    // wipe the entire per-level structure
    SV_ClearServer(common, sv);
    for i in 0..MAX_CONFIGSTRINGS {
        sv.sv.configstrings[i] = CopyString(common, cm, rm, host, c"".as_ptr());
    }

    //rww - RAGDOLL_BEGIN
    G2API_SetTime(g2, host, sv.svs.time, 0);
    //rww - RAGDOLL_END

    // make sure we are not paused
    Cvar_Set(common, cm, rm, host, "cl_paused", "0");

    // get a new checksum feed and restart the file system
    //
    // PORT-NOTE(engine-lcg): `srand`/`rand` route through the engine's own
    // `QRand` LCG field on `common` (ruling 21); the field is unnamed until
    // `QRand` itself lands — referenced as `common.qrand` verbatim per the
    // preamble's naming convention, escalated in missing_symbols.
    common
        .qrand
        .srand(Com_Milliseconds(common, cm, rm, host) as u32);
    sv.sv.checksumFeed = ((common.qrand.rand() as c_int) << 16 ^ common.qrand.rand() as c_int)
        ^ Com_Milliseconds(common, cm, rm, host);
    FS_Restart(common, cm, rm, host, sv.sv.checksumFeed);

    unsafe {
        let map_va = format!(
            "maps/{}.bsp",
            core::ffi::CStr::from_ptr(server).to_string_lossy()
        );
        CM_LoadMap(
            common,
            cm,
            rm,
            rmg,
            host,
            &map_va,
            qboolean::from(0),
            &mut checksum,
        );
    }

    SV_SendMapChange(common, cm, sv, rm, host);

    // set serverinfo visible name
    unsafe {
        Cvar_Set(
            common,
            cm,
            rm,
            host,
            "mapname",
            &core::ffi::CStr::from_ptr(server).to_string_lossy(),
        );
    }

    Cvar_Set(common, cm, rm, host, "sv_mapChecksum", &va(checksum));

    // serverid should be different each time
    sv.sv.serverId = common.frame_time;
    sv.sv.restartedServerId = sv.sv.serverId;
    Cvar_Set(common, cm, rm, host, "sv_serverid", &va(sv.sv.serverId));

    // clear physics interaction links
    SV_ClearWorld(cm, sv);

    // media configstring setting should be done during
    // the loading stage, so connected clients don't have
    // to load during actual gameplay
    sv.sv.state = serverState_t::SS_LOADING;

    // load and spawn all other entities
    SV_InitGameProgs(common, cm, sv, rm, host);

    // don't allow a map_restart if game is modified
    unsafe {
        (*common.sv_gametype).modified = qboolean::from(0);
    }

    // run a few frames to allow everything to settle
    for _ in 0..3 {
        //rww - RAGDOLL_BEGIN
        G2API_SetTime(g2, host, sv.svs.time, 0);
        //rww - RAGDOLL_END
        VM_Call(
            common,
            sv.gvm,
            MpGameExport::GAME_RUN_FRAME as i32,
            &[sv.svs.time as isize],
        );
        SV_BotFrame(common, sv, sv.svs.time);
        sv.svs.time += 100;
    }
    //rww - RAGDOLL_BEGIN
    G2API_SetTime(g2, host, sv.svs.time, 0);
    //rww - RAGDOLL_END

    // create a baseline for more efficient communications
    SV_CreateBaseline(sv);

    unsafe {
        for i in 0..(*common.sv_maxclients).integer {
            // send the new gamestate to all connected clients
            let client = sv.svs.clients.offset(i as isize);
            if (*client).state >= clientState_t::CS_CONNECTED {
                if (*client).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT {
                    if killBots != 0 {
                        SV_DropClient(common, sv, client, "");
                        continue;
                    }
                    isBot = qboolean::from(1);
                } else {
                    isBot = qboolean::from(0);
                }

                // connect the client again
                let denied = VM_ExplicitArgPtr(
                    common,
                    sv.gvm,
                    VM_Call(
                        common,
                        sv.gvm,
                        MpGameExport::GAME_CLIENT_CONNECT as i32,
                        &[i as isize, 0, isBot as isize],
                    ),
                ) as *mut c_char;
                if !denied.is_null() {
                    // this generally shouldn't happen, because the client
                    // was connected before the level change
                    SV_DropClient(
                        common,
                        sv,
                        client,
                        &core::ffi::CStr::from_ptr(denied).to_string_lossy(),
                    );
                } else if isBot == 0 {
                    // when we get the next packet from a connected client,
                    // the new gamestate will be sent
                    (*client).state = clientState_t::CS_CONNECTED;
                } else {
                    (*client).state = clientState_t::CS_ACTIVE;
                    let ent = SV_GentityNum(sv, i);
                    (*ent).s.number = i;
                    (*client).gentity = ent;

                    (*client).deltaMessage = -1;
                    (*client).nextSnapshotTime = sv.svs.time; // generate a snapshot immediately

                    VM_Call(
                        common,
                        sv.gvm,
                        MpGameExport::GAME_CLIENT_BEGIN as i32,
                        &[i as isize],
                    );
                }
            }
        }
    }

    // run another frame to allow things to look at all the players
    VM_Call(
        common,
        sv.gvm,
        MpGameExport::GAME_RUN_FRAME as i32,
        &[sv.svs.time as isize],
    );
    SV_BotFrame(common, sv, sv.svs.time);
    sv.svs.time += 100;
    //rww - RAGDOLL_BEGIN
    G2API_SetTime(g2, host, sv.svs.time, 0);
    //rww - RAGDOLL_END

    unsafe {
        if (*common.sv_pure).integer != 0 {
            // the server sends these to the clients so they will only
            // load pk3s also loaded at the server
            p = FS_LoadedPakChecksums(common);
            Cvar_Set(
                common,
                cm,
                rm,
                host,
                "sv_paks",
                &core::ffi::CStr::from_ptr(p).to_string_lossy(),
            );
            if libc::strlen(p) == 0 {
                Com_Printf(common, "WARNING: sv_pure set but no PK3 files loaded\n");
            }
            p = FS_LoadedPakNames(common);
            Cvar_Set(
                common,
                cm,
                rm,
                host,
                "sv_pakNames",
                &core::ffi::CStr::from_ptr(p).to_string_lossy(),
            );

            // if a dedicated pure server we need to touch the cgame because it could be in a
            // seperate pk3 file and the client will need to load the latest cgame.qvm
            if (*common.com_dedicated).integer != 0 {
                SV_TouchCGame(common, cm, rm, host);
            }
        } else {
            Cvar_Set(common, cm, rm, host, "sv_paks", "");
            Cvar_Set(common, cm, rm, host, "sv_pakNames", "");
        }
    }
    // the server sends these to the clients so they can figure
    // out which pk3s should be auto-downloaded
    p = FS_ReferencedPakChecksums(common);
    unsafe {
        Cvar_Set(
            common,
            cm,
            rm,
            host,
            "sv_referencedPaks",
            &core::ffi::CStr::from_ptr(p).to_string_lossy(),
        );
    }
    p = FS_ReferencedPakNames(common);
    unsafe {
        Cvar_Set(
            common,
            cm,
            rm,
            host,
            "sv_referencedPakNames",
            &core::ffi::CStr::from_ptr(p).to_string_lossy(),
        );
    }

    // save systeminfo and serverinfo strings
    unsafe {
        Q_strncpyz(
            systemInfo.as_mut_ptr(),
            Cvar_InfoString_Big(common, CVAR_SYSTEMINFO),
            systemInfo.len(),
        );
    }
    common.cvar_modifiedFlags &= !CVAR_SYSTEMINFO;
    SV_SetConfigstring(common, cm, sv, rm, host, CS_SYSTEMINFO, unsafe {
        systemInfo.as_ptr()
    });

    SV_SetConfigstring(
        common,
        cm,
        sv,
        rm,
        host,
        CS_SERVERINFO,
        Cvar_InfoString(common, CVAR_SERVERINFO),
    );
    common.cvar_modifiedFlags &= !CVAR_SERVERINFO;

    // any media configstring setting now should issue a warning
    // and any configstring changes should be reliably transmitted
    // to all clients
    sv.sv.state = serverState_t::SS_GAME;

    // send a heartbeat now so the master will get up to date info
    SV_Heartbeat_f(sv);

    Hunk_SetMark(common);

    /* MrE: 2000-09-13: now called in CL_DownloadsComplete
    // don't call when running dedicated
    if ( !com_dedicated->integer ) {
        // note that this is called after setting the hunk mark with Hunk_SetMark
        CL_StartHunkUsers();
    }
    */
}

/// Raven `SV_Init`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:803-886`
pub fn SV_Init(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    bot: &mut BotLib,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    SV_AddOperatorCommands(common, cm, sv, rm, host);

    // serverinfo vars
    Cvar_Get(common, cm, rm, host, "dmflags", "0", CVAR_SERVERINFO);
    Cvar_Get(common, cm, rm, host, "fraglimit", "20", CVAR_SERVERINFO);
    Cvar_Get(common, cm, rm, host, "timelimit", "0", CVAR_SERVERINFO);
    Cvar_Get(common, cm, rm, host, "capturelimit", "0", CVAR_SERVERINFO);

    // Get these to establish them and to make sure they have a default before the menus decide to stomp them.
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "g_maxHolocronCarry",
        "3",
        CVAR_SERVERINFO,
    );
    Cvar_Get(common, cm, rm, host, "g_privateDuel", "1", CVAR_SERVERINFO);
    Cvar_Get(common, cm, rm, host, "g_saberLocking", "1", CVAR_SERVERINFO);
    Cvar_Get(common, cm, rm, host, "g_maxForceRank", "6", CVAR_SERVERINFO);
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "duel_fraglimit",
        "10",
        CVAR_SERVERINFO,
    );
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "g_forceBasedTeams",
        "0",
        CVAR_SERVERINFO,
    );
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "g_duelWeaponDisable",
        "1",
        CVAR_SERVERINFO,
    );

    common.sv_gametype = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "g_gametype",
        "0",
        CVAR_SERVERINFO | CVAR_LATCH,
    );
    common.sv_needpass = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "g_needpass",
        "0",
        CVAR_SERVERINFO | CVAR_ROM,
    );
    Cvar_Get(common, cm, rm, host, "sv_keywords", "", CVAR_SERVERINFO);
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "protocol",
        &va(PROTOCOL_VERSION),
        CVAR_SERVERINFO | CVAR_ROM,
    );
    common.sv_mapname = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "mapname",
        "nomap",
        CVAR_SERVERINFO | CVAR_ROM,
    );
    common.sv_privateClients = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_privateClients",
        "0",
        CVAR_SERVERINFO,
    );
    common.sv_hostname = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_hostname",
        "*Jedi*",
        CVAR_SERVERINFO | CVAR_ARCHIVE,
    );
    common.sv_maxclients = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_maxclients",
        "8",
        CVAR_SERVERINFO | CVAR_LATCH,
    );
    common.sv_maxRate = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_maxRate",
        "0",
        CVAR_ARCHIVE | CVAR_SERVERINFO,
    );
    common.sv_minPing = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_minPing",
        "0",
        CVAR_ARCHIVE | CVAR_SERVERINFO,
    );
    common.sv_maxPing = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_maxPing",
        "0",
        CVAR_ARCHIVE | CVAR_SERVERINFO,
    );
    common.sv_floodProtect = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_floodProtect",
        "1",
        CVAR_ARCHIVE | CVAR_SERVERINFO,
    );
    // systeminfo
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_cheats",
        "0",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );
    common.sv_serverid = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_serverid",
        "0",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );
    // (retail branch kept; `DLL_ONLY`-guarded alternate branch is a dead
    // build config here — §20-class note, not transcribed)
    common.sv_pure = Cvar_Get(common, cm, rm, host, "sv_pure", "1", CVAR_SYSTEMINFO);
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_paks",
        "",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_pakNames",
        "",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_referencedPaks",
        "",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );
    Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_referencedPakNames",
        "",
        CVAR_SYSTEMINFO | CVAR_ROM,
    );

    // server vars
    common.sv_rconPassword = Cvar_Get(common, cm, rm, host, "rconPassword", "", CVAR_TEMP);
    common.sv_privatePassword = Cvar_Get(common, cm, rm, host, "sv_privatePassword", "", CVAR_TEMP);
    common.sv_fps = Cvar_Get(common, cm, rm, host, "sv_fps", "20", CVAR_TEMP);
    common.sv_timeout = Cvar_Get(common, cm, rm, host, "sv_timeout", "200", CVAR_TEMP);
    common.sv_zombietime = Cvar_Get(common, cm, rm, host, "sv_zombietime", "2", CVAR_TEMP);
    Cvar_Get(common, cm, rm, host, "nextmap", "", CVAR_TEMP);

    // (Xbox master/download exclusion branch not taken; retail non-Xbox path kept)
    common.sv_allowDownload = Cvar_Get(
        common,
        cm,
        rm,
        host,
        "sv_allowDownload",
        "0",
        CVAR_SERVERINFO,
    );
    common.sv_master[0] = Cvar_Get(common, cm, rm, host, "sv_master1", MASTER_SERVER_NAME, 0);
    common.sv_master[1] = Cvar_Get(common, cm, rm, host, "sv_master2", "", CVAR_ARCHIVE);
    common.sv_master[2] = Cvar_Get(common, cm, rm, host, "sv_master3", "", CVAR_ARCHIVE);
    common.sv_master[3] = Cvar_Get(common, cm, rm, host, "sv_master4", "", CVAR_ARCHIVE);
    common.sv_master[4] = Cvar_Get(common, cm, rm, host, "sv_master5", "", CVAR_ARCHIVE);
    common.sv_reconnectlimit = Cvar_Get(common, cm, rm, host, "sv_reconnectlimit", "3", 0);
    common.sv_showghoultraces = Cvar_Get(common, cm, rm, host, "sv_showghoultraces", "0", 0);
    common.sv_showloss = Cvar_Get(common, cm, rm, host, "sv_showloss", "0", 0);
    common.sv_padPackets = Cvar_Get(common, cm, rm, host, "sv_padPackets", "0", 0);
    common.sv_killserver = Cvar_Get(common, cm, rm, host, "sv_killserver", "0", 0);
    common.sv_mapChecksum = Cvar_Get(common, cm, rm, host, "sv_mapChecksum", "", CVAR_ROM);

    // initialize bot cvars so they are listed and can be set before loading the botlib
    SV_BotInitCvars(common, cm, rm, host);

    // init the botlib here because we need the pre-compiler in the UI
    SV_BotInitBotLib(common, cm, sv, bot, rm, host);

    // Only allocated once, no point in moving it around and fragmenting
    // create a heap for Ghoul2 to use for game side model vertex transforms used in collision detection
    //
    // PORT-NOTE(g2-vert-space): `G2VertSpaceServer`/`CMiniHeap_singleton`
    // (file-scope statics, sv_init.cpp:468-469) have no ported home yet;
    // referenced by their exact Raven identifiers, escalated in
    // missing_symbols for the finisher.
    common.G2VertSpaceServer = &mut common.CMiniHeap_singleton;
}
