//! `sv_main.cpp` — server main loop helpers: newline expansion, pending
//! server-command replacement, pause check, and master-server resolve throttle.
//!
//! Source: `oracle/codemp/server/sv_main.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::common::common::com_printf;
use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common_fns::Com_Milliseconds;
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::net_chan::{NET_OutOfBandPrint, NET_StringToAdr};
use mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t;
use mp_qshared::shared::swap::BigShort;
use mp_qshared::shared::{qboolean, qfalse, qtrue, MAX_STRING_CHARS};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server_host::{HEARTBEAT_GAME, HEARTBEAT_MSEC, MAX_MASTER_SERVERS, NEW_RESOLVE_DURATION};
use crate::Server;
use mp_engine_qcommon::qcommon::protocol::PORT_MASTER;
use mp_qshared::shared::q_string::{Q_strncmp, Q_strncpyz};

use core::ffi::c_short;
use libc::{sscanf, strlen, strstr};

/// Raven `SV_ExpandNewlines` — expands `\n` in a string to a slash and 'n'
/// (so it can be printed on a single line).
///
/// Source: `oracle/codemp/server/sv_main.cpp:59-76`
pub fn SV_ExpandNewlines(sv: &mut Server, r#in: *mut c_char) -> *mut c_char {
    let string = &mut sv.sv_expand_newlines_string;
    let mut l: usize = 0;

    unsafe {
        let mut p = r#in;
        while *p != 0 && l < string.len() - 3 {
            if *p == b'\n' as c_char {
                string[l] = b'\\' as c_char;
                l += 1;
                string[l] = b'n' as c_char;
                l += 1;
            } else {
                string[l] = *p;
                l += 1;
            }
            p = p.offset(1);
        }
        string[l] = 0;
    }

    string.as_mut_ptr()
}

/// Raven `SV_ReplacePendingServerCommands`.
///
/// Source: `oracle/codemp/server/sv_main.cpp:85-106`
pub fn SV_ReplacePendingServerCommands(client: *mut client_t, cmd: *const c_char) -> c_int {
    unsafe {
        let mut i = (*client).reliableSent + 1;
        while i <= (*client).reliableSequence {
            let index = (i & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;
            //
            if Q_strncmp(
                cmd,
                (*client).reliableCommands[index].as_ptr(),
                strlen(c"cs".as_ptr()) as c_int,
            ) == 0
            {
                let mut csnum1: c_int = 0;
                let mut csnum2: c_int = 0;
                sscanf(cmd, c"cs %i".as_ptr(), &mut csnum1 as *mut c_int);
                sscanf(
                    (*client).reliableCommands[index].as_ptr(),
                    c"cs %i".as_ptr(),
                    &mut csnum2 as *mut c_int,
                );
                if csnum1 == csnum2 {
                    Q_strncpyz(
                        (*client).reliableCommands[index].as_mut_ptr(),
                        cmd,
                        MAX_STRING_CHARS as c_int,
                    );
                    /*
                    if ( client->netchan.remoteAddress.type != NA_BOT ) {
                        Com_Printf( "WARNING: client %i removed double pending config string %i: %s\n", client-svs.clients, csnum1, cmd );
                    }
                    */
                    return qtrue;
                }
            }
            i += 1;
        }
        qfalse
    }
}

/// Raven `SV_AddServerCommand` — the given command will be transmitted to the
/// client, and is guaranteed to not have future snapshot_t executed before it
/// is executed.
///
/// Source: `oracle/codemp/server/sv_main.cpp:116-141`
pub fn SV_AddServerCommand(common: &mut Common, sv: &mut Server, client: *mut client_t, cmd: &str) {

    unsafe {
        // this is very ugly but it's also a waste to for instance send multiple
        // config string updates for the same config string index in one snapshot
        //	if ( SV_ReplacePendingServerCommands( client, cmd ) ) {
        //		return;
        //	}

        (*client).reliableSequence += 1;
        // if we would be losing an old command that hasn't been acknowledged,
        // we must drop the connection
        // we check == instead of >= so a broadcast print added by SV_DropClient()
        // doesn't cause a recursive drop client
        if (*client).reliableSequence - (*client).reliableAcknowledge
            == MAX_RELIABLE_COMMANDS as c_int + 1
        {
            com_printf(common, "===== pending server commands =====\n");
            let mut i = (*client).reliableAcknowledge + 1;
            while i <= (*client).reliableSequence {
                let slot = &(*client).reliableCommands[(i & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize];
                com_printf(
                    common,
                    &format!(
                        "cmd {:5}: {}\n",
                        i,
                        core::ffi::CStr::from_ptr(slot.as_ptr()).to_string_lossy()
                    ),
                );
                i += 1;
            }
            com_printf(common, &format!("cmd {:5}: {}\n", i, cmd));
            crate::SV_DropClient(common, sv, client, c"Server command overflow".as_ptr());
            return;
        }
        let index = ((*client).reliableSequence & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;
        // Q_strncpyz needs a C string; build a nul-terminated copy of `cmd`.
        let buf: Vec<c_char> = cmd
            .bytes()
            .map(|b| b as c_char)
            .chain(core::iter::once(0))
            .collect();
        Q_strncpyz(
            (*client).reliableCommands[index].as_mut_ptr(),
            buf.as_ptr(),
            (*client).reliableCommands[index].len() as c_int,
        );
    }
}

/// Raven `SV_SendServerCommand` — sends a reliable command string to be
/// interpreted by the client game module ("cp", "print", "chat", etc). A NULL
/// client will broadcast to all clients.
///
/// Raven's variadic `vsprintf` into `message` is done by the callers, which
/// pass the already-formatted command text in `fmt`.
///
/// Source: `oracle/codemp/server/sv_main.cpp:153-180`
pub fn SV_SendServerCommand(common: &mut Common, sv: &mut Server, cl: *mut client_t, fmt: &str) {

    if !cl.is_null() {
        SV_AddServerCommand(common, sv, cl, fmt);
        return;
    }

    unsafe {
        // hack to echo broadcast prints to console
        if (*common.com_dedicated).integer != 0 && fmt.as_bytes().starts_with(b"print") {
            let mut message: Vec<c_char> = fmt
                .bytes()
                .map(|b| b as c_char)
                .chain(core::iter::once(0))
                .collect();
            let expanded = SV_ExpandNewlines(sv, message.as_mut_ptr());
            let expanded = core::ffi::CStr::from_ptr(expanded).to_string_lossy();
            com_printf(common, &format!("broadcast: {}\n", expanded));
        }

        // send the data to all relevent clients
        for j in 0..(*common.sv_maxclients).integer {
            let client = sv.svs.clients.offset(j as isize);
            if ((*client).state as c_int) < clientState_t::CS_PRIMED as c_int {
                continue;
            }
            SV_AddServerCommand(common, sv, client, fmt);
        }
    }
}

/// Raven `SV_CheckPaused` — only pause if there is just a single client
/// connected.
///
/// Source: `oracle/codemp/server/sv_main.cpp:759-784`
pub fn SV_CheckPaused(common: &mut Common, sv: &mut Server) -> qboolean {
    unsafe {
        if (*common.cl_paused).integer == 0 {
            return qfalse;
        }

        // only pause if there is just a single client connected
        let mut count = 0;
        for i in 0..(*common.sv_maxclients).integer {
            let cl = sv.svs.clients.offset(i as isize);
            if (*cl).state as c_int >= clientState_t::CS_CONNECTED as c_int
                && (*cl).netchan.remoteAddress.r#type != netadrtype_t::NA_BOT
            {
                count += 1;
            }
        }

        if count > 1 {
            // don't pause
            (*common.sv_paused).integer = 0;
            return qfalse;
        }

        (*common.sv_paused).integer = 1;
        qtrue
    }
}

/// Raven `SV_MasterNeedsResolving` — refresh every so often regardless of if the
/// actual address was modified. -rww
///
/// Source: `oracle/codemp/server/sv_main.cpp:194-207`
pub fn SV_MasterNeedsResolving(sv: &mut Server, server: c_int, time: c_int) -> bool {
    if sv.master_heartbeat[server as usize] > time {
        // time flowed backwards?
        return true;
    }

    if (time - sv.master_heartbeat[server as usize]) > NEW_RESOLVE_DURATION {
        // it's time again
        return true;
    }

    false
}

/// Raven `SV_MasterHeartbeat` — send a message to the masters every few minutes
/// to let it know we are alive, and log information.
///
/// Source: `oracle/codemp/server/sv_main.cpp:222-280`
pub fn SV_MasterHeartbeat(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    sv: &mut Server,
) {
    // "dedicated 1" is for lan play, "dedicated 2" is for inet public play
    if common.com_dedicated.is_null() || unsafe { (*common.com_dedicated).integer } != 2 {
        return; // only dedicated servers send heartbeats
    }

    // if not time yet, don't send anything
    if sv.svs.time < sv.svs.nextHeartbeatTime {
        return;
    }
    sv.svs.nextHeartbeatTime = sv.svs.time + HEARTBEAT_MSEC;

    // we need to use this instead of svs.time since svs.time resets over map
    // changes (or rather every time the game restarts), and we don't really need
    // to resolve every map change
    let time = Com_Milliseconds(common, cm, rm, host);

    // send to group masters
    for i in 0..MAX_MASTER_SERVERS {
        let master = common.sv_master[i];
        if unsafe { *(*master).string } == 0 {
            continue;
        }

        // see if we haven't already resolved the name
        // resolving usually causes hitches on win95, so only
        // do it when needed
        if unsafe { (*master).modified } != qfalse || SV_MasterNeedsResolving(sv, i as c_int, time)
        {
            unsafe {
                (*master).modified = qfalse;
            }

            sv.master_heartbeat[i] = time;

            com_printf(common, &format!("Resolving {}\n", unsafe {
                core::ffi::CStr::from_ptr((*master).string).to_string_lossy()
            }));
            if NET_StringToAdr(unsafe { (*master).string }, &mut sv.master_adr[i]) == qfalse {
                // if the address failed to resolve, clear it
                // so we don't take repeated dns hits
                com_printf(common, &format!("Couldn't resolve address: {}\n", unsafe {
                    core::ffi::CStr::from_ptr((*master).string).to_string_lossy()
                }));
                Cvar_Set(
                    common,
                    cm,
                    rm,
                    host,
                    unsafe { (*master).name },
                    c"".as_ptr(),
                );
                unsafe {
                    (*master).modified = qfalse;
                }
                continue;
            }
            // Raven passes `strstr(":", ...)` with the needle/haystack reversed —
            // preserved verbatim (emergent quirk).
            if unsafe { strstr(c":".as_ptr(), (*master).string) }.is_null() {
                sv.master_adr[i].port = BigShort(PORT_MASTER as c_short) as u16;
            }
            let adr = sv.master_adr[i];
            com_printf(common, &format!(
                "{} resolved to {}.{}.{}.{}:{}\n",
                unsafe { core::ffi::CStr::from_ptr((*master).string).to_string_lossy() },
                adr.ip[0],
                adr.ip[1],
                adr.ip[2],
                adr.ip[3],
                BigShort(adr.port as c_short),
            ));
        }

        com_printf(common, &format!("Sending heartbeat to {}\n", unsafe {
            core::ffi::CStr::from_ptr((*master).string).to_string_lossy()
        }));
        // this command should be changed if the server info / status format
        // ever incompatably changes
        NET_OutOfBandPrint(
            common,
            netsrc_t::NS_SERVER,
            sv.master_adr[i],
            format!("heartbeat {}\n", HEARTBEAT_GAME),
        );
    }
}

/// Raven `SV_MasterShutdown` — informs all masters that this server is going
/// down.
///
/// Source: `oracle/codemp/server/sv_main.cpp:288-299`
pub fn SV_MasterShutdown(
    common: &mut Common,
    cm: &mut CollisionWorld,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    sv: &mut Server,
) {
    // send a hearbeat right now
    sv.svs.nextHeartbeatTime = -9999;
    SV_MasterHeartbeat(common, cm, rm, host, sv);

    // send it again to minimize chance of drops
    sv.svs.nextHeartbeatTime = -9999;
    SV_MasterHeartbeat(common, cm, rm, host, sv);

    // when the master tries to poll the server, it won't respond, so
    // it will be removed from the list
}
