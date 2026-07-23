//! `sv_main.cpp` — server main loop helpers: newline expansion, pending
//! server-command replacement, pause check, and master-server resolve throttle.
//!
//! Source: `oracle/codemp/server/sv_main.cpp`

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_short, c_uint, c_void};
use std::ffi::{CStr, CString};

use libc::{sscanf, strcpy, strlen, strstr};

use mp_abi::game::exports::MpGameExport;
use mp_bg::public::configstring::{CS_SERVERINFO, CS_SYSTEMINFO};
use mp_bg::public::gametype::{GT_DUEL, GT_POWERDUEL};
use mp_engine_ghoul2::api_collision::g2api_set_time;
use mp_engine_ghoul2::ghoul2_system::Ghoul2System;
use mp_engine_qcommon::cmd_common::{Cbuf_AddText, Cmd_Argc, Cmd_Argv, Cmd_TokenizeString};
use mp_engine_qcommon::cmd_pc::Cmd_ExecuteString;
use mp_engine_qcommon::common::common::{com_printf, info_set_report, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common_fns::{
    Com_BeginRedirect, Com_DPrintf, Com_EndRedirect, Com_Milliseconds,
};
use mp_engine_qcommon::cvar_fns::{
    Cvar_InfoString, Cvar_InfoString_Big, Cvar_Set, Cvar_VariableIntegerValue, Cvar_VariableString,
    Cvar_VariableValue,
};
use mp_engine_qcommon::msg::{
    MSG_BeginReadingOOB, MSG_ReadLong, MSG_ReadShort, MSG_ReadStringLine,
};
use mp_engine_qcommon::net_chan::{
    NET_AdrToString, NET_CompareBaseAdr, NET_OutOfBandPrint, NET_StringToAdr,
};
use mp_engine_qcommon::qcommon::huff::Huff_Decompress;
use mp_engine_qcommon::qcommon::net_limits::{MAX_MSGLEN, MAX_RELIABLE_COMMANDS, PACKET_BACKUP};
use mp_engine_qcommon::qcommon::protocol::{PORT_MASTER, PROTOCOL_VERSION};
use mp_engine_qcommon::sys_net::NET_Sleep;
use mp_engine_qcommon::timing::sys_milliseconds;
use mp_engine_qcommon::vm::VM_Call;
use mp_qshared::common::mp::game::g_public::SVF_BOT;
use mp_qshared::common::mp::playerstate::PERS_SCORE;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t;
use mp_qshared::shared::cvar::{CVAR_SERVERINFO, CVAR_SYSTEMINFO};
use mp_qshared::shared::limits::MAX_INFO_STRING;
use mp_qshared::shared::q_string::{Com_sprintf, Q_strncmp, Q_strncpyz};
use mp_qshared::shared::swap::BigShort;
use mp_qshared::shared::{qboolean, qfalse, qtrue, MAX_STRING_CHARS};
use native_string::cstr::buf_to_string;
use native_string::string_to_latin1;
use native_string::q_string::{Q_strcmp, Q_stricmp};
use native_string::{Info_SetValueForKey, Info_ValueForKey};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server_host::{
    HEARTBEAT_GAME, HEARTBEAT_MSEC, MAX_MASTER_SERVERS, NEW_RESOLVE_DURATION, SV_OUTPUTBUF_LENGTH,
};
use crate::sv_bot::SV_BotFrame;
use crate::sv_client::{
    SV_AuthorizeIpPacket, SV_DirectConnect, SV_DropClient, SV_ExecuteClientMessage, SV_GetChallenge,
};
use crate::sv_game::SV_GameClientNum;
use crate::sv_init::{SV_SetConfigstring, SV_Shutdown};
use crate::sv_net_chan::SV_Netchan_Process;
use crate::sv_snapshot::SV_SendClientMessages;
use crate::Server;

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
                let slot = &(*client).reliableCommands
                    [(i & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize];
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
            SV_DropClient(common, sv, client, "Server command overflow");
            return;
        }
        let index = ((*client).reliableSequence & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize;
        // Q_strncpyz needs a C string; build a nul-terminated copy of `cmd` in
        // LATIN-1 WIRE BYTES (one per char) so a non-ASCII payload is stored in
        // reliableCommands exactly as it goes on the wire — `cmd.bytes()` (UTF-8)
        // would re-expand high chars to two bytes. ASCII is byte-identical.
        let buf: Vec<c_char> = string_to_latin1(cmd)
            .into_iter()
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

    if common.cvar(common.com_dedicated).integer != 0 && fmt.as_bytes().starts_with(b"print") {
        let mut message: Vec<c_char> = fmt
            .bytes()
            .map(|b| b as c_char)
            .chain(core::iter::once(0))
            .collect();
        let expanded = unsafe {
            let expanded = SV_ExpandNewlines(sv, message.as_mut_ptr());
            core::ffi::CStr::from_ptr(expanded)
                .to_string_lossy()
                .into_owned()
        };
        com_printf(common, &format!("broadcast: {}\n", expanded));
    }

    unsafe {
        // send the data to all relevent clients
        for j in 0..common.cvar(common.sv_maxclients).integer {
            let client = &mut sv.svs.clients[j as usize] as *mut client_t;
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
    if common.cvar(common.cl_paused).integer == 0 {
        return qfalse;
    }

    // only pause if there is just a single client connected
    let mut count = 0;
    unsafe {
        for i in 0..common.cvar(common.sv_maxclients).integer {
            let cl = &sv.svs.clients[i as usize] as *const client_t;
            if (*cl).state as c_int >= clientState_t::CS_CONNECTED as c_int
                && (*cl).netchan.remoteAddress.r#type != netadrtype_t::NA_BOT
            {
                count += 1;
            }
        }
    }

    if count > 1 {
        // don't pause
        common.cvar_mut(common.sv_paused).integer = 0;
        return qfalse;
    }

    common.cvar_mut(common.sv_paused).integer = 1;
    qtrue
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
pub fn SV_MasterHeartbeat(view: &mut EngineHostView, sv: &mut Server) {
    // "dedicated 1" is for lan play, "dedicated 2" is for inet public play
    if view.common.com_dedicated.is_none()
        || view.common.cvar(view.common.com_dedicated).integer != 2
    {
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
    let time = Com_Milliseconds(view);

    // send to group masters
    for i in 0..MAX_MASTER_SERVERS {
        let master = view.common.sv_master[i];
        if view.common.cvar(master).string.is_empty() {
            continue;
        }

        // see if we haven't already resolved the name
        // resolving usually causes hitches on win95, so only
        // do it when needed
        if view.common.cvar(master).modified || SV_MasterNeedsResolving(sv, i as c_int, time) {
            view.common.cvar_mut(master).modified = false;

            sv.master_heartbeat[i] = time;

            let master_string = view.common.cvar(master).string.clone();
            com_printf(view.common, &format!("Resolving {}\n", master_string));

            let master_string_c = CString::new(master_string.as_str()).unwrap_or_default();
            if NET_StringToAdr(master_string_c.as_ptr(), &mut sv.master_adr[i]) == qfalse {
                // if the address failed to resolve, clear it
                // so we don't take repeated dns hits
                com_printf(
                    view.common,
                    &format!("Couldn't resolve address: {}\n", master_string),
                );
                let name = view.common.cvar(master).name.clone();
                Cvar_Set(view, &name, "");
                view.common.cvar_mut(master).modified = false;
                continue;
            }
            // Raven passes `strstr(":", ...)` with the needle/haystack reversed —
            // preserved verbatim (emergent quirk).
            if unsafe { strstr(c":".as_ptr(), master_string_c.as_ptr()) }.is_null() {
                sv.master_adr[i].port = BigShort(PORT_MASTER as c_short) as u16;
            }
            let adr = sv.master_adr[i];
            com_printf(
                view.common,
                &format!(
                    "{} resolved to {}.{}.{}.{}:{}\n",
                    master_string,
                    adr.ip[0],
                    adr.ip[1],
                    adr.ip[2],
                    adr.ip[3],
                    BigShort(adr.port as c_short),
                ),
            );
        }

        let master_string = view.common.cvar(master).string.clone();
        com_printf(
            view.common,
            &format!("Sending heartbeat to {}\n", master_string),
        );
        // this command should be changed if the server info / status format
        // ever incompatably changes
        NET_OutOfBandPrint(
            view.common,
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
pub fn SV_MasterShutdown(view: &mut EngineHostView, sv: &mut Server) {
    // send a hearbeat right now
    sv.svs.nextHeartbeatTime = -9999;
    SV_MasterHeartbeat(view, sv);

    // send it again to minimize chance of drops
    sv.svs.nextHeartbeatTime = -9999;
    SV_MasterHeartbeat(view, sv);

    // when the master tries to poll the server, it won't respond, so
    // it will be removed from the list
}

// ==========================================================================
// CONNECTIONLESS COMMANDS
// ==========================================================================

/// Raven `SVC_Status` — responds with all the info that qplug or qspy can see
/// about the server and all connected players. Used for getting detailed
/// information after the simple info query.
///
/// Source: `oracle/codemp/server/sv_main.cpp:320-371`
pub fn SVC_Status(view: &mut EngineHostView, sv: &mut Server, from: netadr_t) {
    let mut player = [0 as c_char; 1024];
    let mut status = [0 as c_char; MAX_MSGLEN];

    let mut infostring = Cvar_InfoString(view.common, CVAR_SERVERINFO);
    unsafe {
        // echo back the parameter to status, so master servers can use it as a
        // challenge to prevent timed spoofed reply packets that add ghost servers
        info_set_report(
            Info_SetValueForKey(&mut infostring, "challenge", Cmd_Argv(view.common, 1)),
            "Info string length exceeded\n",
        );

        // add "demo" to the sv_keywords if restricted
        if Cvar_VariableValue(view.common, "fs_restrict") != 0.0 {
            let mut keywords = [0 as c_char; MAX_INFO_STRING];
            let existing = Info_ValueForKey(&infostring, "sv_keywords");
            // Preserve Raven's `Com_sprintf` MAX_INFO_STRING truncation for the
            // intermediate "demo <existing>" build before the set.
            Com_sprintf(
                keywords.as_mut_ptr(),
                keywords.len() as c_int,
                &format!("demo {}", existing),
            );
            info_set_report(
                Info_SetValueForKey(
                    &mut infostring,
                    "sv_keywords",
                    &buf_to_string(CStr::from_ptr(keywords.as_ptr()).to_bytes()),
                ),
                "Info string length exceeded\n",
            );
        }

        status[0] = 0;
        let mut status_length: usize = 0;

        for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
            let cl = &sv.svs.clients[i as usize] as *const client_t;
            if (*cl).state as c_int >= clientState_t::CS_CONNECTED as c_int {
                let ps = SV_GameClientNum(sv, i);
                Com_sprintf(
                    player.as_mut_ptr(),
                    player.len() as c_int,
                    &format!(
                        "{} {} \"{}\"\n",
                        (*ps).persistant[PERS_SCORE as usize],
                        (*cl).ping,
                        (*cl).name
                    ),
                );
                let player_length = strlen(player.as_ptr());
                if status_length + player_length >= status.len() {
                    break; // can't hold any more
                }
                strcpy(status.as_mut_ptr().add(status_length), player.as_ptr());
                status_length += player_length;
            }
        }

        NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            from,
            format!(
                "statusResponse\n{}\n{}",
                infostring,
                CStr::from_ptr(status.as_ptr()).to_string_lossy()
            ),
        );
    }
}

/// Raven `SVC_Info` — responds with a short info message that should be enough
/// to determine if a user is interested in a server to do a full status.
///
/// Source: `oracle/codemp/server/sv_main.cpp:381-469`
pub fn SVC_Info(view: &mut EngineHostView, sv: &mut Server, from: netadr_t) {
    if Cvar_VariableValue(view.common, "ui_singlePlayerActive") != 0.0 {
        return;
    }

    let mut infostring = String::new();

    let gamedir = Cvar_VariableString(view.common, "fs_game").to_string();

    // don't count privateclients
    let mut count = 0;
    for i in view.common.cvar(view.common.sv_privateClients).integer
        ..view.common.cvar(view.common.sv_maxclients).integer
    {
        if sv.svs.clients[i as usize].state as c_int >= clientState_t::CS_CONNECTED as c_int {
            count += 1;
        }
    }

    // echo back the parameter to status, so servers can use it as a
    // challenge to prevent timed spoofed reply packets that add ghost servers
    info_set_report(
        Info_SetValueForKey(&mut infostring, "challenge", Cmd_Argv(view.common, 1)),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(&mut infostring, "protocol", &format!("{PROTOCOL_VERSION}")),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "hostname",
            &view.common.cvar(view.common.sv_hostname).string,
        ),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "mapname",
            &view.common.cvar(view.common.sv_mapname).string,
        ),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(&mut infostring, "clients", &format!("{count}")),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "sv_maxclients",
            &format!(
                "{}",
                view.common.cvar(view.common.sv_maxclients).integer
                    - view.common.cvar(view.common.sv_privateClients).integer
            ),
        ),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "gametype",
            &format!("{}", view.common.cvar(view.common.sv_gametype).integer),
        ),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "needpass",
            &format!("{}", view.common.cvar(view.common.sv_needpass).integer),
        ),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "truejedi",
            &format!("{}", Cvar_VariableIntegerValue(view.common, "g_jediVmerc")),
        ),
        "Info string length exceeded\n",
    );
    let w_disable = if view.common.cvar(view.common.sv_gametype).integer == GT_DUEL
        || view.common.cvar(view.common.sv_gametype).integer == GT_POWERDUEL
    {
        Cvar_VariableIntegerValue(view.common, "g_duelWeaponDisable")
    } else {
        Cvar_VariableIntegerValue(view.common, "g_weaponDisable")
    };
    info_set_report(
        Info_SetValueForKey(&mut infostring, "wdisable", &format!("{w_disable}")),
        "Info string length exceeded\n",
    );
    info_set_report(
        Info_SetValueForKey(
            &mut infostring,
            "fdisable",
            &format!(
                "{}",
                Cvar_VariableIntegerValue(view.common, "g_forcePowerDisable")
            ),
        ),
        "Info string length exceeded\n",
    );
    //Info_SetValueForKey( infostring, "pure", va("%i", sv_pure->integer) );

    if view.common.cvar(view.common.sv_minPing).integer != 0 {
        info_set_report(
            Info_SetValueForKey(
                &mut infostring,
                "minPing",
                &format!("{}", view.common.cvar(view.common.sv_minPing).integer),
            ),
            "Info string length exceeded\n",
        );
    }
    if view.common.cvar(view.common.sv_maxPing).integer != 0 {
        info_set_report(
            Info_SetValueForKey(
                &mut infostring,
                "maxPing",
                &format!("{}", view.common.cvar(view.common.sv_maxPing).integer),
            ),
            "Info string length exceeded\n",
        );
    }
    if !gamedir.is_empty() {
        info_set_report(
            Info_SetValueForKey(&mut infostring, "game", &gamedir),
            "Info string length exceeded\n",
        );
    }

    NET_OutOfBandPrint(
        view.common,
        netsrc_t::NS_SERVER,
        from,
        format!("infoResponse\n{infostring}"),
    );
}

/// No-op redirect flush. Raven installs `SV_FlushRedirect` as the
/// `Com_BeginRedirect` callback, but the migrated `Common.rd_flush` slot is a
/// bare `extern "C" fn(*mut c_char)` that cannot carry the `&mut Common`/
/// `&mut Server` the real flush needs (redirect-threading is deferred,
/// `common.cpp:54`). `SVC_RemoteCommand` therefore installs this no-op only to
/// satisfy `Com_BeginRedirect`'s non-null-callback requirement, and invokes the
/// real `SV_FlushRedirect` directly after `Com_EndRedirect`.
extern "C" fn sv_redirect_flush_noop(_outputbuf: *mut c_char) {}

/// Raven `SV_FlushRedirect` — flushes the redirect buffer to the rcon sender.
///
/// Source: `oracle/codemp/server/sv_main.cpp:477-479`
pub fn SV_FlushRedirect(common: &mut Common, sv: &mut Server, outputbuf: *mut c_char) {
    NET_OutOfBandPrint(
        common,
        netsrc_t::NS_SERVER,
        sv.svs.redirectAddress,
        format!("print\n{}", unsafe {
            CStr::from_ptr(outputbuf).to_string_lossy()
        }),
    );
}

/// Raven `SVC_RemoteCommand` — an rcon packet arrived from the network. Shift
/// down the remaining args, redirect all printfs.
///
/// Source: `oracle/codemp/server/sv_main.cpp:490-533`
pub fn SVC_RemoteCommand(
    view: &mut EngineHostView,
    sv: &mut Server,
    from: netadr_t,
    msg: *mut msg_t,
) {
    // Raven takes `msg` but never reads it.
    let _ = msg;

    // Raven's `char remaining[1024]` strcat scratch becomes an owned String
    // (§19: the C strcat had no bound check).
    let mut remaining = String::new();
    let mut sv_outputbuf = [0 as c_char; SV_OUTPUTBUF_LENGTH];

    let time = Com_Milliseconds(view) as c_uint;
    if time < sv.svc_remote_command_lasttime + 500 {
        return;
    }
    sv.svc_remote_command_lasttime = time;

    let rconpw = view.common.cvar(view.common.sv_rconPassword).string.clone();

    unsafe {
        let valid: qboolean;
        if rconpw.is_empty() || Q_strcmp(Cmd_Argv(view.common, 1), &rconpw) != 0 {
            valid = qfalse;
            let adr = CStr::from_ptr(NET_AdrToString(view.common, from))
                .to_string_lossy()
                .into_owned();
            let arg = Cmd_Argv(view.common, 2).to_owned();
            Com_DPrintf(view.common, &format!("Bad rcon from {adr}:\n{arg}\n"));
        } else {
            valid = qtrue;
            let adr = CStr::from_ptr(NET_AdrToString(view.common, from))
                .to_string_lossy()
                .into_owned();
            let arg = Cmd_Argv(view.common, 2).to_owned();
            Com_DPrintf(view.common, &format!("Rcon from {adr}:\n{arg}\n"));
        }

        // start redirecting all print outputs to the packet
        sv.svs.redirectAddress = from;
        // The redirect callback can't carry state (see `sv_redirect_flush_noop`);
        // install the no-op so `Com_BeginRedirect` accepts a non-null callback and
        // `Com_Printf` routes into `sv_outputbuf`. `flush_slot` is a stack local
        // that outlives the redirect region below.
        let mut flush_slot: extern "C" fn(*mut c_char) = sv_redirect_flush_noop;
        Com_BeginRedirect(
            view.common,
            sv_outputbuf.as_mut_ptr(),
            SV_OUTPUTBUF_LENGTH as c_int,
            &mut flush_slot as *mut extern "C" fn(*mut c_char) as *mut *mut c_void,
        );

        if rconpw.is_empty() {
            com_printf(view.common, "No rconpassword set.\n");
        } else if valid == qfalse {
            com_printf(view.common, "Bad rconpassword.\n");
        } else {
            remaining.clear();

            let argc = Cmd_Argc(view.common);
            for i in 2..argc {
                remaining.push_str(Cmd_Argv(view.common, i));
                remaining.push(' ');
            }

            Cmd_ExecuteString(view, &remaining);
        }

        Com_EndRedirect(view.common);
        // Deviation: the migrated `rd_flush` slot cannot carry `&mut Common`/
        // `&mut Server`, so `SV_FlushRedirect` is invoked here on the accumulated
        // buffer rather than through the callback (see `sv_redirect_flush_noop`).
        SV_FlushRedirect(view.common, sv, sv_outputbuf.as_mut_ptr());
    }
}

/// Raven `SV_ConnectionlessPacket` — a connectionless packet has four leading
/// 0xff characters to distinguish it from a game channel. Clients that are in
/// the game can still send connectionless packets.
///
/// Source: `oracle/codemp/server/sv_main.cpp:545-584`
pub fn SV_ConnectionlessPacket(
    view: &mut EngineHostView,
    sv: &mut Server,
    from: netadr_t,
    msg: *mut msg_t,
) {
    unsafe {
        MSG_BeginReadingOOB(msg);
        MSG_ReadLong(view.common, msg); // skip the -1 marker

        if Q_strncmp(
            c"connect".as_ptr(),
            (*msg).data.offset(4) as *const c_char,
            7,
        ) == 0
        {
            Huff_Decompress(msg, 12);
        }

        let s = MSG_ReadStringLine(view.common, msg);
        Cmd_TokenizeString(view.common, &s);

        let c = Cmd_Argv(view.common, 0).to_owned();
        let adr = CStr::from_ptr(NET_AdrToString(view.common, from))
            .to_string_lossy()
            .into_owned();
        Com_DPrintf(view.common, &format!("SV packet {adr} : {c}\n"));

        if Q_stricmp(&c, "getstatus") == 0 {
            SVC_Status(view, sv, from);
        } else if Q_stricmp(&c, "getinfo") == 0 {
            SVC_Info(view, sv, from);
        } else if Q_stricmp(&c, "getchallenge") == 0 {
            SV_GetChallenge(view, sv, from);
        } else if Q_stricmp(&c, "connect") == 0 {
            SV_DirectConnect(view, sv, from);
        } else if Q_stricmp(&c, "ipAuthorize") == 0 {
            SV_AuthorizeIpPacket(view, sv, from);
        } else if Q_stricmp(&c, "rcon") == 0 {
            SVC_RemoteCommand(view, sv, from, msg);
        } else if Q_stricmp(&c, "disconnect") == 0 {
            // if a client starts up a local server, we may see some spurious
            // server disconnect messages when their new server sees our final
            // sequenced messages to the old client
        } else {
            Com_DPrintf(
                view.common,
                &format!("bad connectionless packet from {adr}:\n{s}\n"),
            );
        }
    }
}

// ==========================================================================

/// Raven `SV_PacketEvent` — the network-packet upcall (`SV_ReadPackets`).
///
/// Source: `oracle/codemp/server/sv_main.cpp:594-649`
pub fn SV_PacketEvent(view: &mut EngineHostView, from: netadr_t, msg: *mut msg_t) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of this
    // slot for the borrow's duration; the (view, sv) callees below take both and
    // never re-cast view.sv (rule 7). SV_GameCommand precedent.
    let sv = unsafe { &mut *(view.sv.as_raw() as *mut Server) };

    unsafe {
        // check for connectionless packet (0xffffffff) first
        if (*msg).cursize >= 4 && *((*msg).data as *const c_int) == -1 {
            SV_ConnectionlessPacket(view, sv, from, msg);
            return;
        }

        // read the qport out of the message so we can fix up
        // stupid address translating routers
        MSG_BeginReadingOOB(msg);
        MSG_ReadLong(view.common, msg); // sequence number
        let qport = MSG_ReadShort(view.common, msg) & 0xffff;

        // find which client the message is from
        for i in 0..view.common.cvar(view.common.sv_maxclients).integer {
            let cl = &mut sv.svs.clients[i as usize] as *mut client_t;
            if (*cl).state == clientState_t::CS_FREE {
                continue;
            }
            if NET_CompareBaseAdr(view.common, from, (*cl).netchan.remoteAddress) == qfalse {
                continue;
            }
            // it is possible to have multiple clients from a single IP address,
            // so they are differentiated by the qport variable
            if (*cl).netchan.qport as c_int != qport {
                continue;
            }

            // the IP port can't be used to differentiate them, because some
            // address translating routers periodically change UDP port assignments
            if (*cl).netchan.remoteAddress.port != from.port {
                com_printf(view.common, "SV_ReadPackets: fixing up a translated port\n");
                (*cl).netchan.remoteAddress.port = from.port;
            }

            // make sure it is a valid, in sequence packet
            if SV_Netchan_Process(view.common, cl, msg) == qtrue {
                // zombie clients still need to do the Netchan_Process to make sure
                // they don't need to retransmit the final reliable message, but
                // they don't do any other processing
                if (*cl).state != clientState_t::CS_ZOMBIE {
                    (*cl).lastPacketTime = sv.svs.time; // don't timeout
                    SV_ExecuteClientMessage(view, sv, cl, msg);
                }
            }
            return;
        }

        // if we received a sequenced packet from an address we don't reckognize,
        // send an out of band disconnect packet to it
        NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            from,
            "disconnect".to_string(),
        );
    }
}

/// Raven `SV_CalcPings` — updates the `cl->ping` variables.
///
/// Source: `oracle/codemp/server/sv_main.cpp:659-704`
pub fn SV_CalcPings(common: &mut Common, sv: &mut Server) {
    unsafe {
        for i in 0..common.cvar(common.sv_maxclients).integer {
            let cl = &mut sv.svs.clients[i as usize] as *mut client_t;
            if (*cl).state != clientState_t::CS_ACTIVE {
                (*cl).ping = 999;
                continue;
            }
            if (*cl).gentity.is_null() {
                (*cl).ping = 999;
                continue;
            }
            if (*(*cl).gentity).r.svFlags & SVF_BOT != 0 {
                (*cl).ping = 0;
                continue;
            }

            // REPLAY: a tape-created replica has no netchan — its ping is a
            // recorded input (`P` records), not a local computation.
            if let Some(taped) = crate::sv_referee::ref_replica_ping(sv, i) {
                (*cl).ping = taped;
                let ps = SV_GameClientNum(sv, i);
                (*ps).ping = taped;
                continue;
            }

            let mut total = 0;
            let mut count = 0;
            for j in 0..PACKET_BACKUP {
                if (*cl).frames[j].messageAcked <= 0 {
                    continue;
                }
                let delta = (*cl).frames[j].messageAcked - (*cl).frames[j].messageSent;
                count += 1;
                total += delta;
            }
            if count == 0 {
                (*cl).ping = 999;
            } else {
                (*cl).ping = total / count;
                if (*cl).ping > 999 {
                    (*cl).ping = 999;
                }
            }

            // let the game dll know about the ping
            let ps = SV_GameClientNum(sv, i);
            (*ps).ping = (*cl).ping;
            // RECORD tap: ps.ping is digested module memory fed by network
            // reality — tape it as an input (change-dedupe inside).
            crate::sv_referee::ref_tap_ping(sv, i, (*ps).ping);
        }
    }
}

/// Raven `SV_CheckTimeouts` — if a packet has not been received from a client
/// for `timeout->integer` seconds, drop the connection. Server time is used
/// instead of realtime to avoid dropping the local client while debugging.
///
/// Source: `oracle/codemp/server/sv_main.cpp:719-751`
pub fn SV_CheckTimeouts(common: &mut Common, sv: &mut Server) {
    let droppoint = sv.svs.time - 1000 * common.cvar(common.sv_timeout).integer;
    let zombiepoint = sv.svs.time - 1000 * common.cvar(common.sv_zombietime).integer;

    unsafe {
        for i in 0..common.cvar(common.sv_maxclients).integer {
            let cl = &mut sv.svs.clients[i as usize] as *mut client_t;
            // message times may be wrong across a changelevel
            if (*cl).lastPacketTime > sv.svs.time {
                (*cl).lastPacketTime = sv.svs.time;
            }

            if (*cl).state == clientState_t::CS_ZOMBIE && (*cl).lastPacketTime < zombiepoint {
                Com_DPrintf(
                    common,
                    &format!(
                        "Going from CS_ZOMBIE to CS_FREE for {}\n",
                        (*cl).name
                    ),
                );
                (*cl).state = clientState_t::CS_FREE; // can now be reused
                continue;
            }
            if (*cl).state as c_int >= clientState_t::CS_CONNECTED as c_int
                && (*cl).lastPacketTime < droppoint
            {
                // wait several frames so a debugger session doesn't cause a timeout
                (*cl).timeoutCount += 1;
                if (*cl).timeoutCount > 5 {
                    SV_DropClient(common, sv, cl, "timed out");
                    (*cl).state = clientState_t::CS_FREE; // don't bother with zombie state
                }
            } else {
                (*cl).timeoutCount = 0;
            }
        }
    }
}

/// Raven `SV_CheckCvars`.
///
/// Source: `oracle/codemp/server/sv_main.cpp:791-816`
pub fn SV_CheckCvars(view: &mut EngineHostView, sv: &mut Server) {
    let mut changed = qfalse;

    if view.common.cvar(view.common.sv_hostname).modificationCount != sv.sv_check_cvars_last_mod {
        let mut hostname = [0 as c_char; MAX_INFO_STRING];
        sv.sv_check_cvars_last_mod = view.common.cvar(view.common.sv_hostname).modificationCount;

        let sv_hostname_c = CString::new(view.common.cvar(view.common.sv_hostname).string.as_str())
            .unwrap_or_default();
        unsafe {
            strcpy(hostname.as_mut_ptr(), sv_hostname_c.as_ptr());
            let mut ci: usize = 0;
            while hostname[ci] != 0 {
                let ch = hostname[ci];
                if ch == b'\\' as c_char || ch == b';' as c_char || ch == b'"' as c_char {
                    hostname[ci] = b'.' as c_char;
                    changed = qtrue;
                }
                ci += 1;
            }
        }
        if changed != qfalse {
            let hostname_s = unsafe { CStr::from_ptr(hostname.as_ptr()) }
                .to_string_lossy()
                .into_owned();
            Cvar_Set(view, "sv_hostname", &hostname_s);
        }
    }
}

/// Raven `SV_Frame` — player movement occurs as a result of packet events,
/// which happen before `SV_Frame` is called.
///
/// Source: `oracle/codemp/server/sv_main.cpp:826-937`
pub fn SV_Frame(view: &mut EngineHostView, msec: c_int) {
    // SAFETY: view-constructor slot, single-threaded, no other live cast of this
    // slot for the borrow's duration; the (view, sv) callees take both and never
    // re-cast view.sv (rule 7). The SV_Shutdown arms early-return: sv's last use
    // precedes the SV_Shutdown hook call, which sources its own sv (task
    // exception). SV_GameCommand precedent.
    let sv = unsafe { &mut *(view.sv.as_raw() as *mut Server) };

    // the menu kills the server with this cvar
    if view.common.cvar(view.common.sv_killserver).integer != 0 {
        SV_Shutdown(view, "Server was killed.\n");
        Cvar_Set(view, "sv_killserver", "0");
        return;
    }

    if view.common.cvar(view.common.com_sv_running).integer == 0 {
        return;
    }

    // allow pause if only the local client is connected
    if SV_CheckPaused(view.common, sv) == qtrue {
        return;
    }

    // if it isn't time for the next frame, do nothing
    if view.common.cvar(view.common.sv_fps).integer < 1 {
        Cvar_Set(view, "sv_fps", "10");
    }
    let frame_msec = 1000 / view.common.cvar(view.common.sv_fps).integer;

    unsafe {
        // Engine referee: RECORD appends `F <msec>`; REPLAY forces msec from the
        // tape so timeResidual (and thus the game-run cadence and sv.svs.time)
        // evolves identically to the recorded run. On tape end it schedules a
        // quit and returns the input msec inertly. FOLLOW returns None while
        // the live tape's next frame block is incomplete (or after its `E`
        // end record) — skip this SV_Frame call entirely; data-paced retry.
        let Some(msec) = crate::sv_referee::ref_frame_begin(view, sv, msec) else {
            return;
        };

        sv.sv.timeResidual += msec;

        if view.common.cvar(view.common.com_dedicated).integer == 0 {
            SV_BotFrame(view.common, sv, sv.svs.time + sv.sv.timeResidual);
        }

        if view.common.cvar(view.common.com_dedicated).integer != 0
            && sv.sv.timeResidual < frame_msec
            && (view.common.com_timescale.is_none()
                || view.common.cvar(view.common.com_timescale).value >= 1.0)
        {
            // NET_Sleep will give the OS time slices until either get a packet
            // or time enough for a server frame has gone by. Referee replay runs
            // faster than real time — keep the (state-identical) early return but
            // skip the wall-clock sleep.
            if !crate::sv_referee::ref_is_replay(sv) {
                NET_Sleep(view.common, frame_msec - sv.sv.timeResidual);
            }
            return;
        }

        // if time is about to hit the 32nd bit, kick all clients and clear
        // sv.time, rather than checking for negative time wraparound everywhere.
        // 2giga-milliseconds = 23 days, so it won't be too often
        if sv.svs.time > 0x70000000 {
            SV_Shutdown(view, "Restarting server due to time wrapping");
            //Cbuf_AddText( "vstr nextmap\n" );
            Cbuf_AddText(view.common, "map_restart 0\n");
            return;
        }
        // this can happen considerably earlier when lots of clients play and the
        // map doesn't change
        if sv.svs.nextSnapshotEntities >= 0x7FFFFFFE - sv.svs.numSnapshotEntities {
            SV_Shutdown(
                view,
                "Restarting server due to numSnapshotEntities wrapping",
            );
            //Cbuf_AddText( "vstr nextmap\n" );
            Cbuf_AddText(view.common, "map_restart 0\n");
            return;
        }

        if sv.sv.restartTime != 0 && sv.svs.time >= sv.sv.restartTime {
            sv.sv.restartTime = 0;
            Cbuf_AddText(view.common, "map_restart 0\n");
            return;
        }

        // update infostrings if anything has been changed
        if view.common.cvar_modifiedFlags & CVAR_SERVERINFO != 0 {
            let s = Cvar_InfoString(view.common, CVAR_SERVERINFO);
            let s_c = CString::new(s.as_str()).unwrap_or_default();
            SV_SetConfigstring(view, sv, CS_SERVERINFO, s_c.as_ptr());
            view.common.cvar_modifiedFlags &= !CVAR_SERVERINFO;
        }
        if view.common.cvar_modifiedFlags & CVAR_SYSTEMINFO != 0 {
            let s = Cvar_InfoString_Big(view.common, CVAR_SYSTEMINFO);
            let s_c = CString::new(s.as_str()).unwrap_or_default();
            SV_SetConfigstring(view, sv, CS_SYSTEMINFO, s_c.as_ptr());
            view.common.cvar_modifiedFlags &= !CVAR_SYSTEMINFO;
        }

        let _start_time = if view.common.cvar(view.common.com_speeds).integer != 0 {
            sys_milliseconds(view.common)
        } else {
            0 // quite a compiler warning
        };

        // update ping based on the all received frames
        SV_CalcPings(view.common, sv);

        // Engine-referee replay: inject this frame's recorded human events
        // (connects, world entries, usercmds, commands, drops) in tape order
        // BEFORE SV_BotFrame — in the recorded session they arrived via the
        // packet loop ahead of SV_Frame, so their module calls (and RNG draws)
        // must land before the bot brains', or the module's RNG stream shifts.
        // RECORD/OFF: no-op.
        crate::sv_referee::ref_frame_inject(view, sv);

        if view.common.cvar(view.common.com_dedicated).integer != 0 {
            SV_BotFrame(view.common, sv, sv.svs.time);
        }

        // run the game simulation in chunks
        let mut ran_game = false;
        while sv.sv.timeResidual >= frame_msec {
            ran_game = true;
            sv.sv.timeResidual -= frame_msec;
            sv.svs.time += frame_msec;

            // let everything in the world think and move
            VM_Call(
                view.common,
                sv.gvm,
                MpGameExport::GAME_RUN_FRAME as c_int,
                &[sv.svs.time as isize],
            );
        }

        // Engine referee: RECORD appends `S <digest>` for a frame that ran the
        // game; REPLAY compares the digest to the tape and logs/counts diverges.
        crate::sv_referee::ref_frame_end(view, sv, ran_game);

        //rww - RAGDOLL_BEGIN
        let time = sv.svs.time;
        // SAFETY: view-constructor slot, single-threaded, no other live cast
        // of the g2 slot; g2api_set_time never re-enters the view (rule 7).
        let g2 = &mut *(view.g2.as_raw() as *mut Ghoul2System);
        g2api_set_time(g2, time, 0);
        //rww - RAGDOLL_END

        if view.common.cvar(view.common.com_speeds).integer != 0 {
            // Raven: `time_game = Sys_Milliseconds() - startTime;` — `Common` has
            // no `time_game` counter (com_speeds reporting is a silent no-op here,
            // matching the Com_Frame com_speeds precedent), so the elapsed value
            // is dropped.
            let _ = _start_time;
        }

        // check timeouts
        SV_CheckTimeouts(view.common, sv);

        // send messages back to the clients
        SV_SendClientMessages(view, sv);

        SV_CheckCvars(view, sv);

        // send a heartbeat to the master if needed
        SV_MasterHeartbeat(view, sv);
    }
}
