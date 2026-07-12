#![allow(non_camel_case_types, non_snake_case)]

//! SP `cg_public.h` snapshot definition.

use sp_qshared::common::sp::qcommon::{entityState_t, playerState_t};
use sp_qshared::common::sp::renderer::refdef_t::MAX_MAP_AREA_BYTES;

/// Raven `MAX_ENTITIES_IN_SNAPSHOT`.
///
/// Source: `oracle/code/cgame/cg_public.h:14`
pub const MAX_ENTITIES_IN_SNAPSHOT: usize = 512;

/// Raven `snapshot_t` (tag `snapshot_s`) — a complete snapshot of game state
/// at a serverTime.
///
/// Type definition source: `oracle/code/cgame/cg_public.h:24-47`
#[repr(C)]
pub struct snapshot_t {
    /// SNAPFLAG_RATE_DELAYED, SNAPFLAG_DROPPED_COMMANDS
    pub snapFlags: i32,
    pub ping: i32,

    /// server time the message is valid for (in msec)
    pub serverTime: i32,

    /// portalarea visibility bits
    pub areamask: [u8; MAX_MAP_AREA_BYTES],

    /// the next cmdNum the server is expecting
    /// client side prediction should start with this cmd
    pub cmdNum: i32,
    /// complete information about the current player at this time
    pub ps: playerState_t,

    /// all of the entities that need to be presented
    pub numEntities: i32,
    /// at the time of this snapshot
    pub entities: [entityState_t; MAX_ENTITIES_IN_SNAPSHOT],

    /// configstrings that have changed since the last
    pub numConfigstringChanges: i32,
    /// acknowledged snapshot_t (which is usually NOT the previous snapshot!)
    pub configstringNum: i32,

    /// text based server commands to execute when this
    pub numServerCommands: i32,
    /// snapshot becomes current
    pub serverCommandSequence: i32,
}

const _: () = assert!(core::mem::size_of::<snapshot_t>() == 144328);
const _: () = assert!(core::mem::offset_of!(snapshot_t, snapFlags) == 0);
const _: () = assert!(core::mem::offset_of!(snapshot_t, ping) == 4);
const _: () = assert!(core::mem::offset_of!(snapshot_t, serverTime) == 8);
const _: () = assert!(core::mem::offset_of!(snapshot_t, areamask) == 12);
const _: () = assert!(core::mem::offset_of!(snapshot_t, cmdNum) == 44);
const _: () = assert!(core::mem::offset_of!(snapshot_t, ps) == 48);
const _: () = assert!(core::mem::offset_of!(snapshot_t, numEntities) == 5040);
const _: () = assert!(core::mem::offset_of!(snapshot_t, entities) == 5044);
const _: () = assert!(core::mem::offset_of!(snapshot_t, numConfigstringChanges) == 144308);
const _: () = assert!(core::mem::offset_of!(snapshot_t, configstringNum) == 144312);
const _: () = assert!(core::mem::offset_of!(snapshot_t, numServerCommands) == 144316);
const _: () = assert!(core::mem::offset_of!(snapshot_t, serverCommandSequence) == 144320);
