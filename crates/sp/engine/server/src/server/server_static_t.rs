#![allow(non_camel_case_types, non_snake_case)]

use sp_qshared::common::sp::qcommon::entity_state::entityState_t;
use sp_qshared::shared::qboolean;

use super::client_s::client_s;

/// Raven `serverStatic_t` — persistent server state, survives `SV_Init` re-runs.
///
/// Type definition source: `oracle/code/server/server.h:142-149`
#[repr(C)]
pub struct serverStatic_t {
    /// `sv_init` has completed
    pub initialized: qboolean,
    /// `[sv_maxclients->integer]`
    pub clients: *mut client_s,
    /// `sv_maxclients->integer*PACKET_BACKUP*MAX_PACKET_ENTITIES`
    pub numSnapshotEntities: i32,
    /// next `snapshotEntities` to use
    pub nextSnapshotEntities: i32,
    /// `[numSnapshotEntities]`
    pub snapshotEntities: *mut entityState_t,
    pub nextHeartbeatTime: i32,
}

const _: () = assert!(core::mem::size_of::<serverStatic_t>() == 40);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, initialized) == 0);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, clients) == 8);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, numSnapshotEntities) == 16);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, nextSnapshotEntities) == 20);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, snapshotEntities) == 24);
const _: () = assert!(core::mem::offset_of!(serverStatic_t, nextHeartbeatTime) == 32);
