#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use sp_engine_qcommon::cm::cmodel_s::cmodel_s;
use sp_qshared::shared::MAX_CONFIGSTRINGS;

use super::server_state_t::serverState_t;
use super::sv_entity_s::svEntity_t;

/// Raven `MAX_MODELS` — models sent over the net as -8 bits.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1461`
pub const MAX_MODELS: usize = 256;

/// Raven `MAX_GENTITIES` — `1 << GENTITYNUM_BITS`.
///
/// Type definition source: `oracle/oracle/code/game/q_shared.h:1450-1451`
pub const MAX_GENTITIES: usize = 1024;

/// Raven `server_t`.
///
/// Raven: be careful, Jake's code uses the 'svEntities' field as a marker to
/// memset-this-far-only inside SV_InitSV()!!!!!
/// Type definition source: `oracle/oracle/code/server/server.h:48-72`
#[repr(C)]
pub struct server_t {
    pub state: serverState_t,
    /// changes each server start
    pub serverId: c_int,
    /// incremented for each snapshot built
    pub snapshotCounter: c_int,
    /// all entities are correct for this time // These 2 saved out
    pub time: c_int,
    /// <= 1000 / sv_frame->value //   during savegame.
    pub timeResidual: c_int,
    /// fraction of a msec accumulated
    pub timeResidualFraction: f32,
    /// when time > nextFrameTime, process world // this doesn't get used anywhere! -Ste
    pub nextFrameTime: c_int,
    pub models: [*mut cmodel_s; MAX_MODELS],
    pub configstrings: [*mut c_char; MAX_CONFIGSTRINGS],
    // be careful, Jake's code uses the 'svEntities' field as a marker to
    // memset-this-far-only inside SV_InitSV()!!!!!
    /// used during game VM init
    pub entityParsePoint: *mut c_char,

    pub mLocalSubBSPIndex: c_int,
    pub mLocalSubBSPModelOffset: c_int,
    pub mLocalSubBSPEntityParsePoint: *mut c_char,

    pub svEntities: [svEntity_t; MAX_GENTITIES],
}
const _: () = assert!(core::mem::size_of::<server_t>() == 397528);
const _: () = assert!(core::mem::offset_of!(server_t, state) == 0);
const _: () = assert!(core::mem::offset_of!(server_t, serverId) == 4);
const _: () = assert!(core::mem::offset_of!(server_t, snapshotCounter) == 8);
const _: () = assert!(core::mem::offset_of!(server_t, time) == 12);
const _: () = assert!(core::mem::offset_of!(server_t, timeResidual) == 16);
const _: () = assert!(core::mem::offset_of!(server_t, timeResidualFraction) == 20);
const _: () = assert!(core::mem::offset_of!(server_t, nextFrameTime) == 24);
const _: () = assert!(core::mem::offset_of!(server_t, models) == 32);
const _: () = assert!(core::mem::offset_of!(server_t, configstrings) == 2080);
const _: () = assert!(core::mem::offset_of!(server_t, entityParsePoint) == 12480);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPIndex) == 12488);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPModelOffset) == 12492);
const _: () = assert!(core::mem::offset_of!(server_t, mLocalSubBSPEntityParsePoint) == 12496);
const _: () = assert!(core::mem::offset_of!(server_t, svEntities) == 12504);
