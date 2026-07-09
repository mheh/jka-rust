#![allow(non_camel_case_types, non_snake_case)]

use mp_engine_qcommon::qcommon::netadr_t::netadr_t;
use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::shared::qboolean;

use super::challenge_t::challenge_t;
use super::client_s::client_s;

/// `MAX_CHALLENGES` is made large to prevent a denial of service attack that
/// could cycle all of them out before legitimate users connected.
///
/// Source: `oracle/codemp/server/server.h:190`
pub const MAX_CHALLENGES: usize = 1024;

/// Raven `serverStatic_t`.
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:208-228`
#[repr(C)]
pub struct serverStatic_t {
    /// sv_init has completed
    pub initialized: qboolean,
    /// will be strictly increasing across level changes
    pub time: i32,
    /// ^= SNAPFLAG_SERVERCOUNT every SV_SpawnServer()
    pub snapFlagServerBit: i32,
    /// [sv_maxclients->integer];
    pub clients: *mut client_s,
    /// sv_maxclients->integer*PACKET_BACKUP*MAX_PACKET_ENTITIES
    pub numSnapshotEntities: i32,
    /// next snapshotEntities to use
    pub nextSnapshotEntities: i32,
    /// [numSnapshotEntities]
    pub snapshotEntities: *mut entityState_t,
    pub nextHeartbeatTime: i32,
    /// to prevent invalid IPs from connecting
    pub challenges: [challenge_t; MAX_CHALLENGES],
    /// for rcon return messages
    pub redirectAddress: netadr_t,
    /// for rcon return messages
    pub authorizeAddress: netadr_t,
}

const _: () = assert!(core::mem::size_of::<serverStatic_t>() == 41048);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, initialized) == 0);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, time) == 4);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, snapFlagServerBit) == 8);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, clients) == 16);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, numSnapshotEntities) == 24);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, nextSnapshotEntities) == 28);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, snapshotEntities) == 32);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, nextHeartbeatTime) == 40);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, challenges) == 44);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, redirectAddress) == 41004);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, authorizeAddress) == 41024);
