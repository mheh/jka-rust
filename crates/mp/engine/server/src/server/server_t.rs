#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int};

use mp_engine_qcommon::cm::cmodel_s::cmodel_s;
use mp_qshared::common::mp::qcommon::player_state::playerState_t;
use mp_qshared::common::mp::qcommon::shared_entity_t::sharedEntity_t;
use mp_qshared::shared::limits::MAX_MODELS as MAX_MODELS_I32;
use mp_qshared::shared::{qboolean, MAX_GENTITIES};

use super::server_state_t::serverState_t;
use super::sv_entity_s::svEntity_t;

/// `usize`-typed dual of the canonical `mp_qshared::shared::limits::MAX_MODELS`
/// (`c_int`), for array sizing.
///
/// Source: `oracle/codemp/game/q_shared.h:2020`
pub const MAX_MODELS: usize = MAX_MODELS_I32 as usize;

// `MAX_GENTITIES` (`q_shared.h:1992-1996`) imported from its canonical Tier-0
// home in `mp_qshared::shared` (relocation noted there; this dedupes the copy
// the mechanical type-port left in `mp_engine_server`).

/// Raven `server_t`.
///
/// Raven: non-`_XBOX` variant (`_XBOX` undefined) is the one this codebase ports.
///
/// (Internal-only shape, like `client_t`: `server_t` never crosses the DLL seam
/// — the game module reaches `configstrings` only through the bounded
/// `SV_GetConfigstring`/`SV_SetConfigstring` trap copies — so `configstrings`
/// is an owned `Vec<String>` and the old `#[repr(C)]` layout asserts are
/// dropped. There is exactly one instance (`Engine.sv.sv`), reached by
/// reference, so no size/stride math depends on the layout.)
/// Type definition source: `oracle/codemp/qcommon/../server/server.h:53-88`
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
    /// Raven's `char *configstrings[MAX_CONFIGSTRINGS]` — owned strings (each
    /// slot was a `CopyString`'d heap block). Length is exactly
    /// `MAX_CONFIGSTRINGS`; an empty string `""` is Raven's null slot
    /// (`SV_GetConfigstring` returns `""` for null, `SV_SetConfigstring`'s
    /// dedupe compares equal). Seated in `Engine::new`; reset in `SV_InitSV`.
    pub configstrings: Vec<String>,
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
