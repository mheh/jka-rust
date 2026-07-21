#![allow(non_camel_case_types, non_snake_case)]

use mp_qshared::common::mp::qcommon::entity_state::entityState_t;
use mp_qshared::common::mp::qcommon::netadr_t::netadr_t;
use mp_qshared::shared::qboolean;

use super::challenge_t::challenge_t;
use super::client_s::client_s;

/// `MAX_CHALLENGES` is made large to prevent a denial of service attack that
/// could cycle all of them out before legitimate users connected.
///
/// Source: `oracle/codemp/server/server.h:190`
pub const MAX_CHALLENGES: usize = 1024;

/// Raven `AUTHORIZE_TIMEOUT` — milliseconds before a pending `challenge_t`
/// authorize-server round-trip is dropped.
///
/// Source: `oracle/codemp/server/server.h:192`
pub const AUTHORIZE_TIMEOUT: i32 = 5000;

/// Raven `serverStatic_t`.
///
/// (§D12 internal-only shape: `serverStatic_t` never crosses the DLL seam, so
/// `clients` is an owned `Vec<client_t>` and the old `#[repr(C)]` layout asserts
/// are dropped — the array that C `Z_Malloc`'d now owns its heap.)
///
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:208-228`
pub struct serverStatic_t {
    /// sv_init has completed
    pub initialized: qboolean,
    /// will be strictly increasing across level changes
    pub time: i32,
    /// ^= SNAPFLAG_SERVERCOUNT every SV_SpawnServer()
    pub snapFlagServerBit: i32,
    /// [sv_maxclients->integer];
    pub clients: Vec<client_s>,
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

