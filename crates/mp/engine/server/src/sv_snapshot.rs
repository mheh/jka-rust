//! `sv_snapshot.cpp` — server snapshot building/sending.

use core::ffi::c_int;

use mp_bg::public::entity_flags::EF_PERMANENT;
use mp_engine_qcommon::cm_load::{CM_LeafArea, CM_LeafCluster};
use mp_engine_qcommon::cm_test::{
    CM_AreasConnected, CM_ClusterPVS, CM_PointLeafnum, CM_WriteAreaBits,
};
use mp_engine_qcommon::collision_world::CollisionWorld;
use mp_engine_qcommon::common::com_error;
use mp_engine_qcommon::common::common::{com_printf, Common};
use mp_engine_qcommon::common::engine_host_view::EngineHostView;
use mp_engine_qcommon::common_fns::{Com_DPrintf, Com_Memset};
use mp_engine_qcommon::cvar_fns::Cvar_Set;
use mp_engine_qcommon::msg::{
    MSG_WriteBits, MSG_WriteByte, MSG_WriteData, MSG_WriteDeltaEntity, MSG_WriteDeltaPlayerstate,
    MSG_WriteLong,
};
use mp_engine_qcommon::qcommon::net_limits::{
    MAX_MSGLEN, MAX_RELIABLE_COMMANDS, PACKET_BACKUP, PACKET_MASK,
};
use mp_engine_qcommon::qcommon::svc_ops_e::svc_ops_e;
use mp_engine_qcommon::vm_fns::VM_ArgPtrWord;
use mp_host_interface::engine_host::EngineHost;
use mp_qshared::common::mp::cgame::refdef_t::MAX_MAP_AREA_BYTES;
use mp_qshared::common::mp::game::g_public::{
    SVF_BOT, SVF_BROADCAST, SVF_NOCLIENT, SVF_NOTSINGLECLIENT, SVF_PORTAL, SVF_SINGLECLIENT,
};
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::msg_t::msg_t;
use mp_qshared::common::mp::qcommon::netadrtype_t::netadrtype_t;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::errorParm_t;
use mp_qshared::shared::limits::{GENTITYNUM_BITS, SNAPFLAG_NOT_ACTIVE, SNAPFLAG_RATE_DELAYED};
use mp_qshared::shared::q_math::{
    _VectorAdd, _VectorCopy, _VectorScale, _VectorSubtract, VectorLength, VectorLengthSquared,
};
use mp_qshared::shared::{qboolean, qfalse, qtrue, vec3_t, MAX_GENTITIES};

use crate::server::client_s::client_t;
use crate::server::client_snapshot_t::clientSnapshot_t;
use crate::server::client_state_t::clientState_t;
use crate::server::server_state_t::serverState_t;
use crate::server::snapshot_entity_numbers_t::{snapshotEntityNumbers_t, MAX_SNAPSHOT_ENTITIES};
use crate::server::sv_entity_s::svEntity_t;
use crate::sv_game::{SV_GameClientNum, SV_GentityNum, SV_SvEntityForGentity};
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
pub fn SV_SendClientMessages(view: &mut EngineHostView, sv: &mut Server) {
    // send a message to each connected client
    let max_clients = view.common.cvar(view.common.sv_maxclients).integer;
    for i in 0..max_clients {
        let c = &mut sv.svs.clients[i as usize] as *mut client_t;
        unsafe {
            if (*c).state as i32 == 0 {
                continue; // not connected
            }

            // A replay replica has no socket; skip its outbound send entirely.
            // While it is a live client, auto-ack its reliable queue (the
            // recorded human acked every command within a round trip; the
            // backlog otherwise drains as extra BOTLIB_GET_CONSOLE_MESSAGE
            // calls when a bot reuses the slot — frame-5964 referee catch).
            // Not during CS_ZOMBIE: drop-time broadcasts can never be acked
            // by the leaving client, and the primary's successor bot drains
            // exactly those.
            if crate::sv_referee::ref_is_replica(sv, i) {
                if (*c).state as c_int >= clientState_t::CS_CONNECTED as c_int {
                    (*c).reliableAcknowledge = (*c).reliableSequence;
                }
                continue;
            }

            if sv.svs.time < (*c).nextSnapshotTime {
                continue; // not time yet
            }

            // send additional message fragments if the last message
            // was too large to send at once
            if (*c).netchan.unsentFragments != 0 {
                (*c).nextSnapshotTime = sv.svs.time
                    + SV_RateMsec(
                        view,
                        sv,
                        c,
                        (*c).netchan.unsentLength - (*c).netchan.unsentFragmentStart,
                    );
                SV_Netchan_TransmitNextFragment(view, &mut (*c).netchan);
                continue;
            }

            // generate and send a new message
            SV_SendClientSnapshot(view, sv, c);
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
    view: &mut EngineHostView,
    _sv: &mut Server,
    client: *mut client_t,
    mut messageSize: c_int,
) -> c_int {
    unsafe {
        // individual messages will never be larger than fragment size
        if messageSize > 1500 {
            messageSize = 1500;
        }
        let mut rate = (*client).rate;
        if view.common.cvar(view.common.sv_maxRate).integer != 0 {
            if view.common.cvar(view.common.sv_maxRate).integer < 1000 {
                Cvar_Set(view, "sv_MaxRate", "1000");
            }
            if view.common.cvar(view.common.sv_maxRate).integer < rate {
                rate = view.common.cvar(view.common.sv_maxRate).integer;
            }
        }
        (messageSize + HEADER_RATE_BYTES) * 1000 / rate
    }
}

/// Raven `SV_UpdateServerCommandsToClient` — (re)send all server commands the
/// client hasn't acknowledged yet.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:225-235`
pub fn SV_UpdateServerCommandsToClient(
    common: &mut Common,
    client: *mut client_t,
    msg: *mut msg_t,
) {
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
    view: &mut EngineHostView,
    sv: &mut Server,
    msg: *mut msg_t,
    client: *mut client_t,
) {
    unsafe {
        // A replay replica has no socket; suppress every transmit to it.
        let slot = (client as *const u8).offset_from(sv.svs.clients.as_ptr() as *const u8) as isize
            / core::mem::size_of::<client_t>() as isize;
        if crate::sv_referee::ref_is_replica(sv, slot as c_int) {
            return;
        }

        // MW - my attempt to fix illegible server message errors caused by
        // packet fragmentation of initial snapshot.
        while (*client).state as c_int != 0 && (*client).netchan.unsentFragments != 0 {
            // send additional message fragments if the last message
            // was too large to send at once
            com_printf(
                view.common,
                &format!(
                    "[ISM]SV_SendClientGameState() [1] for {}, writing out old fragments\n",
                    (*client).name
                ),
            );
            SV_Netchan_TransmitNextFragment(view, &mut (*client).netchan);
        }

        // record information about the message
        let idx = ((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize;
        (*client).frames[idx].messageSize = (*msg).cursize;
        (*client).frames[idx].messageSent = sv.svs.time;
        (*client).frames[idx].messageAcked = -1;

        // Engine-referee wire tap (`ref_snaps`): capture the logical message
        // bytes exactly as the client parser will consume them (pre-netchan).
        crate::sv_referee::ref_tap_client_message(view, sv, client, msg);

        // send the datagram
        SV_Netchan_Transmit(view, client, msg);

        // set nextSnapshotTime based on rate and requested number of updates

        // local clients get snapshots every frame
        if (*client).netchan.remoteAddress.r#type == netadrtype_t::NA_LOOPBACK
            || view.is_lan_address(&(*client).netchan.remoteAddress)
        {
            (*client).nextSnapshotTime = sv.svs.time - 1;
            return;
        }

        // normal rate / snapshotMsec calculation
        let mut rateMsec = SV_RateMsec(view, sv, client, (*msg).cursize);

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
            if (&(*client).downloadName).is_empty() && (*client).nextSnapshotTime < sv.svs.time + 1000 {
                (*client).nextSnapshotTime = sv.svs.time + 1000;
            }
        }
    }
}

/// Raven `SV_SendClientSnapshot` — also called by `SV_FinalMessage`.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:719-798`
pub fn SV_SendClientSnapshot(view: &mut EngineHostView, sv: &mut Server, client: *mut client_t) {
    unsafe {
        // A replay replica has no socket; suppress its snapshot send.
        let slot = (client as *const u8).offset_from(sv.svs.clients.as_ptr() as *const u8) as isize
            / core::mem::size_of::<client_t>() as isize;
        if crate::sv_referee::ref_is_replica(sv, slot as c_int) {
            return;
        }

        let mut msg_buf = [0u8; MAX_MSGLEN as usize];
        let mut msg: msg_t = core::mem::zeroed();

        if (*client).sentGamedir == qfalse {
            // rww - if this is the case then make sure there is an svc_setgame
            // sent before this snap
            mp_engine_qcommon::msg::MSG_Init(
                view,
                &mut msg,
                msg_buf.as_mut_ptr(),
                msg_buf.len() as c_int,
            );

            // have to include this for each message.
            mp_engine_qcommon::msg::MSG_WriteLong(
                view.common,
                &mut msg,
                (*client).lastClientCommand,
            );

            mp_engine_qcommon::msg::MSG_WriteByte(
                view.common,
                &mut msg,
                svc_ops_e::svc_setgame as c_int,
            );

            let fs_gamedir = view.common.cvar(view.common.fs_gamedirvar).string.clone();
            for b in fs_gamedir.bytes() {
                mp_engine_qcommon::msg::MSG_WriteByte(view.common, &mut msg, b as c_int);
            }
            mp_engine_qcommon::msg::MSG_WriteByte(view.common, &mut msg, 0);

            // MW - my attempt to fix illegible server message errors caused by
            // packet fragmentation of initial snapshot. rww - reusing this here
            while (*client).state as c_int != 0 && (*client).netchan.unsentFragments != 0 {
                com_printf(
                    view.common,
                    &format!(
                        "[ISM]SV_SendClientGameState() [1] for {}, writing out old fragments\n",
                        (*client).name
                    ),
                );
                SV_Netchan_TransmitNextFragment(view, &mut (*client).netchan);
            }

            // record information about the message
            let idx = ((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize;
            (*client).frames[idx].messageSize = msg.cursize;
            (*client).frames[idx].messageSent = sv.svs.time;
            (*client).frames[idx].messageAcked = -1;

            // send the datagram
            SV_Netchan_Transmit(view, client, &mut msg);

            (*client).sentGamedir = qtrue;
        }

        // build the snapshot
        SV_BuildClientSnapshot(view, sv, client);

        // bots need to have their snapshots build, but the query them directly
        // without needing to be sent
        if !(*client).gentity.is_null() && (*(*client).gentity).r.svFlags & SVF_BOT != 0 {
            return;
        }

        mp_engine_qcommon::msg::MSG_Init(
            view,
            &mut msg,
            msg_buf.as_mut_ptr(),
            msg_buf.len() as c_int,
        );
        msg.allowoverflow = qtrue;

        // NOTE, MRE: all server->client messages now acknowledge
        // let the client know which reliable clientCommands we have received
        mp_engine_qcommon::msg::MSG_WriteLong(view.common, &mut msg, (*client).lastClientCommand);

        // (re)send any reliable server commands
        SV_UpdateServerCommandsToClient(view.common, client, &mut msg);

        // send over all the relevant entityState_t and the playerState_t
        SV_WriteSnapshotToClient(view, sv, client, &mut msg);

        // Add any download data if the client is downloading
        crate::sv_client::SV_WriteDownloadToClient(view, sv, client, &mut msg);

        // check for overflow
        if msg.overflowed != qfalse {
            com_printf(
                view.common,
                &format!(
                    "WARNING: msg overflowed for {}\n",
                    (*client).name
                ),
            );
            mp_engine_qcommon::msg::MSG_Clear(&mut msg);
        }

        SV_SendMessageToClient(view, sv, &mut msg, client);
    }
}

/// Raven `SV_EmitPacketEntities` — writes a delta update of an `entityState_t`
/// list to the message.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:36-92`
fn SV_EmitPacketEntities(
    common: &mut Common,
    sv: &mut Server,
    from: *mut clientSnapshot_t,
    to: *mut clientSnapshot_t,
    msg: *mut msg_t,
) {
    unsafe {
        // generate the delta update
        let from_num_entities = if from.is_null() {
            0
        } else {
            (*from).num_entities
        };

        let mut newent: *mut entityState_t = core::ptr::null_mut();
        let mut oldent: *mut entityState_t = core::ptr::null_mut();
        let mut newindex = 0;
        let mut oldindex = 0;
        while newindex < (*to).num_entities || oldindex < from_num_entities {
            let newnum;
            if newindex >= (*to).num_entities {
                newnum = 9999;
            } else {
                newent = sv.svs.snapshotEntities.offset(
                    (((*to).first_entity + newindex) % sv.svs.numSnapshotEntities) as isize,
                );
                newnum = (*newent).number;
            }

            let oldnum;
            if oldindex >= from_num_entities {
                oldnum = 9999;
            } else {
                oldent = sv.svs.snapshotEntities.offset(
                    (((*from).first_entity + oldindex) % sv.svs.numSnapshotEntities) as isize,
                );
                oldnum = (*oldent).number;
            }

            if newnum == oldnum {
                // delta update from old position
                // because the force parm is qfalse, this will not result
                // in any bytes being emitted if the entity has not changed at all
                MSG_WriteDeltaEntity(common, msg, oldent, newent, qfalse);
                oldindex += 1;
                newindex += 1;
                continue;
            }

            if newnum < oldnum {
                // this is a new entity, send it from the baseline
                MSG_WriteDeltaEntity(
                    common,
                    msg,
                    &mut sv.sv.svEntities[newnum as usize].baseline,
                    newent,
                    qtrue,
                );
                newindex += 1;
                continue;
            }

            if newnum > oldnum {
                // the old entity isn't present in the new message
                MSG_WriteDeltaEntity(common, msg, oldent, core::ptr::null_mut(), qtrue);
                oldindex += 1;
                continue;
            }
        }

        MSG_WriteBits(common, msg, MAX_GENTITIES as c_int - 1, GENTITYNUM_BITS);
        // end of packetentities
    }
}

/// Raven `SV_WriteSnapshotToClient`. `_ONEBIT_COMBO` is not defined for this
/// build, so the plain `MSG_WriteDeltaPlayerstate` overloads are used. `cm`/`rm`/
/// `host` are threaded for the shared snapshot-send signature but unused here.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:100-208`
pub fn SV_WriteSnapshotToClient(
    view: &mut EngineHostView,
    sv: &mut Server,
    client: *mut client_t,
    msg: *mut msg_t,
) {
    // This function reaches the host only for `common`-tier services (MSG/Com);
    // reborrow it once from the view (no slot cast needed).
    let common = &mut *view.common;
    unsafe {
        // this is the snapshot we are creating
        let frame = &mut (*client).frames
            [((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize]
            as *mut clientSnapshot_t;

        // try to use a previous frame as the source for delta compressing the snapshot
        let oldframe: *mut clientSnapshot_t;
        let lastframe: c_int;
        if (*client).deltaMessage <= 0 || (*client).state != clientState_t::CS_ACTIVE {
            // client is asking for a retransmit
            oldframe = core::ptr::null_mut();
            lastframe = 0;
        } else if (*client).netchan.outgoingSequence - (*client).deltaMessage
            >= (PACKET_BACKUP as c_int - 3)
        {
            // client hasn't gotten a good message through in a long time
            Com_DPrintf(
                common,
                &format!(
                    "{}: Delta request from out of date packet.\n",
                    (*client).name
                ),
            );
            oldframe = core::ptr::null_mut();
            lastframe = 0;
        } else {
            // we have a valid snapshot to delta from
            let of = &mut (*client).frames[((*client).deltaMessage & PACKET_MASK as c_int) as usize]
                as *mut clientSnapshot_t;
            // the snapshot's entities may still have rolled off the buffer, though
            if (*of).first_entity <= sv.svs.nextSnapshotEntities - sv.svs.numSnapshotEntities {
                Com_DPrintf(
                    common,
                    &format!(
                        "{}: Delta request from out of date entities.\n",
                        (*client).name
                    ),
                );
                oldframe = core::ptr::null_mut();
                lastframe = 0;
            } else {
                oldframe = of;
                lastframe = (*client).netchan.outgoingSequence - (*client).deltaMessage;
            }
        }

        MSG_WriteByte(common, msg, svc_ops_e::svc_snapshot as c_int);

        // send over the current server time so the client can drift
        // its view of time to try to match
        MSG_WriteLong(common, msg, sv.svs.time);

        // what we are delta'ing from
        MSG_WriteByte(common, msg, lastframe);

        let mut snapFlags = sv.svs.snapFlagServerBit;
        if (*client).rateDelayed != qfalse {
            snapFlags |= SNAPFLAG_RATE_DELAYED;
        }
        if (*client).state != clientState_t::CS_ACTIVE {
            snapFlags |= SNAPFLAG_NOT_ACTIVE;
        }

        MSG_WriteByte(common, msg, snapFlags);

        // send over the areabits
        MSG_WriteByte(common, msg, (*frame).areabytes);
        MSG_WriteData(
            common,
            msg,
            (*frame).areabits.as_ptr() as *const (),
            (*frame).areabytes,
        );

        // delta encode the playerstate
        if !oldframe.is_null() {
            MSG_WriteDeltaPlayerstate(common, msg, &mut (*oldframe).ps, &mut (*frame).ps, qfalse);
            if (*frame).ps.m_iVehicleNum != 0 {
                // then write the vehicle's playerstate too
                if (*oldframe).ps.m_iVehicleNum == 0 {
                    // if last frame didn't have vehicle, then the old vps isn't
                    // gonna delta properly
                    MSG_WriteDeltaPlayerstate(
                        common,
                        msg,
                        core::ptr::null_mut(),
                        &mut (*frame).vps,
                        qtrue,
                    );
                } else {
                    MSG_WriteDeltaPlayerstate(
                        common,
                        msg,
                        &mut (*oldframe).vps,
                        &mut (*frame).vps,
                        qtrue,
                    );
                }
            }
        } else {
            MSG_WriteDeltaPlayerstate(common, msg, core::ptr::null_mut(), &mut (*frame).ps, qfalse);
            if (*frame).ps.m_iVehicleNum != 0 {
                // then write the vehicle's playerstate too
                MSG_WriteDeltaPlayerstate(
                    common,
                    msg,
                    core::ptr::null_mut(),
                    &mut (*frame).vps,
                    qtrue,
                );
            }
        }

        // delta encode the entities
        SV_EmitPacketEntities(common, sv, oldframe, frame, msg);

        // padding for rate debugging
        if common.cvar(common.sv_padPackets).integer != 0 {
            for _ in 0..common.cvar(common.sv_padPackets).integer {
                MSG_WriteByte(common, msg, svc_ops_e::svc_nop as c_int);
            }
        }
    }
}

/// Raven `g_svCullDist` — per-entity snapshot cull-distance override, `-1.0f`
/// (disabled). Held on [`Server`] (`sv.g_svCullDist`), not a file-scope global.
///
/// Raven `SV_AddEntitiesVisibleFromPoint` — gather all entities visible from
/// `origin` into `eNums`, recursing through portal entities. `_XBOX` is not
/// defined, so `clientpvs`/`bitvector` are the plain `byte*` branch.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:300-503`
fn SV_AddEntitiesVisibleFromPoint(
    common: &mut Common,
    cm: &mut CollisionWorld,
    sv: &mut Server,
    origin: vec3_t,
    frame: *mut clientSnapshot_t,
    eNums: *mut snapshotEntityNumbers_t,
    _portal: qboolean,
) {
    unsafe {
        // during an error shutdown message we may need to transmit
        // the shutdown message after the server has shutdown, so
        // specifically check for it
        if sv.sv.state == serverState_t::SS_DEAD {
            return;
        }

        let leafnum = CM_PointLeafnum(cm, origin);
        let clientarea = CM_LeafArea(cm, leafnum);
        let clientcluster = CM_LeafCluster(cm, leafnum);

        // calculate the visible areas
        (*frame).areabytes =
            CM_WriteAreaBits(common, cm, (*frame).areabits.as_mut_ptr(), clientarea);

        let clientpvs = CM_ClusterPVS(cm, clientcluster);

        for e in 0..sv.sv.num_entities {
            let ent = SV_GentityNum(sv, e);

            // never send entities that aren't linked in
            if (*ent).r.linked == qfalse {
                continue;
            }

            if (*ent).s.eFlags & EF_PERMANENT != 0 {
                // he's permanent, so don't send him down!
                continue;
            }

            if (*ent).s.number != e {
                Com_DPrintf(common, "FIXING ENT->S.NUMBER!!!\n");
                (*ent).s.number = e;
            }

            // entities can be flagged to explicitly not be sent to the client
            if (*ent).r.svFlags & SVF_NOCLIENT != 0 {
                continue;
            }

            // entities can be flagged to be sent to only one client
            if (*ent).r.svFlags & SVF_SINGLECLIENT != 0 {
                if (*ent).r.singleClient != (*frame).ps.clientNum {
                    continue;
                }
            }
            // entities can be flagged to be sent to everyone but one client
            if (*ent).r.svFlags & SVF_NOTSINGLECLIENT != 0 {
                if (*ent).r.singleClient == (*frame).ps.clientNum {
                    continue;
                }
            }

            let svEnt = SV_SvEntityForGentity(sv, ent);

            // don't double add an entity through portals
            if (*svEnt).snapshotCounter == sv.sv.snapshotCounter {
                continue;
            }

            let client_num = (*frame).ps.clientNum;
            // broadcast entities are always sent, and so is the main player so
            // we don't see noclip weirdness
            if (*ent).r.svFlags & SVF_BROADCAST != 0
                || (e == client_num)
                || ((*ent).r.broadcastClients[(client_num / 32) as usize]
                    & (1 << (client_num % 32))
                    != 0)
            {
                SV_AddEntToSnapshot(sv, svEnt, ent, eNums);
                continue;
            }

            if (*ent).s.isPortalEnt != qfalse {
                // rww - portal entities are always sent as well
                SV_AddEntToSnapshot(sv, svEnt, ent, eNums);
                continue;
            }

            if common.com_RMG.is_some() && common.cvar(common.com_RMG).integer != 0 {
                let mut difference: vec3_t = [0.0; 3];
                _VectorAdd((*ent).r.absmax, (*ent).r.absmin, &mut difference);
                _VectorScale(difference, 0.5, &mut difference);
                _VectorSubtract(origin, difference, &mut difference);
                let length = VectorLength(difference);

                // calculate the diameter
                _VectorSubtract((*ent).r.absmax, (*ent).r.absmin, &mut difference);
                let radius = VectorLength(difference);
                if length - radius < 5000.0 {
                    // more of a diameter check
                    SV_AddEntToSnapshot(sv, svEnt, ent, eNums);
                }
            } else {
                // ignore if not touching a PV leaf
                // check area
                if CM_AreasConnected(common, cm, clientarea, (*svEnt).areanum) == qfalse {
                    // doors can legally straddle two areas, so
                    // we may need to check another one
                    if CM_AreasConnected(common, cm, clientarea, (*svEnt).areanum2) == qfalse {
                        continue; // blocked by a door
                    }
                }

                let bitvector = clientpvs;

                // check individual leafs
                if (*svEnt).numClusters == 0 {
                    continue;
                }
                let mut l = 0;
                let mut i = 0;
                while i < (*svEnt).numClusters {
                    l = (*svEnt).clusternums[i as usize];
                    if *bitvector.offset((l >> 3) as isize) & (1 << (l & 7)) != 0 {
                        break;
                    }
                    i += 1;
                }

                // if we haven't found it to be visible,
                // check overflow clusters that couldn't be stored
                if i == (*svEnt).numClusters {
                    if (*svEnt).lastCluster != 0 {
                        while l <= (*svEnt).lastCluster {
                            if *bitvector.offset((l >> 3) as isize) & (1 << (l & 7)) != 0 {
                                break;
                            }
                            l += 1;
                        }
                        if l == (*svEnt).lastCluster {
                            continue; // not visible
                        }
                    } else {
                        continue;
                    }
                }

                if sv.g_svCullDist != -1.0 {
                    // do a distance cull check
                    let mut difference: vec3_t = [0.0; 3];
                    _VectorAdd((*ent).r.absmax, (*ent).r.absmin, &mut difference);
                    _VectorScale(difference, 0.5, &mut difference);
                    _VectorSubtract(origin, difference, &mut difference);
                    let length = VectorLength(difference);

                    // calculate the diameter
                    _VectorSubtract((*ent).r.absmax, (*ent).r.absmin, &mut difference);
                    let radius = VectorLength(difference);
                    if length - radius >= sv.g_svCullDist {
                        // then don't add it
                        continue;
                    }
                }

                // add it
                SV_AddEntToSnapshot(sv, svEnt, ent, eNums);

                // if its a portal entity, add everything visible from its camera position
                if (*ent).r.svFlags & SVF_PORTAL != 0 {
                    if (*ent).s.generic1 != 0 {
                        let mut dir: vec3_t = [0.0; 3];
                        _VectorSubtract((*ent).s.origin, origin, &mut dir);
                        if VectorLengthSquared(dir)
                            > (*ent).s.generic1 as f32 * (*ent).s.generic1 as f32
                        {
                            continue;
                        }
                    }
                    SV_AddEntitiesVisibleFromPoint(
                        common,
                        cm,
                        sv,
                        (*ent).s.origin2,
                        frame,
                        eNums,
                        qtrue,
                    );
                }
            }
        }
    }
}

/// Raven `SV_BuildClientSnapshot` — decide which entities are visible to the
/// client and copy off the playerstate and areabits. Handles multiple recursive
/// portals. `rm`/`host` are threaded for the shared signature but unused here.
///
/// Source: `oracle/codemp/server/sv_snapshot.cpp:507-620`
pub fn SV_BuildClientSnapshot(view: &mut EngineHostView, sv: &mut Server, client: *mut client_t) {
    // Reaches the host only for `common`/`cm`-tier services; reborrow both once
    // from the view (disjoint fields, no slot cast needed).
    let common = &mut *view.common;
    let cm = &mut *view.cm;
    unsafe {
        let mut org: vec3_t = [0.0; 3];
        let mut entityNumbers: snapshotEntityNumbers_t = core::mem::zeroed();

        // bump the counter used to prevent double adding
        sv.sv.snapshotCounter += 1;

        // this is the frame we are creating
        let frame = &mut (*client).frames
            [((*client).netchan.outgoingSequence & PACKET_MASK as c_int) as usize]
            as *mut clientSnapshot_t;

        // clear everything in this snapshot
        entityNumbers.numSnapshotEntities = 0;
        Com_Memset(
            (*frame).areabits.as_mut_ptr() as *mut (),
            0,
            core::mem::size_of_val(&(*frame).areabits),
        );

        (*frame).num_entities = 0;

        let clent = (*client).gentity;
        if clent.is_null() || (*client).state == clientState_t::CS_ZOMBIE {
            return;
        }

        // grab the current playerState_t
        let client_index = ((client as *mut u8).offset_from(sv.svs.clients.as_mut_ptr() as *mut u8) as isize
            / core::mem::size_of::<client_t>() as isize) as c_int;
        let ps = SV_GameClientNum(sv, client_index);
        (*frame).ps = *ps;

        if (*ps).m_iVehicleNum != 0 {
            // get the vehicle's playerstate too then
            let veh = SV_GentityNum(sv, (*ps).m_iVehicleNum);

            if !veh.is_null() && !(*veh).playerState.is_null() {
                // Raven's `VM_ArgPtr((int)veh->playerState)` — the `int` cast is
                // ILP32-era; on LP64 the dll-hosted module stores a full pointer
                // word here, so the AbiWord-widened twin resolves it untruncated.
                let vps = VM_ArgPtrWord(common, (*veh).playerState as isize) as *mut playerState_t;
                (*frame).vps = *vps;
            }
        }

        // never send client's own entity, because it can
        // be regenerated from the playerstate
        let clientNum = (*frame).ps.clientNum;
        if clientNum < 0 || clientNum >= MAX_GENTITIES as c_int {
            com_error(
                errorParm_t::ERR_DROP,
                "SV_SvEntityForGentity: bad gEnt".to_string(),
            );
        }
        let svEnt = &mut sv.sv.svEntities[clientNum as usize] as *mut svEntity_t;
        (*svEnt).snapshotCounter = sv.sv.snapshotCounter;

        // find the client's viewpoint
        _VectorCopy((*ps).origin, &mut org);
        org[2] += (*ps).viewheight as f32;

        // add all the entities directly visible to the eye, which
        // may include portal entities that merge other viewpoints
        SV_AddEntitiesVisibleFromPoint(common, cm, sv, org, frame, &mut entityNumbers, qfalse);

        // if there were portals visible, there may be out of order entities
        // in the list which will need to be resorted for the delta compression
        // to work correctly.  This also catches the error condition
        // of an entity being included twice.
        // Raven's `qsort` + `SV_QsortEntityNumbers` (ascending, `Com_Error` on a
        // duplicate) is preserved as an in-place sort plus an adjacency dup scan.
        let count = entityNumbers.numSnapshotEntities as usize;
        entityNumbers.snapshotEntities[..count].sort_unstable();
        let mut w = 1;
        while w < count {
            if entityNumbers.snapshotEntities[w] == entityNumbers.snapshotEntities[w - 1] {
                com_error(
                    errorParm_t::ERR_DROP,
                    "SV_QsortEntityStates: duplicated entity".to_string(),
                );
            }
            w += 1;
        }

        // now that all viewpoint's areabits have been OR'd together, invert
        // all of them to make it a mask vector, which is what the renderer wants
        for i in 0..(MAX_MAP_AREA_BYTES / 4) {
            let p = ((*frame).areabits.as_mut_ptr() as *mut c_int).add(i);
            *p ^= -1;
        }

        // copy the entity states out
        (*frame).num_entities = 0;
        (*frame).first_entity = sv.svs.nextSnapshotEntities;
        for i in 0..entityNumbers.numSnapshotEntities {
            let ent = SV_GentityNum(sv, entityNumbers.snapshotEntities[i as usize]);
            let state = sv
                .svs
                .snapshotEntities
                .offset((sv.svs.nextSnapshotEntities % sv.svs.numSnapshotEntities) as isize);
            *state = (*ent).s;
            sv.svs.nextSnapshotEntities += 1;
            // this should never hit, map should always be restarted first in SV_Frame
            if sv.svs.nextSnapshotEntities >= 0x7FFF_FFFE {
                com_error(
                    errorParm_t::ERR_FATAL,
                    "svs.nextSnapshotEntities wrapped".to_string(),
                );
            }
            (*frame).num_entities += 1;
        }
    }
}
