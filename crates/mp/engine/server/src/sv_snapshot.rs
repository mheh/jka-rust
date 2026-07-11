//! `sv_snapshot.cpp` — server snapshot building/sending.

use core::ffi::c_int;

use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::cm_load::RenderModels;
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::qcommon::net_limits::{MAX_MSGLEN, MAX_RELIABLE_COMMANDS, PACKET_MASK};
use mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::game::g_public::SVF_BOT;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::{qfalse, qtrue};

use crate::server::client_s::client_t;
use crate::server::client_state_t::clientState_t;
use crate::server::snapshot_entity_numbers_t::{snapshotEntityNumbers_t, MAX_SNAPSHOT_ENTITIES};
use crate::server::sv_entity_s::svEntity_t;
use crate::sv_net_chan::{SV_Netchan_Transmit, SV_Netchan_TransmitNextFragment};
use crate::Server;

/// Raven `SV_AddEntToSnapshot`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:279-293`
pub fn SV_AddEntToSnapshot(
    sv: &mut Server,
    svEnt: *mut svEntity_t,
    gEnt: *mut sharedEntity_t,
    eNums: *mut snapshotEntityNumbers_t,
) {
    unsafe {
        // if we have already added this entity to this snapshot, don't add again
        if (*svEnt).snapshotCounter == sv.sv.snapshotCounter {
            return;
        }
        (*svEnt).snapshotCounter = sv.sv.snapshotCounter;

        // if we are full, silently discard entities
        if (*eNums).numSnapshotEntities as usize == MAX_SNAPSHOT_ENTITIES {
            return;
        }

        (*eNums).snapshotEntities[(*eNums).numSnapshotEntities as usize] = (*gEnt).s.number;
        (*eNums).numSnapshotEntities += 1;
    }
}

/// Raven `SV_SendClientMessages`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:806-832`
pub fn SV_SendClientMessages(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
) {
    // send a message to each connected client
    let max_clients = unsafe { (*common.sv_maxclients).integer };
    for i in 0..max_clients {
        let c = unsafe { sv.svs.clients.offset(i as isize) };
        unsafe {
            if (*c).state as i32 == 0 {
                continue; // not connected
            }

            if sv.svs.time < (*c).nextSnapshotTime {
                continue; // not time yet
            }

            // send additional message fragments if the last message
            // was too large to send at once
            if (*c).netchan.unsentFragments != 0 {
                (*c).nextSnapshotTime = sv.svs.time
                    + SV_RateMsec(
                        common,
                        cm,
                        sv,
                        rm,
                        host,
                        c,
                        (*c).netchan.unsentLength - (*c).netchan.unsentFragmentStart,
                    );
                SV_Netchan_TransmitNextFragment(common, cm, rm, host, &mut (*c).netchan);
                continue;
            }

            // generate and send a new message
            SV_SendClientSnapshot(common, cm, sv, rm, host, c);
        }
    }
}

/// `HEADER_RATE_BYTES` — include our header, IP header, and some overhead.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:622`
const HEADER_RATE_BYTES: c_int = 48;

/// Raven `SV_RateMsec` — return the number of msec a given size message is
/// supposed to take to clear, based on the current rate.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:623-643`
pub fn SV_RateMsec(
    common: &mut Common,
    cm: &mut CollisionWorld,
    _sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    client: *mut client_t,
    mut messageSize: c_int,
) -> c_int {
    unsafe {
        // individual messages will never be larger than fragment size
        if messageSize > 1500 {
            messageSize = 1500;
        }
        let mut rate = (*client).rate;
        if (*common.sv_maxRate).integer != 0 {
            if (*common.sv_maxRate).integer < 1000 {
                Cvar_Set(common, cm, rm, host, c"sv_MaxRate".as_ptr(), c"1000".as_ptr());
            }
            if (*common.sv_maxRate).integer < rate {
                rate = (*common.sv_maxRate).integer;
            }
        }
        (messageSize + HEADER_RATE_BYTES) * 1000 / rate
    }
}

/// Raven `SV_UpdateServerCommandsToClient` — (re)send all server commands the
/// client hasn't acknowledged yet.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:225-235`
pub fn SV_UpdateServerCommandsToClient(common: &mut Common, client: *mut client_t, msg: *mut msg_t) {
    unsafe {
        // write any unacknowledged serverCommands
        let mut i = (*client).reliableAcknowledge + 1;
        while i <= (*client).reliableSequence {
            mp_engine_qcommon::msg::MSG_WriteByte(
                common,
                msg,
                svc_ops_e::svc_serverCommand as c_int,
            );
            mp_engine_qcommon::msg::MSG_WriteLong(common, msg, i);
            mp_engine_qcommon::msg::MSG_WriteString(
                common,
                msg,
                (*client).reliableCommands[(i & (MAX_RELIABLE_COMMANDS as c_int - 1)) as usize]
                    .as_ptr(),
            );
            i += 1;
        }
        (*client).reliableSent = (*client).reliableSequence;
    }
}

/// Raven `SV_SendMessageToClient` — called by `SV_SendClientSnapshot` and
/// `SV_SendClientGameState`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:652-707`
pub fn SV_SendMessageToClient(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    msg: *mut msg_t,
    client: *mut client_t,
) {
    unsafe {
        // MW - my attempt to fix illegible server message errors caused by
        // packet fragmentation of initial snapshot.
        while (*client).state as c_int != 0 && (*client).netchan.unsentFragments != 0 {
            // send additional message fragments if the last message
            // was too large to send at once
            com_printf(
                common,
                &format!(
                    "[ISM]SV_SendClientGameState() [1] for {}, writing out old fragments\n",
                    core::ffi::CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
                ),
            );
            SV_Netchan_TransmitNextFragment(common, cm, rm, host, &mut (*client).netchan);
        }

        // record information about the message
        let idx = ((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize;
        (*client).frames[idx].messageSize = (*msg).cursize;
        (*client).frames[idx].messageSent = sv.svs.time;
        (*client).frames[idx].messageAcked = -1;

        // send the datagram
        SV_Netchan_Transmit(common, cm, rm, host, client, msg);

        // set nextSnapshotTime based on rate and requested number of updates

        // local clients get snapshots every frame
        if (*client).netchan.remoteAddress.r#type == netadrtype_t::NA_LOOPBACK
            || host.is_lan_address(&(*client).netchan.remoteAddress)
        {
            (*client).nextSnapshotTime = sv.svs.time - 1;
            return;
        }

        // normal rate / snapshotMsec calculation
        let mut rateMsec = SV_RateMsec(common, cm, sv, rm, host, client, (*msg).cursize);

        if rateMsec < (*client).snapshotMsec {
            // never send more packets than this, no matter what the rate is at
            rateMsec = (*client).snapshotMsec;
            (*client).rateDelayed = qfalse;
        } else {
            (*client).rateDelayed = qtrue;
        }

        (*client).nextSnapshotTime = sv.svs.time + rateMsec;

        // don't pile up empty snapshots while connecting
        if (*client).state != clientState_t::CS_ACTIVE {
            // a gigantic connection message may have already put the
            // nextSnapshotTime more than a second away, so don't shorten it; do
            // shorten if client is downloading
            if (*client).downloadName[0] == 0 && (*client).nextSnapshotTime < sv.svs.time + 1000 {
                (*client).nextSnapshotTime = sv.svs.time + 1000;
            }
        }
    }
}

/// Raven `SV_SendClientSnapshot` — also called by `SV_FinalMessage`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:719-798`
pub fn SV_SendClientSnapshot(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    rm: &mut RenderModels,
    host: &mut dyn EngineHost,
    client: *mut client_t,
) {
    unsafe {
        let mut msg_buf = [0u8; MAX_MSGLEN as usize];
        let mut msg: msg_t = core::mem::zeroed();

        if (*client).sentGamedir == qfalse {
            // rww - if this is the case then make sure there is an svc_setgame
            // sent before this snap
            let mut i = 0;

            mp_engine_qcommon::msg::MSG_Init(
                common,
                cm,
                rm,
                host,
                &mut msg,
                msg_buf.as_mut_ptr(),
                msg_buf.len() as c_int,
            );

            // have to include this for each message.
            mp_engine_qcommon::msg::MSG_WriteLong(common, &mut msg, (*client).lastClientCommand);

            mp_engine_qcommon::msg::MSG_WriteByte(common, &mut msg, svc_ops_e::svc_setgame as c_int);

            while *(*common.fs_gamedirvar).string.offset(i) != 0 {
                mp_engine_qcommon::msg::MSG_WriteByte(
                    common,
                    &mut msg,
                    *(*common.fs_gamedirvar).string.offset(i) as c_int,
                );
                i += 1;
            }
            mp_engine_qcommon::msg::MSG_WriteByte(common, &mut msg, 0);

            // MW - my attempt to fix illegible server message errors caused by
            // packet fragmentation of initial snapshot. rww - reusing this here
            while (*client).state as c_int != 0 && (*client).netchan.unsentFragments != 0 {
                com_printf(
                    common,
                    &format!(
                        "[ISM]SV_SendClientGameState() [1] for {}, writing out old fragments\n",
                        core::ffi::CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
                    ),
                );
                SV_Netchan_TransmitNextFragment(common, cm, rm, host, &mut (*client).netchan);
            }

            // record information about the message
            let idx = ((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize;
            (*client).frames[idx].messageSize = msg.cursize;
            (*client).frames[idx].messageSent = sv.svs.time;
            (*client).frames[idx].messageAcked = -1;

            // send the datagram
            SV_Netchan_Transmit(common, cm, rm, host, client, &mut msg);

            (*client).sentGamedir = qtrue;
        }

        // build the snapshot
        SV_BuildClientSnapshot(common, cm, sv, rm, host, client);

        // bots need to have their snapshots build, but the query them directly
        // without needing to be sent
        if !(*client).gentity.is_null() && (*(*client).gentity).r.svFlags & SVF_BOT != 0 {
            return;
        }

        mp_engine_qcommon::msg::MSG_Init(
            common,
            cm,
            rm,
            host,
            &mut msg,
            msg_buf.as_mut_ptr(),
            msg_buf.len() as c_int,
        );
        msg.allowoverflow = qtrue;

        // NOTE, MRE: all server->client messages now acknowledge
        // let the client know which reliable clientCommands we have received
        mp_engine_qcommon::msg::MSG_WriteLong(common, &mut msg, (*client).lastClientCommand);

        // (re)send any reliable server commands
        SV_UpdateServerCommandsToClient(common, client, &mut msg);

        // send over all the relevant entityState_t and the playerState_t
        SV_WriteSnapshotToClient(common, cm, sv, rm, host, client, &mut msg);

        // Add any download data if the client is downloading
        SV_WriteDownloadToClient(common, sv, client, &mut msg);

        // check for overflow
        if msg.overflowed != qfalse {
            com_printf(
                common,
                &format!(
                    "WARNING: msg overflowed for {}\n",
                    core::ffi::CStr::from_ptr((*client).name.as_ptr()).to_string_lossy()
                ),
            );
            mp_engine_qcommon::msg::MSG_Clear(&mut msg);
        }

        SV_SendMessageToClient(common, cm, sv, rm, host, &mut msg, client);
    }
}
