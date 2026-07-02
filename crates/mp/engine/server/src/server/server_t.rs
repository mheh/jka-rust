#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::cm::cmodel_s::cmodel_s;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::{qboolean, MAX_CONFIGSTRINGS};

use super::server_state_t::serverState_t;
use super::sv_entity_s::svEntity_t;

/// Raven `MAX_MODELS` — models sent over the net as -12 bits.
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:2020`
pub const MAX_MODELS: usize = 512;

/// Raven `MAX_GENTITIES` — `1 << GENTITYNUM_BITS` (non-`_XBOX`, `GENTITYNUM_BITS` = 10).
///
/// Type definition source: `oracle/oracle/codemp/game/q_shared.h:1992-1996`
pub const MAX_GENTITIES: usize = 1024;

/// Raven `server_t`.
///
/// Raven: non-`_XBOX` variant (`_XBOX` undefined) is the one this codebase ports.
/// Type definition source: `oracle/oracle/codemp/qcommon/../server/server.h:53-88`
#[repr(C)]
pub struct server_t {
    pub state: serverState_t,
    /// if true, send configstring changes during SS_LOADING
    pub restarting: qboolean,
    /// changes each server start
    pub serverId: c_int,
    /// serverId before a map_restart
    pub restartedServerId: c_int,
    pub checksumFeed: c_int,
    /// incremented for each snapshot built
    pub snapshotCounter: c_int,
    /// <= 1000 / sv_frame->value
    pub timeResidual: c_int,
    /// when time > nextFrameTime, process world
    pub nextFrameTime: c_int,
    pub models: [*mut cmodel_s; MAX_MODELS],
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    pub svEntities: [svEntity_t; MAX_GENTITIES],

    /// used during game VM init
    pub entityParsePoint: *mut c_char,

    // the game virtual machine will update these on init and changes
    pub gentities: *mut sharedEntity_t,
    pub gentitySize: c_int,
    /// current number, <= MAX_GENTITIES
    pub num_entities: c_int,

    pub gameClients: *mut playerState_t,
    /// will be > sizeof(playerState_t) due to game private data
    pub gameClientSize: c_int,

    pub restartTime: c_int,

    // rwwRMG - added:
    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *mut c_char,

    pub mSharedMemory: *mut c_char,
}
const _: () = assert!(core::mem::size_of::<server_t>() == 664960);
const _: () = assert!(core::mem::offset_of!(server_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(server_t, restarting) == 4);
const _: () = assert!(core::mem::offset_of!(server_t, serverId) == 8);
const _: () = assert!(core::mem::offset_of!(server_t, restartedServerId) == 12);
const _: () = assert!(core::mem::offset_of!(server_t, checksumFeed) == 16);
const _: () = assert!(core::mem::offset_of!(server_t, snapshotCounter) == 20);
const _: () = assert!(core::mem::offset_of!(server_t, timeResidual) == 24);
const _: () = assert!(core::mem::offset_of!(server_t, nextFrameTime) == 28);
const _: () = assert!(core::mem::offset_of!(server_t, models) == 32);
const _: () = assert!(core::mem::offset_of!(server_t, configstrings) == 4128);
const _: () = assert!(core::mem::offset_of!(server_t, svEntities) == 17728);
const _: () = assert!(core::mem::offset_of!(server_t, entityParsePoint) == 664896);
const _: () = assert!(core::mem::offset_of!(server_t, gentities) == 664904);
const _: () = assert!(core::mem::offset_of!(server_t, gentitySize) == 664912);
const _: () = assert!(core::mem::offset_of!(server_t, num_entities) == 664916);
const _: () = assert!(core::mem::offset_of!(server_t, gameClients) == 664920);
const _: () = assert!(core::mem::offset_of!(server_t, gameClientSize) == 664928);
const _: () = assert!(core::mem::offset_of!(server_t, restartTime) == 664932);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPIndex) == 664936);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPModelOffset) == 664940);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPEntityParsePoint) == 664944);
const _: () = assert!(core::mem::offset_of!(server_t, mSharedMemory) == 664952);
