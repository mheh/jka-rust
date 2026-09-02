#![allow(non_snake_case, non_camel_case_types, unused_variables)]
//! `sv_init.cpp` — server (re)initialization: `sv`/`svs` bootstrap,
//! configstring get/set/add, client-count bounding/resizing, cgame-touch,
//! server startup/shutdown-adjacent glue, and `SV_SpawnServer`/`SV_Init`.
//!
//! Source: `oracle/codemp/server/sv_init.cpp`

use core::ffi::{c_char, c_int, CStr};
use core::ptr::addr_of_mut;

use mp_qshared::common::mp::game::g_public::SVF_NOSERVERINFO;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::tags::memtag_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::force_reload::ForceReload_e;
use mp_qshared::shared::game_state::MAX_CONFIGSTRINGS;
use mp_qshared::shared::limits::{MAX_CLIENTS, MAX_INFO_STRING, MAX_NAME_LENGTH, MAX_STRING_CHARS};
use mp_qshared::shared::qboolean;
use native_string::cstr::strncpyz_string;
use native_string::q_string::Q_stricmpBytes;
use native_string::q_strncpyz::Q_strncpyzBytes;
use native_string::Info_ValueForKey;
use native_string::{latin1_to_string, string_to_latin1};

use mp_engine_ghoul2::api_collision::g2api_set_time;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_host_interface::engine_host::EngineHost;

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::sv_bot::{SV_BotFrame, SV_BotInitBotLib, SV_BotInitCvars};
use crate::sv_ccmds::SV_Heartbeat_f;
use crate::sv_game::{SV_GentityNum, SV_InitGameProgs, SV_ShutdownGameProgs_body};
use crate::sv_renderer::{
    RE_RegisterMedia_LevelLoadBegin, R_InitShaders, R_InitSkins, R_SVModelInit,
};
use crate::Server;
use mp_engine_qcommon::vm::VM_Call;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;

// Canonical homes for the qcommon/qshared free functions this file calls
// (sv_game.rs precedent): `Cvar_*`/`Com_Milliseconds`/`FS_FCloseFile` live in
// `mp_engine_qcommon` and take the threaded `(common, cm, rm, host, …)`
// engine-host receivers with `*const c_char` string params; the pointer
// `Q_strncpyz` is the raw-pointer `q_shared.c` primitive in `mp_qshared`
// (kept for the `*mut c_char` scratch buffers below).
use mp_abi::game::exports::MpGameExport;
use mp_engine_qcommon::common_fns::{Com_Memset, Com_Milliseconds};
use mp_engine_qcommon::cvar_fns::{
    Cvar_Get, Cvar_InfoString, Cvar_InfoString_Big, Cvar_Set, Cvar_VariableValue,
};
use mp_engine_qcommon::files_common::{FS_FCloseFile, FS_FOpenFileRead, FS_Restart};
use mp_engine_qcommon::files_pc::{
    FS_ClearPakReferences, FS_LoadedPakChecksums, FS_LoadedPakNames, FS_ReferencedPakChecksums,
    FS_ReferencedPakNames,
};
use mp_engine_qcommon::vm_fns::VM_ExplicitArgPtr;
use mp_engine_qcommon::z_memman_pc::{Hunk_Clear, Hunk_SetMark, Z_Free, Z_Malloc};
use mp_qshared::shared::fileHandle_t;
use mp_qshared::shared::q_string::Q_strncpyz;
use std::ffi::CString;

use crate::sv_ccmds::SV_AddOperatorCommands;
use crate::sv_ccmds::SV_RemoveOperatorCommands;
use crate::sv_client::{SV_DropClient, SV_SendClientMapChange};
use crate::sv_main::{SV_MasterShutdown, SV_SendServerCommand};
use crate::sv_snapshot::SV_SendClientSnapshot;
use crate::sv_world::SV_ClearWorld;
use mp_engine_qcommon::cm_load::{CM_ClearMap, CM_LoadMap};

use mp_engine_botlib::BotLib;

/// Raven `SV_InitSV`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:284-289`
pub fn SV_InitSV(sv: &mut Server) {
    // `memset(&sv, 0, sizeof(sv))` — faithful full-struct clear via raw
    // zero-write. `configstrings` is now an owned `Vec<String>`, so the raw
    // zero cannot run over it (svs-memset precedent, `SV_Shutdown`): take out
    // the old Vec and drop it (frees every stored string — the Raven
    // `Z_Free` loop equivalent), zero the POD remainder, then write a fresh
    // `MAX_CONFIGSTRINGS`-slot empty Vec back over those zeros without dropping
    // the invalid zero header.
    drop(core::mem::take(&mut sv.sv.configstrings));
    unsafe {
        core::ptr::write_bytes(&mut sv.sv as *mut _, 0u8, 1);
        core::ptr::write(
            addr_of_mut!(sv.sv.configstrings),
            vec![String::new(); MAX_CONFIGSTRINGS],
        );
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
    view: &mut EngineHostView,
    sv: &mut Server,
    index: c_int,
    mut val: *const c_char,
) {
    let maxChunkSize: c_int = MAX_STRING_CHARS as c_int - 24;

    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        view.error(
            errorParm_t::ERR_DROP,
            &format!("SV_SetConfigstring: bad index {}\n", index),
        );
    }

    if val.is_null() {
        val = c"".as_ptr();
    }

    unsafe {
        // don't bother broadcasting an update if no change (Raven's
        // `strcmp(val, configstrings[index]) == 0` maps to byte equality of the
        // owned string).
        if CStr::from_ptr(val).to_bytes() == sv.sv.configstrings[index as usize].as_bytes() {
            return;
        }

        // change the string in sv — the `Z_Free`/`CopyString` heap lifecycle is
        // now the `Vec<String>`'s own `Drop` on reassignment. `CopyString` is
        // unbounded, so the full `val` is stored (no `MAX_INFO_STRING` bound).
        // Latin-1-decode the raw game bytes: configstrings reach the wire (the
        // "cs"/"bcs" server commands below), so every byte must survive verbatim.
        sv.sv.configstrings[index as usize] = latin1_to_string(CStr::from_ptr(val).to_bytes());
    }

    // send it to all the clients if we aren't spawning a new server
    if sv.sv.state == serverState_t::SS_GAME || sv.sv.restarting != 0 {
        // send the data to all relevant clients
        unsafe {
            for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
                let client = &mut sv.svs.clients[i as usize] as *mut client_t;
                if (*client).state < clientState_t::CS_PRIMED {
                    continue;
                }
                // do not always send server info to all clients
                if index == mp_bg::public::configstring::CS_SERVERINFO
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
                        Q_strncpyz(buf.as_mut_ptr(), val.offset(sent as isize), maxChunkSize);

                        SV_SendServerCommand(
                            view.common,
                            sv,
                            client,
                            &format!(
                                "{} {} \"{}\"\n",
                                cmd,
                                index,
                                latin1_to_string(CStr::from_ptr(buf.as_ptr()).to_bytes())
                            ),
                        );

                        sent += maxChunkSize - 1;
                        remaining -= maxChunkSize - 1;
                    }
                } else {
                    // standard cs, just send it
                    SV_SendServerCommand(
                        view.common,
                        sv,
                        client,
                        &format!(
                            "cs {} \"{}\"\n",
                            index,
                            latin1_to_string(CStr::from_ptr(val).to_bytes())
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
        // Resolved signature carries no `host`/`common` receiver, so Raven's
        // `Com_Error` becomes a direct `panic!` (ruling 1: already unwind-shaped).
        panic!("SV_GetConfigstring: bufferSize == {}", bufferSize);
    }
    if index < 0 || index >= MAX_CONFIGSTRINGS as c_int {
        panic!("SV_GetConfigstring: bad index {}\n", index);
    }
    unsafe {
        // "" == Raven's null slot: the empty-string branch returns the empty
        // string exactly as the null branch did.
        if sv.sv.configstrings[index as usize].is_empty() {
            *buffer = 0;
            return;
        }
        // Frozen trap seam: one bounded copy of the owned string into the game's
        // `(buffer, bufferSize)`, emitted as LATIN-1 WIRE BYTES (one per char) —
        // `as_bytes()` (UTF-8) would double-width a non-ASCII payload.
        Q_strncpyzBytes(
            core::slice::from_raw_parts_mut(buffer, bufferSize as usize),
            &string_to_latin1(&sv.sv.configstrings[index as usize]),
            bufferSize as usize,
        );
    }
}

/// Raven `SV_GetUserinfo`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:189-197`
pub fn SV_GetUserinfo(
    common: &mut Common,
    sv: &mut Server,
    index: c_int,
    buffer: *mut c_char,
    bufferSize: c_int,
) {
    if bufferSize < 1 {
        // Same `Com_Error` -> `panic!` divergence as SV_GetConfigstring above.
        panic!("SV_GetUserinfo: bufferSize == {}", bufferSize);
    }
    if index < 0 || index >= common.cvar(common.sv_maxclients).integer {
        panic!("SV_GetUserinfo: bad index {}\n", index);
    }
    unsafe {
        let client = &sv.svs.clients[index as usize] as *const client_t;
        // Frozen trap seam: one bounded copy of the owned userinfo string into
        // the game's `(buffer, bufferSize)`, emitted as LATIN-1 WIRE BYTES (one
        // per char) — `as_bytes()` (UTF-8) would double-width a non-ASCII name.
        Q_strncpyzBytes(
            core::slice::from_raw_parts_mut(buffer, bufferSize as usize),
            &string_to_latin1(&(*client).userinfo),
            bufferSize as usize,
        );
    }
}

/// Raven `SV_SetUserinfo`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:168-179`
pub fn SV_SetUserinfo(common: &mut Common, sv: &mut Server, index: c_int, mut val: *const c_char) {
    if index < 0 || index >= common.cvar(common.sv_maxclients).integer {
        mp_engine_qcommon::common::com_error(
            errorParm_t::ERR_DROP,
            format!("SV_SetUserinfo: bad index {}\n", index),
        );
    }

    unsafe {
        if val.is_null() {
            val = c"".as_ptr();
        }

        let client = &mut sv.svs.clients[index as usize] as *mut client_t;
        // Raven `Q_strncpyz(cl->userinfo, val, sizeof(cl->userinfo))` — byte-
        // truncate `val` to MAX_INFO_STRING into the owned userinfo string.
        // Latin-1-decode the raw game bytes so a non-ASCII name survives verbatim
        // to the wire (a lossy decode would corrupt it); one wire byte per char,
        // so byte-truncation to MAX_INFO_STRING-1 is the wire-byte bound.
        let raw = CStr::from_ptr(val).to_bytes();
        (*client).userinfo = latin1_to_string(&raw[..raw.len().min(MAX_INFO_STRING - 1)]);
        // Raven `Q_strncpyz(cl->name, Info_ValueForKey(val,"name"), sizeof(cl->name))`
        // — extract the name (from the full infostring) and byte-truncate to
        // MAX_NAME_LENGTH into the String.
        let name_src = Info_ValueForKey(&latin1_to_string(raw), "name");
        (*client).name = strncpyz_string(name_src.as_bytes(), MAX_NAME_LENGTH);
    }
}

/// Raven `SV_ClearServer`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:365-387`
pub fn SV_ClearServer(common: &mut Common, sv: &mut Server) {
    // Raven's `for (...) if (configstrings[i]) Z_Free(configstrings[i]);` is
    // subsumed by `SV_InitSV` below: the owned `Vec<String>` frees every stored
    // string when `SV_InitSV` takes-and-drops it before the struct zero.

    //	CM_ClearMap();

    // nope, can't do this anymore.. sv contains entitystates with STL in them.
    //	memset (&sv, 0, sizeof(sv));
    SV_InitSV(sv);
    //	Com_Memset (&sv, 0, sizeof(sv));
}

/// Raven `SV_BoundMaxClients`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:234-245`
pub fn SV_BoundMaxClients(view: &mut EngineHostView, sv: &mut Server, minimum: c_int) {
    // get the current maxclients value
    Cvar_Get(view, "sv_maxclients", "8", 0);

    view.common.cvar_mut(view.common.sv_maxclients).modified = false;

    if view.common.cvar(view.common.sv_maxclients).integer < minimum {
        Cvar_Set(view, "sv_maxclients", &format!("{minimum}"));
    } else if view.common.cvar(view.common.sv_maxclients).integer > MAX_CLIENTS as c_int {
        Cvar_Set(view, "sv_maxclients", &format!("{}", MAX_CLIENTS as c_int));
    }
}

/// Raven `SV_TouchCGame`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:396-412`
pub fn SV_TouchCGame(view: &mut EngineHostView) {
    let mut f: fileHandle_t = 0;
    let filename: String = if Cvar_VariableValue(view.common, "vm_cgame") != 0.0 {
        Com_sprintf_vm_qvm("cgame")
    } else {
        "cgamex86.dll".to_string()
    };

    FS_FOpenFileRead(view, &filename, &mut f, false);
    if f != 0 {
        FS_FCloseFile(view.common, f);
    }
}

/// Raven `Com_sprintf(filename, sizeof(filename), "vm/%s.qvm", "cgame")`
/// (`SV_TouchCGame`, `sv_init.cpp:402`) — `filename` is a `MAX_QPATH` buffer.
fn Com_sprintf_vm_qvm(sub: &str) -> String {
    // The only caller passes "cgame" (13 bytes), far under the MAX_QPATH
    // buffer, so the direct format is byte-identical to Raven's sprintf.
    format!("vm/{sub}.qvm")
}

/// Raven `SV_AddConfigstring`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:123-160`
pub fn SV_AddConfigstring(
    view: &mut EngineHostView,
    sv: &mut Server,
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
            // Raven's `!configstrings[i] || !configstrings[i][0]` (null-or-empty)
            // is the owned string's `is_empty()`.
            if sv.sv.configstrings[(start + i) as usize].is_empty() {
                // Didn't find it
                SV_SetConfigstring(view, sv, start + i, name);
                break;
            } else if Q_stricmpBytes(
                sv.sv.configstrings[(start + i) as usize].as_bytes(),
                CStr::from_ptr(name).to_bytes(),
            ) == 0
            {
                return i;
            }
        }
    }

    0
}

/// Raven `SV_Startup`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:258-278`
pub fn SV_Startup(view: &mut EngineHostView, sv: &mut Server) {
    if sv.svs.initialized != 0 {
        view.error(errorParm_t::ERR_FATAL, "SV_Startup: svs.initialized");
    }
    SV_BoundMaxClients(view, sv, 1);

    // Raven `Z_Malloc(sizeof(client_t)*maxclients, TAG_CLIENTS, zeroit)` — the
    // owned `Vec<client_t>` filled with zero-equivalent Defaults.
    let maxclients = view.common.cvar(view.common.sv_maxclients).integer as usize;
    sv.svs.clients = (0..maxclients).map(|_| client_t::default()).collect();
    if view.common.cvar(view.common.com_dedicated).integer != 0 {
        sv.svs.numSnapshotEntities = view.common.cvar(view.common.sv_maxclients).integer
            * mp_engine_qcommon::qcommon::net_limits::PACKET_BACKUP as c_int
            * 64;
        Cvar_Set(view, "r_ghoul2animsmooth", "0");
        Cvar_Set(view, "r_ghoul2unsqashaftersmooth", "0");
    } else {
        // we don't need nearly as many when playing locally
        sv.svs.numSnapshotEntities = view.common.cvar(view.common.sv_maxclients).integer * 4 * 64;
    }
    sv.svs.initialized = qboolean::from(1);

    Cvar_Set(view, "sv_running", "1");
}

/// Raven `SV_ChangeMaxClients`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:299-358`
pub fn SV_ChangeMaxClients(view: &mut EngineHostView, sv: &mut Server) {
    // get the highest client number in use
    let mut count: c_int = 0;
    for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
        if sv.svs.clients[i as usize].state >= clientState_t::CS_CONNECTED && i > count {
            count = i;
        }
    }
    count += 1;

    let oldMaxClients = view.common.cvar(view.common.sv_maxclients).integer;
    // never go below the highest client number in use
    SV_BoundMaxClients(view, sv, count);
    // if still the same
    if view.common.cvar(view.common.sv_maxclients).integer == oldMaxClients {
        return;
    }

    // Raven copies the connected clients (Hunk temp round-trip) into a freshly
    // zeroed `Z_Malloc` block of the new size. Rust shape: build the new
    // `Vec<client_t>` of Defaults, MOVE each old connected slot across
    // (`mem::take` transfers the owned Strings, no clone), then let the old Vec
    // drop (disconnected slots' Strings free with it). `count <= oldMaxClients`
    // and `count <= newMaxClients`, so every moved index is in bounds.
    let new_max = view.common.cvar(view.common.sv_maxclients).integer as usize;
    let mut new_clients: Vec<client_t> = (0..new_max).map(|_| client_t::default()).collect();
    let mut old_clients = core::mem::take(&mut sv.svs.clients);
    for i in 0..count as usize {
        if old_clients[i].state >= clientState_t::CS_CONNECTED {
            new_clients[i] = core::mem::take(&mut old_clients[i]);
        }
    }
    sv.svs.clients = new_clients;
    drop(old_clients);

    // allocate new snapshot entities
    if view.common.cvar(view.common.com_dedicated).integer != 0 {
        sv.svs.numSnapshotEntities = view.common.cvar(view.common.sv_maxclients).integer
            * mp_engine_qcommon::qcommon::net_limits::PACKET_BACKUP as c_int
            * 64;
    } else {
        // we don't need nearly as many when playing locally
        sv.svs.numSnapshotEntities = view.common.cvar(view.common.sv_maxclients).integer * 4 * 64;
    }
}

/// Raven `SV_SendMapChange`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:414-431`
pub fn SV_SendMapChange(view: &mut EngineHostView, sv: &mut Server) {
    if !sv.svs.clients.is_empty() {
        unsafe {
            for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
                let client = &mut sv.svs.clients[i as usize] as *mut client_t;
                if (*client).state >= clientState_t::CS_CONNECTED
                    && (*client).netchan.remoteAddress.r#type != netadrtype_t::NA_BOT
                {
                    SV_SendClientMapChange(view, sv, client);
                }
            }
        }
    }
}

/// Raven `SV_SpawnServer`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:472-791`
pub fn SV_SpawnServer(
    view: &mut EngineHostView,
    sv: &mut Server,
    g2: &mut Ghoul2System,
    server: &str,
    killBots: qboolean,
    eForceReload: ForceReload_e,
) {
    let mut checksum: c_int = 0;
    let mut isBot: qboolean;
    let mut systemInfo = [0 as c_char; 16384];
    let mut p: *const c_char;

    SV_SendMapChange(view, sv);

    RE_RegisterMedia_LevelLoadBegin(view, server, eForceReload);

    // shut down the existing game if it is running
    SV_ShutdownGameProgs_body(view.common, sv);

    com_printf(view.common, "------ Server Initialization ------\n");
    com_printf(view.common, &format!("Server: {server}\n"));

    /*
    Ghoul2 Insert Start
    */
    // de allocate the snapshot entities
    if !sv.svs.snapshotEntities.is_null() {
        // `delete[] svs.snapshotEntities` — owned Vec/Box drop is the
        // idiomatic-eventual shape; faithful transcription keeps the free +
        // null-out here (§D9 manual-alloc precedent).
        Z_Free(view.common, sv.svs.snapshotEntities as *mut _);
        sv.svs.snapshotEntities = core::ptr::null_mut();
    }
    /*
    Ghoul2 Insert End
    */

    SV_SendMapChange(view, sv);

    // if not running a dedicated server CL_MapLoading will connect the client to the server
    // also print some status stuff
    let cl_map_loading = view.common.hooks.CL_MapLoading.expect("CL_MapLoading hook");
    cl_map_loading(view);

    // make sure all the client stuff is unloaded
    // The hook answers with a no-op on `jampded`, which reproduces Raven's `#ifndef DEDICATED` guard at this call site.
    let cl_shutdown_all = view
        .common
        .hooks
        .CL_ShutdownAll
        .expect("CL_ShutdownAll hook");
    cl_shutdown_all(view);

    CM_ClearMap(view.cm, &mut view.rmg);

    // clear the whole hunk because we're (re)loading the server
    Hunk_Clear(view);

    R_InitSkins(view);
    R_InitShaders(view, qboolean::from(1));

    // init client structures and svs.numSnapshotEntities
    if Cvar_VariableValue(view.common, "sv_running") == 0.0 {
        SV_Startup(view, sv);
    } else {
        // check for maxclients change
        if view.common.cvar(view.common.sv_maxclients).modified {
            SV_ChangeMaxClients(view, sv);
        }
    }

    SV_SendMapChange(view, sv);

    /*
    Ghoul2 Insert Start
    */
    // clear out those shaders, images and Models as long as this
    // isnt a dedicated server.
    if view.common.cvar(view.common.com_dedicated).integer != 0 {
        R_SVModelInit(view);
    }

    SV_SendMapChange(view, sv);

    // clear pak references
    FS_ClearPakReferences(view.common, 0);

    /*
    Ghoul2 Insert Start
    */
    // allocate the snapshot entities on the hunk
    sv.svs.nextSnapshotEntities = 0;

    // allocate the snapshot entities
    // Raven's `new entityState_s[svs.numSnapshotEntities]` — the PC operator
    // new is an internal allocator (§A1: internals are free), reproduced here
    // as the `Z_Malloc` the matching `delete[]`/`Z_Free` deallocation pairs to.
    sv.svs.snapshotEntities = Z_Malloc(
        view,
        core::mem::size_of::<entityState_t>() as c_int * sv.svs.numSnapshotEntities,
        memtag_t::TAG_GENERAL,
        qboolean::from(1),
        0,
    ) as *mut entityState_t;
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
    sv.svs.snapFlagServerBit ^= mp_qshared::shared::limits::SNAPFLAG_SERVERCOUNT;

    // set nextmap to the same map, but it may be overriden
    // by the game startup or another console command
    Cvar_Set(view, "nextmap", "map_restart 0");

    // wipe the entire per-level structure
    SV_ClearServer(view.common, sv);
    for i in 0..MAX_CONFIGSTRINGS {
        // Raven `configstrings[i] = CopyString("")` — the empty owned string.
        sv.sv.configstrings[i] = String::new();
    }

    //rww - RAGDOLL_BEGIN
    g2api_set_time(g2, sv.svs.time, 0);
    //rww - RAGDOLL_END

    // make sure we are not paused
    Cvar_Set(view, "cl_paused", "0");

    // get a new checksum feed and restart the file system
    //
    // `srand`/`rand` are the engine's own `QRand` LCG on `common` (ruling 21).
    let seed_ms = Com_Milliseconds(view);
    view.common.qrand.srand(seed_ms as u32);
    sv.sv.checksumFeed = ((view.common.qrand.rand() as c_int) << 16
        ^ view.common.qrand.rand() as c_int)
        ^ Com_Milliseconds(view);
    FS_Restart(view, sv.sv.checksumFeed);

    CM_LoadMap(
        view,
        &format!("maps/{server}.bsp"),
        qboolean::from(0),
        &mut checksum,
    );

    SV_SendMapChange(view, sv);

    // set serverinfo visible name
    Cvar_Set(view, "mapname", server);

    Cvar_Set(view, "sv_mapChecksum", &format!("{checksum}"));

    // serverid should be different each time
    sv.sv.serverId = view.common.com_frameTime;
    sv.sv.restartedServerId = sv.sv.serverId;
    Cvar_Set(view, "sv_serverid", &format!("{}", sv.sv.serverId));

    // clear physics interaction links
    SV_ClearWorld(view.cm, sv);

    // media configstring setting should be done during
    // the loading stage, so connected clients don't have
    // to load during actual gameplay
    sv.sv.state = serverState_t::SS_LOADING;

    // Engine referee: arm record/replay for this map load. Placed just before
    // SV_InitGameProgs so the GAME_INIT randomSeed pin lands on its
    // Com_Milliseconds read.
    crate::sv_referee::ref_spawn_setup(view, sv, server);

    // load and spawn all other entities
    SV_InitGameProgs(view, sv);

    // Engine referee: now that GAME_INIT has run (and the seed is captured),
    // append the tape `H` header (record mode only).
    crate::sv_referee::ref_spawn_write_header(view, sv, server);

    // don't allow a map_restart if game is modified
    view.common.cvar_mut(view.common.sv_gametype).modified = false;

    // run a few frames to allow everything to settle
    for _ in 0..3 {
        //rww - RAGDOLL_BEGIN
        g2api_set_time(g2, sv.svs.time, 0);
        //rww - RAGDOLL_END
        VM_Call(
            view.common,
            sv.gvm,
            MpGameExport::GAME_RUN_FRAME as i32,
            &[sv.svs.time as isize],
        );
        SV_BotFrame(view.common, sv, sv.svs.time);
        sv.svs.time += 100;
    }
    //rww - RAGDOLL_BEGIN
    g2api_set_time(g2, sv.svs.time, 0);
    //rww - RAGDOLL_END

    // create a baseline for more efficient communications
    SV_CreateBaseline(sv);

    unsafe {
        for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
            // send the new gamestate to all connected clients
            let client = &mut sv.svs.clients[i as usize] as *mut client_t;
            if (*client).state >= clientState_t::CS_CONNECTED {
                if (*client).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT {
                    if killBots != 0 {
                        SV_DropClient(view.common, sv, client, "");
                        continue;
                    }
                    isBot = qboolean::from(1);
                } else {
                    isBot = qboolean::from(0);
                }

                // connect the client again
                let connect_ret = VM_Call(
                    view.common,
                    sv.gvm,
                    MpGameExport::GAME_CLIENT_CONNECT as i32,
                    &[i as isize, 0, isBot as isize],
                );
                let denied = VM_ExplicitArgPtr(view.common, sv.gvm, connect_ret) as *mut c_char;
                if !denied.is_null() {
                    // this generally shouldn't happen, because the client
                    // was connected before the level change
                    // (module-memory seam: convert the denial text at the arm)
                    let denied = latin1_to_string(core::ffi::CStr::from_ptr(denied).to_bytes());
                    SV_DropClient(view.common, sv, client, &denied);
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
                        view.common,
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
        view.common,
        sv.gvm,
        MpGameExport::GAME_RUN_FRAME as i32,
        &[sv.svs.time as isize],
    );
    SV_BotFrame(view.common, sv, sv.svs.time);
    sv.svs.time += 100;
    //rww - RAGDOLL_BEGIN
    g2api_set_time(g2, sv.svs.time, 0);
    //rww - RAGDOLL_END

    if view.common.cvar(view.common.sv_pure).integer != 0 {
        // the server sends these to the clients so they will only
        // load pk3s also loaded at the server
        unsafe {
            p = FS_LoadedPakChecksums(view.common);
            let p_s = latin1_to_string(CStr::from_ptr(p).to_bytes());
            Cvar_Set(view, "sv_paks", &p_s);
            if libc::strlen(p) == 0 {
                com_printf(
                    view.common,
                    "WARNING: sv_pure set but no PK3 files loaded\n",
                );
            }
            p = FS_LoadedPakNames(view.common);
            let p_s = latin1_to_string(CStr::from_ptr(p).to_bytes());
            Cvar_Set(view, "sv_pakNames", &p_s);
        }

        // if a dedicated pure server we need to touch the cgame because it could be in a
        // seperate pk3 file and the client will need to load the latest cgame.qvm
        if view.common.cvar(view.common.com_dedicated).integer != 0 {
            SV_TouchCGame(view);
        }
    } else {
        Cvar_Set(view, "sv_paks", "");
        Cvar_Set(view, "sv_pakNames", "");
    }
    // the server sends these to the clients so they can figure
    // out which pk3s should be auto-downloaded
    unsafe {
        p = FS_ReferencedPakChecksums(view.common);
        let p_s = latin1_to_string(CStr::from_ptr(p).to_bytes());
        Cvar_Set(view, "sv_referencedPaks", &p_s);
        p = FS_ReferencedPakNames(view.common);
        let p_s = latin1_to_string(CStr::from_ptr(p).to_bytes());
        Cvar_Set(view, "sv_referencedPakNames", &p_s);
    }

    // save systeminfo and serverinfo strings
    let systeminfo_big =
        Cvar_InfoString_Big(view.common, mp_qshared::shared::cvar::CVAR_SYSTEMINFO);
    let systeminfo_big_c = CString::new(systeminfo_big.as_str()).unwrap_or_default();
    Q_strncpyz(
        systemInfo.as_mut_ptr(),
        systeminfo_big_c.as_ptr(),
        systemInfo.len() as c_int,
    );
    view.common.cvar_modifiedFlags &= !mp_qshared::shared::cvar::CVAR_SYSTEMINFO;
    SV_SetConfigstring(view, sv, mp_bg::public::configstring::CS_SYSTEMINFO, {
        systemInfo.as_ptr()
    });

    let serverinfo = Cvar_InfoString(view.common, mp_qshared::shared::cvar::CVAR_SERVERINFO);
    let serverinfo_c = CString::new(serverinfo.as_str()).unwrap_or_default();
    SV_SetConfigstring(
        view,
        sv,
        mp_bg::public::configstring::CS_SERVERINFO,
        serverinfo_c.as_ptr(),
    );
    view.common.cvar_modifiedFlags &= !mp_qshared::shared::cvar::CVAR_SERVERINFO;

    // any media configstring setting now should issue a warning
    // and any configstring changes should be reliably transmitted
    // to all clients
    sv.sv.state = serverState_t::SS_GAME;

    // send a heartbeat now so the master will get up to date info
    SV_Heartbeat_f(sv);

    Hunk_SetMark(view.common);

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
pub fn SV_Init(view: &mut EngineHostView) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of
    // this slot for the borrow's duration.
    let sv = unsafe { &mut *(view.sv.as_raw() as *mut Server) };
    // SAFETY: view-constructor slot, single-threaded, no other live cast of
    // this slot for the borrow's duration.
    let bot = unsafe { &mut *(view.bot.as_raw() as *mut BotLib) };

    SV_AddOperatorCommands(view, sv);

    // serverinfo vars
    Cvar_Get(
        view,
        "dmflags",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "fraglimit",
        "20",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "timelimit",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "capturelimit",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );

    // Get these to establish them and to make sure they have a default before the menus decide to stomp them.
    Cvar_Get(
        view,
        "g_maxHolocronCarry",
        "3",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "g_privateDuel",
        "1",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "g_saberLocking",
        "1",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "g_maxForceRank",
        "6",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "duel_fraglimit",
        "10",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "g_forceBasedTeams",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "g_duelWeaponDisable",
        "1",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );

    view.common.sv_gametype = Some(Cvar_Get(
        view,
        "g_gametype",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_LATCH,
    ));
    view.common.sv_needpass = Some(Cvar_Get(
        view,
        "g_needpass",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_ROM,
    ));
    Cvar_Get(
        view,
        "sv_keywords",
        "",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    );
    Cvar_Get(
        view,
        "protocol",
        &format!("{}", mp_engine_qcommon::qcommon::protocol::PROTOCOL_VERSION),
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );
    view.common.sv_mapname = Some(Cvar_Get(
        view,
        "mapname",
        "nomap",
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_ROM,
    ));
    view.common.sv_privateClients = Some(Cvar_Get(
        view,
        "sv_privateClients",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    view.common.sv_hostname = Some(Cvar_Get(
        view,
        "sv_hostname",
        "*Jedi*",
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_ARCHIVE,
    ));
    view.common.sv_maxclients = Some(Cvar_Get(
        view,
        "sv_maxclients",
        "8",
        mp_qshared::shared::cvar::CVAR_SERVERINFO | mp_qshared::shared::cvar::CVAR_LATCH,
    ));
    view.common.sv_maxRate = Some(Cvar_Get(
        view,
        "sv_maxRate",
        "0",
        mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    view.common.sv_minPing = Some(Cvar_Get(
        view,
        "sv_minPing",
        "0",
        mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    view.common.sv_maxPing = Some(Cvar_Get(
        view,
        "sv_maxPing",
        "0",
        mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    view.common.sv_floodProtect = Some(Cvar_Get(
        view,
        "sv_floodProtect",
        "1",
        mp_qshared::shared::cvar::CVAR_ARCHIVE | mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    // systeminfo
    Cvar_Get(
        view,
        "sv_cheats",
        "0",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );
    view.common.sv_serverid = Some(Cvar_Get(
        view,
        "sv_serverid",
        "0",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    ));
    // (retail branch kept; `DLL_ONLY`-guarded alternate branch is a dead
    // build config here — §20-class note, not transcribed)
    view.common.sv_pure = Some(Cvar_Get(
        view,
        "sv_pure",
        "1",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO,
    ));
    Cvar_Get(
        view,
        "sv_paks",
        "",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );
    Cvar_Get(
        view,
        "sv_pakNames",
        "",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );
    Cvar_Get(
        view,
        "sv_referencedPaks",
        "",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );
    Cvar_Get(
        view,
        "sv_referencedPakNames",
        "",
        mp_qshared::shared::cvar::CVAR_SYSTEMINFO | mp_qshared::shared::cvar::CVAR_ROM,
    );

    // server vars
    view.common.sv_rconPassword = Some(Cvar_Get(
        view,
        "rconPassword",
        "",
        mp_qshared::shared::cvar::CVAR_TEMP,
    ));
    view.common.sv_privatePassword = Some(Cvar_Get(
        view,
        "sv_privatePassword",
        "",
        mp_qshared::shared::cvar::CVAR_TEMP,
    ));
    view.common.sv_fps = Some(Cvar_Get(
        view,
        "sv_fps",
        "20",
        mp_qshared::shared::cvar::CVAR_TEMP,
    ));
    view.common.sv_timeout = Some(Cvar_Get(
        view,
        "sv_timeout",
        "200",
        mp_qshared::shared::cvar::CVAR_TEMP,
    ));
    view.common.sv_zombietime = Some(Cvar_Get(
        view,
        "sv_zombietime",
        "2",
        mp_qshared::shared::cvar::CVAR_TEMP,
    ));
    Cvar_Get(view, "nextmap", "", mp_qshared::shared::cvar::CVAR_TEMP);

    // (Xbox master/download exclusion branch not taken; retail non-Xbox path kept)
    view.common.sv_allowDownload = Some(Cvar_Get(
        view,
        "sv_allowDownload",
        "0",
        mp_qshared::shared::cvar::CVAR_SERVERINFO,
    ));
    // `MASTER_SERVER_NAME` (`qcommon/protocol.rs`) as a c-string cvar default,
    // matching the inline-literal precedent of the sibling sv_masterN defaults.
    view.common.sv_master[0] = Some(Cvar_Get(view, "sv_master1", "masterjk3.ravensoft.com", 0));
    // Raven defaults this slot empty, and the live community master replaces the dead retail one.
    // The client browser reads the `sv_masterN` cvars too, so this default is what fills it.
    view.common.sv_master[1] = Some(Cvar_Get(
        view,
        "sv_master2",
        "master.jkhub.org",
        mp_qshared::shared::cvar::CVAR_ARCHIVE,
    ));
    view.common.sv_master[2] = Some(Cvar_Get(
        view,
        "sv_master3",
        "",
        mp_qshared::shared::cvar::CVAR_ARCHIVE,
    ));
    view.common.sv_master[3] = Some(Cvar_Get(
        view,
        "sv_master4",
        "",
        mp_qshared::shared::cvar::CVAR_ARCHIVE,
    ));
    view.common.sv_master[4] = Some(Cvar_Get(
        view,
        "sv_master5",
        "",
        mp_qshared::shared::cvar::CVAR_ARCHIVE,
    ));
    view.common.sv_reconnectlimit = Some(Cvar_Get(view, "sv_reconnectlimit", "3", 0));
    view.common.sv_showghoultraces = Some(Cvar_Get(view, "sv_showghoultraces", "0", 0));
    view.common.sv_showloss = Some(Cvar_Get(view, "sv_showloss", "0", 0));
    view.common.sv_padPackets = Some(Cvar_Get(view, "sv_padPackets", "0", 0));
    view.common.sv_killserver = Some(Cvar_Get(view, "sv_killserver", "0", 0));
    view.common.sv_mapChecksum = Some(Cvar_Get(
        view,
        "sv_mapChecksum",
        "",
        mp_qshared::shared::cvar::CVAR_ROM,
    ));

    // Engine referee (sv_referee.rs) cvars: `ref_record <file>` taps the input/
    // state stream, `ref_replay <file>` drives the engine from a tape, and a
    // nonzero `ref_seed` pins the GAME_INIT randomSeed so record and replay
    // agree. Injection strategy is per-slot (tape-created humans inject, bots
    // regenerate) — not a cvar.
    Cvar_Get(view, "ref_record", "", 0);
    Cvar_Get(view, "ref_replay", "", 0);
    // `ref_follow 1` — ref_replay tail-follows a tape still being written
    // (the lockstep secondary's mode; see sv_referee.rs).
    Cvar_Get(view, "ref_follow", "0", 0);
    // `ref_state 1` — record verbose per-frame state bytes (V records) so a
    // follower names the first divergent field.
    Cvar_Get(view, "ref_state", "0", 0);
    // `ref_haltOnDiverge 1` — freeze both engines into step mode on a
    // divergence (0 = log + resync from the tape's V and continue).
    Cvar_Get(view, "ref_haltOnDiverge", "0", 0);
    // `ref_calls <file>` — dump each frame window's ordered syscall imports
    // (for diffing the two engines' exact call sequences).
    Cvar_Get(view, "ref_calls", "", 0);
    Cvar_Get(view, "ref_seed", "0", 0);
    // `ref_snaps <file>` — raw client-bound wire capture (see sv_referee.rs).
    Cvar_Get(view, "ref_snaps", "", 0);

    // initialize bot cvars so they are listed and can be set before loading the botlib
    SV_BotInitCvars(view);

    // init the botlib here because we need the pre-compiler in the UI
    SV_BotInitBotLib(view, sv, bot);

    // Raven allocates `G2VertSpaceServer = new CMiniHeap(...)` here for game-side
    // model vertex transforms (sv_init.cpp:468-469); `CMiniHeap` is deleted per
    // the ghoul2-server design (the collision path threads no scratch heap), so
    // this allocation drops.
}

/// Raven `SV_FinalMessage` — used by `SV_Shutdown` to send a final message to
/// all connected clients before the server goes down. The messages are sent
/// immediately, not just stuck on the outgoing message list, because the server
/// is going to totally exit after returning from this function.
///
/// Source: `oracle/codemp/server/sv_init.cpp:900-918`
pub fn SV_FinalMessage(view: &mut EngineHostView, sv: &mut Server, message: &str) {
    // send it twice, ignoring rate
    for _j in 0..2 {
        let maxclients = view.common.cvar(view.common.sv_maxclients).integer;
        for i in 0..maxclients {
            let cl = &mut sv.svs.clients[i as usize] as *mut client_t;
            if unsafe { (*cl).state } >= clientState_t::CS_CONNECTED {
                // don't send a disconnect to a local client
                if unsafe { (*cl).netchan.remoteAddress.r#type } != netadrtype_t::NA_LOOPBACK {
                    SV_SendServerCommand(view.common, sv, cl, &format!("print \"{}\"", message));
                    SV_SendServerCommand(view.common, sv, cl, "disconnect");
                }
                // force a snapshot to be sent
                unsafe {
                    (*cl).nextSnapshotTime = -1;
                }
                SV_SendClientSnapshot(view, sv, cl);
            }
        }
    }
}

/// Raven `SV_Shutdown` — called when each game quits, before `Sys_Quit` or
/// `Sys_Error`.
///
/// Source: `oracle/codemp/server/sv_init.cpp:929-990`
pub fn SV_Shutdown(view: &mut EngineHostView, finalmsg: &str) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of
    // this slot for the borrow's duration.
    let sv = unsafe { &mut *(view.sv.as_raw() as *mut Server) };

    if view.common.com_sv_running.is_none()
        || view.common.cvar(view.common.com_sv_running).integer == 0
    {
        return;
    }

    // RECORD tap: seal the tape with its `E` end record before teardown.
    crate::sv_referee::ref_tap_shutdown(sv);

    if !sv.svs.clients.is_empty() && !view.common.error.entered {
        SV_FinalMessage(view, sv, finalmsg);
    }

    SV_RemoveOperatorCommands();
    // `#ifndef _XBOX` — no master on Xbox; this build is never Xbox.
    SV_MasterShutdown(view, sv);
    SV_ShutdownGameProgs_body(view.common, sv);

    // de allocate the snapshot entities
    if !sv.svs.snapshotEntities.is_null() {
        Z_Free(view.common, sv.svs.snapshotEntities as *mut _);
        sv.svs.snapshotEntities = core::ptr::null_mut();
    }

    // free current level
    SV_ClearServer(view.common, sv);
    // jfm: add a clear here since it's commented out in clearServer. This
    // prevents crashing cmShaderTable on exit.
    CM_ClearMap(view.cm, &mut view.rmg);

    // free server static data. Raven Z_Frees svs.clients then Com_Memsets all
    // of serverStatic_t. The clients Vec owns its heap, so drop it here (the
    // Z_Free — `mem::take` leaves an empty Vec), then zero the POD remainder.
    // The memset also zeros the (empty) Vec header, so a fresh empty Vec is
    // `ptr::write`n back over those bytes without dropping the invalid zeros.
    drop(core::mem::take(&mut sv.svs.clients));
    let svs_size = core::mem::size_of_val(&sv.svs);
    Com_Memset(&mut sv.svs as *mut _ as *mut (), 0, svs_size);
    unsafe {
        core::ptr::write(addr_of_mut!(sv.svs.clients), Vec::new());
    }

    Cvar_Set(view, "sv_running", "0");
    Cvar_Set(view, "ui_singlePlayerActive", "0");
}
