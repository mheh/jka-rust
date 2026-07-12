//! `sv_client.cpp` — the server's per-client connection lifecycle: challenge
//! handshake, direct connect, gamestate/download delivery, and the inbound
//! client-message/usercmd dispatch chain.
//!
//! Source: `oracle/codemp/server/sv_client.cpp`
//!
//! PORT-NOTE(qcommon-surface): this worktree's `mp_engine_qcommon` has not yet
//! landed `cmd`/`msg`/`net`/`net_chan`/`vm`/`cvar` free-function bodies (only
//! their type/const homes exist today) — every such callee below is called at
//! its packet-resolved Raven name through the module path the "one module per
//! oracle source file" convention implies (`cmd.cpp` -> `qcommon::cmd`,
//! `msg.cpp` -> `qcommon::msg`, `net_chan.cpp` -> `qcommon::net_chan`,
//! `net.cpp`/`net_ip.cpp` -> `qcommon::net`), matching the sibling
//! `sv_game.rs`'s established guessed-path convention for the same
//! not-yet-landed surface. All are escalated in `missing_symbols`.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, CStr};

use mp_engine_qcommon::common::common::Common;
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::cvar_fns::Cvar_VariableValue;
use mp_engine_qcommon::qcommon::net_limits::{MAX_DOWNLOAD_BLKSIZE, MAX_DOWNLOAD_WINDOW};
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::game::g_public::SVF_BOT;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::netsrc_t::netsrc_t;
use mp_qshared::common::mp::qcommon::usercmd::usercmd_t;
use mp_qshared::shared::error_parm::errorParm_t;
use mp_qshared::shared::{qboolean, qfalse, qtrue};

use crate::server::challenge_t::challenge_t;
use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::server::server_static_t::MAX_CHALLENGES;
use crate::Server;

use libc::{atoi, strcmp, strlen};
use mp_qshared::shared::q_string::{
    Com_sprintf, Info_SetValueForKey, Info_ValueForKey, Q_stricmp, Q_strncpyz,
};

/// Raven `SV_ResetPureClient_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1440-1442`
pub fn SV_ResetPureClient_f(cl: *mut client_t) {
    unsafe {
        (*cl).pureAuthentic = 0;
    }
}

/// Raven `SV_UserinfoChanged`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1452-1500`
pub fn SV_UserinfoChanged(view: &mut EngineHostView, cl: *mut client_t) {
    unsafe {
        let name = Info_ValueForKey((*cl).userinfo.as_mut_ptr(), c"name".as_ptr() as *mut c_char);
        Q_strncpyz((*cl).name.as_mut_ptr(), name, (*cl).name.len() as c_int);

        // if the client is on the same subnet as the server and we aren't running an
        // internet public server, assume they don't need a rate choke
        if view.is_lan_address(&(*cl).netchan.remoteAddress)
            && (*view.common.com_dedicated).integer != 2
        {
            // lans should not rate limit
            (*cl).rate = 99999;
        } else {
            let val =
                Info_ValueForKey((*cl).userinfo.as_mut_ptr(), c"rate".as_ptr() as *mut c_char);
            if strlen(val) != 0 {
                let i = atoi(val);
                (*cl).rate = i;
                if (*cl).rate < 1000 {
                    (*cl).rate = 1000;
                } else if (*cl).rate > 90000 {
                    (*cl).rate = 90000;
                }
            } else {
                (*cl).rate = 3000;
            }
        }

        let val = Info_ValueForKey(
            (*cl).userinfo.as_mut_ptr(),
            c"handicap".as_ptr() as *mut c_char,
        );
        if strlen(val) != 0 {
            let i = atoi(val);
            if i <= 0 || i > 100 || strlen(val) > 4 {
                Info_SetValueForKey(
                    (*cl).userinfo.as_mut_ptr(),
                    c"handicap".as_ptr() as *mut c_char,
                    c"100".as_ptr() as *mut c_char,
                );
            }
        }

        // snaps command
        let val = Info_ValueForKey(
            (*cl).userinfo.as_mut_ptr(),
            c"snaps".as_ptr() as *mut c_char,
        );
        if strlen(val) != 0 {
            let mut i = atoi(val);
            if i < 1 {
                i = 1;
            } else if i > 30 {
                i = 30;
            }
            (*cl).snapshotMsec = 1000 / i;
        } else {
            (*cl).snapshotMsec = 50;
        }
    }
}

/// Raven `SV_GetChallenge`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:31-130`
pub fn SV_GetChallenge(view: &mut EngineHostView, sv: &mut Server, from: netadr_t) {
    if Cvar_VariableValue(view, c"ui_singlePlayerActive".as_ptr() as *const c_char) != 0.0 {
        return;
    }

    let mut oldest: usize = 0;
    let mut oldest_time: c_int = 0x7fffffff;

    // see if we already have a challenge for this ip
    let mut i: usize = 0;
    while i < MAX_CHALLENGES {
        let challenge = &mut sv.svs.challenges[i];
        if challenge.connected == qfalse
            && mp_engine_qcommon::net_chan::NET_CompareAdr(view.common, from, challenge.adr)
                == qtrue
        {
            break;
        }
        if challenge.time < oldest_time {
            oldest_time = challenge.time;
            oldest = i;
        }
        i += 1;
    }

    if i == MAX_CHALLENGES {
        // this is the first time this client has asked for a challenge
        let challenge = &mut sv.svs.challenges[oldest];

        // PORT-NOTE(qrand-field, ruling 21): `common`'s `QRand` field name is
        // pinned when the `QRand` type lands; `common.qrand` is a placeholder
        // reference, escalated in `missing_symbols`.
        challenge.challenge = ((view.common.qrand.irand(0, 0x7fff) << 16)
            ^ view.common.qrand.irand(0, 0x7fff))
            ^ sv.svs.time;
        challenge.adr = from;
        challenge.firstTime = sv.svs.time;
        challenge.time = sv.svs.time;
        challenge.connected = qfalse;
        i = oldest;
    }

    let challenge = &mut sv.svs.challenges[i];

    // if they are on a lan address, send the challengeResponse immediately
    if view.is_lan_address(&from) {
        challenge.pingTime = sv.svs.time;
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            from,
            format!("challengeResponse {}\n", challenge.challenge),
        );
        return;
    }

    // USE_CD_KEY is not defined in this build (WinDed release config,
    // porting-rules FINAL_BUILD precedent) — the `#else` branch is the live
    // path.
    challenge.pingTime = sv.svs.time;
    mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
        view.common,
        netsrc_t::NS_SERVER,
        challenge.adr,
        format!("challengeResponse {}\n", challenge.challenge),
    );
}

/// Raven `SV_WriteRMGAutomapSymbols`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:668-684`
pub fn SV_WriteRMGAutomapSymbols(view: &mut EngineHostView, msg: *mut msg_t) {
    // SAFETY: view-constructor slot; `view.rmg` casts back to the real
    // `mp_engine_rmg::RmManager` at the server boundary (opaque-slot ruling).
    // Held disjoint from `view.common` used for the MSG writes.
    let rmg = unsafe { crate::server_host::rmg_from_slot(&mut view.rmg) };
    let count = rmg.automap_symbol_count();

    mp_engine_qcommon::msg::MSG_WriteShort(view.common, msg, count);

    for i in 0..count {
        if let Some(symbol) = rmg.automap_symbol(i) {
            mp_engine_qcommon::msg::MSG_WriteByte(view.common, msg, symbol.mType as c_int);
            mp_engine_qcommon::msg::MSG_WriteByte(view.common, msg, symbol.mSide as c_int);
            mp_engine_qcommon::msg::MSG_WriteLong(view.common, msg, symbol.mOrigin[0] as c_int);
            mp_engine_qcommon::msg::MSG_WriteLong(view.common, msg, symbol.mOrigin[1] as c_int);
        }
    }
}

/// Raven `SV_SendClientMapChange`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:820-842`
pub fn SV_SendClientMapChange(view: &mut EngineHostView, sv: &mut Server, client: *mut client_t) {
    let mut msg_buffer = [0u8; mp_engine_qcommon::qcommon::net_limits::MAX_MSGLEN as usize];
    let mut msg: msg_t = unsafe { core::mem::zeroed() };
    mp_engine_qcommon::msg::MSG_Init(
        view,
        &mut msg,
        msg_buffer.as_mut_ptr(),
        msg_buffer.len() as c_int,
    );

    // NOTE, MRE: all server->client messages now acknowledge
    // let the client know which reliable clientCommands we have received
    unsafe {
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, (*client).lastClientCommand);
    }

    // send any server commands waiting to be sent first.
    // we have to do this cause we send the client->reliableSequence
    // with a gamestate and it sets the clc.serverCommandSequence at
    // the client side
    crate::SV_UpdateServerCommandsToClient(view.common, client, &mut msg);

    // send the gamestate
    mp_engine_qcommon::msg::MSG_WriteByte(
        view.common,
        &mut msg,
        mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_mapchange as c_int,
    );

    // deliver this to the client
    crate::SV_SendMessageToClient(view, sv, &mut msg, client);
}

/// Raven `SV_ClientEnterWorld`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:943-970`
pub fn SV_ClientEnterWorld(
    common: &mut Common,
    sv: &mut Server,
    client: *mut client_t,
    cmd: *mut usercmd_t,
) {
    unsafe {
        mp_engine_qcommon::common::common::com_printf(
            common,
            &format!(
                "Going from CS_PRIMED to CS_ACTIVE for {}\n",
                CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
            ),
        );
        (*client).state = clientState_t::CS_ACTIVE;

        // set up the entity for the client
        let client_num = ((client as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        let ent = crate::sv_game::SV_GentityNum(sv, client_num);
        (*ent).s.number = client_num;
        (*client).gentity = ent;

        (*client).lastUserInfoChange = 0; // reset the delay
        (*client).lastUserInfoCount = 0; // reset the count

        (*client).deltaMessage = -1;
        (*client).nextSnapshotTime = sv.svs.time; // generate a snapshot immediately
        (*client).lastUsercmd = *cmd;

        // call the game begin function
        mp_engine_qcommon::vm::VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_BEGIN as c_int,
            &[client_num],
        );
    }
}

/// Raven `SV_StopDownload_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1015-1020`
pub fn SV_StopDownload_f(common: &mut Common, sv: &mut Server, cl: *mut client_t) {
    unsafe {
        if *(*cl).downloadName.as_ptr() != 0 {
            let client_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
                / core::mem::size_of::<client_t>() as isize) as c_int;
            mp_engine_qcommon::common::common::com_printf(
                common,
                &format!(
                    "clientDownload: {} : file \"{}\" aborted\n",
                    client_num,
                    CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                ),
            );
        }
        crate::SV_CloseDownload(common, cl);
    }
}

/// Raven `SV_NextDownload_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1043-1065`
pub fn SV_NextDownload_f(common: &mut Common, sv: &mut Server, cl: *mut client_t) {
    unsafe {
        let block = atoi(mp_engine_qcommon::cmd_common::Cmd_Argv(common, 1));

        let client_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;

        if block == (*cl).downloadClientBlock {
            mp_engine_qcommon::common_fns::Com_DPrintf(
                common,
                &format!(
                    "clientDownload: {} : client acknowledge of block {}\n",
                    client_num, block
                ),
            );

            // Find out if we are done.  A zero-length block indicates EOF
            if (*cl).downloadBlockSize[((*cl).downloadClientBlock as usize)
                % mp_engine_qcommon::qcommon::net_limits::MAX_DOWNLOAD_WINDOW as usize]
                == 0
            {
                mp_engine_qcommon::common::common::com_printf(
                    common,
                    &format!(
                        "clientDownload: {} : file \"{}\" completed\n",
                        client_num,
                        CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                    ),
                );
                crate::SV_CloseDownload(common, cl);
                return;
            }

            (*cl).downloadSendTime = sv.svs.time;
            (*cl).downloadClientBlock += 1;
            return;
        }
        // We aren't getting an acknowledge for the correct block, drop the client
        // FIXME: this is bad... the client will never parse the disconnect message
        //			because the cgame isn't loaded yet
        crate::SV_DropClient(common, sv, cl, c"broken download".as_ptr() as *const c_char);
    }
}

/// Raven `SV_BeginDownload_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1072-1080`
pub fn SV_BeginDownload_f(common: &mut Common, cl: *mut client_t) {
    // Kill any existing download
    crate::SV_CloseDownload(common, cl);

    // cl->downloadName is non-zero now, SV_WriteDownloadToClient will see this and open
    // the file itself
    unsafe {
        Q_strncpyz(
            (*cl).downloadName.as_mut_ptr(),
            mp_engine_qcommon::cmd_common::Cmd_Argv(common, 1),
            (*cl).downloadName.len() as c_int,
        );
    }
}

/// Raven `SV_Disconnect_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1265-1268`
pub fn SV_Disconnect_f(common: &mut Common, sv: &mut Server, cl: *mut client_t) {
    // SV_DropClient( cl, "disconnected" );
    let msg = crate::sv_ccmds::SV_GetStringEdString(
        sv,
        c"MP_SVGAME".as_ptr() as *mut c_char,
        c"DISCONNECTED".as_ptr() as *mut c_char,
    );
    crate::SV_DropClient(common, sv, cl, msg);
}

/// Raven `SV_UpdateUserinfo_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1510-1535`
pub fn SV_UpdateUserinfo_f(view: &mut EngineHostView, sv: &mut Server, cl: *mut client_t) {
    unsafe {
        Q_strncpyz(
            (*cl).userinfo.as_mut_ptr(),
            mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 1),
            (*cl).userinfo.len() as c_int,
        );

        // FINAL_BUILD is not defined in this build (porting-rules FINAL_BUILD
        // precedent) — only the unconditional else-branch is live.
        (*cl).lastUserInfoCount = 0;
        (*cl).lastUserInfoChange =
            sv.svs.time + crate::server::sv_client_userinfo::INFO_CHANGE_MIN_INTERVAL;

        SV_UserinfoChanged(view, cl);
        // call prog code to allow overrides
        let client_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        // Real `&mut Server` in scope — reach the game VM directly (`sv.gvm`)
        // rather than through the sv-touching `vm_call` view method (rule 7).
        mp_engine_qcommon::vm::VM_Call(
            view.common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_USERINFO_CHANGED as c_int,
            &[client_num],
        );
    }
}

/// Raven `ucmd_t` — the console-command dispatch table entry.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1536-1540`
#[allow(non_camel_case_types)]
pub struct ucmd_t {
    pub name: &'static str,
}

/// Raven `SV_ExecuteClientCommand`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1564-1583`
///
/// PORT-NOTE(ucmds-table, shape_mismatch): the resolved signature carries only
/// `common`/`sv` receivers, but two of Raven's `ucmds[]` targets
/// (`SV_VerifyPaks_f`, `SV_DoneDownload_f`) need `cm`/`rm`/`rmg`/`host` too —
/// the packet's signature is LAW (porting-rules §C), so those receivers are
/// referenced here even though they are not bound in this fn's scope
/// (integration-time shape conflict, reported in `shape_mismatches`).
pub fn SV_ExecuteClientCommand(
    view: &mut EngineHostView,
    sv: &mut Server,
    cl: *mut client_t,
    s: *const c_char,
    clientOK: qboolean,
) {
    mp_engine_qcommon::cmd_common::Cmd_TokenizeString(view.common, s);

    // see if it is a server level command
    let name = unsafe {
        core::ffi::CStr::from_ptr(mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 0))
    }
    .to_string_lossy();
    let mut matched = false;
    match name.as_ref() {
        "userinfo" => {
            SV_UpdateUserinfo_f(view, sv, cl);
            matched = true;
        }
        "disconnect" => {
            SV_Disconnect_f(view.common, sv, cl);
            matched = true;
        }
        "cp" => {
            SV_VerifyPaks_f(view, sv, cl);
            matched = true;
        }
        "vdr" => {
            SV_ResetPureClient_f(cl);
            matched = true;
        }
        "download" => {
            SV_BeginDownload_f(view.common, cl);
            matched = true;
        }
        "nextdl" => {
            SV_NextDownload_f(view.common, sv, cl);
            matched = true;
        }
        "stopdl" => {
            SV_StopDownload_f(view.common, sv, cl);
            matched = true;
        }
        "donedl" => {
            SV_DoneDownload_f(view, sv, cl);
            matched = true;
        }
        _ => {}
    }

    if clientOK == qtrue {
        // pass unknown strings to the game
        if !matched && sv.sv.state == serverState_t::SS_GAME {
            let client_num = unsafe {
                ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
                    / core::mem::size_of::<client_t>() as isize) as c_int
            };
            // Real `&mut Server` in scope — reach the game VM directly (rule 7).
            mp_engine_qcommon::vm::VM_Call(
                view.common,
                sv.gvm,
                mp_abi::game::exports::MpGameExport::GAME_CLIENT_COMMAND as c_int,
                &[client_num],
            );
        }
    }
}

/// Raven `SV_ClientThink`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1652-1660`
pub fn SV_ClientThink(
    common: &mut Common,
    sv: &mut Server,
    cl: *mut client_t,
    cmd: *mut usercmd_t,
) {
    unsafe {
        (*cl).lastUsercmd = *cmd;

        if (*cl).state != clientState_t::CS_ACTIVE {
            // may have been kicked during the last usercmd
            return;
        }

        let client_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        mp_engine_qcommon::vm::VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_THINK as c_int,
            &[client_num],
        );
    }
}

/// Raven `SV_AuthorizeIpPacket`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:142-211`
pub fn SV_AuthorizeIpPacket(view: &mut EngineHostView, sv: &mut Server, from: netadr_t) {
    if mp_engine_qcommon::net_chan::NET_CompareBaseAdr(view.common, from, sv.svs.authorizeAddress)
        == qfalse
    {
        mp_engine_qcommon::common::common::com_printf(
            view.common,
            "SV_AuthorizeIpPacket: not from authorize server\n",
        );
        return;
    }

    let challenge = unsafe { atoi(mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 1)) };

    let mut i: usize = 0;
    while i < MAX_CHALLENGES {
        if sv.svs.challenges[i].challenge == challenge {
            break;
        }
        i += 1;
    }
    if i == MAX_CHALLENGES {
        mp_engine_qcommon::common::common::com_printf(
            view.common,
            "SV_AuthorizeIpPacket: challenge not found\n",
        );
        return;
    }

    // send a packet back to the original client
    sv.svs.challenges[i].pingTime = sv.svs.time;
    let s = unsafe {
        core::ffi::CStr::from_ptr(mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 2))
    }
    .to_string_lossy()
    .into_owned();
    let r = mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 3); // reason

    if {
        Q_stricmp(
            s.as_ptr() as *const c_char,
            c"demo".as_ptr() as *const c_char,
        )
    } == 0
    {
        if Cvar_VariableValue(view, c"fs_restrict".as_ptr() as *const c_char) != 0.0 {
            // a demo client connecting to a demo server
            mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                view.common,
                netsrc_t::NS_SERVER,
                sv.svs.challenges[i].adr,
                format!("challengeResponse {}", sv.svs.challenges[i].challenge),
            );
            return;
        }
        // they are a demo client trying to connect to a real server
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            sv.svs.challenges[i].adr,
            "print\nServer is not a demo server\n".to_string(),
        );
        // clear the challenge record so it won't timeout and let them through
        sv.svs.challenges[i] = unsafe { core::mem::zeroed::<challenge_t>() };
        return;
    }
    if {
        Q_stricmp(
            s.as_ptr() as *const c_char,
            c"accept".as_ptr() as *const c_char,
        )
    } == 0
    {
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            sv.svs.challenges[i].adr,
            format!("challengeResponse {}", sv.svs.challenges[i].challenge),
        );
        return;
    }
    if {
        Q_stricmp(
            s.as_ptr() as *const c_char,
            c"unknown".as_ptr() as *const c_char,
        )
    } == 0
    {
        if r.is_null() {
            mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                view.common,
                netsrc_t::NS_SERVER,
                sv.svs.challenges[i].adr,
                "print\nAwaiting CD key authorization\n".to_string(),
            );
        } else {
            let r_str = unsafe { core::ffi::CStr::from_ptr(r) }.to_string_lossy();
            mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                view.common,
                netsrc_t::NS_SERVER,
                sv.svs.challenges[i].adr,
                format!("print\n{}\n", r_str),
            );
        }
        // clear the challenge record so it won't timeout and let them through
        sv.svs.challenges[i] = unsafe { core::mem::zeroed::<challenge_t>() };
        return;
    }

    // authorization failed
    if r.is_null() {
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            sv.svs.challenges[i].adr,
            "print\nSomeone is using this CD Key\n".to_string(),
        );
    } else {
        let r_str = unsafe { core::ffi::CStr::from_ptr(r) }.to_string_lossy();
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            sv.svs.challenges[i].adr,
            format!("print\n{}\n", r_str),
        );
    }

    // clear the challenge record so it won't timeout and let them through
    sv.svs.challenges[i] = unsafe { core::mem::zeroed::<challenge_t>() };
}

/// Raven `SV_DirectConnect`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:221-568`
///
/// PORT-NOTE(xbox-dead, §20-style drop): the `#ifdef _XBOX` block
/// (sv_client.cpp:408-500 — XboxOnlineInfo player-list sync) is dead under the
/// dedicated/non-XBOX build this workspace targets; not transcribed, matching
/// the codebase's existing convention of dropping platform-ifdef'd dead code.
pub fn SV_DirectConnect(view: &mut EngineHostView, sv: &mut Server, from: netadr_t) {
    unsafe {
        mp_engine_qcommon::common::common::com_printf(view.common, "SVC_DirectConnect ()\n");

        let userinfo_ptr = mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, 1);
        let mut userinfo = [0 as c_char; mp_qshared::shared::limits::MAX_INFO_STRING as usize];
        Q_strncpyz(userinfo.as_mut_ptr(), userinfo_ptr, userinfo.len() as c_int);

        let version = atoi(Info_ValueForKey(
            userinfo.as_mut_ptr(),
            c"protocol".as_ptr() as *mut c_char,
        ));
        if version != mp_engine_qcommon::qcommon::protocol::PROTOCOL_VERSION {
            mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                view.common,
                netsrc_t::NS_SERVER,
                from,
                format!(
                    "print\nServer uses protocol version {}.\n",
                    mp_engine_qcommon::qcommon::protocol::PROTOCOL_VERSION
                ),
            );
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!("    rejected connect from version {}\n", version),
            );
            return;
        }

        let challenge = atoi(Info_ValueForKey(
            userinfo.as_mut_ptr(),
            c"challenge".as_ptr() as *mut c_char,
        ));
        let qport = atoi(Info_ValueForKey(
            userinfo.as_mut_ptr(),
            c"qport".as_ptr() as *mut c_char,
        ));

        let max_clients = (*view.common.sv_maxclients).integer;

        // quick reject
        let mut reconnect_cl: *mut client_t = core::ptr::null_mut();
        {
            let mut i: c_int = 0;
            while i < max_clients {
                let cl = sv.svs.clients.offset(i as isize);
                if mp_engine_qcommon::net_chan::NET_CompareBaseAdr(
                    view.common,
                    from,
                    (*cl).netchan.remoteAddress,
                ) == qtrue
                    && ((*cl).netchan.qport == qport
                        || from.port == (*cl).netchan.remoteAddress.port)
                {
                    if (sv.svs.time - (*cl).lastConnectTime)
                        < ((*view.common.sv_reconnectlimit).integer * 1000)
                    {
                        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                            view.common,
                            netsrc_t::NS_SERVER,
                            from,
                            "print\nReconnect rejected : too soon\n".to_string(),
                        );
                        let adr = mp_engine_qcommon::net_chan::NET_AdrToString(view.common, from);
                        mp_engine_qcommon::common::common::com_printf(
                            view.common,
                            &format!(
                                "{}:reconnect rejected : too soon\n",
                                core::ffi::CStr::from_ptr(adr).to_string_lossy()
                            ),
                        );
                        return;
                    }
                    break;
                }
                i += 1;
            }
        }

        // see if the challenge is valid (LAN clients don't need to challenge)
        if mp_engine_qcommon::net_chan::NET_IsLocalAddress(from) == qfalse {
            let mut i: usize = 0;
            while i < MAX_CHALLENGES {
                if mp_engine_qcommon::net_chan::NET_CompareAdr(
                    view.common,
                    from,
                    sv.svs.challenges[i].adr,
                ) == qtrue
                {
                    if challenge == sv.svs.challenges[i].challenge {
                        break; // good
                    }
                }
                i += 1;
            }
            if i == MAX_CHALLENGES {
                mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                    view.common,
                    netsrc_t::NS_SERVER,
                    from,
                    "print\nNo or bad challenge for address.\n".to_string(),
                );
                return;
            }
            // force the IP key/value pair so the game can filter based on ip
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                c"ip".as_ptr() as *mut c_char,
                mp_engine_qcommon::net_chan::NET_AdrToString(view.common, from) as *mut c_char,
            );

            let ping = sv.svs.time - sv.svs.challenges[i].pingTime;
            let conn_msg = mp_engine_qcommon::stringed::SE_GetString2(
                view,
                "MP_SVGAME",
                "CLIENT_CONN_WITH_PING",
            )
            .replace("%i", &i.to_string())
            .replacen("%i", &ping.to_string(), 1);
            mp_engine_qcommon::common::common::com_printf(view.common, &conn_msg);
            sv.svs.challenges[i].connected = qtrue;

            // never reject a LAN client based on ping
            if !view.is_lan_address(&from) {
                let min_ping = (*view.common.sv_minPing).value;
                if min_ping != 0.0 && (ping as f32) < min_ping {
                    // don't let them keep trying until they get a big delay
                    let high_ping_msg = mp_engine_qcommon::stringed::SE_GetString2(
                        view,
                        "MP_SVGAME",
                        "SERVER_FOR_HIGH_PING",
                    );
                    mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                        view.common,
                        netsrc_t::NS_SERVER,
                        from,
                        format!("print\n{}\n", high_ping_msg),
                    );
                    let rejected_msg = mp_engine_qcommon::stringed::SE_GetString2(
                        view,
                        "MP_SVGAME",
                        "CLIENT_REJECTED_LOW_PING",
                    )
                    .replace("%i", &i.to_string());
                    mp_engine_qcommon::common::common::com_printf(view.common, &rejected_msg);
                    // reset the address otherwise their ping will keep increasing
                    // with each connect message and they'd eventually be able to connect
                    sv.svs.challenges[i].adr.port = 0;
                    return;
                }
                let max_ping = (*view.common.sv_maxPing).value;
                if max_ping != 0.0 && (ping as f32) > max_ping {
                    let low_ping_msg = mp_engine_qcommon::stringed::SE_GetString2(
                        view,
                        "MP_SVGAME",
                        "SERVER_FOR_LOW_PING",
                    );
                    mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                        view.common,
                        netsrc_t::NS_SERVER,
                        from,
                        format!("print\n{}\n", low_ping_msg),
                    );
                    let rejected_msg = mp_engine_qcommon::stringed::SE_GetString2(
                        view,
                        "MP_SVGAME",
                        "CLIENT_REJECTED_HIGH_PING",
                    )
                    .replace("%i", &i.to_string());
                    mp_engine_qcommon::common::common::com_printf(view.common, &rejected_msg);
                    return;
                }
            }
        } else {
            // force the "ip" info key to "localhost"
            Info_SetValueForKey(
                userinfo.as_mut_ptr(),
                c"ip".as_ptr() as *mut c_char,
                c"localhost".as_ptr() as *mut c_char,
            );
        }

        let newcl: client_t = core::mem::zeroed();
        let mut reconnect = false;
        let cl_ptr: *mut client_t;

        // if there is already a slot for this ip, reuse it
        {
            let mut i: c_int = 0;
            let mut found = false;
            while i < max_clients {
                let cl = sv.svs.clients.offset(i as isize);
                if (*cl).state == clientState_t::CS_FREE {
                    i += 1;
                    continue;
                }
                if mp_engine_qcommon::net_chan::NET_CompareBaseAdr(
                    view.common,
                    from,
                    (*cl).netchan.remoteAddress,
                ) == qtrue
                    && ((*cl).netchan.qport == qport
                        || from.port == (*cl).netchan.remoteAddress.port)
                {
                    let adr = mp_engine_qcommon::net_chan::NET_AdrToString(view.common, from);
                    mp_engine_qcommon::common::common::com_printf(
                        view.common,
                        &format!(
                            "{}:reconnect\n",
                            core::ffi::CStr::from_ptr(adr).to_string_lossy()
                        ),
                    );
                    reconnect_cl = cl;
                    reconnect = true;
                    // VVFIXME - both SOF2 and Wolf remove this call, claiming it blows away the user's info
                    // disconnect the client from the game first so any flags the
                    // player might have are dropped
                    let cl_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
                        / core::mem::size_of::<client_t>() as isize)
                        as c_int;
                    // Real `&mut Server` in scope — reach the game VM directly (rule 7).
                    mp_engine_qcommon::vm::VM_Call(
                        view.common,
                        sv.gvm,
                        mp_abi::game::exports::MpGameExport::GAME_CLIENT_DISCONNECT as c_int,
                        &[cl_num],
                    );
                    found = true;
                    break;
                }
                i += 1;
            }
            if found {
                cl_ptr = reconnect_cl;
            } else {
                // find a client slot
                // if "sv_privateClients" is set > 0, then that number
                // of client slots will be reserved for connections that
                // have "password" set to the value of "sv_privatePassword"
                let password =
                    Info_ValueForKey(userinfo.as_mut_ptr(), c"password".as_ptr() as *mut c_char);
                let start_index: c_int =
                    if strcmp(password, (*view.common.sv_privatePassword).string) == 0 {
                        0
                    } else {
                        (*view.common.sv_privateClients).integer
                    };

                let mut new_slot: *mut client_t = core::ptr::null_mut();
                {
                    let mut j = start_index;
                    while j < max_clients {
                        let cl = sv.svs.clients.offset(j as isize);
                        if (*cl).state == clientState_t::CS_FREE {
                            new_slot = cl;
                            break;
                        }
                        j += 1;
                    }
                }

                if new_slot.is_null() {
                    if mp_engine_qcommon::net_chan::NET_IsLocalAddress(from) == qtrue {
                        let mut count = 0;
                        let mut j = start_index;
                        while j < max_clients {
                            let cl = sv.svs.clients.offset(j as isize);
                            if (*cl).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT {
                                count += 1;
                            }
                            j += 1;
                        }
                        // if they're all bots
                        if count >= max_clients - start_index {
                            let last = sv.svs.clients.offset((max_clients - 1) as isize);
                            crate::SV_DropClient(
                                view.common,
                                sv,
                                last,
                                c"only bots on server".as_ptr() as *const c_char,
                            );
                            new_slot = sv.svs.clients.offset((max_clients - 1) as isize);
                        } else {
                            mp_engine_qcommon::common::com_error(
                                errorParm_t::ERR_FATAL,
                                "server is full on local connect\n".to_string(),
                            );
                        }
                    } else {
                        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                            view.common,
                            netsrc_t::NS_SERVER,
                            from,
                            format!(
                                "print\n{}\n",
                                crate::SV_GetStringEdString_str(sv, "MP_SVGAME", "SERVER_IS_FULL")
                            ),
                        );
                        mp_engine_qcommon::common::common::com_printf(
                            view.common,
                            "Rejected a connection.\n",
                        );
                        return;
                    }
                }

                // we got a newcl, so reset the reliableSequence and reliableAcknowledge
                (*new_slot).reliableAcknowledge = 0;
                (*new_slot).reliableSequence = 0;
                cl_ptr = new_slot;
            }
        }

        // build a new connection
        // accept the new client
        // this is the only place a client_t is ever initialized
        *cl_ptr = newcl;
        let client_num = ((cl_ptr as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        let ent = crate::sv_game::SV_GentityNum(sv, client_num);
        (*cl_ptr).gentity = ent;

        // save the challenge
        (*cl_ptr).challenge = challenge;

        // save the address
        mp_engine_qcommon::net_chan::Netchan_Setup(
            netsrc_t::NS_SERVER,
            &mut (*cl_ptr).netchan,
            from,
            qport,
        );

        // save the userinfo
        Q_strncpyz(
            (*cl_ptr).userinfo.as_mut_ptr(),
            userinfo.as_ptr(),
            (*cl_ptr).userinfo.len() as c_int,
        );

        // get the game a chance to reject this connection or modify the userinfo
        // Real `&mut Server` in scope — reach the game VM directly (rule 7).
        let denied = mp_engine_qcommon::vm::VM_Call(
            view.common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_CONNECT as c_int,
            &[client_num, qtrue as c_int, qfalse as c_int], // firstTime = qtrue
        );
        if denied != 0 {
            // we can't just use VM_ArgPtr, because that is only valid inside a VM_Call
            //TODO: Port VM_ExplicitArgPtr
            // Source: oracle/codemp/qcommon/vm.cpp — the denied-connect reason
            // string is resolved through the direct `VM_ExplicitArgPtr` call
            // below (the game VM's shifted arg block), not an `EngineHost` seam.
            let denied_ptr =
                mp_engine_qcommon::vm_fns::VM_ExplicitArgPtr(view.common, sv.gvm, denied)
                    as *const c_char;
            mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
                view.common,
                netsrc_t::NS_SERVER,
                from,
                format!(
                    "print\n{}\n",
                    core::ffi::CStr::from_ptr(denied_ptr).to_string_lossy()
                ),
            );
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!(
                    "Game rejected a connection: {}.\n",
                    core::ffi::CStr::from_ptr(denied_ptr).to_string_lossy()
                ),
            );
            return;
        }

        SV_UserinfoChanged(view, cl_ptr);

        // send the connect packet to the client
        mp_engine_qcommon::net_chan::NET_OutOfBandPrint(
            view.common,
            netsrc_t::NS_SERVER,
            from,
            "connectResponse".to_string(),
        );

        mp_engine_qcommon::common::common::com_printf(
            view.common,
            &format!(
                "Going from CS_FREE to CS_CONNECTED for {}\n",
                CStr::from_ptr((*cl_ptr).name.as_ptr()).to_string_lossy()
            ),
        );

        (*cl_ptr).state = clientState_t::CS_CONNECTED;
        (*cl_ptr).nextSnapshotTime = sv.svs.time;
        (*cl_ptr).lastPacketTime = sv.svs.time;
        (*cl_ptr).lastConnectTime = sv.svs.time;

        // when we receive the first packet from the client, we will
        // notice that it is from a different serverid and that the
        // gamestate message was not just sent, forcing a retransmit
        (*cl_ptr).gamestateMessageNum = -1;

        (*cl_ptr).lastUserInfoChange = 0; // reset the delay
        (*cl_ptr).lastUserInfoCount = 0; // reset the count

        // if this was the first client on the server, or the last client
        // the server can hold, send a heartbeat to the master.
        let mut count = 0;
        let mut i: c_int = 0;
        while i < max_clients {
            if (*sv.svs.clients.offset(i as isize)).state as c_int
                >= clientState_t::CS_CONNECTED as c_int
            {
                count += 1;
            }
            i += 1;
        }
        if count == 1 || count == max_clients {
            crate::sv_ccmds::SV_Heartbeat_f(sv);
        }

        let _ = reconnect;
        let _ = denied;
    }
}

/// Raven `SV_SendClientGameState`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:697-817`
pub fn SV_SendClientGameState(view: &mut EngineHostView, sv: &mut Server, client: *mut client_t) {
    unsafe {
        // MW - my attempt to fix illegible server message errors caused by
        // packet fragmentation of initial snapshot.
        while (*client).state != clientState_t::CS_FREE as clientState_t as _
            && (*client).netchan.unsentFragments != qfalse
        {
            // send additional message fragments if the last message
            // was too large to send at once
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!(
                    "[ISM]SV_SendClientGameState() [2] for {}, writing out old fragments\n",
                    CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
                ),
            );
            mp_engine_qcommon::net_chan::Netchan_TransmitNextFragment(view, &mut (*client).netchan);
        }

        mp_engine_qcommon::common::common::com_printf(
            view.common,
            &format!(
                "SV_SendClientGameState() for {}\n",
                CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
            ),
        );
        mp_engine_qcommon::common::common::com_printf(
            view.common,
            &format!(
                "Going from CS_CONNECTED to CS_PRIMED for {}\n",
                CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
            ),
        );
        (*client).state = clientState_t::CS_PRIMED;
        (*client).pureAuthentic = 0;

        // when we receive the first packet from the client, we will
        // notice that it is from a different serverid and that the
        // gamestate message was not just sent, forcing a retransmit
        (*client).gamestateMessageNum = (*client).netchan.outgoingSequence;

        let mut msg_buffer = [0u8; mp_engine_qcommon::qcommon::net_limits::MAX_MSGLEN as usize];
        let mut msg: msg_t = core::mem::zeroed();
        mp_engine_qcommon::msg::MSG_Init(
            view,
            &mut msg,
            msg_buffer.as_mut_ptr(),
            msg_buffer.len() as c_int,
        );

        // NOTE, MRE: all server->client messages now acknowledge
        // let the client know which reliable clientCommands we have received
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, (*client).lastClientCommand);

        // send any server commands waiting to be sent first.
        crate::SV_UpdateServerCommandsToClient(view.common, client, &mut msg);

        // send the gamestate
        mp_engine_qcommon::msg::MSG_WriteByte(
            view.common,
            &mut msg,
            mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_gamestate as c_int,
        );
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, (*client).reliableSequence);

        // write the configstrings
        for start in 0..mp_qshared::shared::game_state::MAX_CONFIGSTRINGS {
            let cs = sv.sv.configstrings[start as usize];
            if !cs.is_null() && *cs != 0 {
                mp_engine_qcommon::msg::MSG_WriteByte(
                    view.common,
                    &mut msg,
                    mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_configstring as c_int,
                );
                mp_engine_qcommon::msg::MSG_WriteShort(view.common, &mut msg, start as c_int);
                mp_engine_qcommon::msg::MSG_WriteBigString(view.common, &mut msg, cs);
            }
        }

        // write the baselines
        let mut nullstate: mp_qshared::common::mp::qcommon::entity_state::entityState_t =
            core::mem::zeroed();
        for start in 0..mp_qshared::shared::limits::MAX_GENTITIES {
            let base = &mut sv.sv.svEntities[start as usize].baseline;
            if base.number == 0 {
                continue;
            }
            mp_engine_qcommon::msg::MSG_WriteByte(
                view.common,
                &mut msg,
                mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_baseline as c_int,
            );
            mp_engine_qcommon::msg::MSG_WriteDeltaEntity(
                view.common,
                &mut msg,
                &mut nullstate,
                base,
                qtrue,
            );
        }

        mp_engine_qcommon::msg::MSG_WriteByte(
            view.common,
            &mut msg,
            mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_EOF as c_int,
        );

        let client_num = ((client as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, client_num);

        // write the checksum feed
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, sv.sv.checksumFeed);

        // rwwRMG - send info for the terrain
        // Raven's `if (TheRandomMissionManager)` NULL test → the `cm.land_scape`
        // Option presence (RmManager is a non-optional Engine field per ruling
        // 12; the landscape lives on the CollisionWorld per rmg-terrain RMG-D1).
        //
        // `land` holds a shared borrow of `view.cm`; the MSG writes below reach
        // only the disjoint `view.common` field. `land`'s last use is
        // `get_rand_seed()`, so the `view.cm` borrow is released before the
        // whole-`view` `SV_WriteRMGAutomapSymbols` call (NLL).
        if let Some(land) = view.cm.land_scape.as_ref() {
            let mut heightmap = [0u8; 15000];
            let height_src = land.height_map().as_ptr();
            let real_area = land.real_area();
            let total_out_h = mp_engine_qcommon::zlib_seam::deflate_sync_flush(
                height_src,
                real_area,
                &mut heightmap,
            );
            mp_engine_qcommon::msg::MSG_WriteShort(view.common, &mut msg, total_out_h as c_int);
            mp_engine_qcommon::msg::MSG_WriteBits(view.common, &mut msg, 1, 1);
            mp_engine_qcommon::msg::MSG_WriteData(
                view.common,
                &mut msg,
                heightmap.as_ptr() as *const (),
                total_out_h as c_int,
            );

            let flatten_src = land.flatten_map().as_ptr();
            let total_out_f = mp_engine_qcommon::zlib_seam::deflate_sync_flush(
                flatten_src,
                real_area,
                &mut heightmap,
            );
            mp_engine_qcommon::msg::MSG_WriteShort(view.common, &mut msg, total_out_f as c_int);
            mp_engine_qcommon::msg::MSG_WriteBits(view.common, &mut msg, 1, 1);
            mp_engine_qcommon::msg::MSG_WriteData(
                view.common,
                &mut msg,
                heightmap.as_ptr() as *const (),
                total_out_f as c_int,
            );

            // Seed is needed for misc ents and noise
            mp_engine_qcommon::msg::MSG_WriteLong(
                view.common,
                &mut msg,
                land.get_rand_seed() as c_int,
            );

            SV_WriteRMGAutomapSymbols(view, &mut msg);
        } else {
            mp_engine_qcommon::msg::MSG_WriteShort(view.common, &mut msg, 0);
        }

        // deliver this to the client
        crate::SV_SendMessageToClient(view, sv, &mut msg, client);
    }
}

/// Raven `SV_VerifyPaks_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1283-1433`
pub fn SV_VerifyPaks_f(view: &mut EngineHostView, sv: &mut Server, cl: *mut client_t) {
    // _XBOX is not defined in this build — the whole body is live.
    if unsafe { (*view.common.sv_pure).integer } == 0 {
        return;
    }

    let mut n_chk_sum1: c_int;
    let mut n_chk_sum2: c_int = 0;
    let mut b_good;

    if mp_engine_qcommon::cvar_fns::Cvar_VariableValue(view, c"vm_cgame".as_ptr() as *const c_char)
        != 0.0
    {
        let mut cs1 = 0;
        b_good = mp_engine_qcommon::files_pc::FS_FileIsInPAK(
            view.common,
            c"vm/cgame.qvm".as_ptr() as *const c_char,
            &mut cs1,
        ) == 1;
        n_chk_sum1 = cs1;
    } else {
        let mut cs1 = 0;
        b_good = mp_engine_qcommon::files_pc::FS_FileIsInPAK(
            view.common,
            c"cgamex86.dll".as_ptr() as *const c_char,
            &mut cs1,
        ) == 1;
        n_chk_sum1 = cs1;
    }

    if b_good {
        if mp_engine_qcommon::cvar_fns::Cvar_VariableValue(view, c"vm_ui".as_ptr() as *const c_char)
            != 0.0
        {
            let mut cs2 = 0;
            b_good = mp_engine_qcommon::files_pc::FS_FileIsInPAK(
                view.common,
                c"vm/ui.qvm".as_ptr() as *const c_char,
                &mut cs2,
            ) == 1;
            n_chk_sum2 = cs2;
        } else {
            let mut cs2 = 0;
            b_good = mp_engine_qcommon::files_pc::FS_FileIsInPAK(
                view.common,
                c"uix86.dll".as_ptr() as *const c_char,
                &mut cs2,
            ) == 1;
            n_chk_sum2 = cs2;
        }
    }

    let mut n_client_paks = mp_engine_qcommon::cmd_common::Cmd_Argc(view.common);
    // start at arg 1 (skip cl_paks)
    let mut n_cur_arg: c_int = 1;

    let mut n_client_chk_sum = [0i32; 1024];
    let mut n_server_chk_sum = [0i32; 1024];

    // we basically use this while loop to avoid using 'goto' :)
    while b_good {
        // must be at least 6: "cl_paks cgame ui @ firstref ... numChecksums"
        if n_client_paks < 6 {
            b_good = false;
            break;
        }
        // verify first to be the cgame checksum
        let p_arg = mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, n_cur_arg);
        n_cur_arg += 1;
        if p_arg.is_null()
            || unsafe { *p_arg } == b'@' as c_char
            || unsafe { atoi(p_arg) } != n_chk_sum1
        {
            b_good = false;
            break;
        }
        // verify the second to be the ui checksum
        let p_arg = mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, n_cur_arg);
        n_cur_arg += 1;
        if p_arg.is_null()
            || unsafe { *p_arg } == b'@' as c_char
            || unsafe { atoi(p_arg) } != n_chk_sum2
        {
            b_good = false;
            break;
        }
        // should be sitting at the delimeter now
        let p_arg = mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, n_cur_arg);
        n_cur_arg += 1;
        if unsafe { *p_arg } != b'@' as c_char {
            b_good = false;
            break;
        }
        // store checksums since tokenization is not re-entrant
        let mut i: usize = 0;
        while n_cur_arg < n_client_paks {
            n_client_chk_sum[i] = unsafe {
                atoi(mp_engine_qcommon::cmd_common::Cmd_Argv(
                    view.common,
                    n_cur_arg,
                ))
            };
            n_cur_arg += 1;
            i += 1;
        }

        // store number to compare against (minus one cause the last is the number of checksums)
        n_client_paks = i as c_int - 1;

        // make sure none of the client check sums are the same
        // so the client can't send 5 the same checksums
        let mut i: c_int = 0;
        'outer: while i < n_client_paks {
            let mut j: c_int = 0;
            while j < n_client_paks {
                if i == j {
                    j += 1;
                    continue;
                }
                if n_client_chk_sum[i as usize] == n_client_chk_sum[j as usize] {
                    b_good = false;
                    break 'outer;
                }
                j += 1;
            }
            i += 1;
        }
        if !b_good {
            break;
        }

        // get the pure checksums of the pk3 files loaded by the server
        let p_paks = mp_engine_qcommon::files_pc::FS_LoadedPakPureChecksums(view.common);
        mp_engine_qcommon::cmd_common::Cmd_TokenizeString(view.common, p_paks);
        let mut n_server_paks = mp_engine_qcommon::cmd_common::Cmd_Argc(view.common);
        if n_server_paks > 1024 {
            n_server_paks = 1024;
        }

        let mut i: c_int = 0;
        while i < n_server_paks {
            n_server_chk_sum[i as usize] =
                unsafe { atoi(mp_engine_qcommon::cmd_common::Cmd_Argv(view.common, i)) };
            i += 1;
        }

        // check if the client has provided any pure checksums of pk3 files not loaded by the server
        let mut i: c_int = 0;
        let mut bad = false;
        while i < n_client_paks {
            let mut j: c_int = 0;
            while j < n_server_paks {
                if n_client_chk_sum[i as usize] == n_server_chk_sum[j as usize] {
                    break;
                }
                j += 1;
            }
            if j >= n_server_paks {
                b_good = false;
                bad = true;
                break;
            }
            i += 1;
        }
        if bad || !b_good {
            break;
        }

        // check if the number of checksums was correct
        n_chk_sum1 = sv.sv.checksumFeed;
        let mut i: c_int = 0;
        while i < n_client_paks {
            n_chk_sum1 ^= n_client_chk_sum[i as usize];
            i += 1;
        }
        n_chk_sum1 ^= n_client_paks;
        if n_chk_sum1 != n_client_chk_sum[n_client_paks as usize] {
            b_good = false;
            break;
        }

        // break out
        break;
    }

    if b_good {
        unsafe {
            (*cl).pureAuthentic = 1;
        }
    } else {
        unsafe {
            (*cl).pureAuthentic = 0;
            (*cl).nextSnapshotTime = -1;
            (*cl).state = clientState_t::CS_ACTIVE;
        }
        crate::SV_SendClientSnapshot(view, sv, cl);
        crate::SV_DropClient(
            view.common,
            sv,
            cl,
            c"Unpure client detected. Invalid .PK3 files referenced!".as_ptr() as *const c_char,
        );
    }
}

/// Raven `SV_DoneDownload_f`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1029-1033`
pub fn SV_DoneDownload_f(view: &mut EngineHostView, sv: &mut Server, cl: *mut client_t) {
    unsafe {
        mp_engine_qcommon::common::common::com_printf(
            view.common,
            &format!(
                "clientDownload: {} Done\n",
                CStr::from_ptr((*cl).name.as_ptr()).to_string_lossy()
            ),
        );
    }
    // resend the game state to update any clients that entered during the download
    SV_SendClientGameState(view, sv, cl);
}

/// Raven `SV_ClientCommand`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1590-1639`
pub fn SV_ClientCommand(
    view: &mut EngineHostView,
    sv: &mut Server,
    cl: *mut client_t,
    msg: *mut msg_t,
) -> qboolean {
    unsafe {
        let seq = mp_engine_qcommon::msg::MSG_ReadLong(view.common, msg);
        let s = mp_engine_qcommon::msg::MSG_ReadString(view.common, msg);
        let mut client_ok = qtrue;

        // see if we have already executed it
        if (*cl).lastClientCommand >= seq {
            return qtrue;
        }

        mp_engine_qcommon::common::common::com_printf(
            view.common,
            &format!(
                "clientCommand: {} : {} : {}\n",
                CStr::from_ptr((*cl).name.as_ptr()).to_string_lossy(),
                seq,
                CStr::from_ptr(s).to_string_lossy()
            ),
        );

        // drop the connection if we have somehow lost commands
        if seq > (*cl).lastClientCommand + 1 {
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!(
                    "Client {} lost {} clientCommands\n",
                    CStr::from_ptr((*cl).name.as_ptr()).to_string_lossy(),
                    seq - (*cl).lastClientCommand + 1
                ),
            );
            crate::SV_DropClient(
                view.common,
                sv,
                cl,
                c"Lost reliable commands".as_ptr() as *const c_char,
            );
            return qfalse;
        }

        // malicious users may try using too many string commands
        // to lag other players.  If we decide that we want to stall
        // the command, we will stop processing the rest of the packet,
        // including the usercmd.  This causes flooders to lag themselves
        // but not other people
        // We don't do this when the client hasn't been active yet since its
        // normal to spam a lot of commands when downloading
        if (*view.common.com_cl_running).integer == 0
            && (*cl).state as c_int >= clientState_t::CS_ACTIVE as c_int
            && (*view.common.sv_floodProtect).integer != 0
            && sv.svs.time < (*cl).nextReliableTime
        {
            // ignore any other text messages from this client but let them keep playing
            client_ok = qfalse;
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!(
                    "client text ignored for {}\n",
                    CStr::from_ptr((*cl).name.as_ptr()).to_string_lossy()
                ),
            );
        }

        // don't allow another command for one second
        (*cl).nextReliableTime = sv.svs.time + 1000;

        SV_ExecuteClientCommand(view, sv, cl, s, client_ok);

        (*cl).lastClientCommand = seq;
        let s_str = core::ffi::CStr::from_ptr(s).to_string_lossy();
        Com_sprintf(
            (*cl).lastClientCommandString.as_mut_ptr(),
            (*cl).lastClientCommandString.len() as c_int,
            &s_str,
        );

        qtrue // continue procesing
    }
}

/// Raven `SV_UserMove`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1674-1755`
pub fn SV_UserMove(
    common: &mut Common,
    sv: &mut Server,
    cl: *mut client_t,
    msg: *mut msg_t,
    delta: qboolean,
) {
    unsafe {
        if delta == qtrue {
            (*cl).deltaMessage = (*cl).messageAcknowledge;
        } else {
            (*cl).deltaMessage = -1;
        }

        let cmd_count = mp_engine_qcommon::msg::MSG_ReadByte(common, msg);

        if cmd_count < 1 {
            mp_engine_qcommon::common::common::com_printf(common, "cmdCount < 1\n");
            return;
        }

        if cmd_count > mp_engine_qcommon::qcommon::net_limits::MAX_PACKET_USERCMDS as c_int {
            mp_engine_qcommon::common::common::com_printf(
                common,
                "cmdCount > MAX_PACKET_USERCMDS\n",
            );
            return;
        }

        // use the checksum feed in the key
        let mut key = sv.sv.checksumFeed;
        // also use the message acknowledge
        key ^= (*cl).messageAcknowledge;
        // also use the last acknowledged server command in the key
        let idx = ((*cl).reliableAcknowledge
            & (mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS as c_int - 1))
            as usize;
        key ^= mp_engine_qcommon::common_fns::Com_HashKey(
            (*cl).reliableCommands[idx].as_mut_ptr(),
            32,
        );

        let mut cmds = [core::mem::zeroed::<usercmd_t>();
            mp_engine_qcommon::qcommon::net_limits::MAX_PACKET_USERCMDS as usize];
        let mut nullcmd: usercmd_t = core::mem::zeroed();
        let mut oldcmd: *mut usercmd_t = &mut nullcmd;
        let mut i: c_int = 0;
        while i < cmd_count {
            let cmd = &mut cmds[i as usize] as *mut usercmd_t;
            mp_engine_qcommon::msg::MSG_ReadDeltaUsercmdKey(common, msg, key, oldcmd, cmd);
            oldcmd = cmd;
            i += 1;
        }

        // save time for ping calculation
        let pmask_idx = ((*cl).messageAcknowledge
            & mp_engine_qcommon::qcommon::net_limits::PACKET_MASK as c_int)
            as usize;
        (*cl).frames[pmask_idx].messageAcked = sv.svs.time;

        // if this is the first usercmd we have received
        // this gamestate, put the client into the world
        if (*cl).state == clientState_t::CS_PRIMED {
            SV_ClientEnterWorld(common, sv, cl, &mut cmds[0]);
            // the moves can be processed normaly
        }

        // _XBOX is not defined — pure check is live
        if (*common.sv_pure).integer != 0 && (*cl).pureAuthentic == 0 {
            crate::SV_DropClient(
                common,
                sv,
                cl,
                c"Cannot validate pure client!".as_ptr() as *const c_char,
            );
            return;
        }

        if (*cl).state != clientState_t::CS_ACTIVE {
            (*cl).deltaMessage = -1;
            return;
        }

        // usually, the first couple commands will be duplicates
        // of ones we have previously received, but the servertimes
        // in the commands will cause them to be immediately discarded
        let mut i: c_int = 0;
        while i < cmd_count {
            // if this is a cmd from before a map_restart ignore it
            if cmds[i as usize].serverTime > cmds[(cmd_count - 1) as usize].serverTime {
                i += 1;
                continue;
            }
            // don't execute if this is an old cmd which is already executed
            // these old cmds are included when cl_packetdup > 0
            if cmds[i as usize].serverTime <= (*cl).lastUsercmd.serverTime {
                i += 1;
                continue;
            }
            SV_ClientThink(common, sv, cl, &mut cmds[i as usize]);
            i += 1;
        }
    }
}

/// Raven `SV_ExecuteClientMessage`.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1773-1854`
pub fn SV_ExecuteClientMessage(
    view: &mut EngineHostView,
    sv: &mut Server,
    cl: *mut client_t,
    msg: *mut msg_t,
) {
    unsafe {
        mp_engine_qcommon::msg::MSG_Bitstream(msg);

        let server_id = mp_engine_qcommon::msg::MSG_ReadLong(view.common, msg);
        (*cl).messageAcknowledge = mp_engine_qcommon::msg::MSG_ReadLong(view.common, msg);

        if (*cl).messageAcknowledge < 0 {
            // usually only hackers create messages like this
            // it is more annoying for them to let them hanging
            // SV_DropClient( cl, "illegible client message" );
            return;
        }

        (*cl).reliableAcknowledge = mp_engine_qcommon::msg::MSG_ReadLong(view.common, msg);

        // NOTE: when the client message is fux0red the acknowledgement numbers
        // can be out of range, this could cause the server to send thousands of server
        // commands which the server thinks are not yet acknowledged in SV_UpdateServerCommandsToClient
        if (*cl).reliableAcknowledge
            < (*cl).reliableSequence
                - mp_engine_qcommon::qcommon::net_limits::MAX_RELIABLE_COMMANDS as c_int
        {
            // usually only hackers create messages like this
            // it is more annoying for them to let them hanging
            // SV_DropClient( cl, "illegible client message" );
            (*cl).reliableAcknowledge = (*cl).reliableSequence;
            return;
        }

        // if this is a usercmd from a previous gamestate,
        // ignore it or retransmit the current gamestate
        //
        // if the client was downloading, let it stay at whatever serverId and
        // gamestate it was at.  This allows it to keep downloading even when
        // the gamestate changes.  After the download is finished, we'll
        // notice and send it a new game state
        //
        // _XBOX is not defined — the non-XBOX arm (download-aware) is live.
        if server_id != sv.sv.serverId && *(*cl).downloadName.as_ptr() == 0 {
            if server_id == sv.sv.restartedServerId {
                // they just haven't caught the map_restart yet
                return;
            }
            // if we can tell that the client has dropped the last
            // gamestate we sent them, resend it
            if (*cl).messageAcknowledge > (*cl).gamestateMessageNum {
                mp_engine_qcommon::common::common::com_printf(
                    view.common,
                    &format!(
                        "{} : dropped gamestate, resending\n",
                        CStr::from_ptr((*cl).name.as_ptr()).to_string_lossy()
                    ),
                );
                SV_SendClientGameState(view, sv, cl);
            }
            return;
        }

        // read optional clientCommand strings
        let mut c: c_int;
        loop {
            c = mp_engine_qcommon::msg::MSG_ReadByte(view.common, msg);
            if c == mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_EOF as c_int {
                break;
            }
            if c != mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_clientCommand as c_int {
                break;
            }
            if SV_ClientCommand(view, sv, cl, msg) == qfalse {
                return; // we couldn't execute it because of the flood protection
            }
            if (*cl).state == clientState_t::CS_ZOMBIE {
                return; // disconnect command
            }
        }

        // read the usercmd_t
        if c == mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_move as c_int {
            SV_UserMove(view.common, sv, cl, msg, qtrue);
        } else if c == mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_moveNoDelta as c_int {
            SV_UserMove(view.common, sv, cl, msg, qfalse);
        } else if c != mp_engine_qcommon::qcommon::clc_ops_e::clc_ops_e::clc_EOF as c_int {
            let client_num = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
                / core::mem::size_of::<client_t>() as isize) as c_int;
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!("WARNING: bad command byte for client {}\n", client_num),
            );
        }
    }
}

/// Raven `SV_DropClient` — called when the player is totally leaving the
/// server, either willingly or unwillingly. This is NOT called if the entire
/// server is quiting or crashing — `SV_FinalMessage()` handles that.
///
/// Source: `oracle/codemp/server/sv_client.cpp:580-666`
pub fn SV_DropClient(
    common: &mut Common,
    sv: &mut Server,
    drop: *mut client_t,
    reason: *const c_char,
) {
    unsafe {
        let drop_index = (drop as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize;

        if (*drop).state == clientState_t::CS_ZOMBIE {
            return; // already dropped
        }

        if (*drop).gentity.is_null() || (*(*drop).gentity).r.svFlags & SVF_BOT == 0 {
            // see if we already have a challenge for this ip
            for i in 0..MAX_CHALLENGES {
                let challenge = &mut sv.svs.challenges[i];
                if mp_engine_qcommon::net_chan::NET_CompareAdr(
                    common,
                    (*drop).netchan.remoteAddress,
                    challenge.adr,
                ) == qtrue
                {
                    challenge.connected = qfalse;
                    break;
                }
            }
        }

        // Kill any download
        crate::SV_CloseDownload(common, drop);

        // tell everyone why they got dropped
        let name = core::ffi::CStr::from_ptr((*drop).name.as_ptr()).to_string_lossy();
        let reason_str = core::ffi::CStr::from_ptr(reason).to_string_lossy();
        crate::SV_SendServerCommand(
            common,
            sv,
            core::ptr::null_mut(),
            // "%s" S_COLOR_WHITE " %s\n" — S_COLOR_WHITE is "^7"
            &format!("print \"{}^7 {}\n\"", name, reason_str),
        );

        mp_engine_qcommon::common_fns::Com_DPrintf(
            common,
            &format!("Going to CS_ZOMBIE for {}\n", name),
        );
        (*drop).state = clientState_t::CS_ZOMBIE; // become free in a few seconds

        if (*drop).download != 0 {
            mp_engine_qcommon::files_common::FS_FCloseFile(common, (*drop).download);
            (*drop).download = 0;
        }

        // call the prog function for removing a client
        // this will remove the body, among other things
        mp_engine_qcommon::vm::VM_Call(
            common,
            sv.gvm,
            mp_abi::game::exports::MpGameExport::GAME_CLIENT_DISCONNECT as c_int,
            &[drop_index as c_int],
        );

        // add the disconnect command
        crate::SV_SendServerCommand(common, sv, drop, &format!("disconnect \"{}\"", reason_str));

        if (*drop).netchan.remoteAddress.r#type == netadrtype_t::NA_BOT {
            crate::SV_BotFreeClient(common, sv, drop_index as c_int);
        }

        // nuke user info
        crate::SV_SetUserinfo(common, sv, drop_index as c_int, c"".as_ptr());

        // if this was the last client on the server, send a heartbeat
        // to the master so it is known the server is empty
        let mut i = 0;
        while i < (*common.sv_maxclients).integer {
            if (*sv.svs.clients.offset(i as isize)).state >= clientState_t::CS_CONNECTED {
                break;
            }
            i += 1;
        }
        if i == (*common.sv_maxclients).integer {
            crate::sv_ccmds::SV_Heartbeat_f(sv);
        }
    }
}

/// Raven `SV_CloseDownload` — clear/free any download vars.
///
/// Source: `oracle/codemp/server/sv_client.cpp:988-1006`
pub fn SV_CloseDownload(common: &mut Common, cl: *mut client_t) {
    unsafe {
        // EOF
        if (*cl).download != 0 {
            mp_engine_qcommon::files_common::FS_FCloseFile(common, (*cl).download);
        }
        (*cl).download = 0;
        (*cl).downloadName[0] = 0;

        // Free the temporary buffer space
        for i in 0..MAX_DOWNLOAD_WINDOW {
            if !(*cl).downloadBlocks[i].is_null() {
                mp_engine_qcommon::z_memman_pc::Z_Free(common, (*cl).downloadBlocks[i] as *mut ());
                (*cl).downloadBlocks[i] = core::ptr::null_mut();
            }
        }
    }
}

/// Raven `SV_WriteDownloadToClient` — check whether the client wants a file,
/// open it if needed, and pump download blocks into `msg` (the download-window
/// protocol). `#ifndef _XBOX`; this build does not compress downloads.
///
/// Source: `oracle/codemp/server/sv_client.cpp:1090-1253`
pub fn SV_WriteDownloadToClient(
    view: &mut EngineHostView,
    sv: &mut Server,
    cl: *mut client_t,
    msg: *mut msg_t,
) {
    unsafe {
        if (*cl).downloadName[0] == 0 {
            return; // Nothing being downloaded
        }

        let client_index = ((cl as *mut u8).offset_from(sv.svs.clients as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;

        if (*cl).download == 0 {
            // We open the file here
            mp_engine_qcommon::common::common::com_printf(
                view.common,
                &format!(
                    "clientDownload: {} : begining \"{}\"\n",
                    client_index,
                    core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                ),
            );

            let missionPack = mp_engine_qcommon::files_pc::FS_idPak(
                (*cl).downloadName.as_mut_ptr(),
                c"missionpack".as_ptr() as *mut c_char,
            );
            let idPack = missionPack != qfalse
                || mp_engine_qcommon::files_pc::FS_idPak(
                    (*cl).downloadName.as_mut_ptr(),
                    c"base".as_ptr() as *mut c_char,
                ) != qfalse;

            let mut downloadOpenFailed = false;
            if (*view.common.sv_allowDownload).integer == 0 || idPack {
                downloadOpenFailed = true;
            } else {
                (*cl).downloadSize = mp_engine_qcommon::files_common::FS_SV_FOpenFileRead(
                    view.common,
                    (*cl).downloadName.as_ptr(),
                    &mut (*cl).download,
                );
                if (*cl).downloadSize <= 0 {
                    downloadOpenFailed = true;
                }
            }

            if downloadOpenFailed {
                // cannot auto-download file
                let mut errorMessage: [c_char; 1024] = [0; 1024];
                if idPack {
                    mp_engine_qcommon::common::common::com_printf(
                        view.common,
                        &format!(
                            "clientDownload: {} : \"{}\" cannot download id pk3 files\n",
                            client_index,
                            core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr())
                                .to_string_lossy()
                        ),
                    );
                    if missionPack != qfalse {
                        Com_sprintf(
                            errorMessage.as_mut_ptr(),
                            errorMessage.len() as c_int,
                            &format!(
                                "Cannot autodownload Team Arena file \"{}\"\nThe Team Arena mission pack can be found in your local game store.",
                                core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                            ),
                        );
                    } else {
                        Com_sprintf(
                            errorMessage.as_mut_ptr(),
                            errorMessage.len() as c_int,
                            &format!(
                                "Cannot autodownload id pk3 file \"{}\"",
                                core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr())
                                    .to_string_lossy()
                            ),
                        );
                    }
                } else if (*view.common.sv_allowDownload).integer == 0 {
                    mp_engine_qcommon::common::common::com_printf(
                        view.common,
                        &format!(
                            "clientDownload: {} : \"{}\" download disabled",
                            client_index,
                            core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr())
                                .to_string_lossy()
                        ),
                    );
                    if (*view.common.sv_pure).integer != 0 {
                        Com_sprintf(
                            errorMessage.as_mut_ptr(),
                            errorMessage.len() as c_int,
                            &format!(
                                "Could not download \"{}\" because autodownloading is disabled on the server.\n\nYou will need to get this file elsewhere before you can connect to this pure server.\n",
                                core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                            ),
                        );
                    } else {
                        Com_sprintf(
                            errorMessage.as_mut_ptr(),
                            errorMessage.len() as c_int,
                            &format!(
                                "Could not download \"{}\" because autodownloading is disabled on the server.\n\nSet autodownload to No in your settings and you might be able to connect if you do have the file.\n",
                                core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr()).to_string_lossy()
                            ),
                        );
                    }
                } else {
                    mp_engine_qcommon::common::common::com_printf(
                        view.common,
                        &format!(
                            "clientDownload: {} : \"{}\" file not found on server\n",
                            client_index,
                            core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr())
                                .to_string_lossy()
                        ),
                    );
                    Com_sprintf(
                        errorMessage.as_mut_ptr(),
                        errorMessage.len() as c_int,
                        &format!(
                            "File \"{}\" not found on server for autodownloading.\n",
                            core::ffi::CStr::from_ptr((*cl).downloadName.as_ptr())
                                .to_string_lossy()
                        ),
                    );
                }
                mp_engine_qcommon::msg::MSG_WriteByte(
                    view.common,
                    msg,
                    mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_download as c_int,
                );
                mp_engine_qcommon::msg::MSG_WriteShort(view.common, msg, 0); // client is expecting block zero
                mp_engine_qcommon::msg::MSG_WriteLong(view.common, msg, -1); // illegal file size
                mp_engine_qcommon::msg::MSG_WriteString(view.common, msg, errorMessage.as_ptr());

                (*cl).downloadName[0] = 0;
                return;
            }

            // Init
            (*cl).downloadCurrentBlock = 0;
            (*cl).downloadClientBlock = 0;
            (*cl).downloadXmitBlock = 0;
            (*cl).downloadCount = 0;
            (*cl).downloadEOF = qfalse;
        }

        // Perform any reads that we need to
        while (*cl).downloadCurrentBlock - (*cl).downloadClientBlock < MAX_DOWNLOAD_WINDOW as c_int
            && (*cl).downloadSize != (*cl).downloadCount
        {
            let curindex = ((*cl).downloadCurrentBlock % MAX_DOWNLOAD_WINDOW as c_int) as usize;

            if (*cl).downloadBlocks[curindex].is_null() {
                (*cl).downloadBlocks[curindex] = mp_engine_qcommon::z_memman_pc::Z_Malloc(
                    view,
                    MAX_DOWNLOAD_BLKSIZE as c_int,
                    mp_qshared::common::mp::qcommon::tags::memtag_t::TAG_DOWNLOAD,
                    qtrue,
                    0,
                ) as *mut u8;
            }

            (*cl).downloadBlockSize[curindex] = mp_engine_qcommon::files_common::FS_Read(
                view.common,
                (*cl).downloadBlocks[curindex] as *mut (),
                MAX_DOWNLOAD_BLKSIZE as c_int,
                (*cl).download,
            );

            if (*cl).downloadBlockSize[curindex] < 0 {
                // EOF right now
                (*cl).downloadCount = (*cl).downloadSize;
                break;
            }

            (*cl).downloadCount += (*cl).downloadBlockSize[curindex];

            // Load in next block
            (*cl).downloadCurrentBlock += 1;
        }

        // Check to see if we have eof condition and add the EOF block
        if (*cl).downloadCount == (*cl).downloadSize
            && (*cl).downloadEOF == qfalse
            && (*cl).downloadCurrentBlock - (*cl).downloadClientBlock < MAX_DOWNLOAD_WINDOW as c_int
        {
            (*cl).downloadBlockSize
                [((*cl).downloadCurrentBlock % MAX_DOWNLOAD_WINDOW as c_int) as usize] = 0;
            (*cl).downloadCurrentBlock += 1;

            (*cl).downloadEOF = qtrue; // We have added the EOF block
        }

        // Loop up to window size times based on how many blocks we can fit in the
        // client snapMsec and rate

        // based on the rate, how many bytes can we fit in the snapMsec time of the client
        // normal rate / snapshotMsec calculation
        let mut rate = (*cl).rate;
        if (*view.common.sv_maxRate).integer != 0 {
            if (*view.common.sv_maxRate).integer < 1000 {
                mp_engine_qcommon::cvar_fns::Cvar_Set(
                    view,
                    c"sv_MaxRate".as_ptr(),
                    c"1000".as_ptr(),
                );
            }
            if (*view.common.sv_maxRate).integer < rate {
                rate = (*view.common.sv_maxRate).integer;
            }
        }

        let mut blockspersnap = if rate == 0 {
            1
        } else {
            (rate * (*cl).snapshotMsec / 1000 + MAX_DOWNLOAD_BLKSIZE as c_int)
                / MAX_DOWNLOAD_BLKSIZE as c_int
        };

        if blockspersnap < 0 {
            blockspersnap = 1;
        }

        while blockspersnap > 0 {
            blockspersnap -= 1;

            // Write out the next section of the file, if we have already reached
            // our window, automatically start retransmitting
            if (*cl).downloadClientBlock == (*cl).downloadCurrentBlock {
                return; // Nothing to transmit
            }

            if (*cl).downloadXmitBlock == (*cl).downloadCurrentBlock {
                // We have transmitted the complete window, should we start resending?
                // FIXME: This uses a hardcoded one second timeout for lost blocks
                if sv.svs.time - (*cl).downloadSendTime > 1000 {
                    (*cl).downloadXmitBlock = (*cl).downloadClientBlock;
                } else {
                    return;
                }
            }

            // Send current block
            let curindex = ((*cl).downloadXmitBlock % MAX_DOWNLOAD_WINDOW as c_int) as usize;

            mp_engine_qcommon::msg::MSG_WriteByte(
                view.common,
                msg,
                mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e::svc_download as c_int,
            );
            mp_engine_qcommon::msg::MSG_WriteShort(view.common, msg, (*cl).downloadXmitBlock);

            // block zero is special, contains file size
            if (*cl).downloadXmitBlock == 0 {
                mp_engine_qcommon::msg::MSG_WriteLong(view.common, msg, (*cl).downloadSize);
            }

            mp_engine_qcommon::msg::MSG_WriteShort(
                view.common,
                msg,
                (*cl).downloadBlockSize[curindex],
            );

            // Write the block
            if (*cl).downloadBlockSize[curindex] != 0 {
                mp_engine_qcommon::msg::MSG_WriteData(
                    view.common,
                    msg,
                    (*cl).downloadBlocks[curindex] as *const (),
                    (*cl).downloadBlockSize[curindex],
                );
            }

            mp_engine_qcommon::common_fns::Com_DPrintf(
                view.common,
                &format!(
                    "clientDownload: {} : writing block {}\n",
                    client_index,
                    (*cl).downloadXmitBlock
                ),
            );

            // Move on to the next block
            // It will get sent with next snap shot.  The rate will keep us in line.
            (*cl).downloadXmitBlock += 1;

            (*cl).downloadSendTime = sv.svs.time;
        }
    }
}
